// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::{
    actor_dispatch, actor_error, extract_send_result,
    runtime::{ActorCode, Runtime},
    ActorError, BURNT_FUNDS_ACTOR_ADDR, FIRST_EXPORTED_METHOD_NUMBER, SYSTEM_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::{MethodNum, METHOD_SEND};

use crate::state::State;
use crate::{ConstructorParams, Method, REBATE_POOL_ACTOR_NAME};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(RebatePoolActor);

pub struct RebatePoolActor;

impl ActorCode for RebatePoolActor {
    type Methods = Method;

    actor_dispatch! {
        Constructor => constructor,
        AcceptForRebate => accept_for_rebate,
        _ => fallback,
    }

    fn name() -> &'static str {
        REBATE_POOL_ACTOR_NAME
    }
}

/// Accumulate tokens for distribution at a later time.
/// Currently, 50% of the tokens get burned.
/// Parameter `alpha` is only used as a stub. Not actually used for calculation.
/// The hardcoded params should be moved to genesis once we have storage commitments and the relevant calculation.
impl RebatePoolActor {
    pub fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let state = State::new(params.alpha);
        rt.create(&state)
    }

    /// Send 50% of the value received to `BURNT_FUNDS_ACTOR_ADDR`.
    pub fn accept_for_rebate(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let value_received = rt.message().value_received();
        let amount_to_retire = value_received.clone().div_floor(2); // 50%
        extract_send_result(rt.send_simple(
            &BURNT_FUNDS_ACTOR_ADDR,
            METHOD_SEND,
            None,
            amount_to_retire,
        ))?;
        Ok(())
    }

    /// Fallback method for unimplemented method numbers.
    pub fn fallback(
        rt: &impl Runtime,
        method: MethodNum,
        _: Option<IpldBlock>,
    ) -> Result<Option<IpldBlock>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        if method >= FIRST_EXPORTED_METHOD_NUMBER {
            Ok(None)
        } else {
            Err(actor_error!(unhandled_message; "invalid method: {}", method))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fil_actors_runtime::test_utils::{expect_empty, MockRuntime, SYSTEM_ACTOR_CODE_ID};
    use fvm_shared::address::Address;
    use fvm_shared::econ::TokenAmount;
    use fvm_shared::error::ExitCode;
    use std::ops::Sub;

    pub fn construct_and_verify() -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let result = rt
            .call::<RebatePoolActor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&ConstructorParams { alpha: (0, 1) }).unwrap(),
            )
            .unwrap();
        expect_empty(result);
        rt.verify();
        rt.reset();
        rt
    }

    #[test]
    pub fn test_accept_for_rebate() {
        let rt = construct_and_verify();
        let value_sent = TokenAmount::from_whole(10);
        let initial_balance = TokenAmount::from_whole(11);
        let to_retire = value_sent.clone().div_floor(2);
        rt.set_received(value_sent.clone());
        rt.set_balance(initial_balance.clone());
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BURNT_FUNDS_ACTOR_ADDR,
            METHOD_SEND,
            None,
            to_retire.clone(),
            None,
            ExitCode::OK,
        );
        rt.call::<RebatePoolActor>(Method::AcceptForRebate as u64, None)
            .unwrap();
        rt.verify();
        assert_eq!(rt.get_balance(), initial_balance.sub(to_retire));
    }
}
