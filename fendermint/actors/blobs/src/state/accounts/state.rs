// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::accounts::Account;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use recall_ipld::hamt::{self, map::TrackedFlushResult, BytesKey};

/// HAMT wrapper for accounts state.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AccountsState {
    /// The HAMT root.
    pub root: hamt::Root<Address, Account>,
    /// Size of the collection.
    size: u64,
    /// The next account to debit in the current debit cycle.
    /// If this is None, we have finished the debit cycle.
    next_debit_address: Option<Address>,
}

impl AccountsState {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Address, Account>::new(store, "accounts")?;
        Ok(Self {
            root,
            size: 0,
            next_debit_address: None,
        })
    }

    pub fn hamt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<'a, BS, Address, Account>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<Address, Account>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size
    }

    pub fn save_debit_progress(&mut self, next_address: Option<Address>) {
        self.next_debit_address = next_address;
    }

    pub fn get_debit_start_address(&self) -> Option<BytesKey> {
        self.next_debit_address
            .map(|address| BytesKey::from(address.to_bytes()))
    }

    pub fn len(&self) -> u64 {
        self.size
    }
}
