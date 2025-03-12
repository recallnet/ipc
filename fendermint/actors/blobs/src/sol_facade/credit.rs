// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Error;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use recall_actor_sdk::{token_to_biguint, TryIntoEVMEvent};
use recall_sol_facade::credit as sol;
use recall_sol_facade::primitives::U256;
use recall_sol_facade::types::{BigUintWrapper, H160};

pub struct CreditPurchased {
    from: Address,
    amount: TokenAmount,
}
impl CreditPurchased {
    pub fn new(from: Address, amount: TokenAmount) -> Self {
        Self { from, amount }
    }
}
impl TryIntoEVMEvent for CreditPurchased {
    type Target = sol::Event;
    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        let from: H160 = self.from.try_into()?;
        let amount = token_to_biguint(Some(self.amount));
        Ok(sol::Event::CreditPurchased(sol::CreditPurchased {
            from: from.into(),
            amount: BigUintWrapper(amount).into(),
        }))
    }
}

pub struct CreditApproved {
    pub from: Address,
    pub to: Address,
    pub credit_limit: Option<TokenAmount>,
    pub gas_fee_limit: Option<TokenAmount>,
    pub expiry: Option<ChainEpoch>,
}
impl TryIntoEVMEvent for CreditApproved {
    type Target = sol::Event;
    fn try_into_evm_event(self) -> Result<sol::Event, Error> {
        let from: H160 = self.from.try_into()?;
        let to: H160 = self.to.try_into()?;
        let credit_limit = token_to_biguint(self.credit_limit);
        let gas_fee_limit = token_to_biguint(self.gas_fee_limit);
        Ok(sol::Event::CreditApproved(sol::CreditApproved {
            from: from.into(),
            to: to.into(),
            creditLimit: BigUintWrapper(credit_limit).into(),
            gasFeeLimit: BigUintWrapper(gas_fee_limit).into(),
            expiry: U256::from(self.expiry.unwrap_or_default()),
        }))
    }
}

pub struct CreditRevoked {
    pub from: Address,
    pub to: Address,
}
impl CreditRevoked {
    pub fn new(from: Address, to: Address) -> Self {
        Self { from, to }
    }
}
impl TryIntoEVMEvent for CreditRevoked {
    type Target = sol::Event;
    fn try_into_evm_event(self) -> Result<sol::Event, Error> {
        let from: H160 = self.from.try_into()?;
        let to: H160 = self.to.try_into()?;
        Ok(sol::Event::CreditRevoked(sol::CreditRevoked {
            from: from.into(),
            to: to.into(),
        }))
    }
}

pub struct CreditDebited {
    pub amount: TokenAmount,
    pub num_accounts: u64,
    pub more_accounts: bool,
}
impl TryIntoEVMEvent for CreditDebited {
    type Target = sol::Event;
    fn try_into_evm_event(self) -> Result<sol::Event, Error> {
        let amount = token_to_biguint(Some(self.amount));
        Ok(sol::Event::CreditDebited(sol::CreditDebited {
            amount: BigUintWrapper(amount).into(),
            numAccounts: U256::from(self.num_accounts),
            moreAccounts: self.more_accounts,
        }))
    }
}
