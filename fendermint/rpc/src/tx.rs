// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::marker::PhantomData;

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use bytes::Bytes;
use fendermint_vm_message::query::{FvmQueryHeight, GasEstimate};
use tendermint::abci::response::DeliverTx;
use tendermint_rpc::endpoint::broadcast::{tx_async, tx_commit, tx_sync};

use fendermint_actor_accumulator::PushReturn;
use fendermint_actor_machine::{Metadata, WriteAccess};
use fendermint_actor_objectstore::{
    DeleteParams, GetParams, ListParams, Object, ObjectList, PutParams,
};
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;

use fendermint_vm_actor_interface::{adm, eam};
use fendermint_vm_message::chain::ChainMessage;

use crate::message::{GasParams, SignedMessageFactory};
use crate::query::{QueryClient, QueryResponse};
use crate::response::{
    decode_acc_get_at, decode_acc_push_return, decode_bytes, decode_cid_string, decode_fevm_create,
    decode_fevm_invoke, decode_machine_create, decode_machine_get, decode_machine_list,
    decode_os_get, decode_os_list,
};

/// Abstracting away what the return value is based on whether
/// we broadcast transactions in sync, async or commit mode.
pub trait BroadcastMode {
    type Response<T>;
}

pub trait BoundClient {
    fn message_factory_mut(&mut self) -> &mut SignedMessageFactory;

    fn address(&mut self) -> Address {
        *self.message_factory_mut().address()
    }
}

/// Fendermint client for submitting transactions.
#[async_trait]
pub trait TxClient<M: BroadcastMode = TxCommit>: BoundClient + Send + Sync {
    /// Transfer tokens to another account.
    async fn transfer(
        &mut self,
        to: Address,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<M::Response<()>> {
        let mf = self.message_factory_mut();
        let msg = mf.transfer(to, value, gas_params)?;
        let fut = self.perform(msg, |_| Ok(()));
        let res = fut.await?;
        Ok(res)
    }

    /// Send a message to an actor.
    async fn transaction(
        &mut self,
        to: Address,
        method_num: MethodNum,
        params: RawBytes,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<M::Response<RawBytes>> {
        let mf = self.message_factory_mut();
        let msg = mf.transaction(to, method_num, params, value, gas_params, None)?;
        let fut = self.perform(msg, decode_bytes);
        let res = fut.await?;
        Ok(res)
    }

    /// Create an object store.
    async fn os_create(
        &mut self,
        write_access: WriteAccess,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<M::Response<adm::CreateExternalReturn>> {
        let mf = self.message_factory_mut();
        let msg = mf.os_create(write_access, value, gas_params)?;
        let fut = self.perform(msg, decode_machine_create);
        let res = fut.await?;
        Ok(res)
    }

    /// Put an object into an object store.
    async fn os_put(
        &mut self,
        address: Address,
        params: PutParams,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<M::Response<String>> {
        let mf = self.message_factory_mut();
        let msg = mf.os_put(address, params, value, gas_params)?;
        let fut = self.perform(msg, decode_cid_string);
        let res = fut.await?;
        Ok(res)
    }

    /// Delete an object from an object store.
    async fn os_delete(
        &mut self,
        address: Address,
        params: DeleteParams,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<M::Response<String>> {
        let mf = self.message_factory_mut();
        let msg = mf.os_delete(address, params, value, gas_params)?;
        let fut = self.perform(msg, decode_cid_string);
        let res = fut.await?;
        Ok(res)
    }

    /// Create an accumulator.
    async fn acc_create(
        &mut self,
        write_access: WriteAccess,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<M::Response<adm::CreateExternalReturn>> {
        let mf = self.message_factory_mut();
        let msg = mf.acc_create(write_access, value, gas_params)?;
        let fut = self.perform(msg, decode_machine_create);
        let res = fut.await?;
        Ok(res)
    }

    /// Push an event to an accumulator.
    async fn acc_push(
        &mut self,
        address: Address,
        event: Bytes,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<M::Response<PushReturn>> {
        let mf = self.message_factory_mut();
        let msg = mf.acc_push(address, event, value, gas_params)?;
        let fut = self.perform(msg, decode_acc_push_return);
        let res = fut.await?;
        Ok(res)
    }

    /// Deploy a FEVM contract.
    async fn fevm_create(
        &mut self,
        contract: Bytes,
        constructor_args: Bytes,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<M::Response<eam::CreateReturn>> {
        let mf = self.message_factory_mut();
        let msg = mf.fevm_create(contract, constructor_args, value, gas_params)?;
        let fut = self.perform(msg, decode_fevm_create);
        let res = fut.await?;
        Ok(res)
    }

    /// Invoke a method on a FEVM contract.
    async fn fevm_invoke(
        &mut self,
        contract: Address,
        calldata: Bytes,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<M::Response<Vec<u8>>> {
        let mf = self.message_factory_mut();
        let msg = mf.fevm_invoke(contract, calldata, value, gas_params)?;
        let fut = self.perform(msg, decode_fevm_invoke);
        let res = fut.await?;
        Ok(res)
    }

    async fn perform<F, T>(&self, msg: ChainMessage, f: F) -> anyhow::Result<M::Response<T>>
    where
        F: FnOnce(&DeliverTx) -> anyhow::Result<T> + Sync + Send,
        T: Sync + Send;
}

/// Convenience trait to call FEVM methods in read-only mode, without doing a transaction.
#[async_trait]
pub trait CallClient: QueryClient + BoundClient {
    /// List machine metadata by owner.
    async fn list_machines_call(
        &mut self,
        owner: Address,
        value: TokenAmount,
        gas_params: GasParams,
        height: FvmQueryHeight,
    ) -> anyhow::Result<CallResponse<Vec<adm::Metadata>>> {
        let msg = self
            .message_factory_mut()
            .list_machines(owner, value, gas_params)?;

        let response = self.call(msg, height).await?;
        if response.value.code.is_err() {
            return Err(anyhow!("{}", response.value.info));
        }
        let return_data = decode_machine_list(&response.value)
            .context("error decoding data from deliver_tx in call")?;

        let response = CallResponse {
            response,
            return_data: Some(return_data),
        };

        Ok(response)
    }

    /// Get machine metadata.
    async fn get_machine_call(
        &mut self,
        address: Address,
        value: TokenAmount,
        gas_params: GasParams,
        height: FvmQueryHeight,
    ) -> anyhow::Result<CallResponse<Metadata>> {
        let msg = self
            .message_factory_mut()
            .get_machine(address, value, gas_params)?;

        let response = self.call(msg, height).await?;
        if response.value.code.is_err() {
            return Err(anyhow!("{}", response.value.info));
        }
        let return_data = decode_machine_get(&response.value)
            .context("error decoding data from deliver_tx in call")?;

        let response = CallResponse {
            response,
            return_data: Some(return_data),
        };

        Ok(response)
    }

    /// Get an object in an object store without including a transaction on the blockchain.
    async fn os_get_call(
        &mut self,
        address: Address,
        params: GetParams,
        value: TokenAmount,
        gas_params: GasParams,
        height: FvmQueryHeight,
    ) -> anyhow::Result<CallResponse<Object>> {
        let msg = self
            .message_factory_mut()
            .os_get(address, params, value, gas_params)?;

        let response = self.call(msg, height).await?;
        if response.value.code.is_err() {
            return Err(anyhow!("{}", response.value.info));
        }
        let return_data = decode_os_get(&response.value)
            .context("error decoding data from deliver_tx in call")?;

        let response = CallResponse {
            response,
            return_data,
        };

        Ok(response)
    }

    /// List objects in an object store without including a transaction on the blockchain.
    async fn os_list_call(
        &mut self,
        address: Address,
        params: ListParams,
        value: TokenAmount,
        gas_params: GasParams,
        height: FvmQueryHeight,
    ) -> anyhow::Result<CallResponse<ObjectList>> {
        let msg = self
            .message_factory_mut()
            .os_list(address, params, value, gas_params)?;

        let response = self.call(msg, height).await?;
        if response.value.code.is_err() {
            return Err(anyhow!("{}", response.value.info));
        }
        let return_data = decode_os_list(&response.value)
            .context("error decoding data from deliver_tx in call")?;

        let response = CallResponse {
            response,
            return_data: Some(return_data),
        };

        Ok(response)
    }

    /// Get object at the given index in an accumulator without including a transaction on the
    /// blockchain.
    async fn acc_get_at_call(
        &mut self,
        address: Address,
        value: TokenAmount,
        gas_params: GasParams,
        height: FvmQueryHeight,
        index: u64,
    ) -> anyhow::Result<CallResponse<Vec<u8>>> {
        let msg = self
            .message_factory_mut()
            .acc_get_at(address, value, gas_params, index)?;

        let response = self.call(msg, height).await?;
        if response.value.code.is_err() {
            return Err(anyhow!("{}", response.value.info));
        }

        let return_data = decode_acc_get_at(&response.value)
            .context("error decoding data from deliver_tx in call")?;

        let response = CallResponse {
            response,
            return_data,
        };

        Ok(response)
    }

    /// Get root commitment in an accumulator without including a transaction on the blockchain.
    async fn acc_root_call(
        &mut self,
        address: Address,
        value: TokenAmount,
        gas_params: GasParams,
        height: FvmQueryHeight,
    ) -> anyhow::Result<CallResponse<String>> {
        let msg = self
            .message_factory_mut()
            .acc_root(address, value, gas_params)?;

        let response = self.call(msg, height).await?;
        if response.value.code.is_err() {
            return Err(anyhow!("{}", response.value.info));
        }
        let root = decode_cid_string(&response.value)
            .context("error decoding data from deliver_tx in call")?;

        let response = CallResponse {
            response,
            return_data: Some(root),
        };

        Ok(response)
    }

    /// Call a method on a FEVM contract without including a transaction on the blockchain.
    async fn fevm_call(
        &mut self,
        contract: Address,
        calldata: Bytes,
        value: TokenAmount,
        gas_params: GasParams,
        height: FvmQueryHeight,
    ) -> anyhow::Result<CallResponse<Vec<u8>>> {
        let msg = self
            .message_factory_mut()
            .fevm_call(contract, calldata, value, gas_params)?;

        let response = self.call(msg, height).await?;

        let return_data = if response.value.code.is_err() {
            None
        } else {
            let return_data = decode_fevm_invoke(&response.value)
                .context("error decoding data from deliver_tx in query")?;
            Some(return_data)
        };

        let response = CallResponse {
            response,
            return_data,
        };

        Ok(response)
    }

    /// Estimate the gas limit of a FEVM call.
    async fn fevm_estimate_gas(
        &mut self,
        contract: Address,
        calldata: Bytes,
        value: TokenAmount,
        gas_params: GasParams,
        height: FvmQueryHeight,
    ) -> anyhow::Result<QueryResponse<GasEstimate>> {
        let msg = self
            .message_factory_mut()
            .fevm_call(contract, calldata, value, gas_params)?;

        self.estimate_gas(msg, height).await
    }
}

/// Auto-implement this trait for anything that satisfies the bounds.
impl<C> CallClient for C where C: QueryClient + BoundClient + Send + Sync {}

/// Return immediately after the transaction is broadcasted without waiting for check results.
pub struct TxAsync;

/// Wait for the check results before returning from broadcast.
pub struct TxSync;

/// Wait for the delivery results before returning from broadcast.
pub struct TxCommit;

pub struct AsyncResponse<T> {
    /// Response from Tendermint.
    pub response: tx_async::Response,
    pub return_data: PhantomData<T>,
}

pub struct SyncResponse<T> {
    /// Response from Tendermint.
    pub response: tx_sync::Response,
    pub return_data: PhantomData<T>,
}

pub struct CommitResponse<T> {
    /// Response from Tendermint.
    pub response: tx_commit::Response,
    /// Parsed return data, if the response indicates success.
    pub return_data: Option<T>,
}

pub struct CallResponse<T> {
    /// Response from Tendermint.
    pub response: QueryResponse<DeliverTx>,
    /// Parsed return data, if the response indicates success.
    pub return_data: Option<T>,
}

impl BroadcastMode for TxAsync {
    type Response<T> = AsyncResponse<T>;
}

impl BroadcastMode for TxSync {
    type Response<T> = SyncResponse<T>;
}

impl BroadcastMode for TxCommit {
    type Response<T> = CommitResponse<T>;
}
