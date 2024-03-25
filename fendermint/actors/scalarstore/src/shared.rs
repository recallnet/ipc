// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub const SCALARSTORE_ACTOR_NAME: &str = "scalarstore";

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    StoreNumber = frc42_dispatch::method_hash!("StoreNumber"),
    GetNumber = frc42_dispatch::method_hash!("GetNumber"),
}

// The state represents an mmr with peaks stored in an Amt
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    // todo add a string
    pub stored_number: u64,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self> {
        // TODO: Does this need to initialize off of the blockstore?
        Ok(Self { stored_number: 0 })
    }

    pub fn get_number<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<u64> {
        Ok(self.stored_number)
    }

    pub fn store_number<BS: Blockstore>(&mut self, store: &BS, value: u64) -> anyhow::Result<()> {
        // TODO: Does this need to store into the blockstore?
        self.stored_number = value;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let state = State::new(&store);
        assert!(state.is_ok());
        let state = state.unwrap();
        assert_eq!(state.stored_number, 1); // todo 0
    }

    #[test]
    fn test_number() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        state.store_number(&store, 5).expect("store_number failed");
        assert_eq!(5, state.get_number(&store).expect("get_number failed")); // todo expect?
    }
}
