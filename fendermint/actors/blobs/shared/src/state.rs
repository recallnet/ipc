// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;
use std::fmt;
use std::ops::{Div, Mul};

use fil_actors_runtime::ActorError;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use recall_ipld::hamt::MapKey;
use serde::{Deserialize, Serialize};

/// Credit is counted the same way as tokens.
/// The smallest indivisible unit is 1 atto, and 1 credit = 1e18 atto credits.
pub type Credit = TokenAmount;

/// TokenCreditRate determines how much atto credits can be bought by a certain amount of RECALL.
#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct TokenCreditRate {
    rate: BigInt,
}

impl TokenCreditRate {
    pub const RATIO: u128 = 10u128.pow(18);

    pub fn from(rate: impl Into<BigInt>) -> Self {
        Self { rate: rate.into() }
    }

    pub fn rate(&self) -> &BigInt {
        &self.rate
    }
}

impl fmt::Display for TokenCreditRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rate)
    }
}

impl Mul<&TokenCreditRate> for TokenAmount {
    type Output = Credit;

    fn mul(self, rate: &TokenCreditRate) -> Self::Output {
        (self * &rate.rate).div_floor(TokenCreditRate::RATIO)
    }
}

impl Div<&TokenCreditRate> for &Credit {
    type Output = TokenAmount;

    fn div(self, rate: &TokenCreditRate) -> Self::Output {
        #[allow(clippy::suspicious_arithmetic_impl)]
        (self * TokenCreditRate::RATIO).div_floor(rate.rate.clone())
    }
}

impl PartialOrd for TokenCreditRate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TokenCreditRate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rate.cmp(&other.rate)
    }
}

/// The stored representation of a credit account.
#[derive(Clone, Debug, Default, PartialEq, Serialize_tuple, Deserialize_tuple)]
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
    pub approvals_to: HashMap<String, CreditApproval>,
    /// Credit approvals to this account from other accounts, keyed by sender.
    pub approvals_from: HashMap<String, CreditApproval>,
    /// The maximum allowed TTL for actor's blobs.
    pub max_ttl: ChainEpoch,
    /// The total token value an account has used to buy credits.
    pub gas_allowance: TokenAmount,
}

impl Account {
    pub fn new(current_epoch: ChainEpoch, max_ttl: ChainEpoch) -> Self {
        Self {
            last_debit_epoch: current_epoch,
            max_ttl,
            ..Default::default()
        }
    }
}

/// A credit approval from one account to another.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct CreditApproval {
    /// Optional credit approval limit.
    pub credit_limit: Option<Credit>,
    /// Used to limit gas fee delegation.
    pub gas_fee_limit: Option<TokenAmount>,
    /// Optional credit approval expiry epoch.
    pub expiry: Option<ChainEpoch>,
    /// Counter for how much credit has been used via this approval.
    pub credit_used: Credit,
    /// Used to track gas fees paid for by the delegation
    pub gas_fee_used: TokenAmount,
}

/// Gas allowance for an account.
#[derive(Debug, Default, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct GasAllowance {
    /// The amount from the account.
    pub amount: TokenAmount,
    /// The account's default sponsor.
    pub sponsor: Option<Address>,
    /// The amount from the account's default sponsor.
    pub sponsored_amount: TokenAmount,
}

impl GasAllowance {
    /// Returns the total allowance from self and default sponsor.
    pub fn total(&self) -> TokenAmount {
        &self.amount + &self.sponsored_amount
    }
}

/// Blob blake3 hash.
#[derive(
    Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Hash(pub [u8; 32]);

impl TryInto<Hash> for &[u8] {
    type Error = String;

    fn try_into(self) -> Result<Hash, Self::Error> {
        if self.len() == 32 {
            let mut array = [0u8; 32];
            array.copy_from_slice(self);
            Ok(Hash(array))
        } else {
            Err("hash slice must be exactly 32 bytes".into())
        }
    }
}

impl MapKey for Hash {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        b.try_into()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.0.to_vec())
    }
}

/// Source https://github.com/n0-computer/iroh/blob/main/iroh-base/src/hash.rs
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // the result will be 52 bytes
        let mut res = [b'b'; 52];
        // write the encoded bytes
        data_encoding::BASE32_NOPAD.encode_mut(self.0.as_slice(), &mut res);
        // convert to string, this is guaranteed to succeed
        let t = std::str::from_utf8_mut(res.as_mut()).unwrap();
        // hack since data_encoding doesn't have BASE32LOWER_NOPAD as a const
        t.make_ascii_lowercase();
        // write the str, no allocations
        f.write_str(t)
    }
}

impl TryFrom<&str> for Hash {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut res = [0u8; 32];
        data_encoding::BASE32_NOPAD
            .decode_mut(value.as_bytes(), &mut res)
            .map_err(|_| anyhow::anyhow!("invalid hash"))?;
        Ok(Self(res))
    }
}

impl From<u64> for Hash {
    fn from(value: u64) -> Self {
        let mut padded = [0u8; 32];
        padded[24..].copy_from_slice(&value.to_be_bytes());
        Self(padded)
    }
}

/// Iroh node public key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublicKey(pub [u8; 32]);

/// The stored representation of a blob.
#[derive(Clone, PartialEq, Debug, Default, Serialize_tuple, Deserialize_tuple)]
pub struct Blob {
    /// The size of the content.
    pub size: u64,
    /// Blob metadata that contains information for blob recovery.
    pub metadata_hash: Hash,
    /// Active subscribers (accounts) that are paying for the blob.
    pub subscribers: HashMap<String, SubscriptionGroup>,
    /// Blob status.
    pub status: BlobStatus,
}

/// An object used to determine what [`Account`](s) are accountable for a blob, and for how long.
/// Subscriptions allow us to distribute the cost of a blob across multiple accounts that
/// have added the same blob.   
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Subscription {
    /// Added block.
    pub added: ChainEpoch,
    /// Expiry block.
    pub expiry: ChainEpoch,
    /// Source Iroh node ID used for ingestion.
    /// This might be unique to each instance of the same blob.
    /// It's included here for record keeping.
    pub source: PublicKey,
    /// The delegate origin that may have created the subscription via a credit approval.
    pub delegate: Option<Address>,
    /// Whether the subscription failed due to an issue resolving the target blob.
    pub failed: bool,
}

/// User-defined identifier used to differentiate blob subscriptions for the same subscriber.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId {
    inner: String,
}

impl SubscriptionId {
    pub const MAX_LEN: usize = 64;

    pub fn new(value: &str) -> Result<Self, ActorError> {
        if value.len() > Self::MAX_LEN {
            return Err(ActorError::illegal_argument(format!(
                "subscription ID length is {} but must not exceed the maximum of {} characters",
                value.len(),
                Self::MAX_LEN
            )));
        }
        Ok(Self {
            inner: value.to_string(),
        })
    }
}

impl TryFrom<String> for SubscriptionId {
    type Error = ActorError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.inner.is_empty() {
            write!(f, "default")
        } else {
            write!(f, "{}", self.inner)
        }
    }
}

/// A group of subscriptions for the same subscriber.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubscriptionGroup {
    /// Subscription group keys.
    pub subscriptions: HashMap<String, Subscription>,
}

impl SubscriptionGroup {
    /// Returns the current group max expiry and the group max expiry after adding the provided ID
    /// and new value.
    pub fn max_expiries(
        &self,
        target_id: &SubscriptionId,
        new_value: Option<ChainEpoch>,
    ) -> (Option<ChainEpoch>, Option<ChainEpoch>) {
        let mut max = None;
        let mut new_max = None;
        for (id, sub) in self.subscriptions.iter() {
            if sub.failed {
                continue;
            }
            if sub.expiry > max.unwrap_or(0) {
                max = Some(sub.expiry);
            }
            let new_value = if *id == target_id.to_string() {
                new_value.unwrap_or_default()
            } else {
                sub.expiry
            };
            if new_value > new_max.unwrap_or(0) {
                new_max = Some(new_value);
            }
        }
        // Target ID may not be in the current group
        if let Some(new_value) = new_value {
            if new_value > new_max.unwrap_or(0) {
                new_max = Some(new_value);
            }
        }
        (max, new_max)
    }

    /// Returns whether the provided ID corresponds to a subscription that has the minimum
    /// added epoch and the next minimum added epoch in the group.
    pub fn is_min_added(
        &self,
        trim_id: &SubscriptionId,
    ) -> anyhow::Result<(bool, Option<ChainEpoch>), ActorError> {
        let tid = trim_id.to_string();
        let trim = self
            .subscriptions
            .get(&tid)
            .ok_or(ActorError::not_found(format!(
                "subscription id {} not found",
                trim_id
            )))?;
        let mut next_min = None;
        for (id, sub) in self.subscriptions.iter() {
            if sub.failed || *id == tid {
                continue;
            }
            if sub.added < trim.added {
                return Ok((false, None));
            }
            if sub.added < next_min.unwrap_or(ChainEpoch::MAX) {
                next_min = Some(sub.added);
            }
        }
        Ok((true, next_min))
    }
}

/// The status of a blob.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlobStatus {
    /// Blob is added but not resolving.
    #[default]
    Added,
    /// Blob is pending resolve.
    Pending,
    /// Blob was successfully resolved.
    Resolved,
    /// Blob resolution failed.
    Failed,
}

impl fmt::Display for BlobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlobStatus::Added => write!(f, "added"),
            BlobStatus::Pending => write!(f, "pending"),
            BlobStatus::Resolved => write!(f, "resolved"),
            BlobStatus::Failed => write!(f, "failed"),
        }
    }
}

/// The TTL status of an account.
/// This controls the max TTL that the user is allowed to set on their blobs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum TtlStatus {
    // Default TTL.
    #[default]
    Default,
    /// Reduced TTL.
    Reduced,
    /// Extended TTL.
    Extended,
}

impl TtlStatus {
    /// Returns the max allowed TTL.
    pub fn get_max_ttl(&self, default_max_ttl: ChainEpoch) -> ChainEpoch {
        match self {
            TtlStatus::Default => default_max_ttl,
            TtlStatus::Reduced => 0,
            TtlStatus::Extended => ChainEpoch::MAX,
        }
    }
}

impl fmt::Display for TtlStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TtlStatus::Default => write!(f, "default"),
            TtlStatus::Reduced => write!(f, "reduced"),
            TtlStatus::Extended => write!(f, "extended"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_id_length() {
        let id_str = |len: usize| "a".repeat(len);
        let id = SubscriptionId::new(&id_str(SubscriptionId::MAX_LEN)).unwrap();
        assert_eq!(id.inner, id_str(SubscriptionId::MAX_LEN));

        let id = SubscriptionId::new(&id_str(SubscriptionId::MAX_LEN + 1));
        assert!(id.is_err());
    }

    #[test]
    fn test_token_credit_rate() {
        struct TestCase {
            tokens: TokenAmount,
            rate: TokenCreditRate,
            expected: &'static str,
            description: &'static str,
        }

        let test_cases = vec![
            TestCase {
                tokens: TokenAmount::from_whole(1),
                rate: TokenCreditRate::from(BigInt::from(1)),
                expected: "0.000000000000000001",
                description: "lower bound: 1 RECALL buys 1 atto credit",
            },
            TestCase {
                tokens: TokenAmount::from_nano(BigInt::from(500000000)), // 0.5 RECALL
                rate: TokenCreditRate::from(BigInt::from(1)),
                expected: "0.0",
                description: "crossing lower bound. 0.5 RECALL cannot buy 1 atto credit",
            },
            TestCase {
                tokens: TokenAmount::from_whole(BigInt::from(1)),
                rate: TokenCreditRate::from(BigInt::from(2)),
                expected: "0.000000000000000002",
                description: "1 RECALL buys 2 atto credits",
            },
            TestCase {
                tokens: TokenAmount::from_whole(BigInt::from(1)),
                rate: TokenCreditRate::from(BigInt::from(10u64.pow(18))),
                expected: "1.0",
                description: "1 RECALL buys 1 whole credit",
            },
            TestCase {
                tokens: TokenAmount::from_whole(BigInt::from(50)),
                rate: TokenCreditRate::from(BigInt::from(10u64.pow(18))),
                expected: "50.0",
                description: "50 RECALL buys 50 whole credits",
            },
            TestCase {
                tokens: TokenAmount::from_nano(BigInt::from(233432100u64)),
                rate: TokenCreditRate::from(BigInt::from(10u64.pow(18))),
                expected: "0.2334321",
                description: "0.2334321 RECALL buys 0.2334321 credits",
            },
            TestCase {
                tokens: TokenAmount::from_nano(BigInt::from(233432100u64)),
                rate: TokenCreditRate::from(BigInt::from(10u128.pow(36))),
                expected: "233432100000000000.0",
                description: "0.2334321 RECALL buys 233432100000000000 credits",
            },
            TestCase {
                tokens: TokenAmount::from_atto(BigInt::from(1)), // 1 attoRECALL
                rate: TokenCreditRate::from(BigInt::from(10u128.pow(36))),
                expected: "1.0",
                description: "1 atto RECALL buys 1 credit",
            },
            TestCase {
                tokens: TokenAmount::from_whole(BigInt::from(1)),
                rate: TokenCreditRate::from(BigInt::from(10u128.pow(18)).div(4)),
                expected: "0.25",
                description: "1 RECALL buys 0.25 credit",
            },
            TestCase {
                tokens: TokenAmount::from_whole(BigInt::from(1)),
                rate: TokenCreditRate::from(BigInt::from(10u128.pow(18)).div(3)),
                expected: "0.333333333333333333",
                description: "1 RECALL buys 0.333333333333333333 credit",
            },
        ];

        for t in test_cases {
            let credits = t.tokens.clone() * &t.rate;
            assert_eq!(
                t.expected,
                credits.to_string(),
                "tc: {}, {}, {}",
                t.description,
                t.tokens,
                t.rate
            );
        }
    }
}
