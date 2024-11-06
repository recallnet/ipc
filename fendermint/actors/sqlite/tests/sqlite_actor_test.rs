use std::collections::HashMap;

use fendermint_actor_machine::ConstructorParams;
use fendermint_actor_sqlite::actor::Actor as SqliteActor;
use fendermint_actor_sqlite::{
    ExecuteParams, ExecuteReturn, Method, QueryParams, QueryReturn, State, DB, SQLITE_BUCKET_SIZE,
    SQLITE_PAGE_SIZE,
};
use fil_actors_evm_shared::address::EthAddress;
use fil_actors_runtime::{test_utils::*, INIT_ACTOR_ADDR};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::address::Address;
use fvm_shared::error::ExitCode;
use fvm_shared::MethodNum;
use serde_ipld_dagcbor::from_slice;
use serial_test::serial;

fn replace_state_with_test_db(rt: &MockRuntime) {
    let db = DB::new(
        &rt.store,
        include_bytes!("../testdata/test.db"),
        SQLITE_PAGE_SIZE,
        SQLITE_BUCKET_SIZE,
    )
    .unwrap();
    let mut state: State = rt.get_state();
    state.db = db;
    rt.replace_state(&state);
}

/// returns an empty db by default
fn construct_and_verify(owner: Address) -> MockRuntime {
    let rt = MockRuntime {
        receiver: Address::new_id(10),
        ..Default::default()
    };
    rt.set_caller(*SYSTEM_ACTOR_CODE_ID, INIT_ACTOR_ADDR);
    rt.expect_validate_caller_addr(vec![INIT_ACTOR_ADDR]);
    let metadata = HashMap::new();

    let actor_construction = rt
        .call::<SqliteActor>(
            Method::Constructor as u64,
            IpldBlock::serialize_cbor(&ConstructorParams { owner, metadata }).unwrap(),
        )
        .unwrap();
    expect_empty(actor_construction);
    rt.verify();
    rt.reset();
    rt
}

fn setup_test(db_with_data: bool) -> MockRuntime {
    let id_addr = Address::new_id(119);
    let eth_addr = EthAddress(hex_literal::hex!(
        "CAFEB0BA00000000000000000000000000000000"
    ));
    let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

    let rt = construct_and_verify(f4_eth_addr);
    rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
    rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
    rt.set_origin(id_addr);
    if db_with_data {
        replace_state_with_test_db(&rt);
    }
    rt
}

#[test]
#[serial]
fn execution() {
    fn execute(
        db_with_data: bool,
        stmts: Vec<&str>,
        exit_code: ExitCode,
        effected_rows: usize,
        need_beacon_randomness: bool,
    ) {
        let rt = setup_test(db_with_data);

        rt.expect_validate_caller_any();
        // NOTE (sander): This is a hack. Try expecting randomness to be called
        // for all tests and you'll see that, for some reason probably related to
        // me removing the Sync requirement on the VFS, the tests share some aspect
        // of the underlying SQLite connection, even though each test has its own
        // VFS instance. TL;DR, the journal header is written only once during the
        // first test run, which calls the random method on VFS. This is also the
        // reason why all tests are run serial.
        // I'm not 100% sure, but this shouldn't be an issue when compiling to WASM.
        if need_beacon_randomness {
            let epoch = 1234;
            rt.set_epoch(epoch);
            rt.expect_get_randomness_from_beacon(
                fil_actors_runtime::runtime::DomainSeparationTag::EvmPrevRandao,
                epoch,
                Vec::from(*b"prevrandao"),
                [0xff; 32],
            );
        }

        let params = IpldBlock::serialize_cbor(&ExecuteParams {
            stmts: stmts.iter().map(|s| s.to_string()).collect(),
        })
        .unwrap();

        if exit_code.is_success() {
            let block: IpldBlock = rt
                .call::<SqliteActor>(Method::Execute as MethodNum, params)
                .unwrap()
                .unwrap();
            let ret = from_slice::<ExecuteReturn>(block.data.as_slice()).unwrap();
            assert_eq!(ret.effected_rows, effected_rows);
        } else {
            expect_abort(
                exit_code,
                rt.call::<SqliteActor>(Method::Execute as MethodNum, params),
            )
        }
        rt.verify();
    }

    // Create and insert on existing db
    execute(
        true,
        vec![
            "create table my_table(id integer primary key, msg text);",
            "insert into my_table(msg) values('hello');",
            "insert into my_table(msg) values('world');",
        ],
        ExitCode::OK,
        2,
        false,
    );
    // // Create and insert on empty db
    execute(
        false,
        vec![
            "create table my_table(id integer primary key, msg text)",
            "insert into my_table(msg) values('hello')",
            "insert into my_table(msg) values('world')",
        ],
        ExitCode::OK,
        2,
        false,
    );
    // Insert on empty db
    execute(
        false,
        vec!["insert into my_table(msg) values('hello');"],
        ExitCode::USR_ILLEGAL_STATE,
        0,
        false,
    );
}

#[test]
#[serial]
fn queries() {
    fn query(
        db_with_data: bool,
        stmt: &str,
        exit_code: ExitCode,
        col_count: usize,
        row_count: usize,
    ) {
        let rt = setup_test(db_with_data);
        rt.expect_validate_caller_any();

        let params = IpldBlock::serialize_cbor(&QueryParams {
            stmt: stmt.to_string(),
        })
        .unwrap();

        if exit_code.is_success() {
            let block = rt.call::<SqliteActor>(Method::Query as MethodNum, params);
            let block = block.unwrap().unwrap();
            let ret = from_slice::<QueryReturn>(block.data.as_slice()).unwrap();
            assert_eq!(ret.cols.len(), col_count);
            assert_eq!(ret.rows.len(), row_count);
        } else {
            expect_abort(
                exit_code,
                rt.call::<SqliteActor>(Method::Query as MethodNum, params),
            )
        }
        rt.verify();
    }

    // Query existing db
    query(
        true,
        "select \
            Track.Name, Track.Composer, Track.Milliseconds, Track.Bytes, Track.UnitPrice, Genre.Name as GenreName \
            from Track \
            join Genre on Track.GenreId = Genre.GenreId \
            limit 10;",
        ExitCode::OK,
        6,
        10,
    );
    // Query empty db
    query(
        false,
        "select * from not_a_table;",
        ExitCode::USR_ILLEGAL_STATE,
        0,
        0,
    );
}
