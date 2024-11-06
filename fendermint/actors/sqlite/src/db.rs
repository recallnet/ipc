use cid::multihash::Code;
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{tuple::*, CborStore};

use crate::errors::Error;

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct DB {
    pub page_size: usize,
    pub page_count: usize,
    pub bucket_size: usize,
    pub tree_height: usize,
    pub page_tree: Vec<Cid>,
}

impl DB {
    pub fn new(
        store: &impl Blockstore,
        data: &[u8],
        page_size: usize,
        bucket_size: usize,
    ) -> Result<Self, Error> {

        let len: usize = data.len();
        let mut page_count = len / page_size;
        if len % page_size > 0 {
            page_count += 1;
        }

        // Store pages
        let mut pages = vec![Cid::default(); page_count];
        for p in 0..page_count {
            let mut end = (p + 1) * page_size;
            if end > len {
                end = len;
            }
            let page = &data[p * page_size..end];
            pages[p] = store
                .put_cbor(&page, Code::Blake2b256)
                .map_err(|e| Error::from(e.to_string()))?;
        }

        // Bucket pages into binary-ish tree
        let mut tree: Vec<Cid> = pages;
        let mut h = 0;
        while tree.len() > bucket_size {
            let mut buck = vec![];
            for p in tree.chunks(bucket_size) {
                buck.push(
                    store
                        .put_cbor(&p, Code::Blake2b256)
                        .map_err(|e| Error::from(e.to_string()))?,
                );
            }
            tree = buck;
            h += 1;
        }

        Ok(DB {
            page_size,
            page_count,
            bucket_size,
            tree_height: h,
            page_tree: tree,
        })
    }

    pub fn get_page(&self, store: &impl Blockstore, index: usize) -> Result<Vec<u8>, Error> {
        // If 'tree_height=0', 'page_tree' is raw pages.
        if self.tree_height == 0 {
            return DB::get_cbor(store, self.page_tree[index]);
        }

        // At each tree height, find and open the bucket that holds the requested page.
        // Buckets have capacity `buck_size ^ tree_height` pages.
        let mut buck = vec![Cid::default(); self.bucket_size];
        for h in (1..=self.tree_height).rev() {
            let cap = usize::pow(self.bucket_size, h as u32);
            let idx = index / cap;
            if h == self.tree_height {
                buck = DB::get_cbor(store, self.page_tree[idx])?;
            } else {
                buck = DB::get_cbor(store, buck[idx])?;
            }
        }

        // Finally, open and return the page.
        DB::get_cbor(store, buck[index % self.bucket_size])
    }

    fn get_cbor<T>(store: &impl Blockstore, cid: Cid) -> Result<Vec<T>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        match store
            .get_cbor::<Vec<T>>(&cid)
            .map_err(|e| Error::from(e.to_string()))?
        {
            Some(a) => Ok(a),
            None => Err(Error::from(format!("failed to get {} from store", cid))),
        }
    }
}
