// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::{
    actor_dispatch, actor_error,
    builtin::singletons::SYSTEM_ACTOR_ADDR,
    runtime::{ActorCode, Runtime},
    ActorDowncast, ActorError, FIRST_EXPORTED_METHOD_NUMBER,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::{error::ExitCode, MethodNum};

use crate::{Method, State, SCALARSTORE_ACTOR_NAME};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

impl Actor {
    fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        // FIXME:(carsonfarmer) We're setting this up to be a subnet-wide actor for a single repo.
        // FIXME:(carsonfarmer) In the future, this could be deployed dynamically for multi repo subnets.
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let state = State::new(rt.store()).map_err(|e| {
            e.downcast_default(
                ExitCode::USR_ILLEGAL_STATE,
                "failed to construct empty store",
            )
        })?;
        rt.create(&state)
    }

    fn store_number(rt: &impl Runtime, num: u64) -> Result<(), ActorError> {
        // FIXME:(carsonfarmer) We'll want to validate the caller is the owner of the repo.
        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, rt| {
            st.store_number(rt.store(), num).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to store number")
            })
        })
    }

    fn get_number(rt: &impl Runtime) -> Result<u64, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        st.get_number(rt.store())
            .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to get number"))
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

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        SCALARSTORE_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        StoreNumber => store_number,
        GetNumber => get_number,
        _ => fallback,
    }
}
