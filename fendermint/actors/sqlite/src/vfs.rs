use fil_actors_runtime::runtime::Runtime;
use fvm_ipld_encoding::CborStore;
use multihash::Code;
use sqlite_vfs::{LockKind, OpenKind, OpenOptions, Vfs};
use std::io::{self, ErrorKind};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::state::State;

pub struct PagesVfs<'r, const PAGE_SIZE: usize, RT: Runtime> {
    lock_state: Arc<Mutex<LockState>>,
    rt: &'r RT,
}

#[derive(Debug, Default)]
struct LockState {
    read: usize,
    write: Option<bool>,
}

pub struct Connection<'r, const PAGE_SIZE: usize, RT: Runtime>
where
    RT::Blockstore: Clone,
{
    lock_state: Arc<Mutex<LockState>>,
    lock: LockKind,
    rt: &'r RT,
}

impl<'r, const PAGE_SIZE: usize, RT: Runtime> PagesVfs<'r, PAGE_SIZE, RT> {
    pub fn new(rt: &'r RT) -> Self
    where
        RT::Blockstore: Clone,
    {
        PagesVfs {
            lock_state: Arc::new(Mutex::new(Default::default())),
            rt,
        }
    }
}

impl<'r, const PAGE_SIZE: usize, RT: Runtime> Vfs for PagesVfs<'r, PAGE_SIZE, RT>
where
    RT::Blockstore: Clone,
{
    type Handle = Connection<'r, PAGE_SIZE, RT>;

    fn open(&self, db: &str, opts: OpenOptions) -> Result<Self::Handle, io::Error> {
        // Always open the same database for now.
        if db != "main.db" {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                format!("unexpected database name `{db}`; expected `main.db`"),
            ));
        }

        // Only main databases supported right now (no journal, wal, temporary, ...)
        if opts.kind != OpenKind::MainDb {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "only main database supported right now (no journal, wal, ...)",
            ));
        }

        Ok(Connection {
            lock_state: self.lock_state.clone(),
            lock: LockKind::None,
            rt: self.rt,
        })
    }

    fn delete(&self, _db: &str) -> Result<(), io::Error> {
        // Only used to delete journal or wal files, which both are not implemented yet, thus simply
        // ignored for now.
        Ok(())
    }

    fn exists(&self, db: &str) -> Result<bool, io::Error> {
        let st: State = self.rt.state().unwrap();
        Ok(db == "main.db" && st.db.page_count > 0)
    }

    fn temporary_name(&self) -> String {
        String::from("main.db")
    }

    fn random(&self, buffer: &mut [i8]) {
        // NOTE (sander): We don't have access to OS randomness:
        //   rand::Rng::fill(&mut rand::thread_rng(), buffer);
        // NOTE (sander): I'm trying to use one of the runtime randomness methods.
        // I pulled the rand_epoch and entropy from the evm actor's usage of
        // this method, but I don't know if it will actually work / lead to problems.
        // let randomness = self
        //     .rt
        //     .get_randomness_from_beacon(
        //         DomainSeparationTag::EvmPrevRandao,
        //         self.rt.curr_epoch(),
        //         b"prevrandao",
        //     )
        //     .unwrap()
        //     .map(|x| x as i8);
        // buffer[..randomness.len()].copy_from_slice(&randomness);

        // NOTE (sander): The above panic in Fendermint w/o a good error. For now,
        // just return zeros.
        let data = (0..buffer.len()).map(|_| 0_u128 as i8).collect::<Vec<_>>();
        buffer.copy_from_slice(&data);
    }

    fn sleep(&self, _duration: Duration) -> Duration {
        // NOTE (sander): We don't have access to OS time or CPU and therefore cannot sleep.
        // Is this safe? Probably not!
        Duration::from_millis(0)
    }
}

impl<'r, const PAGE_SIZE: usize, RT: Runtime> sqlite_vfs::DatabaseHandle
    for Connection<'r, PAGE_SIZE, RT>
where
    RT::Blockstore: Clone,
{
    type WalIndex = sqlite_vfs::WalDisabled;

    fn size(&self) -> Result<u64, io::Error> {
        let size = self.page_count() * PAGE_SIZE;
        println!("size={size}");
        Ok(size as u64)
    }

    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> Result<(), io::Error> {
        let index = offset as usize / PAGE_SIZE;
        let offset = offset as usize % PAGE_SIZE;

        let data = self.get_page(index as u32);
        if data.len() < buf.len() + offset {
            eprintln!(
                "read {} < {} -> UnexpectedEof",
                data.len(),
                buf.len() + offset
            );
            return Err(ErrorKind::UnexpectedEof.into());
        }

        println!("read index={} len={} offset={}", index, buf.len(), offset);
        buf.copy_from_slice(&data[offset..offset + buf.len()]);

        Ok(())
    }

    fn write_all_at(&mut self, buf: &[u8], offset: u64) -> Result<(), io::Error> {
        if offset as usize % PAGE_SIZE > 0 {
            return Err(io::Error::new(
                ErrorKind::Other,
                "unexpected write across page boundaries",
            ));
        }

        let index = offset as usize / PAGE_SIZE;
        let page = buf.try_into().map_err(|_| {
            io::Error::new(
                ErrorKind::Other,
                format!(
                    "unexpected write size {}; expected {}",
                    buf.len(),
                    PAGE_SIZE
                ),
            )
        })?;
        println!("write index={} len={}", index, buf.len());
        self.put_page(index as u32, page);

        Ok(())
    }

    fn sync(&mut self, _data_only: bool) -> Result<(), io::Error> {
        // Everything is directly written to storage, so no extra steps necessary to sync.
        Ok(())
    }

    fn set_len(&mut self, size: u64) -> Result<(), io::Error> {
        let mut page_count = size as usize / PAGE_SIZE;
        if size as usize % PAGE_SIZE > 0 {
            page_count += 1;
        }

        let current_page_count = self.page_count();
        if page_count > 0 && page_count < current_page_count {
            // NOTE (sander): The example VFS removed pages in the following way:
            //   for i in (page_count..current_page_count).into_iter().rev() {
            //       self.del_page(i as u32);
            //   }
            // Because, AFAICT, pages are never removed from the middle of the database,
            // we may as well remove them all at once from the end. This saves
            // multiple roundtrips to the blockstore.
            self.del_last_pages(page_count as u32);
        }

        Ok(())
    }

    fn lock(&mut self, lock: LockKind) -> Result<bool, io::Error> {
        let ok = Self::lock(self, lock);
        println!("locked={}", ok);
        Ok(ok)
    }

    fn reserved(&mut self) -> Result<bool, io::Error> {
        Ok(Self::reserved(self))
    }

    fn current_lock(&self) -> Result<LockKind, io::Error> {
        Ok(self.lock)
    }

    fn set_chunk_size(&self, chunk_size: usize) -> Result<(), io::Error> {
        if chunk_size != PAGE_SIZE {
            eprintln!("set_chunk_size={chunk_size} (rejected)");
            Err(io::Error::new(
                ErrorKind::Other,
                "changing chunk size is not allowed",
            ))
        } else {
            // TODO: no op curently
            eprintln!("set_chunk_size={chunk_size}");
            Ok(())
        }
    }

    fn wal_index(&self, _readonly: bool) -> Result<Self::WalIndex, io::Error> {
        Ok(sqlite_vfs::WalDisabled)
    }
}

impl<'r, const PAGE_SIZE: usize, RT: Runtime> Connection<'r, PAGE_SIZE, RT>
where
    RT::Blockstore: Clone,
{
    fn get_page(&self, ix: u32) -> [u8; PAGE_SIZE] {
        let st: State = self.rt.state().unwrap();
        println!("get_page; ix={}; pages={}", ix, st.db.page_count);

        // Fetch page
        let mut data = [0u8; PAGE_SIZE];
        if (ix as usize) < st.db.page_count {
            let page: Vec<u8> = st.db.get_page(self.rt.store(), ix as usize).unwrap();
            println!("{:?}", page);
            data.copy_from_slice(&page);
        }
        data
    }

    fn put_page(&self, ix: u32, data: &[u8; PAGE_SIZE]) {
        let mut st: State = self.rt.state().unwrap();
        println!("put_page; ix={}; pages={}", ix, st.db.page_count);

        // Add the new page to the blockstore
        let page = self
            .rt
            .store()
            .put_cbor(&data.to_vec(), Code::Blake2b256)
            .unwrap();

        // Add or replace in page state
        if ix as usize == st.db.page_count {
            st.db.page_tree.push(page);
            st.db.page_count += 1;
        } else {
            st.db.page_tree[ix as usize] = page;
        }

        // Save new state
        let new_root = self.rt.store().put_cbor(&st, Code::Blake2b256).unwrap();
        self.rt.set_state_root(&new_root).unwrap();
    }

    fn del_last_pages(&self, retain: u32) {
        let mut st: State = self.rt.state().unwrap();
        println!(
            "del_last_pages; retain={}; pages={}",
            retain, st.db.page_count
        );

        // Retain some pages
        st.db.page_tree = st.db.page_tree[..(retain as usize) * PAGE_SIZE].to_vec();

        // Save new state
        let new_root = self.rt.store().put_cbor(&st, Code::Blake2b256).unwrap();
        self.rt.set_state_root(&new_root).unwrap();
    }

    fn page_count(&self) -> usize {
        let st: State = self.rt.state().unwrap();
        println!("page_count; pages={}", st.db.page_count);

        st.db.page_count
    }

    fn lock(&mut self, to: LockKind) -> bool {
        if self.lock == to {
            return true;
        }

        let mut lock_state = self.lock_state.lock().unwrap();

        // eprintln!(
        //     "lock state={:?} from={:?} to={:?}",
        //     lock_state, self.lock, to
        // );

        // The following locking implementation is probably not sound (wouldn't be surprised if it
        // potentially dead-locks), but suffice for the experiment.

        match to {
            LockKind::None => {
                if self.lock == LockKind::Shared {
                    lock_state.read -= 1;
                } else if self.lock > LockKind::Shared {
                    lock_state.write = None;
                }
                self.lock = LockKind::None;
                true
            }

            LockKind::Shared => {
                if lock_state.write == Some(true) && self.lock <= LockKind::Shared {
                    return false;
                }

                lock_state.read += 1;
                if self.lock > LockKind::Shared {
                    lock_state.write = None;
                }
                self.lock = LockKind::Shared;
                true
            }

            LockKind::Reserved => {
                if lock_state.write.is_some() || self.lock != LockKind::Shared {
                    return false;
                }

                if self.lock == LockKind::Shared {
                    lock_state.read -= 1;
                }
                lock_state.write = Some(false);
                self.lock = LockKind::Reserved;
                true
            }

            LockKind::Pending => {
                // cannot be requested directly
                false
            }

            LockKind::Exclusive => {
                if lock_state.write.is_some() && self.lock <= LockKind::Shared {
                    return false;
                }

                if self.lock == LockKind::Shared {
                    lock_state.read -= 1;
                }

                lock_state.write = Some(true);
                if lock_state.read == 0 {
                    self.lock = LockKind::Exclusive;
                    true
                } else {
                    self.lock = LockKind::Pending;
                    false
                }
            }
        }
    }

    fn reserved(&self) -> bool {
        if self.lock > LockKind::Shared {
            return true;
        }

        let lock_state = self.lock_state.lock().unwrap();
        lock_state.write.is_some()
    }
}

impl<'r, const PAGE_SIZE: usize, RT: Runtime> Drop for Connection<'r, PAGE_SIZE, RT>
where
    RT::Blockstore: Clone,
{
    fn drop(&mut self) {
        if self.lock != LockKind::None {
            self.lock(LockKind::None);
        }
    }
}
