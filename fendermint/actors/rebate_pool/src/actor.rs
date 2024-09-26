use fil_actors_runtime::{actor_dispatch, actor_error, FIRST_EXPORTED_METHOD_NUMBER, runtime::{ActorCode, Runtime}, ActorError, SYSTEM_ACTOR_ADDR};
use fvm_shared::MethodNum;
use fvm_ipld_encoding::ipld_block::IpldBlock;

use crate::{ConstructorParams, Method, REBATE_POOL_ACTOR_NAME};
use crate::state::State;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(RebatePoolActor);

pub struct RebatePoolActor;

impl ActorCode for RebatePoolActor {
    type Methods = Method;

    actor_dispatch! {
        Constructor => constructor,
        _ => fallback,
    }

    fn name() -> &'static str {
        REBATE_POOL_ACTOR_NAME
    }
}

impl RebatePoolActor {
    pub fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let state = State::new(params.alpha);
        rt.create(&state)
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
