// Copyright 2024 Textile Inc
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_ipld_hamt::{BytesKey, Hamt};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

const BIT_WIDTH: u32 = 3;

// The state represents a map of fingerprints to fingerpritn metadata
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    pub fingerprints: Cid,
}

/// An object in the object store
#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct FingerprintMetadata {
    /// The address node that calculated the fingerprint.
    pub proposer: String,

    /// The height at which the fingerprint was proposed.
    pub height: u64,

    /// Whether the fingerprint has been verified.
    pub verified: bool,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self> {
        let fingerprints =
            match Hamt::<_, FingerprintMetadata>::new_with_bit_width(store, BIT_WIDTH).flush() {
                Ok(cid) => cid,
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "accumulator actor failed to create empty Amt: {}",
                        e
                    ))
                }
            };

        Ok(Self { fingerprints })
    }

    pub fn set_pending<BS: Blockstore>(
        &mut self,
        store: &BS,
        fingerprint: BytesKey,
        proposer: String,
        height: u64,
    ) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, FingerprintMetadata>::load_with_bit_width(
            &self.fingerprints,
            store,
            BIT_WIDTH,
        )?;
        let meta = FingerprintMetadata {
            proposer,
            height,
            verified: false,
        };
        hamt.set(fingerprint, meta)?;

        self.fingerprints = hamt.flush()?;
        Ok(self.fingerprints)
    }

    // TODO: Instead of setting it verified should we just delete it?
    pub fn set_verified<BS: Blockstore>(
        &mut self,
        store: &BS,
        fingerprint: BytesKey,
    ) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, FingerprintMetadata>::load_with_bit_width(
            &self.fingerprints,
            store,
            BIT_WIDTH,
        )?;
        match hamt
            .get(&fingerprint)
            .map(|v| v.map(|inner| inner.clone()))?
        {
            Some(mut meta) => {
                meta.verified = true;
                hamt.set(fingerprint, meta)?;
            }
            None => {
                return Err(anyhow::anyhow!("fingerprint not found"));
            }
        }

        self.fingerprints = hamt.flush()?;
        Ok(self.fingerprints)
    }

    pub fn list<BS: Blockstore>(
        &self,
        store: &BS,
    ) -> anyhow::Result<Vec<(Vec<u8>, FingerprintMetadata)>> {
        let hamt = Hamt::<_, FingerprintMetadata>::load_with_bit_width(
            &self.fingerprints,
            store,
            BIT_WIDTH,
        )?;
        let mut keys = Vec::new();
        hamt.for_each(|k, v| {
            keys.push((k.0.to_owned(), v.clone()));
            Ok(())
        })?;
        Ok(keys)
    }

    pub fn get<BS: Blockstore>(
        &self,
        store: &BS,
        fingerprint: BytesKey,
    ) -> anyhow::Result<Option<FingerprintMetadata>> {
        let hamt = Hamt::<_, FingerprintMetadata>::load_with_bit_width(
            &self.fingerprints,
            store,
            BIT_WIDTH,
        )?;
        let value = hamt.get(&fingerprint).map(|v| v.cloned())?;
        Ok(value)
    }
}

pub const FINGERPRINT_ACTOR_NAME: &str = "fingerprint";

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    SetPending = frc42_dispatch::method_hash!("SetPending"),
    SetVerified = frc42_dispatch::method_hash!("SetVerified"),
}

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct FingerprintParams {
    pub proposer: Vec<u8>,
    pub height: ChainEpoch,
    pub fingerprint: Vec<u8>,
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
            state.fingerprints,
            Cid::from_str("bafy2bzaceamp42wmmgr2g2ymg46euououzfyck7szknvfacqscohrvaikwfay")
                .unwrap()
        );
    }

    #[test]
    fn test_set_pending() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let fingerprint = BytesKey(Cid::default().to_bytes());
        let proposer = "proposer".to_string();
        let height = 1;

        assert!(state
            .set_pending(&store, fingerprint.clone(), proposer, height)
            .is_ok());

        assert_eq!(
            state.fingerprints,
            Cid::from_str("bafy2bzacebsjg67pjklb2zvs3n7h3ecxzppaewlsqtfpmv3d2tuaxpa2ngyaa")
                .unwrap()
        );

        let result = state.get(&store, fingerprint);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(FingerprintMetadata {
                proposer: "proposer".to_string(),
                height: 1,
                verified: false
            })
        );
    }

    #[test]
    fn test_set_verified_wrong_fingerprint() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let fingerprint = BytesKey(Cid::default().to_bytes());
        
        assert!(state.set_verified(&store, fingerprint.clone()).is_err());
    }

    #[test]
    fn test_set_verified() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let fingerprint = BytesKey(Cid::default().to_bytes());
        let proposer = "proposer".to_string();
        let height = 1;

        // Set pending
        state
            .set_pending(&store, fingerprint.clone(), proposer, height)
            .unwrap();

        // Set verified
        assert!(state.set_verified(&store, fingerprint.clone()).is_ok());

        let result = state.get(&store, fingerprint);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(FingerprintMetadata {
                proposer: "proposer".to_string(),
                height: 1,
                verified: true
            })
        );
    }

    #[test]
    fn test_get() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let fingerprint = BytesKey(Cid::default().to_bytes());
        let proposer = "proposer".to_string();
        let height = 1;

        state
            .set_pending(&store, fingerprint.clone(), proposer, height)
            .unwrap();
        let result = state.get(&store, fingerprint);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(FingerprintMetadata {
                proposer: "proposer".to_string(),
                height: 1,
                verified: false
            })
        );
    }

    #[test]
    fn test_list() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let fingerprint = BytesKey(Cid::default().to_bytes());
        let proposer = "proposer".to_string();
        let height = 1;

        state
            .set_pending(&store, fingerprint.clone(), proposer, height)
            .unwrap();
        let result = state.list(&store);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result[0].0, fingerprint.0);
        assert_eq!(result[0].1.proposer, "proposer".to_string());
        assert_eq!(result[0].1.height, 1);
        assert_eq!(result[0].1.verified, false);
    }
}
