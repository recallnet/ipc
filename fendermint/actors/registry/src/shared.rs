// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_machine::METHOD_CONSTRUCTOR;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

pub use crate::state::State;

pub const REGISTRY_ACTOR_NAME: &str = "registry";

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

#[derive(Serialize, Deserialize, Debug)]
pub struct ConstructorParams {}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone, Eq, PartialEq)]
#[serde(transparent)]
pub struct SetExtendedTTLParams {
    pub target: Address,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone, Eq, PartialEq)]
#[serde(transparent)]
pub struct SetDefaultTTLParams {
    pub target: Address,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone, Eq, PartialEq)]
#[serde(transparent)]
pub struct SetNoTTLParams {
    pub target: Address,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone, Eq, PartialEq)]
pub struct SetCustomTTLParams {
    pub target: Address,
    pub ttl: ChainEpoch,
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
