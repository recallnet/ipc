// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;

use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::sys::SendFlags;
use fvm_shared::{ActorID, MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;
use recall_fil_actors_runtime::runtime::Runtime;
use recall_fil_actors_runtime::{deserialize_block, extract_send_result, ActorError};

use crate::state::{Account, Credit, CreditApproval, Subscription};

pub mod params;
pub mod state;

pub const BLOBS_ACTOR_ID: ActorID = 66;
pub const BLOBS_ACTOR_ADDR: Address = Address::new_id(BLOBS_ACTOR_ID);

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,

    // User methods
    BuyCredit = recall_frc42_dispatch::method_hash!("BuyCredit"),
    ApproveCredit = recall_frc42_dispatch::method_hash!("ApproveCredit"),
    RevokeCredit = recall_frc42_dispatch::method_hash!("RevokeCredit"),
    SetAccountSponsor = recall_frc42_dispatch::method_hash!("SetAccountSponsor"),
    GetAccount = recall_frc42_dispatch::method_hash!("GetAccount"),
    GetCreditApproval = recall_frc42_dispatch::method_hash!("GetCreditApproval"),
    AddBlob = recall_frc42_dispatch::method_hash!("AddBlob"),
    GetBlob = recall_frc42_dispatch::method_hash!("GetBlob"),
    DeleteBlob = recall_frc42_dispatch::method_hash!("DeleteBlob"),
    OverwriteBlob = recall_frc42_dispatch::method_hash!("OverwriteBlob"),

    // System methods
    GetGasAllowance = recall_frc42_dispatch::method_hash!("GetGasAllowance"),
    UpdateGasAllowance = recall_frc42_dispatch::method_hash!("UpdateGasAllowance"),
    GetBlobStatus = recall_frc42_dispatch::method_hash!("GetBlobStatus"),
    GetAddedBlobs = recall_frc42_dispatch::method_hash!("GetAddedBlobs"),
    GetPendingBlobs = recall_frc42_dispatch::method_hash!("GetPendingBlobs"),
    SetBlobPending = recall_frc42_dispatch::method_hash!("SetBlobPending"),
    FinalizeBlob = recall_frc42_dispatch::method_hash!("FinalizeBlob"),
    DebitAccounts = recall_frc42_dispatch::method_hash!("DebitAccounts"),

    // Admin methods
    SetAccountStatus = recall_frc42_dispatch::method_hash!("SetAccountStatus"),
    TrimBlobExpiries = recall_frc42_dispatch::method_hash!("TrimBlobExpiries"),

    // Metrics methods
    GetStats = recall_frc42_dispatch::method_hash!("GetStats"),
    // EVM Interop
    InvokeContract = recall_frc42_dispatch::method_hash!("InvokeEVM"),
}

pub fn buy_credit(rt: &impl Runtime, to: Address) -> Result<Account, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::BuyCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::BuyCreditParams(to))?,
        rt.message().value_received(),
    ))?)
}

pub fn approve_credit(
    rt: &impl Runtime,
    from: Address,
    to: Address,
    caller_allowlist: Option<HashSet<Address>>,
    credit_limit: Option<Credit>,
    gas_fee_limit: Option<TokenAmount>,
    ttl: Option<ChainEpoch>,
) -> Result<CreditApproval, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::ApproveCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::ApproveCreditParams {
            from,
            to,
            caller_allowlist,
            credit_limit,
            gas_fee_limit,
            ttl,
        })?,
        rt.message().value_received(),
    ))?)
}

pub fn get_credit_approval(
    rt: &impl Runtime,
    from: Address,
    to: Address,
) -> Result<Option<CreditApproval>, ActorError> {
    let params = params::GetCreditApprovalParams { from, to };

    deserialize_block(extract_send_result(rt.send(
        &BLOBS_ACTOR_ADDR,
        Method::GetCreditApproval as MethodNum,
        IpldBlock::serialize_cbor(&params)?,
        rt.message().value_received(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}

/// Returns `true` if `from` and `to` are the same address,
/// or if `from` has a credit delegation to `to` that has not yet expired.
pub fn has_credit_approval(
    rt: &impl Runtime,
    from: Address,
    to: Address,
) -> Result<bool, ActorError> {
    if from != to {
        let approval = get_credit_approval(rt, from, to)?;
        let curr_epoch = rt.curr_epoch();
        Ok(approval.is_some_and(|a| a.expiry.map_or(true, |e| e >= curr_epoch)))
    } else {
        Ok(true)
    }
}

pub fn revoke_credit(
    rt: &impl Runtime,
    from: Address,
    to: Address,
    for_caller: Option<Address>,
) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::RevokeCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::RevokeCreditParams {
            from,
            to,
            for_caller,
        })?,
        rt.message().value_received(),
    ))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn add_blob(
    rt: &impl Runtime,
    from: Address,
    sub_id: state::SubscriptionId,
    hash: state::Hash,
    sponsor: Option<Address>,
    source: state::PublicKey,
    metadata_hash: state::Hash,
    size: u64,
    ttl: Option<ChainEpoch>,
) -> Result<Subscription, ActorError> {
    let params = IpldBlock::serialize_cbor(&params::AddBlobParams {
        sponsor,
        source,
        hash,
        metadata_hash,
        id: sub_id,
        size,
        ttl,
        from,
    })?;
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::AddBlob as MethodNum,
        params,
        rt.message().value_received(),
    ))?)
}

pub fn get_blob(
    rt: &impl Runtime,
    hash: state::Hash,
) -> Result<Option<state::BlobInfo>, ActorError> {
    deserialize_block(extract_send_result(rt.send(
        &BLOBS_ACTOR_ADDR,
        Method::GetBlob as MethodNum,
        IpldBlock::serialize_cbor(&params::GetBlobParams(hash))?,
        rt.message().value_received(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}

pub fn delete_blob(
    rt: &impl Runtime,
    from: Address,
    sub_id: state::SubscriptionId,
    hash: state::Hash,
    sponsor: Option<Address>,
) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::DeleteBlob as MethodNum,
        IpldBlock::serialize_cbor(&params::DeleteBlobParams {
            sponsor,
            hash,
            id: sub_id,
            from,
        })?,
        rt.message().value_received(),
    ))?;
    Ok(())
}

/// Overwrite a blob, i.e., delete one and add another in a single call.
#[allow(clippy::too_many_arguments)]
pub fn overwrite_blob(
    rt: &impl Runtime,
    from: Address,
    old_hash: state::Hash,
    sub_id: state::SubscriptionId,
    hash: state::Hash,
    sponsor: Option<Address>,
    source: state::PublicKey,
    metadata_hash: state::Hash,
    size: u64,
    ttl: Option<ChainEpoch>,
) -> Result<Subscription, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::OverwriteBlob as MethodNum,
        IpldBlock::serialize_cbor(&params::OverwriteBlobParams {
            old_hash,
            add: params::AddBlobParams {
                sponsor,
                id: sub_id,
                source,
                hash,
                metadata_hash,
                size,
                ttl,
                from,
            },
        })?,
        rt.message().value_received(),
    ))?)
}
