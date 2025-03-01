// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;
use std::error::Error;
use std::str::from_utf8;

use fendermint_actor_blobs_shared::{
    params::GetStatsReturn,
    state::{
        Account, Blob, BlobStatus, BlobSubscribers, Credit, Hash, PublicKey, Subscription,
        SubscriptionGroup, SubscriptionId, TtlStatus,
    },
};
use fendermint_actor_recall_config_shared::RecallConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, bigint::BigInt, clock::ChainEpoch, econ::TokenAmount};
use log::{debug, warn};
use num_traits::{ToPrimitive, Zero};
use recall_ipld::hamt::BytesKey;

type BlobSourcesResult =
    anyhow::Result<Vec<(Hash, u64, HashSet<(Address, SubscriptionId, PublicKey)>)>, ActorError>;

mod accounts;
mod blobs;
mod expiries;

use crate::caller::Caller;
use crate::credit::CommitCapacityParams;
use accounts::AccountsState;
pub use blobs::{AddBlobStateParams, DeleteBlobStateParams, FinalizeBlobStateParams};
use blobs::{BlobsProgressCollection, BlobsState};
use expiries::{ExpiriesState, ExpiryUpdate};

/// The state represents all accounts and stored blobs.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The total used storage capacity of the subnet.
    pub capacity_used: u64,
    /// The total number of credits sold in the subnet.
    pub credit_sold: Credit,
    /// The total number of credits committed to active storage in the subnet.
    pub credit_committed: Credit,
    /// The total number of credits debited in the subnet.
    pub credit_debited: Credit,
    /// Map of expiries to blob hashes.
    pub expiries: ExpiriesState,
    /// Map of currently added blob hashes to account and source Iroh node IDs.
    pub added: BlobsProgressCollection,
    /// Map of currently pending blob hashes to account and source Iroh node IDs.
    pub pending: BlobsProgressCollection,
    /// HAMT containing all accounts keyed by robust (non-ID) actor address.
    pub accounts: AccountsState,
    /// HAMT containing all blobs keyed by blob hash.
    pub blobs: BlobsState,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self, ActorError> {
        Ok(Self {
            capacity_used: 0,
            credit_sold: Credit::zero(),
            credit_committed: Credit::zero(),
            credit_debited: Credit::zero(),
            expiries: ExpiriesState::new(store)?,
            added: BlobsProgressCollection::new(store, "added blobs queue")?,
            pending: BlobsProgressCollection::new(store, "pending blobs queue")?,
            accounts: AccountsState::new(store)?,
            blobs: BlobsState::new(store)?,
        })
    }

    pub fn get_stats(&self, config: &RecallConfig, balance: TokenAmount) -> GetStatsReturn {
        GetStatsReturn {
            balance,
            capacity_free: self.capacity_available(config.blob_capacity),
            capacity_used: self.capacity_used,
            credit_sold: self.credit_sold.clone(),
            credit_committed: self.credit_committed.clone(),
            credit_debited: self.credit_debited.clone(),
            token_credit_rate: config.token_credit_rate.clone(),
            num_accounts: self.accounts.len(),
            num_blobs: self.blobs.len(),
            num_added: self.added.len(),
            bytes_added: self.added.bytes_size(),
            num_resolving: self.pending.len(),
            bytes_resolving: self.pending.bytes_size(),
        }
    }

    /// Returns an error if the subnet storage is at capacity.
    pub fn ensure_capacity(&self, capacity: u64) -> anyhow::Result<(), ActorError> {
        if self.capacity_available(capacity).is_zero() {
            return Err(ActorError::forbidden(
                "subnet has reached storage capacity".into(),
            ));
        }
        Ok(())
    }

    /// Return available capacity as a difference between `blob_capacity_total` and `capacity_used`.
    fn capacity_available(&self, blob_capacity_total: u64) -> u64 {
        // Prevent underflow. We only care if free capacity is > 0 anyway.
        blob_capacity_total.saturating_sub(self.capacity_used)
    }

    pub fn get_account<BS: Blockstore>(
        &self,
        store: &BS,
        address: Address,
    ) -> anyhow::Result<Option<Account>, ActorError> {
        let accounts = self.accounts.hamt(store)?;
        accounts.get(&address)
    }

    pub fn set_account_status<BS: Blockstore>(
        &mut self,
        store: &BS,
        config: &RecallConfig,
        address: Address,
        status: TtlStatus,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        let mut accounts = self.accounts.hamt(store)?;
        match status {
            // We don't want to create an account for default TTL
            TtlStatus::Default => {
                if let Some(mut account) = accounts.get(&address)? {
                    account.max_ttl = status.get_max_ttl(config.blob_default_ttl);
                    self.accounts
                        .save_tracked(accounts.set_and_flush_tracked(&address, account)?);
                }
            }
            _ => {
                // Get or create a new account
                let max_ttl = status.get_max_ttl(config.blob_default_ttl);
                let mut account = accounts
                    .get_or_create(&address, || Account::new(store, current_epoch, max_ttl))?;
                account.max_ttl = max_ttl;
                self.accounts
                    .save_tracked(accounts.set_and_flush_tracked(&address, account)?);
            }
        }
        Ok(())
    }

    pub fn debit_accounts<BS: Blockstore>(
        &mut self,
        store: &BS,
        config: &RecallConfig,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(HashSet<Hash>, bool), ActorError> {
        // Delete expired subscriptions
        let mut delete_from_disc = HashSet::new();
        let mut num_deleted = 0;
        let mut expiries = self.expiries.clone();
        expiries.foreach_up_to_epoch(
            store,
            current_epoch,
            Some(config.blob_delete_batch_size),
            |_, subscriber, key| {
                match self.delete_blob(
                    store,
                    subscriber,
                    None,
                    DeleteBlobStateParams {
                        hash: key.hash,
                        id: key.id.clone(),
                        epoch: current_epoch,
                    },
                ) {
                    Ok((from_disc, _)) => {
                        num_deleted += 1;
                        if from_disc {
                            delete_from_disc.insert(key.hash);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "failed to delete blob {} for {} (id: {}): {}",
                            key.hash, subscriber, key.id, e
                        )
                    }
                }
                Ok(())
            },
        )?;

        debug!("deleted {} expired subscriptions", num_deleted);
        debug!(
            "{} blobs marked for deletion from disc",
            delete_from_disc.len()
        );

        // Debit accounts for existing usage
        let reader = self.accounts.hamt(store)?;
        let mut writer = self.accounts.hamt(store)?;
        let start_key = self.accounts.get_debit_start_address();
        let (count, next_account) = reader.for_each_ranged(
            start_key.as_ref(),
            Some(config.account_debit_batch_size as usize),
            |address, account| {
                let mut caller =
                    Caller::load_account(store, &reader, address, account.clone(), None)?;
                let debit_duration = current_epoch - caller.subscriber().last_debit_epoch;
                let debit = self.get_debit_for_caller(&caller, debit_duration);
                self.debit_caller(&mut caller, &debit, current_epoch);
                caller.save(&mut writer)?;
                Ok(())
            },
        )?;

        // Save accounts
        self.accounts.save_tracked(writer.flush_tracked()?);
        self.accounts.save_debit_progress(next_account);

        debug!(
            "finished debiting {:#?} accounts, next account: {:#?}",
            count, next_account
        );

        Ok((delete_from_disc, next_account.is_some()))
    }

    /// Adds a blob.
    /// - Subscriber is either the caller or a designated sponsor
    /// - Subscriber's account/credits are used for:
    ///   - Initial storage transaction
    ///   - Ongoing storage costs over time
    /// - Typically managed by a wrapping Actor (e.g., Buckets, Timehub) that owns
    ///   the subscription
    pub fn add_blob<BS: Blockstore>(
        &mut self,
        store: &BS,
        config: &RecallConfig,
        caller: Address,
        sponsor: Option<Address>,
        params: AddBlobStateParams,
    ) -> anyhow::Result<(Subscription, TokenAmount), ActorError> {
        // Get or create a new account
        let mut accounts = self.accounts.hamt(store)?;
        let mut caller = Caller::load_or_create(
            store,
            &accounts,
            caller,
            sponsor,
            params.epoch,
            config.blob_default_ttl,
        )?;

        // Validate the TTL
        let ttl = caller.validate_ttl_usage(config, params.ttl)?;

        // Capacity updates and required credit depend on whether the subscriber is already
        // subscribing to this blob
        let expiry = i64::saturating_add(params.epoch, ttl);
        let mut new_subnet_capacity: u64 = 0;
        let mut new_caller_capacity: u64 = 0;
        let credit_required: Credit;

        // Get or create a new blob
        let mut blobs = self.blobs.hamt(store)?;
        let (sub, blob) = if let Some(mut blob) = blobs.get(&params.hash)? {
            let mut subscribers = blob.subscribers.hamt(store)?;
            let sub = if let Some(mut group) = subscribers.get(&caller.subscriber_address())? {
                let (group_expiry, new_group_expiry) =
                    group.max_expiries(store, &params.id, Some(expiry))?;

                // If the subscriber has been debited after the group's max expiry, we need to
                // clean up the accounting with a refund.
                // If the ensure-credit check below fails, the refund won't be saved in the
                // subscriber's state.
                // However, they will get rerefunded during the next auto debit tick.
                if let Some(group_expiry) = group_expiry {
                    if caller.subscriber().last_debit_epoch > group_expiry {
                        // Return over-debited credit.
                        // The refund extends up to the current epoch because we need to
                        // account for the charge that will happen below at the current epoch.
                        let return_credits = Credit::from_whole(
                            self.get_storage_cost(params.epoch - group_expiry, &params.size),
                        );
                        self.return_committed_credit_for_caller(&mut caller, &return_credits);
                    }
                }

                // Ensure subscriber has enough credits, considering the subscription group may
                // have expiries that cover a portion of the addition.
                // Required credit can be negative if subscriber is reducing expiry.
                // When adding, the new group expiry will always contain a value.
                let new_group_expiry = new_group_expiry.unwrap();
                let group_expiry = group_expiry.map_or(params.epoch, |e| e.max(params.epoch));
                credit_required = Credit::from_whole(
                    self.get_storage_cost(new_group_expiry - group_expiry, &params.size),
                );

                // Add / update subscription
                let mut group_hamt = group.hamt(store)?;
                let sub = if let Some(mut sub) = group_hamt.get(&params.id)? {
                    // Update expiry index
                    if expiry != sub.expiry {
                        self.expiries.update_index(
                            store,
                            caller.subscriber_address(),
                            params.hash,
                            &params.id,
                            vec![ExpiryUpdate::Add(expiry), ExpiryUpdate::Remove(sub.expiry)],
                        )?;
                    }
                    sub.expiry = expiry;

                    // Overwrite source allows subscriber to retry resolving
                    sub.source = params.source;
                    sub.delegate = caller.delegate_address();
                    sub.failed = false;
                    group.save_tracked(group_hamt.set_and_flush_tracked(&params.id, sub.clone())?);

                    debug!(
                        "updated subscription to blob {} for {} (key: {})",
                        params.hash,
                        caller.subscriber_address(),
                        params.id
                    );
                    sub
                } else {
                    // Add new subscription
                    let sub = Subscription {
                        added: params.epoch,
                        expiry,
                        source: params.source,
                        delegate: caller.delegate_address(),
                        failed: false,
                    };
                    group.save_tracked(group_hamt.set_and_flush_tracked(&params.id, sub.clone())?);

                    // Update expiry index
                    self.expiries.update_index(
                        store,
                        caller.subscriber_address(),
                        params.hash,
                        &params.id,
                        vec![ExpiryUpdate::Add(expiry)],
                    )?;

                    debug!(
                        "created new subscription to blob {} for {} (key: {})",
                        params.hash,
                        caller.subscriber_address(),
                        params.id
                    );
                    sub
                };

                blob.subscribers.save_tracked(
                    subscribers.set_and_flush_tracked(&caller.subscriber_address(), group)?,
                );

                sub
            } else {
                new_caller_capacity = params.size;

                // One or more accounts have already committed credit.
                // However, we still need to reserve the full required credit from the new
                // subscriber, as the existing account(s) may decide to change the expiry or cancel.
                credit_required = Credit::from_whole(self.get_storage_cost(ttl, &params.size));

                // Add new subscription
                let sub = Subscription {
                    added: params.epoch,
                    expiry,
                    source: params.source,
                    delegate: caller.delegate_address(),
                    failed: false,
                };
                let mut subscribers = blob.subscribers.hamt(store)?;
                let mut subscription_group = SubscriptionGroup::new(store)?;
                let mut subscription_group_hamt = subscription_group.hamt(store)?;
                subscription_group.save_tracked(
                    subscription_group_hamt.set_and_flush_tracked(&params.id, sub.clone())?,
                );
                blob.subscribers.save_tracked(
                    subscribers
                        .set_and_flush_tracked(&caller.subscriber_address(), subscription_group)?,
                );

                // Update expiry index
                self.expiries.update_index(
                    store,
                    caller.subscriber_address(),
                    params.hash,
                    &params.id,
                    vec![ExpiryUpdate::Add(expiry)],
                )?;

                debug!(
                    "created new subscription to blob {} for {} (key: {})",
                    params.hash,
                    caller.subscriber_address(),
                    params.id
                );
                sub
            };

            // Update blob status
            if !matches!(blob.status, BlobStatus::Resolved) {
                // It's pending or failed, reset to added status
                blob.status = BlobStatus::Added;

                // Add to or update the source in the added queue
                self.added.upsert(
                    store,
                    params.hash,
                    (caller.subscriber_address(), params.id, params.source),
                    blob.size,
                )?;
            }

            (sub, blob)
        } else {
            new_caller_capacity = params.size;

            // New blob increases network capacity as well.
            new_subnet_capacity = params.size;
            credit_required = Credit::from_whole(self.get_storage_cost(ttl, &params.size));

            // Create new blob
            let sub = Subscription {
                added: params.epoch,
                expiry,
                source: params.source,
                delegate: caller.delegate_address(),
                failed: false,
            };
            let blob_subscribers = BlobSubscribers::new(store)?;
            let mut subscribers = blob_subscribers.hamt(store)?;
            let mut blob = Blob {
                size: params.size.to_u64().unwrap(),
                metadata_hash: params.metadata_hash,
                subscribers: blob_subscribers,
                status: BlobStatus::Added,
            };
            let mut subscription_group = SubscriptionGroup::new(store)?;
            let mut subscription_group_hamt = subscription_group.hamt(store)?;
            subscription_group.save_tracked(
                subscription_group_hamt.set_and_flush_tracked(&params.id, sub.clone())?,
            );
            blob.subscribers.save_tracked(
                subscribers
                    .set_and_flush_tracked(&caller.subscriber_address(), subscription_group)?,
            );

            // Update expiry index
            self.expiries.update_index(
                store,
                caller.subscriber_address(),
                params.hash,
                &params.id,
                vec![ExpiryUpdate::Add(expiry)],
            )?;

            // Add the source to the added queue
            self.added.upsert(
                store,
                params.hash,
                (
                    caller.subscriber_address(),
                    params.id.clone(),
                    params.source,
                ),
                blob.size,
            )?;

            debug!("created new blob {}", params.hash);
            debug!(
                "created new subscription to blob {} for {} (key: {})",
                params.hash,
                caller.subscriber_address(),
                params.id
            );
            (sub, blob)
        };

        // Account capacity is changing, debit for existing usage
        let debit_duration = params.epoch - caller.subscriber().last_debit_epoch;
        let debit = self.get_debit_for_caller(&caller, debit_duration);
        self.debit_caller(&mut caller, &debit, params.epoch);

        // Account for new size and commit credit
        let token_rebate = if credit_required.is_positive() {
            self.commit_capacity_for_caller(
                &mut caller,
                config,
                CommitCapacityParams {
                    subnet_size: new_subnet_capacity,
                    caller_size: new_caller_capacity,
                    cost: credit_required,
                    value: params.token_amount,
                    epoch: params.epoch,
                },
            )?
        } else {
            self.uncommit_capacity_for_caller(
                &mut caller,
                new_subnet_capacity,
                new_caller_capacity,
                &-credit_required,
            );
            TokenAmount::zero()
        };

        // Save caller
        self.save_caller(&mut caller, &mut accounts)?;

        // Save blob
        self.blobs
            .save_tracked(blobs.set_and_flush_tracked(&params.hash, blob)?);

        Ok((sub, token_rebate))
    }

    fn get_storage_cost(&self, duration: i64, size: &u64) -> BigInt {
        duration * BigInt::from(*size)
    }

    fn get_debit_for_caller<BS: Blockstore>(
        &self,
        caller: &Caller<BS>,
        duration: ChainEpoch,
    ) -> Credit {
        Credit::from_whole(BigInt::from(caller.subscriber().capacity_used) * duration)
    }

    pub fn get_blob<BS: Blockstore>(
        &self,
        store: &BS,
        hash: Hash,
    ) -> anyhow::Result<Option<Blob>, ActorError> {
        let blobs = self.blobs.hamt(store)?;
        blobs.get(&hash)
    }

    pub fn get_blob_status<BS: Blockstore>(
        &self,
        store: &BS,
        subscriber: Address,
        hash: Hash,
        id: SubscriptionId,
    ) -> anyhow::Result<Option<BlobStatus>, ActorError> {
        let blob = if let Some(blob) = self
            .blobs
            .hamt(store)
            .ok()
            .and_then(|blobs| blobs.get(&hash).ok())
            .flatten()
        {
            blob
        } else {
            return Ok(None);
        };

        let subscribers = blob.subscribers.hamt(store)?;
        if subscribers.contains_key(&subscriber)? {
            match blob.status {
                BlobStatus::Added => Ok(Some(BlobStatus::Added)),
                BlobStatus::Pending => Ok(Some(BlobStatus::Pending)),
                BlobStatus::Resolved => Ok(Some(BlobStatus::Resolved)),
                BlobStatus::Failed => {
                    // The blob state's status may have been finalized as failed by another
                    // subscription.
                    // We need to see if this specific subscription failed.
                    let group = subscribers.get(&subscriber)?.unwrap(); // safe here
                    let group_hamt = group.hamt(store)?;
                    if let Some(sub) = group_hamt.get(&id)? {
                        if sub.failed {
                            Ok(Some(BlobStatus::Failed))
                        } else {
                            Ok(Some(BlobStatus::Pending))
                        }
                    } else {
                        Ok(None)
                    }
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_added_blobs<BS: Blockstore>(&self, store: &BS, size: u32) -> BlobSourcesResult {
        let blobs = self.blobs.hamt(store)?;
        self.added
            .take_page(store, size)?
            .into_iter()
            .map(|(hash, sources)| {
                let blob = blobs
                    .get(&hash)?
                    .ok_or_else(|| ActorError::not_found(format!("blob {} not found", hash)))?;
                Ok((hash, blob.size, sources))
            })
            .collect()
    }

    pub fn get_pending_blobs<BS: Blockstore>(&self, store: &BS, size: u32) -> BlobSourcesResult {
        let blobs = self.blobs.hamt(store)?;
        self.pending
            .take_page(store, size)?
            .into_iter()
            .map(|(hash, sources)| {
                let blob = blobs
                    .get(&hash)?
                    .ok_or_else(|| ActorError::not_found(format!("blob {} not found", hash)))?;
                Ok((hash, blob.size, sources))
            })
            .collect()
    }

    pub fn set_blob_pending<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        hash: Hash,
        size: u64,
        id: SubscriptionId,
        source: PublicKey,
    ) -> anyhow::Result<(), ActorError> {
        let mut blobs = self.blobs.hamt(store)?;
        let mut blob = if let Some(blob) = blobs.get(&hash)? {
            blob
        } else {
            // The blob may have been deleted before it was set to pending
            return Ok(());
        };
        // check if the blob's size matches the size provided when it was added
        if blob.size != size {
            return Err(ActorError::assertion_failed(format!(
                "blob {} size mismatch (expected: {}; actual: {})",
                hash, size, blob.size
            )));
        }
        blob.status = BlobStatus::Pending;

        // Add the source to the pending queue
        self.pending
            .upsert(store, hash, (subscriber, id, source), blob.size)?;

        // Remove entire blob entry from the added queue
        self.added.remove_entry(store, &hash, blob.size)?;

        // Save blob
        self.blobs
            .save_tracked(blobs.set_and_flush_tracked(&hash, blob)?);
        Ok(())
    }

    pub fn finalize_blob<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        params: FinalizeBlobStateParams,
    ) -> anyhow::Result<(), ActorError> {
        // Validate incoming status
        if matches!(params.status, BlobStatus::Added | BlobStatus::Pending) {
            return Err(ActorError::illegal_state(format!(
                "cannot finalize blob {} as added or pending",
                params.hash
            )));
        }

        // Get the blob
        let mut blobs = self.blobs.hamt(store)?;
        let mut blob = if let Some(blob) = blobs.get(&params.hash)? {
            blob
        } else {
            // The blob may have been deleted before it was finalized
            return Ok(());
        };
        if matches!(blob.status, BlobStatus::Added) {
            return Err(ActorError::illegal_state(format!(
                "blob {} cannot be finalized from status added",
                params.hash
            )));
        } else if matches!(blob.status, BlobStatus::Resolved) {
            // Blob is already finalized as resolved.
            // We can ignore later finalizations, even if they are failed.
            return Ok(());
        }
        let mut subscribers = blob.subscribers.hamt(store)?;
        let mut group = subscribers
            .get(&subscriber)?
            .ok_or(ActorError::forbidden(format!(
                "subscriber {} is not subscribed to blob {}",
                subscriber, params.hash
            )))?;

        // Get max expiries with the current subscription removed in case we need them below.
        // We have to do this here to avoid breaking borrow rules.
        let (group_expiry, new_group_expiry) = group.max_expiries(store, &params.id, Some(0))?;
        let (sub_is_min_added, next_min_added) = group.is_min_added(store, &params.id)?;
        let mut group_hamt = group.hamt(store)?;
        let mut sub = group_hamt
            .get(&params.id)?
            .ok_or(ActorError::not_found(format!(
                "subscription id {} not found",
                params.id.clone()
            )))?;

        // Load the caller account and delegation.
        let mut accounts = self.accounts.hamt(store)?;
        let mut caller = Caller::load(
            store,
            &accounts,
            sub.delegate.unwrap_or(subscriber),
            sub.delegate.map(|_| subscriber),
        )?;

        // Update blob status
        blob.status = params.status.clone();
        if matches!(blob.status, BlobStatus::Failed) {
            // We're not going to make a debit, but we need to refund any spent credits that may
            // have been used on this group in the event the last debit is later than the
            // added epoch.
            // When failing, the existing group expiry will always contain a value.
            let group_expiry = group_expiry.unwrap();
            let size = blob.size;
            let last_debit_epoch = caller.subscriber().last_debit_epoch;
            if last_debit_epoch > sub.added && sub_is_min_added {
                // The refund extends up to either the next minimum added epoch that is less
                // than the last debit epoch, or the last debit epoch.
                let cutoff = next_min_added
                    .unwrap_or(last_debit_epoch)
                    .min(last_debit_epoch);
                let refund_credits =
                    Credit::from_whole(self.get_storage_cost(cutoff - sub.added, &size));
                let correction_credits = if cutoff > group_expiry {
                    Credit::from_whole(self.get_storage_cost(cutoff - group_expiry, &size))
                } else {
                    Credit::zero()
                };
                self.refund_caller(&mut caller, &refund_credits, &correction_credits);
            }

            // If there's no new group expiry, all subscriptions have failed.
            let reclaim_capacity = if new_group_expiry.is_none() { size } else { 0 };

            // Release credits considering other subscriptions may still be pending.
            let reclaim_credits = if last_debit_epoch < group_expiry {
                Credit::from_whole(self.get_storage_cost(
                    group_expiry
                        - new_group_expiry.map_or(last_debit_epoch, |e| e.max(last_debit_epoch)),
                    &size,
                ))
            } else {
                Credit::zero()
            };
            self.uncommit_capacity_for_caller(
                &mut caller,
                reclaim_capacity,
                reclaim_capacity,
                &reclaim_credits,
            );

            // Mark the subscription as failed
            sub.failed = true;

            // Save the subscription
            group.save_tracked(group_hamt.set_and_flush_tracked(&params.id, sub.clone())?);
        }

        // Remove the source from the pending queue
        self.pending.remove_source(
            store,
            params.hash,
            (subscriber, params.id, sub.source),
            blob.size,
        )?;

        // Save accounts
        caller.save(&mut accounts)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);

        // Save blob
        blob.subscribers
            .save_tracked(subscribers.set_and_flush_tracked(&subscriber, group)?);
        self.blobs
            .save_tracked(blobs.set_and_flush_tracked(&params.hash, blob)?);

        debug!("finalized blob {} to status {}", params.hash, params.status);

        Ok(())
    }

    pub fn delete_blob<BS: Blockstore>(
        &mut self,
        store: &BS,
        caller: Address,
        sponsor: Option<Address>,
        params: DeleteBlobStateParams,
    ) -> anyhow::Result<(bool, u64), ActorError> {
        // Load the caller account and delegation.
        let mut accounts = self.accounts.hamt(store)?;
        let mut caller = Caller::load(store, &accounts, caller, sponsor)?;
        caller.validate_delegate_expiration(params.epoch)?;

        // Get the blob
        let mut blobs = self.blobs.hamt(store)?;
        let mut blob = if let Some(blob) = blobs.get(&params.hash)? {
            blob
        } else {
            // We could error here, but since this method is called from other actors,
            // they would need to be able to identify this specific case.
            // For example, the bucket actor may need to delete a blob while overwriting
            // an existing key.
            // However, the system may have already deleted the blob due to expiration or
            // insufficient funds.
            // We could use a custom error code, but this is easier.
            return Ok((false, 0));
        };
        let mut subscribers = blob.subscribers.hamt(store)?;
        let num_subscribers = blob.subscribers.len();
        let mut group =
            subscribers
                .get(&caller.subscriber_address())?
                .ok_or(ActorError::forbidden(format!(
                    "subscriber {} is not subscribed to blob {}",
                    caller.subscriber_address(),
                    params.hash
                )))?;
        let mut group_hamt = group.hamt(store)?;
        let sub = group_hamt
            .get(&params.id)?
            .ok_or(ActorError::not_found(format!(
                "subscription id {} not found",
                params.id.clone()
            )))?;

        // Do not allow deletion if status is added or pending.
        // This would cause issues with deletion from disc.
        if matches!(blob.status, BlobStatus::Added) || matches!(blob.status, BlobStatus::Pending) {
            return Err(ActorError::forbidden(format!(
                "blob {} pending finalization; please wait",
                params.hash
            )));
        }

        // Since the charge will be for all the account's blobs, we can only
        // account for capacity up to this blob's expiry if it is less than
        // the current epoch.
        // If the subscription is failed, there may be no group expiry.
        let (group_expiry, new_group_expiry) = group.max_expiries(store, &params.id, Some(0))?;
        if let Some(group_expiry) = group_expiry {
            let debit_epoch = group_expiry.min(params.epoch);
            // Account capacity is changing, debit for existing usage.
            // It could be possible that debit epoch is less than the last debit,
            // in which case we need to refund for that duration.
            let last_debit_epoch = caller.subscriber().last_debit_epoch;
            if last_debit_epoch < debit_epoch {
                let debit = self.get_debit_for_caller(&caller, debit_epoch - last_debit_epoch);
                self.debit_caller(&mut caller, &debit, debit_epoch);
            } else if last_debit_epoch != debit_epoch {
                // The account was debited after this blob's expiry
                // Return over-debited credit
                let return_credits = Credit::from_whole(
                    self.get_storage_cost(last_debit_epoch - group_expiry, &blob.size),
                );
                self.return_committed_credit_for_caller(&mut caller, &return_credits);
            }
        }

        // Account for reclaimed size and move committed credit to free credit
        // If blob failed, capacity and committed credits have already been returned
        let size = blob.size;
        if !matches!(blob.status, BlobStatus::Failed) && !sub.failed {
            // If there's no new group expiry, we can reclaim capacity.
            let (reclaim_subnet_capacity, reclaim_caller_capacity) = if new_group_expiry.is_none() {
                (size, if num_subscribers == 1 { size } else { 0 })
            } else {
                (0, 0)
            };

            // We can release credits if the new group expiry is in the future,
            // considering other subscriptions may still be active.
            let reclaim_credits = group_expiry
                .map(|group_expiry| {
                    let last_debit_epoch = caller.subscriber().last_debit_epoch;
                    if last_debit_epoch < group_expiry {
                        let reclaim_duration_start =
                            new_group_expiry.map_or(last_debit_epoch, |e| e.max(last_debit_epoch));
                        Credit::from_whole(
                            self.get_storage_cost(group_expiry - reclaim_duration_start, &size),
                        )
                    } else {
                        Credit::zero()
                    }
                })
                .unwrap_or_default();
            self.uncommit_capacity_for_caller(
                &mut caller,
                reclaim_subnet_capacity,
                reclaim_caller_capacity,
                &reclaim_credits,
            );
        }

        // Update expiry index
        self.expiries.update_index(
            store,
            caller.subscriber_address(),
            params.hash,
            &params.id,
            vec![ExpiryUpdate::Remove(sub.expiry)],
        )?;

        // Remove the source from the added queue
        self.added.remove_source(
            store,
            params.hash,
            (caller.subscriber_address(), params.id.clone(), sub.source),
            size,
        )?;

        // Remove the source from the pending queue
        self.pending.remove_source(
            store,
            params.hash,
            (caller.subscriber_address(), params.id.clone(), sub.source),
            size,
        )?;

        // Delete subscription
        group.save_tracked(group_hamt.delete_and_flush_tracked(&params.id)?.0);
        debug!(
            "deleted subscription to blob {} for {} (key: {})",
            params.hash,
            caller.subscriber_address(),
            params.id
        );

        // Delete the group if empty
        let delete_blob = if group.is_empty() {
            blob.subscribers.save_tracked(
                subscribers
                    .delete_and_flush_tracked(&caller.subscriber_address())?
                    .0,
            );
            debug!(
                "deleted subscriber {} to blob {}",
                caller.subscriber_address(),
                params.hash
            );

            // Delete or update blob
            let delete_blob = subscribers.is_empty();
            if delete_blob {
                self.blobs
                    .save_tracked(blobs.delete_and_flush_tracked(&params.hash)?.0);
                debug!("deleted blob {}", params.hash);
            }
            delete_blob
        } else {
            blob.subscribers.save_tracked(
                subscribers.set_and_flush_tracked(&caller.subscriber_address(), group)?,
            );
            self.blobs
                .save_tracked(blobs.set_and_flush_tracked(&params.hash, blob)?);
            false
        };

        // Save accounts
        caller.save(&mut accounts)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);

        Ok((delete_blob, size))
    }

    /// Adjusts all subscriptions for `account` according to its max TTL.
    /// Returns the number of subscriptions processed and the next key to continue iteration.
    /// If `starting_hash` is `None`, iteration starts from the beginning.
    /// If `limit` is `None`, all subscriptions are processed.
    /// If `limit` is not `None`, iteration stops after examining `limit` blobs.
    pub fn trim_blob_expiries<BS: Blockstore>(
        &mut self,
        config: &RecallConfig,
        store: &BS,
        subscriber: Address,
        current_epoch: ChainEpoch,
        starting_hash: Option<Hash>,
        limit: Option<usize>,
    ) -> anyhow::Result<(u32, Option<Hash>, Vec<Hash>), ActorError> {
        let new_ttl = self.get_account_max_ttl(config, store, subscriber)?;
        let mut deleted_blobs = Vec::new();
        let mut processed = 0;
        let blobs = self.blobs.hamt(store)?;
        let starting_key = starting_hash.map(|h| BytesKey::from(h.0.as_slice()));

        fn err_map<E>(e: E) -> ActorError
        where
            E: Error,
        {
            ActorError::illegal_state(format!(
                "subscriptions group cannot be iterated over: {}",
                e
            ))
        }

        let (_, next_key) = blobs.for_each_ranged(
            starting_key.as_ref(),
            limit,
            |hash, blob| -> Result<(), ActorError> {
                let subscribers = blob.subscribers.hamt(store)?;
                if let Some(group) = subscribers.get(&subscriber)? {
                    let group_hamt = group.hamt(store)?;
                    for val in group_hamt.iter() {
                        let (id_bytes, sub) = val.map_err(err_map)?;
                        let id = from_utf8(id_bytes).map_err(err_map)?;

                        if sub.expiry - sub.added > new_ttl {
                            if new_ttl == 0 {
                                // Delete subscription
                                let (from_disc, _) = self.delete_blob(
                                    store,
                                    subscriber,
                                    None,
                                    DeleteBlobStateParams {
                                        epoch: current_epoch,
                                        hash,
                                        id: SubscriptionId::new(id)?,
                                    },
                                )?;
                                if from_disc {
                                    deleted_blobs.push(hash);
                                };
                            } else {
                                self.add_blob(
                                    store,
                                    config,
                                    subscriber,
                                    None,
                                    AddBlobStateParams {
                                        hash,
                                        metadata_hash: blob.metadata_hash,
                                        id: SubscriptionId::new(id)?,
                                        size: blob.size,
                                        ttl: Some(new_ttl),
                                        source: sub.source,
                                        epoch: current_epoch,
                                        token_amount: TokenAmount::zero(),
                                    },
                                )?;
                            }
                            processed += 1;
                        }
                    }
                }
                Ok(())
            },
        )?;

        Ok((processed, next_key, deleted_blobs))
    }

    pub fn get_account_max_ttl<BS: Blockstore>(
        &self,
        config: &RecallConfig,
        store: &BS,
        address: Address,
    ) -> Result<ChainEpoch, ActorError> {
        let accounts = self.accounts.hamt(store)?;
        Ok(accounts
            .get(&address)?
            .map_or(config.blob_default_ttl, |account| account.max_ttl))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::caller::DelegationOptions;
    use fendermint_actor_blobs_testing::{
        new_address, new_hash, new_metadata_hash, new_pk, new_subscription_id, setup_logs,
    };
    use fvm_ipld_blockstore::MemoryBlockstore;
    use rand::seq::SliceRandom;
    use rand::Rng;
    use std::collections::{BTreeMap, HashMap};
    use std::ops::{AddAssign, SubAssign};

    fn check_approval_used<BS: Blockstore>(
        state: &State,
        store: &BS,
        caller: Address,
        sponsor: Address,
    ) {
        assert_ne!(caller, sponsor);
        let subscriber_account = state.get_account(&store, sponsor).unwrap().unwrap();
        let subscriber_approval = subscriber_account
            .approvals_to
            .hamt(store)
            .unwrap()
            .get(&caller)
            .unwrap()
            .unwrap();
        assert_eq!(
            subscriber_approval.credit_used,
            state.credit_debited.clone() + subscriber_account.credit_committed.clone()
        );
        let origin_account = state.get_account(&store, caller).unwrap().unwrap();
        let origin_approval = origin_account
            .approvals_from
            .hamt(store)
            .unwrap()
            .get(&sponsor)
            .unwrap()
            .unwrap();
        assert_eq!(
            subscriber_approval.credit_used,
            &state.credit_debited + &subscriber_account.credit_committed
        );
        assert_eq!(subscriber_approval.credit_used, origin_approval.credit_used);
    }

    #[test]
    fn test_debit_accounts_delete_from_disc() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, &config, caller, token_amount.clone(), current_epoch)
            .unwrap();
        debit_accounts_delete_from_disc(
            &config,
            &store,
            state,
            caller,
            None,
            current_epoch,
            token_amount,
            false,
        );
    }

    #[test]
    fn test_debit_accounts_delete_from_disc_with_approval() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let sponsor = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &store,
                &config,
                sponsor,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(
                &config,
                &store,
                sponsor,
                caller,
                DelegationOptions::default(),
                current_epoch,
            )
            .unwrap();
        debit_accounts_delete_from_disc(
            &config,
            &store,
            state,
            caller,
            Some(sponsor),
            current_epoch,
            token_amount,
            true,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn debit_accounts_delete_from_disc<BS: Blockstore>(
        config: &RecallConfig,
        store: &BS,
        mut state: State,
        caller: Address,
        sponsor: Option<Address>,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
        using_approval: bool,
    ) {
        let subscriber = sponsor.unwrap_or(caller);
        let mut credit_amount =
            Credit::from_atto(token_amount.atto().clone()) * &config.token_credit_rate;

        // Add blob with default a subscription ID
        let (hash, size) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::default();
        let ttl1 = ChainEpoch::from(config.blob_min_ttl);
        let source = new_pk();
        let res = state.add_blob(
            &store,
            config,
            caller,
            sponsor,
            AddBlobStateParams {
                hash,
                metadata_hash: new_metadata_hash(),
                id: id1.clone(),
                size,
                ttl: Some(ttl1),
                source,
                epoch: add1_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());

        let stats = state.get_stats(config, TokenAmount::zero());
        // Using a credit delegation creates both the from and to account
        let expected_num_accounts = if using_approval { 2 } else { 1 };
        assert_eq!(stats.num_accounts, expected_num_accounts);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size);

        // Set to status pending
        let res = state.set_blob_pending(&store, subscriber, hash, size, id1.clone(), source);
        assert!(res.is_ok());
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 1);
        assert_eq!(stats.bytes_resolving, size);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &store,
            subscriber,
            FinalizeBlobStateParams {
                hash,
                id: id1.clone(),
                status: BlobStatus::Resolved,
                epoch: finalize_epoch,
            },
        );
        assert!(res.is_ok());
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            Credit::from_whole(ttl1 as u64 * size)
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size);

        // Add the same blob but this time uses a different subscription ID
        let add2_epoch = ChainEpoch::from(21);
        let ttl2 = ChainEpoch::from(config.blob_min_ttl);
        let id2 = SubscriptionId::new("foo").unwrap();
        let source = new_pk();
        let res = state.add_blob(
            &store,
            config,
            caller,
            sponsor,
            AddBlobStateParams {
                hash,
                metadata_hash: new_metadata_hash(),
                id: id2.clone(),
                size,
                ttl: Some(ttl2),
                source,
                epoch: add2_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());

        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        assert_eq!(
            account.credit_committed, // stays the same becuase we're starting over
            Credit::from_whole(ttl2 as u64 * size),
        );
        credit_amount -= Credit::from_whole((add2_epoch - add1_epoch) as u64 * size);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size); // not changed

        // Check the subscription group
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        let subscribers = blob.subscribers.hamt(store).unwrap();
        let group = subscribers.get(&subscriber).unwrap().unwrap();
        assert_eq!(group.len(), 2);

        // Debit all accounts at an epoch between the two expiries (3601-3621)
        let debit_epoch = ChainEpoch::from(config.blob_min_ttl + 11);
        let (deletes_from_disc, _) = state.debit_accounts(&store, config, debit_epoch).unwrap();
        assert!(deletes_from_disc.is_empty());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // debit reduces this
            Credit::from_whole((ttl2 - (debit_epoch - add2_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size); // not changed

        // Check the subscription group
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        let subscribers = blob.subscribers.hamt(&store).unwrap();
        let group = subscribers.get(&subscriber).unwrap().unwrap();
        assert_eq!(group.len(), 1); // the first subscription was deleted

        // Debit all accounts at an epoch greater than group expiry (3621)
        let debit_epoch = ChainEpoch::from(config.blob_min_ttl + 31);
        let (deletes_from_disc, _) = state.debit_accounts(&store, config, debit_epoch).unwrap();
        assert!(!deletes_from_disc.is_empty()); // blob is marked for deletion

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // the second debit reduces this to zero
            Credit::from_whole(0),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, 0);

        // Check state
        assert_eq!(state.credit_committed, Credit::from_whole(0)); // credit was released
        assert_eq!(
            state.credit_debited,
            token_amount * &config.token_credit_rate - &account.credit_free
        );
        assert_eq!(state.capacity_used, 0); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 0);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        if using_approval {
            check_approval_used(&state, store, caller, subscriber);
        }
    }

    #[test]
    fn test_paginated_debit_accounts() {
        let config = RecallConfig {
            account_debit_batch_size: 5, // Process 5 accounts at a time (10 accounts total)
            ..Default::default()
        };

        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let current_epoch = ChainEpoch::from(1);

        // Create more than one batch worth of accounts (>5)
        for i in 0..10 {
            let address = Address::new_id(1000 + i);
            let token_amount = TokenAmount::from_whole(10);

            // Buy credits for each account
            state
                .buy_credit(
                    &store,
                    &config,
                    address,
                    token_amount.clone(),
                    current_epoch,
                )
                .unwrap();

            // Add some storage usage
            let mut accounts = state.accounts.hamt(&store).unwrap();
            let mut account = accounts.get(&address).unwrap().unwrap();
            account.capacity_used = 1000;
            accounts.set(&address, account).unwrap();
        }

        // First batch (should process 5 accounts)
        assert!(state.accounts.get_debit_start_address().is_none());
        let (deletes1, _) = state
            .debit_accounts(&store, &config, current_epoch + 1)
            .unwrap();
        assert!(deletes1.is_empty()); // No expired blobs
        assert!(state.accounts.get_debit_start_address().is_some());

        // Second batch (should process remaining 5 accounts and clear state)
        let (deletes2, _) = state
            .debit_accounts(&store, &config, current_epoch + 1)
            .unwrap();
        assert!(deletes2.is_empty());
        assert!(state.accounts.get_debit_start_address().is_none()); // The state should be cleared after all accounts processed

        // Verify all accounts were processed
        let reader = state.accounts.hamt(&store).unwrap();
        reader
            .for_each(|_, account| {
                assert_eq!(account.last_debit_epoch, current_epoch + 1);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_multiple_debit_cycles() {
        let config = RecallConfig {
            account_debit_batch_size: 5, // Process 5 accounts at a time (10 accounts total)
            ..Default::default()
        };

        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let current_epoch = ChainEpoch::from(1);

        // Create accounts
        for i in 0..10 {
            let address = Address::new_id(1000 + i);
            let token_amount = TokenAmount::from_whole(10);
            state
                .buy_credit(
                    &store,
                    &config,
                    address,
                    token_amount.clone(),
                    current_epoch,
                )
                .unwrap();

            let mut accounts = state.accounts.hamt(&store).unwrap();
            let mut account = accounts.get(&address).unwrap().unwrap();
            account.capacity_used = 1000;
            accounts.set(&address, account).unwrap();
        }

        // First cycle
        let (deletes1, _) = state
            .debit_accounts(&store, &config, current_epoch + 1)
            .unwrap();
        assert!(deletes1.is_empty());
        assert!(state.accounts.get_debit_start_address().is_some());

        let (deletes2, _) = state
            .debit_accounts(&store, &config, current_epoch + 1)
            .unwrap();
        assert!(deletes2.is_empty());
        assert!(state.accounts.get_debit_start_address().is_none()); // First cycle complete

        // Second cycle
        let (deletes3, _) = state
            .debit_accounts(&store, &config, current_epoch + 2)
            .unwrap();
        assert!(deletes3.is_empty());
        assert!(state.accounts.get_debit_start_address().is_some());

        let (deletes4, _) = state
            .debit_accounts(&store, &config, current_epoch + 2)
            .unwrap();
        assert!(deletes4.is_empty());
        assert!(state.accounts.get_debit_start_address().is_none()); // Second cycle complete
    }

    #[test]
    fn test_add_blob_refund() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, &config, caller, token_amount.clone(), current_epoch)
            .unwrap();
        add_blob_refund(
            &config,
            &store,
            state,
            caller,
            None,
            current_epoch,
            token_amount,
            false,
        );
    }

    #[test]
    fn test_add_blob_refund_with_approval() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let sponsor = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &store,
                &config,
                sponsor,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(
                &config,
                &store,
                sponsor,
                caller,
                DelegationOptions::default(),
                current_epoch,
            )
            .unwrap();
        add_blob_refund(
            &config,
            &store,
            state,
            caller,
            Some(sponsor),
            current_epoch,
            token_amount,
            true,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn add_blob_refund<BS: Blockstore>(
        config: &RecallConfig,
        store: &BS,
        mut state: State,
        caller: Address,
        sponsor: Option<Address>,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
        using_approval: bool,
    ) {
        let subscriber = sponsor.unwrap_or(caller);
        let token_credit_rate = BigInt::from(1_000_000_000_000_000_000u64);
        let mut credit_amount = token_amount.clone() * &config.token_credit_rate;

        // Add blob with default a subscription ID
        let (hash1, size1) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::default();
        let source = new_pk();
        let res = state.add_blob(
            &store,
            config,
            caller,
            sponsor,
            AddBlobStateParams {
                hash: hash1,
                metadata_hash: new_metadata_hash(),
                id: id1.clone(),
                size: size1,
                ttl: Some(config.blob_min_ttl),
                source,
                epoch: add1_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size1);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            Credit::from_whole(config.blob_min_ttl as u64 * size1),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1);

        assert!(state
            .set_account_status(
                &store,
                config,
                subscriber,
                TtlStatus::Extended,
                current_epoch
            )
            .is_ok());

        // Add another blob past the first blob's expiry
        let (hash2, size2) = new_hash(2048);
        let add2_epoch = ChainEpoch::from(config.blob_min_ttl + 11);
        let id2 = SubscriptionId::new("foo").unwrap();
        let source = new_pk();
        let res = state.add_blob(
            &store,
            config,
            caller,
            sponsor,
            AddBlobStateParams {
                hash: hash2,
                metadata_hash: new_metadata_hash(),
                id: id2.clone(),
                size: size2,
                ttl: Some(config.blob_min_ttl),
                source,
                epoch: add2_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 2);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 2);
        assert_eq!(stats.bytes_added, size1 + size2);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        let blob1_expiry = ChainEpoch::from(config.blob_min_ttl + add1_epoch);
        let overcharge = BigInt::from((add2_epoch - blob1_expiry) as u64 * size1);
        assert_eq!(
            account.credit_committed, // this includes an overcharge that needs to be refunded
            Credit::from_whole(config.blob_min_ttl as u64 * size2 - overcharge),
        );
        credit_amount -= Credit::from_whole(config.blob_min_ttl as u64 * size2);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1 + size2);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            (token_amount.clone() * &token_credit_rate)
                - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 2);
        assert_eq!(state.added.len(), 2);
        assert_eq!(state.pending.len(), 0);

        // Add the first (now expired) blob again
        let add3_epoch = ChainEpoch::from(config.blob_min_ttl + 21);
        let id1 = SubscriptionId::default();
        let source = new_pk();
        let res = state.add_blob(
            &store,
            config,
            caller,
            sponsor,
            AddBlobStateParams {
                hash: hash1,
                metadata_hash: new_metadata_hash(),
                id: id1.clone(),
                size: size1,
                ttl: Some(config.blob_min_ttl),
                source,
                epoch: add3_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 2);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 2);
        assert_eq!(stats.bytes_added, size1 + size2);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add3_epoch);
        assert_eq!(
            account.credit_committed, // should not include overcharge due to refund
            Credit::from_whole(
                (config.blob_min_ttl - (add3_epoch - add2_epoch)) as u64 * size2
                    + config.blob_min_ttl as u64 * size1
            ),
        );
        credit_amount -= Credit::from_whole(config.blob_min_ttl as u64 * size1);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1 + size2);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            token_amount.clone() * &token_credit_rate
                - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 2);
        assert_eq!(state.added.len(), 2);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        if using_approval {
            check_approval_used(&state, store, caller, subscriber);
        }
    }

    #[test]
    fn test_add_blob_same_hash_same_account() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, &config, caller, token_amount.clone(), current_epoch)
            .unwrap();
        add_blob_same_hash_same_account(
            &config,
            &store,
            state,
            caller,
            None,
            current_epoch,
            token_amount,
            false,
        );
    }

    #[test]
    fn test_add_blob_same_hash_same_account_with_approval() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let sponsor = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &store,
                &config,
                sponsor,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(
                &config,
                &store,
                sponsor,
                caller,
                DelegationOptions::default(),
                current_epoch,
            )
            .unwrap();
        add_blob_same_hash_same_account(
            &config,
            &store,
            state,
            caller,
            Some(sponsor),
            current_epoch,
            token_amount,
            true,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn add_blob_same_hash_same_account<BS: Blockstore>(
        config: &RecallConfig,
        store: &BS,
        mut state: State,
        caller: Address,
        sponsor: Option<Address>,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
        using_approval: bool,
    ) {
        let subscriber = sponsor.unwrap_or(caller);
        let mut credit_amount =
            Credit::from_atto(token_amount.atto().clone()) * &config.token_credit_rate;

        assert!(state
            .set_account_status(
                &store,
                config,
                subscriber,
                TtlStatus::Extended,
                current_epoch
            )
            .is_ok());

        // Add blob with default a subscription ID
        let (hash, size) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::default();
        let source = new_pk();
        let res = state.add_blob(
            &store,
            config,
            caller,
            sponsor,
            AddBlobStateParams {
                hash,
                metadata_hash: new_metadata_hash(),
                id: id1.clone(),
                size,
                ttl: Some(config.blob_min_ttl),
                source,
                epoch: add1_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());
        let (sub, _) = res.unwrap();
        assert_eq!(sub.added, add1_epoch);
        assert_eq!(sub.expiry, add1_epoch + config.blob_min_ttl);
        assert_eq!(sub.source, source);
        assert!(!sub.failed);
        if subscriber != caller {
            assert_eq!(sub.delegate, Some(caller));
        }

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size);

        // Check the blob status
        assert_eq!(
            state
                .get_blob_status(&store, subscriber, hash, id1.clone())
                .unwrap(),
            Some(BlobStatus::Added)
        );

        // Check the blob
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        let subscribers = blob.subscribers.hamt(store).unwrap();
        assert_eq!(blob.subscribers.len(), 1);
        assert_eq!(blob.status, BlobStatus::Added);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = subscribers.get(&subscriber).unwrap().unwrap();
        let group_hamt = group.hamt(store).unwrap();
        assert_eq!(group.len(), 1);
        let got_sub = group_hamt.get(&id1.clone()).unwrap().unwrap();
        assert_eq!(got_sub, sub);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            Credit::from_whole(config.blob_min_ttl as u64 * size),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size);

        // Set to status pending
        let res = state.set_blob_pending(&store, subscriber, hash, size, id1.clone(), source);
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 1);
        assert_eq!(stats.bytes_resolving, size);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &store,
            subscriber,
            FinalizeBlobStateParams {
                hash,
                id: id1.clone(),
                status: BlobStatus::Resolved,
                epoch: finalize_epoch,
            },
        );
        assert!(res.is_ok());
        assert_eq!(
            state
                .get_blob_status(&store, subscriber, hash, id1.clone())
                .unwrap(),
            Some(BlobStatus::Resolved)
        );

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Add the same blob again with a default subscription ID
        let add2_epoch = ChainEpoch::from(21);
        let source = new_pk();
        let res = state.add_blob(
            &store,
            config,
            caller,
            sponsor,
            AddBlobStateParams {
                hash,
                metadata_hash: new_metadata_hash(),
                id: id1.clone(),
                size,
                ttl: Some(config.blob_min_ttl),
                source,
                epoch: add2_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());
        let (sub, _) = res.unwrap();
        assert_eq!(sub.added, add1_epoch); // added should not change
        assert_eq!(sub.expiry, add2_epoch + config.blob_min_ttl);
        assert_eq!(sub.source, source);
        assert!(!sub.failed);
        if subscriber != caller {
            assert_eq!(sub.delegate, Some(caller));
        }

        // Check the blob status
        // Should already be resolved
        assert_eq!(
            state
                .get_blob_status(&store, subscriber, hash, id1.clone())
                .unwrap(),
            Some(BlobStatus::Resolved)
        );

        // Check the blob
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        let subscribers = blob.subscribers.hamt(store).unwrap();
        assert_eq!(blob.subscribers.len(), 1);
        assert_eq!(blob.status, BlobStatus::Resolved);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = subscribers.get(&subscriber).unwrap().unwrap();
        let group_hamt = group.hamt(store).unwrap();
        assert_eq!(group.len(), 1); // Still only one subscription
        let got_sub = group_hamt.get(&id1.clone()).unwrap().unwrap();
        assert_eq!(got_sub, sub);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        assert_eq!(
            account.credit_committed, // stays the same becuase we're starting over
            Credit::from_whole(config.blob_min_ttl as u64 * size),
        );
        credit_amount -= Credit::from_whole((add2_epoch - add1_epoch) as u64 * size);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size); // not changed

        // Add the same blob again but use a different subscription ID
        let add3_epoch = ChainEpoch::from(31);
        let id2 = SubscriptionId::new("foo").unwrap();
        let source = new_pk();
        let res = state.add_blob(
            &store,
            config,
            caller,
            sponsor,
            AddBlobStateParams {
                hash,
                metadata_hash: new_metadata_hash(),
                id: id2.clone(),
                size,
                ttl: Some(config.blob_min_ttl),
                source,
                epoch: add3_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());
        let (sub, _) = res.unwrap();
        assert_eq!(sub.added, add3_epoch);
        assert_eq!(sub.expiry, add3_epoch + config.blob_min_ttl);
        assert_eq!(sub.source, source);
        assert!(!sub.failed);
        if subscriber != caller {
            assert_eq!(sub.delegate, Some(caller));
        }

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Check the blob status
        // Should already be resolved
        assert_eq!(
            state
                .get_blob_status(&store, subscriber, hash, id2.clone())
                .unwrap(),
            Some(BlobStatus::Resolved)
        );

        // Check the blob
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        let subscribers = blob.subscribers.hamt(store).unwrap();
        assert_eq!(blob.subscribers.len(), 1); // still only one subscriber
        assert_eq!(blob.status, BlobStatus::Resolved);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = subscribers.get(&subscriber).unwrap().unwrap();
        let group_hamt = group.hamt(store).unwrap();
        assert_eq!(group.len(), 2);
        let got_sub = group_hamt.get(&id2.clone()).unwrap().unwrap();
        assert_eq!(got_sub, sub);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add3_epoch);
        assert_eq!(
            account.credit_committed, // stays the same becuase we're starting over
            Credit::from_whole(config.blob_min_ttl as u64 * size),
        );
        credit_amount -= Credit::from_whole((add3_epoch - add2_epoch) as u64 * size);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size); // not changed

        // Debit all accounts
        let debit_epoch = ChainEpoch::from(41);
        let (deletes_from_disc, _) = state.debit_accounts(&store, config, debit_epoch).unwrap();
        assert!(deletes_from_disc.is_empty());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // debit reduces this
            Credit::from_whole((config.blob_min_ttl - (debit_epoch - add3_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size); // not changed

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 2);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);

        // Delete the default subscription ID
        let delete_epoch = ChainEpoch::from(51);
        let res = state.delete_blob(
            &store,
            caller,
            sponsor,
            DeleteBlobStateParams {
                hash,
                id: id1.clone(),
                epoch: delete_epoch,
            },
        );

        assert!(res.is_ok());
        let (delete_from_disk, deleted_size) = res.unwrap();
        assert!(!delete_from_disk);
        assert_eq!(deleted_size, size);

        // Check the blob
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        let subscribers = blob.subscribers.hamt(store).unwrap();

        assert_eq!(blob.subscribers.len(), 1); // still one subscriber
        assert_eq!(blob.status, BlobStatus::Resolved);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = subscribers.get(&subscriber).unwrap().unwrap();
        let group_hamt = group.hamt(store).unwrap();
        assert_eq!(group.len(), 1);
        let sub = group_hamt.get(&id2.clone()).unwrap().unwrap();
        assert_eq!(sub.added, add3_epoch);
        assert_eq!(sub.expiry, add3_epoch + config.blob_min_ttl);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, delete_epoch);
        assert_eq!(
            account.credit_committed, // debit reduces this
            Credit::from_whole((config.blob_min_ttl - (delete_epoch - add3_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size); // not changed

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            (token_amount.clone() * &config.token_credit_rate)
                - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, size);

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 1);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        if using_approval {
            check_approval_used(&state, store, caller, subscriber);
        }
    }

    #[test]
    fn test_finalize_blob_from_bad_state() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
            .unwrap();

        // Add a blob
        let (hash, size) = new_hash(1024);
        let res = state.add_blob(
            &store,
            &config,
            caller,
            None,
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

        // Finalize as pending
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &store,
            caller,
            FinalizeBlobStateParams {
                hash,
                id: SubscriptionId::default(),
                status: BlobStatus::Pending,
                epoch: finalize_epoch,
            },
        );
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!("cannot finalize blob {} as added or pending", hash)
        );
    }

    #[test]
    fn test_add_blob_with_overflowing_ttl() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(1000000);
        state
            .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
            .unwrap();

        let res =
            state.set_account_status(&store, &config, caller, TtlStatus::Extended, current_epoch);
        assert!(res.is_ok());

        let (hash, size) = new_hash(1024);
        let res = state.add_blob(
            &store,
            &config,
            caller,
            None,
            AddBlobStateParams {
                hash,
                metadata_hash: new_metadata_hash(),
                id: SubscriptionId::default(),
                size,
                ttl: Some(ChainEpoch::MAX),
                source: new_pk(),
                epoch: current_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());
        let (sub, _) = res.unwrap();
        assert_eq!(sub.expiry, ChainEpoch::MAX);
    }

    #[test]
    fn test_finalize_blob_resolved() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
            .unwrap();

        // Add a blob
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &store,
            &config,
            caller,
            None,
            AddBlobStateParams {
                hash,
                metadata_hash: new_metadata_hash(),
                id: SubscriptionId::default(),
                size,
                ttl: None,
                source,
                epoch: current_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());

        // Set to status pending
        let res = state.set_blob_pending(
            &store,
            caller,
            hash,
            size,
            SubscriptionId::default(),
            source,
        );
        assert!(res.is_ok());

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &store,
            caller,
            FinalizeBlobStateParams {
                hash,
                id: SubscriptionId::default(),
                status: BlobStatus::Resolved,
                epoch: finalize_epoch,
            },
        );
        assert!(res.is_ok());

        // Check status
        let status = state
            .get_blob_status(&store, caller, hash, SubscriptionId::default())
            .unwrap()
            .unwrap();
        assert!(matches!(status, BlobStatus::Resolved));

        // Check indexes
        assert_eq!(state.expiries.len(&store).unwrap(), 1);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_finalize_blob_failed() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
            .unwrap();
        let credit_amount = amount * &config.token_credit_rate;

        // Add a blob
        let add_epoch = current_epoch;
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &store,
            &config,
            caller,
            None,
            AddBlobStateParams {
                hash,
                metadata_hash: new_metadata_hash(),
                id: SubscriptionId::default(),
                size,
                ttl: None,
                source,
                epoch: add_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());

        // Set to status pending
        let res = state.set_blob_pending(
            &store,
            caller,
            hash,
            size,
            SubscriptionId::default(),
            source,
        );
        assert!(res.is_ok());

        // Finalize as failed
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &store,
            caller,
            FinalizeBlobStateParams {
                hash,
                id: SubscriptionId::default(),
                status: BlobStatus::Failed,
                epoch: finalize_epoch,
            },
        );
        assert!(res.is_ok());

        // Check status
        let status = state
            .get_blob_status(&store, caller, hash, SubscriptionId::default())
            .unwrap()
            .unwrap();
        assert!(matches!(status, BlobStatus::Failed));

        // Check the account balance
        let account = state.get_account(&store, caller).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add_epoch);
        assert_eq!(account.credit_committed, Credit::from_whole(0)); // credit was released
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, 0); // capacity was released

        // Check state
        assert_eq!(state.credit_committed, Credit::from_whole(0)); // credit was released
        assert_eq!(state.credit_debited, Credit::from_whole(0));
        assert_eq!(state.capacity_used, 0); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(&store).unwrap(), 1); // remains until the blob is explicitly deleted
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_finalize_blob_failed_refund() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
            .unwrap();
        let mut credit_amount = amount.clone() * &config.token_credit_rate;

        assert!(state
            .set_account_status(&store, &config, caller, TtlStatus::Extended, current_epoch)
            .is_ok());

        // Add a blob
        let add_epoch = current_epoch;
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &store,
            &config,
            caller,
            None,
            AddBlobStateParams {
                hash,
                metadata_hash: new_metadata_hash(),
                id: SubscriptionId::default(),
                size,
                ttl: Some(config.blob_min_ttl),
                source,
                epoch: add_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, caller).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add_epoch);
        assert_eq!(
            account.credit_committed,
            Credit::from_whole(config.blob_min_ttl as u64 * size),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(state.credit_debited, Credit::from_whole(0));
        assert_eq!(state.capacity_used, account.capacity_used); // capacity was released

        // Debit accounts to trigger a refund when we fail below
        let debit_epoch = ChainEpoch::from(11);
        let (deletes_from_disc, _) = state.debit_accounts(&store, &config, debit_epoch).unwrap();
        assert!(deletes_from_disc.is_empty());

        // Check the account balance
        let account = state.get_account(&store, caller).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed,
            Credit::from_whole((config.blob_min_ttl - (debit_epoch - add_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            Credit::from_whole((debit_epoch - add_epoch) as u64 * size)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Set to status pending
        let res = state.set_blob_pending(
            &store,
            caller,
            hash,
            size,
            SubscriptionId::default(),
            source,
        );
        assert!(res.is_ok());

        // Finalize as failed
        let finalize_epoch = ChainEpoch::from(21);
        let res = state.finalize_blob(
            &store,
            caller,
            FinalizeBlobStateParams {
                hash,
                id: SubscriptionId::default(),
                status: BlobStatus::Failed,
                epoch: finalize_epoch,
            },
        );
        assert!(res.is_ok());

        // Check status
        let status = state
            .get_blob_status(&store, caller, hash, SubscriptionId::default())
            .unwrap()
            .unwrap();
        assert!(matches!(status, BlobStatus::Failed));

        // Check the account balance
        let account = state.get_account(&store, caller).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(account.credit_committed, Credit::from_whole(0)); // credit was released
        assert_eq!(
            account.credit_free,
            amount.clone() * &config.token_credit_rate
        ); // credit was refunded
        assert_eq!(account.capacity_used, 0); // capacity was released

        // Check state
        assert_eq!(state.credit_committed, Credit::from_whole(0)); // credit was released
        assert_eq!(state.credit_debited, Credit::from_whole(0)); // credit was refunded and released
        assert_eq!(state.capacity_used, 0); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(&store).unwrap(), 1); // remains until the blob is explicitly deleted
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_delete_blob_refund() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, &config, caller, token_amount.clone(), current_epoch)
            .unwrap();
        delete_blob_refund(
            &config,
            &store,
            state,
            caller,
            None,
            current_epoch,
            token_amount,
            false,
        );
    }

    #[test]
    fn test_delete_blob_refund_with_approval() {
        setup_logs();
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let sponsor = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &store,
                &config,
                sponsor,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(
                &config,
                &store,
                sponsor,
                caller,
                DelegationOptions::default(),
                current_epoch,
            )
            .unwrap();
        delete_blob_refund(
            &config,
            &store,
            state,
            caller,
            Some(sponsor),
            current_epoch,
            token_amount,
            true,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn delete_blob_refund<BS: Blockstore>(
        config: &RecallConfig,
        store: &BS,
        mut state: State,
        caller: Address,
        sponsor: Option<Address>,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
        using_approval: bool,
    ) {
        let subscriber = sponsor.unwrap_or(caller);
        let mut credit_amount = token_amount * &config.token_credit_rate;

        // Add a blob
        let add1_epoch = current_epoch;
        let (hash1, size1) = new_hash(1024);
        let source1 = new_pk();
        let res = state.add_blob(
            &store,
            config,
            caller,
            sponsor,
            AddBlobStateParams {
                hash: hash1,
                metadata_hash: new_metadata_hash(),
                id: SubscriptionId::default(),
                size: size1,
                ttl: Some(config.blob_min_ttl),
                source: source1,
                epoch: add1_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());

        // Finalize as resolved
        let res = state.set_blob_pending(
            &store,
            subscriber,
            hash1,
            size1,
            SubscriptionId::default(),
            source1,
        );
        assert!(res.is_ok());
        let finalize_epoch = ChainEpoch::from(current_epoch + 1);
        let res = state.finalize_blob(
            &store,
            subscriber,
            FinalizeBlobStateParams {
                hash: hash1,
                id: SubscriptionId::default(),
                status: BlobStatus::Resolved,
                epoch: finalize_epoch,
            },
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            Credit::from_whole(config.blob_min_ttl as u64 * size1),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1);

        // Add another blob past the first blob expiry
        // This will trigger a debit on the account
        let add2_epoch = ChainEpoch::from(config.blob_min_ttl + 10);
        let (hash2, size2) = new_hash(2048);
        let res = state.add_blob(
            &store,
            config,
            caller,
            sponsor,
            AddBlobStateParams {
                hash: hash2,
                metadata_hash: new_metadata_hash(),
                id: SubscriptionId::default(),
                size: size2,
                ttl: Some(config.blob_min_ttl),
                source: new_pk(),
                epoch: add2_epoch,
                token_amount: TokenAmount::zero(),
            },
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 2);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size2);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        let blob1_expiry = ChainEpoch::from(config.blob_min_ttl + add1_epoch);
        let overcharge = BigInt::from((add2_epoch - blob1_expiry) as u64 * size1);
        assert_eq!(
            account.credit_committed, // this includes an overcharge that needs to be refunded
            Credit::from_whole(config.blob_min_ttl as u64 * size2 - overcharge),
        );
        credit_amount -= Credit::from_whole(config.blob_min_ttl as u64 * size2);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1 + size2);

        // Delete the first blob
        let delete_epoch = ChainEpoch::from(config.blob_min_ttl + 20);
        let (delete_from_disc, deleted_size) = state
            .delete_blob(
                &store,
                caller,
                sponsor,
                DeleteBlobStateParams {
                    hash: hash1,
                    id: SubscriptionId::default(),
                    epoch: delete_epoch,
                },
            )
            .unwrap();
        assert!(delete_from_disc);
        assert_eq!(size1, deleted_size);

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size2);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch); // not changed, blob is expired
        assert_eq!(
            account.credit_committed, // should not include overcharge due to refund
            Credit::from_whole(config.blob_min_ttl as u64 * size2),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size2);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed); // credit was released
        assert_eq!(
            state.credit_debited,
            Credit::from_whole(config.blob_min_ttl as u64 * size1)
        );
        assert_eq!(state.capacity_used, size2); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 1);
        assert_eq!(state.added.len(), 1);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        if using_approval {
            check_approval_used(&state, store, caller, subscriber);
        }
    }

    #[test]
    fn test_if_blobs_ttl_exceeds_accounts_ttl_should_error() {
        setup_logs();

        let config = RecallConfig::default();
        const YEAR: ChainEpoch = 365 * 24 * 60 * 60;

        // Test cases structure
        struct TestCase {
            name: &'static str,
            account_ttl_status: TtlStatus,
            blob_ttl: Option<ChainEpoch>,
            should_succeed: bool,
            expected_account_ttl: ChainEpoch,
            expected_blob_ttl: ChainEpoch,
        }

        // Define test cases
        let test_cases = vec![
            TestCase {
                name: "Reduced status rejects even minimum TTL",
                account_ttl_status: TtlStatus::Reduced,
                blob_ttl: Some(config.blob_min_ttl),
                should_succeed: false,
                expected_account_ttl: 0,
                expected_blob_ttl: 0,
            },
            TestCase {
                name: "Reduced status rejects no TTL",
                account_ttl_status: TtlStatus::Reduced,
                blob_ttl: Some(config.blob_min_ttl),
                should_succeed: false,
                expected_account_ttl: 0,
                expected_blob_ttl: 0,
            },
            TestCase {
                name: "Default status allows default TTL",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: Some(config.blob_default_ttl),
                should_succeed: true,
                expected_account_ttl: config.blob_default_ttl,
                expected_blob_ttl: config.blob_default_ttl,
            },
            TestCase {
                name: "Default status sets no TTL to default without auto renew",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: None,
                should_succeed: true,
                expected_account_ttl: config.blob_default_ttl,
                expected_blob_ttl: config.blob_default_ttl,
            },
            TestCase {
                name: "Default status preserves given TTL if it's less than default",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: Some(config.blob_default_ttl - 1),
                should_succeed: true,
                expected_account_ttl: config.blob_default_ttl,
                expected_blob_ttl: config.blob_default_ttl - 1,
            },
            TestCase {
                name: "Default status rejects TTLs higher than default",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: Some(config.blob_default_ttl + 1),
                should_succeed: false,
                expected_account_ttl: config.blob_default_ttl,
                expected_blob_ttl: 0,
            },
            TestCase {
                name: "Extended status allows any TTL",
                account_ttl_status: TtlStatus::Extended,
                blob_ttl: Some(YEAR),
                should_succeed: true,
                expected_account_ttl: ChainEpoch::MAX,
                expected_blob_ttl: YEAR,
            },
        ];

        // Run all test cases
        for tc in test_cases {
            let config = RecallConfig::default();
            let store = MemoryBlockstore::default();
            let mut state = State::new(&store).unwrap();
            let caller = new_address();
            let current_epoch = ChainEpoch::from(1);
            let amount = TokenAmount::from_whole(10);

            state
                .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
                .unwrap();
            state
                .set_account_status(
                    &store,
                    &config,
                    caller,
                    tc.account_ttl_status,
                    current_epoch,
                )
                .unwrap();

            let (hash, size) = new_hash(1024);
            let res = state.add_blob(
                &store,
                &config,
                caller,
                None,
                AddBlobStateParams {
                    hash,
                    metadata_hash: new_metadata_hash(),
                    id: SubscriptionId::default(),
                    size,
                    ttl: tc.blob_ttl,
                    source: new_pk(),
                    epoch: current_epoch,
                    token_amount: TokenAmount::zero(),
                },
            );

            let account_ttl = state.get_account_max_ttl(&config, &store, caller).unwrap();
            assert_eq!(
                account_ttl, tc.expected_account_ttl,
                "Test case '{}' has unexpected account TTL (expected {}, got {})",
                tc.name, tc.expected_account_ttl, account_ttl
            );

            if tc.should_succeed {
                assert!(
                    res.is_ok(),
                    "Test case '{}' should succeed but failed: {:?}",
                    tc.name,
                    res.err()
                );

                let res = state.get_blob(&store, hash);
                assert!(res.is_ok(), "Failed to get blob: {:?}", res.err());
                let blob = res.unwrap().unwrap();
                let subscribers = blob.subscribers.hamt(&store).unwrap();
                subscribers
                    .for_each(|_, group| {
                        let group_hamt = group.hamt(&store).unwrap();
                        for val in group_hamt.iter() {
                            let (_, sub) = val.unwrap();
                            assert_eq!(
                                sub.expiry,
                                current_epoch + tc.expected_blob_ttl,
                                "Test case '{}' has unexpected blob expiry",
                                tc.name
                            );
                        }
                        Ok(())
                    })
                    .unwrap();
            } else {
                assert!(
                    res.is_err(),
                    "Test case '{}' should fail but succeeded",
                    tc.name
                );
                assert_eq!(
                    res.err().unwrap().msg(),
                    format!(
                        "attempt to add a blob with TTL ({}) that exceeds account's max allowed TTL ({})",
                        tc.blob_ttl.map_or_else(|| "none".to_string(), |ttl| ttl.to_string()), tc.account_ttl_status.get_max_ttl(config.blob_default_ttl),
                    ),
                    "Test case '{}' failed with unexpected error message",
                    tc.name
                );
            }
        }
    }

    #[test]
    fn test_set_ttl_status() {
        setup_logs();

        let config = RecallConfig::default();

        struct TestCase {
            name: &'static str,
            initial_ttl_status: Option<TtlStatus>, // None means don't set initial status
            new_ttl_status: TtlStatus,
            expected_ttl: ChainEpoch,
        }

        let test_cases = vec![
            TestCase {
                name: "Setting Reduced on new account",
                initial_ttl_status: None,
                new_ttl_status: TtlStatus::Reduced,
                expected_ttl: 0,
            },
            TestCase {
                name: "Setting Default on new account",
                initial_ttl_status: None,
                new_ttl_status: TtlStatus::Default,
                expected_ttl: config.blob_default_ttl,
            },
            TestCase {
                name: "Changing from Default to Reduced",
                initial_ttl_status: Some(TtlStatus::Default),
                new_ttl_status: TtlStatus::Reduced,
                expected_ttl: 0,
            },
            TestCase {
                name: "Changing from Extended to Reduced",
                initial_ttl_status: Some(TtlStatus::Extended),
                new_ttl_status: TtlStatus::Reduced,
                expected_ttl: 0,
            },
            TestCase {
                name: "Changing from Reduced to Extended",
                initial_ttl_status: Some(TtlStatus::Reduced),
                new_ttl_status: TtlStatus::Extended,
                expected_ttl: ChainEpoch::MAX,
            },
        ];

        for tc in test_cases {
            let store = MemoryBlockstore::default();
            let mut state = State::new(&store).unwrap();
            let address = new_address();
            let current_epoch = ChainEpoch::from(1);

            // Initialize the account if needed
            if tc.initial_ttl_status.is_some() {
                state
                    .set_account_status(
                        &store,
                        &config,
                        address,
                        tc.initial_ttl_status.unwrap(),
                        current_epoch,
                    )
                    .unwrap();
            }

            // Change TTL status
            let res = state.set_account_status(
                &store,
                &config,
                address,
                tc.new_ttl_status,
                current_epoch,
            );
            assert!(
                res.is_ok(),
                "Test case '{}' failed to set TTL status",
                tc.name
            );

            // Verify max TTL
            let max_ttl = state.get_account_max_ttl(&config, &store, address).unwrap();
            assert_eq!(
                max_ttl, tc.expected_ttl,
                "Test case '{}' failed: expected max TTL {}, got {}",
                tc.name, tc.expected_ttl, max_ttl
            );
        }
    }

    #[test]
    fn test_adjust_blob_ttls_for_account() {
        setup_logs();
        let config = RecallConfig::default();

        const HOUR: ChainEpoch = 3600;
        const TWO_HOURS: ChainEpoch = HOUR * 2;
        const DAY: ChainEpoch = HOUR * 24;
        const YEAR: ChainEpoch = DAY * 365;

        let blobs_ttls: Vec<Option<ChainEpoch>> =
            vec![None, Some(HOUR), Some(TWO_HOURS), Some(DAY), Some(YEAR)];

        struct TestCase {
            name: &'static str,
            account_ttl: TtlStatus,
            expected_ttls: Vec<ChainEpoch>,
            limit: Option<usize>, // None means process all at once
        }

        let test_cases = vec![
            TestCase {
                name: "Set to zero with Reduced status",
                account_ttl: TtlStatus::Reduced,
                expected_ttls: vec![0, 0, 0, 0, 0],
                limit: None,
            },
            TestCase {
                name: "Set to default with Default status",
                account_ttl: TtlStatus::Default,
                expected_ttls: vec![DAY, HOUR, TWO_HOURS, DAY, DAY],
                limit: None,
            },
            TestCase {
                name: "Set to extended with Extended status",
                account_ttl: TtlStatus::Extended,
                expected_ttls: vec![DAY, HOUR, TWO_HOURS, DAY, YEAR],
                limit: None,
            },
        ];

        for tc in test_cases {
            let store = MemoryBlockstore::default();
            let mut state = State::new(&store).unwrap();
            let caller = new_address();
            let current_epoch = ChainEpoch::from(1);

            // Setup account with credits and TTL status
            let token = TokenAmount::from_whole(1000);
            state
                .buy_credit(&store, &config, caller, token, current_epoch)
                .unwrap();

            // Set extended TTL status to allow adding all blobs
            state
                .set_account_status(&store, &config, caller, TtlStatus::Extended, current_epoch)
                .unwrap();

            // Add blobs
            let mut blob_hashes = Vec::new();
            let mut total_cost = Credit::zero();
            let mut expected_credits = Credit::zero();
            for (i, ttl) in blobs_ttls.iter().enumerate() {
                let size = (i + 1) * 1024;
                let (hash, _) = new_hash(size);
                let size = size as u64;
                let id = SubscriptionId::try_from(format!("blob-{}", i)).unwrap();
                let source = new_pk();
                blob_hashes.push(hash);

                state
                    .add_blob(
                        &store,
                        &config,
                        caller,
                        None,
                        AddBlobStateParams {
                            hash,
                            metadata_hash: new_metadata_hash(),
                            id: id.clone(),
                            size,
                            ttl: *ttl,
                            source,
                            epoch: current_epoch,
                            token_amount: TokenAmount::zero(),
                        },
                    )
                    .unwrap();
                state
                    .set_blob_pending(&store, caller, hash, size, id.clone(), source)
                    .unwrap();
                state
                    .finalize_blob(
                        &store,
                        caller,
                        FinalizeBlobStateParams {
                            hash,
                            id,
                            status: BlobStatus::Resolved,
                            epoch: current_epoch,
                        },
                    )
                    .unwrap();

                total_cost += Credit::from_whole(
                    state.get_storage_cost(ttl.unwrap_or(config.blob_default_ttl), &size),
                );
                expected_credits +=
                    Credit::from_whole(state.get_storage_cost(tc.expected_ttls[i], &size));
            }

            let account = state.get_account(&store, caller).unwrap().unwrap();
            assert_eq!(
                account.credit_committed, total_cost,
                "Test case '{}' failed: committed credits don't match",
                tc.name
            );

            state
                .set_account_status(&store, &config, caller, tc.account_ttl, current_epoch)
                .unwrap();

            let res =
                state.trim_blob_expiries(&config, &store, caller, current_epoch, None, tc.limit);
            assert!(
                res.is_ok(),
                "Test case '{}' failed to adjust TTLs: {}",
                tc.name,
                res.err().unwrap()
            );

            // Verify TTLs were adjusted correctly
            for (i, hash) in blob_hashes.iter().enumerate() {
                // If the TTL is zero, the blob should be deleted
                if tc.expected_ttls[i] == 0 {
                    assert!(
                        state.get_blob(&store, *hash).unwrap().is_none(),
                        "Test case '{}' failed: blob {} not deleted",
                        tc.name,
                        i
                    );
                } else {
                    let blob = state.get_blob(&store, *hash).unwrap().unwrap();
                    let subscribers = blob.subscribers.hamt(&store).unwrap();
                    let group = subscribers.get(&caller).unwrap().unwrap();
                    let group_hamt = group.hamt(&store).unwrap();
                    let sub = group_hamt
                        .get(&SubscriptionId::new(&format!("blob-{}", i)).unwrap())
                        .unwrap()
                        .unwrap();

                    assert_eq!(
                        sub.expiry - sub.added,
                        tc.expected_ttls[i],
                        "Test case '{}' failed: blob {} TTL not adjusted correctly. Expected {}, got {}",
                        tc.name,
                        i,
                        tc.expected_ttls[i],
                        sub.expiry - sub.added,
                    );
                }
            }

            let account = state.get_account(&store, caller).unwrap().unwrap();
            assert_eq!(
                account.credit_committed, expected_credits,
                "Test case '{}' failed: account's committed credits after blob adjustment don't match",
                tc.name
            );

            assert_eq!(
                state.credit_committed, expected_credits,
                "Test case '{}' failed: state's committed credits after blob adjustment don't match",
                tc.name
            );
        }
    }

    #[test]
    fn test_adjust_blob_ttls_pagination() {
        setup_logs();
        let config = RecallConfig::default();

        // Test cases for pagination
        struct PaginationTest {
            name: &'static str,
            limit: Option<usize>,
            start: Option<usize>,
            expected_next_key: Option<usize>,
            expected_processed: usize,
        }

        let test_cases = vec![
            PaginationTest {
                name: "Process all at once",
                limit: None,
                start: None,
                expected_next_key: None,
                expected_processed: 5,
            },
            PaginationTest {
                name: "Process two at a time from beginning",
                limit: Some(2),
                start: None,
                expected_next_key: Some(2),
                expected_processed: 2,
            },
            PaginationTest {
                name: "Process one at a time with offset",
                limit: Some(1),
                start: Some(1),
                expected_next_key: Some(2),
                expected_processed: 1,
            },
            PaginationTest {
                name: "Out of bounds limit",
                limit: Some(10),
                start: Some(1),
                expected_next_key: None,
                expected_processed: 4,
            },
            PaginationTest {
                name: "With offset ending at last item",
                limit: Some(2),
                start: Some(3),
                expected_next_key: None,
                expected_processed: 2,
            },
        ];

        for tc in test_cases {
            let store = MemoryBlockstore::default();
            let mut state = State::new(&store).unwrap();
            let caller = new_address();
            let current_epoch = ChainEpoch::from(1);

            // Setup account with credits and Extended TTL status to allow adding all blobs
            state
                .buy_credit(
                    &store,
                    &config,
                    caller,
                    TokenAmount::from_whole(1000),
                    current_epoch,
                )
                .unwrap();
            state
                .set_account_status(&store, &config, caller, TtlStatus::Extended, current_epoch)
                .unwrap();

            // Add 5 blobs with different sizes to ensure different hashes
            for i in 0..5 {
                let (hash, size) = new_hash((i + 1) * 1024);
                let id = SubscriptionId::try_from(format!("blob-{}", i)).unwrap();
                let source = new_pk();
                state
                    .add_blob(
                        &store,
                        &config,
                        caller,
                        None,
                        AddBlobStateParams {
                            hash,
                            metadata_hash: new_metadata_hash(),
                            id: id.clone(),
                            size,
                            ttl: Some(7200), // 2 hours
                            source,
                            epoch: current_epoch,
                            token_amount: TokenAmount::zero(),
                        },
                    )
                    .unwrap();
                state
                    .set_blob_pending(&store, caller, hash, size, id.clone(), source)
                    .unwrap();
                state
                    .finalize_blob(
                        &store,
                        caller,
                        FinalizeBlobStateParams {
                            hash,
                            id,
                            status: BlobStatus::Resolved,
                            epoch: current_epoch,
                        },
                    )
                    .unwrap();
            }

            // Range over all blobs and store their hashes
            let mut blob_hashes = Vec::with_capacity(5);
            for _ in 0..5 {
                let res = state.blobs.hamt(&store).unwrap().for_each(
                    |hash, _| -> Result<(), ActorError> {
                        blob_hashes.push(hash);
                        Ok(())
                    },
                );
                assert!(
                    res.is_ok(),
                    "Failed to iterate over blobs: {}",
                    res.err().unwrap()
                );
            }

            // Change to Reduced status and process blobs with pagination
            state
                .set_account_status(&store, &config, caller, TtlStatus::Reduced, current_epoch)
                .unwrap();

            let res = state.trim_blob_expiries(
                &config,
                &store,
                caller,
                current_epoch,
                tc.start.map(|ind| blob_hashes[ind]),
                tc.limit,
            );
            assert!(
                res.is_ok(),
                "Test case '{}' failed to adjust TTLs: {}",
                tc.name,
                res.err().unwrap()
            );

            let (processed, next, deleted_blobs) = res.unwrap();

            assert_eq!(
                processed as usize, tc.expected_processed,
                "Test case '{}' had unexpected number of items processed",
                tc.name
            );

            assert_eq!(
                deleted_blobs.len(),
                tc.expected_processed,
                "Test case '{}' had unexpected number of deleted blobs",
                tc.name
            );

            if let Some(expected_next_key) = tc.expected_next_key {
                assert!(next.is_some(), "Test case '{}' expected next key", tc.name);
                assert_eq!(
                    next.unwrap(),
                    blob_hashes[expected_next_key],
                    "Test case '{}' had unexpected next key",
                    tc.name
                );
            } else {
                assert!(next.is_none(), "Test case '{}' had no next key", tc.name);
            }
        }
    }

    #[test]
    fn test_adjust_blob_ttls_for_multiple_accounts() {
        setup_logs();

        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let address1 = new_address();
        let address2 = new_address();
        let current_epoch = ChainEpoch::from(1);

        // Setup accounts with credits and Extended TTL status to allow adding all blobs
        state
            .buy_credit(
                &store,
                &config,
                address1,
                TokenAmount::from_whole(1000),
                current_epoch,
            )
            .unwrap();
        state
            .buy_credit(
                &store,
                &config,
                address2,
                TokenAmount::from_whole(1000),
                current_epoch,
            )
            .unwrap();
        state
            .set_account_status(
                &store,
                &config,
                address1,
                TtlStatus::Extended,
                current_epoch,
            )
            .unwrap();
        state
            .set_account_status(
                &store,
                &config,
                address2,
                TtlStatus::Extended,
                current_epoch,
            )
            .unwrap();

        // Add blobs for both accounts
        let mut blob_hashes_account1 = Vec::new();
        let mut blob_hashes_account2 = Vec::new();
        for i in 0..3 {
            let (hash, size) = new_hash((i + 1) * 1024);
            let id = SubscriptionId::try_from(format!("blob-1-{}", i)).unwrap();
            let source = new_pk();
            blob_hashes_account1.push(hash);
            state
                .add_blob(
                    &store,
                    &config,
                    address1,
                    None,
                    AddBlobStateParams {
                        hash,
                        metadata_hash: new_metadata_hash(),
                        id: id.clone(),
                        size,
                        ttl: Some(7200), // 2 hours
                        source,
                        epoch: current_epoch,
                        token_amount: TokenAmount::zero(),
                    },
                )
                .unwrap();
            state
                .set_blob_pending(&store, address1, hash, size, id.clone(), source)
                .unwrap();
            state
                .finalize_blob(
                    &store,
                    address1,
                    FinalizeBlobStateParams {
                        hash,
                        id,
                        status: BlobStatus::Resolved,
                        epoch: current_epoch,
                    },
                )
                .unwrap();
        }
        for i in 0..3 {
            let (hash, size) = new_hash((i + 1) * 1024);
            let id = SubscriptionId::try_from(format!("blob-2-{}", i)).unwrap();
            let source = new_pk();
            blob_hashes_account2.push(hash);
            state
                .add_blob(
                    &store,
                    &config,
                    address2,
                    None,
                    AddBlobStateParams {
                        hash,
                        metadata_hash: new_metadata_hash(),
                        id: id.clone(),
                        size,
                        ttl: Some(7200), // 2 hours
                        source,
                        epoch: current_epoch,
                        token_amount: TokenAmount::zero(),
                    },
                )
                .unwrap();
            state
                .set_blob_pending(&store, address2, hash, size, id.clone(), source)
                .unwrap();
            state
                .finalize_blob(
                    &store,
                    address2,
                    FinalizeBlobStateParams {
                        hash,
                        id,
                        status: BlobStatus::Resolved,
                        epoch: current_epoch,
                    },
                )
                .unwrap();
        }

        // Change TTL status for account1 and adjust blobs
        state
            .set_account_status(&store, &config, address1, TtlStatus::Reduced, current_epoch)
            .unwrap();
        let res = state.trim_blob_expiries(&config, &store, address1, current_epoch, None, None);
        assert!(
            res.is_ok(),
            "Failed to adjust TTLs for account1: {}",
            res.err().unwrap()
        );

        // Verify account1's blobs were adjusted
        for hash in &blob_hashes_account1 {
            assert!(
                state.get_blob(&store, *hash).unwrap().is_none(),
                "Blob {} for account1 was not deleted",
                hash,
            );
        }

        // Verify account2's blobs were not adjusted
        for hash in &blob_hashes_account2 {
            assert!(
                state.get_blob(&store, *hash).unwrap().is_some(),
                "Blob {} for account2 was incorrectly deleted",
                hash,
            );
        }
    }

    #[test]
    fn test_simulate_one_day() {
        setup_logs();

        let config = RecallConfig {
            blob_credit_debit_interval: ChainEpoch::from(60),
            blob_min_ttl: ChainEpoch::from(10),
            ..Default::default()
        };

        #[derive(Clone, Debug, Hash, PartialEq, Eq)]
        struct TestBlob {
            hash: Hash,
            metadata_hash: Hash,
            size: u64,
            added: Option<ChainEpoch>,
            resolve: Option<ChainEpoch>,
        }

        fn generate_test_blobs(count: i64, min_size: usize, max_size: usize) -> Vec<TestBlob> {
            let mut blobs = Vec::new();
            let mut rng = rand::thread_rng();

            for _ in 0..count {
                let size = rng.gen_range(min_size..=max_size);
                let (hash, size) = new_hash(size);
                blobs.push(TestBlob {
                    hash,
                    metadata_hash: new_metadata_hash(),
                    size,
                    added: None,
                    resolve: None,
                });
            }
            blobs
        }

        fn generate_test_users<BS: Blockstore>(
            config: &RecallConfig,
            store: &BS,
            state: &mut State,
            credit_tokens: TokenAmount,
            count: i64,
        ) -> Vec<Address> {
            let mut users = Vec::new();
            for _ in 0..count {
                let user = new_address();
                state
                    .buy_credit(&store, config, user, credit_tokens.clone(), 0)
                    .unwrap();
                users.push(user);
            }
            users
        }

        // Test params
        let epochs: i64 = 360; // num. epochs to run test for
        let user_pool_size: i64 = 10; // some may not be used, some will be used more than once
        let blob_pool_size: i64 = epochs; // some may not be used, some will be used more than once
        let min_ttl = config.blob_min_ttl;
        let max_ttl = epochs;
        let min_size = 8;
        let max_size = 1024;
        let add_intervals = [1, 2, 4, 8, 10, 12, 15, 20]; // used to add at random intervals
        let max_resolve_epochs = 30; // max num. epochs in future to resolve
        let debit_interval: i64 = config.blob_credit_debit_interval; // interval at which to debit all accounts
        let percent_fail_resolve = 0.1; // controls % of subscriptions that fail resolve

        // Set up store and state
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let mut rng = rand::thread_rng();

        // Get some users
        let credit_tokens = TokenAmount::from_whole(100); // buy a lot
        let user_credit: Credit = credit_tokens.clone() * &config.token_credit_rate;
        let users = generate_test_users(&config, &store, &mut state, credit_tokens, user_pool_size);

        // Get some blobs.
        let mut blobs = generate_test_blobs(blob_pool_size, min_size, max_size);

        // Map of resolve epochs to set of blob indexes
        #[allow(clippy::type_complexity)]
        let mut resolves: BTreeMap<
            ChainEpoch,
            HashMap<Address, HashMap<usize, (SubscriptionId, PublicKey, Credit)>>,
        > = BTreeMap::new();

        // Walk epochs.
        // We go for twice the paramaterized epochs to ensure all subscriptions can expire.
        let mut num_added = 0;
        let mut num_readded = 0;
        let mut num_resolved = 0;
        let mut num_failed = 0;
        let mut credit_used: HashMap<Address, Credit> = HashMap::new();
        for epoch in 1..=epochs * 2 {
            if epoch <= epochs {
                let add_interval = add_intervals.choose(&mut rng).unwrap().to_owned();
                if epoch % add_interval == 0 {
                    // Add a random blob with a random user
                    let blob_index = rng.gen_range(0..blobs.len());
                    let blob = unsafe { blobs.get_unchecked_mut(blob_index) };
                    if blob.added.is_none() {
                        let user_index = rng.gen_range(0..users.len());
                        let user = users[user_index];
                        let sub_id = new_subscription_id(7);
                        let ttl = rng.gen_range(min_ttl..=max_ttl);
                        let source = new_pk();
                        let res = state.add_blob(
                            &store,
                            &config,
                            user,
                            None,
                            AddBlobStateParams {
                                hash: blob.hash,
                                metadata_hash: blob.metadata_hash,
                                id: sub_id.clone(),
                                size: blob.size,
                                ttl: Some(ttl),
                                source,
                                epoch,
                                token_amount: TokenAmount::zero(),
                            },
                        );
                        assert!(res.is_ok());
                        if blob.added.is_none() {
                            num_added += 1;
                            warn!(
                                "added new blob {} at epoch {} with ttl {}",
                                blob.hash, epoch, ttl
                            );
                        } else {
                            warn!(
                                "added new sub to blob {} at epoch {} with ttl {}",
                                blob.hash, epoch, ttl
                            );
                            num_readded += 1;
                        }
                        blob.added = Some(epoch);

                        // Determine how much credit should get committed for this blob
                        let credit = Credit::from_whole(state.get_storage_cost(ttl, &blob.size));
                        // Track credit amount for user, assuming the whole committed amount gets debited
                        credit_used
                            .entry(user)
                            .and_modify(|c| c.add_assign(&credit))
                            .or_insert(credit.clone());

                        // Schedule a resolve to happen in the future
                        let resolve = rng.gen_range(1..=max_resolve_epochs) + epoch;
                        resolves
                            .entry(resolve)
                            .and_modify(|entry| {
                                entry
                                    .entry(user)
                                    .and_modify(|subs| {
                                        subs.insert(
                                            blob_index,
                                            (sub_id.clone(), source, credit.clone()),
                                        );
                                    })
                                    .or_insert(HashMap::from([(
                                        blob_index,
                                        (sub_id.clone(), source, credit.clone()),
                                    )]));
                            })
                            .or_insert(HashMap::from([(
                                user,
                                HashMap::from([(blob_index, (sub_id, source, credit))]),
                            )]));
                    }
                }
            }

            // Resolve blob(s)
            if let Some(users) = resolves.get(&epoch) {
                for (user, index) in users {
                    for (i, (sub_id, source, credit)) in index {
                        let blob = unsafe { blobs.get_unchecked(*i) };
                        let fail = rng.gen_bool(percent_fail_resolve);
                        let status = if fail {
                            num_failed += 1;
                            credit_used
                                .entry(*user)
                                .and_modify(|c| c.sub_assign(credit));
                            BlobStatus::Failed
                        } else {
                            num_resolved += 1;
                            BlobStatus::Resolved
                        };
                        // Simulate the chain putting this blob into pending state, which is
                        // required before finalization.
                        state
                            .set_blob_pending(
                                &store,
                                *user,
                                blob.hash,
                                blob.size,
                                sub_id.clone(),
                                *source,
                            )
                            .unwrap();
                        state
                            .finalize_blob(
                                &store,
                                *user,
                                FinalizeBlobStateParams {
                                    hash: blob.hash,
                                    id: sub_id.clone(),
                                    status,
                                    epoch,
                                },
                            )
                            .unwrap();
                    }
                }
            }

            // Every debit interval epochs we debit all acounts
            if epoch % debit_interval == 0 {
                let (deletes_from_disc, _) = state.debit_accounts(&store, &config, epoch).unwrap();
                warn!(
                    "deleting {} blobs at epoch {}",
                    deletes_from_disc.len(),
                    epoch
                );
            }
        }

        let mut total_credit_used = Credit::zero();
        for (_, credit) in credit_used.clone() {
            total_credit_used.add_assign(&credit);
        }

        debug!("credit used: {}", total_credit_used);
        debug!("num. blobs added: {}", num_added);
        debug!("num. blobs re-added: {}", num_readded);
        debug!("num. blobs resolved: {}", num_resolved);
        debug!("num. blobs failed: {}", num_failed);

        // Check the account balances
        for (i, user) in users.iter().enumerate() {
            let account = state.get_account(&store, *user).unwrap().unwrap();
            debug!("account {}: {:#?}", i, account);
            assert_eq!(account.capacity_used, 0);
            assert_eq!(account.credit_committed, Credit::zero());
            let credit_used = credit_used.get(user).unwrap();
            assert_eq!(account.credit_free, &user_credit - credit_used);
        }

        // Check state.
        // Everything should be empty except for credit_debited.
        let stats = state.get_stats(&config, TokenAmount::zero());
        debug!("stats: {:#?}", stats);
        assert_eq!(stats.capacity_used, 0);
        assert_eq!(stats.credit_committed, Credit::zero());
        assert_eq!(stats.credit_debited, total_credit_used);
        assert_eq!(stats.num_blobs, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
    }
}
