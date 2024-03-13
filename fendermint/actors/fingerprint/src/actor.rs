// Copyright 2024 Textile Inc
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::actor_dispatch;
use fil_actors_runtime::actor_error;
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::ActorDowncast;
use fil_actors_runtime::ActorError;
use fvm_ipld_hamt::BytesKey;

use fvm_shared::address::Address;
use fvm_shared::error::ExitCode;

use crate::FingerprintParams;
use crate::{Method, State, FINGERPRINT_ACTOR_NAME};

fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

impl Actor {
    fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let state = State::new(rt.store()).map_err(|e| {
            e.downcast_default(
                ExitCode::USR_ILLEGAL_STATE,
                "failed to construct empty store",
            )
        })?;
        rt.create(&state)
    }

    fn set_pending(
        rt: &impl Runtime,
        fingerprint_params: FingerprintParams,
    ) -> Result<Cid, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let proposer = Address::from_bytes(&fingerprint_params.proposer).unwrap();
        let height = fingerprint_params.height as u64;
        let fingerprint = BytesKey(fingerprint_params.fingerprint);

        rt.transaction(|st: &mut State, rt| {
            st.set_pending::<_, State>(rt.store(), fingerprint, proposer.to_string(), height)
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to push object")
                })
        })
    }

    fn set_verified(
        rt: &impl Runtime,
        fingerprint_params: FingerprintParams,
    ) -> Result<Cid, ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, rt| {
            st.set_verified(rt.store(), BytesKey(fingerprint_params.fingerprint))
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to resolve object")
                })
        })
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        FINGERPRINT_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        SetPending => set_pending,
        SetVerified => set_verified,
    }
}
