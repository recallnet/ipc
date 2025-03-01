// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::{Account, Credit, CreditApproval, TokenCreditRate};
use fendermint_actor_recall_config_shared::RecallConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};
use recall_ipld::hamt;

/// Helper struct to track caller and optional sponsor with its credit delegation.
pub struct Caller<'a, BS: Blockstore> {
    /// The caller address.
    address: Address,
    /// The caller account.
    account: Account,
    /// An optional credit delegation from sponsor to caller.
    delegation: Option<CreditDelegation<'a, &'a BS>>,
}

impl<'a, BS: Blockstore> Caller<'a, BS> {
    /// Loads the caller and optional sponsor account with its credit delegation.
    pub fn load_or_err(
        store: &'a BS,
        accounts: &hamt::map::Hamt<'a, &'a BS, Address, Account>,
        caller: Address,
        sponsor: Address,
    ) -> anyhow::Result<Self, ActorError> {
        let account = accounts.get_or_err(&caller)?;
        Self::load(store, accounts, caller, account, sponsor)
    }

    /// Loads the caller and optional sponsor account with its credit delegation.
    /// The caller account will be created if one does not exist.
    pub fn load_or_create(
        store: &'a BS,
        accounts: &hamt::map::Hamt<'a, &'a BS, Address, Account>,
        caller: Address,
        sponsor: Address,
        current_epoch: ChainEpoch,
        max_ttl: ChainEpoch,
    ) -> anyhow::Result<Self, ActorError> {
        let account =
            accounts.get_or_create(&caller, || Account::new(store, current_epoch, max_ttl))?;
        Self::load(store, accounts, caller, account, sponsor)
    }

    /// Loads the caller and optional sponsor account with its credit delegation.
    fn load(
        store: &'a BS,
        accounts: &hamt::map::Hamt<'a, &'a BS, Address, Account>,
        caller: Address,
        caller_account: Account,
        sponsor: Address,
    ) -> anyhow::Result<Self, ActorError> {
        if sponsor != caller {
            let sponsor_account = accounts.get_or_err(&sponsor)?;
            let approvals_to = sponsor_account.approvals_to.hamt(store)?;
            let approval_to = approvals_to
                .get(&caller)?
                .ok_or(ActorError::forbidden(format!(
                    "approval to {} from {} not found",
                    caller, sponsor
                )))?;
            let approvals_from = caller_account.approvals_from.hamt(store)?;
            let approval_from =
                approvals_from
                    .get(&sponsor)?
                    .ok_or(ActorError::forbidden(format!(
                        "approval from {} to {} not found",
                        sponsor, caller
                    )))?;
            let delegation = CreditDelegation {
                from: sponsor,
                from_account: sponsor_account,
                approvals_from,
                approvals_to,
                approval_from,
                approval_to,
            };
            Ok(Self {
                address: caller,
                account: caller_account,
                delegation: Some(delegation),
            })
        } else {
            Ok(Self {
                address: caller,
                account: caller_account,
                delegation: None,
            })
        }
    }

    /// Returns the caller address.
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns the subscriber address.
    /// The subscriber is the account responsible for credit and gas fees.
    /// The subscriber is the caller or the sponsor if one exists.
    pub fn subscriber_address(&self) -> Address {
        self.delegation
            .as_ref()
            .map(|delegation| delegation.from)
            .unwrap_or(self.address)
    }

    /// Returns the delegate address.
    /// The delegate only exists if there's a sponsor.
    /// If present, the delegate address will be the caller address.
    pub fn delegate_address(&self) -> Option<Address> {
        self.delegation.as_ref().map(|_| self.address)
    }

    /// Returns whether the caller is a delegate.
    pub fn is_delegate(&self) -> bool {
        self.delegation.is_some()
    }

    /// Returns the subscriber account.
    /// The subscriber is the account responsible for credit and gas fees.
    /// The subscriber is the caller or the sponsor if one exists.
    pub fn subscriber(&self) -> &Account {
        if let Some(delegation) = self.delegation.as_ref() {
            &delegation.from_account
        } else {
            &self.account
        }
    }

    /// Validates whether the subscriber has enough credit to cover the amount,
    /// and whether the delegation has enough credit to cover the amount.
    pub fn validate_credit_usage(
        &mut self,
        amount: &Credit,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        // Check subscriber
        if &self.subscriber().credit_free < amount {
            return Err(ActorError::insufficient_funds(format!(
                "account {} has insufficient credit (available: {}; required: {})",
                self.subscriber_address(),
                &self.subscriber().credit_free,
                amount
            )));
        }
        // Check delegation
        if let Some(delegation) = self.delegation.as_ref() {
            delegation.validate_credit_usage(amount, current_epoch)?;
        }
        Ok(())
    }

    pub fn add_credit(&mut self, amount: &Credit) {
        if let Some(delegation) = self.delegation.as_mut() {
            delegation.from_account.credit_free += amount;
        } else {
            self.account.credit_free += amount;
        }
    }

    // pub fn ensure_credit_or_buy(
    //     &mut self,
    //     amount: &Credit,
    //     value: &TokenAmount,
    //     credit_rate: &TokenCreditRate,
    //     current_epoch: ChainEpoch,
    // ) -> anyhow::Result<TokenAmount, ActorError> {
    //     // Sanity check
    //     if value.is_negative() {
    //         return Err(ActorError::illegal_argument("negative token value".into()));
    //     }
    //
    //     // Check delegate
    //     if let Some(delegation) = self.delegation.as_mut() {
    //         if value.is_positive() {
    //             return Err(ActorError::illegal_argument(
    //                 "cannot auto-buy credits for a sponsor".into(),
    //             ));
    //         }
    //     }
    // }

    /// Returns an amount to the subscriber committed credit.
    pub fn return_committed_credit(&mut self, amount: &Credit) {
        if let Some(delegation) = self.delegation.as_mut() {
            delegation.from_account.credit_committed += amount;
        } else {
            self.account.credit_committed += amount;
        }
    }

    pub fn add_gas_allowance(&mut self, amount: &TokenAmount) {
        if let Some(delegation) = self.delegation.as_mut() {
            delegation.from_account.gas_allowance += amount;
        } else {
            self.account.gas_allowance += amount;
        }
    }

    /// Deducts gas allowance from the subscriber.
    pub fn deduct_gas_allowance(
        &mut self,
        amount: &TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        if let Some(delegation) = self.delegation.as_mut() {
            delegation.deduct_gas_allowance(amount, current_epoch)?;
            delegation.from_account.gas_allowance -= amount;
        } else {
            self.account.gas_allowance -= amount;
        }
        Ok(())
    }

    /// Validates a blob TTL for the subscriber.
    pub fn validate_ttl_usage(
        &self,
        config: &RecallConfig,
        ttl: Option<ChainEpoch>,
    ) -> anyhow::Result<ChainEpoch, ActorError> {
        let ttl = ttl.unwrap_or(config.blob_default_ttl);
        if ttl < config.blob_min_ttl {
            return Err(ActorError::illegal_argument(format!(
                "minimum blob TTL is {}",
                config.blob_min_ttl
            )));
        } else if ttl > self.subscriber().max_ttl {
            return Err(ActorError::forbidden(format!(
                "attempt to add a blob with TTL ({}) that exceeds account's max allowed TTL ({})",
                ttl,
                self.subscriber().max_ttl,
            )));
        }
        Ok(ttl)
    }

    /// Saves the caller state to accounts.
    pub fn save(
        &mut self,
        accounts: &mut hamt::map::Hamt<'a, &'a BS, Address, Account>,
    ) -> anyhow::Result<(), ActorError> {
        if let Some(delegation) = self.delegation.as_mut() {
            // Save the caller's "from" approval
            self.account.approvals_from.save_tracked(
                delegation
                    .approvals_from
                    .set_and_flush_tracked(&delegation.from, delegation.approval_from.clone())?,
            );
            // Save the sponsor's "to" approval
            delegation.from_account.approvals_to.save_tracked(
                delegation
                    .approvals_to
                    .set_and_flush_tracked(&self.address, delegation.approval_to.clone())?,
            );
            // Save the sponsor account
            accounts.set(&delegation.from, delegation.from_account.clone())?;
        }
        // Save the caller account
        accounts.set(&self.address, self.account.clone())?;
        Ok(())
    }
}

/// Helper for handling credit approvals.
struct CreditDelegation<'a, BS: Blockstore> {
    /// The optional sponsor address.
    pub from: Address,
    /// An optional sponsor account.
    /// If present, the sponsor is responsible for credit and gas fees.
    pub from_account: Account,
    /// Approvals from sponsor to caller.
    pub approvals_from: hamt::map::Hamt<'a, BS, Address, CreditApproval>,
    /// Approvals to caller from sponsor.
    pub approvals_to: hamt::map::Hamt<'a, BS, Address, CreditApproval>,
    /// Approval from sponsor to caller.
    pub approval_from: CreditApproval,
    /// Approval to caller from sponsor.
    pub approval_to: CreditApproval,
}

impl<'a, BS: Blockstore> CreditDelegation<'a, BS> {
    /// Validate whether the delegation has enough allowance for the credit amount.
    pub fn validate_credit_usage(
        &self,
        amount: &TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        self.approval_from.validate_expiration(current_epoch)?;
        self.approval_from.validate_credit_usage(amount)?;
        self.approval_to.validate_expiration(current_epoch)?;
        self.approval_to.validate_credit_usage(amount)?;
        Ok(())
    }

    /// Deduct gas allowance from the delegation.
    /// Note: This does not flush the approval maps or the "from" account.
    pub fn deduct_gas_allowance(
        &mut self,
        amount: &TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        self.approval_from.validate_expiration(current_epoch)?;
        self.approval_from.validate_gas_usage(amount)?;
        self.approval_to.validate_expiration(current_epoch)?;
        self.approval_to.validate_gas_usage(amount)?;
        // Add to both approvals used amount
        self.approval_from.gas_allowance_used += amount;
        self.approval_to.gas_allowance_used += amount;
        Ok(())
    }
}
