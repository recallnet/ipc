// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::{Account, Credit, CreditApproval, GasAllowance};
use fendermint_actor_recall_config_shared::RecallConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::error::ExitCode;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};
use log::debug;
use num_traits::Zero;

use crate::caller::Caller;
use crate::State;

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

        // Get or create a new account
        let mut accounts = self.accounts.hamt(store)?;
        let mut caller = Caller::load_or_create(
            store,
            &accounts,
            to,
            to,
            current_epoch,
            config.blob_default_ttl,
        )?;

        let amount: Credit = value.clone() * &config.token_credit_rate;
        caller.add_credit(&amount);
        caller.add_gas_allowance(&value);
        caller.save(&mut accounts)?;

        // Update global state
        self.credit_sold += &amount;

        // Save account
        self.accounts.save_tracked(accounts.flush_tracked()?);

        debug!("sold {} credits to {}", amount, caller.subscriber_address());
        Ok(caller.subscriber().clone())
    }

    // Note: does not save state
    pub fn use_or_buy_credit_for_caller<'a, BS: Blockstore>(
        &mut self,
        caller: &mut Caller<'a, BS>,
        config: &RecallConfig,
        amount: &Credit,
        value: &TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<TokenAmount, ActorError> {
        self.ensure_capacity(config.blob_capacity)?;
        ensure_positive_amount(amount)?;
        ensure_positive_amount(value)?;

        match caller.validate_credit_usage(amount, current_epoch) {
            Ok(()) => Ok(value.clone()),
            Err(e) => {
                // Buy credit to cover the amount
                if e.exit_code() == ExitCode::USR_INSUFFICIENT_FUNDS && !value.is_zero() {
                    if caller.is_delegate() {
                        return Err(ActorError::forbidden(
                            "cannot auto-buy credits for a sponsor".into(),
                        ));
                    }

                    let amount: Credit = amount - &caller.subscriber().credit_free;
                    let value_required = &amount / &config.token_credit_rate;
                    let value_remaining = value - &value_required;
                    if value_remaining.is_negative() {
                        return Err(ActorError::insufficient_funds(format!(
                            "insufficient value (received: {}; required: {})",
                            value, value_required
                        )));
                    }
                    caller.add_credit(&amount);
                    caller.add_gas_allowance(&value_required);

                    // Update global state
                    self.credit_sold += &amount;

                    debug!("sold {} credits to {}", amount, caller.subscriber_address());
                    Ok(value_remaining)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn update_gas_allowance<BS: Blockstore>(
        &mut self,
        store: &BS,
        from: Address,
        sponsor: Option<Address>,
        add_amount: TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        let subscriber = sponsor.unwrap_or(from);
        let mut accounts = self.accounts.hamt(store)?;
        let mut caller = Caller::load_or_err(store, &accounts, from, subscriber)?;

        // Add / deduct gas allowance
        caller.deduct_gas_allowance(&add_amount, current_epoch)?;
        caller.save(&mut accounts)?;

        // Save accounts
        self.accounts.save_tracked(accounts.flush_tracked()?);

        if add_amount.is_positive() {
            debug!("refunded {} atto to {}", add_amount.atto(), subscriber);
        } else {
            debug!(
                "debited {} atto from {}",
                add_amount.atto().magnitude(),
                subscriber
            );
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn approve_credit<BS: Blockstore>(
        &mut self,
        config: &RecallConfig,
        store: &BS,
        from: Address,
        to: Address,
        current_epoch: ChainEpoch,
        credit_limit: Option<Credit>,
        gas_fee_limit: Option<TokenAmount>,
        ttl: Option<ChainEpoch>,
    ) -> anyhow::Result<CreditApproval, ActorError> {
        let credit_limit = credit_limit.map(Credit::from);
        let gas_fee_limit = gas_fee_limit.map(TokenAmount::from);
        if let Some(ttl) = ttl {
            if ttl < config.blob_min_ttl {
                return Err(ActorError::illegal_argument(format!(
                    "minimum approval TTL is {}",
                    config.blob_min_ttl
                )));
            }
        }
        let expiry = ttl.map(|t| i64::saturating_add(t, current_epoch));
        // Get or create a new account
        let mut accounts = self.accounts.hamt(store)?;
        let mut from_account = accounts.get_or_create(&from, || {
            Account::new(store, current_epoch, config.blob_default_ttl)
        })?;
        let mut to_account = accounts.get_or_create(&to, || {
            Account::new(store, current_epoch, config.blob_default_ttl)
        })?;
        // Get or add a new approval
        let approval = CreditApproval {
            credit_limit: credit_limit.clone(),
            gas_allowance_limit: gas_fee_limit.clone(),
            expiry,
            credit_used: Credit::zero(),
            gas_allowance_used: TokenAmount::zero(),
        };
        let mut from_approval = from_account
            .approvals_to
            .hamt(store)?
            .get_or_create(&to, || Ok(approval.clone()))?;
        let mut to_approval = to_account
            .approvals_from
            .hamt(store)?
            .get_or_create(&from, || Ok(approval))?;
        if from_approval != to_approval {
            return Err(ActorError::illegal_state(format!(
                "approval in 'from' account ({}) doesn't match approval in 'to' account ({})",
                from, to,
            )));
        }

        // Validate approval changes
        if let Some(limit) = credit_limit.clone() {
            if from_approval.credit_used > limit {
                return Err(ActorError::illegal_argument(format!(
                    "limit cannot be less than amount of already used credits ({})",
                    from_approval.credit_used
                )));
            }
        }

        if let Some(limit) = gas_fee_limit.clone() {
            if from_approval.gas_allowance_used > limit {
                return Err(ActorError::illegal_argument(format!(
                    "limit cannot be less than amount of already used gas fees ({})",
                    from_approval.gas_allowance_used
                )));
            }
        }
        from_approval.credit_limit = credit_limit.clone();
        from_approval.gas_allowance_limit = gas_fee_limit.clone();
        from_approval.expiry = expiry;
        to_approval.credit_limit = credit_limit;
        to_approval.gas_allowance_limit = gas_fee_limit;
        to_approval.expiry = expiry;

        from_account.approvals_to.save_tracked(
            from_account
                .approvals_to
                .hamt(store)?
                .set_and_flush_tracked(&to, from_approval.clone())?,
        );

        to_account.approvals_from.save_tracked(
            to_account
                .approvals_from
                .hamt(store)?
                .set_and_flush_tracked(&from, to_approval)?,
        );

        // Save accounts
        let from_approval = from_approval.clone();
        accounts.set(&from, from_account)?;
        accounts.set(&to, to_account)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);

        debug!(
            "approved credits from {} to {} (credit limit: {:?}; gas fee limit: {:?}, expiry: {:?}",
            from,
            to,
            from_approval.credit_limit,
            from_approval.gas_allowance_limit,
            from_approval.expiry
        );
        Ok(from_approval)
    }

    /// Revokes credit from one account to another.
    pub fn revoke_credit<BS: Blockstore>(
        &mut self,
        store: &BS,
        from: Address,
        to: Address,
    ) -> anyhow::Result<(), ActorError> {
        // Get the account
        let mut accounts = self.accounts.hamt(store)?;
        let mut from_account = accounts.get_or_err(&from)?;
        let (tracked_result, approval) = from_account
            .approvals_to
            .hamt(store)?
            .delete_and_flush_tracked(&to)?;
        if approval.is_none() {
            return Err(ActorError::not_found(format!(
                "approval from {} to {} not found",
                from, to
            )));
        }
        from_account.approvals_to.save_tracked(tracked_result);

        let mut to_account = accounts.get_or_err(&to)?;
        let (tracked_result, approval) = to_account
            .approvals_from
            .hamt(store)?
            .delete_and_flush_tracked(&from)?;
        if approval.is_none() {
            return Err(ActorError::not_found(format!(
                "approval from {} to {} not found in 'to' account",
                from, to
            )));
        }
        to_account.approvals_from.save_tracked(tracked_result);

        // Save accounts
        accounts.set(&from, from_account)?;
        accounts.set(&to, to_account)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);

        debug!("revoked credits from {} to {}", from, to);
        Ok(())
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
        let account = accounts
            .get(&from)?
            .ok_or(ActorError::not_found(format!("account {} not found", from)))?;
        let approval = account.approvals_to.hamt(store)?.get(&to)?;
        Ok(approval)
    }

    /// Returns the gas allowance for the given address, including an amount from a default sponsor.
    /// An error returned from this method would be fatal, as it's called from the FVM executor.
    pub fn get_gas_allowance<BS: Blockstore>(
        &self,
        store: &BS,
        from: Address,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<GasAllowance, ActorError> {
        // Get the account or return default allowance
        let accounts = self.accounts.hamt(store)?;
        let account = match accounts.get(&from)? {
            None => return Ok(GasAllowance::default()),
            Some(account) => account,
        };
        let mut allowance = GasAllowance {
            amount: account.gas_allowance.clone(),
            ..Default::default()
        };
        if let Some(credit_sponsor) = account.credit_sponsor {
            let sponsor = match accounts.get(&credit_sponsor)? {
                None => return Ok(allowance),
                Some(account) => account,
            };
            let sponsored = sponsor
                .approvals_to
                .hamt(store)?
                .get(&from)?
                .and_then(|approval| {
                    let expiry_valid = approval
                        .expiry
                        .map_or(true, |expiry| expiry > current_epoch);
                    if !expiry_valid {
                        return None;
                    }
                    let gas_allowance = sponsor.gas_allowance.clone();
                    let used = approval.gas_allowance_used.clone();
                    let amount = approval
                        .gas_allowance_limit
                        .clone()
                        .map_or(gas_allowance.clone(), |limit| {
                            (limit - used).min(gas_allowance)
                        });
                    Some(amount)
                })
                .unwrap_or(TokenAmount::zero());
            allowance.sponsor = Some(credit_sponsor);
            allowance.sponsored_amount = sponsored;
        } else {
            return Ok(allowance);
        }
        Ok(allowance)
    }

    pub fn set_account_sponsor<BS: Blockstore>(
        &mut self,
        config: &RecallConfig,
        store: &BS,
        from: Address,
        sponsor: Option<Address>,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        // Get or create a new account
        let mut accounts = self.accounts.hamt(store)?;
        let mut account = accounts.get_or_create(&from, || {
            Account::new(store, current_epoch, config.blob_default_ttl)
        })?;
        account.credit_sponsor = sponsor;
        // Save account
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&from, account)?);

        debug!("set credit sponsor for {} to {:?}", from, sponsor);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fendermint_actor_blobs_shared::state::{CreditApproval, SubscriptionId};
    use fendermint_actor_blobs_testing::{
        new_address, new_hash, new_metadata_hash, new_pk, setup_logs,
    };
    use fvm_ipld_blockstore::MemoryBlockstore;

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
        let recipient = new_address();
        let amount = TokenAmount::from_whole(-1);

        let res = state.buy_credit(&store, &config, recipient, amount, 1);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().msg(), "token amount must be positive");
    }

    #[test]
    fn test_buy_credit_at_capacity() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let recipient = new_address();
        let amount = TokenAmount::from_whole(1);

        state.capacity_used = config.blob_capacity;
        let res = state.buy_credit(&store, &config, recipient, amount, 1);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            "credits not available (subnet has reached storage capacity)"
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
        let res = state.approve_credit(&config, &store, from, to, current_epoch, None, None, None);
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
            current_epoch,
            Some(Credit::from_whole(limit)),
            None,
            None,
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
            current_epoch,
            None,
            Some(TokenAmount::from_atto(limit)),
            None,
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
            current_epoch,
            Some(Credit::from_whole(limit)),
            None,
            Some(ttl),
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
            current_epoch,
            None,
            None,
            Some(ttl),
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
            current_epoch,
            None,
            None,
            Some(ChainEpoch::MAX),
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
        let res = state.approve_credit(&config, &store, from, to, current_epoch, None, None, None);
        assert!(res.is_ok());

        // Add a blob
        let (hash, size) = new_hash(1024);
        let res = state.add_blob(
            &config,
            &store,
            to,
            from,
            current_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::default(),
            size,
            None,
            new_pk(),
            TokenAmount::zero(),
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
            current_epoch,
            Some(Credit::from_whole(limit)),
            None,
            None,
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
        let res = state.approve_credit(&config, &store, from, to, current_epoch, None, None, None);
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
            format!("{} not found in accounts", from)
        );
    }
}
