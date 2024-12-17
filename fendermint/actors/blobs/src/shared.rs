// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub use crate::state::State;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::bigint::BigInt;

pub const BLOBS_ACTOR_NAME: &str = "blobs";

/// Params for actor construction.
/// TODO: Remove constructor params
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    /// The total storage capacity of the subnet.
    pub blob_capacity: u64,
    /// The token to credit rate. The amount of atto credits that 1 atto buys.
    pub token_credit_rate: BigInt,
}
