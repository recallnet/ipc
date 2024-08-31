// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Blob blake3 hash.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hash(pub [u8; 32]);

/// Source https://github.com/n0-computer/iroh/blob/main/iroh-base/src/hash.rs
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // the result will be 52 bytes
        let mut res = [b'b'; 52];
        // write the encoded bytes
        data_encoding::BASE32_NOPAD.encode_mut(self.0.as_slice(), &mut res);
        // convert to string, this is guaranteed to succeed
        let t = std::str::from_utf8_mut(res.as_mut()).unwrap();
        // hack since data_encoding doesn't have BASE32LOWER_NOPAD as a const
        t.make_ascii_lowercase();
        // write the str, no allocations
        f.write_str(t)
    }
}

/// Iroh node public key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublicKey(pub [u8; 32]);

/// The stored representation of a credit account.
#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Account {
    /// Total size of all blobs managed by the account.
    pub capacity_used: BigInt,
    /// Current free credit in byte-blocks that can be used for new commitments.
    pub credit_free: BigInt,
    /// Current committed credit in byte-blocks that will be used for debits.
    pub credit_committed: BigInt,
    /// The chain epoch of the last debit.
    pub last_debit_epoch: ChainEpoch,
}

/// The stored representation of a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Blob {
    /// The size of the content.
    pub size: u64,
    /// Active subscribers (accounts) that are paying for the blob.
    pub subs: HashMap<Address, Subscription>,
    /// Blob status.
    pub status: BlobStatus,
}

/// The status of a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BlobStatus {
    /// Blob was added at [`ChainEpoch`].
    Added(ChainEpoch),
    /// Blob was successfully resolved.
    Resolved,
    /// Blob resolution failed.
    Failed,
}

impl fmt::Display for BlobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlobStatus::Added(epoch) => write!(f, "added({})", epoch),
            BlobStatus::Resolved => write!(f, "resolved"),
            BlobStatus::Failed => write!(f, "failed"),
        }
    }
}

/// An object used to determine what [`Account`](s) are accountable for a blob, and for how long.
/// Subscriptions allow us to distribute the cost of a blob across multiple accounts that
/// have added the same blob.   
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Subscription {
    /// Expiry block.
    pub expiry: ChainEpoch,
    /// Source Iroh node ID used for ingestion.
    /// This might be unique to each instance of the same blob.
    /// It's included here for record keeping.
    pub source: PublicKey,
}
