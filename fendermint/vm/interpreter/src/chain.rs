// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::fvm::state::ipc::GatewayCaller;
use crate::fvm::{topdown, FvmApplyRet, PowerUpdates};
use crate::{
    fvm::state::FvmExecState,
    fvm::FvmMessage,
    signed::{SignedMessageApplyRes, SignedMessageCheckRes, SyntheticMessage, VerifiableMessage},
    CheckInterpreter, ExecInterpreter, GenesisInterpreter, ProposalInterpreter, QueryInterpreter,
};
use anyhow::{bail, Context};
use async_stm::atomically;
use async_trait::async_trait;
use fendermint_vm_actor_interface::{fingerprint, ipc, objectstore, system};
use fendermint_vm_fingerprint_resolver::pool::{
    FingerprintTask, ResolveKey as FingerprintResolveKey, ResolvePool as FingerprintResolvePool,
};
use fendermint_vm_ipfs_resolver::pool::{
    ResolveKey as IpfsResolveKey, ResolvePool as IpfsResolvePool,
};
use fendermint_vm_message::ipc::ParentFinality;
use fendermint_vm_message::signed::Object;
use fendermint_vm_message::{
    chain::ChainMessage,
    ipc::{BottomUpCheckpoint, CertifiedMessage, IpcMessage, SignedRelayedMessage},
};
use fendermint_vm_resolver::pool::{ResolveKey, ResolvePool};
use fendermint_vm_topdown::proxy::IPCProviderProxy;
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
use num_traits::Zero;
use std::sync::Arc;

/// A resolution pool for bottom-up and top-down checkpoints.
pub type CheckpointPool = ResolvePool<CheckpointPoolItem>;
pub type TopDownFinalityProvider = Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>;
pub type ObjectPool = IpfsResolvePool<ObjectPoolItem>;
pub type FingerprintPool = FingerprintResolvePool<FingerprintPoolItem>;

/// These are the extra state items that the chain interpreter needs,
/// a sort of "environment" supporting IPC.
#[derive(Clone)]
pub struct ChainEnv {
    /// CID resolution pool.
    pub checkpoint_pool: CheckpointPool,
    /// The parent finality provider for top down checkpoint
    pub parent_finality_provider: TopDownFinalityProvider,
    pub parent_finality_votes: VoteTally,
    /// IPFS pin resolution pool.
    pub object_pool: ObjectPool,
    /// Fingerprint resolution pool.
    pub fingerprint_pool: FingerprintPool,
    /// Chain ids of the chains where the fingerprint is submitted.
    pub fingerprint_chains: Vec<u64>,
    /// Number of blocks between fingerprinting rounds.
    pub fingerprint_interval: u64,
    /// Validator address
    pub validator_address: Option<Address>,
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
pub struct ObjectPoolItem {
    obj: Object,
}

impl From<&ObjectPoolItem> for IpfsResolveKey {
    fn from(value: &ObjectPoolItem) -> Self {
        value.obj.value
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct FingerprintPoolItem {
    fingerprint: cid::Cid,
    proposer: Address,
    proposed_at: ChainEpoch,
}

impl From<&FingerprintPoolItem> for FingerprintResolveKey {
    fn from(value: &FingerprintPoolItem) -> Self {
        value.fingerprint.clone()
    }
}

impl FingerprintTask for FingerprintPoolItem {
    fn proposer_address(&self) -> Address {
        self.proposer
    }

    fn proposed_at_height(&self) -> ChainEpoch {
        self.proposed_at
    }

    fn fingerprint(&self) -> cid::Cid {
        self.fingerprint.clone()
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
/// CID lookups to turn references into self-contained user or cross messages.
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
    type State = (ChainEnv, crate::fvm::state::FvmQueryState<DB>);
    type Message = ChainMessage;

    /// Check whether there are any "ready" messages in the IPLD resolution mempool which can be appended to the proposal.
    ///
    /// We could also use this to select the most profitable user transactions, within the gas limit. We can also take into
    /// account the transactions which are part of top-down or bottom-up checkpoints, to stay within gas limits.
    async fn prepare(
        &self,
        (env, state): Self::State,
        mut msgs: Vec<Self::Message>,
    ) -> anyhow::Result<Vec<Self::Message>> {
        // Collect resolved CIDs ready to be proposed from the pool.
        let ckpts = atomically(|| env.checkpoint_pool.collect_resolved()).await;

        // Create transactions ready to be included on the chain.
        let ckpts = ckpts.into_iter().map(|ckpt| match ckpt {
            CheckpointPoolItem::BottomUp(ckpt) => ChainMessage::Ipc(IpcMessage::BottomUpExec(ckpt)),
        });

        // Collect resolved objects ready to be proposed from the pool.
        let objects = atomically(|| env.object_pool.collect_resolved()).await;

        // Create transactions ready to be included on the chain.
        let objects = objects
            .into_iter()
            .map(|item| ChainMessage::Ipc(IpcMessage::ObjectResolved(item.obj)));

        // Collect sent fingerprints ready to be proposed from the pool.
        let fingerprints = atomically(|| env.fingerprint_pool.collect_resolved()).await;

        // Prepare top down proposals.
        // Before we try to find a quorum, pause incoming votes. This is optional but if there are lots of votes coming in it might hold up proposals.
        atomically(|| env.parent_finality_votes.pause_votes_until_find_quorum()).await;

        // The pre-requisite for proposal is that there is a quorum of gossiped votes at that height.
        // The final proposal can be at most as high as the quorum, but can be less if we have already,
        // hit some limits such as how many blocks we can propose in a single step.
        let maybe_finality = atomically(|| {
            if let Some((quorum_height, quorum_hash)) = env.parent_finality_votes.find_quorum()? {
                if let Some(finality) = env.parent_finality_provider.next_proposal()? {
                    let finality = if finality.height <= quorum_height {
                        finality
                    } else {
                        IPCParentFinality {
                            height: quorum_height,
                            block_hash: quorum_hash,
                        }
                    };
                    return Ok(Some(finality));
                }
            }
            Ok(None)
        })
        .await;

        if let Some(finality) = maybe_finality {
            msgs.push(ChainMessage::Ipc(IpcMessage::TopDownExec(ParentFinality {
                height: finality.height as ChainEpoch,
                block_hash: finality.block_hash,
            })))
        }

        // Append at the end - if we run out of block space, these are going to be reproposed in the next block.
        msgs.extend(ckpts);
        msgs.extend(objects);

        // Create transactions ready to be included on the chain.
        // If the node is validating, then we should peek into the fingerprint pool
        // and see if there are any fingerprints that are ready to be proposed.
        if let Some(block_proposer) = env.validator_address {
            let fingerprints = fingerprints.into_iter().filter_map(|item| {
                if item.proposer != block_proposer {
                    tracing::info!(
                        "FingerprintVerified({}, {}, {})",
                        item.fingerprint.to_string(),
                        item.proposer.to_string(),
                        item.proposed_at
                    );
                    Some(ChainMessage::Ipc(IpcMessage::FingerprintVerified(
                        item.fingerprint,
                        item.proposer,
                        item.proposed_at,
                    )))
                } else {
                    None
                }
            });
            msgs.extend(fingerprints);

            // extend msgs with fingerprint ready transactions
            // Fingerprinting Rounds.
            //
            // Every 20 blocks, we will submit a task to the background worker
            // to send the fingerprint to pre-configured chains on an predefined
            // contract. When the fingerprint will be sent. The block proposer will
            // included that fingerprint in a new transaction. The validator will
            // verify that the fingerprint is correct and also exists in the
            // desitination chain(s).
            //
            // If the fingerprint is correct, the validator will mint rewards for
            // the block proposer.
            let block_height = state.block_height();
            if block_height as u64 % env.fingerprint_interval == 0 && block_height != 0 {
                let subnet_fingerprint = state.state_params().state_root;
                tracing::info!(
                    "FingerprintReady({}, {}, {})",
                    subnet_fingerprint.to_string(),
                    block_proposer.to_string(),
                    block_height
                );
                msgs.push(ChainMessage::Ipc(IpcMessage::FingerprintReady(
                    subnet_fingerprint,
                    block_proposer,
                    block_height,
                )));
            }
        }

        Ok(msgs)
    }

    /// Perform finality checks on top-down transactions and availability checks on bottom-up transactions.
    async fn process(
        &self,
        (env, _): Self::State,
        msgs: Vec<Self::Message>,
    ) -> anyhow::Result<bool> {
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
                        atomically(|| match env.checkpoint_pool.get_status(&item)? {
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
                        atomically(|| env.parent_finality_provider.check_proposal(&prop)).await;
                    if !is_final {
                        return Ok(false);
                    }
                }
                ChainMessage::Ipc(IpcMessage::ObjectResolved(obj)) => {
                    let item = ObjectPoolItem { obj };

                    let is_resolved = atomically(|| match env.object_pool.get_status(&item)? {
                        None => Ok(false),
                        Some(status) => status.is_resolved(),
                    })
                    .await;

                    if !is_resolved {
                        return Ok(false);
                    }

                    atomically(|| env.object_pool.remove(&item)).await;
                }
                ChainMessage::Ipc(IpcMessage::FingerprintVerified(key, address, heght)) => {
                    let item = FingerprintPoolItem {
                        fingerprint: key,
                        proposer: address,
                        proposed_at: heght,
                    };

                    let is_resolved =
                        atomically(|| match env.fingerprint_pool.get_status(&item)? {
                            None => Ok(false),
                            Some(status) => status.is_resolved(),
                        })
                        .await;

                    if !is_resolved {
                        return Ok(false);
                    }

                    atomically(|| env.fingerprint_pool.remove(&item)).await;
                }
                _ => {}
            };
        }
        Ok(true)
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
        EndOutput = PowerUpdates,
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

                if ret.is_ok() {
                    if let Some(obj) = msg.object {
                        atomically(|| env.object_pool.add(ObjectPoolItem { obj: obj.clone() }))
                            .await;
                    }
                }

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
                        // For now try to get it from the child subnet. If the same comes up for execution, include own.
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

                    // The commitment of the finality for block `N` triggers
                    // the execution of all side-effects up till `N-1`, as for
                    // deferred execution chains, this is the latest state that
                    // we know for sure that we have available.
                    let execution_fr = prev_height;
                    let execution_to = finality.height - 1;

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

                    // error happens if we cannot get the cross messages from ipc agent after retries
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

                    atomically(|| {
                        env.parent_finality_provider
                            .set_new_finality(finality.clone(), prev_finality.clone())?;

                        env.parent_finality_votes
                            .set_finalized(finality.height, finality.block_hash.clone())?;

                        Ok(())
                    })
                    .await;

                    tracing::debug!(
                        finality = finality.to_string(),
                        "chain interpreter has set new"
                    );

                    Ok(((env, state), ChainMessageApplyRet::Ipc(ret)))
                }
                IpcMessage::ObjectResolved(obj) => {
                    let from = system::SYSTEM_ACTOR_ADDR;
                    let to = objectstore::OBJECTSTORE_ACTOR_ADDR;
                    let method_num = fendermint_actor_objectstore::Method::ResolveObject as u64;
                    let gas_limit = 10_000_000_000; // max

                    // TODO(sander): Clean up with From.
                    let input = fendermint_actor_objectstore::ObjectParams {
                        key: obj.key,
                        value: obj.value,
                    };
                    let params = RawBytes::serialize(input)?;
                    let msg = Message {
                        version: Default::default(),
                        from,
                        to,
                        sequence: 0, // TODO(sander): This works but is it okay?
                        value: TokenAmount::zero(),
                        method_num,
                        params,
                        gas_limit,
                        gas_fee_cap: TokenAmount::zero(),
                        gas_premium: TokenAmount::zero(),
                    };

                    let (apply_ret, emitters) = state.execute_implicit(msg)?;

                    let ret = FvmApplyRet {
                        apply_ret,
                        from: system::SYSTEM_ACTOR_ADDR,
                        to,
                        method_num,
                        gas_limit,
                        emitters,
                    };

                    Ok(((env, state), ChainMessageApplyRet::Ipc(ret)))
                }
                IpcMessage::FingerprintReady(_, proposer, proposed_at) => {
                    // how to get the state root at height = `proposed_at`?
                    let fingerprint = state.current_state_root().unwrap();
                    let chain_ids = env.fingerprint_chains.clone();

                    // Schedule the task on the local node
                    // For the block proposer, the task should send the fingerprint to the
                    // external chains.
                    //
                    // For the validator nodes, the task should verify that:
                    // 1. fingerprint matches their local state root
                    // 2. fingerprint exists in the destination chain(s)
                    atomically(|| {
                        env.fingerprint_pool.add(FingerprintPoolItem {
                            fingerprint,
                            proposer,
                            proposed_at,
                        })
                    })
                    .await;

                    let from = system::SYSTEM_ACTOR_ADDR;
                    let to = fingerprint::FINGERPRINT_ACTOR_ADDR;
                    let method_num = fendermint_actor_fingerprint::Method::SetPending as u64;
                    let gas_limit = 10_000_000_000;
                    let params =
                        RawBytes::serialize(fendermint_actor_fingerprint::FingerprintParams {
                            proposer: proposer.to_bytes(),
                            height: proposed_at as i64,
                            fingerprint: fingerprint.to_bytes(),
                            chain_ids,
                        })?;

                    let msg = Message {
                        version: Default::default(),
                        from,
                        to,
                        sequence: 0, // TODO This works but is it okay?
                        value: TokenAmount::zero(),
                        method_num,
                        params,
                        gas_limit,
                        gas_fee_cap: TokenAmount::zero(),
                        gas_premium: TokenAmount::zero(),
                    };

                    let (apply_ret, emitters) = state.execute_implicit(msg)?;

                    let ret = FvmApplyRet {
                        apply_ret,
                        from: system::SYSTEM_ACTOR_ADDR,
                        to,
                        method_num,
                        gas_limit,
                        emitters,
                    };

                    tracing::info!(
                        "FVM exec: FingerprintReady({}, {}, {})",
                        fingerprint.to_string(),
                        proposer.to_string(),
                        proposed_at
                    );

                    Ok(((env, state), ChainMessageApplyRet::Ipc(ret)))
                }
                IpcMessage::FingerprintVerified(fingerprint, proposer, proposed_at) => {
                    let from = system::SYSTEM_ACTOR_ADDR;
                    let to = fingerprint::FINGERPRINT_ACTOR_ADDR;
                    let method_num = fendermint_actor_fingerprint::Method::SetVerified as u64;
                    let gas_limit = 10_000_000_000;
                    let params = RawBytes::serialize(fingerprint.to_bytes())?;

                    let msg = Message {
                        version: Default::default(),
                        from,
                        to,
                        sequence: 0, // TODO This works but is it okay?
                        value: TokenAmount::zero(),
                        method_num,
                        params,
                        gas_limit,
                        gas_fee_cap: TokenAmount::zero(),
                        gas_premium: TokenAmount::zero(),
                    };

                    let (apply_ret, emitters) = state.execute_implicit(msg)?;

                    let ret = FvmApplyRet {
                        apply_ret,
                        from: system::SYSTEM_ACTOR_ADDR,
                        to,
                        method_num,
                        gas_limit,
                        emitters,
                    };

                    tracing::info!(
                        "FVM exec: FingerprintVerified({}, {}, {})",
                        fingerprint.to_string(),
                        proposer.to_string(),
                        proposed_at
                    );

                    Ok(((env, state), ChainMessageApplyRet::Ipc(ret)))
                }
            },
        }
    }

    async fn begin(
        &self,
        (env, state): Self::State,
    ) -> anyhow::Result<(Self::State, Self::BeginOutput)> {
        let (state, out) = self.inner.begin(state).await?;
        Ok(((env, state), out))
    }

    async fn end(
        &self,
        (env, state): Self::State,
    ) -> anyhow::Result<(Self::State, Self::EndOutput)> {
        let (state, out) = self.inner.end(state).await?;

        // Update any component that needs to know about changes in the power table.
        if !out.0.is_empty() {
            let power_updates = out
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
                    | IpcMessage::ObjectResolved(_)
                    | IpcMessage::FingerprintReady(_, _, _)
                    | IpcMessage::FingerprintVerified(_, _, _) => {
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

#[async_trait]
impl<I, DB> GenesisInterpreter for ChainMessageInterpreter<I, DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    I: GenesisInterpreter,
{
    type State = I::State;
    type Genesis = I::Genesis;
    type Output = I::Output;

    async fn init(
        &self,
        state: Self::State,
        genesis: Self::Genesis,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        self.inner.init(state, genesis).await
    }
}

/// Convert a signed relayed bottom-up checkpoint to a syntetic message we can send to the FVM.
///
/// By mapping to an FVM message we invoke the right contract to validate the checkpoint,
/// and automatically charge the relayer gas for the execution of the check, but not the
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
