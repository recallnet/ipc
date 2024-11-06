use fendermint_actor_machine::MachineActor;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::actor_dispatch;
use fil_actors_runtime::{actor_error, ActorError};
use getrandom::register_custom_getrandom;
use rusqlite::{types::Value, Connection, OpenFlags};
use sqlite_vfs::register;

use crate::errors::Error;
use crate::{state::State, vfs, ExecuteParams, ExecuteReturn, Method, QueryParams, QueryReturn};
use crate::{SQLITE_ACTOR_NAME, SQLITE_PAGE_SIZE};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

// NOTE (sander): This is a custom randomness function for dependencies that use the
// getrandom crate. I haven't seen this method actually be called in my testing, but
// w/o it, the actor won't compile. The VFS has its own randomness method.
pub fn randomness(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // Just return zeros.
    let data = (0..buf.len()).map(|_| 0_u128 as u8).collect::<Vec<_>>();
    buf.copy_from_slice(&data);
    Ok(())
}
register_custom_getrandom!(randomness);

/// Sqlite Actor
pub struct Actor;

impl Actor {
    pub fn execute<RT>(rt: &RT, params: ExecuteParams) -> Result<ExecuteReturn, ActorError>
    where
        RT: Runtime,
        RT::Blockstore: Clone,
    {
        rt.validate_immediate_caller_accept_any()?;
        let mut conn = new_connection(rt)?;

        // Always run statements within a transaction
        // See https://medium.com/@JasonWyatt/squeezing-performance-from-sqlite-insertions-971aff98eef2
        let tx = conn.transaction().map_err(Error::from)?;
        let mut effected_rows: usize = 0;
        for stmt in params.stmts {
            effected_rows += tx.execute(stmt.as_str(), []).map_err(Error::from)?;
        }
        tx.commit().map_err(Error::from)?;

        Ok(ExecuteReturn { effected_rows })
    }

    pub fn query<RT>(rt: &RT, params: QueryParams) -> Result<QueryReturn, ActorError>
    where
        RT: Runtime,
        RT::Blockstore: Clone,
    {
        rt.validate_immediate_caller_accept_any()?;
        let conn = new_connection(rt)?;

        let mut stmt = conn.prepare(params.stmt.as_str()).map_err(Error::from)?;
        let cols: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();
        let num_cols = cols.len();
        let mut res: Vec<Vec<Value>> = vec![];
        let mut rows = stmt.query([]).map_err(Error::from)?;
        while let Some(r) = rows.next().map_err(|ref e| Error::from(e.to_string()))? {
            let mut row: Vec<Value> = vec![Value::Null; num_cols];
            #[allow(clippy::needless_range_loop)]
            for c in 0..num_cols {
                row[c] = r.get::<_, Value>(c).map_err(Error::from)?;
            }
            res.push(row);
        }

        Ok(QueryReturn { cols, rows: res })
    }
}

impl MachineActor for Actor {
    type State = crate::state::State;
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        SQLITE_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        Init => init,
        GetAddress => get_address,
        GetMetadata => get_metadata,
        Execute => execute,
        Query => query,
        _ => fallback,
    }
}

fn new_connection<RT>(rt: &RT) -> Result<Connection, ActorError>
where
    RT: Runtime,
    RT::Blockstore: Clone,
{
    let st: State = rt.state()?;
    let is_new = st.db.page_tree.is_empty();

    register("vfs", vfs::PagesVfs::<SQLITE_PAGE_SIZE, RT>::new(rt), true).map_err(Error::from)?;
    let conn = Connection::open_with_flags_and_vfs(
        "main.db",
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        "vfs",
    )
    .map_err(Error::from)?;

    if is_new {
        conn.execute(
            format!("PRAGMA page_size = {};", SQLITE_PAGE_SIZE).as_str(),
            [],
        )
        .map_err(Error::from)?;
    }
    let page_size: usize = conn
        .query_row("PRAGMA page_size", [], |row| row.get(0))
        .map_err(Error::from)?;
    if page_size != SQLITE_PAGE_SIZE {
        return Err(ActorError::illegal_state(
            format!("db must use page_size = {}", SQLITE_PAGE_SIZE).to_string(),
        ));
    }

    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode = MEMORY", [], |row| row.get(0))
        .map_err(Error::from)?;
    if journal_mode != "memory" {
        return Err(ActorError::illegal_state(
            "db must use journal_mode = MEMORY".to_string(),
        ));
    }

    Ok(conn)
}
