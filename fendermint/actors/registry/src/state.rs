// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;

/// The state represents a bucket backed by a Hamt.
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    pub ttls: HashMap<Address, ChainEpoch>,
}

impl State {
    pub fn new<BS: Blockstore>(_: &BS) -> Self {
        Self {
            ttls: HashMap::new(),
        }
    }
}
