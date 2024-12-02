// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::{ActorID, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;

pub const REGISTRY_ACTOR_ID: ActorID = 77;
pub const REGISTRY_ACTOR_ADDR: Address = Address::new_id(REGISTRY_ACTOR_ID);

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    SetExtendedTTL = frc42_dispatch::method_hash!("SetExtendedTTL"),
    SetNoTTL = frc42_dispatch::method_hash!("SetNoTTL"),
    SetDefaultTTL = frc42_dispatch::method_hash!("SetDefaultTTL"),
    SetCustomTTL = frc42_dispatch::method_hash!("SetCustomTTL"),
    GetTTL = frc42_dispatch::method_hash!("GetTTL"),
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone, Eq, PartialEq)]
#[serde(transparent)]
pub struct GetTTLParams {
    pub target: Address,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Default)]
pub struct GetTTLReturn {
    pub ttl: ChainEpoch,
}
