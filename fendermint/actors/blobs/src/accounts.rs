// Copyright 2024 Hoku Contributors

use std::ops::{Deref, DerefMut};
use cid::Cid;
use fendermint_actor_blobs_shared::state::Account;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use hoku_ipld::map::{Map, DEFAULT_HAMT_CONFIG};


/// A simple wrapper type around an Account that takes care of flushing the Accounts HAMT when
/// it goes out of scope.
pub struct AccountHolder<'a, BS: Blockstore> {
    /// The wrapped Account
    pub account: Account,

    /// The address of the account
    address: Address,

    /// The HAMT that's responsible for storing the Account.
    accounts: Accounts<BS>,

    /// A reference to the root of the Accounts HAMT that needs to be updated when this
    /// AccountHolder goes out of scope and flushes the underlying HAMT.
    accounts_root: &'a mut Cid,
}

impl<'a, BS> AccountHolder<'a, BS>
where
    BS: Blockstore,
{
    pub fn new(account: Account, address: Address, accounts: Accounts<BS>, accounts_root: &'a mut Cid) -> Self {
        Self { account, address, accounts, accounts_root }
    }
}

impl<'a, BS> Deref for AccountHolder<'a, BS>
where
    BS: Blockstore,
{
    type Target = Account;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl<'a, BS> DerefMut for AccountHolder<'a, BS>
where
    BS: Blockstore,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}

impl<'a, BS> Drop for AccountHolder<'a, BS>
where
    BS: Blockstore,
{
    fn drop(&mut self) {
        // TODO: no unwrap
        let root = self.accounts.set_and_flush(&self.address, self.account.clone()).unwrap();
        *self.accounts_root = root;
    }
}

pub type AccountMap<BS> = Map<BS, Address, Account>;

pub struct Accounts<BS: Blockstore> {
    pub map: AccountMap<BS>,
}

impl<'a, BS> Accounts<BS>
where
    BS: Blockstore,
{
    pub fn flush_empty(store: BS) -> Result<Cid, ActorError> {
        AccountMap::flush_empty(store, DEFAULT_HAMT_CONFIG)
    }

    pub fn load(store: &'a BS, root: &Cid) -> Result<Accounts<&'a BS>, ActorError> {
        let map = AccountMap::load(store, root, DEFAULT_HAMT_CONFIG, "accounts")?;
        Ok(Accounts { map })
    }

    pub fn get(&self, addr: &Address) -> Result<Option<Account>, ActorError> {
        self.map.get(addr).map(|a| a.cloned())
    }

    pub fn get_or_err(&self, addr: &Address) -> Result<Account, ActorError> {
        self.get(addr)?
            .ok_or(ActorError::not_found(format!("account {} not found", addr)))
    }

    pub fn get_or_create(
        self,
        addr: &Address, // todo take by value
        current_epoch: ChainEpoch,
        accounts_root: &mut Cid,
    ) -> Result<AccountHolder<'a, BS>, ActorError> {
        let account: Account = if let Some(a) = self.map.get(addr)? {
            Ok(a.clone())
        } else {
            Ok(Account::new(current_epoch))
        }?;

        Ok(AccountHolder::new(account, addr.clone(), self, accounts_root))
    }

    pub fn set_and_flush(&mut self, addr: &Address, account: Account) -> Result<Cid, ActorError> {
        self.map.set(addr, account)?;
        self.map.flush()
    }

    /// Consumes the underlying map's HAMT and returns the Blockstore it owns.
    pub fn into_store(self) -> BS {
        self.map.into_store()
    }
}
