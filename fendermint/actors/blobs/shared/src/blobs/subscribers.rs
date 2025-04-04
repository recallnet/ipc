// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use recall_ipld::{hamt, hamt::map::TrackedFlushResult};

use super::SubscriptionGroup;

/// HAMT wrapper tracking blob [`SubscriptionGroup`]s by subscriber address.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct BlobSubscribers {
    pub root: hamt::Root<Address, SubscriptionGroup>,
    size: u64,
}

impl BlobSubscribers {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Address, SubscriptionGroup>::new(store, "blob_subscribers")?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<'a, BS, Address, SubscriptionGroup>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(
        &mut self,
        tracked_flush_result: TrackedFlushResult<Address, SubscriptionGroup>,
    ) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    pub fn len(&self) -> u64 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}
