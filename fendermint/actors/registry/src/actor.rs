// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError,
};
use fvm_shared::clock::ChainEpoch;

use crate::shared::{
    ConstructorParams, SetCustomTTLParams, SetDefaultTTLParams, SetExtendedTTLParams,
    SetNoTTLParams, REGISTRY_ACTOR_NAME,
};
use crate::state::State;
use fendermint_actor_registry_shared::{GetTTLParams, GetTTLReturn, Method};
use fil_actors_runtime::SYSTEM_ACTOR_ADDR;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

const MAX_TTL: ChainEpoch = 1 << 62;
const DEFAULT_TTL: ChainEpoch = 24 * 60 * 60; // 1 day
const NO_TTL: ChainEpoch = 0;

pub struct Actor;

impl Actor {
    pub fn constructor(rt: &impl Runtime, _: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let st = State::new(rt.store());

        rt.create(&st)
    }

    fn set_extended_ttl(rt: &impl Runtime, params: SetExtendedTTLParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, _| {
            st.ttls.insert(params.target, MAX_TTL);
            Ok(())
        })?;
        Ok(())
    }

    fn set_default_ttl(rt: &impl Runtime, params: SetDefaultTTLParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, _| {
            st.ttls.remove(&params.target);
            Ok(())
        })?;
        Ok(())
    }

    fn set_no_ttl(rt: &impl Runtime, params: SetNoTTLParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, _| {
            st.ttls.insert(params.target, NO_TTL);
            Ok(())
        })?;
        Ok(())
    }

    fn set_custom_ttl(rt: &impl Runtime, params: SetCustomTTLParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, _| {
            st.ttls.insert(params.target, params.ttl);
            Ok(())
        })?;
        Ok(())
    }

    fn get_ttl(rt: &impl Runtime, params: GetTTLParams) -> Result<GetTTLReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let st = rt.state::<State>()?;
        let ttl = st
            .ttls
            .get(&params.target)
            .map_or_else(|| DEFAULT_TTL, |ttl| *ttl);

        Ok(GetTTLReturn { ttl })
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        REGISTRY_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        SetExtendedTTL => set_extended_ttl,
        SetDefaultTTL => set_default_ttl,
        SetNoTTL => set_no_ttl,
        SetCustomTTL => set_custom_ttl,
        GetTTL => get_ttl,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fil_actors_runtime::test_utils::{expect_empty, MockRuntime, SYSTEM_ACTOR_CODE_ID};
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;

    fn setup() -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let result = rt
            .call::<Actor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&ConstructorParams {}).unwrap(),
            )
            .unwrap();
        expect_empty(result);
        rt.verify();
        rt.reset();
        rt
    }

    fn get_ttl(rt: &MockRuntime, target: Address) -> ChainEpoch {
        rt.expect_validate_caller_any();
        rt.call::<Actor>(
            Method::GetTTL as u64,
            IpldBlock::serialize_cbor(&GetTTLParams { target }).unwrap(),
        )
        .unwrap()
        .unwrap()
        .deserialize::<GetTTLReturn>()
        .unwrap()
        .ttl
    }

    #[test]
    fn test_constructor() {
        let rt = setup();
        let state = rt.state::<State>().unwrap();
        assert!(state.ttls.is_empty());
    }

    #[test]
    fn test_get_default_ttl() {
        let rt = setup();
        let target = Address::new_id(1000);
        assert_eq!(get_ttl(&rt, target), DEFAULT_TTL);
    }

    #[test]
    fn test_set_extended_ttl() {
        let rt = setup();
        rt.expect_validate_caller_any();

        let target = Address::new_id(1000);
        rt.call::<Actor>(
            Method::SetExtendedTTL as u64,
            IpldBlock::serialize_cbor(&SetExtendedTTLParams { target }).unwrap(),
        )
        .unwrap();

        assert_eq!(get_ttl(&rt, target), MAX_TTL);
    }

    #[test]
    fn test_set_no_ttl() {
        let rt = setup();
        rt.expect_validate_caller_any();

        let target = Address::new_id(1000);
        rt.call::<Actor>(
            Method::SetNoTTL as u64,
            IpldBlock::serialize_cbor(&SetNoTTLParams { target }).unwrap(),
        )
        .unwrap();

        assert_eq!(get_ttl(&rt, target), NO_TTL);
    }

    #[test]
    fn test_set_custom_ttl() {
        let rt = setup();
        rt.expect_validate_caller_any();

        let target = Address::new_id(1000);
        let custom_ttl = 7200; // 2 hours
        rt.call::<Actor>(
            Method::SetCustomTTL as u64,
            IpldBlock::serialize_cbor(&SetCustomTTLParams {
                target,
                ttl: custom_ttl,
            })
            .unwrap(),
        )
        .unwrap();

        assert_eq!(get_ttl(&rt, target), custom_ttl);
    }

    #[test]
    fn test_set_default_ttl_removes_custom() {
        let rt = setup();
        rt.expect_validate_caller_any();

        let target = Address::new_id(1000);
        let custom_ttl = 7200;

        // First set a custom TTL
        rt.call::<Actor>(
            Method::SetCustomTTL as u64,
            IpldBlock::serialize_cbor(&SetCustomTTLParams {
                target,
                ttl: custom_ttl,
            })
            .unwrap(),
        )
        .unwrap();

        assert_eq!(get_ttl(&rt, target), custom_ttl);

        // Then set it back to default
        rt.expect_validate_caller_any();
        rt.call::<Actor>(
            Method::SetDefaultTTL as u64,
            IpldBlock::serialize_cbor(&SetDefaultTTLParams { target }).unwrap(),
        )
        .unwrap();

        assert_eq!(get_ttl(&rt, target), DEFAULT_TTL);
    }
}
