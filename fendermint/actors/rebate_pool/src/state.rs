// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;

use crate::Alpha;

/// Rebate Pool actor state.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    alpha: Alpha,
}

impl State {
    pub fn new(alpha: Alpha) -> Self {
        Self { alpha }
    }
}
