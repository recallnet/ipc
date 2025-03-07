// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_blockstore::Blockstore;
use fendermint_actor_blobs_shared::state::{Blob, BlobRequest, BlobStatus, Hash, PublicKey, SubscriptionId};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fendermint_actor_blobs_shared::params::{AddBlobParams, DeleteBlobParams, GetAccountParams, GetAddedBlobsParams, GetBlobParams, GetBlobStatusParams, GetPendingBlobsParams, GetStatsReturn, OverwriteBlobParams};
use fil_actors_runtime::{actor_error, ActorError};
use recall_actor_sdk::{TryIntoEVMEvent};
use recall_sol_facade::blobs as sol;
use recall_sol_facade::primitives::U256;
use recall_sol_facade::types::{BigUintWrapper, SolCall, SolInterface, H160};
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
            let subscription_id = item.1.clone();
            let public_key = item.2;

            Ok(sol::BlobSourceInfo {
                subscriber: H160::try_from(address)?.into(),
                subscriptionId: subscription_id.into(),
                source: public_key.into(),
            })
        }).collect::<Result<Vec<_>, anyhow::Error>>();

        let blob_hash = blob_request.0;
        Ok(sol::BlobTuple {
            blobHash: blob_hash.into(),
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

impl AbiCall for sol::getBlobStatusCall {
    type Params = Result<GetBlobStatusParams, AbiEncodeError>;
    type Returns = Option<BlobStatus>;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let subscriber: Address = H160::from(self.subscriber).into();
        let blob_hash: Hash = Hash::try_from(self.blobHash.clone())?;
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
    type Params = Result<AddBlobParams, AbiEncodeError>;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        let sponsor: Option<Address> = H160::from(self.params.sponsor).as_option().map(|a| a.into());
        let source = PublicKey::try_from(self.params.source.clone())?;
        let hash = Hash::try_from(self.params.blobHash.clone())?;
        let metadata_hash = Hash::try_from(self.params.metadataHash.clone())?;
        let subscription_id: SubscriptionId = self.params.subscriptionId.clone().try_into()?;
        let size =  self.params.size;
        let ttl = if self.params.ttl.is_zero() { None } else { Some(self.params.ttl as ChainEpoch) };
        let from: Address = H160::from(self.params.from).into();
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
    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

impl AbiCall for sol::deleteBlobCall {
    type Params = Result<DeleteBlobParams, AbiEncodeError>;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        let subscriber = H160::from(self.subscriber).as_option().map(|a| a.into());
        let hash = Hash::try_from(self.blobHash.clone())?;
        let subscription_id: SubscriptionId = self.subscriptionId.clone().try_into()?;
        let from: Address = H160::from(self.from).into();
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

impl BlobTraversed {
    pub fn from_store<T: Blockstore>(store: T, blob: Blob) -> Result<BlobTraversed, AbiEncodeError> {
        let subscribers_hamt = blob.subscribers.hamt(&store)?;
        let subscribers = subscribers_hamt.iter().map(|subscriber| {
            let (bytes_key, subscription_group) = subscriber.map_err(anyhow::Error::msg)?;
            let subscriber = Address::from_bytes(bytes_key.as_slice()).map_err(anyhow::Error::msg)?;
            let subscription_group = subscription_group.hamt(&store)?;
            let subscriptions = subscription_group.iter().map(|entry| {
                let (bytes_key, subscription) = entry.map_err(anyhow::Error::msg)?;
                let subscription_id = SubscriptionId::from_bytes(bytes_key.as_slice())?;
                let subscription = sol::SubscriptionGroup {
                    subscriptionId: subscription_id.into(),
                    subscription: sol::Subscription {
                        added: subscription.added as u64,
                        expiry: subscription.expiry as u64,
                        source: subscription.source.into(),
                        delegate: subscription.delegate.map(|address| H160::try_from(address)).transpose()?.unwrap_or_default().into(),
                        failed: subscription.failed,
                    }
                };
                Ok(subscription)
            }).collect::<Result<Vec<_>, AbiEncodeError>>()?;
            let subscriber_traversed = sol::Subscriber {
                subscriber: H160::try_from(subscriber)?.into(),
                subscriptionGroup: subscriptions,
            };
            Ok(subscriber_traversed)
        }).collect::<Result<Vec<_>, AbiEncodeError>>()?;
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
    type Params = Result<GetBlobParams, AbiEncodeError>;
    type Returns = Option<BlobTraversed>;
    type Output = Result<Vec<u8>, AbiEncodeError>;
    fn params(&self) -> Self::Params {
        let blob_hash = Hash::try_from(self.blobHash.clone())?;
        Ok(GetBlobParams(blob_hash))
    }
    fn returns(&self, blob: Self::Returns) -> Self::Output {
        let blob = if let Some(blob) = blob {
            sol::Blob {
                size: blob.size,
                metadataHash: blob.metadata_hash.into(),
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

impl AbiCall for sol::getStorageUsageCall {
    type Params = GetAccountParams;
    type Returns = Option<u64>;
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        let address: Address = H160::from(self.addr).into();
        GetAccountParams(address)
    }
    fn returns(&self, capacity_used: Self::Returns) -> Self::Output {
        let capacity_used = capacity_used.unwrap_or_default();
        Self::abi_encode_returns(&(U256::from(capacity_used),))
    }
}

impl AbiCall for sol::getSubnetStatsCall {
    type Params = ();
    type Returns = GetStatsReturn;
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        ()
    }
    fn returns(&self, stats: Self::Returns) -> Self::Output {
        let subnet_stats = sol::SubnetStats {
            balance: BigUintWrapper::from(stats.balance).into(),
            capacityFree: stats.capacity_free,
            capacityUsed: stats.capacity_used,
            creditSold: BigUintWrapper::from(stats.credit_sold).into(),
            creditCommitted: BigUintWrapper::from(stats.credit_committed).into(),
            creditDebited: BigUintWrapper::from(stats.credit_debited).into(),
            tokenCreditRate: BigUintWrapper(stats.token_credit_rate.rate().clone()).into(),
            numAccounts: stats.num_accounts,
            numBlobs: stats.num_blobs,
            numAdded: stats.num_added,
            bytesAdded: stats.bytes_added,
            numResolving: stats.num_resolving,
            bytesResolving: stats.bytes_resolving,
        };
        Self::abi_encode_returns(&(subnet_stats,))
    }
}

impl AbiCall for sol::overwriteBlobCall {
    type Params = Result<OverwriteBlobParams, AbiEncodeError>;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        let old_hash: Hash = Hash::try_from(self.oldHash.clone())?;
        let sponsor = H160::from(self.params.sponsor).as_option().map(|a| a.into());
        let source: PublicKey = PublicKey::try_from(self.params.source.clone())?;
        let hash: Hash = Hash::try_from(self.params.blobHash.clone())?;
        let metadata_hash: Hash = Hash::try_from(self.params.metadataHash.clone())?;
        let subscription_id: SubscriptionId = self.params.subscriptionId.clone().try_into()?;
        let size =  self.params.size;
        let ttl = if self.params.ttl.is_zero() { None } else { Some(self.params.ttl as ChainEpoch) };
        let from: Address = H160::from(self.params.from.clone()).into();
        Ok(OverwriteBlobParams {
            old_hash,
            add: AddBlobParams {
                sponsor,
                source,
                hash,
                metadata_hash,
                id: subscription_id,
                size,
                ttl,
                from,
            },
        })
    }
    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}