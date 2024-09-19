pub use gateway_diamond::*;
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
pub mod gateway_diamond {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::Some(::ethers::core::abi::ethabi::Constructor {
                inputs: ::std::vec![
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("_diamondCut"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Array(
                            ::std::boxed::Box::new(
                                ::ethers::core::abi::ethabi::ParamType::Tuple(
                                    ::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                        ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ::ethers::core::abi::ethabi::ParamType::Array(
                                            ::std::boxed::Box::new(
                                                ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                        ),
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned(
                                "struct IDiamond.FacetCut[]",
                            ),
                        ),
                    },
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("params"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                            ::std::vec![
                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                ::ethers::core::abi::ethabi::ParamType::Uint(16usize),
                                ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                ::ethers::core::abi::ethabi::ParamType::Tuple(
                                    ::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ::ethers::core::abi::ethabi::ParamType::Array(
                                            ::std::boxed::Box::new(
                                                ::ethers::core::abi::ethabi::ParamType::Address,
                                            ),
                                        ),
                                    ],
                                ),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::Tuple(
                                            ::std::vec![
                                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                ::ethers::core::abi::ethabi::ParamType::Address,
                                                ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ],
                                        ),
                                    ),
                                ),
                                ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                            ],
                        ),
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned(
                                "struct GatewayDiamond.ConstructorParams",
                            ),
                        ),
                    },
                ],
            }),
            functions: ::std::collections::BTreeMap::new(),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("DiamondCut"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("DiamondCut"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("_diamondCut"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                                        ),
                                                    ),
                                                ],
                                            ),
                                        ),
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("_init"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("_calldata"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("MembershipUpdated"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("MembershipUpdated"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                        ],
                                                    ),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OwnershipTransferred"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OwnershipTransferred",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("oldOwner"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newOwner"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned(
                        "CannotAddFunctionToDiamondThatAlreadyExists",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotAddFunctionToDiamondThatAlreadyExists",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CannotAddSelectorsToZeroAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotAddSelectorsToZeroAddress",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selectors"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4[]"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "CannotRemoveFunctionThatDoesNotExist",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotRemoveFunctionThatDoesNotExist",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CannotRemoveImmutableFunction"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotRemoveImmutableFunction",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "CannotReplaceFunctionThatDoesNotExists",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotReplaceFunctionThatDoesNotExists",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "CannotReplaceFunctionsFromFacetWithZeroAddress",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotReplaceFunctionsFromFacetWithZeroAddress",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selectors"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4[]"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CannotReplaceImmutableFunction"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotReplaceImmutableFunction",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("FunctionNotFound"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("FunctionNotFound"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_functionSelector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("IncorrectFacetCutAction"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "IncorrectFacetCutAction",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_action"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "enum IDiamond.FacetCutAction",
                                        ),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InitializationFunctionReverted"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InitializationFunctionReverted",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "_initializationContractAddress",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_calldata"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidMajorityPercentage"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidMajorityPercentage",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidSubmissionPeriod"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidSubmissionPeriod",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NoBytecodeAtAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NoBytecodeAtAddress",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_contractAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_message"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "NoSelectorsProvidedForFacetForCut",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NoSelectorsProvidedForFacetForCut",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_facetAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OldConfigurationNumber"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OldConfigurationNumber",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "RemoveFacetAddressMustBeZeroAddress",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "RemoveFacetAddressMustBeZeroAddress",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_facetAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
            ]),
            receive: true,
            fallback: true,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static GATEWAYDIAMOND_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4a\x0C\xA0Wa\x1A=\x808\x03\x80\x91a\0\x1C\x82\x85a\r\xFBV[\x839\x81\x01\x90`@\x81\x83\x03\x12a\x0C\xA0W\x80Q`\x01`\x01`@\x1B\x03\x81\x11a\x0C\xA0W\x81\x01\x82`\x1F\x82\x01\x12\x15a\x0C\xA0W\x80Q\x91a\0T\x83a\x0E\x1EV[\x91a\0b`@Q\x93\x84a\r\xFBV[\x83\x83R` \x83\x01` \x81\x95`\x05\x1B\x83\x01\x01\x91\x86\x83\x11a\x0C\xA0W` \x81\x01\x91[\x83\x83\x10a\x0C\xD1WPPPP` \x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\x0C\xA0W\x01`\xC0\x81\x85\x03\x12a\x0C\xA0W`@Q\x93`\xC0\x85\x01`\x01`\x01`@\x1B\x03\x81\x11\x86\x82\x10\x17a\x05\x16W`@R\x81Q\x85R` \x82\x01Qa\xFF\xFF\x81\x16\x81\x03a\x0C\xA0W` \x86\x01R`@\x82\x01Q`\xFF\x81\x16\x81\x03a\x0C\xA0W`@\x86\x01R``\x82\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x0C\xA0W\x82\x01`@\x81\x83\x03\x12a\x0C\xA0W`@Q\x90a\x01!\x82a\r\xC5V[\x80Q`\x01`\x01`@\x1B\x03\x81\x16\x81\x03a\x0C\xA0W\x82R` \x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\x0C\xA0W\x01\x82`\x1F\x82\x01\x12\x15a\x0C\xA0W\x80Q\x90a\x01a\x82a\x0E\x1EV[\x91a\x01o`@Q\x93\x84a\r\xFBV[\x80\x83R` \x80\x84\x01\x91`\x05\x1B\x83\x01\x01\x91\x85\x83\x11a\x0C\xA0W` \x01\x90[\x82\x82\x10a\x0C\xB9WPPP` \x82\x01R``\x86\x01R`\x80\x82\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x0C\xA0W\x82\x01\x90\x80`\x1F\x83\x01\x12\x15a\x0C\xA0W\x81Q\x91a\x01\xCC\x83a\x0E\x1EV[\x92a\x01\xDA`@Q\x94\x85a\r\xFBV[\x80\x84R` \x80\x85\x01\x91`\x05\x1B\x83\x01\x01\x91\x83\x83\x11a\x0C\xA0W` \x81\x01\x91[\x83\x83\x10a\x0B\xCDW\x89\x89\x89`\xA0\x8A\x8A`\x80\x86\x01R\x01Q`\xA0\x84\x01R\x82Q\x15a\x0B\xBCW`\xFF`@\x84\x01Q\x16`3\x81\x10\x90\x81\x15a\x0B\xB1W[Pa\x0B\xA0W\x7F\x8B\xE0\x07\x9CS\x16Y\x14\x13D\xCD\x1F\xD0\xA4\xF2\x84\x19I\x7F\x97\"\xA3\xDA\xAF\xE3\xB4\x18okdW\xE0`@`\x01\x80`\xA0\x1B\x03`\0\x80Q` a\x19\xBD\x839\x81Q\x91RT\x163`\x01\x80`\xA0\x1B\x03\x19`\0\x80Q` a\x19\xBD\x839\x81Q\x91RT\x16\x17`\0\x80Q` a\x19\xBD\x839\x81Q\x91RU\x81Q\x90\x81R3` \x82\x01R\xA1` \x91`@Qa\x02\xB3\x84\x82a\r\xFBV[`\0\x81R`\x1F\x19\x84\x016\x85\x83\x017\x82Q`\0[\x81\x81\x10a\x05\xA2WPP`@Q\x92``\x84\x01\x90``\x85RQ\x80\x91R`\x80\x84\x01\x90`\x80\x81`\x05\x1B\x86\x01\x01\x93\x91`\0\x90[\x82\x82\x10a\x05,W\x88\x88\x7F\x8F\xAAp\x87\x86q\xCC\xD2\x12\xD2\x07q\xB7\x95\xC5\n\xF8\xFD?\xF6\xCF'\xF4\xBD\xE5~]M\xE0\xAE\xB6s\x89\x80a\x038\x8B\x8B`\0\x87\x85\x01R\x83\x82\x03`@\x85\x01Ra\x0E\xEBV[\x03\x90\xA1c\x01\xFF\xC9\xA7`\xE0\x1B`\0\x90\x81R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4\x80\x83R`@\x80\x83 \x80T`\xFF\x19\x90\x81\x16`\x01\x90\x81\x17\x90\x92Uc\x07\xE4\xC7\x07`\xE2\x1B\x85R\x83\x86R\x82\x85 \x80T\x82\x16\x83\x17\x90UcH\xE2\xB0\x93`\xE0\x1B\x85R\x92\x85R\x92 \x80T\x90\x91\x16\x90\x91\x17\x90U`\x06\x80Tb\xFF\xFF\xFF\x19\x16a\x01\x02\x17\x90U``\x82\x01Q\x80Q`\x12\x80T`\x01`\x01`@\x1B\x03\x19\x16`\x01`\x01`@\x1B\x03\x92\x83\x16\x17\x90U\x90\x82\x01Q\x80Q\x93\x91\x84\x11a\x05\x16Wh\x01\0\0\0\0\0\0\0\0\x84\x11a\x05\x16W\x82\x90`\x13T\x85`\x13U\x80\x86\x10a\x04\xF8W[P\x01\x92`\x13`\0R\x82`\0 `\0[\x82\x81\x10a\x04\xDDW\x83Q`\x01\x90\x81U`@\x80\x86\x01Q`\x04\x80T`\xFF\x19\x16`\xFF\x92\x90\x92\x16\x91\x90\x91\x17\x90U`\x0C\x91\x90\x91U`\xA0\x85\x01Q`\x05U`\x03\x80T`\x01`\x01`\xC0\x1B\x03\x16`\x05`\xC1\x1B\x17\x90U` \x85\x01Q`\x14\x80Tb\xFF\xFF\0\x19\x16`\x08\x92\x90\x92\x1Bb\xFF\xFF\0\x16\x91\x90\x91\x17\x90U`\x1E\x80Th\x01\0\0\0\0\0\0\0\x01`\x01`\x01`\x80\x1B\x03\x19\x90\x91\x16\x17\x90U`\x80\x85\x01Q\x90Qa\x04\xCE\x91`\0\x90\x88\x90a\x04\xC4\x84a\r\xC5V[\x83R\x82\x01Ra\x12GV[`@Qa\x01\"\x90\x81a\x18\x9B\x829\xF3[\x85Q`\x01`\x01`\xA0\x1B\x03\x16\x82\x82\x01U\x94\x84\x01\x94`\x01\x01a\x04#V[a\x05\x10\x90`\x13`\0R\x86\x84`\0 \x91\x82\x01\x91\x01a\x0ElV[\x85a\x04\x14V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x86\x86\x03`\x7F\x19\x01\x81R\x83Q\x80Q`\x01`\x01`\xA0\x1B\x03\x16\x87R\x88\x81\x01Q\x94\x96\x93\x94\x92\x93\x91\x92\x91\x90`\x03\x83\x10\x15a\x05\x8CWa\x05~\x82```@\x8D\x95\x94`\x01\x97\x87\x80\x97\x01R\x01Q\x91\x81`@\x82\x01R\x01\x90a\x0E\xADV[\x97\x01\x92\x01\x92\x01\x90\x92\x91a\x02\xF4V[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`@a\x05\xAE\x82\x87a\x0E\x83V[Q\x01Q`\x01`\x01`\xA0\x1B\x03a\x05\xC3\x83\x88a\x0E\x83V[QQ\x16\x90\x80Q\x15a\x0B\x8BW\x87a\x05\xD9\x84\x89a\x0E\x83V[Q\x01Q\x91`\x03\x83\x10\x92\x83\x15a\x05\x8CW`\0\x81a\x07\xF0WPP\x80\x91\x92P\x15a\x07\xC8Wa\xFF\xFF`\0\x80Q` a\x19\xFD\x839\x81Q\x91RT\x16\x90a\x06Z``a\x06!`@Q\x91\x82a\r\xFBV[`!\x81R\x7FdiamondCut: Add facet has no cod\x8B\x82\x01R`e`\xF8\x1B`@\x82\x01R\x82a\x18\x1FV[\x82Q\x92`\0\x92[\x84\x84\x10a\x06wWPPPPP`\x01\x90[\x01a\x02\xC6V[`\x01`\x01`\xE0\x1B\x03\x19a\x06\x8A\x85\x84a\x0E\x83V[Q\x16`\0\x81\x81R`\0\x80Q` a\x1A\x1D\x839\x81Q\x91R\x8DR`@\x90 T\x90\x91\x90`\x01`\x01`\xA0\x1B\x03\x16a\x07\xB3W\x90a\x07)\x8C\x92a\xFF\xFF`@Q\x91a\x06\xCD\x83a\r\xC5V[\x87\x83R\x81\x16\x85\x83\x01\x81\x81R`\0\x86\x81R`\0\x80Q` a\x1A\x1D\x839\x81Q\x91R\x90\x97R`@\x90\x96 \x92Q\x83T\x96Q`\x01`\x01`\xB0\x1B\x03\x19\x90\x97\x16`\x01`\x01`\xA0\x1B\x03\x91\x90\x91\x16\x17\x95\x90\x91\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x94\x90\x94\x17\x90UV[`\0\x80Q` a\x19\xFD\x839\x81Q\x91RT\x90h\x01\0\0\0\0\0\0\0\0\x82\x10\x15a\x05\x16Wa\x07k\x82`\x01a\x07\x88\x94\x01`\0\x80Q` a\x19\xFD\x839\x81Q\x91RUa\x17+V[\x90\x91\x90c\xFF\xFF\xFF\xFF\x83T\x91`\x03\x1B\x92`\xE0\x1C\x83\x1B\x92\x1B\x19\x16\x17\x90UV[a\xFF\xFF\x81\x14a\x07\x9DW`\x01\x93\x84\x01\x93\x01a\x06aV[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[Pc\xEB\xBF]\x07`\xE0\x1B`\0R`\x04R`$`\0\xFD[P\x86a\x07\xEC`@Q\x92\x83\x92c\x02\xB8\xDA\x07`\xE2\x1B\x84R`\x04\x84\x01R`$\x83\x01\x90a\x0E\xADV[\x03\x90\xFD[P`\x01\x81\x03a\tWWP\x80\x91\x92P\x15a\t3Wa\x08U``a\x08\x15`@Q\x91\x82a\r\xFBV[`(\x81R\x7FLibDiamondCut: Replace facet has\x8A\x82\x01Rg no code`\xC0\x1B`@\x82\x01R\x82a\x18\x1FV[\x81Q\x91`\0[\x83\x81\x10a\x08nWPPPP`\x01\x90a\x06qV[`\x01`\x01`\xE0\x1B\x03\x19a\x08\x81\x82\x84a\x0E\x83V[Q\x16`\0\x81\x81R`\0\x80Q` a\x1A\x1D\x839\x81Q\x91R\x8CR`@\x90 T`\x01`\x01`\xA0\x1B\x03\x160\x81\x14a\t\x1EW\x84\x81\x14a\t\tW\x15a\x08\xF5W`\0\x90\x81R`\0\x80Q` a\x1A\x1D\x839\x81Q\x91R\x8BR`@\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x85\x16\x17\x90U`\x01\x01a\x08[V[cty\xF99`\xE0\x1B`\0R`\x04R`$`\0\xFD[Pc\x1A\xC6\xCE\x8D`\xE1\x1B`\0R`\x04R`$`\0\xFD[Pc)\x01\x80m`\xE1\x1B`\0R`\x04R`$`\0\xFD[P\x86a\x07\xEC`@Q\x92\x83\x92c\xCD\x98\xA9o`\xE0\x1B\x84R`\x04\x84\x01R`$\x83\x01\x90a\x0E\xADV[`\0\x93`\x02\x82\x03a\x0B`WPP`\0\x80Q` a\x19\xFD\x839\x81Q\x91RT\x90\x80a\x0BNWP\x81Q\x91\x83[\x83\x81\x10a\t\x94WPPPPP`\x01\x90a\x06qV[`\x01`\x01`\xE0\x1B\x03\x19a\t\xA7\x82\x84a\x0E\x83V[Q\x16\x80\x86R`\0\x80Q` a\x1A\x1D\x839\x81Q\x91R\x8CR`@\x86 \x93`@Q\x94a\t\xCF\x86a\r\xC5V[T`\x01`\x01`\xA0\x1B\x03\x81\x16\x80\x87R`\xA0\x91\x90\x91\x1Ca\xFF\xFF\x16\x86\x8F\x01R\x15a\x0B:W\x84Q`\x01`\x01`\xA0\x1B\x03\x160\x14a\x0B&W\x80\x15a\x0B\x12W\x8C\x90`\0\x19\x01\x94\x85a\xFF\xFF\x83\x83\x01Q\x16\x03a\n\x9AW[PP`\0\x80Q` a\x19\xFD\x839\x81Q\x91RT\x80\x15a\n\x86W`\x01\x92\x91\x90`\0\x19\x01a\nG\x81a\x17+V[c\xFF\xFF\xFF\xFF\x82T\x91`\x03\x1B\x1B\x19\x16\x90U`\0\x80Q` a\x19\xFD\x839\x81Q\x91RU\x86R`\0\x80Q` a\x1A\x1D\x839\x81Q\x91R\x8CR\x85`@\x81 U\x01a\t\x80V[cNH{q`\xE0\x1B\x87R`1`\x04R`$\x87\xFD[`\0\x80Q` a\x1A\x1D\x839\x81Q\x91R\x82a\xFF\xFFa\x0B\x0B\x94a\n\xBA\x8Aa\x17+V[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x94a\n\xD7\x86a\x07k\x85\x85\x85\x01Q\x16a\x17+V[\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x90\x94\x16\x8BR\x91\x90R`@\x89 \x80Ta\xFF\xFF`\xA0\x1B\x19\x16\x91\x90\x92\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90UV[\x8B\x8Ea\n\x1DV[cNH{q`\xE0\x1B\x87R`\x11`\x04R`$\x87\xFD[c\r\xF5\xFDa`\xE3\x1B\x87R`\x04\x82\x90R`$\x87\xFD[cz\x08\xA2-`\xE0\x1B\x87R`\x04\x82\x90R`$\x87\xFD[c\xD0\x91\xBC\x81`\xE0\x1B\x84R`\x04R`$\x83\xFD[c?\xF4\xD2\x0F`\xE1\x1B\x85R`$\x91\x85\x91\x15a\x0ByW`\x04R\xFD[PcNH{q`\xE0\x1B\x81R`!`\x04R\xFD[Pc\xE7g\xF9\x1F`\xE0\x1B`\0R`\x04R`$`\0\xFD[cu\xC3\xB4'`\xE0\x1B`\0R`\x04`\0\xFD[`d\x91P\x11\x84a\x02,V[c1/\x8E\x05`\xE0\x1B`\0R`\x04`\0\xFD[\x82Q`\x01`\x01`@\x1B\x03\x81\x11a\x0C\xA0W\x82\x01\x90`\x80\x82\x87\x03`\x1F\x19\x01\x12a\x0C\xA0W`@Q\x90a\x0B\xFB\x82a\r\xE0V[` \x83\x01Q\x82Ra\x0C\x0E`@\x84\x01a\x0E5V[` \x83\x01R``\x83\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x0C\xA0W` \x90\x84\x01\x01\x91\x87`\x1F\x84\x01\x12\x15a\x0C\xA0W\x82Q`\0`\x01`\x01`@\x1B\x03\x82\x11a\x0C\xA5WP`@Q\x94a\x0Cc`\x1F\x83\x01`\x1F\x19\x16` \x01\x87a\r\xFBV[\x81\x86R\x89` \x83\x87\x01\x01\x11a\x0C\xA0W\x85a\x0C\x89`\x80\x93` \x98\x97\x89\x80\x80\x9A\x01\x91\x01a\x0EIV[`@\x84\x01R\x01Q``\x82\x01R\x81R\x01\x92\x01\x91a\x01\xF7V[`\0\x80\xFD[cNH{q`\xE0\x1B\x81R`A`\x04R`$\x90\xFD[` \x80\x91a\x0C\xC6\x84a\x0E5V[\x81R\x01\x91\x01\x90a\x01\x8BV[\x82Q`\x01`\x01`@\x1B\x03\x81\x11a\x0C\xA0W\x82\x01``\x81\x8A\x03`\x1F\x19\x01\x12a\x0C\xA0W`@Q\x90``\x82\x01`\x01`\x01`@\x1B\x03\x81\x11\x83\x82\x10\x17a\x05\x16W`@Ra\r\x1A` \x82\x01a\x0E5V[\x82R`@\x81\x01Q`\x03\x81\x10\x15a\x0C\xA0W` \x83\x01R``\x81\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x0C\xA0W` \x91\x01\x01\x89`\x1F\x82\x01\x12\x15a\x0C\xA0W\x80Qa\r]\x81a\x0E\x1EV[\x91a\rk`@Q\x93\x84a\r\xFBV[\x81\x83R` \x80\x84\x01\x92`\x05\x1B\x82\x01\x01\x90\x8C\x82\x11a\x0C\xA0W` \x01\x91[\x81\x83\x10a\r\xA4WPPP`@\x82\x01R\x81R` \x92\x83\x01\x92\x01a\0\x81V[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03a\x0C\xA0W\x81R` \x92\x83\x01\x92\x01a\r\x87V[`@\x81\x01\x90\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17a\x05\x16W`@RV[`\x80\x81\x01\x90\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17a\x05\x16W`@RV[`\x1F\x90\x91\x01`\x1F\x19\x16\x81\x01\x90`\x01`\x01`@\x1B\x03\x82\x11\x90\x82\x10\x17a\x05\x16W`@RV[`\x01`\x01`@\x1B\x03\x81\x11a\x05\x16W`\x05\x1B` \x01\x90V[Q\x90`\x01`\x01`\xA0\x1B\x03\x82\x16\x82\x03a\x0C\xA0WV[`\0[\x83\x81\x10a\x0E\\WPP`\0\x91\x01RV[\x81\x81\x01Q\x83\x82\x01R` \x01a\x0ELV[\x81\x81\x10a\x0EwWPPV[`\0\x81U`\x01\x01a\x0ElV[\x80Q\x82\x10\x15a\x0E\x97W` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x90` \x80\x83Q\x92\x83\x81R\x01\x92\x01\x90`\0[\x81\x81\x10a\x0E\xCBWPPP\x90V[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\x0E\xBEV[\x90` \x91a\x0F\x04\x81Q\x80\x92\x81\x85R\x85\x80\x86\x01\x91\x01a\x0EIV[`\x1F\x01`\x1F\x19\x16\x01\x01\x90V[\x90\x80` \x83Q\x91\x82\x81R\x01\x91` \x80\x83`\x05\x1B\x83\x01\x01\x94\x01\x92`\0\x91[\x83\x83\x10a\x0F<WPPPPP\x90V[\x90\x91\x92\x93\x94` \x80`\x01\x92`\x1F\x19\x85\x82\x03\x01\x86R\x88Q\x90\x81Q\x81R\x84\x80`\xA0\x1B\x03\x83\x83\x01Q\x16\x83\x82\x01R``\x80a\x0F\x82`@\x85\x01Q`\x80`@\x86\x01R`\x80\x85\x01\x90a\x0E\xEBV[\x93\x01Q\x91\x01R\x97\x01\x93\x01\x93\x01\x91\x93\x92\x90a\x0F-V[\x90`\x01\x82\x81\x1C\x92\x16\x80\x15a\x0F\xC7W[` \x83\x10\x14a\x0F\xB1WV[cNH{q`\xE0\x1B`\0R`\"`\x04R`$`\0\xFD[\x91`\x7F\x16\x91a\x0F\xA6V[\x91\x90`\x1F\x81\x11a\x0F\xE0WPPPV[a\x10\x0C\x92`\0R` `\0 \x90` `\x1F\x84\x01`\x05\x1C\x83\x01\x93\x10a\x10\x0EW[`\x1F\x01`\x05\x1C\x01\x90a\x0ElV[V[\x90\x91P\x81\x90a\x0F\xFFV[\x91\x90\x91\x82\x81\x14a\x10\xF6Wa\x10,\x83Ta\x0F\x97V[`\x01`\x01`@\x1B\x03\x81\x11a\x05\x16Wa\x10N\x81a\x10H\x84Ta\x0F\x97V[\x84a\x0F\xD1V[`\0\x93`\x1F\x82\x11`\x01\x14a\x10\x90Wa\x10\x81\x92\x93\x94\x82\x91`\0\x92a\x10\x85W[PP\x81`\x01\x1B\x91`\0\x19\x90`\x03\x1B\x1C\x19\x16\x17\x90V[\x90UV[\x01T\x90P8\x80a\x10lV[\x84R` \x80\x85 \x83\x86R\x90\x85 \x90\x94`\x1F\x19\x83\x16\x81[\x81\x81\x10a\x10\xDEWP\x95\x83`\x01\x95\x96\x97\x10a\x10\xC5W[PPP\x81\x1B\x01\x90UV[\x01T`\0\x19`\xF8\x84`\x03\x1B\x16\x1C\x19\x16\x90U8\x80\x80a\x10\xBBV[\x91\x92`\x01\x80` \x92\x86\x8B\x01T\x81U\x01\x94\x01\x92\x01a\x10\xA6V[P\x90PV[`\x07T\x81\x10\x15a\x0E\x97W`\x07`\0R` `\0 \x90`\x02\x1B\x01\x90`\0\x90V[\x91\x90a\x121W\x80Q\x82U` \x81\x01Q`\x01\x83\x01\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x92\x90\x92\x16\x91\x90\x91\x17\x90U`@\x81\x01Q\x80Q`\x02\x84\x01\x91`\x01`\x01`@\x1B\x03\x82\x11a\x05\x16Wa\x11y\x82a\x11s\x85Ta\x0F\x97V[\x85a\x0F\xD1V[` \x90`\x1F\x83\x11`\x01\x14a\x11\xC5W\x82`\x03\x95\x93``\x95\x93a\x11\xB0\x93`\0\x92a\x11\xBAWPP\x81`\x01\x1B\x91`\0\x19\x90`\x03\x1B\x1C\x19\x16\x17\x90V[\x90U[\x01Q\x91\x01UV[\x01Q\x90P8\x80a\x10lV[\x90`\x1F\x19\x83\x16\x91\x84`\0R\x81`\0 \x92`\0[\x81\x81\x10a\x12\x19WP\x92`\x01\x92\x85\x92`\x03\x98\x96``\x98\x96\x10a\x12\x01W[PPP\x81\x1B\x01\x90Ua\x11\xB3V[\x01Q`\0\x19\x83\x89\x1B`\xF8\x16\x1C\x19\x16\x90U8\x80\x80a\x11\xF4V[\x92\x93` `\x01\x81\x92\x87\x86\x01Q\x81U\x01\x95\x01\x93\x01a\x11\xD8V[cNH{q`\xE0\x1B`\0R`\0`\x04R`$`\0\xFD[`@Q` \x81R\x7F(\xDC\x92\x98-\xDD`~\xD1e\xD6\x8A+3\x12 \xB7%\\\x94{,\x12\xF2\xB59\xA1!w\x08!\xF9a\x12\x85\x83Q`@` \x85\x01R``\x84\x01\x90a\x0F\x10V[` \x84\x01\x80Q`\x01`\x01`@\x1B\x03\x16`@\x85\x01R\x92\x90\x81\x90\x03\x90\xA1`\x08T`\x01`\x01`@\x1B\x03\x16\x80a\x15\xA3W[P`\x07Th\x01\0\0\0\0\0\0\0\0\x81\x11a\x05\x16W`\tT\x81`\tU\x80\x82\x10a\x14\xE9W[P`\t`\0\x90\x81R\x7Fn\x15@\x17\x1Bl\x0C\x96\x0Bq\xA7\x02\r\x9F`\x07\x7Fj\xF91\xA8\xBB\xF5\x90\xDA\x02#\xDA\xCFu\xC7\xAF`\0\x80Q` a\x19\xDD\x839\x81Q\x91R[\x83\x83\x10a\x14\x8FWPPPP`\x01\x80`@\x1B\x03`\x08T\x16`\x01\x80`@\x1B\x03\x19`\nT\x16\x17`\nU\x81QQ\x90`\x07T\x92`\0[\x83\x81\x10a\x14%WPPQ`\x08\x80T`\x01`\x01`@\x1B\x03\x19\x16`\x01`\x01`@\x1B\x03\x92\x90\x92\x16\x91\x90\x91\x17\x90U\x80\x82\x11a\x13tWPPV[\x81\x81\x10a\x13\x7FWPPV[`\x07T\x80\x15a\x14\x0FW`\0\x19\x01\x90a\x13\x96\x82a\x10\xFBV[\x92\x90\x92a\x121W`\0`\x03\x84\x82`\x01\x96U\x82\x86\x82\x01U`\x02\x81\x01a\x13\xBA\x81Ta\x0F\x97V[\x90\x81a\x13\xCEW[PP\x01U`\x07U\x01a\x13tV[\x81`\x1F\x86\x93\x11\x89\x14a\x13\xE4WPU[8\x80a\x13\xC1V[\x81\x83R` \x83 a\x13\xFF\x91`\x1F\x01`\x05\x1C\x81\x01\x90\x8A\x01a\x0ElV[\x80\x82R\x81` \x81 \x91UUa\x13\xDDV[cNH{q`\xE0\x1B`\0R`1`\x04R`$`\0\xFD[\x84\x81\x10\x15a\x14TW\x80a\x14Na\x14>`\x01\x93\x85Qa\x0E\x83V[Qa\x14H\x83a\x10\xFBV[\x90a\x11\x1AV[\x01a\x13?V[a\x14_\x81\x83Qa\x0E\x83V[Q\x90`\x07Th\x01\0\0\0\0\0\0\0\0\x81\x10\x15a\x05\x16W`\x01\x92a\x14H\x82\x85a\x14\x8A\x94\x01`\x07Ua\x10\xFBV[a\x14NV[`\x04\x80\x82`\x01\x93\x85\x03a\x14\xA9W[\x01\x92\x01\x92\x01\x91\x90a\x13\x0EV[\x80T\x85U\x83\x81\x01T\x84\x86\x01\x80T`\xA0\x87\x90\x1B\x87\x90\x03\x92\x83\x16\x92\x19\x16\x91\x90\x91\x17\x90Ua\x14\xDA`\x02\x80\x83\x01\x90\x87\x01a\x10\x18V[`\x03\x81\x01T`\x03\x86\x01Ua\x14\x9DV[`\x01`\x01`\xFE\x1B\x03\x81\x16\x81\x03a\x07\x9DW`\x01`\x01`\xFE\x1B\x03\x82\x16\x82\x03a\x07\x9DW`\t`\0R` `\0 \x90`\x02\x1B\x81\x01\x90\x82`\x02\x1B\x01[\x81\x81\x10a\x15-WPa\x12\xD5V[`\x04\x90`\0\x81U`\0`\x01\x82\x01U`\x02\x81\x01a\x15I\x81Ta\x0F\x97V[\x90\x81a\x15_W[PP`\0`\x03\x82\x01U\x01a\x15 V[\x81`\x1F`\0\x93\x11`\x01\x14a\x15wWPU[8\x80a\x15PV[\x81\x83R` \x83 a\x15\x93\x91`\x1F\x01`\x05\x1C\x81\x01\x90`\x01\x01a\x0ElV[\x80\x82R\x81` \x81 \x91UUa\x15pV[\x81Q`\nT`\x01`\x01`@\x1B\x03\x90\x81\x16\x91\x16\x81\x81\x14a\x17$W\x10a\x17\x13W`@Q\x90a\x15\xCE\x82a\r\xC5V[`\x07Ta\x15\xDA\x81a\x0E\x1EV[\x90a\x15\xE8`@Q\x92\x83a\r\xFBV[\x80\x82R`\x07`\0\x90\x81R\x90`\0\x80Q` a\x19\xDD\x839\x81Q\x91R` \x84\x01[\x82\x84\x10a\x162WPPP\x90\x83RP` \x82\x01Ra\x16$\x90\x83a\x17\x86V[a\x16.W8a\x12\xB2V[PPV[`@Qa\x16>\x81a\r\xE0V[\x82T\x81R`\x01\x83\x01T`\x01`\x01`\xA0\x1B\x03\x16` \x82\x01R`@Q`\x02\x84\x01\x80T`\0\x91a\x16j\x82a\x0F\x97V[\x80\x85R\x91`\x01\x81\x16\x90\x81\x15a\x16\xEEWP`\x01\x14a\x16\xB6W[PP\x92`\x04\x92\x82a\x16\x99` \x94`\x01\x97\x03\x82a\r\xFBV[`@\x82\x01R`\x03\x86\x01T``\x82\x01R\x81R\x01\x92\x01\x93\x01\x92\x90a\x16\x07V[`\0\x90\x81R` \x81 \x90\x92P[\x81\x83\x10a\x16\xD8WPP\x81\x01` \x01\x82\x82a\x16\x82V[`\x01\x81` \x92T\x83\x86\x88\x01\x01R\x01\x92\x01\x91a\x16\xC3V[`\xFF\x19\x16` \x80\x87\x01\x91\x90\x91R\x92\x15\x15`\x05\x1B\x85\x01\x90\x92\x01\x92P\x84\x91P\x83\x90Pa\x16\x82V[c7F\xBE%`\xE1\x1B`\0R`\x04`\0\xFD[PPPPPV[\x90`\0\x80Q` a\x19\xFD\x839\x81Q\x91RT\x82\x10\x15a\x0E\x97W`\0\x80Q` a\x19\xFD\x839\x81Q\x91R`\0R`\x03\x82\x90\x1C\x7F\xB6[\xEC\xA8\xB6\xFAx\x8B\xCB\x15(\xC2\xAB_M\xC6\xBC\x98\xE5\x89eP\xBA\xA0\x13\xD83\x0F\xAB\x0B\x86\xF4\x01\x91`\x02\x1B`\x1C\x16\x90V[` \x81\x81\x01Q\x90\x83\x01Q`\x01`\x01`@\x1B\x03\x91\x82\x16\x91\x16\x03a\x18\x18Wa\x17\xAB\x81a\x18^V[a\x17\xB4\x83a\x18^V[\x03a\x18\x18WQ\x80Q\x82QQ\x03a\x18\x18W`@Qa\x17\xEF\x81a\x17\xE1` \x82\x01\x94` \x86R`@\x83\x01\x90a\x0F\x10V[\x03`\x1F\x19\x81\x01\x83R\x82a\r\xFBV[Q\x90 \x90Q`@Qa\x18\x11\x81a\x17\xE1` \x82\x01\x94` \x86R`@\x83\x01\x90a\x0F\x10V[Q\x90 \x14\x90V[PP`\0\x90V[\x80;\x15a\x18*WPPV[`@\x80Qc\x91\x984\xB9`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x04\x83\x01R`$\x82\x01R\x90\x81\x90a\x07\xEC\x90`D\x83\x01\x90a\x0E\xEBV[\x80QQ\x90`\0\x91`\0\x91[\x81\x83\x10a\x18vWPPP\x90V[\x90\x91\x92a\x18\x84\x84\x83Qa\x0E\x83V[QQ\x81\x01\x80\x91\x11a\x07\x9DW\x92`\x01\x01\x91\x90a\x18iV\xFE`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`sW`\0\x80\x836\x82\x807\x816\x91Z\xF4=`\0\x80>\x15`nW=`\0\xF3[=`\0\xFD[c\n\x82\xDDs`\xE3\x1B`\0R`\x04R`$`\0\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`sW`\0\x80\x836\x82\x807\x816\x91Z\xF4=`\0\x80>\x15`nW=`\0\xF3\xFE\xA2dipfsX\"\x12 \x0C\x0B\xB6B\xE3\xB5\xEE\x10\xA9\xCA\xC5\xEC\\\xBB%\xD4\xD5\x97\xF8(\xDF|Nh\xBEl\x8B\"\x81\xD1\x0EJdsolcC\0\x08\x1A\x003\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5\xA6l\xC9(\xB5\xED\xB8*\xF9\xBDI\x92)T\x15Z\xB7\xB0\x94&\x94\xBE\xA4\xCEDf\x1D\x9A\x876\xC6\x88\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2";
    /// The bytecode of the contract.
    pub static GATEWAYDIAMOND_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`sW`\0\x80\x836\x82\x807\x816\x91Z\xF4=`\0\x80>\x15`nW=`\0\xF3[=`\0\xFD[c\n\x82\xDDs`\xE3\x1B`\0R`\x04R`$`\0\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`sW`\0\x80\x836\x82\x807\x816\x91Z\xF4=`\0\x80>\x15`nW=`\0\xF3\xFE\xA2dipfsX\"\x12 \x0C\x0B\xB6B\xE3\xB5\xEE\x10\xA9\xCA\xC5\xEC\\\xBB%\xD4\xD5\x97\xF8(\xDF|Nh\xBEl\x8B\"\x81\xD1\x0EJdsolcC\0\x08\x1A\x003";
    /// The deployed bytecode of the contract.
    pub static GATEWAYDIAMOND_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
    pub struct GatewayDiamond<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for GatewayDiamond<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for GatewayDiamond<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for GatewayDiamond<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for GatewayDiamond<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(GatewayDiamond))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> GatewayDiamond<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                GATEWAYDIAMOND_ABI.clone(),
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
                GATEWAYDIAMOND_ABI.clone(),
                GATEWAYDIAMOND_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Gets the contract's `DiamondCut` event
        pub fn diamond_cut_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, DiamondCutFilter> {
            self.0.event()
        }
        ///Gets the contract's `MembershipUpdated` event
        pub fn membership_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, MembershipUpdatedFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `OwnershipTransferred` event
        pub fn ownership_transferred_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, OwnershipTransferredFilter>
        {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, GatewayDiamondEvents>
        {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for GatewayDiamond<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `CannotAddFunctionToDiamondThatAlreadyExists` with signature `CannotAddFunctionToDiamondThatAlreadyExists(bytes4)` and selector `0xebbf5d07`
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
    #[etherror(
        name = "CannotAddFunctionToDiamondThatAlreadyExists",
        abi = "CannotAddFunctionToDiamondThatAlreadyExists(bytes4)"
    )]
    pub struct CannotAddFunctionToDiamondThatAlreadyExists {
        pub selector: [u8; 4],
    }
    ///Custom Error type `CannotAddSelectorsToZeroAddress` with signature `CannotAddSelectorsToZeroAddress(bytes4[])` and selector `0x0ae3681c`
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
    #[etherror(
        name = "CannotAddSelectorsToZeroAddress",
        abi = "CannotAddSelectorsToZeroAddress(bytes4[])"
    )]
    pub struct CannotAddSelectorsToZeroAddress {
        pub selectors: ::std::vec::Vec<[u8; 4]>,
    }
    ///Custom Error type `CannotRemoveFunctionThatDoesNotExist` with signature `CannotRemoveFunctionThatDoesNotExist(bytes4)` and selector `0x7a08a22d`
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
    #[etherror(
        name = "CannotRemoveFunctionThatDoesNotExist",
        abi = "CannotRemoveFunctionThatDoesNotExist(bytes4)"
    )]
    pub struct CannotRemoveFunctionThatDoesNotExist {
        pub selector: [u8; 4],
    }
    ///Custom Error type `CannotRemoveImmutableFunction` with signature `CannotRemoveImmutableFunction(bytes4)` and selector `0x6fafeb08`
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
    #[etherror(
        name = "CannotRemoveImmutableFunction",
        abi = "CannotRemoveImmutableFunction(bytes4)"
    )]
    pub struct CannotRemoveImmutableFunction {
        pub selector: [u8; 4],
    }
    ///Custom Error type `CannotReplaceFunctionThatDoesNotExists` with signature `CannotReplaceFunctionThatDoesNotExists(bytes4)` and selector `0x7479f939`
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
    #[etherror(
        name = "CannotReplaceFunctionThatDoesNotExists",
        abi = "CannotReplaceFunctionThatDoesNotExists(bytes4)"
    )]
    pub struct CannotReplaceFunctionThatDoesNotExists {
        pub selector: [u8; 4],
    }
    ///Custom Error type `CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet` with signature `CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(bytes4)` and selector `0x358d9d1a`
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
    #[etherror(
        name = "CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet",
        abi = "CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(bytes4)"
    )]
    pub struct CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet {
        pub selector: [u8; 4],
    }
    ///Custom Error type `CannotReplaceFunctionsFromFacetWithZeroAddress` with signature `CannotReplaceFunctionsFromFacetWithZeroAddress(bytes4[])` and selector `0xcd98a96f`
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
    #[etherror(
        name = "CannotReplaceFunctionsFromFacetWithZeroAddress",
        abi = "CannotReplaceFunctionsFromFacetWithZeroAddress(bytes4[])"
    )]
    pub struct CannotReplaceFunctionsFromFacetWithZeroAddress {
        pub selectors: ::std::vec::Vec<[u8; 4]>,
    }
    ///Custom Error type `CannotReplaceImmutableFunction` with signature `CannotReplaceImmutableFunction(bytes4)` and selector `0x520300da`
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
    #[etherror(
        name = "CannotReplaceImmutableFunction",
        abi = "CannotReplaceImmutableFunction(bytes4)"
    )]
    pub struct CannotReplaceImmutableFunction {
        pub selector: [u8; 4],
    }
    ///Custom Error type `FunctionNotFound` with signature `FunctionNotFound(bytes4)` and selector `0x5416eb98`
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
    #[etherror(name = "FunctionNotFound", abi = "FunctionNotFound(bytes4)")]
    pub struct FunctionNotFound {
        pub function_selector: [u8; 4],
    }
    ///Custom Error type `IncorrectFacetCutAction` with signature `IncorrectFacetCutAction(uint8)` and selector `0x7fe9a41e`
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
    #[etherror(
        name = "IncorrectFacetCutAction",
        abi = "IncorrectFacetCutAction(uint8)"
    )]
    pub struct IncorrectFacetCutAction {
        pub action: u8,
    }
    ///Custom Error type `InitializationFunctionReverted` with signature `InitializationFunctionReverted(address,bytes)` and selector `0x192105d7`
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
    #[etherror(
        name = "InitializationFunctionReverted",
        abi = "InitializationFunctionReverted(address,bytes)"
    )]
    pub struct InitializationFunctionReverted {
        pub initialization_contract_address: ::ethers::core::types::Address,
        pub calldata: ::ethers::core::types::Bytes,
    }
    ///Custom Error type `InvalidMajorityPercentage` with signature `InvalidMajorityPercentage()` and selector `0x75c3b427`
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
    #[etherror(
        name = "InvalidMajorityPercentage",
        abi = "InvalidMajorityPercentage()"
    )]
    pub struct InvalidMajorityPercentage;
    ///Custom Error type `InvalidSubmissionPeriod` with signature `InvalidSubmissionPeriod()` and selector `0x312f8e05`
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
    #[etherror(name = "InvalidSubmissionPeriod", abi = "InvalidSubmissionPeriod()")]
    pub struct InvalidSubmissionPeriod;
    ///Custom Error type `NoBytecodeAtAddress` with signature `NoBytecodeAtAddress(address,string)` and selector `0x919834b9`
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
    #[etherror(
        name = "NoBytecodeAtAddress",
        abi = "NoBytecodeAtAddress(address,string)"
    )]
    pub struct NoBytecodeAtAddress {
        pub contract_address: ::ethers::core::types::Address,
        pub message: ::std::string::String,
    }
    ///Custom Error type `NoSelectorsProvidedForFacetForCut` with signature `NoSelectorsProvidedForFacetForCut(address)` and selector `0xe767f91f`
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
    #[etherror(
        name = "NoSelectorsProvidedForFacetForCut",
        abi = "NoSelectorsProvidedForFacetForCut(address)"
    )]
    pub struct NoSelectorsProvidedForFacetForCut {
        pub facet_address: ::ethers::core::types::Address,
    }
    ///Custom Error type `OldConfigurationNumber` with signature `OldConfigurationNumber()` and selector `0x6e8d7c4a`
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
    #[etherror(name = "OldConfigurationNumber", abi = "OldConfigurationNumber()")]
    pub struct OldConfigurationNumber;
    ///Custom Error type `RemoveFacetAddressMustBeZeroAddress` with signature `RemoveFacetAddressMustBeZeroAddress(address)` and selector `0xd091bc81`
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
    #[etherror(
        name = "RemoveFacetAddressMustBeZeroAddress",
        abi = "RemoveFacetAddressMustBeZeroAddress(address)"
    )]
    pub struct RemoveFacetAddressMustBeZeroAddress {
        pub facet_address: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayDiamondErrors {
        CannotAddFunctionToDiamondThatAlreadyExists(CannotAddFunctionToDiamondThatAlreadyExists),
        CannotAddSelectorsToZeroAddress(CannotAddSelectorsToZeroAddress),
        CannotRemoveFunctionThatDoesNotExist(CannotRemoveFunctionThatDoesNotExist),
        CannotRemoveImmutableFunction(CannotRemoveImmutableFunction),
        CannotReplaceFunctionThatDoesNotExists(CannotReplaceFunctionThatDoesNotExists),
        CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(
            CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet,
        ),
        CannotReplaceFunctionsFromFacetWithZeroAddress(
            CannotReplaceFunctionsFromFacetWithZeroAddress,
        ),
        CannotReplaceImmutableFunction(CannotReplaceImmutableFunction),
        FunctionNotFound(FunctionNotFound),
        IncorrectFacetCutAction(IncorrectFacetCutAction),
        InitializationFunctionReverted(InitializationFunctionReverted),
        InvalidMajorityPercentage(InvalidMajorityPercentage),
        InvalidSubmissionPeriod(InvalidSubmissionPeriod),
        NoBytecodeAtAddress(NoBytecodeAtAddress),
        NoSelectorsProvidedForFacetForCut(NoSelectorsProvidedForFacetForCut),
        OldConfigurationNumber(OldConfigurationNumber),
        RemoveFacetAddressMustBeZeroAddress(RemoveFacetAddressMustBeZeroAddress),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for GatewayDiamondErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <CannotAddFunctionToDiamondThatAlreadyExists as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotAddFunctionToDiamondThatAlreadyExists(decoded));
            }
            if let Ok(decoded) =
                <CannotAddSelectorsToZeroAddress as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CannotAddSelectorsToZeroAddress(decoded));
            }
            if let Ok(decoded) =
                <CannotRemoveFunctionThatDoesNotExist as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                )
            {
                return Ok(Self::CannotRemoveFunctionThatDoesNotExist(decoded));
            }
            if let Ok(decoded) =
                <CannotRemoveImmutableFunction as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CannotRemoveImmutableFunction(decoded));
            }
            if let Ok(decoded) =
                <CannotReplaceFunctionThatDoesNotExists as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                )
            {
                return Ok(Self::CannotReplaceFunctionThatDoesNotExists(decoded));
            }
            if let Ok(decoded) = <CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(
                    Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = <CannotReplaceFunctionsFromFacetWithZeroAddress as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotReplaceFunctionsFromFacetWithZeroAddress(decoded));
            }
            if let Ok(decoded) =
                <CannotReplaceImmutableFunction as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CannotReplaceImmutableFunction(decoded));
            }
            if let Ok(decoded) = <FunctionNotFound as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FunctionNotFound(decoded));
            }
            if let Ok(decoded) =
                <IncorrectFacetCutAction as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::IncorrectFacetCutAction(decoded));
            }
            if let Ok(decoded) =
                <InitializationFunctionReverted as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InitializationFunctionReverted(decoded));
            }
            if let Ok(decoded) =
                <InvalidMajorityPercentage as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidMajorityPercentage(decoded));
            }
            if let Ok(decoded) =
                <InvalidSubmissionPeriod as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidSubmissionPeriod(decoded));
            }
            if let Ok(decoded) =
                <NoBytecodeAtAddress as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NoBytecodeAtAddress(decoded));
            }
            if let Ok(decoded) =
                <NoSelectorsProvidedForFacetForCut as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NoSelectorsProvidedForFacetForCut(decoded));
            }
            if let Ok(decoded) =
                <OldConfigurationNumber as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::OldConfigurationNumber(decoded));
            }
            if let Ok(decoded) =
                <RemoveFacetAddressMustBeZeroAddress as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                )
            {
                return Ok(Self::RemoveFacetAddressMustBeZeroAddress(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for GatewayDiamondErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::CannotAddFunctionToDiamondThatAlreadyExists(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotAddSelectorsToZeroAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotRemoveFunctionThatDoesNotExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotRemoveImmutableFunction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotReplaceFunctionThatDoesNotExists(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotReplaceFunctionsFromFacetWithZeroAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotReplaceImmutableFunction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FunctionNotFound(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::IncorrectFacetCutAction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializationFunctionReverted(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidMajorityPercentage(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidSubmissionPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoBytecodeAtAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoSelectorsProvidedForFacetForCut(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OldConfigurationNumber(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RemoveFacetAddressMustBeZeroAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for GatewayDiamondErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <CannotAddFunctionToDiamondThatAlreadyExists as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotAddSelectorsToZeroAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotRemoveFunctionThatDoesNotExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotRemoveImmutableFunction as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotReplaceFunctionThatDoesNotExists as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotReplaceFunctionsFromFacetWithZeroAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotReplaceImmutableFunction as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FunctionNotFound as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <IncorrectFacetCutAction as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InitializationFunctionReverted as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidMajorityPercentage as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidSubmissionPeriod as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoBytecodeAtAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoSelectorsProvidedForFacetForCut as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OldConfigurationNumber as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RemoveFacetAddressMustBeZeroAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for GatewayDiamondErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CannotAddFunctionToDiamondThatAlreadyExists(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotAddSelectorsToZeroAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotRemoveFunctionThatDoesNotExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotRemoveImmutableFunction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotReplaceFunctionThatDoesNotExists(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotReplaceFunctionsFromFacetWithZeroAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotReplaceImmutableFunction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FunctionNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::IncorrectFacetCutAction(element) => ::core::fmt::Display::fmt(element, f),
                Self::InitializationFunctionReverted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidMajorityPercentage(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidSubmissionPeriod(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoBytecodeAtAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoSelectorsProvidedForFacetForCut(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OldConfigurationNumber(element) => ::core::fmt::Display::fmt(element, f),
                Self::RemoveFacetAddressMustBeZeroAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for GatewayDiamondErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<CannotAddFunctionToDiamondThatAlreadyExists> for GatewayDiamondErrors {
        fn from(value: CannotAddFunctionToDiamondThatAlreadyExists) -> Self {
            Self::CannotAddFunctionToDiamondThatAlreadyExists(value)
        }
    }
    impl ::core::convert::From<CannotAddSelectorsToZeroAddress> for GatewayDiamondErrors {
        fn from(value: CannotAddSelectorsToZeroAddress) -> Self {
            Self::CannotAddSelectorsToZeroAddress(value)
        }
    }
    impl ::core::convert::From<CannotRemoveFunctionThatDoesNotExist> for GatewayDiamondErrors {
        fn from(value: CannotRemoveFunctionThatDoesNotExist) -> Self {
            Self::CannotRemoveFunctionThatDoesNotExist(value)
        }
    }
    impl ::core::convert::From<CannotRemoveImmutableFunction> for GatewayDiamondErrors {
        fn from(value: CannotRemoveImmutableFunction) -> Self {
            Self::CannotRemoveImmutableFunction(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionThatDoesNotExists> for GatewayDiamondErrors {
        fn from(value: CannotReplaceFunctionThatDoesNotExists) -> Self {
            Self::CannotReplaceFunctionThatDoesNotExists(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet>
        for GatewayDiamondErrors
    {
        fn from(value: CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet) -> Self {
            Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionsFromFacetWithZeroAddress>
        for GatewayDiamondErrors
    {
        fn from(value: CannotReplaceFunctionsFromFacetWithZeroAddress) -> Self {
            Self::CannotReplaceFunctionsFromFacetWithZeroAddress(value)
        }
    }
    impl ::core::convert::From<CannotReplaceImmutableFunction> for GatewayDiamondErrors {
        fn from(value: CannotReplaceImmutableFunction) -> Self {
            Self::CannotReplaceImmutableFunction(value)
        }
    }
    impl ::core::convert::From<FunctionNotFound> for GatewayDiamondErrors {
        fn from(value: FunctionNotFound) -> Self {
            Self::FunctionNotFound(value)
        }
    }
    impl ::core::convert::From<IncorrectFacetCutAction> for GatewayDiamondErrors {
        fn from(value: IncorrectFacetCutAction) -> Self {
            Self::IncorrectFacetCutAction(value)
        }
    }
    impl ::core::convert::From<InitializationFunctionReverted> for GatewayDiamondErrors {
        fn from(value: InitializationFunctionReverted) -> Self {
            Self::InitializationFunctionReverted(value)
        }
    }
    impl ::core::convert::From<InvalidMajorityPercentage> for GatewayDiamondErrors {
        fn from(value: InvalidMajorityPercentage) -> Self {
            Self::InvalidMajorityPercentage(value)
        }
    }
    impl ::core::convert::From<InvalidSubmissionPeriod> for GatewayDiamondErrors {
        fn from(value: InvalidSubmissionPeriod) -> Self {
            Self::InvalidSubmissionPeriod(value)
        }
    }
    impl ::core::convert::From<NoBytecodeAtAddress> for GatewayDiamondErrors {
        fn from(value: NoBytecodeAtAddress) -> Self {
            Self::NoBytecodeAtAddress(value)
        }
    }
    impl ::core::convert::From<NoSelectorsProvidedForFacetForCut> for GatewayDiamondErrors {
        fn from(value: NoSelectorsProvidedForFacetForCut) -> Self {
            Self::NoSelectorsProvidedForFacetForCut(value)
        }
    }
    impl ::core::convert::From<OldConfigurationNumber> for GatewayDiamondErrors {
        fn from(value: OldConfigurationNumber) -> Self {
            Self::OldConfigurationNumber(value)
        }
    }
    impl ::core::convert::From<RemoveFacetAddressMustBeZeroAddress> for GatewayDiamondErrors {
        fn from(value: RemoveFacetAddressMustBeZeroAddress) -> Self {
            Self::RemoveFacetAddressMustBeZeroAddress(value)
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
    #[ethevent(
        name = "DiamondCut",
        abi = "DiamondCut((address,uint8,bytes4[])[],address,bytes)"
    )]
    pub struct DiamondCutFilter {
        pub diamond_cut: ::std::vec::Vec<FacetCut>,
        pub init: ::ethers::core::types::Address,
        pub calldata: ::ethers::core::types::Bytes,
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
    #[ethevent(
        name = "MembershipUpdated",
        abi = "MembershipUpdated(((uint256,address,bytes,uint256)[],uint64))"
    )]
    pub struct MembershipUpdatedFilter(pub Membership);
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
    #[ethevent(
        name = "OwnershipTransferred",
        abi = "OwnershipTransferred(address,address)"
    )]
    pub struct OwnershipTransferredFilter {
        pub old_owner: ::ethers::core::types::Address,
        pub new_owner: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayDiamondEvents {
        DiamondCutFilter(DiamondCutFilter),
        MembershipUpdatedFilter(MembershipUpdatedFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
    }
    impl ::ethers::contract::EthLogDecode for GatewayDiamondEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = DiamondCutFilter::decode_log(log) {
                return Ok(GatewayDiamondEvents::DiamondCutFilter(decoded));
            }
            if let Ok(decoded) = MembershipUpdatedFilter::decode_log(log) {
                return Ok(GatewayDiamondEvents::MembershipUpdatedFilter(decoded));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(GatewayDiamondEvents::OwnershipTransferredFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for GatewayDiamondEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::DiamondCutFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::MembershipUpdatedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::OwnershipTransferredFilter(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<DiamondCutFilter> for GatewayDiamondEvents {
        fn from(value: DiamondCutFilter) -> Self {
            Self::DiamondCutFilter(value)
        }
    }
    impl ::core::convert::From<MembershipUpdatedFilter> for GatewayDiamondEvents {
        fn from(value: MembershipUpdatedFilter) -> Self {
            Self::MembershipUpdatedFilter(value)
        }
    }
    impl ::core::convert::From<OwnershipTransferredFilter> for GatewayDiamondEvents {
        fn from(value: OwnershipTransferredFilter) -> Self {
            Self::OwnershipTransferredFilter(value)
        }
    }
    ///`ConstructorParams(uint256,uint16,uint8,(uint64,address[]),(uint256,address,bytes,uint256)[],bytes32)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct ConstructorParams {
        pub bottom_up_check_period: ::ethers::core::types::U256,
        pub active_validators_limit: u16,
        pub majority_percentage: u8,
        pub network_name: SubnetID,
        pub genesis_validators: ::std::vec::Vec<Validator>,
        pub commit_sha: [u8; 32],
    }
    ///`FacetCut(address,uint8,bytes4[])`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct FacetCut {
        pub facet_address: ::ethers::core::types::Address,
        pub action: u8,
        pub function_selectors: ::std::vec::Vec<[u8; 4]>,
    }
    ///`Membership((uint256,address,bytes,uint256)[],uint64)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct Membership {
        pub validators: ::std::vec::Vec<Validator>,
        pub configuration_number: u64,
    }
    ///`SubnetID(uint64,address[])`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct SubnetID {
        pub root: u64,
        pub route: ::std::vec::Vec<::ethers::core::types::Address>,
    }
    ///`Validator(uint256,address,bytes,uint256)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct Validator {
        pub weight: ::ethers::core::types::U256,
        pub addr: ::ethers::core::types::Address,
        pub metadata: ::ethers::core::types::Bytes,
        pub storage_amount: ::ethers::core::types::U256,
    }
}
