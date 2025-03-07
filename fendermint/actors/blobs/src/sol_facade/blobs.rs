// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::{BlobRequest, BlobStatus, Hash, PublicKey, SubscriptionId};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fendermint_actor_blobs_shared::params::{GetAddedBlobsParams, GetBlobStatusParams, GetPendingBlobsParams, GetStatsReturn};
use fil_actors_runtime::{actor_error, ActorError};
use recall_actor_sdk::{TryIntoEVMEvent};
use recall_sol_facade::blobs as sol;
use recall_sol_facade::primitives::U256;
use recall_sol_facade::types::{base32, SolCall, SolInterface, H160};

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

pub fn parse_input(input: &recall_actor_sdk::InputData) -> Result<Calls, fil_actors_runtime::ActorError> {
    Calls::abi_decode_raw(input.selector(), input.calldata(), true).map_err(|e| {
        fil_actors_runtime::actor_error!(illegal_argument, format!("invalid call: {}", e))
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

impl AbiCall for sol::getBlobStatusCall {
    type Params = Result<GetBlobStatusParams, ActorError>;
    type Returns = Option<BlobStatus>;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let subscriber: Address = H160::from(self.subscriber).try_into().map_err(|_| {
            actor_error!(illegal_argument, "invalid subscriber param".to_string())
        })?;
        let blob_hash: Hash = base32::decode(self.blobHash.as_bytes()).and_then(|vec| Hash::try_from(vec).map_err(anyhow::Error::msg)).map_err(|_| {
            actor_error!(illegal_argument, "invalid hash param".to_string())
        })?;
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