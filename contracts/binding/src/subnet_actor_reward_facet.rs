pub use subnet_actor_reward_facet::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types
)]
pub mod subnet_actor_reward_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([(
                ::std::borrow::ToOwned::to_owned("claim"),
                ::std::vec![::ethers::core::abi::ethabi::Function {
                    name: ::std::borrow::ToOwned::to_owned("claim"),
                    inputs: ::std::vec![],
                    outputs: ::std::vec![],
                    constant: ::core::option::Option::None,
                    state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                },],
            )]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("CollateralClaimed"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("CollateralClaimed"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("validator"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("amount"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                                indexed: false,
                            },
                        ],
                        anonymous: false,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("Paused"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("Paused"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::EventParam {
                            name: ::std::borrow::ToOwned::to_owned("account"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            indexed: false,
                        },],
                        anonymous: false,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("Unpaused"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("Unpaused"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::EventParam {
                            name: ::std::borrow::ToOwned::to_owned("account"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            indexed: false,
                        },],
                        anonymous: false,
                    },],
                ),
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("EnforcedPause"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("EnforcedPause"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ExpectedPause"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("ExpectedPause"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NoCollateralToWithdraw"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NoCollateralToWithdraw",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotEnoughBalance"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NotEnoughBalance"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ReentrancyError"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("ReentrancyError"),
                        inputs: ::std::vec![],
                    },],
                ),
            ]),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static SUBNETACTORREWARDFACET_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80`@R4\x80\x15`\x0FW`\0\x80\xFD[Pa\x05\x8A\x80a\0\x1F`\09`\0\xF3\xFE`\x80`@R4\x80\x15a\0\x10W`\0\x80\xFD[P`\x046\x10a\0+W`\x005`\xE0\x1C\x80cNq\xD9-\x14a\x000W[`\0\x80\xFD[a\08a\0:V[\0[\x7Fi\x1B\xB0?\xFC\x16\xC5o\xC9k\x82\xFD\x16\xCD\x1B7\x15\xF0\xBC<\xDCd\x07\0_I\xBBb\x05\x86\0\x95\x80T`\0\x19\x01a\0~W`@Qc)\xF7E\xA7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x81Ua\0\x8Aa\x01\x03V[`\0a\0\x953a\x01HV[\x90P\x80\x15a\0\xFCW`@\x80Q\x80\x82\x01\x90\x91R`\x08\x80Ta\0\xF9\x923\x92\x85\x92\x82\x90`\xFF\x16`\x01\x81\x11\x15a\0\xC9Wa\0\xC9a\x04\xB6V[`\x01\x81\x11\x15a\0\xDAWa\0\xDAa\x04\xB6V[\x81R\x90Ta\x01\0\x90\x04`\x01`\x01`\xA0\x1B\x03\x16` \x90\x91\x01R\x91\x90a\x01\xA1V[PP[P`\0\x90UV[\x7F\xC4Q\xC9B\x9C'\xDBh\xF2\x86\xAB\x8Ah\xF3\x11\xF1\xDC\xCA\xB7\x03\xBA\x94#\xAE\xD2\x9C\xD3\x97\xAEc\xF8cT`\xFF\x16\x15a\x01FW`@Qc\xD9<\x06e`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[V[`\0\x80a\x01V`\x17\x84a\x02\x13V[`@\x80Q`\x01`\x01`\xA0\x1B\x03\x86\x16\x81R` \x81\x01\x83\x90R\x91\x93P\x7F\x19|XcS\xEA\xED\n\x1CS\xE6\xE5@D[\x94\xBE\xFA\xB8\xF92\xC8\x11]\x11!\x15\xEC\xBE\xEE\xD5\x14\x91\x01`@Q\x80\x91\x03\x90\xA1P\x91\x90PV[`\0``\x81\x85Q`\x01\x81\x11\x15a\x01\xB9Wa\x01\xB9a\x04\xB6V[\x03a\x01\xE1Wa\x01\xC8\x84\x84a\x02zV[`@\x80Q` \x81\x01\x90\x91R`\0\x81R\x90\x92P\x90Pa\x02\x0BV[`\x01\x85Q`\x01\x81\x11\x15a\x01\xF6Wa\x01\xF6a\x04\xB6V[\x03a\x02\x0BWa\x02\x06\x85\x85\x85a\x02\xFAV[\x91P\x91P[\x93P\x93\x91PPV[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\x01\x83\x01` R`@\x81 \x81\x90\x81\x90a\x02:\x90a\x03\xC2V[\x91P\x91P\x80a\xFF\xFF\x16`\0\x03a\x02pW`\x01`\x01`\xA0\x1B\x03\x84\x16`\0\x90\x81R`\x01\x86\x01` R`@\x90 \x80Tc\xFF\xFF\xFF\xFF\x19\x16\x90U[P\x90P[\x92\x91PPV[`\0\x81G\x10\x15a\x02\x9DW`@QcV\x9DE\xCF`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x83`\x01`\x01`\xA0\x1B\x03\x16\x83`@Q`\0`@Q\x80\x83\x03\x81\x85\x87Z\xF1\x92PPP=\x80`\0\x81\x14a\x02\xEAW`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=`\0` \x84\x01>a\x02\xEFV[``\x91P[P\x90\x95\x94PPPPPV[`\0``\x84` \x01Q`\x01`\x01`\xA0\x1B\x03\x16c\xA9\x05\x9C\xBB`\xE0\x1B\x85\x85`@Q` \x01a\x03;\x92\x91\x90`\x01`\x01`\xA0\x1B\x03\x92\x90\x92\x16\x82R` \x82\x01R`@\x01\x90V[`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x90\x82\x90Ra\x03Y\x92\x91` \x01a\x04\xFCV[`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x90\x82\x90Ra\x03s\x91a\x05 V[`\0`@Q\x80\x83\x03\x81`\0\x86Z\xF1\x91PP=\x80`\0\x81\x14a\x03\xB0W`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=`\0` \x84\x01>a\x03\xB5V[``\x91P[P\x91P\x91P\x93P\x93\x91PPV[\x80T`\0\x90\x81\x90a\xFF\xFF\x16\x80\x82\x03a\x03\xEDW`@Qcd\xB0U\x7F`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x83Tb\x01\0\0\x90\x04a\xFF\xFF\x16\x81`\0[\x83a\xFF\xFF\x16\x83a\xFF\xFF\x16\x10\x15a\x04\x8BWa\xFF\xFF\x83\x16`\0\x90\x81R`\x01\x80\x89\x01` \x90\x81R`@\x92\x83\x90 \x83Q\x80\x85\x01\x90\x94R\x80T\x80\x85R\x92\x01T\x90\x83\x01RC\x10\x15a\x04HWPa\x04\x8BV[` \x81\x01Qa\x04W\x90\x83a\x053V[a\xFF\xFF\x85\x16`\0\x90\x81R`\x01\x8A\x81\x01` R`@\x82 \x82\x81U\x81\x01\x91\x90\x91U\x90\x94\x01\x93`\0\x19\x93\x90\x93\x01\x92\x91Pa\x03\xFD\x90PV[\x86Tc\xFF\xFF\xFF\xFF\x19\x16b\x01\0\0a\xFF\xFF\x94\x85\x16\x02a\xFF\xFF\x19\x16\x17\x92\x82\x16\x92\x90\x92\x17\x90\x95U\x94\x92PPPV[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`\0\x81Q`\0[\x81\x81\x10\x15a\x04\xEDW` \x81\x85\x01\x81\x01Q\x86\x83\x01R\x01a\x04\xD3V[P`\0\x93\x01\x92\x83RP\x90\x91\x90PV[`\x01`\x01`\xE0\x1B\x03\x19\x83\x16\x81R`\0a\x05\x18`\x04\x83\x01\x84a\x04\xCCV[\x94\x93PPPPV[`\0a\x05,\x82\x84a\x04\xCCV[\x93\x92PPPV[\x80\x82\x01\x80\x82\x11\x15a\x02tWcNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD\xFE\xA2dipfsX\"\x12 z\xA3^\x1A\x01\xEC\xDF\xC9\xAB\xAA\x0C\xAE=z\xF0\xFF\xC7v\xE2V-\x9E\xFBAk\xA1\xE1\x83\xD5\x87\xCB~dsolcC\0\x08\x1A\x003";
    /// The bytecode of the contract.
    pub static SUBNETACTORREWARDFACET_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R4\x80\x15a\0\x10W`\0\x80\xFD[P`\x046\x10a\0+W`\x005`\xE0\x1C\x80cNq\xD9-\x14a\x000W[`\0\x80\xFD[a\08a\0:V[\0[\x7Fi\x1B\xB0?\xFC\x16\xC5o\xC9k\x82\xFD\x16\xCD\x1B7\x15\xF0\xBC<\xDCd\x07\0_I\xBBb\x05\x86\0\x95\x80T`\0\x19\x01a\0~W`@Qc)\xF7E\xA7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x81Ua\0\x8Aa\x01\x03V[`\0a\0\x953a\x01HV[\x90P\x80\x15a\0\xFCW`@\x80Q\x80\x82\x01\x90\x91R`\x08\x80Ta\0\xF9\x923\x92\x85\x92\x82\x90`\xFF\x16`\x01\x81\x11\x15a\0\xC9Wa\0\xC9a\x04\xB6V[`\x01\x81\x11\x15a\0\xDAWa\0\xDAa\x04\xB6V[\x81R\x90Ta\x01\0\x90\x04`\x01`\x01`\xA0\x1B\x03\x16` \x90\x91\x01R\x91\x90a\x01\xA1V[PP[P`\0\x90UV[\x7F\xC4Q\xC9B\x9C'\xDBh\xF2\x86\xAB\x8Ah\xF3\x11\xF1\xDC\xCA\xB7\x03\xBA\x94#\xAE\xD2\x9C\xD3\x97\xAEc\xF8cT`\xFF\x16\x15a\x01FW`@Qc\xD9<\x06e`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[V[`\0\x80a\x01V`\x17\x84a\x02\x13V[`@\x80Q`\x01`\x01`\xA0\x1B\x03\x86\x16\x81R` \x81\x01\x83\x90R\x91\x93P\x7F\x19|XcS\xEA\xED\n\x1CS\xE6\xE5@D[\x94\xBE\xFA\xB8\xF92\xC8\x11]\x11!\x15\xEC\xBE\xEE\xD5\x14\x91\x01`@Q\x80\x91\x03\x90\xA1P\x91\x90PV[`\0``\x81\x85Q`\x01\x81\x11\x15a\x01\xB9Wa\x01\xB9a\x04\xB6V[\x03a\x01\xE1Wa\x01\xC8\x84\x84a\x02zV[`@\x80Q` \x81\x01\x90\x91R`\0\x81R\x90\x92P\x90Pa\x02\x0BV[`\x01\x85Q`\x01\x81\x11\x15a\x01\xF6Wa\x01\xF6a\x04\xB6V[\x03a\x02\x0BWa\x02\x06\x85\x85\x85a\x02\xFAV[\x91P\x91P[\x93P\x93\x91PPV[`\x01`\x01`\xA0\x1B\x03\x81\x16`\0\x90\x81R`\x01\x83\x01` R`@\x81 \x81\x90\x81\x90a\x02:\x90a\x03\xC2V[\x91P\x91P\x80a\xFF\xFF\x16`\0\x03a\x02pW`\x01`\x01`\xA0\x1B\x03\x84\x16`\0\x90\x81R`\x01\x86\x01` R`@\x90 \x80Tc\xFF\xFF\xFF\xFF\x19\x16\x90U[P\x90P[\x92\x91PPV[`\0\x81G\x10\x15a\x02\x9DW`@QcV\x9DE\xCF`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\0\x83`\x01`\x01`\xA0\x1B\x03\x16\x83`@Q`\0`@Q\x80\x83\x03\x81\x85\x87Z\xF1\x92PPP=\x80`\0\x81\x14a\x02\xEAW`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=`\0` \x84\x01>a\x02\xEFV[``\x91P[P\x90\x95\x94PPPPPV[`\0``\x84` \x01Q`\x01`\x01`\xA0\x1B\x03\x16c\xA9\x05\x9C\xBB`\xE0\x1B\x85\x85`@Q` \x01a\x03;\x92\x91\x90`\x01`\x01`\xA0\x1B\x03\x92\x90\x92\x16\x82R` \x82\x01R`@\x01\x90V[`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x90\x82\x90Ra\x03Y\x92\x91` \x01a\x04\xFCV[`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x90\x82\x90Ra\x03s\x91a\x05 V[`\0`@Q\x80\x83\x03\x81`\0\x86Z\xF1\x91PP=\x80`\0\x81\x14a\x03\xB0W`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=`\0` \x84\x01>a\x03\xB5V[``\x91P[P\x91P\x91P\x93P\x93\x91PPV[\x80T`\0\x90\x81\x90a\xFF\xFF\x16\x80\x82\x03a\x03\xEDW`@Qcd\xB0U\x7F`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x83Tb\x01\0\0\x90\x04a\xFF\xFF\x16\x81`\0[\x83a\xFF\xFF\x16\x83a\xFF\xFF\x16\x10\x15a\x04\x8BWa\xFF\xFF\x83\x16`\0\x90\x81R`\x01\x80\x89\x01` \x90\x81R`@\x92\x83\x90 \x83Q\x80\x85\x01\x90\x94R\x80T\x80\x85R\x92\x01T\x90\x83\x01RC\x10\x15a\x04HWPa\x04\x8BV[` \x81\x01Qa\x04W\x90\x83a\x053V[a\xFF\xFF\x85\x16`\0\x90\x81R`\x01\x8A\x81\x01` R`@\x82 \x82\x81U\x81\x01\x91\x90\x91U\x90\x94\x01\x93`\0\x19\x93\x90\x93\x01\x92\x91Pa\x03\xFD\x90PV[\x86Tc\xFF\xFF\xFF\xFF\x19\x16b\x01\0\0a\xFF\xFF\x94\x85\x16\x02a\xFF\xFF\x19\x16\x17\x92\x82\x16\x92\x90\x92\x17\x90\x95U\x94\x92PPPV[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`\0\x81Q`\0[\x81\x81\x10\x15a\x04\xEDW` \x81\x85\x01\x81\x01Q\x86\x83\x01R\x01a\x04\xD3V[P`\0\x93\x01\x92\x83RP\x90\x91\x90PV[`\x01`\x01`\xE0\x1B\x03\x19\x83\x16\x81R`\0a\x05\x18`\x04\x83\x01\x84a\x04\xCCV[\x94\x93PPPPV[`\0a\x05,\x82\x84a\x04\xCCV[\x93\x92PPPV[\x80\x82\x01\x80\x82\x11\x15a\x02tWcNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD\xFE\xA2dipfsX\"\x12 z\xA3^\x1A\x01\xEC\xDF\xC9\xAB\xAA\x0C\xAE=z\xF0\xFF\xC7v\xE2V-\x9E\xFBAk\xA1\xE1\x83\xD5\x87\xCB~dsolcC\0\x08\x1A\x003";
    /// The deployed bytecode of the contract.
    pub static SUBNETACTORREWARDFACET_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
    pub struct SubnetActorRewardFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for SubnetActorRewardFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for SubnetActorRewardFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for SubnetActorRewardFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for SubnetActorRewardFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(SubnetActorRewardFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> SubnetActorRewardFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                SUBNETACTORREWARDFACET_ABI.clone(),
                client,
            ))
        }
        /// Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it.
        /// Returns a new instance of a deployer that returns an instance of this contract after sending the transaction
        ///
        /// Notes:
        /// - If there are no constructor arguments, you should pass `()` as the argument.
        /// - The default poll duration is 7 seconds.
        /// - The default number of confirmations is 1 block.
        ///
        ///
        /// # Example
        ///
        /// Generate contract bindings with `abigen!` and deploy a new contract instance.
        ///
        /// *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact.
        ///
        /// ```ignore
        /// # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {
        ///     abigen!(Greeter, "../greeter.json");
        ///
        ///    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();
        ///    let msg = greeter_contract.greet().call().await.unwrap();
        /// # }
        /// ```
        pub fn deploy<T: ::ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::core::result::Result<
            ::ethers::contract::builders::ContractDeployer<M, Self>,
            ::ethers::contract::ContractError<M>,
        > {
            let factory = ::ethers::contract::ContractFactory::new(
                SUBNETACTORREWARDFACET_ABI.clone(),
                SUBNETACTORREWARDFACET_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `claim` (0x4e71d92d) function
        pub fn claim(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([78, 113, 217, 45], ())
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `CollateralClaimed` event
        pub fn collateral_claimed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, CollateralClaimedFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `Paused` event
        pub fn paused_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, PausedFilter> {
            self.0.event()
        }
        ///Gets the contract's `Unpaused` event
        pub fn unpaused_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, UnpausedFilter> {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, SubnetActorRewardFacetEvents>
        {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for SubnetActorRewardFacet<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `EnforcedPause` with signature `EnforcedPause()` and selector `0xd93c0665`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "EnforcedPause", abi = "EnforcedPause()")]
    pub struct EnforcedPause;
    ///Custom Error type `ExpectedPause` with signature `ExpectedPause()` and selector `0x8dfc202b`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "ExpectedPause", abi = "ExpectedPause()")]
    pub struct ExpectedPause;
    ///Custom Error type `NoCollateralToWithdraw` with signature `NoCollateralToWithdraw()` and selector `0x64b0557f`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "NoCollateralToWithdraw", abi = "NoCollateralToWithdraw()")]
    pub struct NoCollateralToWithdraw;
    ///Custom Error type `NotEnoughBalance` with signature `NotEnoughBalance()` and selector `0xad3a8b9e`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "NotEnoughBalance", abi = "NotEnoughBalance()")]
    pub struct NotEnoughBalance;
    ///Custom Error type `ReentrancyError` with signature `ReentrancyError()` and selector `0x29f745a7`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "ReentrancyError", abi = "ReentrancyError()")]
    pub struct ReentrancyError;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetActorRewardFacetErrors {
        EnforcedPause(EnforcedPause),
        ExpectedPause(ExpectedPause),
        NoCollateralToWithdraw(NoCollateralToWithdraw),
        NotEnoughBalance(NotEnoughBalance),
        ReentrancyError(ReentrancyError),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetActorRewardFacetErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <EnforcedPause as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnforcedPause(decoded));
            }
            if let Ok(decoded) = <ExpectedPause as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectedPause(decoded));
            }
            if let Ok(decoded) =
                <NoCollateralToWithdraw as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NoCollateralToWithdraw(decoded));
            }
            if let Ok(decoded) = <NotEnoughBalance as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NotEnoughBalance(decoded));
            }
            if let Ok(decoded) = <ReentrancyError as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReentrancyError(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetActorRewardFacetErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::EnforcedPause(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ExpectedPause(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::NoCollateralToWithdraw(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughBalance(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ReentrancyError(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for SubnetActorRewardFacetErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector == <EnforcedPause as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector == <ExpectedPause as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoCollateralToWithdraw as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector == <NotEnoughBalance as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector == <ReentrancyError as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for SubnetActorRewardFacetErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::EnforcedPause(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectedPause(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoCollateralToWithdraw(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotEnoughBalance(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReentrancyError(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for SubnetActorRewardFacetErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<EnforcedPause> for SubnetActorRewardFacetErrors {
        fn from(value: EnforcedPause) -> Self {
            Self::EnforcedPause(value)
        }
    }
    impl ::core::convert::From<ExpectedPause> for SubnetActorRewardFacetErrors {
        fn from(value: ExpectedPause) -> Self {
            Self::ExpectedPause(value)
        }
    }
    impl ::core::convert::From<NoCollateralToWithdraw> for SubnetActorRewardFacetErrors {
        fn from(value: NoCollateralToWithdraw) -> Self {
            Self::NoCollateralToWithdraw(value)
        }
    }
    impl ::core::convert::From<NotEnoughBalance> for SubnetActorRewardFacetErrors {
        fn from(value: NotEnoughBalance) -> Self {
            Self::NotEnoughBalance(value)
        }
    }
    impl ::core::convert::From<ReentrancyError> for SubnetActorRewardFacetErrors {
        fn from(value: ReentrancyError) -> Self {
            Self::ReentrancyError(value)
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(name = "CollateralClaimed", abi = "CollateralClaimed(address,uint256)")]
    pub struct CollateralClaimedFilter {
        pub validator: ::ethers::core::types::Address,
        pub amount: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(name = "Paused", abi = "Paused(address)")]
    pub struct PausedFilter {
        pub account: ::ethers::core::types::Address,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(name = "Unpaused", abi = "Unpaused(address)")]
    pub struct UnpausedFilter {
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetActorRewardFacetEvents {
        CollateralClaimedFilter(CollateralClaimedFilter),
        PausedFilter(PausedFilter),
        UnpausedFilter(UnpausedFilter),
    }
    impl ::ethers::contract::EthLogDecode for SubnetActorRewardFacetEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = CollateralClaimedFilter::decode_log(log) {
                return Ok(SubnetActorRewardFacetEvents::CollateralClaimedFilter(
                    decoded,
                ));
            }
            if let Ok(decoded) = PausedFilter::decode_log(log) {
                return Ok(SubnetActorRewardFacetEvents::PausedFilter(decoded));
            }
            if let Ok(decoded) = UnpausedFilter::decode_log(log) {
                return Ok(SubnetActorRewardFacetEvents::UnpausedFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for SubnetActorRewardFacetEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CollateralClaimedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::PausedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::UnpausedFilter(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<CollateralClaimedFilter> for SubnetActorRewardFacetEvents {
        fn from(value: CollateralClaimedFilter) -> Self {
            Self::CollateralClaimedFilter(value)
        }
    }
    impl ::core::convert::From<PausedFilter> for SubnetActorRewardFacetEvents {
        fn from(value: PausedFilter) -> Self {
            Self::PausedFilter(value)
        }
    }
    impl ::core::convert::From<UnpausedFilter> for SubnetActorRewardFacetEvents {
        fn from(value: UnpausedFilter) -> Self {
            Self::UnpausedFilter(value)
        }
    }
    ///Container type for all input parameters for the `claim` function with signature `claim()` and selector `0x4e71d92d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "claim", abi = "claim()")]
    pub struct ClaimCall;
}
