// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;
use std::sync::Arc;

use crate::selector::{GasLimitSelector, MessageSelector};
use crate::{
    fvm::state::ipc::GatewayCaller,
    fvm::state::FvmExecState,
    fvm::store::ReadOnlyBlockstore,
    fvm::FvmMessage,
    fvm::{topdown, BlockGasLimit, FvmApplyRet, PowerUpdates},
    signed::{SignedMessageApplyRes, SignedMessageCheckRes, SyntheticMessage, VerifiableMessage},
    CheckInterpreter, ExecInterpreter, ProposalInterpreter, QueryInterpreter,
};
use anyhow::{anyhow, bail, Context};
use async_stm::atomically;
use async_trait::async_trait;
use fendermint_actor_blobs_shared::{
    params::{
        FinalizeBlobParams, GetAddedBlobsParams, GetBlobStatusParams, GetStatsReturn,
        SetBlobPendingParams,
    },
    state::{BlobStatus, SubscriptionId},
    Method::{DebitAccounts, FinalizeBlob, GetAddedBlobs, GetBlobStatus, GetStats, SetBlobPending},
};
use fendermint_actor_readreq::{
    CloseReadRequestParams, GetOpenReadRequestsParams, GetReadRequestStatusParams,
    Method::{CloseReadRequest, GetOpenReadRequests, GetReadRequestStatus, SetReadRequestPending},
    ReadRequestStatus, SetReadRequestPendingParams,
};
use fendermint_tracing::emit;
use fendermint_vm_actor_interface::{blobs, ipc, readreq, system};
use fendermint_vm_event::ParentFinalityMissingQuorum;
use fendermint_vm_iroh_resolver::observe::{
    BlobsFinalityAddedBlobs, BlobsFinalityAddedBytes, BlobsFinalityPendingBlobs,
    BlobsFinalityPendingBytes,
};
use fendermint_vm_iroh_resolver::pool::{
    ResolveKey as IrohResolveKey, ResolvePool as IrohResolvePool,
    ResolveSource as IrohResolveSource, TaskType as IrohTaskType,
};
use fendermint_vm_message::ipc::{
    ClosedReadRequest, FinalizedBlob, ParentFinality, PendingBlob, PendingReadRequest,
};
use fendermint_vm_message::{
    chain::ChainMessage,
    ipc::{BottomUpCheckpoint, CertifiedMessage, IpcMessage, SignedRelayedMessage},
};
use fendermint_vm_resolver::pool::{ResolveKey, ResolvePool};
use fendermint_vm_topdown::proxy::IPCProviderProxyWithLatency;
use fendermint_vm_topdown::voting::{ValidatorKey, VoteTally};
use fendermint_vm_topdown::{
    CachedFinalityProvider, IPCParentFinality, ParentFinalityProvider, ParentViewProvider, Toggle,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use fvm_shared::MethodNum;
use iroh::base::key::PublicKey;
use iroh::blobs::Hash;
use iroh::net::NodeId;
use num_traits::Zero;
use tokio_util::bytes;

/// A resolution pool for bottom-up and top-down checkpoints.
pub type CheckpointPool = ResolvePool<CheckpointPoolItem>;
pub type TopDownFinalityProvider = Arc<Toggle<CachedFinalityProvider<IPCProviderProxyWithLatency>>>;
pub type BlobPool = IrohResolvePool<BlobPoolItem>;
pub type ReadRequestPool = IrohResolvePool<ReadRequestPoolItem>;

type AddedBlobItem = (Hash, HashSet<(Address, SubscriptionId, PublicKey)>);
type OpenReadRequestItem = (Hash, Hash, u32, u32, Address, MethodNum);

/// These are the extra state items that the chain interpreter needs,
/// a sort of "environment" supporting IPC.
#[derive(Clone)]
pub struct ChainEnv {
    /// CID resolution pool.
    pub checkpoint_pool: CheckpointPool,
    /// The parent finality provider for top-down checkpoint
    pub parent_finality_provider: TopDownFinalityProvider,
    pub parent_finality_votes: VoteTally,
    /// Iroh blob resolution pool.
    pub blob_pool: BlobPool,
    /// Number of pending blobs to process in parallel.
    pub blob_concurrency: u32,
    /// Read request pool.
    pub read_request_pool: ReadRequestPool,
    /// Number of pending read requests to process in parallel.
    pub read_request_concurrency: u32,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum CheckpointPoolItem {
    /// BottomUp checkpoints to be resolved from the originating subnet or the current one.
    BottomUp(CertifiedMessage<BottomUpCheckpoint>),
    // We can extend this to include top-down checkpoints as well, with slightly
    // different resolution semantics (resolving it from a trusted parent, and
    // awaiting finality before declaring it available).
}

impl From<&CheckpointPoolItem> for ResolveKey {
    fn from(value: &CheckpointPoolItem) -> Self {
        match value {
            CheckpointPoolItem::BottomUp(cp) => {
                (cp.message.subnet_id.clone(), cp.message.bottom_up_messages)
            }
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct BlobPoolItem {
    subscriber: Address,
    hash: Hash,
    id: SubscriptionId,
    source: NodeId,
}

impl From<&BlobPoolItem> for IrohResolveKey {
    fn from(value: &BlobPoolItem) -> Self {
        Self { hash: value.hash }
    }
}

impl From<&BlobPoolItem> for IrohTaskType {
    fn from(value: &BlobPoolItem) -> Self {
        Self::ResolveBlob {
            source: IrohResolveSource { id: value.source },
        }
    }
}

/// A read request pool item. This is the task that needs to be resolved by iroh resolver.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ReadRequestPoolItem {
    /// The unique id of the read request.
    id: Hash,
    /// The hash of the blob that the read request is for.
    blob_hash: Hash,
    /// The offset of the read request.
    offset: u32,
    /// The length of the read request.
    len: u32,
    /// The address and method to callback when the read request is closed.
    callback: (Address, MethodNum),
}

impl From<&ReadRequestPoolItem> for IrohResolveKey {
    fn from(value: &ReadRequestPoolItem) -> Self {
        Self { hash: value.id }
    }
}

impl From<&ReadRequestPoolItem> for IrohTaskType {
    fn from(value: &ReadRequestPoolItem) -> Self {
        Self::CloseReadRequest {
            blob_hash: value.blob_hash,
            offset: value.offset,
            len: value.len,
        }
    }
}

/// A user sent a transaction which they are not allowed to do.
pub struct IllegalMessage;

// For now this is the only option, later we can expand.
pub enum ChainMessageApplyRet {
    Signed(SignedMessageApplyRes),
    /// The IPC chain message execution result
    Ipc(FvmApplyRet),
}

/// We only allow signed messages into the mempool.
pub type ChainMessageCheckRes = Result<SignedMessageCheckRes, IllegalMessage>;

/// Interpreter working on chain messages; in the future it will schedule
/// CID lookups to turn references into self-contained user or cross-messages.
#[derive(Clone)]
pub struct ChainMessageInterpreter<I, DB> {
    inner: I,
    gateway_caller: GatewayCaller<DB>,
}

impl<I, DB> ChainMessageInterpreter<I, DB> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            gateway_caller: GatewayCaller::default(),
        }
    }
}

#[async_trait]
impl<I, DB> ProposalInterpreter for ChainMessageInterpreter<I, DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    I: Sync + Send,
{
    type State = (ChainEnv, FvmExecState<ReadOnlyBlockstore<Arc<DB>>>);
    type Message = ChainMessage;

    /// Check whether there are any "ready" messages in the IPLD resolution mempool which can be
    /// appended to the proposal.
    ///
    /// We could also use this to select the most profitable user transactions within the gas limit.
    /// We can also take into account the transactions which are part of top-down or bottom-up
    /// checkpoints, to stay within gas limits.
    async fn prepare(
        &self,
        (chain_env, mut state): Self::State,
        mut msgs: Vec<Self::Message>,
    ) -> anyhow::Result<Vec<Self::Message>> {
        msgs = messages_selection(msgs, &state)?;

        // Collect resolved CIDs ready to be proposed from the pool.
        let ckpts = atomically(|| chain_env.checkpoint_pool.collect_resolved()).await;

        // Create transactions ready to be included on the chain.
        let ckpts = ckpts.into_iter().map(|ckpt| match ckpt {
            CheckpointPoolItem::BottomUp(ckpt) => ChainMessage::Ipc(IpcMessage::BottomUpExec(ckpt)),
        });

        // Prepare top down proposals.
        // Before we try to find a quorum, pause incoming votes. This is optional but if there are lots of votes coming in it might hold up proposals.
        atomically(|| {
            chain_env
                .parent_finality_votes
                .pause_votes_until_find_quorum()
        })
        .await;

        // The pre-requisite for a proposal is that there is a quorum of gossiped votes at that height.
        // The final proposal can be at most as high as the quorum, but can be less if we have already
        // hit some limits such as how many blocks we can propose in a single step.
        let finalities = atomically(|| {
            let parent = chain_env.parent_finality_provider.next_proposal()?;
            let quorum = chain_env
                .parent_finality_votes
                .find_quorum()?
                .map(|(height, block_hash)| IPCParentFinality { height, block_hash });

            Ok((parent, quorum))
        })
        .await;

        let maybe_finality = match finalities {
            (Some(parent), Some(quorum)) => Some(if parent.height <= quorum.height {
                parent
            } else {
                quorum
            }),
            (Some(parent), None) => {
                emit!(
                    DEBUG,
                    ParentFinalityMissingQuorum {
                        block_height: parent.height,
                        block_hash: &hex::encode(&parent.block_hash),
                    }
                );
                None
            }
            (None, _) => {
                // This is normal, the parent probably hasn't produced a block yet.
                None
            }
        };

        if let Some(finality) = maybe_finality {
            msgs.push(ChainMessage::Ipc(IpcMessage::TopDownExec(ParentFinality {
                height: finality.height as ChainEpoch,
                block_hash: finality.block_hash,
            })))
        }

        // Append at the end - if we run out of block space, these are going to be reproposed in
        // the next block.
        msgs.extend(ckpts);

        // Collect and enqueue blobs that are not added in the pool yet.
        state.state_tree_mut().begin_transaction();
        let added_blobs = get_added_blobs(&mut state, chain_env.blob_concurrency)?;
        state
            .state_tree_mut()
            .end_transaction(true)
            .expect("we just started a transaction");
        for (hash, sources) in added_blobs {
            for (subscriber, id, source) in sources {
                msgs.push(ChainMessage::Ipc(IpcMessage::BlobPending(PendingBlob {
                    subscriber,
                    hash,
                    id: id.clone(),
                    source,
                })));
            }
        }

        // Maybe debit all credit accounts
        let current_height = state.block_height();
        let debit_interval = state.credit_debit_interval();
        if current_height > 0 && debit_interval > 0 && current_height % debit_interval == 0 {
            msgs.push(ChainMessage::Ipc(IpcMessage::DebitCreditAccounts));
        }

        // Collect locally completed blobs from the pool. We're relying on the proposer's local
        // view of blob resolution, rather than considering those that _might_ have a quorum,
        // but have not yet been resolved by _this_ proposer. However, a blob like this will get
        // picked up by a different proposer who _does_ consider it resolved.
        let local_finalized_blobs = atomically(|| chain_env.blob_pool.collect_done()).await;

        // Create transactions ready to be included on the chain. These are from locally resolved
        // or failed blobs that have reached a global quorum and are not yet finalized.
        //
        // If the blob has already been finalized, i.e., it was proposed in an earlier block with
        // a quorum that did not include _this_ proposer, we can just remove it from the local
        // resolve pool. If we were to propose it, it would be rejected in the process step.
        if !local_finalized_blobs.is_empty() {
            let mut blobs: Vec<ChainMessage> = vec![];
            // We start a blockstore transaction that can be reverted
            state.state_tree_mut().begin_transaction();
            for item in local_finalized_blobs.iter() {
                if is_blob_finalized(&mut state, item.subscriber, item.hash, item.id.clone())? {
                    tracing::debug!(hash = ?item.hash, "blob already finalized on chain; removing from pool");
                    atomically(|| chain_env.blob_pool.remove_task(item)).await;
                    continue;
                }

                let (is_globally_finalized, succeeded) = atomically(|| {
                    chain_env
                        .parent_finality_votes
                        .find_blob_quorum(&item.hash.as_bytes().to_vec())
                })
                .await;
                if is_globally_finalized {
                    tracing::debug!(hash = ?item.hash, "blob has quorum; adding tx to chain");
                    blobs.push(ChainMessage::Ipc(IpcMessage::BlobFinalized(
                        FinalizedBlob {
                            subscriber: item.subscriber,
                            hash: item.hash,
                            id: item.id.clone(),
                            source: item.source,
                            succeeded,
                        },
                    )));
                }
            }
            state
                .state_tree_mut()
                .end_transaction(true)
                .expect("we just started a transaction");

            let pending_blobs = atomically(|| chain_env.blob_pool.count()).await;
            tracing::info!(size = pending_blobs, "blob pool status");

            // Append at the end - if we run out of block space,
            // these are going to be reproposed in the next block.
            msgs.extend(blobs);
        }

        // Get pending read requests from the readreq actor
        let open_requests = with_state_transaction(&mut state, |state| {
            let requests = get_open_read_requests(state, chain_env.read_request_concurrency)?;
            tracing::debug!(size = requests.len(), "read requests fetched from chain");
            Ok(requests)
        })?;

        // create IPC messages to add read requests to the pool
        for (id, blob_hash, offset, len, callback_addr, callback_method) in open_requests {
            msgs.push(ChainMessage::Ipc(IpcMessage::ReadRequestPending(
                PendingReadRequest {
                    id,
                    blob_hash,
                    offset,
                    len,
                    callback: (callback_addr, callback_method),
                },
            )));
        }

        // collect locally resolved read requests
        let locally_resolved_read_requests =
            atomically(|| chain_env.read_request_pool.collect_done()).await;
        tracing::debug!(
            size = locally_resolved_read_requests.len(),
            "locally resolved read requests"
        );

        if !locally_resolved_read_requests.is_empty() {
            let mut read_requests: Vec<ChainMessage> = vec![];
            // We start a blockstore transaction that can be reverted
            state.state_tree_mut().begin_transaction();
            for item in locally_resolved_read_requests.iter() {
                // check if the read request is closed i.e. not open or pending
                // if a request is not found in actor state but exists in the pool, it is considered closed
                if get_read_request_status(&mut state, item.id)?.is_none() {
                    tracing::debug!(request_id = ?item.id, "read request already fulfilled on chain; removing from pool");
                    atomically(|| chain_env.read_request_pool.remove_task(item)).await;
                    continue;
                }
                let read_response = atomically(|| chain_env.read_request_pool.get_result(item))
                    .await
                    .unwrap_or(vec![]);
                // Remove the result from the pool
                atomically(|| chain_env.read_request_pool.remove_result(item)).await;

                // Extend request id with response data to use as the vote hash.
                // This ensures that the all validators are voting
                // on the same response from IROH.
                let mut request_id = item.id.as_bytes().to_vec();
                request_id.extend(read_response.clone());
                let vote_hash = Hash::new(request_id);
                let (is_globally_finalized, _) = atomically(|| {
                    chain_env
                        .parent_finality_votes
                        .find_blob_quorum(&vote_hash.as_bytes().to_vec())
                })
                .await;
                tracing::debug!(
                    request_id = ?item.id,
                    quorum = ?is_globally_finalized,
                    "read request quorum"
                );
                let item = ReadRequestPoolItem {
                    id: item.id,
                    blob_hash: item.blob_hash,
                    offset: item.offset,
                    len: item.len,
                    callback: item.callback,
                };

                if is_globally_finalized {
                    tracing::debug!(request_id = ?item.id, "read request has quorum; adding tx to chain");
                    read_requests.push(ChainMessage::Ipc(IpcMessage::ReadRequestClosed(
                        ClosedReadRequest {
                            id: item.id,
                            blob_hash: item.blob_hash,
                            offset: item.offset,
                            len: item.len as u32,
                            callback: item.callback,
                            response: read_response,
                        },
                    )));
                }
            }
            state
                .state_tree_mut()
                .end_transaction(true)
                .expect("we just started a transaction");

            let pending_read_requests = atomically(|| chain_env.read_request_pool.count()).await;
            tracing::info!(size = pending_read_requests, "read request pool status");
            // Append at the end - if we run out of block space,
            // these are going to be reproposed in the next block.
            msgs.extend(read_requests);
        }

        Ok(msgs)
    }

    /// Perform finality checks on top-down transactions and availability checks on bottom-up transactions.
    async fn process(
        &self,
        (chain_env, mut state): Self::State,
        msgs: Vec<Self::Message>,
    ) -> anyhow::Result<bool> {
        let mut block_gas_usage = 0;
        for msg in msgs {
            match msg {
                ChainMessage::Ipc(IpcMessage::BottomUpExec(msg)) => {
                    let item = CheckpointPoolItem::BottomUp(msg);

                    // We can just look in memory because when we start the application, we should retrieve any
                    // pending checkpoints (relayed but not executed) from the ledger, so they should be there.
                    // We don't have to validate the checkpoint here, because
                    // 1) we validated it when it was relayed, and
                    // 2) if a validator proposes something invalid, we can make them pay during execution.
                    let is_resolved =
                        atomically(|| match chain_env.checkpoint_pool.get_status(&item)? {
                            None => Ok(false),
                            Some(status) => status.is_resolved(),
                        })
                        .await;

                    if !is_resolved {
                        return Ok(false);
                    }
                }
                ChainMessage::Ipc(IpcMessage::TopDownExec(ParentFinality {
                    height,
                    block_hash,
                })) => {
                    let prop = IPCParentFinality {
                        height: height as u64,
                        block_hash,
                    };
                    let is_final =
                        atomically(|| chain_env.parent_finality_provider.check_proposal(&prop))
                            .await;
                    if !is_final {
                        return Ok(false);
                    }
                }
                ChainMessage::Ipc(IpcMessage::BlobPending(blob)) => {
                    // Check that blobs that are being enqueued are still in "added" state in the actor
                    // Once we enqueue a blob, the actor will transition it to "pending" state.
                    if !is_blob_added(&mut state, blob.subscriber, blob.hash, blob.id)? {
                        tracing::debug!(hash = ?blob.hash, "blob is not added onchain; rejecting proposal");
                        return Ok(false);
                    }
                }
                ChainMessage::Ipc(IpcMessage::BlobFinalized(blob)) => {
                    // Ensure that the blob is ready to be included on-chain.
                    // We can accept the proposal if the blob has reached a global quorum and is
                    // not yet finalized.
                    // Start a blockstore transaction that can be reverted.
                    state.state_tree_mut().begin_transaction();
                    if is_blob_finalized(&mut state, blob.subscriber, blob.hash, blob.id.clone())? {
                        tracing::debug!(hash = ?blob.hash, "blob is already finalized on chain; rejecting proposal");
                        return Ok(false);
                    }
                    state
                        .state_tree_mut()
                        .end_transaction(true)
                        .expect("we just started a transaction");

                    let (is_globally_finalized, succeeded) = atomically(|| {
                        chain_env
                            .parent_finality_votes
                            .find_blob_quorum(&blob.hash.as_bytes().to_vec())
                    })
                    .await;
                    if !is_globally_finalized {
                        tracing::debug!(hash = ?blob.hash, "blob is not globally finalized; rejecting proposal");
                        return Ok(false);
                    }
                    if blob.succeeded != succeeded {
                        tracing::debug!(
                            hash = ?blob.hash,
                            quorum = ?succeeded,
                            message = ?blob.succeeded,
                            "blob finalization mismatch; rejecting proposal"
                        );
                        return Ok(false);
                    }

                    // Remove from pool if locally resolved
                    let item = BlobPoolItem {
                        subscriber: blob.subscriber,
                        hash: blob.hash,
                        id: blob.id,
                        source: blob.source,
                    };
                    let is_locally_finalized =
                        atomically(|| match chain_env.blob_pool.get_status(&item)? {
                            None => Ok(false),
                            Some(status) => Ok(status.is_resolved()? || status.is_failed()?),
                        })
                        .await;
                    if is_locally_finalized {
                        tracing::debug!(hash = ?blob.hash, "blob is locally finalized; removing from pool");
                        atomically(|| chain_env.blob_pool.remove_task(&item)).await;
                    } else {
                        tracing::debug!(hash = ?blob.hash, "blob is not locally finalized");
                    }
                }
                ChainMessage::Ipc(IpcMessage::DebitCreditAccounts) => {
                    // Ensure that this is a valid height to debit accounts
                    let current_height = state.block_height();
                    let debit_interval = state.credit_debit_interval();
                    if !(current_height > 0
                        && debit_interval > 0
                        && current_height % debit_interval == 0)
                    {
                        tracing::debug!(
                            interval = ?debit_interval,
                            height = ?current_height,
                            "invalid height for credit debit; rejecting proposal"
                        );
                        return Ok(false);
                    }
                }
                ChainMessage::Signed(signed) => {
                    block_gas_usage += signed.message.gas_limit;
                }
                ChainMessage::Ipc(IpcMessage::ReadRequestClosed(read_request)) => {
                    // Ensure that the read request is ready to be fulfilled.
                    // We can accept the proposal if the read request has reached a quorum and is
                    // not yet closed.
                    let status = with_state_transaction(&mut state, |state| {
                        get_read_request_status(state, read_request.id)
                    })?;
                    if !matches!(status, Some(ReadRequestStatus::Pending)) {
                        tracing::info!(hash = ?read_request.id, "only pending read requests can be closed; rejecting proposal");
                        return Ok(false);
                    }

                    let (is_globally_finalized, succeeded) = atomically(|| {
                        chain_env
                            .parent_finality_votes
                            .find_blob_quorum(&read_request.id.as_bytes().to_vec())
                    })
                    .await;
                    if !is_globally_finalized {
                        tracing::info!(hash = ?read_request.id, "read request is not globally finalized; rejecting proposal");
                        return Ok(false);
                    }

                    // Remove from pool if locally resolved
                    let item = ReadRequestPoolItem {
                        id: read_request.id,
                        blob_hash: read_request.blob_hash,
                        offset: read_request.offset,
                        len: read_request.len,
                        callback: read_request.callback,
                    };
                    let is_locally_finalized =
                        atomically(|| match chain_env.read_request_pool.get_status(&item)? {
                            None => Ok(false),
                            Some(status) => Ok(status.is_resolved()? || status.is_failed()?),
                        })
                        .await;
                    if is_locally_finalized {
                        tracing::debug!(request_id = ?read_request.id, "read request is locally finalized; removing from pool");
                        atomically(|| chain_env.read_request_pool.remove_task(&item)).await;
                        // Remove the result from the pool
                        atomically(|| chain_env.read_request_pool.remove_result(&item)).await;
                    } else {
                        tracing::debug!(request_id = ?read_request.id, "read request is not locally finalized");
                    }
                }
                ChainMessage::Ipc(IpcMessage::ReadRequestPending(read_request)) => {
                    // Check that the read request is still open
                    let status = get_read_request_status(&mut state, read_request.id)?;
                    if !matches!(status, Some(ReadRequestStatus::Open)) {
                        tracing::info!(request_id = ?read_request.id, "read request is not open; rejecting proposal");
                        return Ok(false);
                    }
                }
                _ => {}
            };
        }

        Ok(block_gas_usage <= state.block_gas_tracker().available())
    }
}

#[async_trait]
impl<I, DB> ExecInterpreter for ChainMessageInterpreter<I, DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync + Clone,
    I: ExecInterpreter<
        Message = VerifiableMessage,
        DeliverOutput = SignedMessageApplyRes,
        State = FvmExecState<DB>,
        EndOutput = (PowerUpdates, BlockGasLimit),
    >,
{
    // The state consists of the resolver pool, which this interpreter needs, and the rest of the
    // state which the inner interpreter uses. This is a technical solution because the pool doesn't
    // fit with the state we use for execution messages further down the stack, which depend on block
    // height and are used in queries as well.
    type State = (ChainEnv, I::State);
    type Message = ChainMessage;
    type BeginOutput = I::BeginOutput;
    type DeliverOutput = ChainMessageApplyRet;
    type EndOutput = I::EndOutput;

    async fn begin(
        &self,
        (env, state): Self::State,
    ) -> anyhow::Result<(Self::State, Self::BeginOutput)> {
        let (state, out) = self.inner.begin(state).await?;
        Ok(((env, state), out))
    }

    async fn deliver(
        &self,
        (env, mut state): Self::State,
        msg: Self::Message,
    ) -> anyhow::Result<(Self::State, Self::DeliverOutput)> {
        match msg {
            ChainMessage::Signed(msg) => {
                let (state, ret) = self
                    .inner
                    .deliver(state, VerifiableMessage::Signed(msg.clone()))
                    .await?;
                Ok(((env, state), ChainMessageApplyRet::Signed(ret)))
            }
            ChainMessage::Ipc(msg) => match msg {
                IpcMessage::BottomUpResolve(msg) => {
                    let smsg = relayed_bottom_up_ckpt_to_fvm(&msg)
                        .context("failed to syntesize FVM message")?;

                    // Let the FVM validate the checkpoint quorum certificate and take note of the relayer for rewards.
                    let (state, ret) = self
                        .inner
                        .deliver(state, VerifiableMessage::Synthetic(smsg))
                        .await
                        .context("failed to deliver bottom up checkpoint")?;

                    // If successful, add the CID to the background resolution pool.
                    let is_success = match ret {
                        Ok(ref ret) => ret.fvm.apply_ret.msg_receipt.exit_code.is_success(),
                        Err(_) => false,
                    };

                    if is_success {
                        // For now, try to get it from the child subnet. If the same comes up for execution, include own.
                        atomically(|| {
                            env.checkpoint_pool.add(
                                CheckpointPoolItem::BottomUp(msg.message.message.clone()),
                                false,
                            )
                        })
                        .await;
                    }

                    // We can use the same result type for now, it's isomorphic.
                    Ok(((env, state), ChainMessageApplyRet::Signed(ret)))
                }
                IpcMessage::BottomUpExec(_) => {
                    todo!("#197: implement BottomUp checkpoint execution")
                }
                IpcMessage::TopDownExec(p) => {
                    if !env.parent_finality_provider.is_enabled() {
                        bail!("cannot execute IPC top-down message: parent provider disabled");
                    }

                    // commit parent finality first
                    let finality = IPCParentFinality::new(p.height, p.block_hash);
                    tracing::debug!(
                        finality = finality.to_string(),
                        "chain interpreter received topdown exec proposal",
                    );

                    let (prev_height, prev_finality) = topdown::commit_finality(
                        &self.gateway_caller,
                        &mut state,
                        finality.clone(),
                        &env.parent_finality_provider,
                    )
                    .await
                    .context("failed to commit finality")?;

                    tracing::debug!(
                        previous_committed_height = prev_height,
                        previous_committed_finality = prev_finality
                            .as_ref()
                            .map(|f| format!("{f}"))
                            .unwrap_or_else(|| String::from("None")),
                        "chain interpreter committed topdown finality",
                    );

                    // The height range we pull top-down effects from. This _includes_ the proposed
                    // finality, as we assume that the interface we query publishes only fully
                    // executed blocks as the head of the chain. This is certainly the case for
                    // Ethereum-compatible JSON-RPC APIs, like Filecoin's. It should be the case
                    // too for future Filecoin light clients.
                    //
                    // Another factor to take into account is the chain_head_delay, which must be
                    // non-zero. So even in the case where deferred execution leaks through our
                    // query mechanism, it should not be problematic because we're guaranteed to
                    // be _at least_ 1 height behind.
                    let (execution_fr, execution_to) = (prev_height + 1, finality.height);

                    // error happens if we cannot get the validator set from ipc agent after retries
                    let validator_changes = env
                        .parent_finality_provider
                        .validator_changes_from(execution_fr, execution_to)
                        .await
                        .context("failed to fetch validator changes")?;

                    tracing::debug!(
                        from = execution_fr,
                        to = execution_to,
                        msgs = validator_changes.len(),
                        "chain interpreter received total validator changes"
                    );

                    self.gateway_caller
                        .store_validator_changes(&mut state, validator_changes)
                        .context("failed to store validator changes")?;

                    // error happens if we cannot get the cross-messages from ipc agent after retries
                    let msgs = env
                        .parent_finality_provider
                        .top_down_msgs_from(execution_fr, execution_to)
                        .await
                        .context("failed to fetch top down messages")?;

                    tracing::debug!(
                        number_of_messages = msgs.len(),
                        start = execution_fr,
                        end = execution_to,
                        "chain interpreter received topdown msgs",
                    );

                    let ret = topdown::execute_topdown_msgs(&self.gateway_caller, &mut state, msgs)
                        .await
                        .context("failed to execute top down messages")?;

                    tracing::debug!("chain interpreter applied topdown msgs");

                    let local_block_height = state.block_height() as u64;
                    let proposer = state
                        .block_producer()
                        .map(|id| hex::encode(id.serialize_compressed()));
                    let proposer_ref = proposer.as_deref();

                    atomically(|| {
                        env.parent_finality_provider
                            .set_new_finality(finality.clone(), prev_finality.clone())?;

                        env.parent_finality_votes.set_finalized(
                            finality.height,
                            finality.block_hash.clone(),
                            proposer_ref,
                            Some(local_block_height),
                        )?;

                        Ok(())
                    })
                    .await;

                    tracing::debug!(
                        finality = finality.to_string(),
                        "chain interpreter has set new"
                    );

                    Ok(((env, state), ChainMessageApplyRet::Ipc(ret)))
                }
                IpcMessage::BlobPending(blob) => {
                    let from = system::SYSTEM_ACTOR_ADDR;
                    let to = blobs::BLOBS_ACTOR_ADDR;
                    let method_num = SetBlobPending as u64;
                    let gas_limit = fvm_shared::BLOCK_GAS_LIMIT;

                    let source =
                        fendermint_actor_blobs_shared::state::PublicKey(*blob.source.as_bytes());
                    let hash = fendermint_actor_blobs_shared::state::Hash(*blob.hash.as_bytes());
                    let params = SetBlobPendingParams {
                        source,
                        subscriber: blob.subscriber,
                        hash,
                        id: blob.id.clone(),
                    };
                    let params = RawBytes::serialize(params)?;
                    let msg = Message {
                        version: Default::default(),
                        from,
                        to,
                        sequence: 0,
                        value: Default::default(),
                        method_num,
                        params,
                        gas_limit,
                        gas_fee_cap: Default::default(),
                        gas_premium: Default::default(),
                    };

                    let (apply_ret, emitters) = state.execute_implicit(msg)?;

                    let info = apply_ret
                        .failure_info
                        .clone()
                        .map(|i| i.to_string())
                        .filter(|s| !s.is_empty());
                    tracing::info!(
                        exit_code = apply_ret.msg_receipt.exit_code.value(),
                        from = from.to_string(),
                        to = to.to_string(),
                        method_num = method_num,
                        gas_limit = gas_limit,
                        gas_used = apply_ret.msg_receipt.gas_used,
                        info = info.unwrap_or_default(),
                        "implicit tx delivered"
                    );

                    tracing::debug!(
                        hash = ?blob.hash,
                        "chain interpreter has set blob to pending"
                    );

                    // Add the blob the resolution pool
                    atomically(|| {
                        env.blob_pool.add(BlobPoolItem {
                            subscriber: blob.subscriber,
                            hash: blob.hash,
                            id: blob.id.clone(),
                            source: blob.source,
                        })
                    })
                    .await;

                    let ret = FvmApplyRet {
                        apply_ret,
                        from,
                        to,
                        method_num,
                        gas_limit,
                        emitters,
                    };

                    Ok(((env, state), ChainMessageApplyRet::Ipc(ret)))
                }
                IpcMessage::BlobFinalized(blob) => {
                    let from = system::SYSTEM_ACTOR_ADDR;
                    let to = blobs::BLOBS_ACTOR_ADDR;
                    let method_num = FinalizeBlob as u64;
                    let gas_limit = fvm_shared::BLOCK_GAS_LIMIT;

                    let hash = fendermint_actor_blobs_shared::state::Hash(*blob.hash.as_bytes());
                    let status = if blob.succeeded {
                        BlobStatus::Resolved
                    } else {
                        BlobStatus::Failed
                    };
                    let params = FinalizeBlobParams {
                        subscriber: blob.subscriber,
                        hash,
                        id: blob.id.into(),
                        status,
                    };
                    let params = RawBytes::serialize(params)?;
                    let msg = Message {
                        version: Default::default(),
                        from,
                        to,
                        sequence: 0,
                        value: Default::default(),
                        method_num,
                        params,
                        gas_limit,
                        gas_fee_cap: Default::default(),
                        gas_premium: Default::default(),
                    };

                    let (apply_ret, emitters) = state.execute_implicit(msg)?;

                    let info = apply_ret
                        .failure_info
                        .clone()
                        .map(|i| i.to_string())
                        .filter(|s| !s.is_empty());
                    tracing::info!(
                        exit_code = apply_ret.msg_receipt.exit_code.value(),
                        from = from.to_string(),
                        to = to.to_string(),
                        method_num = method_num,
                        gas_limit = gas_limit,
                        gas_used = apply_ret.msg_receipt.gas_used,
                        info = info.unwrap_or_default(),
                        "implicit tx delivered"
                    );

                    tracing::debug!(
                        hash = ?blob.hash,
                        "chain interpreter has finalized blob"
                    );

                    // Once the blob is finalized on the parent, we can clean up the votes
                    atomically(|| {
                        env.parent_finality_votes
                            .clear_blob(blob.hash.as_bytes().to_vec())?;
                        Ok(())
                    })
                    .await;

                    let ret = FvmApplyRet {
                        apply_ret,
                        from,
                        to,
                        method_num,
                        gas_limit,
                        emitters,
                    };

                    Ok(((env, state), ChainMessageApplyRet::Ipc(ret)))
                }
                IpcMessage::DebitCreditAccounts => {
                    let from = system::SYSTEM_ACTOR_ADDR;
                    let to = blobs::BLOBS_ACTOR_ADDR;
                    let method_num = DebitAccounts as u64;
                    let gas_limit = fvm_shared::BLOCK_GAS_LIMIT;

                    let msg = Message {
                        version: Default::default(),
                        from,
                        to,
                        sequence: 0, // We will use implicit execution which doesn't check or modify this.
                        value: Default::default(),
                        method_num,
                        params: Default::default(),
                        gas_limit,
                        gas_fee_cap: Default::default(),
                        gas_premium: Default::default(),
                    };

                    let (apply_ret, emitters) = state.execute_implicit(msg)?;

                    let info = apply_ret
                        .failure_info
                        .clone()
                        .map(|i| i.to_string())
                        .filter(|s| !s.is_empty());
                    tracing::info!(
                        exit_code = apply_ret.msg_receipt.exit_code.value(),
                        from = from.to_string(),
                        to = to.to_string(),
                        method_num = method_num,
                        gas_limit = gas_limit,
                        gas_used = apply_ret.msg_receipt.gas_used,
                        info = info.unwrap_or_default(),
                        "implicit tx delivered"
                    );

                    tracing::debug!("chain interpreter debited accounts");

                    let ret = FvmApplyRet {
                        apply_ret,
                        from,
                        to,
                        method_num,
                        gas_limit,
                        emitters,
                    };

                    Ok(((env, state), ChainMessageApplyRet::Ipc(ret)))
                }
                IpcMessage::ReadRequestClosed(read_request) => {
                    // send the data to the callback address
                    // if this fails (e.g. the callback address is not reachable),
                    // we will still close the request
                    read_request_callback(&mut state, &read_request)?;
                    // set the status of the request to fulfilled
                    let ret = close_read_request(&mut state, read_request.id)?;

                    tracing::debug!(
                        hash = ?read_request.id,
                        "read request is closed"
                    );

                    Ok(((env, state), ChainMessageApplyRet::Ipc(ret)))
                }
                IpcMessage::ReadRequestPending(read_request) => {
                    // Set the read request to "pending" state
                    let from = system::SYSTEM_ACTOR_ADDR;
                    let to = readreq::READREQ_ACTOR_ADDR;
                    let method_num = SetReadRequestPending as u64;
                    let gas_limit = fvm_shared::BLOCK_GAS_LIMIT;
                    let params = SetReadRequestPendingParams(fendermint_actor_readreq::Hash(
                        *read_request.id.as_bytes(),
                    ));
                    let params = RawBytes::serialize(params)?;
                    let msg = Message {
                        version: Default::default(),
                        from,
                        to,
                        sequence: 0,
                        value: Default::default(),
                        method_num,
                        params,
                        gas_limit,
                        gas_fee_cap: Default::default(),
                        gas_premium: Default::default(),
                    };
                    let (apply_ret, emitters) = state.execute_implicit(msg)?;
                    let ret = FvmApplyRet {
                        apply_ret,
                        from,
                        to,
                        method_num,
                        gas_limit,
                        emitters,
                    };

                    // Add the read request to the pool
                    atomically(|| {
                        env.read_request_pool.add(ReadRequestPoolItem {
                            id: read_request.id,
                            blob_hash: read_request.blob_hash,
                            offset: read_request.offset,
                            len: read_request.len,
                            callback: read_request.callback,
                        })
                    })
                    .await;
                    tracing::info!(request_id = ?read_request.id, "read request added to pool");
                    Ok(((env, state), ChainMessageApplyRet::Ipc(ret)))
                }
            },
        }
    }

    async fn end(
        &self,
        (env, state): Self::State,
    ) -> anyhow::Result<(Self::State, Self::EndOutput)> {
        let (mut state, out) = self.inner.end(state).await?;

        // Update any component that needs to know about changes in the power table.
        if !out.0 .0.is_empty() {
            let power_updates = out
                .0
                 .0
                .iter()
                .map(|v| {
                    let vk = ValidatorKey::from(v.public_key.0);
                    let w = v.power.0;
                    (vk, w)
                })
                .collect::<Vec<_>>();

            atomically(|| {
                env.parent_finality_votes
                    .update_power_table(power_updates.clone())
            })
            .await;
        }

        // Get pending blobs count and bytes count to emit metrics
        let stats = get_blobs_stats(&mut state)?;
        ipc_observability::emit(BlobsFinalityPendingBlobs(stats.num_resolving));
        ipc_observability::emit(BlobsFinalityPendingBytes(stats.bytes_resolving));
        ipc_observability::emit(BlobsFinalityAddedBlobs(stats.num_added));
        ipc_observability::emit(BlobsFinalityAddedBytes(stats.bytes_added));

        Ok(((env, state), out))
    }
}

#[async_trait]
impl<I, DB> CheckInterpreter for ChainMessageInterpreter<I, DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    I: CheckInterpreter<Message = VerifiableMessage, Output = SignedMessageCheckRes>,
{
    type State = I::State;
    type Message = ChainMessage;
    type Output = ChainMessageCheckRes;

    async fn check(
        &self,
        state: Self::State,
        msg: Self::Message,
        is_recheck: bool,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        match msg {
            ChainMessage::Signed(msg) => {
                let (state, ret) = self
                    .inner
                    .check(state, VerifiableMessage::Signed(msg), is_recheck)
                    .await?;

                Ok((state, Ok(ret)))
            }
            ChainMessage::Ipc(msg) => {
                match msg {
                    IpcMessage::BottomUpResolve(msg) => {
                        let msg = relayed_bottom_up_ckpt_to_fvm(&msg)
                            .context("failed to syntesize FVM message")?;

                        let (state, ret) = self
                            .inner
                            .check(state, VerifiableMessage::Synthetic(msg), is_recheck)
                            .await
                            .context("failed to check bottom up resolve")?;

                        Ok((state, Ok(ret)))
                    }
                    IpcMessage::TopDownExec(_)
                    | IpcMessage::BottomUpExec(_)
                    | IpcMessage::BlobPending(_)
                    | IpcMessage::BlobFinalized(_)
                    | IpcMessage::DebitCreditAccounts
                    | IpcMessage::ReadRequestClosed(_)
                    | IpcMessage::ReadRequestPending(_) => {
                        // Users cannot send these messages, only validators can propose them in blocks.
                        Ok((state, Err(IllegalMessage)))
                    }
                }
            }
        }
    }
}

#[async_trait]
impl<I, DB> QueryInterpreter for ChainMessageInterpreter<I, DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    I: QueryInterpreter,
{
    type State = I::State;
    type Query = I::Query;
    type Output = I::Output;

    async fn query(
        &self,
        state: Self::State,
        qry: Self::Query,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        self.inner.query(state, qry).await
    }
}

/// Convert a signed relayed bottom-up checkpoint to a syntetic message we can send to the FVM.
///
/// By mapping to an FVM message, we invoke the right contract to validate the checkpoint.
/// We automatically charge the relayer gas for the execution of the check, but not the
/// execution of the cross-messages, which aren't part of the payload.
fn relayed_bottom_up_ckpt_to_fvm(
    relayed: &SignedRelayedMessage<CertifiedMessage<BottomUpCheckpoint>>,
) -> anyhow::Result<SyntheticMessage> {
    // TODO #192: Convert the checkpoint to what the actor expects.
    let params = RawBytes::default();

    let msg = FvmMessage {
        version: 0,
        from: relayed.message.relayer,
        to: ipc::GATEWAY_ACTOR_ADDR,
        sequence: relayed.message.sequence,
        value: TokenAmount::zero(),
        method_num: ipc::gateway::METHOD_INVOKE_CONTRACT,
        params,
        gas_limit: relayed.message.gas_limit,
        gas_fee_cap: relayed.message.gas_fee_cap.clone(),
        gas_premium: relayed.message.gas_premium.clone(),
    };

    let msg = SyntheticMessage::new(msg, &relayed.message, relayed.signature.clone())
        .context("failed to create syntetic message")?;

    Ok(msg)
}

/// Get added blobs from on chain state.
fn get_added_blobs<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    size: u32,
) -> anyhow::Result<Vec<AddedBlobItem>>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let params = GetAddedBlobsParams(size);
    let params = RawBytes::serialize(params)?;
    let msg = FvmMessage {
        version: 0,
        from: system::SYSTEM_ACTOR_ADDR,
        to: blobs::BLOBS_ACTOR_ADDR,
        sequence: 0,
        value: Default::default(),
        method_num: GetAddedBlobs as u64,
        params,
        gas_limit: fvm_shared::BLOCK_GAS_LIMIT,
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };
    let (apply_ret, _) = state.execute_implicit(msg)?;

    let data: bytes::Bytes = apply_ret.msg_receipt.return_data.to_vec().into();
    fvm_ipld_encoding::from_slice::<Vec<AddedBlobItem>>(&data)
        .map_err(|e| anyhow!("error parsing added blobs: {e}"))
}

/// Helper function to check blob status by reading its on-chain state.
fn get_blob_status<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    subscriber: Address,
    hash: Hash,
    id: SubscriptionId,
) -> anyhow::Result<Option<BlobStatus>>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let hash = fendermint_actor_blobs_shared::state::Hash(*hash.as_bytes());
    let params = GetBlobStatusParams {
        subscriber,
        hash,
        id,
    };
    let params = RawBytes::serialize(params)?;
    let msg = FvmMessage {
        version: 0,
        from: system::SYSTEM_ACTOR_ADDR,
        to: blobs::BLOBS_ACTOR_ADDR,
        sequence: 0,
        value: Default::default(),
        method_num: GetBlobStatus as u64,
        params,
        gas_limit: fvm_shared::BLOCK_GAS_LIMIT,
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };
    let (apply_ret, _) = state.execute_implicit(msg)?;

    let data: bytes::Bytes = apply_ret.msg_receipt.return_data.to_vec().into();
    fvm_ipld_encoding::from_slice::<Option<BlobStatus>>(&data)
        .map_err(|e| anyhow!("error parsing blob status: {e}"))
}

/// Check if a blob is in added state, by reading its on-chain state.
fn is_blob_added<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    subscriber: Address,
    hash: Hash,
    id: SubscriptionId,
) -> anyhow::Result<bool>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let status = get_blob_status(state, subscriber, hash, id)?;
    let added = if let Some(status) = status {
        matches!(status, BlobStatus::Added)
    } else {
        false
    };
    Ok(added)
}

/// Check if a blob is finalized (if it is resolved or failed), by reading its on-chain state.
fn is_blob_finalized<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    subscriber: Address,
    hash: Hash,
    id: SubscriptionId,
) -> anyhow::Result<bool>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let status = get_blob_status(state, subscriber, hash, id)?;
    let finalized = if let Some(status) = status {
        matches!(status, BlobStatus::Resolved | BlobStatus::Failed)
    } else {
        false
    };
    Ok(finalized)
}

/// Returns credit and blob stats from on-chain state.
fn get_blobs_stats<DB>(state: &mut FvmExecState<DB>) -> anyhow::Result<GetStatsReturn>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let msg = FvmMessage {
        version: 0,
        from: system::SYSTEM_ACTOR_ADDR,
        to: blobs::BLOBS_ACTOR_ADDR,
        sequence: 0,
        value: Default::default(),
        method_num: GetStats as u64,
        params: RawBytes::default(),
        gas_limit: fvm_shared::BLOCK_GAS_LIMIT,
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };
    let (apply_ret, _) = state.execute_implicit(msg)?;

    let data: bytes::Bytes = apply_ret.msg_receipt.return_data.to_vec().into();
    fvm_ipld_encoding::from_slice::<GetStatsReturn>(&data)
        .map_err(|e| anyhow!("error parsing stats: {e}"))
}

/// Selects messages to be executed. Currently, this is a static function whose main purpose is to
/// coordinate various selectors. However, it does not have formal semantics for doing so, e.g.
/// do we daisy-chain selectors, do we parallelize, how do we treat rejections and acceptances?
/// It hasn't been well thought out yet. When we refactor the whole *Interpreter stack, we will
/// revisit this and make the selection function properly pluggable.
fn messages_selection<DB: Blockstore + Clone + 'static>(
    msgs: Vec<ChainMessage>,
    state: &FvmExecState<DB>,
) -> anyhow::Result<Vec<ChainMessage>> {
    let mut user_msgs = msgs
        .into_iter()
        .map(|msg| match msg {
            ChainMessage::Signed(inner) => Ok(inner),
            ChainMessage::Ipc(_) => Err(anyhow!("should not have ipc messages in user proposals")),
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    // Currently only one selector, we can potentially extend to more selectors
    // This selector enforces that the total cumulative gas limit of all messages is less than the
    // currently active block gas limit.
    let selectors = vec![GasLimitSelector {}];
    for s in selectors {
        user_msgs = s.select_messages(state, user_msgs)
    }

    Ok(user_msgs.into_iter().map(ChainMessage::Signed).collect())
}

fn get_open_read_requests<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    size: u32,
) -> anyhow::Result<Vec<OpenReadRequestItem>>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let params = GetOpenReadRequestsParams(size);
    let params = RawBytes::serialize(params)?;
    let msg = FvmMessage {
        from: system::SYSTEM_ACTOR_ADDR,
        to: readreq::READREQ_ACTOR_ADDR,
        sequence: 0,
        gas_limit: fvm_shared::BLOCK_GAS_LIMIT,
        method_num: GetOpenReadRequests as u64,
        params,
        value: Default::default(),
        version: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    let (apply_ret, _emitters) = state.execute_implicit(msg)?;
    if let Some(err) = apply_ret.failure_info {
        anyhow::bail!("failed to apply get read requests blob message: {}", err);
    } else {
        tracing::info!("get read requests blob message applied successfully");
    }

    let return_data: bytes::Bytes = apply_ret.msg_receipt.return_data.to_vec().into();
    let read_requests =
        fvm_ipld_encoding::from_slice::<Vec<OpenReadRequestItem>>(&return_data).unwrap();

    Ok(read_requests)
}

fn read_request_callback<DB>(
    state: &mut FvmExecState<DB>,
    read_request: &ClosedReadRequest,
) -> anyhow::Result<()>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let from = system::SYSTEM_ACTOR_ADDR;
    let ClosedReadRequest {
        id,
        blob_hash,
        offset,
        len,
        callback: (to, method_num),
        response,
    } = read_request.clone();
    // if request failed, we will send an empty vector back
    let msg = Message {
        version: Default::default(),
        from,
        to,
        sequence: 0,
        value: Default::default(),
        method_num,
        params: RawBytes::serialize(response)?,
        gas_limit: fvm_shared::BLOCK_GAS_LIMIT,
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    let (apply_ret, _emitters) = state.execute_implicit(msg)?;
    let info = apply_ret
        .failure_info
        .clone()
        .map(|i| i.to_string())
        .filter(|s| !s.is_empty());

    tracing::info!(
        exit_code = apply_ret.msg_receipt.exit_code.value(),
        from = from.to_string(),
        to = to.to_string(),
        method_num = method_num,
        read_request_id = id.to_string(),
        blob_hash = blob_hash.to_string(),
        read_offset = offset,
        read_len = len,
        gas_limit = fvm_shared::BLOCK_GAS_LIMIT,
        gas_used = apply_ret.msg_receipt.gas_used,
        info = info.unwrap_or_default(),
        "implicit tx delivered"
    );

    Ok(())
}

fn with_state_transaction<F, R, DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    f: F,
) -> anyhow::Result<R>
where
    F: FnOnce(&mut FvmExecState<ReadOnlyBlockstore<DB>>) -> anyhow::Result<R>,
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    state.state_tree_mut().begin_transaction();
    let result = f(state);
    state
        .state_tree_mut()
        .end_transaction(true)
        .expect("we just started a transaction");
    result
}

fn close_read_request<DB>(state: &mut FvmExecState<DB>, id: Hash) -> anyhow::Result<FvmApplyRet>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let from = system::SYSTEM_ACTOR_ADDR;
    let to = readreq::READREQ_ACTOR_ADDR;
    let method_num = CloseReadRequest as u64;
    let gas_limit = fvm_shared::BLOCK_GAS_LIMIT;
    let params = RawBytes::serialize(CloseReadRequestParams(fendermint_actor_readreq::Hash(
        *id.as_bytes(),
    )))?;
    let msg = Message {
        version: Default::default(),
        from,
        to,
        sequence: 0,
        value: Default::default(),
        method_num,
        params,
        gas_limit,
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    let (apply_ret, emitters) = state.execute_implicit(msg)?;
    let ret = FvmApplyRet {
        apply_ret,
        from,
        to,
        method_num,
        gas_limit,
        emitters,
    };
    Ok(ret)
}

fn get_read_request_status<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    id: Hash,
) -> anyhow::Result<Option<ReadRequestStatus>>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let request_id = fendermint_actor_readreq::Hash(*id.as_bytes());
    let msg = FvmMessage {
        from: system::SYSTEM_ACTOR_ADDR,
        to: readreq::READREQ_ACTOR_ADDR,
        sequence: 0,
        gas_limit: fvm_shared::BLOCK_GAS_LIMIT,
        method_num: GetReadRequestStatus as u64,
        params: RawBytes::serialize(GetReadRequestStatusParams(request_id))?,
        value: Default::default(),
        version: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    let (apply_ret, _emitters) = state.execute_implicit(msg)?;
    let data: bytes::Bytes = apply_ret.msg_receipt.return_data.to_vec().into();
    fvm_ipld_encoding::from_slice::<Option<ReadRequestStatus>>(&data)
        .map_err(|e| anyhow!("error parsing as Option<ReadRequestStatus>: {e}"))
}
