use anyhow::Error;
use recall_sol_facade::blobs as sol;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fendermint_actor_blobs_shared::state::Hash;
use recall_actor_sdk::TryIntoEVMEvent;
use recall_sol_facade::primitives::U256;
use recall_sol_facade::types::H160;

pub struct BlobAdded<'a> {
    pub subscriber: Address,
    pub hash: &'a Hash,
    pub size: u64,
    pub expiry: ChainEpoch,
    pub bytes_used: u64,
}

impl TryIntoEVMEvent for BlobAdded<'_> {
    type Target = sol::Event;

    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        let subscriber: H160 = self.subscriber.try_into()?;
        Ok(sol::Event::BlobAdded(sol::BlobAdded {
            subscriber: subscriber.into(),
            hash: self.hash.0.into(),
            size: U256::from(self.size),
            expiry: U256::from(self.expiry),
            bytesUsed: U256::from(self.bytes_used),
        }))
    }
}