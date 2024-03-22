// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::multihash::{Code, MultihashDigest};
use cid::Cid;
use fvm_ipld_amt::Amt;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{
    de::DeserializeOwned,
    ser::Serialize,
    to_vec,
    tuple::{Deserialize_tuple, Serialize_tuple},
    CborStore, DAG_CBOR,
};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub const STRINGSTORE_ACTOR_NAME: &str = "stringstore";
const BIT_WIDTH: u32 = 3;

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    // Push = frc42_dispatch::method_hash!("Push"),
    // Root = frc42_dispatch::method_hash!("Root"),
    // Peaks = frc42_dispatch::method_hash!("Peaks"),
    // Count = frc42_dispatch::method_hash!("Count"),
}

// The state represents an mmr with peaks stored in an Amt
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    pub storedNumber: u64, // todo make it a string
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self> {
        Ok(Self {
            storedNumber: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_constructor() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let state = State::new(&store);
        assert!(state.is_ok());
        let state = state.unwrap();
        assert_eq!(
            state.peaks,
            Cid::from_str("bafy2bzacedijw74yui7otvo63nfl3hdq2vdzuy7wx2tnptwed6zml4vvz7wee")
                .unwrap()
        );
        assert_eq!(state.leaf_count, 0);
    }

    #[test]
    fn test_push_simple() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let obj = vec![1, 2, 3];
        assert_eq!(
            state.push(&store, obj).expect("push failed"),
            state.get_root(&store).expect("get_root failed")
        );
        assert_eq!(state.leaf_count, 1);
    }

    #[test]
    fn test_get_peaks() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let obj = vec![1, 2, 3];
        assert!(state.push(&store, obj).is_ok());
        assert_eq!(state.leaf_count, 1);
        let peaks = state.get_peaks(&store);
        assert!(peaks.is_ok());
        let peaks = peaks.unwrap();
        assert_eq!(peaks.len(), 1);
        assert_eq!(
            peaks[0],
            Cid::from_str("bafy2bzacebltuz74cvzod3x7cx3eledj4gn5vjcer7znymoq56htf2e3cclok")
                .unwrap()
        );
    }

    #[test]
    fn test_bag_peaks() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        state.push(&store, vec![1]).unwrap();
        state.push(&store, vec![2]).unwrap();
        state.push(&store, vec![3]).unwrap();
        state.push(&store, vec![4]).unwrap();
        state.push(&store, vec![5]).unwrap();
        state.push(&store, vec![6]).unwrap();
        state.push(&store, vec![7]).unwrap();
        state.push(&store, vec![8]).unwrap();
        state.push(&store, vec![9]).unwrap();
        state.push(&store, vec![10]).unwrap();
        let root = state.push(&store, vec![11]).unwrap();
        let peaks = state.get_peaks(&store).unwrap();
        assert_eq!(peaks.len(), 3);
        assert_eq!(state.leaf_count, 11);
        assert_eq!(root, state.get_root(&store).expect("get_root failed"));
    }
}
