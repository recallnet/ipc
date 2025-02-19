pub use diamond_loupe_facet::*;
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
pub mod diamond_loupe_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("facetAddress"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("facetAddress"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_functionSelector"),
                            kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes4"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("facetAddress_"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("facetAddresses"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("facetAddresses"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("facetAddresses_"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address[]"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("facetFunctionSelectors"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("facetFunctionSelectors",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_facet"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_facetFunctionSelectors",),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes4[]"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("facets"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("facets"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("facets_"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                        ::ethers::core::abi::ethabi::ParamType::Array(
                                            ::std::boxed::Box::new(
                                                ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                                    4usize
                                                ),
                                            ),
                                        ),
                                    ],),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct IDiamondLoupe.Facet[]",),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("supportsInterface"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("supportsInterface"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_interfaceId"),
                            kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes4"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bool"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
            ]),
            events: ::std::collections::BTreeMap::new(),
            errors: ::std::collections::BTreeMap::new(),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static DIAMONDLOUPEFACET_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4a\0\x16Wa\x08O\x90\x81a\0\x1C\x829\xF3[`\0\x80\xFD\xFE`\x80`@R`\x046\x10\x15a\0\x12W`\0\x80\xFD[`\x005`\xE0\x1C\x80c\x01\xFF\xC9\xA7\x14a\0gW\x80cR\xEFk,\x14a\0bW\x80cz\x0E\xD6'\x14a\0]W\x80c\xAD\xFC\xA1^\x14a\0XWc\xCD\xFF\xAC\xC6\x14a\0SW`\0\x80\xFD[a\x05\xEDV[a\x052V[a\x02\xFCV[a\x01'V[4a\0\xBAW`\x01`\x01`\xE0\x1B\x03\x19a\0~6a\0\xBFV[\x16`\0R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4` R`\xFF`@`\0 T\x16\x15\x15`\x80R` `\x80\xF3[`\0\x80\xFD[` \x90`\x03\x19\x01\x12a\0\xBAW`\x045`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03a\0\xBAW\x90V[` \x90` `@\x81\x83\x01\x92\x82\x81R\x85Q\x80\x94R\x01\x93\x01\x91`\0[\x82\x81\x10a\x01\nWPPPP\x90V[\x83Q`\x01`\x01`\xA0\x1B\x03\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01a\0\xFCV[4a\0\xBAW`\0\x80`\x03\x196\x01\x12a\x02?W`\0\x80Q` a\x07\xFA\x839\x81Q\x91RT\x90a\x01S\x82a\x06\xA3V[\x90\x80\x80[\x84\x82\x10a\x01sW\x82\x84R`@Q\x80a\x01o\x86\x82a\0\xE2V[\x03\x90\xF3[\x90\x91a\x01\xA6a\x01\x99a\x01\x94a\x01\x87\x86a\x06\xE6V[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x90V[a\x06\x12V[T`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x82`\x01`\x01`\xA0\x1B\x03\x82\x16\x81[\x84\x81\x10a\x01\xFCW[PPa\x01\xF2W\x81a\x01\xE4a\x01\xE9\x92a\x01\xD5`\x01\x95\x89a\x07DV[`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90RV[a\x07nV[\x92[\x01\x90a\x01WV[P\x91`\x01\x90a\x01\xEBV[a\x02%a\x02\x19a\x02\x0C\x83\x8Ba\x07DV[Q`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x82\x14a\x023W`\x01\x01a\x01\xB3V[PPP`\x018\x80a\x01\xBBV[\x80\xFD[` \x80\x82\x01\x90\x80\x83R\x83Q\x80\x92R`@\x92`@\x81\x01\x82`@\x85`\x05\x1B\x84\x01\x01\x96\x01\x94`\0\x80\x93[\x86\x85\x10a\x02{WPPPPPPPP\x90V[\x90\x91\x92\x93\x94\x80\x96\x97\x98`?\x19\x83\x82\x03\x01\x86R\x89Q\x82``\x81\x88\x85\x01\x93`\x01\x80`\xA0\x1B\x03\x81Q\x16\x86R\x01Q\x93\x88\x83\x82\x01R\x84Q\x80\x94R\x01\x92\x01\x90\x85\x90[\x80\x82\x10a\x02\xD8WPPP\x90\x80`\x01\x92\x9A\x01\x95\x01\x95\x01\x93\x96\x95\x94\x92\x91\x90a\x02iV[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R\x8A\x94\x93\x84\x01\x93\x90\x92\x01\x91`\x01\x91\x90\x91\x01\x90a\x02\xB7V[4a\0\xBAW`\0\x80`\x03\x196\x01\x12a\x02?W`\0\x80Q` a\x07\xFA\x839\x81Q\x91RTa\x03'\x81a\x07\x82V[\x90a\x031\x81a\x06\xA3V[\x92\x80\x91\x81\x90[\x80\x82\x10a\x03\x91WPP[\x81\x81\x10a\x03YW\x81\x83R`@Q\x80a\x01o\x85\x82a\x02BV[\x80a\x03{a\x03ta\x03l`\x01\x94\x88a\x07DV[Qa\xFF\xFF\x16\x90V[a\xFF\xFF\x16\x90V[` a\x03\x87\x83\x87a\x07DV[Q\x01QR\x01a\x03AV[\x90\x92a\x03\x9Fa\x01\x87\x85a\x06\xE6V[a\x03\xABa\x01\x99\x82a\x06\x12V[\x84`\x01`\x01`\xA0\x1B\x03\x82\x16\x81[\x85\x81\x10a\x04UW[PPa\x04JW\x91a\x04.a\x04A\x92a\x03\xEE`\x01\x95a\x03\xDE\x85\x8Ca\x07DV[Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90RV[a\x04\x1E\x83a\x03\xFB\x88a\x06\xA3V[\x8Ba\x04\t` \x93\x84\x92a\x07DV[Q\x01Ra\x04\x16\x85\x8Ca\x07DV[Q\x01Qa\x077V[`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16\x90RV[a\x01\xE4a\x04;\x82\x8Aa\x07DV[`\x01\x90RV[\x93[\x01\x90a\x037V[PP\x92`\x01\x90a\x04CV[\x8A\x85\x8B\x84a\x04wa\x02\x19a\x04i\x87\x85a\x07DV[QQ`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x14a\x04\x87WPPP`\x01\x01a\x03\xB8V[a\x04\xE3\x95P\x83\x80\x95Pa\x04\xD3\x93a\x04\xC9a\x04\xCE\x94a\x04\x1E` a\x04\xB0a\x04\xDA\x9Aa\x03l\x98a\x07DV[Q\x01Qa\x04\xC3a\x03ta\x03l\x88\x88a\x07DV[\x90a\x07DV[a\x07DV[a\x07\xE6V[\x91\x8Ba\x07DV[\x90a\xFF\xFF\x16\x90RV[`\x018\x80a\x03\xC0V[` \x90` `@\x81\x83\x01\x92\x82\x81R\x85Q\x80\x94R\x01\x93\x01\x91`\0[\x82\x81\x10a\x05\x14WPPPP\x90V[\x83Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01a\x05\x06V[4a\0\xBAW` 6`\x03\x19\x01\x12a\0\xBAW`\x01`\x01`\xA0\x1B\x03`\x045\x81\x81\x16\x90\x81\x90\x03a\0\xBAW`\0\x80Q` a\x07\xFA\x839\x81Q\x91RT\x91`\0\x90a\x05v\x84a\x06\xA3V[\x92`\0[\x85\x81\x10a\x05\x92W\x83\x85R`@Q\x80a\x01o\x87\x82a\x04\xECV[a\x05\x9B\x81a\x06\xE6V[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x83a\x05\xAF\x82a\x06\x12V[T\x16\x83\x14a\x05\xC1W[P`\x01\x01a\x05zV[\x84a\x05\xE6\x91a\x05\xD3`\x01\x94\x97\x89a\x07DV[`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16\x90Ra\x07nV[\x93\x90a\x05\xB8V[4a\0\xBAW` `\x01`\x01`\xA0\x1B\x03a\x06\x08a\x01\x946a\0\xBFV[T\x16`@Q\x90\x81R\xF3[c\xFF\xFF\xFF\xFF`\xE0\x1B\x16`\0R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@`\0 \x90V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[`@Q\x91\x90`\x1F\x01`\x1F\x19\x16\x82\x01g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x83\x82\x10\x17a\x06\x86W`@RV[a\x06JV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x06\x86W`\x05\x1B` \x01\x90V[\x90a\x06\xB5a\x06\xB0\x83a\x06\x8BV[a\x06`V[\x82\x81R\x80\x92a\x06\xC6`\x1F\x19\x91a\x06\x8BV[\x01\x90` 6\x91\x017V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x90`\0\x80Q` a\x07\xFA\x839\x81Q\x91R\x80T\x83\x10\x15a\x072W`\0R`\x1C\x82`\x03\x1C\x7F\xB6[\xEC\xA8\xB6\xFAx\x8B\xCB\x15(\xC2\xAB_M\xC6\xBC\x98\xE5\x89eP\xBA\xA0\x13\xD83\x0F\xAB\x0B\x86\xF4\x01\x92`\x02\x1B\x16\x90V[a\x06\xD0V[\x80Q\x15a\x072W` \x01\x90V[\x80Q\x82\x10\x15a\x072W` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`\0\x19\x81\x14a\x07}W`\x01\x01\x90V[a\x07XV[\x90a\x07\x8Fa\x06\xB0\x83a\x06\x8BV[\x82\x81R\x80\x92a\x07\xA0`\x1F\x19\x91a\x06\x8BV[\x01`\0\x80[\x82\x81\x10a\x07\xB2WPPPPV[`@\x90\x81Q\x82\x81\x01\x81\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x06\x86W` \x93R\x83\x81R\x82``\x81\x83\x01R\x82\x87\x01\x01R\x01a\x07\xA5V[a\xFF\xFF\x80\x91\x16\x90\x81\x14a\x07}W`\x01\x01\x90V\xFE\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\xA2dipfsX\"\x12 \x8F@\xB7\xCCC\xBE\xF5Z\xF6b\xAE\xF9\xDD\xBBHr\x90[gl\x83\x97\x92\xDF\xDC\xE8\xBE\xAF\xE2\xCE\x83wdsolcC\0\x08\x17\x003";
    /// The bytecode of the contract.
    pub static DIAMONDLOUPEFACET_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R`\x046\x10\x15a\0\x12W`\0\x80\xFD[`\x005`\xE0\x1C\x80c\x01\xFF\xC9\xA7\x14a\0gW\x80cR\xEFk,\x14a\0bW\x80cz\x0E\xD6'\x14a\0]W\x80c\xAD\xFC\xA1^\x14a\0XWc\xCD\xFF\xAC\xC6\x14a\0SW`\0\x80\xFD[a\x05\xEDV[a\x052V[a\x02\xFCV[a\x01'V[4a\0\xBAW`\x01`\x01`\xE0\x1B\x03\x19a\0~6a\0\xBFV[\x16`\0R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4` R`\xFF`@`\0 T\x16\x15\x15`\x80R` `\x80\xF3[`\0\x80\xFD[` \x90`\x03\x19\x01\x12a\0\xBAW`\x045`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03a\0\xBAW\x90V[` \x90` `@\x81\x83\x01\x92\x82\x81R\x85Q\x80\x94R\x01\x93\x01\x91`\0[\x82\x81\x10a\x01\nWPPPP\x90V[\x83Q`\x01`\x01`\xA0\x1B\x03\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01a\0\xFCV[4a\0\xBAW`\0\x80`\x03\x196\x01\x12a\x02?W`\0\x80Q` a\x07\xFA\x839\x81Q\x91RT\x90a\x01S\x82a\x06\xA3V[\x90\x80\x80[\x84\x82\x10a\x01sW\x82\x84R`@Q\x80a\x01o\x86\x82a\0\xE2V[\x03\x90\xF3[\x90\x91a\x01\xA6a\x01\x99a\x01\x94a\x01\x87\x86a\x06\xE6V[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x90V[a\x06\x12V[T`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x82`\x01`\x01`\xA0\x1B\x03\x82\x16\x81[\x84\x81\x10a\x01\xFCW[PPa\x01\xF2W\x81a\x01\xE4a\x01\xE9\x92a\x01\xD5`\x01\x95\x89a\x07DV[`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90RV[a\x07nV[\x92[\x01\x90a\x01WV[P\x91`\x01\x90a\x01\xEBV[a\x02%a\x02\x19a\x02\x0C\x83\x8Ba\x07DV[Q`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x82\x14a\x023W`\x01\x01a\x01\xB3V[PPP`\x018\x80a\x01\xBBV[\x80\xFD[` \x80\x82\x01\x90\x80\x83R\x83Q\x80\x92R`@\x92`@\x81\x01\x82`@\x85`\x05\x1B\x84\x01\x01\x96\x01\x94`\0\x80\x93[\x86\x85\x10a\x02{WPPPPPPPP\x90V[\x90\x91\x92\x93\x94\x80\x96\x97\x98`?\x19\x83\x82\x03\x01\x86R\x89Q\x82``\x81\x88\x85\x01\x93`\x01\x80`\xA0\x1B\x03\x81Q\x16\x86R\x01Q\x93\x88\x83\x82\x01R\x84Q\x80\x94R\x01\x92\x01\x90\x85\x90[\x80\x82\x10a\x02\xD8WPPP\x90\x80`\x01\x92\x9A\x01\x95\x01\x95\x01\x93\x96\x95\x94\x92\x91\x90a\x02iV[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R\x8A\x94\x93\x84\x01\x93\x90\x92\x01\x91`\x01\x91\x90\x91\x01\x90a\x02\xB7V[4a\0\xBAW`\0\x80`\x03\x196\x01\x12a\x02?W`\0\x80Q` a\x07\xFA\x839\x81Q\x91RTa\x03'\x81a\x07\x82V[\x90a\x031\x81a\x06\xA3V[\x92\x80\x91\x81\x90[\x80\x82\x10a\x03\x91WPP[\x81\x81\x10a\x03YW\x81\x83R`@Q\x80a\x01o\x85\x82a\x02BV[\x80a\x03{a\x03ta\x03l`\x01\x94\x88a\x07DV[Qa\xFF\xFF\x16\x90V[a\xFF\xFF\x16\x90V[` a\x03\x87\x83\x87a\x07DV[Q\x01QR\x01a\x03AV[\x90\x92a\x03\x9Fa\x01\x87\x85a\x06\xE6V[a\x03\xABa\x01\x99\x82a\x06\x12V[\x84`\x01`\x01`\xA0\x1B\x03\x82\x16\x81[\x85\x81\x10a\x04UW[PPa\x04JW\x91a\x04.a\x04A\x92a\x03\xEE`\x01\x95a\x03\xDE\x85\x8Ca\x07DV[Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90RV[a\x04\x1E\x83a\x03\xFB\x88a\x06\xA3V[\x8Ba\x04\t` \x93\x84\x92a\x07DV[Q\x01Ra\x04\x16\x85\x8Ca\x07DV[Q\x01Qa\x077V[`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16\x90RV[a\x01\xE4a\x04;\x82\x8Aa\x07DV[`\x01\x90RV[\x93[\x01\x90a\x037V[PP\x92`\x01\x90a\x04CV[\x8A\x85\x8B\x84a\x04wa\x02\x19a\x04i\x87\x85a\x07DV[QQ`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x14a\x04\x87WPPP`\x01\x01a\x03\xB8V[a\x04\xE3\x95P\x83\x80\x95Pa\x04\xD3\x93a\x04\xC9a\x04\xCE\x94a\x04\x1E` a\x04\xB0a\x04\xDA\x9Aa\x03l\x98a\x07DV[Q\x01Qa\x04\xC3a\x03ta\x03l\x88\x88a\x07DV[\x90a\x07DV[a\x07DV[a\x07\xE6V[\x91\x8Ba\x07DV[\x90a\xFF\xFF\x16\x90RV[`\x018\x80a\x03\xC0V[` \x90` `@\x81\x83\x01\x92\x82\x81R\x85Q\x80\x94R\x01\x93\x01\x91`\0[\x82\x81\x10a\x05\x14WPPPP\x90V[\x83Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01a\x05\x06V[4a\0\xBAW` 6`\x03\x19\x01\x12a\0\xBAW`\x01`\x01`\xA0\x1B\x03`\x045\x81\x81\x16\x90\x81\x90\x03a\0\xBAW`\0\x80Q` a\x07\xFA\x839\x81Q\x91RT\x91`\0\x90a\x05v\x84a\x06\xA3V[\x92`\0[\x85\x81\x10a\x05\x92W\x83\x85R`@Q\x80a\x01o\x87\x82a\x04\xECV[a\x05\x9B\x81a\x06\xE6V[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x83a\x05\xAF\x82a\x06\x12V[T\x16\x83\x14a\x05\xC1W[P`\x01\x01a\x05zV[\x84a\x05\xE6\x91a\x05\xD3`\x01\x94\x97\x89a\x07DV[`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16\x90Ra\x07nV[\x93\x90a\x05\xB8V[4a\0\xBAW` `\x01`\x01`\xA0\x1B\x03a\x06\x08a\x01\x946a\0\xBFV[T\x16`@Q\x90\x81R\xF3[c\xFF\xFF\xFF\xFF`\xE0\x1B\x16`\0R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@`\0 \x90V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[`@Q\x91\x90`\x1F\x01`\x1F\x19\x16\x82\x01g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x83\x82\x10\x17a\x06\x86W`@RV[a\x06JV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x06\x86W`\x05\x1B` \x01\x90V[\x90a\x06\xB5a\x06\xB0\x83a\x06\x8BV[a\x06`V[\x82\x81R\x80\x92a\x06\xC6`\x1F\x19\x91a\x06\x8BV[\x01\x90` 6\x91\x017V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x90`\0\x80Q` a\x07\xFA\x839\x81Q\x91R\x80T\x83\x10\x15a\x072W`\0R`\x1C\x82`\x03\x1C\x7F\xB6[\xEC\xA8\xB6\xFAx\x8B\xCB\x15(\xC2\xAB_M\xC6\xBC\x98\xE5\x89eP\xBA\xA0\x13\xD83\x0F\xAB\x0B\x86\xF4\x01\x92`\x02\x1B\x16\x90V[a\x06\xD0V[\x80Q\x15a\x072W` \x01\x90V[\x80Q\x82\x10\x15a\x072W` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`\0\x19\x81\x14a\x07}W`\x01\x01\x90V[a\x07XV[\x90a\x07\x8Fa\x06\xB0\x83a\x06\x8BV[\x82\x81R\x80\x92a\x07\xA0`\x1F\x19\x91a\x06\x8BV[\x01`\0\x80[\x82\x81\x10a\x07\xB2WPPPPV[`@\x90\x81Q\x82\x81\x01\x81\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x06\x86W` \x93R\x83\x81R\x82``\x81\x83\x01R\x82\x87\x01\x01R\x01a\x07\xA5V[a\xFF\xFF\x80\x91\x16\x90\x81\x14a\x07}W`\x01\x01\x90V\xFE\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\xA2dipfsX\"\x12 \x8F@\xB7\xCCC\xBE\xF5Z\xF6b\xAE\xF9\xDD\xBBHr\x90[gl\x83\x97\x92\xDF\xDC\xE8\xBE\xAF\xE2\xCE\x83wdsolcC\0\x08\x17\x003";
    /// The deployed bytecode of the contract.
    pub static DIAMONDLOUPEFACET_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
    pub struct DiamondLoupeFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for DiamondLoupeFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for DiamondLoupeFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for DiamondLoupeFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for DiamondLoupeFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(DiamondLoupeFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> DiamondLoupeFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                DIAMONDLOUPEFACET_ABI.clone(),
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
                DIAMONDLOUPEFACET_ABI.clone(),
                DIAMONDLOUPEFACET_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `facetAddress` (0xcdffacc6) function
        pub fn facet_address(
            &self,
            function_selector: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::Address> {
            self.0
                .method_hash([205, 255, 172, 198], function_selector)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `facetAddresses` (0x52ef6b2c) function
        pub fn facet_addresses(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers::core::types::Address>,
        > {
            self.0
                .method_hash([82, 239, 107, 44], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `facetFunctionSelectors` (0xadfca15e) function
        pub fn facet_function_selectors(
            &self,
            facet: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<[u8; 4]>> {
            self.0
                .method_hash([173, 252, 161, 94], facet)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `facets` (0x7a0ed627) function
        pub fn facets(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<Facet>> {
            self.0
                .method_hash([122, 14, 214, 39], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `supportsInterface` (0x01ffc9a7) function
        pub fn supports_interface(
            &self,
            interface_id: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([1, 255, 201, 167], interface_id)
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for DiamondLoupeFacet<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Container type for all input parameters for the `facetAddress` function with signature `facetAddress(bytes4)` and selector `0xcdffacc6`
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
    #[ethcall(name = "facetAddress", abi = "facetAddress(bytes4)")]
    pub struct FacetAddressCall {
        pub function_selector: [u8; 4],
    }
    ///Container type for all input parameters for the `facetAddresses` function with signature `facetAddresses()` and selector `0x52ef6b2c`
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
    #[ethcall(name = "facetAddresses", abi = "facetAddresses()")]
    pub struct FacetAddressesCall;
    ///Container type for all input parameters for the `facetFunctionSelectors` function with signature `facetFunctionSelectors(address)` and selector `0xadfca15e`
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
    #[ethcall(
        name = "facetFunctionSelectors",
        abi = "facetFunctionSelectors(address)"
    )]
    pub struct FacetFunctionSelectorsCall {
        pub facet: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `facets` function with signature `facets()` and selector `0x7a0ed627`
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
    #[ethcall(name = "facets", abi = "facets()")]
    pub struct FacetsCall;
    ///Container type for all input parameters for the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `0x01ffc9a7`
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
    #[ethcall(name = "supportsInterface", abi = "supportsInterface(bytes4)")]
    pub struct SupportsInterfaceCall {
        pub interface_id: [u8; 4],
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum DiamondLoupeFacetCalls {
        FacetAddress(FacetAddressCall),
        FacetAddresses(FacetAddressesCall),
        FacetFunctionSelectors(FacetFunctionSelectorsCall),
        Facets(FacetsCall),
        SupportsInterface(SupportsInterfaceCall),
    }
    impl ::ethers::core::abi::AbiDecode for DiamondLoupeFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <FacetAddressCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FacetAddress(decoded));
            }
            if let Ok(decoded) =
                <FacetAddressesCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FacetAddresses(decoded));
            }
            if let Ok(decoded) =
                <FacetFunctionSelectorsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FacetFunctionSelectors(decoded));
            }
            if let Ok(decoded) = <FacetsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Facets(decoded));
            }
            if let Ok(decoded) =
                <SupportsInterfaceCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::SupportsInterface(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for DiamondLoupeFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::FacetAddress(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::FacetAddresses(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::FacetFunctionSelectors(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Facets(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SupportsInterface(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for DiamondLoupeFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::FacetAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::FacetAddresses(element) => ::core::fmt::Display::fmt(element, f),
                Self::FacetFunctionSelectors(element) => ::core::fmt::Display::fmt(element, f),
                Self::Facets(element) => ::core::fmt::Display::fmt(element, f),
                Self::SupportsInterface(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<FacetAddressCall> for DiamondLoupeFacetCalls {
        fn from(value: FacetAddressCall) -> Self {
            Self::FacetAddress(value)
        }
    }
    impl ::core::convert::From<FacetAddressesCall> for DiamondLoupeFacetCalls {
        fn from(value: FacetAddressesCall) -> Self {
            Self::FacetAddresses(value)
        }
    }
    impl ::core::convert::From<FacetFunctionSelectorsCall> for DiamondLoupeFacetCalls {
        fn from(value: FacetFunctionSelectorsCall) -> Self {
            Self::FacetFunctionSelectors(value)
        }
    }
    impl ::core::convert::From<FacetsCall> for DiamondLoupeFacetCalls {
        fn from(value: FacetsCall) -> Self {
            Self::Facets(value)
        }
    }
    impl ::core::convert::From<SupportsInterfaceCall> for DiamondLoupeFacetCalls {
        fn from(value: SupportsInterfaceCall) -> Self {
            Self::SupportsInterface(value)
        }
    }
    ///Container type for all return fields from the `facetAddress` function with signature `facetAddress(bytes4)` and selector `0xcdffacc6`
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
    pub struct FacetAddressReturn {
        pub facet_address: ::ethers::core::types::Address,
    }
    ///Container type for all return fields from the `facetAddresses` function with signature `facetAddresses()` and selector `0x52ef6b2c`
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
    pub struct FacetAddressesReturn {
        pub facet_addresses: ::std::vec::Vec<::ethers::core::types::Address>,
    }
    ///Container type for all return fields from the `facetFunctionSelectors` function with signature `facetFunctionSelectors(address)` and selector `0xadfca15e`
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
    pub struct FacetFunctionSelectorsReturn {
        pub facet_function_selectors: ::std::vec::Vec<[u8; 4]>,
    }
    ///Container type for all return fields from the `facets` function with signature `facets()` and selector `0x7a0ed627`
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
    pub struct FacetsReturn {
        pub facets: ::std::vec::Vec<Facet>,
    }
    ///Container type for all return fields from the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `0x01ffc9a7`
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
    pub struct SupportsInterfaceReturn(pub bool);
    ///`Facet(address,bytes4[])`
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
    pub struct Facet {
        pub facet_address: ::ethers::core::types::Address,
        pub function_selectors: ::std::vec::Vec<[u8; 4]>,
    }
}
