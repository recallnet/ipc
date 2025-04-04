// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fil_actors_runtime::{runtime::Runtime, ActorError};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::clock::ChainEpoch;

use super::{BlobStatus, BlobSubscribers, SubscriptionId};
use crate::bytes::B256;

/// The stored representation of a blob.
#[derive(Clone, PartialEq, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Blob {
    /// The size of the content.
    pub size: u64,
    /// Blob metadata that contains information for blob recovery.
    pub metadata_hash: B256,
    /// Active subscribers (accounts) that are paying for the blob.
    pub subscribers: BlobSubscribers,
    /// Blob status.
    pub status: BlobStatus,
}

/// The return type used for Blob.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct BlobInfo {
    /// The size of the content.
    pub size: u64,
    /// Blob metadata that contains information for blob recovery.
    pub metadata_hash: B256,
    /// Active subscribers (accounts) that are paying for the blob to expiry.
    pub subscribers: HashMap<SubscriptionId, ChainEpoch>,
    /// Blob status.
    pub status: BlobStatus,
}

impl BlobInfo {
    /// Returns [`BlobInfo`] from [`Blob`].
    /// TODO: HAMTs should carry max expiry such that we don't full scan here.
    pub fn from(rt: &impl Runtime, blob: Blob) -> Result<Self, ActorError> {
        let store = rt.store();
        let mut subscribers = HashMap::new();
        blob.subscribers.hamt(store)?.for_each(|_, group| {
            group.hamt(store)?.for_each(|id, sub| {
                subscribers.insert(id, sub.expiry);
                Ok(())
            })?;
            Ok(())
        })?;
        Ok(Self {
            size: blob.size,
            metadata_hash: blob.metadata_hash,
            subscribers,
            status: blob.status,
        })
    }
}
