use std::collections::HashMap;
use anyhow::Error;
use fendermint_actor_blobs_shared::state::Hash;
use recall_actor_sdk::TryIntoEVMEvent;
use recall_sol_facade::bucket as sol;

pub struct ObjectAdded<'a> {
    pub key: &'a Vec<u8>,
    pub blob_hash: &'a Hash,
    pub metadata: &'a HashMap<String, String>,
}
impl<'a> ObjectAdded<'a> {
    pub fn new(key: &'a Vec<u8>, blob_hash: &'a Hash, metadata: &'a HashMap<String, String>) -> Self {
        Self { key, blob_hash, metadata }
    }
}
impl TryIntoEVMEvent for ObjectAdded<'_> {
    type Target = sol::Event;

    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        let metadata = fvm_ipld_encoding::to_vec(self.metadata)?;
        Ok(sol::Event::ObjectAdded(sol::ObjectAdded {
            key: self.key.clone().into(),
            blobHash: self.blob_hash.0.into(),
            metadata: metadata.into(),
        }))
    }
}