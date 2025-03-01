// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::{Account, Credit, CreditApproval, GasAllowance};
use fendermint_actor_recall_config_shared::RecallConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount, error::ExitCode};
use log::debug;
use recall_ipld::hamt;

use crate::caller::{Caller, Delegation, DelegationOptions};
use crate::State;

/// Params for committing capacity.
#[derive(Debug)]
pub struct CommitCapacityParams {
    /// Commitment size for subnet,
    /// which may be less than caller if the data already exists in the subnet.
    pub subnet_size: u64,
    /// Commitment size for caller.
    pub caller_size: u64,
    /// Commitment cost
    pub cost: Credit,
    /// Token amount available to commitment.
    pub value: TokenAmount,
    /// Commitment chain epoch.
    pub epoch: ChainEpoch,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::state::AddBlobStateParams;
    use fendermint_actor_blobs_shared::state::{CreditApproval, SubscriptionId};
    use fendermint_actor_blobs_testing::{
        new_address, new_hash, new_metadata_hash, new_pk, setup_logs,
    };
    use fvm_ipld_blockstore::MemoryBlockstore;
    use num_traits::Zero;

    fn check_approvals_match(
        state: &State,
        store: &MemoryBlockstore,
        from: Address,
        to: Address,
        expected: CreditApproval,
    ) {
        let from_account = state.get_account(&store, from).unwrap().unwrap();
        assert_eq!(
            from_account
                .approvals_to
                .hamt(store)
                .unwrap()
                .get(&to)
                .unwrap()
                .unwrap(),
            expected
        );
        let to_account = state.get_account(&store, to).unwrap().unwrap();
        assert_eq!(
            to_account
                .approvals_from
                .hamt(store)
                .unwrap()
                .get(&from)
                .unwrap()
                .unwrap(),
            expected
        );
    }

    #[test]
    fn test_buy_credit_success() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let to = new_address();
        let amount = TokenAmount::from_whole(1);

        let res = state.buy_credit(&store, &config, to, amount.clone(), 1);
        assert!(res.is_ok());
        let account = res.unwrap();
        let credit_sold = amount.clone() * &config.token_credit_rate;
        assert_eq!(account.credit_free, credit_sold);
        assert_eq!(account.gas_allowance, amount);
        assert_eq!(state.credit_sold, credit_sold);
        let account_back = state.get_account(&store, to).unwrap().unwrap();
        assert_eq!(account, account_back);
    }

    #[test]
    fn test_buy_credit_negative_amount() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let to = new_address();
        let amount = TokenAmount::from_whole(-1);

        let res = state.buy_credit(&store, &config, to, amount, 1);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().msg(), "amount must be positive");
    }

    #[test]
    fn test_buy_credit_at_capacity() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let to = new_address();
        let amount = TokenAmount::from_whole(1);

        state.capacity_used = config.blob_capacity;
        let res = state.buy_credit(&store, &config, to, amount, 1);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            "subnet has reached storage capacity"
        );
    }

    #[test]
    fn test_approve_credit_success() {
        setup_logs();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        let config = RecallConfig::default();

        // No limit or expiry
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            DelegationOptions::default(),
            current_epoch,
        );
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.credit_limit, None);
        assert_eq!(approval.gas_allowance_limit, None);
        assert_eq!(approval.expiry, None);
        check_approvals_match(&state, &store, from, to, approval);

        // Add credit limit
        let limit = 1_000_000_000_000_000_000u64;
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            DelegationOptions {
                credit_limit: Some(Credit::from_whole(limit)),
                ..Default::default()
            },
            current_epoch,
        );
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.credit_limit, Some(Credit::from_whole(limit)));
        assert_eq!(approval.gas_allowance_limit, None);
        assert_eq!(approval.expiry, None);
        check_approvals_match(&state, &store, from, to, approval);

        // Add gas fee limit
        let limit = 1_000_000_000_000_000_000u64;
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            DelegationOptions {
                gas_fee_limit: Some(TokenAmount::from_atto(limit)),
                ..Default::default()
            },
            current_epoch,
        );
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.credit_limit, None);
        assert_eq!(
            approval.gas_allowance_limit,
            Some(TokenAmount::from_atto(limit))
        );
        assert_eq!(approval.expiry, None);
        check_approvals_match(&state, &store, from, to, approval);

        // Add ttl
        let ttl = ChainEpoch::from(config.blob_min_ttl);
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            DelegationOptions {
                credit_limit: Some(Credit::from_whole(limit)),
                ttl: Some(ttl),
                ..Default::default()
            },
            current_epoch,
        );
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.credit_limit, Some(Credit::from_whole(limit)));
        assert_eq!(approval.gas_allowance_limit, None);
        assert_eq!(approval.expiry, Some(ttl + current_epoch));
        check_approvals_match(&state, &store, from, to, approval);
    }

    #[test]
    fn test_approve_credit_invalid_ttl() {
        setup_logs();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        let config = RecallConfig::default();
        let ttl = ChainEpoch::from(config.blob_min_ttl - 1);
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            DelegationOptions {
                ttl: Some(ttl),
                ..Default::default()
            },
            current_epoch,
        );
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!("minimum approval TTL is {}", config.blob_min_ttl)
        );
    }

    #[test]
    fn test_approve_credit_overflowing_ttl() {
        setup_logs();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        let config = RecallConfig::default();

        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            DelegationOptions {
                ttl: Some(ChainEpoch::MAX),
                ..Default::default()
            },
            current_epoch,
        );
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.expiry, Some(i64::MAX));
    }

    #[test]
    fn test_approve_credit_insufficient_credit() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, &config, from, amount.clone(), current_epoch)
            .unwrap();
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            DelegationOptions::default(),
            current_epoch,
        );
        assert!(res.is_ok());

        // Add a blob
        let (hash, size) = new_hash(1024);
        let res = state.add_blob(
            &store,
            &config,
            to,
            Some(from),
            AddBlobStateParams {
                hash,
                metadata_hash: new_metadata_hash(),
                id: SubscriptionId::default(),
                size,
                ttl: None,
                source: new_pk(),
                epoch: current_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());

        // Check approval
        let account = state.get_account(&store, from).unwrap().unwrap();
        let approval = account
            .approvals_to
            .hamt(&store)
            .unwrap()
            .get(&to)
            .unwrap()
            .unwrap();
        assert_eq!(account.credit_committed, approval.credit_used);

        // Try to update approval with a limit below what's already been committed
        let limit = 1_000u64;
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            DelegationOptions {
                credit_limit: Some(Credit::from_whole(limit)),
                ..Default::default()
            },
            current_epoch,
        );
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!(
                "limit cannot be less than amount of already used credits ({})",
                approval.credit_used
            )
        );
    }

    #[test]
    fn test_revoke_credit_success() {
        setup_logs();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        let config = RecallConfig::default();
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            DelegationOptions::default(),
            current_epoch,
        );
        assert!(res.is_ok());

        // Check the account approvals
        let from_account = state.get_account(&store, from).unwrap().unwrap();
        assert_eq!(from_account.approvals_to.len(), 1);
        let to_account = state.get_account(&store, to).unwrap().unwrap();
        assert_eq!(to_account.approvals_from.len(), 1);

        // Remove the approval
        let res = state.revoke_credit(&store, from, to);
        assert!(res.is_ok());
        let from_account = state.get_account(&store, from).unwrap().unwrap();
        assert_eq!(from_account.approvals_to.len(), 0);
        let to_account = state.get_account(&store, to).unwrap().unwrap();
        assert_eq!(to_account.approvals_from.len(), 0);
    }

    #[test]
    fn test_revoke_credit_account_not_found() {
        setup_logs();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let from = new_address();
        let to = new_address();

        let res = state.revoke_credit(&store, from, to);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!("{} not found in accounts", to)
        );
    }
}
