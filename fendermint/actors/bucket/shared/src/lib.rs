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