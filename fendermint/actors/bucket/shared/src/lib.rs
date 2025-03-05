use std::collections::HashMap;
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fendermint_actor_blobs_shared::state::{Hash, PublicKey};

/// Params for adding an object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AddParams {
    /// Source Iroh node ID used for ingestion.
    pub source: PublicKey,
    /// Object key.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    /// Object blake3 hash.
    pub hash: Hash,
    /// Blake3 hash of the metadata to use for object recovery.
    pub recovery_hash: Hash,
    /// Object size.
    pub size: u64,
    /// Object time-to-live epochs.
    /// If not specified, the current default TTL from the config actor is used.
    pub ttl: Option<ChainEpoch>,
    /// Object metadata.
    pub metadata: HashMap<String, String>,
    /// Whether to overwrite a key if it already exists.
    pub overwrite: bool,
    /// Account address that initiated the call
    pub from: Address,
}

/// Params for deleting an object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct DeleteParams {
    /// Key of the object to delete from a bucket.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    /// Account address that initiated the call
    pub from: Address,
}

/// The stored representation of an object in the bucket.
#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Object {
    /// The object blake3 hash.
    pub hash: Hash,
    /// Blake3 hash of the metadata to use for object recovery.
    pub recovery_hash: Hash,
    /// The object size.
    pub size: u64,
    /// Expiry block.
    pub expiry: ChainEpoch,
    /// User-defined object metadata (e.g., last modified timestamp, etc.).
    pub metadata: HashMap<String, String>,
}

/// Params for listing objects.
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ListParams {
    /// The prefix to filter objects by.
    #[serde(with = "strict_bytes")]
    pub prefix: Vec<u8>,
    /// The delimiter used to define object hierarchy.
    #[serde(with = "strict_bytes")]
    pub delimiter: Vec<u8>,
    /// The key to start listing objects from.
    pub start_key: Option<Vec<u8>>,
    /// The maximum number of objects to list.
    pub limit: u64,
}