// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fendermint_actor_blobs_shared::{
    self as shared,
    blobs::{BlobStatus, Subscription},
    bytes::B256,
};
use fil_actors_runtime::{runtime::Runtime, ActorError};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::clock::ChainEpoch;
use log::debug;
use recall_ipld::hamt::{self, map::TrackedFlushResult};

use super::{AddBlobStateParams, BlobSource, Expiries, ExpiryUpdate, Queue, Subscribers};
use crate::caller::Caller;

/// Represents the result of a blob upsert.
#[derive(Debug, Clone)]
pub struct UpsertBlobResult {
    /// New or updated subscription.
    pub subscription: Subscription,
    /// New capacity used by the caller.
    pub new_caller_capacity: u64,
    /// New capacity used by the subnet.
    pub new_subnet_capacity: u64,
    /// Duration for the new credit commitment.
    pub commit_duration: ChainEpoch,
    /// Duration for the returned credit commitment.
    pub return_duration: ChainEpoch,
}

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
    /// Returns a new [`Blob`].
    pub fn new<BS: Blockstore>(
        store: &BS,
        size: u64,
        metadata_hash: B256,
    ) -> Result<Self, ActorError> {
        Ok(Self {
            size,
            metadata_hash,
            subscribers: Subscribers::new(store)?,
            status: BlobStatus::Added,
        })
    }

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
    /// Map of expiries to blob hashes.
    pub expiries: Expiries,
    /// Map of currently added blob hashes to account and source Iroh node IDs.
    pub added: Queue,
    /// Map of currently pending blob hashes to account and source Iroh node IDs.
    pub pending: Queue,
    /// Number of blobs in the collection.
    /// A blob with multiple subscribers and/or subscriptions is only counted once.
    size: u64,
    /// Returns the number of blob bytes in the collection.
    /// A blob with multiple subscribers and/or subscriptions is only counted once.
    pub bytes_size: u64,
}

impl Blobs {
    /// Returns a blob collection.
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<B256, Blob>::new(store, "blobs")?;
        Ok(Self {
            root,
            expiries: Expiries::new(store)?,
            added: Queue::new(store, "added blobs queue")?,
            pending: Queue::new(store, "pending blobs queue")?,
            size: 0,
            bytes_size: 0,
        })
    }

    /// Returns the underlying [`hamt::map::Hamt`].
    pub fn hamt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<'a, BS, B256, Blob>, ActorError> {
        self.root.hamt(store, self.size)
    }

    /// Saves the state from the [`TrackedFlushResult`].
    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<B256, Blob>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    /// Number of blobs in the collection.
    /// A blob with multiple subscribers and/or subscriptions is only counted once.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns the number of blob bytes in the collection.
    /// A blob with multiple subscribers and/or subscriptions is only counted once.
    pub fn bytes_size(&self) -> u64 {
        self.bytes_size
    }

    /// Adds/updates a blob.
    pub fn upsert<BS: Blockstore>(
        &mut self,
        store: &BS,
        caller: &Caller<BS>,
        params: &AddBlobStateParams,
        expiry: ChainEpoch,
    ) -> Result<UpsertBlobResult, ActorError> {
        let mut blobs = self.hamt(store)?;
        let (mut blob, blob_added) = if let Some(blob) = blobs.get(&params.hash)? {
            (blob, false)
        } else {
            (Blob::new(store, params.size, params.metadata_hash)?, true)
        };

        // Add/update subscriber and the subscription
        let result = blob.subscribers.upsert(store, caller, params, expiry)?;

        // Update blob status and added index if the blob is not already resolved
        if !matches!(blob.status, BlobStatus::Resolved) {
            // It's pending or failed, reset to added status
            blob.status = BlobStatus::Added;

            // Add to or update the source in the added queue
            self.added.upsert(
                store,
                params.hash,
                BlobSource::new(
                    caller.subscriber_address(),
                    params.id.clone(),
                    params.source,
                ),
                blob.size,
            )?;
        }

        // Update expiry index
        let mut expiry_updates = vec![];
        if let Some(previous_expiry) = result.previous_subscription_expiry {
            if previous_expiry != expiry {
                expiry_updates.push(ExpiryUpdate::Remove(previous_expiry));
                expiry_updates.push(ExpiryUpdate::Add(expiry));
            }
        } else {
            expiry_updates.push(ExpiryUpdate::Add(expiry));
        }
        self.expiries.update(
            store,
            caller.subscriber_address(),
            params.hash,
            &params.id,
            expiry_updates,
        )?;

        self.save_tracked(blobs.set_and_flush_tracked(&params.hash, blob)?);

        if blob_added {
            debug!("created new blob {}", params.hash);
        }

        Ok(UpsertBlobResult {
            subscription: result.subscription,
            new_caller_capacity: if result.subscriber_added {
                params.size
            } else {
                0
            },
            new_subnet_capacity: if blob_added { params.size } else { 0 },
            commit_duration: result.commit_duration,
            return_duration: result.return_duration,
        })
    }
}
