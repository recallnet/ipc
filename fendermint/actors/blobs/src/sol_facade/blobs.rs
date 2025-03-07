// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::fmt::Display;
use std::str::FromStr;
use fvm_ipld_blockstore::Blockstore;
use fendermint_actor_blobs_shared::state::{Blob, BlobRequest, BlobStatus, Hash, PublicKey, Subscription, SubscriptionId};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fendermint_actor_blobs_shared::params::{AddBlobParams, DeleteBlobParams, GetAddedBlobsParams, GetBlobParams, GetBlobStatusParams, GetPendingBlobsParams, GetStatsReturn};
use fil_actors_runtime::{actor_error, ActorError};
use recall_actor_sdk::{TryIntoEVMEvent};
use recall_sol_facade::blobs as sol;
use recall_sol_facade::primitives::U256;
use recall_sol_facade::types::{base32, Base32, SolCall, SolInterface, H160};
use num_traits::Zero;
use recall_ipld::hamt::MapKey;
pub use recall_sol_facade::blobs::Calls;

use crate::sol_facade::{AbiCall, AbiEncodeError};

// ----- Events ----- //

pub struct BlobAdded<'a> {
    pub subscriber: Address,
    pub hash: &'a Hash,
    pub size: u64,
    pub expiry: ChainEpoch,
    pub bytes_used: u64,
}

impl TryIntoEVMEvent for BlobAdded<'_> {
    type Target = sol::Event;

    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error> {
        let subscriber: H160 = self.subscriber.try_into()?;
        Ok(sol::Event::BlobAdded(sol::BlobAdded {
            subscriber: subscriber.into(),
            hash: self.hash.0.into(),
            size: U256::from(self.size),
            expiry: U256::from(self.expiry),
            bytesUsed: U256::from(self.bytes_used),
        }))
    }
}

pub struct BlobPending<'a> {
    pub subscriber: Address,
    pub hash: &'a Hash,
    pub source: &'a PublicKey,
}
impl TryIntoEVMEvent for BlobPending<'_> {
    type Target = sol::Event;
    fn try_into_evm_event(self) -> Result<sol::Event, anyhow::Error> {
        let subscriber: H160 = self.subscriber.try_into()?;
        Ok(sol::Event::BlobPending(sol::BlobPending {
            subscriber: subscriber.into(),
            hash: self.hash.0.into(),
            sourceId: self.source.0.into(),
        }))
    }
}

pub struct BlobFinalized<'a> {
    pub subscriber: Address,
    pub hash: &'a Hash,
    pub resolved: bool,
}
impl TryIntoEVMEvent for BlobFinalized<'_> {
    type Target = sol::Event;
    fn try_into_evm_event(self) -> Result<sol::Event, anyhow::Error> {
        let subscriber: H160 = self.subscriber.try_into()?;
        Ok(sol::Event::BlobFinalized(sol::BlobFinalized {
            subscriber: subscriber.into(),
            hash: self.hash.0.into(),
            resolved: self.resolved,
        }))
    }
}

pub struct BlobDeleted<'a> {
    pub subscriber: Address,
    pub hash: &'a Hash,
    pub size: u64,
    pub bytes_released: u64,
}
impl TryIntoEVMEvent for BlobDeleted<'_> {
    type Target = sol::Event;
    fn try_into_evm_event(self) -> Result<sol::Event, anyhow::Error> {
        let subscriber: H160 = self.subscriber.try_into()?;
        Ok(sol::Event::BlobDeleted(sol::BlobDeleted {
            subscriber: subscriber.into(),
            hash: self.hash.0.into(),
            size: U256::from(self.size),
            bytesReleased: U256::from(self.bytes_released),
        }))
    }
}

// ----- Calls ----- //

pub fn can_handle(input_data: &recall_actor_sdk::InputData) -> bool {
    Calls::valid_selector(input_data.selector())
}

pub fn parse_input(input: &recall_actor_sdk::InputData) -> Result<Calls, ActorError> {
    Calls::abi_decode_raw(input.selector(), input.calldata(), true).map_err(|e| {
        actor_error!(illegal_argument, format!("invalid call: {}", e))
    })
}

fn blob_requests_to_tuple(blob_requests: Vec<BlobRequest>) -> Result<Vec<sol::BlobTuple>, anyhow::Error> {
    blob_requests.iter().map(|blob_request| {
        let source_info: Result<Vec<sol::BlobSourceInfo>, anyhow::Error> = blob_request.1.iter().map(|item| {
            let address = item.0;
            let public_key = item.2;
            let subscription_id = item.1.clone();

            Ok(sol::BlobSourceInfo {
                subscriber: H160::try_from(address)?.into(),
                subscriptionId: subscription_id.into(),
                source: base32::encode(public_key),
            })
        }).collect::<Result<Vec<_>, anyhow::Error>>();

        let blob_hash = blob_request.0;
        Ok(sol::BlobTuple {
            blobHash: base32::encode(blob_hash),
            sourceInfo: source_info?
        })
    }).collect::<Result<Vec<_>, anyhow::Error>>()
}

impl AbiCall for sol::getAddedBlobsCall {
    type Params = GetAddedBlobsParams;
    type Returns = Vec<BlobRequest>;
    type Output = Result<Vec<u8>, AbiEncodeError>;

    fn params(&self) -> Self::Params {
        GetAddedBlobsParams(self.size)
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let blob_tuples = blob_requests_to_tuple(returns)?;
        Ok(Self::abi_encode_returns(&(blob_tuples,)))
    }
}

fn try_decode_address<T: AsRef<[u8]>>(data: T) -> Result<Address, AbiEncodeError> {
    let k = H160::try_from(data.as_ref())?;
    k.try_into().map_err(Into::into)
}

fn try_decode_option_address<T: AsRef<[u8]>>(data: T) -> Result<Option<Address>, AbiEncodeError> {
    let address: H160 = H160::try_from(data.as_ref())?;
    if address.is_null() {
        Ok(None)
    } else {
        let address = address.try_into()?;
        Ok(Some(address))
    }
}

fn try_decode_hash<T: AsRef<[u8]>>(data: T) -> Result<Hash, AbiEncodeError> {
    let base32 = Base32::decode(data.as_ref())?;
    Hash::try_from(base32.as_ref()).map_err(Into::into)
}

fn try_decode_public_key<T: AsRef<[u8]>>(data: T) -> Result<PublicKey, AbiEncodeError> {
    let base32 = Base32::decode(data.as_ref())?;
    PublicKey::try_from(base32.as_ref()).map_err(Into::into)
}

impl AbiCall for sol::getBlobStatusCall {
    type Params = Result<GetBlobStatusParams, ActorError>;
    type Returns = Option<BlobStatus>;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let subscriber: Address = try_decode_address(self.subscriber)?;
        let blob_hash: Hash = try_decode_hash(self.blobHash.clone())?;
        let subscription_id: SubscriptionId = self.subscriptionId.clone().try_into()?;
        Ok(GetBlobStatusParams {
            subscriber: subscriber.into(),
            hash: blob_hash,
            id: subscription_id,
        })
    }

    fn returns(&self, blob_status: Self::Returns) -> Self::Output {
        // Use BlobStatus::Failed if None got passed
        let blob_status = blob_status.unwrap_or(BlobStatus::Failed);
        let value = blob_status_as_solidity_enum(blob_status);
        Self::abi_encode_returns(&(value,))
    }
}

fn blob_status_as_solidity_enum(blob_status: BlobStatus) -> u8 {
    match blob_status {
        BlobStatus::Added => 0,
        BlobStatus::Pending => 1,
        BlobStatus::Resolved => 2,
        BlobStatus::Failed => 3
    }
}

impl AbiCall for sol::getPendingBlobsCall {
    type Params = GetPendingBlobsParams;
    type Returns = Vec<BlobRequest>;
    type Output = Result<Vec<u8>, AbiEncodeError>;
    fn params(&self) -> Self::Params {
        GetPendingBlobsParams(self.size)
    }
    fn returns(&self, blob_requests: Self::Returns) -> Self::Output {
        let blob_tuples = blob_requests_to_tuple(blob_requests)?;
        Ok(Self::abi_encode_returns(&(blob_tuples,)))
    }
}

impl AbiCall for sol::getPendingBlobsCountCall {
    type Params = ();
    type Returns = u64;
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        ()
    }
    fn returns(&self, num_resolving: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&(&num_resolving,))
    }
}

impl AbiCall for sol::getPendingBytesCountCall {
    type Params = ();
    type Returns = u64;
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        ()
    }
    fn returns(&self, bytes_resolving: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&(&bytes_resolving,))
    }
}

impl AbiCall for sol::getStorageStatsCall {
    type Params = ();
    type Returns = GetStatsReturn;
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        ()
    }
    fn returns(&self, stats: Self::Returns) -> Self::Output {
        let storage_stats = sol::StorageStats {
            capacityFree: stats.capacity_free,
            capacityUsed: stats.capacity_used,
            numBlobs: stats.num_blobs,
            numResolving: stats.num_resolving,
            numAccounts: stats.num_accounts,
            bytesResolving: stats.bytes_resolving,
            numAdded: stats.num_added,
            bytesAdded: stats.bytes_added,
        };
        Self::abi_encode_returns(&(storage_stats,))
    }
}

impl AbiCall for sol::addBlobCall {
    type Params = Result<AddBlobParams, ActorError>;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        let sponsor = try_decode_option_address(self.params.sponsor)?;
        let source = try_decode_public_key(self.params.source.clone())?;
        let hash = try_decode_hash(self.params.blobHash.clone())?;
        let metadata_hash = try_decode_hash(self.params.metadataHash.clone())?;
        let subscription_id: SubscriptionId = self.params.subscriptionId.clone().try_into()?;
        let size =  self.params.size;
        let ttl = if self.params.ttl.is_zero() { None } else { Some(self.params.ttl as ChainEpoch) };
        let from = try_decode_address(self.params.from)?;
        Ok(AddBlobParams {
            sponsor,
            source,
            hash,
            metadata_hash,
            id: subscription_id,
            size,
            ttl,
            from
        })
    }
    fn returns(&self, _: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&())
    }
}

impl AbiCall for sol::deleteBlobCall {
    type Params = Result<DeleteBlobParams, ActorError>;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        let subscriber = try_decode_option_address(self.subscriber)?;
        let hash = try_decode_hash(self.blobHash.clone())?;
        let subscription_id: SubscriptionId = self.subscriptionId.clone().try_into()?;
        let from = try_decode_address(self.from)?;
        Ok(DeleteBlobParams {
            sponsor: subscriber,
            hash,
            id: subscription_id,
            from
        })
    }
    fn returns(&self, _: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&())
    }
}

/// Blob, but subscribers are ready for EVM.
pub struct BlobTraversed {
    pub size: u64,
    pub metadata_hash: Hash,
    pub status: BlobStatus,
    pub subscribers: Vec<sol::Subscriber>,
}

fn as_illegal_state<E: Display>(err: E) -> ActorError {
    actor_error!(illegal_state; "{}", err)
}

impl BlobTraversed {
    pub fn from_store<T: Blockstore>(store: T, blob: Blob) -> Result<BlobTraversed, ActorError> {
        let subscribers_hamt = blob.subscribers.hamt(&store)?;
        let subscribers = subscribers_hamt.iter().map(|subscriber| {
            let (bytes_key, subscription_group) = subscriber.map_err(as_illegal_state)?;
            let subscriber = Address::from_bytes(bytes_key.as_slice()).map_err(as_illegal_state)?;
            let subscription_group = subscription_group.hamt(&store)?;
            let subscriptions = subscription_group.iter().map(|entry| {
                let (bytes_key, subscription) = entry.map_err(as_illegal_state)?;
                let subscription_id = SubscriptionId::from_bytes(bytes_key.as_slice()).map_err(as_illegal_state)?;
                let subscription = sol::SubscriptionGroup {
                    subscriptionId: subscription_id.into(),
                    subscription: sol::Subscription {
                        added: subscription.added as u64,
                        expiry: subscription.expiry as u64,
                        source: Base32::from_slice(subscription.source.as_ref()).encode(),
                        delegate: subscription.delegate.map(|address| H160::try_from(address)).transpose().map_err(as_illegal_state)?.unwrap_or_default().into(),
                        failed: subscription.failed,
                    }
                };
                Ok(subscription)
            }).collect::<Result<Vec<_>, ActorError>>()?;
            let subscriber_traversed = sol::Subscriber {
                subscriber: H160::try_from(subscriber).map_err(as_illegal_state)?.into(),
                subscriptionGroup: subscriptions,
            };
            Ok(subscriber_traversed)
        }).collect::<Result<Vec<_>, ActorError>>()?;
        let blob = BlobTraversed {
            size: blob.size,
            metadata_hash: blob.metadata_hash,
            status: blob.status,
            subscribers,
        };
        Ok(blob)
    }
}

impl AbiCall for sol::getBlobCall {
    type Params = Result<GetBlobParams, ActorError>;
    type Returns = Option<BlobTraversed>;
    type Output = Result<Vec<u8>, AbiEncodeError>;
    fn params(&self) -> Self::Params {
        let blob_hash = try_decode_hash(self.blobHash.clone())?;
        Ok(GetBlobParams(blob_hash))
    }
    fn returns(&self, blob: Self::Returns) -> Self::Output {
        let blob = if let Some(blob) = blob {
            sol::Blob {
                size: blob.size,
                metadataHash: Base32::from_slice(blob.metadata_hash.as_ref()).encode(),
                status: blob_status_as_solidity_enum(blob.status),
                subscribers: blob.subscribers,
            }
        } else {
            sol::Blob {
                size: 0,
                metadataHash: String::default(),
                status: blob_status_as_solidity_enum(BlobStatus::Failed),
                subscribers: Vec::default(),
            }
        };
        Ok(Self::abi_encode_returns(&(blob,)))
    }
}

// impl AbiCall for sol::addBlobCall {
//     type Params = ();
//     type Returns = ();
//     type Output = ();
//     fn params(&self) -> Self::Params {
//         todo!()
//     }
//     fn returns(&self, returns: Self::Returns) -> Self::Output {
//         todo!()
//     }
// }