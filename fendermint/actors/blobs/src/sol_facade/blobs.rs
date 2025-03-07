// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Error;
use fendermint_actor_blobs_shared::state::{BlobRequest, Hash, PublicKey};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fendermint_actor_blobs_shared::params::GetAccountParams;
use fil_actors_runtime::{actor_error, ActorError};
use recall_actor_sdk::{handle_abi_input, TryIntoEVMEvent};
use recall_sol_facade::blobs as sol;
use recall_sol_facade::primitives::U256;
use recall_sol_facade::types::{Base32String, SolCall, SolInterface, H160};

pub use recall_sol_facade::blobs::Calls;

use crate::sol_facade::{AbiEncodeError, AbiEncodeReturns, TryAbiEncodeReturns};

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

    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
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
    fn try_into_evm_event(self) -> Result<sol::Event, Error> {
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
    fn try_into_evm_event(self) -> Result<sol::Event, Error> {
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
    fn try_into_evm_event(self) -> Result<sol::Event, Error> {
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
handle_abi_input!(Calls);

fn blob_requests_to_tuple(blob_requests: Vec<BlobRequest>) -> Result<Vec<sol::BlobTuple>, anyhow::Error> {
    blob_requests.iter().map(|blob_request| {
        let source_info: Result<Vec<sol::BlobSourceInfo>, anyhow::Error> = blob_request.1.iter().map(|item| {
            let address = item.0;
            let public_key = item.2;
            let subscription_id = item.1.clone();

            Ok(sol::BlobSourceInfo {
                subscriber: H160::try_from(address)?.into(),
                subscriptionId: subscription_id.into(),
                source: Base32String::from(public_key).into(),
            })
        }).collect::<Result<Vec<_>, anyhow::Error>>();

        let blob_hash = blob_request.0;
        Ok(sol::BlobTuple {
            blobHash: Base32String::from(blob_hash).into(),
            sourceInfo: source_info?
        })
    }).collect::<Result<Vec<_>, anyhow::Error>>()
}


impl TryAbiEncodeReturns<Vec<BlobRequest>> for sol::getAddedBlobsCall {
    fn try_returns(&self, blob_requests: Vec<BlobRequest>) -> Result<Vec<u8>, AbiEncodeError> {
        let blob_tuples = blob_requests_to_tuple(blob_requests)?;
        Ok(Self::abi_encode_returns(&(blob_tuples,)))
    }
}

impl AbiEncodeReturns<u64> for sol::getPendingBlobsCountCall {
    fn returns(&self, value: u64) -> Vec<u8> {
        Self::abi_encode_returns(&(&value,))
    }
}

impl AbiEncodeReturns<u64> for sol::getPendingBytesCountCall {
    fn returns(&self, value: u64) -> Vec<u8> {
        Self::abi_encode_returns(&(&value,))
    }
}

// impl Into<GetAccountParams> for sol::getStorageUsageCall {
//     fn into(self) -> GetAccountParams {
//         let address: fvm_shared::address::Address = self.addr.into_eth_address().into();
//         GetAccountParams(address)
//     }
// }

// impl AbiEncodeReturns<Option<u64>> for IBlobsFacade::getStorageUsageCall {
//     fn returns(&self, value: Option<u64>) -> Vec<u8> {
//         let value = value.unwrap_or(u64::default()); // Value or zero, as per Solidity
//         Self::abi_encode_returns(&(U256::from(value),))
//     }
// }
//
// impl AbiEncodeReturns<GetStatsReturn> for IBlobsFacade::getStorageStatsCall {
//     fn returns(&self, stats: GetStatsReturn) -> Vec<u8> {
//         let storage_stats = IBlobsFacade::StorageStats {
//             capacityFree: stats.capacity_free,
//             capacityUsed: stats.capacity_used,
//             numBlobs: stats.num_blobs,
//             numResolving: stats.num_resolving,
//             numAccounts: stats.num_accounts,
//             bytesResolving: stats.bytes_resolving,
//             numAdded: stats.num_added,
//             bytesAdded: stats.bytes_added,
//         };
//         Self::abi_encode_returns(&(storage_stats,))
//     }
// }
//
// fn blob_requests_to_tuple(blob_requests: Vec<BlobRequest>) -> Result<Vec<IBlobsFacade::BlobTuple>, anyhow::Error> {
//     blob_requests.iter().map(|blob_request| {
//         let source_info: Result<Vec<IBlobsFacade::BlobSourceInfo>> = blob_request.1.iter().map(|item| {
//             let address = item.0;
//             let public_key = item.2;
//             let subscription_id = item.1.clone();
//
//             Ok(IBlobsFacade::BlobSourceInfo {
//                 subscriber: H160::try_from(address)?.into(),
//                 subscriptionId: subscription_id.into(),
//                 source: public_key.into()
//             })
//         }).collect::<Result<Vec<_>>>();
//
//         let blob_hash = blob_request.0;
//         Ok::<IBlobsFacade::BlobTuple, anyhow::Error>(IBlobsFacade::BlobTuple {
//             blobHash: blob_hash.into(),
//             sourceInfo: source_info?
//         })
//     }).collect::<Result<Vec<_>>>()
// }
//
// impl TryAbiEncodeReturns<Vec<BlobRequest>> for IBlobsFacade::getAddedBlobsCall {
//     fn try_returns(&self, blob_requests: Vec<BlobRequest>) -> Result<Vec<u8>, AbiEncodeError> {
//         let blob_tuples = blob_requests_to_tuple(blob_requests)?;
//         Ok(Self::abi_encode_returns(&(blob_tuples,)))
//     }
// }
//
// impl TryAbiEncodeReturns<Vec<BlobRequest>> for IBlobsFacade::getPendingBlobsCall {
//     fn try_returns(&self, blob_requests: Vec<BlobRequest>) -> Result<Vec<u8>, AbiEncodeError> {
//         let blob_tuples = blob_requests_to_tuple(blob_requests)?;
//         Ok(Self::abi_encode_returns(&(blob_tuples,)))
//     }
// }
//
// impl TryInto<GetBlobStatusParams> for IBlobsFacade::getBlobStatusCall {
//     type Error = ActorError;
//
//     fn try_into(self) -> std::result::Result<GetBlobStatusParams, Self::Error> {
//         let subscriber = self.subscriber.into_eth_address();
//         let blob_hash: Hash = self.blobHash.try_into().map_err(|e| {
//             actor_error!(serialization, format!("invalid hash value {}", e))
//         })?;
//         let subscription_id: SubscriptionId = self.subscriptionId.clone().try_into()?;
//         Ok(GetBlobStatusParams {
//             subscriber: subscriber.into(),
//             hash: blob_hash,
//             id: subscription_id,
//         })
//     }
// }
//
// impl AbiEncodeReturns<BlobStatus> for IBlobsFacade::getBlobStatusCall {
//     fn returns(&self, blob_status: BlobStatus) -> Vec<u8> {
//         let value = blob_status_as_solidity_enum(blob_status);
//         Self::abi_encode_returns(&(value,))
//     }
// }
//
// fn blob_status_as_solidity_enum(blob_status: BlobStatus) -> u8 {
//     match blob_status {
//         BlobStatus::Added => 0,
//         BlobStatus::Pending => 1,
//         BlobStatus::Resolved => 2,
//         BlobStatus::Failed => 3
//     }
// }
//
// impl AbiEncodeReturns<Option<BlobStatus>> for IBlobsFacade::getBlobStatusCall {
//     fn returns(&self, blob_status: Option<BlobStatus>) -> Vec<u8> {
//         // Use BlobStatus::Failed if None got passed
//         let blob_status = blob_status.unwrap_or(BlobStatus::Failed);
//         self.returns(blob_status)
//     }
// }
//
// impl AbiEncodeReturns<GetStatsReturn> for IBlobsFacade::getSubnetStatsCall {
//     fn returns(&self, stats: GetStatsReturn) -> Vec<u8> {
//         let subnet_stats = IBlobsFacade::SubnetStats {
//             balance: BigUintWrapper::from(stats.balance).into(),
//             capacityFree: stats.capacity_free,
//             capacityUsed: stats.capacity_used,
//             creditSold: BigUintWrapper::from(stats.credit_sold).into(),
//             creditCommitted: BigUintWrapper::from(stats.credit_committed).into(),
//             creditDebited: BigUintWrapper::from(stats.credit_debited).into(),
//             tokenCreditRate: BigUintWrapper(stats.token_credit_rate.rate().clone()).into(),
//             numAccounts: stats.num_accounts,
//             numBlobs: stats.num_blobs,
//             numAdded: stats.num_added,
//             bytesAdded: stats.bytes_added,
//             numResolving: stats.num_resolving,
//             bytesResolving: stats.bytes_resolving,
//         };
//         Self::abi_encode_returns(&(subnet_stats,))
//     }
// }
//
// impl TryInto<GetBlobParams> for IBlobsFacade::getBlobCall {
//     type Error = ActorError;
//
//     fn try_into(self) -> std::result::Result<GetBlobParams, Self::Error> {
//         let blob_hash: Hash = self.blobHash.clone().try_into().map_err(|e| {
//             actor_error!(serialization, format!("invalid hash value {}", e))
//         })?;
//         Ok(GetBlobParams(blob_hash))
//     }
// }
//
// impl TryAbiEncodeReturns<Option<Blob>> for IBlobsFacade::getBlobCall {
//     fn try_returns(&self, value: Option<Blob>) -> Result<Vec<u8>, AbiEncodeError> {
//         let facade_blob = if let Some(blob) = value {
//             let subscribers = blob.subscribers.iter().map(|(fvm_address, subscription_group)| {
//                 let subscription_group = subscription_group.subscriptions.iter().map(|(subscription_id, subscription)| {
//                     let delegate = subscription.delegate.map(|fvm_address| H160::try_from(fvm_address)).transpose()?.unwrap_or_default();
//                     Ok(IBlobsFacade::SubscriptionGroup {
//                        subscriptionId: subscription_id.into(),
//                        subscription: IBlobsFacade::Subscription {
//                            added: subscription.added as u64,
//                            expiry: subscription.expiry as u64,
//                            source: subscription.source.into(),
//                            delegate: delegate.into(),
//                            failed: subscription.failed,
//                        },
//                    })
//                 }).collect::<Result<Vec<_>, anyhow::Error>>()?;
//                 let fvm_address = FVMAddress::from_str(fvm_address)?;
//                 let h160_address: H160 = fvm_address.try_into()?;
//                 Ok(IBlobsFacade::Subscriber {
//                     subscriber: h160_address.into(),
//                     subscriptionGroup: subscription_group,
//                 })
//             }).collect::<Result<Vec<_>>>()?;
//             IBlobsFacade::Blob {
//                 size: blob.size,
//                 metadataHash: blob.metadata_hash.into(),
//                 status: blob_status_as_solidity_enum(blob.status),
//                 subscribers,
//             }
//         } else {
//             IBlobsFacade::Blob {
//                 size: 0,
//                 metadataHash: Hash::default().into(),
//                 status: blob_status_as_solidity_enum(BlobStatus::Failed),
//                 subscribers: vec![]
//             }
//         };
//         Ok(Self::abi_encode_returns(&(facade_blob,)))
//     }
// }
//
// impl TryInto<AddBlobParams> for IBlobsFacade::addBlobCall {
//     type Error = ActorError;
//
//     fn try_into(self) -> std::result::Result<AddBlobParams, Self::Error> {
//         let sponsor: EthAddress = self.params.sponsor.into_eth_address();
//         let sponsor: Option<FVMAddress> = if sponsor.is_null() { None } else { Some(sponsor.into()) };
//         let source = try_into_public_key(self.params.source.clone())?;
//         let hash = try_into_hash(self.params.blobHash)?;
//         let metadata_hash = try_into_hash(self.params.metadataHash)?;
//         let subscription_id: SubscriptionId = self.params.subscriptionId.clone().try_into()?;
//         let size =  self.params.size;
//         let ttl = if self.params.ttl.is_zero() { None } else { Some(self.params.ttl as ChainEpoch) };
//         let from: FVMAddress = self.params.from.into_eth_address().into();
//         Ok(AddBlobParams {
//             sponsor,
//             source,
//             hash,
//             metadata_hash,
//             id: subscription_id,
//             size,
//             ttl,
//             from
//         })
//     }
// }
//
// impl TryInto<DeleteBlobParams> for IBlobsFacade::deleteBlobCall {
//     type Error = ActorError;
//     fn try_into(self) -> std::result::Result<DeleteBlobParams, Self::Error> {
//         let subscriber: EthAddress = self.subscriber.into_eth_address();
//         let subscriber: Option<fvm_shared::address::Address> = if subscriber.is_null() { None } else { Some(subscriber.into()) };
//         let hash: Hash = try_into_hash(self.blobHash)?;
//         let subscription_id: SubscriptionId = self.subscriptionId.clone().try_into()?;
//         let from: fvm_shared::address::Address = self.from.into_eth_address().into();
//         Ok(DeleteBlobParams {
//             sponsor: subscriber,
//             hash,
//             id: subscription_id,
//             from
//         })
//     }
// }
//
// impl TryInto<OverwriteBlobParams> for IBlobsFacade::overwriteBlobCall {
//     type Error = ActorError;
//
//     fn try_into(self) -> std::result::Result<OverwriteBlobParams, Self::Error> {
//         let old_hash: Hash = try_into_hash(self.oldHash)?;
//         let sponsor: EthAddress = self.params.sponsor.into_eth_address();
//         let sponsor: Option<fvm_shared::address::Address> = if sponsor.is_null() { None } else { Some(sponsor.into()) };
//         let source: PublicKey = try_into_public_key(self.params.source)?;
//         let hash: Hash = try_into_hash(self.params.blobHash)?;
//         let metadata_hash: Hash = try_into_hash(self.params.metadataHash)?;
//         let subscription_id: SubscriptionId = self.params.subscriptionId.clone().try_into()?;
//         let size =  self.params.size;
//         let ttl = if self.params.ttl.is_zero() { None } else { Some(self.params.ttl as ChainEpoch) };
//         let from: fvm_shared::address::Address = self.params.from.into_eth_address().into();
//         Ok(OverwriteBlobParams {
//             old_hash,
//             add: AddBlobParams {
//                 sponsor,
//                 source,
//                 hash,
//                 metadata_hash,
//                 id: subscription_id,
//                 size,
//                 ttl,
//                 from,
//             },
//         })
//     }
// }