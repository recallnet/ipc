// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::{ActorID, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;

pub const REBATE_POOL_ACTOR_NAME: &str = "rebate_pool";
pub const REBATE_POOL_ACTOR_ID: ActorID = 50;
pub const REBATE_POOL_ACTOR_ADDR: Address = Address::new_id(REBATE_POOL_ACTOR_ID);

/// Alpha is a floating number of range [0, 1].
/// We better not use floating arithmetic though, so it gets represented as a (numerator, denominator) part.
pub type Alpha = (u32, u32);

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
}

/// Params for actor construction.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    /// Alpha parameter for Cobb-Douglas based share determination.
    pub alpha: Alpha,
}
