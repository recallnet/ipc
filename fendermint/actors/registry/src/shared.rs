// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

pub use crate::state::State;

pub const REGISTRY_ACTOR_NAME: &str = "registry";

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
