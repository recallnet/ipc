// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;
use std::fmt::Write;
use std::str::FromStr;

use fendermint_actor_blobs_shared::state::{Blob, Hash};
use fendermint_actor_blobs_shared::state::{PublicKey, SubscriptionId};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use recall_ipld::hamt;
use recall_ipld::hamt::map::TrackedFlushResult;

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct BlobsState {
    pub root: hamt::Root<Hash, Blob>,
    size: u64,
}

impl BlobsState {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Hash, Blob>::new(store, "blobs")?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, Hash, Blob>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<Hash, Blob>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    pub fn len(&self) -> u64 {
        self.size
    }
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct BlobsProgressCollection {
    pub root: hamt::Root<Hash, BlobSourceSet>,
    /// Number of blobs in the collection.
    /// A blob with multiple sources is only counted once.
    size: u64,
    /// Number of blob bytes in the collection.
    /// A blob with multiple sources is only counted once.
    bytes_size: u64,
}

/// Blob source is a tuple of subscriber [`Address`], blob [`SubscriptionId`],
/// and an Iroh node [`PublicKey`].
type BlobSource = (Address, SubscriptionId, PublicKey);

/// A set of [`BlobSource`]s implemented with Hamt.
/// A blob in the collection may have multiple sources.
#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple, PartialEq)]
pub struct BlobSourceSet {
    pub root: hamt::Root<String, ()>,
    size: u64,
}

fn pubkey_as_array(slice: &[u8]) -> Result<[u8; 32], ActorError> {
    if slice.len() == 32 {
        let arr = slice.try_into().map_err(|e| {
            ActorError::illegal_argument(format!("cannot convert slice to array: {}", e))
        })?;
        Ok(arr)
    } else {
        Err(ActorError::illegal_argument(String::from(
            "Slice does not have exactly 32 elements",
        )))
    }
}

impl BlobSourceSet {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<String, ()>::new(store, "blob_source_set")?;
        Ok(Self { root, size: 0 })
    }

    // These two statics allow us to map the hamt back to a hash set for read requests that use `take_page`,
    // and also work around the fact a tuple is the set key type.
    pub fn get_id_from_source(source: &BlobSource) -> String {
        let addr_string = format!("{}", source.0);
        let subcr_string = source.1.to_string();
        let pub_key_string = format!("{:?}", source.2);

        // Note: the `:` delimiter can be used to separate addr from the rest of the string, or pub_key from
        //  the rest of the string, but subcription_string is a user defined value and can contain the `:` character
        format!("{}:{}:{}", addr_string, subcr_string, pub_key_string)
    }

    pub fn get_source_from_id(id: String) -> Result<BlobSource, ActorError> {
        let mut parts = id.split(':');
        let addr_string = parts.next().unwrap();
        let pub_key_string = parts.next_back().unwrap();
        // the middle value of the string might contain our delimiter, so we can't simply use `next`
        let sub_id_string = parts.fold(String::new(), |mut id, v| {
            let _ = write!(id, "{:?}", v);
            id
        });

        let addr = Address::from_str(addr_string).map_err(|e| {
            ActorError::illegal_state(format!("invalid hash set key in BlobSourceSet: {}", e))
        })?;
        let sub_id = SubscriptionId::new(&sub_id_string)?;
        let pub_key = PublicKey(pubkey_as_array(pub_key_string.as_bytes()).map_err(|e| {
            ActorError::illegal_state(format!("invalid hash set key in BlobSourceSet: {}", e))
        })?);

        Ok((addr, sub_id, pub_key))
    }

    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, String, ()>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn as_hash_set<BS: Blockstore>(
        &self,
        store: &BS,
    ) -> Result<HashSet<BlobSource>, ActorError> {
        let mut hset: HashSet<BlobSource> = HashSet::new();
        let hamt = self.hamt(store)?;

        hamt.for_each(|key, _| {
            hset.insert(Self::get_source_from_id(key)?);
            Ok(())
        })?;

        Ok(hset)
    }

    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<String, ()>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    pub fn len(&self) -> u64 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn insert<BS: Blockstore>(
        &mut self,
        store: &BS,
        source: &BlobSource,
    ) -> Result<(), ActorError> {
        let mut source_set_hamt = self.hamt(store)?;
        let upsert_source_id = Self::get_id_from_source(source);
        self.save_tracked(source_set_hamt.set_and_flush_tracked(&upsert_source_id, ())?);

        Ok(())
    }

    pub fn remove<BS: Blockstore>(
        &mut self,
        store: &BS,
        source: &BlobSource,
    ) -> Result<bool, ActorError> {
        let mut source_set_hamt = self.hamt(store)?;
        let remove_source_id = Self::get_id_from_source(source);
        let (del_result, was_deleted) =
            source_set_hamt.delete_and_flush_tracked(&remove_source_id)?;
        self.save_tracked(del_result);

        let was_deleted = was_deleted.is_some();

        Ok(was_deleted)
    }
}

// type BlobSourceSet = HashSet<(Address, SubscriptionId, PublicKey)>;

impl BlobsProgressCollection {
    /// Returns a new progress collection.
    pub fn new<BS: Blockstore>(store: &BS, name: &str) -> Result<Self, ActorError> {
        let root = hamt::Root::<Hash, BlobSourceSet>::new(store, name)?;
        Ok(Self {
            root,
            size: 0,
            bytes_size: 0,
        })
    }

    /// Returns the underlying [`hamt::map::Hamt`].
    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, Hash, BlobSourceSet>, ActorError> {
        self.root.hamt(store, self.size)
    }

    /// Saves the state from the [`TrackedFlushResult`].
    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<Hash, BlobSourceSet>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    /// Number of blobs in the collection.
    /// A blob with multiple sources is only counted once.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Returns the number of blob bytes in the collection.
    /// A blob with multiple sources is only counted once.
    pub fn bytes_size(&self) -> u64 {
        self.bytes_size
    }

    /// Adds/updates an entry in the collection.
    pub fn upsert<BS: Blockstore>(
        &mut self,
        store: BS,
        hash: Hash,
        source: BlobSource,
        blob_size: u64,
    ) -> Result<(), ActorError> {
        let mut map = self.hamt(&store)?;
        let blob_source_set = map.get(&hash)?;

        match blob_source_set {
            Some(mut source_set) => {
                // Modify existing entry
                let mut source_set_hamt = source_set.hamt(&store)?;
                let upsert_source_id = BlobSourceSet::get_id_from_source(&source);
                source_set
                    .save_tracked(source_set_hamt.set_and_flush_tracked(&upsert_source_id, ())?);

                map.set(&hash, source_set)?;
            }
            None => {
                // create new entry
                let mut source_set = BlobSourceSet::new(&store)?;
                let mut source_set_hamt = source_set.hamt(&store)?;
                let upsert_source_id = BlobSourceSet::get_id_from_source(&source);
                source_set
                    .save_tracked(source_set_hamt.set_and_flush_tracked(&upsert_source_id, ())?);

                map.set(&hash, source_set)?;
                // Entry did not exist, add to tracked bytes size
                self.bytes_size += blob_size;
            }
        };
        self.save_tracked(map.flush_tracked()?);
        Ok(())
    }

    /// Returns a page of entries from the collection.
    pub fn take_page<BS: Blockstore>(
        &self,
        store: BS,
        size: u32,
    ) -> Result<Vec<(Hash, HashSet<BlobSource>)>, ActorError> {
        let map = self.hamt(&store)?;
        let mut page = Vec::with_capacity(size as usize);
        map.for_each_ranged(None, Some(size as usize), |hash, set| {
            page.push((hash, set.clone().as_hash_set(&store)?));
            Ok(())
        })?;
        page.shrink_to_fit();
        Ok(page)
    }

    /// Removes a source from an entry in the collection.
    /// If the entry is empty after removing the source, the entry is also removed.
    pub fn remove_source<BS: Blockstore>(
        &mut self,
        store: BS,
        hash: Hash,
        source: BlobSource,
        blob_size: u64,
    ) -> Result<(), ActorError> {
        let mut map = self.hamt(&store)?;
        if let Some(mut set) = map.get(&hash)? {
            if set.remove(&store, &source)? {
                if set.is_empty() {
                    map.delete(&hash)?;
                    self.bytes_size -= blob_size;
                } else {
                    map.set(&hash, set)?;
                }
                self.save_tracked(map.flush_tracked()?);
            }
        }
        Ok(())
    }

    /// Removes an entry from the collection.
    pub fn remove_entry<BS: Blockstore>(
        &mut self,
        store: BS,
        hash: &Hash,
        blob_size: u64,
    ) -> Result<(), ActorError> {
        let mut map = self.hamt(store)?;
        let (res, deleted) = map.delete_and_flush_tracked(hash)?;
        self.save_tracked(res);
        if deleted.is_some() {
            self.bytes_size -= blob_size;
        }
        Ok(())
    }
}
