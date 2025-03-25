// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::{deserialize_block, extract_send_result, runtime::Runtime, ActorError};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::{address::Address, sys::SendFlags, ActorID, MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;

use crate::state::{CreditApproval, Subscription};

pub mod params;
pub mod state;

/// The unique identifier for the blob actor in the system.
pub const BLOBS_ACTOR_ID: ActorID = 66;
/// The address of the blob actor, derived from its actor ID.
pub const BLOBS_ACTOR_ADDR: Address = Address::new_id(BLOBS_ACTOR_ID);

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,

    // EVM Interop
    InvokeContract = frc42_dispatch::method_hash!("InvokeEVM"),

    // User methods
    BuyCredit = frc42_dispatch::method_hash!("BuyCredit"),
    ApproveCredit = frc42_dispatch::method_hash!("ApproveCredit"),
    RevokeCredit = frc42_dispatch::method_hash!("RevokeCredit"),
    SetAccountSponsor = frc42_dispatch::method_hash!("SetAccountSponsor"),
    GetAccount = frc42_dispatch::method_hash!("GetAccount"),
    GetCreditApproval = frc42_dispatch::method_hash!("GetCreditApproval"),
    AddBlob = frc42_dispatch::method_hash!("AddBlob"),
    GetBlob = frc42_dispatch::method_hash!("GetBlob"),
    DeleteBlob = frc42_dispatch::method_hash!("DeleteBlob"),
    OverwriteBlob = frc42_dispatch::method_hash!("OverwriteBlob"),

    // System methods
    GetGasAllowance = frc42_dispatch::method_hash!("GetGasAllowance"),
    UpdateGasAllowance = frc42_dispatch::method_hash!("UpdateGasAllowance"),
    GetBlobStatus = frc42_dispatch::method_hash!("GetBlobStatus"),
    GetAddedBlobs = frc42_dispatch::method_hash!("GetAddedBlobs"),
    GetPendingBlobs = frc42_dispatch::method_hash!("GetPendingBlobs"),
    SetBlobPending = frc42_dispatch::method_hash!("SetBlobPending"),
    FinalizeBlob = frc42_dispatch::method_hash!("FinalizeBlob"),
    DebitAccounts = frc42_dispatch::method_hash!("DebitAccounts"),

    // Admin methods
    SetAccountStatus = frc42_dispatch::method_hash!("SetAccountStatus"),
    TrimBlobExpiries = frc42_dispatch::method_hash!("TrimBlobExpiries"),

    // Metrics methods
    GetStats = frc42_dispatch::method_hash!("GetStats"),
}

/// Returns a credit approval from one account to another if it exists.
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

/// Adds a blob.
pub fn add_blob(
    rt: &impl Runtime,
    params: params::AddBlobParams,
) -> Result<Subscription, ActorError> {
    let params = IpldBlock::serialize_cbor(&params)?;
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::AddBlob as MethodNum,
        params,
        rt.message().value_received(),
    ))?)
}

/// Returns information about a blob.
pub fn get_blob(
    rt: &impl Runtime,
    params: params::GetBlobParams,
) -> Result<Option<state::BlobInfo>, ActorError> {
    deserialize_block(extract_send_result(rt.send(
        &BLOBS_ACTOR_ADDR,
        Method::GetBlob as MethodNum,
        IpldBlock::serialize_cbor(&params)?,
        rt.message().value_received(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}

/// Deletes a blob.
pub fn delete_blob(rt: &impl Runtime, params: params::DeleteBlobParams) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::DeleteBlob as MethodNum,
        IpldBlock::serialize_cbor(&params)?,
        rt.message().value_received(),
    ))?;
    Ok(())
}

/// Overwrite a blob, i.e., delete one and add another in a single call.
pub fn overwrite_blob(
    rt: &impl Runtime,
    params: params::OverwriteBlobParams,
) -> Result<Subscription, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::OverwriteBlob as MethodNum,
        IpldBlock::serialize_cbor(&params)?,
        rt.message().value_received(),
    ))?)
}
