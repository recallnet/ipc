// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::TokenCreditRate;
use fendermint_actor_machine::to_id_address;
use fendermint_actor_recall_config_shared::{
    Method, RecallConfig, SetAdminParams, SetConfigParams,
};
use fil_actors_runtime::actor_error;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::{actor_dispatch, ActorError};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

pub const ACTOR_NAME: &str = "recall_config";

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct State {
    /// The admin address that is allowed to update the config.
    pub admin: Option<Address>,
    /// The Recall network configuration.
    pub config: RecallConfig,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct ConstructorParams {
    initial_blob_capacity: u64,
    initial_token_credit_rate: TokenCreditRate,
    initial_blob_credit_debit_interval: ChainEpoch,
    initial_blob_min_ttl: ChainEpoch,
    initial_blob_default_ttl: ChainEpoch,
}

pub struct Actor {}

impl Actor {
    /// Creates the actor
    pub fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let st = State {
            admin: None,
            config: RecallConfig {
                blob_capacity: params.initial_blob_capacity,
                token_credit_rate: params.initial_token_credit_rate,
                blob_credit_debit_interval: params.initial_blob_credit_debit_interval,
                blob_min_ttl: params.initial_blob_min_ttl,
                blob_default_ttl: params.initial_blob_default_ttl,
            },
        };
        rt.create(&st)
    }

    fn set_admin(rt: &impl Runtime, params: SetAdminParams) -> Result<(), ActorError> {
        Self::ensure_update_allowed(rt)?;
        let new_admin = to_id_address(rt, params.0, false)?;
        rt.transaction(|st: &mut State, _rt| {
            st.admin = Some(new_admin);
            Ok(())
        })
    }

    fn get_admin(rt: &impl Runtime) -> Result<Option<Address>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>().map(|s| s.admin)
    }

    fn set_config(rt: &impl Runtime, params: SetConfigParams) -> Result<(), ActorError> {
        let admin_exists = Self::ensure_update_allowed(rt)?;

        if params.token_credit_rate <= TokenCreditRate::from(BigInt::from(0)) {
            return Err(actor_error!(
                illegal_argument,
                "token credit rate must be positive"
            ));
        }

        if params.blob_capacity == 0 {
            return Err(actor_error!(
                illegal_argument,
                "blob capacity must be positive"
            ));
        }

        if params.blob_credit_debit_interval <= 0 {
            return Err(actor_error!(
                illegal_argument,
                "credit debit interval must be positive"
            ));
        }

        if params.blob_min_ttl <= 0 {
            return Err(actor_error!(
                illegal_argument,
                "minimum TTL must be positive"
            ));
        }

        if params.blob_default_ttl <= 0 {
            return Err(actor_error!(
                illegal_argument,
                "default TTL must be positive"
            ));
        }

        if params.blob_default_ttl < params.blob_min_ttl {
            return Err(actor_error!(
                illegal_argument,
                "default TTL must be greater than or equal to minimum TTL"
            ));
        }

        let new_admin = if !admin_exists {
            // The first caller becomes admin
            let new_admin = to_id_address(rt, rt.message().caller(), false)?;
            Some(new_admin)
        } else {
            None
        };
        rt.transaction(|st: &mut State, _rt| {
            if let Some(new_admin) = new_admin {
                st.admin = Some(new_admin);
            }
            st.config = params;
            Ok(())
        })?;

        Ok(())
    }

    fn get_config(rt: &impl Runtime) -> Result<RecallConfig, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>().map(|s| s.config)
    }

    /// Ensures that immediate caller is allowed to update the config.
    /// Returns whether the admin exists.
    fn ensure_update_allowed(rt: &impl Runtime) -> Result<bool, ActorError> {
        let st = rt.state::<State>()?;
        let admin_exists = if let Some(admin) = st.admin {
            if let Some(admin_id) = rt.resolve_address(&admin) {
                rt.validate_immediate_caller_is(std::iter::once(&Address::new_id(admin_id)))?
            } else {
                // This should not happen.
                return Err(ActorError::forbidden(String::from(
                    "failed to resolve config admin id",
                )));
            }
            true
        } else {
            // The first caller becomes the admin
            rt.validate_immediate_caller_accept_any()?;
            false
        };
        Ok(admin_exists)
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        SetAdmin => set_admin,
        GetAdmin => get_admin,
        SetConfig => set_config,
        GetConfig => get_config,
    }
}

#[cfg(test)]
mod tests {
    use crate::{Actor, ConstructorParams, Method};
    use fendermint_actor_blobs_shared::state::TokenCreditRate;
    use fendermint_actor_recall_config_shared::{RecallConfig, RECALL_CONFIG_ACTOR_ID};
    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ETHACCOUNT_ACTOR_CODE_ID, SYSTEM_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::bigint::BigInt;
    use fvm_shared::clock::ChainEpoch;

    pub fn construct_and_verify(
        token_credit_rate: TokenCreditRate,
        blob_capacity: u64,
        blob_credit_debit_interval: i32,
        initial_blob_min_ttl: ChainEpoch,
        initial_blob_default_ttl: ChainEpoch,
    ) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(RECALL_CONFIG_ACTOR_ID),
            ..Default::default()
        };

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let result = rt
            .call::<Actor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&ConstructorParams {
                    initial_token_credit_rate: token_credit_rate,
                    initial_blob_capacity: blob_capacity,
                    initial_blob_credit_debit_interval: ChainEpoch::from(
                        blob_credit_debit_interval,
                    ),
                    initial_blob_min_ttl,
                    initial_blob_default_ttl,
                })
                .unwrap(),
            )
            .unwrap();
        expect_empty(result);
        rt.verify();
        rt.reset();

        rt
    }

    #[test]
    fn test_get_config() {
        let rt = construct_and_verify(
            TokenCreditRate::from(BigInt::from(5)),
            1024,
            3600,
            3600,
            3600,
        );

        rt.expect_validate_caller_any();
        let recall_config = rt
            .call::<Actor>(Method::GetConfig as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<RecallConfig>()
            .unwrap();

        assert_eq!(
            recall_config.token_credit_rate,
            TokenCreditRate::from(BigInt::from(5))
        );
        assert_eq!(recall_config.blob_capacity, 1024);
        assert_eq!(recall_config.blob_credit_debit_interval, 3600);
    }

    #[test]
    fn test_set_config() {
        let rt = construct_and_verify(
            TokenCreditRate::from(BigInt::from(5)),
            1024,
            3600,
            3600,
            3600,
        );

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);

        rt.expect_validate_caller_any();
        let result = rt.call::<Actor>(
            Method::SetConfig as u64,
            IpldBlock::serialize_cbor(&RecallConfig {
                blob_capacity: 2048,
                token_credit_rate: TokenCreditRate::from(BigInt::from(10)),
                blob_credit_debit_interval: ChainEpoch::from(1800),
                blob_min_ttl: ChainEpoch::from(2 * 60 * 60),
                blob_default_ttl: ChainEpoch::from(24 * 60 * 60),
            })
            .unwrap(),
        );
        assert!(result.is_ok());

        rt.expect_validate_caller_any();
        let recall_config = rt
            .call::<Actor>(Method::GetConfig as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<RecallConfig>()
            .unwrap();

        assert_eq!(
            recall_config.token_credit_rate,
            TokenCreditRate::from(BigInt::from(10))
        );
        assert_eq!(recall_config.blob_capacity, 2048);
        assert_eq!(recall_config.blob_credit_debit_interval, 1800);
        assert_eq!(recall_config.blob_min_ttl, ChainEpoch::from(2 * 60 * 60));
        assert_eq!(
            recall_config.blob_default_ttl,
            ChainEpoch::from(24 * 60 * 60)
        );
    }

    #[test]
    fn test_set_invalid_config() {
        struct TestCase {
            name: &'static str,
            config: RecallConfig,
        }

        let valid_config = RecallConfig {
            blob_capacity: 2048,
            token_credit_rate: TokenCreditRate::from(BigInt::from(10)),
            blob_credit_debit_interval: ChainEpoch::from(1800),
            blob_min_ttl: ChainEpoch::from(2 * 60 * 60),
            blob_default_ttl: ChainEpoch::from(24 * 60 * 60),
        };

        let test_cases = vec![
            // Token credit rate validation
            TestCase {
                name: "token credit rate cannot be zero",
                config: RecallConfig {
                    token_credit_rate: TokenCreditRate::from(BigInt::from(0)),
                    ..valid_config.clone()
                },
            },
            TestCase {
                name: "token credit rate cannot be negative",
                config: RecallConfig {
                    token_credit_rate: TokenCreditRate::from(BigInt::from(-1)),
                    ..valid_config.clone()
                },
            },
            // Blob capacity validation
            TestCase {
                name: "blob capacity cannot be zero",
                config: RecallConfig {
                    blob_capacity: 0,
                    ..valid_config.clone()
                },
            },
            // Credit debit interval validation
            TestCase {
                name: "blob credit debit interval cannot be zero",
                config: RecallConfig {
                    blob_credit_debit_interval: 0,
                    ..valid_config.clone()
                },
            },
            TestCase {
                name: "blob credit debit interval cannot be negative",
                config: RecallConfig {
                    blob_credit_debit_interval: -1,
                    ..valid_config.clone()
                },
            },
            // TTL validations
            TestCase {
                name: "blob min ttl cannot be negative",
                config: RecallConfig {
                    blob_min_ttl: -1,
                    ..valid_config.clone()
                },
            },
            TestCase {
                name: "blob min ttl cannot be zero",
                config: RecallConfig {
                    blob_min_ttl: 0,
                    ..valid_config.clone()
                },
            },
            TestCase {
                name: "blob default ttl must be greater than or equal to min ttl",
                config: RecallConfig {
                    blob_min_ttl: 4 * 60 * 60,
                    blob_default_ttl: 2 * 60 * 60,
                    ..valid_config.clone()
                },
            },
            TestCase {
                name: "blob default ttl cannot be zero",
                config: RecallConfig {
                    blob_default_ttl: 0,
                    ..valid_config.clone()
                },
            },
            TestCase {
                name: "blob default ttl cannot be negative",
                config: RecallConfig {
                    blob_default_ttl: -1,
                    ..valid_config.clone()
                },
            },
        ];

        let rt = construct_and_verify(
            TokenCreditRate::from(BigInt::from(5)),
            1024,
            3600,
            3600,
            3600,
        );

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);

        // Now test all invalid configurations
        for test_case in test_cases {
            rt.expect_validate_caller_any();
            let result = rt.call::<Actor>(
                Method::SetConfig as u64,
                IpldBlock::serialize_cbor(&test_case.config).unwrap(),
            );
            assert!(
                result.is_err(),
                "expected case \"{}\" to fail but it succeeded",
                test_case.name
            );
        }
    }
}
