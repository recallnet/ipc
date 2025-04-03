// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use arbitrary::{Arbitrary, Unstructured};
use fvm_shared::{bigint::Integer, econ::TokenAmount};
use recall_fendermint_testing::arb::ArbTokenAmount;

pub mod machine;
pub mod state;

fn choose_amount(u: &mut Unstructured<'_>, max: &TokenAmount) -> arbitrary::Result<TokenAmount> {
    if max.is_zero() {
        Ok(TokenAmount::from_atto(0))
    } else {
        let tokens = ArbTokenAmount::arbitrary(u)?.0;
        Ok(TokenAmount::from_atto(tokens.atto().mod_floor(max.atto())))
    }
}
