// Copyright 2024 Textile Inc
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::de::DeserializeOwned;
use fvm_ipld_encoding::ser::Serialize;
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

    pub fn set_pending<BS: Blockstore, S: DeserializeOwned + Serialize>(
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
