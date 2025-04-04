// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::credit::{Credit, CreditApproval, GasAllowance};
use fendermint_actor_recall_config_shared::RecallConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount, error::ExitCode};
use log::debug;
use recall_ipld::hamt;

use super::CommitCapacityParams;
use crate::{
    caller::{Caller, Delegation, DelegationOptions},
    state::accounts::Account,
    State,
};

/// Returns an error if the amount is negative.
pub fn ensure_positive_amount(amount: &TokenAmount) -> anyhow::Result<(), ActorError> {
    if amount.is_negative() {
        return Err(ActorError::illegal_argument(
            "amount must be positive".into(),
        ));
    }
    Ok(())
}

impl State {
    /// Buys credit for an account.
    /// Flushes state to the blockstore.
    pub fn buy_credit<BS: Blockstore>(
        &mut self,
        store: &BS,
        config: &RecallConfig,
        to: Address,
        value: TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<Account, ActorError> {
        self.ensure_capacity(config.blob_capacity)?;
        ensure_positive_amount(&value)?;

        let mut accounts = self.accounts.hamt(store)?;
        let mut caller = Caller::load_or_create(
            store,
            &accounts,
            to,
            None,
            current_epoch,
            config.blob_default_ttl,
        )?;

        let amount: Credit = value.clone() * &config.token_credit_rate;
        caller.add_allowances(&amount, &value);

        // Update global state
        self.credit_sold += &amount;

        // Save caller
        self.save_caller(&mut caller, &mut accounts)?;

        Ok(caller.subscriber().clone())
    }

    /// Sets the default credit and gas fee sponsor for an account.
    /// Flushes state to the blockstore.
    pub fn set_account_sponsor<BS: Blockstore>(
        &mut self,
        config: &RecallConfig,
        store: &BS,
        from: Address,
        sponsor: Option<Address>,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        let mut accounts = self.accounts.hamt(store)?;
        let mut caller = Caller::load_or_create(
            store,
            &accounts,
            from,
            None,
            current_epoch,
            config.blob_default_ttl,
        )?;

        caller.set_default_sponsor(sponsor);

        // Save caller
        self.save_caller(&mut caller, &mut accounts)
    }

    /// Updates (adds/removes) gas allowance for an account.
    /// Flushes state to the blockstore.
    pub fn update_gas_allowance<BS: Blockstore>(
        &mut self,
        store: &BS,
        from: Address,
        sponsor: Option<Address>,
        add_amount: TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        let mut accounts = self.accounts.hamt(store)?;
        let mut caller = Caller::load(store, &accounts, from, sponsor)?;

        caller.update_gas_allowance(&add_amount, current_epoch)?;

        // Save caller
        self.save_caller(&mut caller, &mut accounts)
    }

    /// Approves credit and gas allowance spend from one account to another.
    /// Flushes state to the blockstore.
    pub fn approve_credit<BS: Blockstore>(
        &mut self,
        config: &RecallConfig,
        store: &BS,
        from: Address,
        to: Address,
        options: DelegationOptions,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<CreditApproval, ActorError> {
        let mut accounts = self.accounts.hamt(store)?;
        let mut delegation = Delegation::update_or_create(
            store,
            config,
            &accounts,
            from,
            to,
            options,
            current_epoch,
        )?;

        // Save delegation
        self.save_delegation(&mut delegation, &mut accounts)?;

        Ok(delegation.approval().clone())
    }

    /// Revokes credit and gas allowance spend from one account to another.
    /// Flushes state to the blockstore.
    pub fn revoke_credit<BS: Blockstore>(
        &mut self,
        store: &BS,
        from: Address,
        to: Address,
    ) -> anyhow::Result<(), ActorError> {
        let mut accounts = self.accounts.hamt(store)?;
        let mut caller = Caller::load(store, &accounts, to, Some(from))?;

        caller.cancel_delegation(&mut accounts)?;

        // Save caller
        self.save_caller(&mut caller, &mut accounts)
    }

    /// Returns a [`CreditApproval`] from the given address to the given address
    /// or [`None`] if no approval exists.
    pub fn get_credit_approval<BS: Blockstore>(
        &self,
        store: &BS,
        from: Address,
        to: Address,
    ) -> anyhow::Result<Option<CreditApproval>, ActorError> {
        let accounts = self.accounts.hamt(store)?;
        let caller = Caller::load(store, &accounts, to, Some(from))?;
        Ok(caller.delegate_approval().cloned())
    }

    /// Returns the gas allowance for the given address, including an amount from a default sponsor.
    /// An error returned from this method would be fatal, as it's called from the FVM executor.
    pub fn get_gas_allowance<BS: Blockstore>(
        &self,
        store: &BS,
        from: Address,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<GasAllowance, ActorError> {
        let accounts = self.accounts.hamt(store)?;
        let allowance = Caller::load_with_default_sponsor(store, &accounts, from)
            .map(|caller| caller.gas_allowance(current_epoch))
            .unwrap_or_default();
        Ok(allowance)
    }

    /// Debits credit from the caller.
    /// Does NOT flush the state to the blockstore.
    pub(crate) fn debit_caller<BS: Blockstore>(
        &mut self,
        caller: &mut Caller<BS>,
        amount: &Credit,
        current_epoch: ChainEpoch,
    ) {
        caller.debit_credit(amount, current_epoch);

        // Update global state
        self.credit_debited += amount;
        self.credit_committed -= amount;
    }

    /// Refunds credit to the caller.
    /// Does NOT flush the state to the blockstore.
    pub(crate) fn refund_caller<BS: Blockstore>(
        &mut self,
        caller: &mut Caller<BS>,
        amount: &Credit,
        correction: &Credit,
    ) {
        caller.refund_credit(amount, correction);

        // Update global state
        self.credit_debited -= amount;
        self.credit_committed += correction;
    }

    /// Commits new capacity for the caller.
    /// The caller may pay for capacity with free credit or token value.
    /// Does NOT flush the state to the blockstore.
    pub(crate) fn commit_capacity_for_caller<BS: Blockstore>(
        &mut self,
        caller: &mut Caller<BS>,
        config: &RecallConfig,
        params: CommitCapacityParams,
    ) -> anyhow::Result<TokenAmount, ActorError> {
        self.ensure_capacity(config.blob_capacity)?;
        ensure_positive_amount(&params.cost)?;
        ensure_positive_amount(&params.value)?;

        let value_remaining =
            match caller.commit_capacity(params.caller_size, &params.cost, params.epoch) {
                Ok(()) => Ok(params.value.clone()),
                Err(e) => {
                    // Buy credit to cover the amount
                    if e.exit_code() == ExitCode::USR_INSUFFICIENT_FUNDS && !params.value.is_zero()
                    {
                        if caller.is_delegate() {
                            return Err(ActorError::forbidden(
                                "cannot auto-buy credits for a sponsor".into(),
                            ));
                        }

                        let remainder: Credit = &params.cost - &caller.subscriber().credit_free;
                        let value_required = &remainder / &config.token_credit_rate;
                        let value_remaining = &params.value - &value_required;
                        if value_remaining.is_negative() {
                            return Err(ActorError::insufficient_funds(format!(
                                "insufficient value (received: {}; required: {})",
                                params.value, value_required
                            )));
                        }
                        caller.add_allowances(&remainder, &value_required);

                        // Update global state
                        self.credit_sold += &remainder;

                        // Try again
                        caller.commit_capacity(params.caller_size, &params.cost, params.epoch)?;
                        Ok(value_remaining)
                    } else {
                        Err(e)
                    }
                }
            }?;

        // Update global state
        self.capacity_used += params.subnet_size;
        self.credit_committed += &params.cost;

        debug!("used {} bytes from subnet", params.subnet_size);

        Ok(value_remaining)
    }

    /// Uncommits capacity for the caller.
    /// Does NOT flush the state to the blockstore.
    pub(crate) fn uncommit_capacity_for_caller<BS: Blockstore>(
        &mut self,
        caller: &mut Caller<BS>,
        subnet_size: u64,
        caller_size: u64,
        cost: &Credit,
    ) {
        caller.uncommit_capacity(caller_size, cost);

        // Update global state
        self.capacity_used -= subnet_size;
        self.credit_committed -= cost;

        debug!("released {} bytes to subnet", subnet_size);
    }

    /// Returns committed credit to the caller.
    /// Does NOT flush the state to the blockstore.
    pub(crate) fn return_committed_credit_for_caller<BS: Blockstore>(
        &mut self,
        caller: &mut Caller<BS>,
        amount: &Credit,
    ) {
        caller.return_committed_credit(amount);

        // Update global state
        self.credit_debited -= amount;
        self.credit_committed += amount;
    }

    /// Save the caller state to the accounts HAMT.
    pub(crate) fn save_caller<'a, BS: Blockstore>(
        &mut self,
        caller: &mut Caller<'a, BS>,
        accounts: &mut hamt::map::Hamt<'a, &'a BS, Address, Account>,
    ) -> anyhow::Result<(), ActorError> {
        caller.save(accounts)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);
        Ok(())
    }

    /// Save the delegation state to the accounts HAMT.
    pub(crate) fn save_delegation<'a, BS: Blockstore>(
        &mut self,
        delegation: &mut Delegation<'a, &'a BS>,
        accounts: &mut hamt::map::Hamt<'a, &'a BS, Address, Account>,
    ) -> anyhow::Result<(), ActorError> {
        delegation.save(accounts)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);
        Ok(())
    }
}
