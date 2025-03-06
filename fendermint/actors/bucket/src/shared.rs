// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_machine::{
    GET_ADDRESS_METHOD, GET_METADATA_METHOD, INIT_METHOD, METHOD_CONSTRUCTOR,
};
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

pub use crate::state::State;

pub const BUCKET_ACTOR_NAME: &str = "bucket";
pub const MAX_METADATA_ENTRIES: u32 = 20;
pub const MAX_METADATA_KEY_SIZE: u32 = 32;
pub const MAX_METADATA_VALUE_SIZE: u32 = 128;

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Init = INIT_METHOD,
    GetAddress = GET_ADDRESS_METHOD,
    GetMetadata = GET_METADATA_METHOD,
    AddObject = frc42_dispatch::method_hash!("AddObject"),
    DeleteObject = frc42_dispatch::method_hash!("DeleteObject"),
    GetObject = frc42_dispatch::method_hash!("GetObject"),
    ListObjects = frc42_dispatch::method_hash!("ListObjects"),
    UpdateObjectMetadata = frc42_dispatch::method_hash!("UpdateObjectMetadata"),
    // EVM Interop
    InvokeContract = frc42_dispatch::method_hash!("InvokeEVM"),
}

/// Params for getting an object.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetParams(#[serde(with = "strict_bytes")] pub Vec<u8>);

// FIXME SU This should be moved to bucket shared crate