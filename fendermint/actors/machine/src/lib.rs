// Copyright 2024 Hoku Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

pub use fil_actor_adm::Kind;
use fil_actors_runtime::{
    actor_error, deserialize_block, runtime::builtins::Type, runtime::Runtime, ActorError,
    AsActorError, ADM_ACTOR_ADDR, FIRST_EXPORTED_METHOD_NUMBER, INIT_ACTOR_ADDR,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{ipld_block::IpldBlock, tuple::*};
pub use fvm_shared::METHOD_CONSTRUCTOR;
use fvm_shared::{address::Address, error::ExitCode, sys::SendFlags, MethodNum};
use num_traits::Zero;
use serde::{de::DeserializeOwned, Serialize};

/// Params for creating a machine.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    /// The machine owner robust address.
    pub owner: Address,
    /// User-defined metadata.
    pub metadata: HashMap<String, String>,
}

/// Params for initializing a machine.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct InitParams {
    /// The machine reorg safe address.
    pub robust_address: Address,
}

/// Machine initialization method number.
pub const INIT_METHOD: MethodNum = 2;
/// Get machine address method number.
pub const GET_ADDRESS_METHOD: MethodNum = frc42_dispatch::method_hash!("GetAddress");
/// Get machine metadata method number.
pub const GET_METADATA_METHOD: MethodNum = frc42_dispatch::method_hash!("GetMetadata");

/// Returns an error if the address does not match the message origin or caller.
pub fn ensure_addr_is_origin_or_caller(
    rt: &impl Runtime,
    address: Address,
) -> Result<(), ActorError> {
    if rt.message().origin() == rt.message().caller() {
        let (origin, _) = resolve_external(rt, rt.message().origin())?;
        if address == origin {
            return Ok(());
        }
    } else {
        let (origin, _) = resolve_external(rt, rt.message().origin())?;
        if address == origin {
            return Ok(());
        }
        let (caller, _) = resolve_external(rt, rt.message().caller())?;
        if address == caller {
            return Ok(());
        }
    }
    Err(ActorError::illegal_argument(format!(
        "address {} does not match origin or caller",
        address
    )))
}

/// Actor type that includes Hoku Machines.
pub enum ActorType {
    Account,
    Placeholder,
    Evm,
    EthAccount,
    Machine,
}

impl ActorType {
    fn from_builtin_type(t: Type) -> Result<Self, ActorError> {
        match t {
            Type::Account => Ok(ActorType::Account),
            Type::Placeholder => Ok(ActorType::Placeholder),
            Type::EVM => Ok(ActorType::Evm),
            Type::EthAccount => Ok(ActorType::EthAccount),
            _ => Err(ActorError::illegal_argument(format!(
                "builtin actor type {:?} cannot be mapped to actor type",
                t
            ))),
        }
    }
}

/// Resolve robust address and ensure it is not a Machine actor type.
/// See `resolve_external`.
pub fn resolve_external_non_machine(
    rt: &impl Runtime,
    address: Address,
) -> Result<Address, ActorError> {
    let (address, actor_type) = resolve_external(rt, address)?;
    if matches!(actor_type, ActorType::Machine) {
        Err(ActorError::illegal_argument(format!(
            "address {} cannot be a machine",
            address
        )))
    } else {
        Ok(address)
    }
}

/// Resolves robust address of an actor.
pub fn resolve_external(
    rt: &impl Runtime,
    address: Address,
) -> Result<(Address, ActorType), ActorError> {
    let actor_id = rt
        .resolve_address(&address)
        .ok_or(ActorError::not_found(format!(
            "actor {} not found",
            address
        )))?;
    let code_cid = rt
        .get_actor_code_cid(&actor_id)
        .expect("failed to lookup caller code");
    match rt.resolve_builtin_actor_type(&code_cid) {
        Some(t) => match t {
            Type::Placeholder | Type::EVM | Type::EthAccount => {
                let delegated_addr =
                    rt.lookup_delegated_address(actor_id)
                        .ok_or(ActorError::illegal_argument(format!(
                            "actor {} does not have delegated address",
                            actor_id
                        )))?;
                Ok((delegated_addr, ActorType::from_builtin_type(t)?))
            }
            _ => Err(ActorError::forbidden(format!(
                "disallowed caller type {} for address {}",
                t.name(),
                address
            ))),
        },
        None => {
            // The caller might be a machine
            let result = rt
                .send(
                    &address,
                    GET_ADDRESS_METHOD,
                    None,
                    Zero::zero(),
                    None,
                    SendFlags::READ_ONLY,
                )
                .context_code(
                    ExitCode::USR_ASSERTION_FAILED,
                    "machine failed to return its key address",
                )?;
            if !result.exit_code.is_success() {
                return Err(ActorError::forbidden(format!(
                    "disallowed caller code {code_cid}"
                )));
            }
            let robust_addr: Address = deserialize_block(result.return_data)?;
            Ok((robust_addr, ActorType::Machine))
        }
    }
}

// TODO: Add method for changing owner from ADM actor.
pub trait MachineActor {
    type State: MachineState + Serialize + DeserializeOwned;

    /// Machine actor constructor.
    fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&INIT_ACTOR_ADDR))?;
        let state = Self::State::new(rt.store(), params.owner, params.metadata)?;
        rt.create(&state)
    }

    /// Initializes the machine with a robust address.
    fn init(rt: &impl Runtime, params: InitParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&ADM_ACTOR_ADDR))?;
        rt.transaction(|st: &mut Self::State, _| st.init(params.robust_address))
    }

    /// Get machine robust address.
    fn get_address(rt: &impl Runtime) -> Result<Address, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st = rt.state::<Self::State>()?;
        st.address().get()
    }

    /// Get machine metadata.
    fn get_metadata(rt: &impl Runtime) -> Result<Metadata, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st = rt.state::<Self::State>()?;
        Ok(Metadata {
            owner: st.owner(),
            kind: st.kind(),
            metadata: st.metadata(),
        })
    }

    /// Ensures that immediate caller is allowed to write to the machine.
    fn ensure_write_allowed(rt: &impl Runtime) -> Result<(), ActorError> {
        Ok(())
    }

    fn fallback(
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

/// Machine metadata.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Metadata {
    /// Machine kind.
    pub kind: Kind,
    /// Machine owner robust address.
    pub owner: Address,
    /// User-defined data.
    pub metadata: HashMap<String, String>,
}

/// Trait that must be implemented by machine state.
pub trait MachineState {
    fn new<BS: Blockstore>(
        store: &BS,
        owner: Address,
        metadata: HashMap<String, String>,
    ) -> Result<Self, ActorError>
    where
        Self: Sized;
    fn init(&mut self, address: Address) -> Result<(), ActorError>;
    fn address(&self) -> MachineAddress;
    fn kind(&self) -> Kind;
    fn owner(&self) -> Address;
    fn metadata(&self) -> HashMap<String, String>;
}

/// Machine address wrapper.
#[derive(Debug, Clone, Default, Serialize_tuple, Deserialize_tuple)]
pub struct MachineAddress {
    address: Option<Address>,
}

impl MachineAddress {
    /// Get machine address.
    pub fn get(&self) -> Result<Address, ActorError> {
        self.address.ok_or(ActorError::illegal_state(String::from(
            "machine address not set",
        )))
    }

    /// Set machine address. This can only be called once.
    pub fn set(&mut self, address: Address) -> Result<(), ActorError> {
        if self.address.is_some() {
            return Err(ActorError::forbidden(String::from(
                "machine address already set",
            )));
        }
        self.address = Some(address);
        Ok(())
    }
}
