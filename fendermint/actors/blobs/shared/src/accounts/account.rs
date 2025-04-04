// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fil_actors_runtime::{runtime::Runtime, ActorError};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};
use recall_actor_sdk::util::to_delegated_address;

use crate::credit::{Credit, CreditApproval, CreditApprovals};

/// The stored representation of a credit account.
#[derive(Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Account {
    /// Total size of all blobs managed by the account.
    pub capacity_used: u64,
    /// Current free credit in byte-blocks that can be used for new commitments.
    pub credit_free: Credit,
    /// Current committed credit in byte-blocks that will be used for debits.
    pub credit_committed: Credit,
    /// Optional default sponsor account address.
    pub credit_sponsor: Option<Address>,
    /// The chain epoch of the last debit.
    pub last_debit_epoch: ChainEpoch,
    /// Credit approvals to other accounts from this account, keyed by receiver.
    pub approvals_to: CreditApprovals,
    /// Credit approvals to this account from other accounts, keyed by sender.
    pub approvals_from: CreditApprovals,
    /// The maximum allowed TTL for actor's blobs.
    pub max_ttl: ChainEpoch,
    /// The total token value an account has used to buy credits.
    pub gas_allowance: TokenAmount,
}

impl Account {
    pub fn new<BS: Blockstore>(
        store: &BS,
        current_epoch: ChainEpoch,
        max_ttl: ChainEpoch,
    ) -> Result<Self, ActorError> {
        Ok(Self {
            capacity_used: 0,
            credit_free: Credit::default(),
            credit_committed: Credit::default(),
            credit_sponsor: None,
            last_debit_epoch: current_epoch,
            approvals_to: CreditApprovals::new(store)?,
            approvals_from: CreditApprovals::new(store)?,
            max_ttl,
            gas_allowance: TokenAmount::default(),
        })
    }
}

impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("capacity_used", &self.capacity_used)
            .field("credit_free", &self.credit_free)
            .field("credit_committed", &self.credit_committed)
            .field("credit_sponsor", &self.credit_sponsor)
            .field("last_debit_epoch", &self.last_debit_epoch)
            .field("max_ttl", &self.max_ttl)
            .field("gas_allowance", &self.gas_allowance)
            .finish()
    }
}

/// The return type used for Account.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AccountInfo {
    /// Total size of all blobs managed by the account.
    pub capacity_used: u64,
    /// Current free credit in byte-blocks that can be used for new commitments.
    pub credit_free: Credit,
    /// Current committed credit in byte-blocks that will be used for debits.
    pub credit_committed: Credit,
    /// Optional default sponsor account address.
    pub credit_sponsor: Option<Address>,
    /// The chain epoch of the last debit.
    pub last_debit_epoch: ChainEpoch,
    /// Credit approvals to other accounts from this account, keyed by receiver.
    pub approvals_to: HashMap<Address, CreditApproval>,
    /// Credit approvals to this account from other accounts, keyed by sender.
    pub approvals_from: HashMap<Address, CreditApproval>,
    /// The maximum allowed TTL for actor's blobs.
    pub max_ttl: ChainEpoch,
    /// The total token value an account has used to buy credits.
    pub gas_allowance: TokenAmount,
}

impl AccountInfo {
    pub fn from(rt: &impl Runtime, account: Account) -> Result<Self, ActorError> {
        let store = rt.store();
        let mut approvals_to = HashMap::new();
        account
            .approvals_to
            .hamt(store)?
            .for_each(|address, approval| {
                let external_account_address = to_delegated_address(rt, address)?;
                approvals_to.insert(external_account_address, approval.clone());
                Ok(())
            })?;

        let mut approvals_from = HashMap::new();
        account
            .approvals_from
            .hamt(store)?
            .for_each(|address, approval| {
                let external_account_address = to_delegated_address(rt, address)?;
                approvals_from.insert(external_account_address, approval.clone());
                Ok(())
            })?;

        Ok(AccountInfo {
            capacity_used: account.capacity_used,
            credit_free: account.credit_free,
            credit_committed: account.credit_committed,
            credit_sponsor: account.credit_sponsor,
            last_debit_epoch: account.last_debit_epoch,
            approvals_to,
            approvals_from,
            max_ttl: account.max_ttl,
            gas_allowance: account.gas_allowance,
        })
    }
}
