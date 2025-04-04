// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fendermint_actor_blobs_shared::{self as shared, blobs::BlobStatus, bytes::B256};
use fil_actors_runtime::{runtime::Runtime, ActorError};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use recall_ipld::hamt::{self, map::TrackedFlushResult};

use super::Subscribers;

/// The stored representation of a blob.
#[derive(Clone, PartialEq, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Blob {
    /// The size of the content.
    pub size: u64,
    /// Blob metadata that contains information for blob recovery.
    pub metadata_hash: B256,
    /// Active subscribers (accounts) that are paying for the blob.
    pub subscribers: Subscribers,
    /// Blob status.
    pub status: BlobStatus,
}

impl Blob {
    /// Returns a [`shared::blobs::Blob`] that is safe to return from actor methods.
    /// TODO: HAMTs should carry max expiry such that we don't full scan here.
    pub fn to_shared(&self, rt: &impl Runtime) -> Result<shared::blobs::Blob, ActorError> {
        let store = rt.store();
        let mut subscribers = HashMap::new();
        self.subscribers.hamt(store)?.for_each(|_, group| {
            group.hamt(store)?.for_each(|id, sub| {
                subscribers.insert(id, sub.expiry);
                Ok(())
            })?;
            Ok(())
        })?;
        Ok(shared::blobs::Blob {
            size: self.size,
            metadata_hash: self.metadata_hash,
            subscribers,
            status: self.status.clone(),
        })
    }
}

/// HAMT wrapper for blobs state.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Blobs {
    /// The HAMT root.
    pub root: hamt::Root<B256, Blob>,
    /// Size of the collection.
    size: u64,
}

impl Blobs {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<B256, Blob>::new(store, "blobs")?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<'a, BS, B256, Blob>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<B256, Blob>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    pub fn len(&self) -> u64 {
        self.size
    }
}
