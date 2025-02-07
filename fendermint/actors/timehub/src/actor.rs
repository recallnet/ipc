// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fendermint_actor_blobs_shared::get_credit_approval;
use fendermint_actor_machine::MachineActor;
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError,
};
use tracing::debug;

use crate::{Leaf, Method, PushParams, PushReturn, State, TIMEHUB_ACTOR_NAME};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(TimehubActor);

pub struct TimehubActor;

// Raw type persisted in the store.
// This avoids using CID so that the store does not try to validate or resolve it.
type RawLeaf = (u64, Vec<u8>);

impl TimehubActor {
    fn push(rt: &impl Runtime, params: PushParams) -> Result<PushReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        // Check access control.
        // Either the caller needs to be the Timehub owner, or the owner needs to have given a
        // credit approval to the caller.
        let state = rt.state::<State>()?;
        let owner = state.owner;
        let origin = rt.message().origin();
        let actor_address = state.address.get()?;
        if origin != owner {
            let approval = get_credit_approval(rt, owner, origin)?;
            let cur_epoch = rt.curr_epoch();
            if approval.is_none()
                || approval
                    .unwrap()
                    .expiry
                    .is_some_and(|expiry| expiry < cur_epoch)
            {
                return Err(actor_error!(
                    forbidden;
                    format!("Unauthorized: missing credit approval from Timehub owner {} to origin {} for Timehub {}", owner, origin, actor_address)));
            }
        }

        // Decode the raw bytes as a Cid and report any errors.
        // However, we pass opaque bytes to the store as it tries to validate and resolve any CIDs
        // it stores.
        let _cid = Cid::try_from(params.0.as_slice()).map_err(|_err| {
            actor_error!(illegal_argument;
                    "data must be valid CID bytes")
        })?;
        let timestamp = rt.tipset_timestamp();
        let data: RawLeaf = (timestamp, params.0);

        rt.transaction(|st: &mut State, rt| st.push(rt.store(), data))
    }

    fn get_leaf_at(rt: &impl Runtime, index: u64) -> Result<Option<Leaf>, ActorError> {
        debug!(index, "get_leaf_at");
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        // Decode leaf as timestamp and raw bytes. Then decode as a CID
        let leaf: Option<RawLeaf> = st.get_leaf_at(rt.store(), index)?;
        leaf.map(|(timestamp, bytes)| -> Result<Leaf, ActorError> {
            Ok(Leaf {
                timestamp,
                witnessed: Cid::try_from(bytes).map_err(
                    |_err| actor_error!(illegal_argument; "internal bytes are not a valid CID"),
                )?,
            })
        })
        .transpose()
    }

    fn get_root(rt: &impl Runtime) -> Result<Cid, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        st.get_root(rt.store())
    }

    fn get_peaks(rt: &impl Runtime) -> Result<Vec<Cid>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        st.get_peaks(rt.store())
    }

    fn get_count(rt: &impl Runtime) -> Result<u64, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        Ok(st.leaf_count)
    }
}

impl MachineActor for TimehubActor {
    type State = State;
}

impl ActorCode for TimehubActor {
    type Methods = Method;

    fn name() -> &'static str {
        TIMEHUB_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        Init => init,
        GetAddress => get_address,
        GetMetadata => get_metadata,
        Push => push,
        Get => get_leaf_at,
        Root => get_root,
        Peaks => get_peaks,
        Count => get_count,
        _ => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fendermint_actor_blobs_shared::params::GetCreditApprovalParams;
    use fendermint_actor_blobs_shared::state::CreditApproval;
    use fendermint_actor_blobs_shared::{Method as BlobMethod, BLOBS_ACTOR_ADDR};
    use fendermint_actor_machine::{ConstructorParams, InitParams};
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ADM_ACTOR_CODE_ID, ETHACCOUNT_ACTOR_CODE_ID, INIT_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::{ADM_ACTOR_ADDR, INIT_ACTOR_ADDR};
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::clock::ChainEpoch;
    use fvm_shared::econ::TokenAmount;
    use fvm_shared::error::ExitCode;
    use fvm_shared::sys::SendFlags;
    use fvm_shared::MethodNum;
    use std::collections::HashMap;
    use std::str::FromStr;

    pub fn construct_runtime(actor_address: Address, owner: Address) -> MockRuntime {
        let rt = MockRuntime {
            receiver: actor_address,
            ..Default::default()
        };
        rt.set_caller(*INIT_ACTOR_CODE_ID, INIT_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![INIT_ACTOR_ADDR]);
        let metadata = HashMap::new();

        let result = rt
            .call::<TimehubActor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&ConstructorParams { owner, metadata }).unwrap(),
            )
            .unwrap();
        expect_empty(result);
        rt.verify();

        rt.set_caller(*ADM_ACTOR_CODE_ID, ADM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![ADM_ACTOR_ADDR]);
        let actor_init = rt
            .call::<TimehubActor>(
                Method::Init as u64,
                IpldBlock::serialize_cbor(&InitParams {
                    address: actor_address,
                })
                .unwrap(),
            )
            .unwrap();
        expect_empty(actor_init);
        rt.verify();

        rt.reset();
        rt
    }

    fn get_count(rt: &MockRuntime) -> u64 {
        rt.expect_validate_caller_any();
        rt.call::<TimehubActor>(Method::Count as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<u64>()
            .unwrap()
    }

    fn get_root(rt: &MockRuntime) -> Cid {
        rt.expect_validate_caller_any();
        rt.call::<TimehubActor>(Method::Root as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<Cid>()
            .unwrap()
    }

    fn get_leaf(rt: &MockRuntime, index: u64) -> Leaf {
        rt.expect_validate_caller_any();
        rt.call::<TimehubActor>(
            Method::Get as u64,
            IpldBlock::serialize_cbor(&index).unwrap(),
        )
        .unwrap()
        .unwrap()
        .deserialize::<Option<Leaf>>()
        .unwrap()
        .unwrap()
    }

    fn push_cid(rt: &mut MockRuntime, cid: Cid, timestamp: u64) -> PushReturn {
        rt.expect_validate_caller_any();
        rt.tipset_timestamp = timestamp;
        let push_params = PushParams(cid.to_bytes());
        rt.call::<TimehubActor>(
            Method::Push as u64,
            IpldBlock::serialize_cbor(&push_params).unwrap(),
        )
        .unwrap()
        .unwrap()
        .deserialize::<PushReturn>()
        .unwrap()
    }

    #[test]
    pub fn test_basic_crud() {
        let owner = Address::new_id(110);
        let actor_address = Address::new_id(111);

        let mut rt = construct_runtime(actor_address, owner);

        // Push calls comes from Timehub owner
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, owner);
        rt.set_origin(owner);

        // Check the initial count
        let count = get_count(&rt);
        assert_eq!(count, 0);

        // Check the initial root
        let root = get_root(&rt);
        assert_eq!(root, Cid::from_str("baeaaaaa").unwrap());

        // Push one CID
        let t0 = 1738787063;
        let cid0 = Cid::from_str("bafk2bzacecmnyfiwb52tkbwmm2dsd7ysi3nvuxl3lmspy7pl26wxj4zj7w4wi")
            .unwrap();
        let result0 = push_cid(&mut rt, cid0, t0);

        assert_eq!(0, result0.index);
        let expected_root0 =
            Cid::from_str("bafy2bzacebva5uaq4ayn6ax7zzywcqapf3w4q3oamez6sukidiqiz3m4c6osu")
                .unwrap();
        assert_eq!(result0.root, expected_root0);

        // Read the value pushed
        let leaf = get_leaf(&rt, 0);
        assert_eq!(leaf.witnessed, cid0);
        assert_eq!(leaf.timestamp, t0);

        // Check the root
        let root = get_root(&rt);
        assert_eq!(root, expected_root0);

        // Check the count
        let count = get_count(&rt);
        assert_eq!(count, 1);

        // Push a second CID
        let t1 = t0 + 1;
        let cid1 =
            Cid::from_str("baeabeidtz333ke5c4ultzeg6jkyzgdmvduytt2so3ahozm4zqstiuwq33e").unwrap();
        let result1 = push_cid(&mut rt, cid1, t1);

        assert_eq!(1, result1.index);
        let expected_root1 =
            Cid::from_str("bafy2bzaceb6nrirwdm2ebk5ygl4nhwqjaegpbhavjg2obkshcgoogy4kbovds")
                .unwrap();
        assert_eq!(result1.root, expected_root1);

        // Read the first value pushed
        let leaf0 = get_leaf(&rt, 0);
        assert_eq!(leaf0.witnessed, cid0);
        assert_eq!(leaf0.timestamp, t0);

        // Read the second value pushed
        let leaf1 = get_leaf(&rt, 1);
        assert_eq!(leaf1.witnessed, cid1);
        assert_eq!(leaf1.timestamp, t1);

        // Check the root
        let root = get_root(&rt);
        assert_eq!(root, expected_root1);

        // Check the count
        let count = get_count(&rt);
        assert_eq!(count, 2);

        rt.verify();
    }

    #[test]
    pub fn test_push_access_control_with_no_approval() {
        let owner = Address::new_id(110);
        let actor_address = Address::new_id(111);
        let origin = Address::new_id(112);

        let rt = construct_runtime(actor_address, owner);

        // Push calls comes from the origin Address, which is *not* the Timehub owner.
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, origin);
        rt.set_origin(origin);

        // Set up that the account doing the push does not have a credit approval from the Timehub owner
        let missing_approval: Option<CreditApproval> = None;
        rt.expect_send(
            BLOBS_ACTOR_ADDR,
            BlobMethod::GetCreditApproval as MethodNum,
            IpldBlock::serialize_cbor(&GetCreditApprovalParams {
                from: owner,
                to: origin,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            SendFlags::READ_ONLY,
            IpldBlock::serialize_cbor(&missing_approval).unwrap(),
            ExitCode::OK,
            None,
        );

        // Attempt to push a CID, should fail with access control error.
        let cid = Cid::from_str("bafk2bzacecmnyfiwb52tkbwmm2dsd7ysi3nvuxl3lmspy7pl26wxj4zj7w4wi")
            .unwrap();
        let push_params = PushParams(cid.to_bytes());
        rt.expect_validate_caller_any();

        let err = rt
            .call::<TimehubActor>(
                Method::Push as u64,
                IpldBlock::serialize_cbor(&push_params).unwrap(),
            )
            .expect_err("Push succeeded despite not having a valid credit approval");
        assert_eq!(err.exit_code(), ExitCode::USR_FORBIDDEN);

        rt.verify();
    }

    #[test]
    pub fn test_push_access_control_with_valid_approval_no_expiry() {
        let owner = Address::new_id(110);
        let actor_address = Address::new_id(111);
        let origin = Address::new_id(112);

        let mut rt = construct_runtime(actor_address, owner);

        // Push calls comes from the origin Address, which is *not* the Timehub owner.
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, origin);
        rt.set_origin(origin);

        // Set up valid credit approval from the Timehub owner to the address that will perform the push
        let approval = CreditApproval {
            credit_limit: None,
            gas_fee_limit: None,
            expiry: None,
            credit_used: Default::default(),
            gas_fee_used: Default::default(),
        };
        rt.expect_send(
            BLOBS_ACTOR_ADDR,
            BlobMethod::GetCreditApproval as MethodNum,
            IpldBlock::serialize_cbor(&GetCreditApprovalParams {
                from: owner,
                to: origin,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            SendFlags::READ_ONLY,
            IpldBlock::serialize_cbor(&approval).unwrap(),
            ExitCode::OK,
            None,
        );

        // Push a CID
        let tipset_timestamp = 1738787063;
        let cid = Cid::from_str("bafk2bzacecmnyfiwb52tkbwmm2dsd7ysi3nvuxl3lmspy7pl26wxj4zj7w4wi")
            .unwrap();
        let result = push_cid(&mut rt, cid, tipset_timestamp);

        assert_eq!(0, result.index);
        let expected_root0 =
            Cid::from_str("bafy2bzacebva5uaq4ayn6ax7zzywcqapf3w4q3oamez6sukidiqiz3m4c6osu")
                .unwrap();
        assert_eq!(result.root, expected_root0);

        rt.verify();
    }

    #[test]
    pub fn test_push_access_control_with_valid_approval_future_expiry() {
        let owner = Address::new_id(110);
        let actor_address = Address::new_id(111);
        let origin = Address::new_id(112);

        let mut rt = construct_runtime(actor_address, owner);

        // Push calls comes from the origin Address, which is *not* the Timehub owner.
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, origin);
        rt.set_origin(origin);

        // Set up valid credit approval from the Timehub owner to the address that will perform the push
        let epoch0: ChainEpoch = 100;
        let epoch1 = epoch0 + 1;
        rt.set_epoch(epoch0);

        let approval = CreditApproval {
            credit_limit: None,
            gas_fee_limit: None,
            expiry: Some(epoch1),
            credit_used: Default::default(),
            gas_fee_used: Default::default(),
        };
        rt.expect_send(
            BLOBS_ACTOR_ADDR,
            BlobMethod::GetCreditApproval as MethodNum,
            IpldBlock::serialize_cbor(&GetCreditApprovalParams {
                from: owner,
                to: origin,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            SendFlags::READ_ONLY,
            IpldBlock::serialize_cbor(&approval).unwrap(),
            ExitCode::OK,
            None,
        );

        // Push a CID
        let tipset_timestamp = 1738787063;
        let cid = Cid::from_str("bafk2bzacecmnyfiwb52tkbwmm2dsd7ysi3nvuxl3lmspy7pl26wxj4zj7w4wi")
            .unwrap();

        let result = push_cid(&mut rt, cid, tipset_timestamp);
        assert_eq!(0, result.index);
        let expected_root0 =
            Cid::from_str("bafy2bzacebva5uaq4ayn6ax7zzywcqapf3w4q3oamez6sukidiqiz3m4c6osu")
                .unwrap();
        assert_eq!(result.root, expected_root0);

        rt.verify();
    }

    #[test]
    pub fn test_push_access_control_with_expired_approval() {
        let owner = Address::new_id(110);
        let actor_address = Address::new_id(111);
        let origin = Address::new_id(112);

        let rt = construct_runtime(actor_address, owner);

        // Push calls comes from the origin Address, which is *not* the Timehub owner.
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, origin);
        rt.set_origin(origin);

        // Set up that the account doing the push does has a credit approval from the Timehub owner,
        // but it is expired
        let epoch0: ChainEpoch = 100;
        let epoch1 = epoch0 + 1;
        rt.set_epoch(epoch1);

        let expired_approval = CreditApproval {
            credit_limit: None,
            gas_fee_limit: None,
            expiry: Some(epoch0),
            credit_used: Default::default(),
            gas_fee_used: Default::default(),
        };
        rt.expect_send(
            BLOBS_ACTOR_ADDR,
            BlobMethod::GetCreditApproval as MethodNum,
            IpldBlock::serialize_cbor(&GetCreditApprovalParams {
                from: owner,
                to: origin,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            SendFlags::READ_ONLY,
            IpldBlock::serialize_cbor(&expired_approval).unwrap(),
            ExitCode::OK,
            None,
        );

        // Attempt to push a CID, should fail with access control error.
        let cid = Cid::from_str("bafk2bzacecmnyfiwb52tkbwmm2dsd7ysi3nvuxl3lmspy7pl26wxj4zj7w4wi")
            .unwrap();
        let push_params = PushParams(cid.to_bytes());
        rt.expect_validate_caller_any();

        let err = rt
            .call::<TimehubActor>(
                Method::Push as u64,
                IpldBlock::serialize_cbor(&push_params).unwrap(),
            )
            .expect_err("Push succeeded despite not having a valid credit approval");
        assert_eq!(err.exit_code(), ExitCode::USR_FORBIDDEN);

        rt.verify();
    }
}
