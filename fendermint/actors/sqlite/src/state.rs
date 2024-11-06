use std::collections::HashMap;

use fendermint_actor_machine::{Kind, MachineAddress, MachineState};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;

use crate::db::DB;
use crate::{SQLITE_BUCKET_SIZE, SQLITE_PAGE_SIZE};

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The machine address set by the init actor.
    pub address: MachineAddress,
    /// The machine rubust owner address.
    pub owner: Address,
    /// User-defined metadata.
    pub metadata: HashMap<String, String>,
    /// The sqlite database
    pub db: DB,
}

impl MachineState for State {
    fn new<BS: Blockstore>(
        store: &BS,
        owner: Address,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<Self, ActorError> {
        let db = DB::new(store, &[], SQLITE_PAGE_SIZE, SQLITE_BUCKET_SIZE)?;
        Ok(Self {
            address: Default::default(),
            owner,
            db,
            metadata,
        })
    }

    fn init(&mut self, address: Address) -> anyhow::Result<(), ActorError> {
        self.address.set(address)
    }

    fn address(&self) -> MachineAddress {
        self.address.clone()
    }

    fn kind(&self) -> Kind {
        Kind::Timehub
    }

    fn owner(&self) -> Address {
        self.owner
    }

    fn metadata(&self) -> HashMap<String, String> {
        self.metadata.clone()
    }
}
