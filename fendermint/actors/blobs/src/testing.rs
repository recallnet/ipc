// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::params::{AddBlobParams, BuyCreditParams};
use fendermint_actor_blobs_shared::Method;
use fendermint_actor_machine::{events::to_actor_event, util::token_to_biguint};
use fendermint_actor_recall_config_shared::{RecallConfig, RECALL_CONFIG_ACTOR_ADDR};
use fil_actors_runtime::test_utils::{expect_empty, MockRuntime, SYSTEM_ACTOR_CODE_ID};
use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::{
    address::Address, clock::ChainEpoch, econ::TokenAmount, error::ExitCode, sys::SendFlags,
    MethodNum,
};
use num_traits::Zero;
use recall_sol_facade::{
    blobs::blob_added,
    credit::{credit_approved, credit_purchased, credit_revoked},
};

use crate::actor::BlobsActor;
use crate::State;

pub fn construct_and_verify() -> MockRuntime {
    let rt = MockRuntime {
        receiver: Address::new_id(10),
        ..Default::default()
    };
    rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
    rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
    let result = rt
        .call::<BlobsActor>(Method::Constructor as u64, None)
        .unwrap();
    expect_empty(result);
    rt.verify();
    rt.reset();
    rt
}

pub fn expect_get_config(rt: &MockRuntime) {
    rt.expect_send(
        RECALL_CONFIG_ACTOR_ADDR,
        fendermint_actor_recall_config_shared::Method::GetConfig as MethodNum,
        None,
        TokenAmount::zero(),
        None,
        SendFlags::READ_ONLY,
        IpldBlock::serialize_cbor(&RecallConfig::default()).unwrap(),
        ExitCode::OK,
        None,
    );
}

pub fn expect_emitted_purchase_event(
    rt: &MockRuntime,
    params: &BuyCreditParams,
    amount: TokenAmount,
) {
    let event = to_actor_event(credit_purchased(params.0, token_to_biguint(Some(amount))).unwrap())
        .unwrap();
    rt.expect_emitted_event(event);
}

pub fn expect_emitted_approve_event(
    rt: &MockRuntime,
    from: Address,
    to: Address,
    credit_limit: Option<TokenAmount>,
    gas_fee_limit: Option<TokenAmount>,
    expiry: ChainEpoch,
) {
    let credit_limit = token_to_biguint(credit_limit);
    let gas_fee_limit = token_to_biguint(gas_fee_limit);
    let event = to_actor_event(
        credit_approved(from, to, credit_limit, gas_fee_limit, expiry as u64).unwrap(),
    )
    .unwrap();
    rt.expect_emitted_event(event);
}

pub fn expect_emitted_revoke_event(rt: &MockRuntime, from: Address, to: Address) {
    let event = to_actor_event(credit_revoked(from, to).unwrap()).unwrap();
    rt.expect_emitted_event(event);
}

pub fn expect_emitted_add_event(
    rt: &MockRuntime,
    current_epoch: ChainEpoch,
    params: &AddBlobParams,
    subscriber: Address,
    used: u64,
) {
    let event = to_actor_event(
        blob_added(
            subscriber,
            &params.hash.0,
            params.size,
            (params.ttl.unwrap_or(86400) + current_epoch) as u64,
            used,
        )
        .unwrap(),
    )
    .unwrap();
    rt.expect_emitted_event(event);
}

pub fn check_approval_used<BS: Blockstore>(
    state: &State,
    store: &BS,
    caller: Address,
    sponsor: Address,
) {
    assert_ne!(caller, sponsor);
    let subscriber_account = state.get_account(&store, sponsor).unwrap().unwrap();
    let subscriber_approval = subscriber_account
        .approvals_to
        .hamt(store)
        .unwrap()
        .get(&caller)
        .unwrap()
        .unwrap();
    assert_eq!(
        subscriber_approval.credit_used,
        state.credit_debited.clone() + subscriber_account.credit_committed.clone()
    );
    let origin_account = state.get_account(&store, caller).unwrap().unwrap();
    let origin_approval = origin_account
        .approvals_from
        .hamt(store)
        .unwrap()
        .get(&sponsor)
        .unwrap()
        .unwrap();
    assert_eq!(
        subscriber_approval.credit_used,
        &state.credit_debited + &subscriber_account.credit_committed
    );
    assert_eq!(subscriber_approval.credit_used, origin_approval.credit_used);
}
