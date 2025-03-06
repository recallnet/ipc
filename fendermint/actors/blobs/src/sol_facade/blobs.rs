// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::params::{
    AddBlobParams, DeleteBlobParams, GetBlobParams, GetStatsReturn, OverwriteBlobParams,
    TrimBlobExpiriesParams,
};
use fendermint_actor_blobs_shared::state::{Blob, BlobStatus, Hash, PublicKey, SubscriptionId};
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{actor_error, ActorError};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use num_traits::Zero;
use recall_actor_sdk::TryIntoEVMEvent;
use recall_ipld::hamt::MapKey;
use recall_sol_facade::blobs as sol;
use recall_sol_facade::primitives::U256;
use recall_sol_facade::types::{BigUintWrapper, SolCall, SolInterface, H160};

pub use recall_sol_facade::blobs::Calls;

use crate::sol_facade::{AbiCall, AbiCallRuntime, AbiEncodeError};

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
    Calls::abi_decode_raw(input.selector(), input.calldata(), true)
        .map_err(|e| actor_error!(illegal_argument, format!("invalid call: {}", e)))
}

fn blob_status_as_solidity_enum(blob_status: BlobStatus) -> u8 {
    match blob_status {
        BlobStatus::Added => 0,
        BlobStatus::Pending => 1,
        BlobStatus::Resolved => 2,
        BlobStatus::Failed => 3,
    }
}

impl AbiCallRuntime for sol::addBlobCall {
    type Params = Result<AddBlobParams, AbiEncodeError>;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let sponsor: Option<Address> = H160::from(self.params.sponsor)
            .as_option()
            .map(|a| a.into());
        let source = PublicKey(self.params.source.into());
        let hash = Hash(self.params.blobHash.into());
        let metadata_hash = Hash(self.params.metadataHash.into());
        let subscription_id: SubscriptionId = self.params.subscriptionId.clone().try_into()?;
        let size = self.params.size;
        let ttl = if self.params.ttl.is_zero() {
            None
        } else {
            Some(self.params.ttl as ChainEpoch)
        };
        let from: Address = rt.message().caller();
        Ok(AddBlobParams {
            sponsor,
            source,
            hash,
            metadata_hash,
            id: subscription_id,
            size,
            ttl,
            from,
        })
    }
    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

impl AbiCallRuntime for sol::deleteBlobCall {
    type Params = Result<DeleteBlobParams, AbiEncodeError>;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let subscriber = H160::from(self.subscriber).as_option().map(|a| a.into());
        let hash = Hash(self.blobHash.into());
        let subscription_id: SubscriptionId = self.subscriptionId.clone().try_into()?;
        let from: Address = rt.message().caller();
        Ok(DeleteBlobParams {
            sponsor: subscriber,
            hash,
            id: subscription_id,
            from,
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
    pub fn from_store<T: Blockstore>(
        store: T,
        blob: Blob,
    ) -> Result<BlobTraversed, AbiEncodeError> {
        let subscribers_hamt = blob.subscribers.hamt(&store)?;
        let subscribers = subscribers_hamt
            .iter()
            .map(|subscriber| {
                let (bytes_key, subscription_group) = subscriber.map_err(anyhow::Error::msg)?;
                let subscriber =
                    Address::from_bytes(bytes_key.as_slice()).map_err(anyhow::Error::msg)?;
                let subscription_group = subscription_group.hamt(&store)?;
                let subscriptions = subscription_group
                    .iter()
                    .map(|entry| {
                        let (bytes_key, subscription) = entry.map_err(anyhow::Error::msg)?;
                        let subscription_id = SubscriptionId::from_bytes(bytes_key.as_slice())?;
                        let subscription = sol::SubscriptionGroup {
                            subscriptionId: subscription_id.into(),
                            subscription: sol::Subscription {
                                added: subscription.added as u64,
                                expiry: subscription.expiry as u64,
                                source: subscription.source.0.into(),
                                delegate: subscription
                                    .delegate
                                    .map(|address| H160::try_from(address))
                                    .transpose()?
                                    .unwrap_or_default()
                                    .into(),
                                failed: subscription.failed,
                            },
                        };
                        Ok(subscription)
                    })
                    .collect::<Result<Vec<_>, AbiEncodeError>>()?;
                let subscriber_traversed = sol::Subscriber {
                    subscriber: H160::try_from(subscriber)?.into(),
                    subscriptionGroup: subscriptions,
                };
                Ok(subscriber_traversed)
            })
            .collect::<Result<Vec<_>, AbiEncodeError>>()?;
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
        let blob_hash: Hash = Hash(self.blobHash.into());
        Ok(GetBlobParams(blob_hash))
    }
    fn returns(&self, blob: Self::Returns) -> Self::Output {
        let blob = if let Some(blob) = blob {
            sol::Blob {
                size: blob.size,
                metadataHash: blob.metadata_hash.0.into(),
                status: blob_status_as_solidity_enum(blob.status),
                subscribers: blob.subscribers,
            }
        } else {
            sol::Blob {
                size: 0,
                metadataHash: [0u8; 32].into(),
                status: blob_status_as_solidity_enum(BlobStatus::Failed),
                subscribers: Vec::default(),
            }
        };
        Ok(Self::abi_encode_returns(&(blob,)))
    }
}

impl AbiCall for sol::getStatsCall {
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

impl AbiCallRuntime for sol::overwriteBlobCall {
    type Params = Result<OverwriteBlobParams, AbiEncodeError>;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let old_hash = Hash(self.oldHash.into());
        let sponsor = H160::from(self.params.sponsor)
            .as_option()
            .map(|a| a.into());
        let source: PublicKey = PublicKey(self.params.source.into());
        let hash: Hash = Hash(self.params.blobHash.into());
        let metadata_hash: Hash = Hash(self.params.metadataHash.into());
        let subscription_id: SubscriptionId = self.params.subscriptionId.clone().try_into()?;
        let size = self.params.size;
        let ttl = if self.params.ttl.is_zero() {
            None
        } else {
            Some(self.params.ttl as ChainEpoch)
        };
        let from: Address = rt.message().caller();
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

impl AbiCall for sol::trimBlobExpiriesCall {
    type Params = TrimBlobExpiriesParams;
    type Returns = (u32, Option<Hash>);
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let limit = self.params.limit;
        let limit = if limit.is_zero() { None } else { Some(limit) };
        let hash: [u8; 32] = self.params.startingHash.into();
        let hash = if hash == [0; 32] {
            None
        } else {
            Some(Hash(hash))
        };
        TrimBlobExpiriesParams {
            subscriber: H160::from(self.params.subscriber).into(),
            limit,
            starting_hash: hash,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let next_key = returns.1;
        let next_key = next_key.unwrap_or_default();
        let cursor = sol::TrimBlobExpiries {
            processed: returns.0,
            nextKey: next_key.0.into(),
        };
        Self::abi_encode_returns(&(cursor,))
    }
}
