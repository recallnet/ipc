// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};
use recall_ipld::{hamt, hamt::map::TrackedFlushResult};

use crate::credit::Credit;

/// A credit approval from one account to another.
#[derive(Debug, Default, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct CreditApproval {
    /// Optional credit approval limit.
    pub credit_limit: Option<Credit>,
    /// Used to limit gas fee delegation.
    pub gas_allowance_limit: Option<TokenAmount>,
    /// Optional credit approval expiry epoch.
    pub expiry: Option<ChainEpoch>,
    /// Counter for how much credit has been used via this approval.
    pub credit_used: Credit,
    /// Used to track gas fees paid for by the delegation
    pub gas_allowance_used: TokenAmount,
}

impl CreditApproval {
    /// Returns a new credit approval.
    pub fn new(
        credit_limit: Option<Credit>,
        gas_allowance_limit: Option<TokenAmount>,
        expiry: Option<ChainEpoch>,
    ) -> Self {
        Self {
            credit_limit,
            gas_allowance_limit,
            expiry,
            ..Default::default()
        }
    }

    /// Validates whether the approval has enough allowance for the credit amount.
    pub fn validate_credit_usage(&self, amount: &TokenAmount) -> Result<(), ActorError> {
        if let Some(credit_limit) = self.credit_limit.as_ref() {
            let unused = &(credit_limit - &self.credit_used);
            if unused < amount {
                return Err(ActorError::forbidden(format!(
                    "usage would exceed approval credit limit (available: {}; required: {})",
                    unused, amount
                )));
            }
        }
        Ok(())
    }

    /// Validates whether the approval has enough allowance for the gas amount.
    pub fn validate_gas_usage(&self, amount: &TokenAmount) -> Result<(), ActorError> {
        if let Some(gas_limit) = self.gas_allowance_limit.as_ref() {
            let unused = &(gas_limit - &self.gas_allowance_used);
            if unused < amount {
                return Err(ActorError::forbidden(format!(
                    "usage would exceed approval gas allowance (available: {}; required: {})",
                    unused, amount
                )));
            }
        }
        Ok(())
    }

    /// Validates whether the approval has a valid expiration.
    pub fn validate_expiration(&self, current_epoch: ChainEpoch) -> Result<(), ActorError> {
        if let Some(expiry) = self.expiry {
            if expiry <= current_epoch {
                return Err(ActorError::forbidden("approval expired".into()));
            }
        }
        Ok(())
    }
}

/// HAMT wrapper tracking [`CreditApproval`]s by account address.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct CreditApprovals {
    pub root: hamt::Root<Address, CreditApproval>,
    size: u64,
}

impl CreditApprovals {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Address, CreditApproval>::new(store, "credit_approvals")?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<'a, BS, Address, CreditApproval>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(
        &mut self,
        tracked_flush_result: TrackedFlushResult<Address, CreditApproval>,
    ) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size
    }

    pub fn len(&self) -> u64 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}
