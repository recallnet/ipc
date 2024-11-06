// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_machine::{
    GET_ADDRESS_METHOD, GET_METADATA_METHOD, INIT_METHOD, METHOD_CONSTRUCTOR,
};
use fvm_ipld_encoding::tuple::*;
use num_derive::FromPrimitive;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{serde_as, DeserializeAs, SerializeAs};

pub const SQLITE_ACTOR_NAME: &str = "sqlite";
pub const SQLITE_PAGE_SIZE: usize = 4096;
pub const SQLITE_BUCKET_SIZE: usize = 256; // IPLD block limit w/ standard page size of 4096

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Init = INIT_METHOD,
    GetAddress = GET_ADDRESS_METHOD,
    GetMetadata = GET_METADATA_METHOD,
    Query = frc42_dispatch::method_hash!("Query"),
    Execute = frc42_dispatch::method_hash!("Execute"),
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct ExecuteParams {
    pub stmts: Vec<String>,
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct ExecuteReturn {
    pub effected_rows: usize,
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct QueryParams {
    pub stmt: String,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryReturn {
    pub cols: Vec<String>,
    #[serde_as(as = "Vec<Vec<ValueDef>>")]
    pub rows: Vec<Vec<rusqlite::types::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "rusqlite::types::Value", untagged)]
pub enum ValueDef {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl SerializeAs<rusqlite::types::Value> for ValueDef {
    fn serialize_as<S>(value: &rusqlite::types::Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ValueDef::serialize(value, serializer)
    }
}

impl<'de> DeserializeAs<'de, rusqlite::types::Value> for ValueDef {
    fn deserialize_as<D>(deserializer: D) -> Result<rusqlite::types::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        ValueDef::deserialize(deserializer)
    }
}
