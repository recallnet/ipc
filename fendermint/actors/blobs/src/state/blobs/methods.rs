// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::error::Error;
use std::str::from_utf8;

use fendermint_actor_blobs_shared::{
    blobs::{BlobRequest, BlobStatus, Subscription, SubscriptionId},
    bytes::B256,
    credit::Credit,
};
use fendermint_actor_recall_config_shared::RecallConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, bigint::BigInt, clock::ChainEpoch, econ::TokenAmount};
use log::debug;
use num_traits::Zero;
use recall_ipld::hamt::BytesKey;

use super::{
    AddBlobStateParams, Blob, BlobSource, DeleteBlobStateParams, ExpiryUpdate,
    FinalizeBlobStateParams,
};
use crate::{caller::Caller, state::credit::CommitCapacityParams, State};

/// Return type for blob queues.
type BlobSourcesResult = Result<Vec<BlobRequest>, ActorError>;

impl State {
    /// Adds or updates a blob subscription.
    ///
    /// This method handles the entire process of adding a new blob or updating an existing
    /// blob subscription, including
    /// - Managing subscriber and sponsorship relationships
    /// - Handling blob creation or update
    /// - Processing subscription groups and expiry tracking
    /// - Managing capacity accounting and credit commitments
    /// - Updating blob status and indexing
    ///
    /// Flushes state to the blockstore.
    pub fn add_blob<BS: Blockstore>(
        &mut self,
        store: &BS,
        config: &RecallConfig,
        caller: Address,
        sponsor: Option<Address>,
        params: AddBlobStateParams,
    ) -> Result<(Subscription, TokenAmount), ActorError> {
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
        let expiry = i64::saturating_add(params.epoch, ttl);

        // Get or create a new blob
        let result = self.blobs.upsert(store, &caller, &params, expiry)?;

        // Determine credit commitments
        let credit_return = self.get_storage_cost(result.return_duration, &params.size);
        self.return_committed_credit_for_caller(&mut caller, &credit_return);
        let credit_required = self.get_storage_cost(result.commit_duration, &params.size);

        // Account capacity is changing, debit for existing usage
        self.debit_caller(&mut caller, params.epoch);

        // Account for new size and commit credit
        let token_rebate = if credit_required.is_positive() {
            self.commit_capacity_for_caller(
                &mut caller,
                config,
                CommitCapacityParams {
                    subnet_size: result.new_subnet_capacity,
                    caller_size: result.new_caller_capacity,
                    cost: credit_required,
                    value: params.token_amount,
                    epoch: params.epoch,
                },
            )?
        } else {
            self.release_capacity_for_caller(
                &mut caller,
                result.new_subnet_capacity,
                result.new_caller_capacity,
                &-credit_required,
            );
            TokenAmount::zero()
        };

        // Save caller
        self.save_caller(&mut caller, &mut accounts)?;

        Ok((result.subscription, token_rebate))
    }

    /// Retuns a [`Blob`] by hash.
    pub fn get_blob<BS: Blockstore>(
        &self,
        store: &BS,
        hash: B256,
    ) -> Result<Option<Blob>, ActorError> {
        let blobs = self.blobs.hamt(store)?;
        blobs.get(&hash)
    }

    /// Returns [`BlobStatus`] by hash.
    pub fn get_blob_status<BS: Blockstore>(
        &self,
        store: &BS,
        subscriber: Address,
        hash: B256,
        id: SubscriptionId,
    ) -> Result<Option<BlobStatus>, ActorError> {
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

    /// Retrieves a page of newly added blobs that need to be resolved.
    ///
    /// This method fetches blobs from the "added" queue, which contains blobs that have been
    /// added to the system but haven't yet been successfully resolved and stored.
    pub fn get_added_blobs<BS: Blockstore>(&self, store: &BS, size: u32) -> BlobSourcesResult {
        let blobs = self.blobs.hamt(store)?;
        self.blobs
            .added
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

    /// Retrieves a page of blobs that are pending resolve.
    ///
    /// This method fetches blobs from the "pending" queue, which contains blobs that are
    /// actively being resolved but are still in a pending state.
    pub fn get_pending_blobs<BS: Blockstore>(&self, store: &BS, size: u32) -> BlobSourcesResult {
        let blobs = self.blobs.hamt(store)?;
        self.blobs
            .pending
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

    /// Marks a blob as pending resolution.
    ///
    /// This method transitions a blob from 'added' to 'pending' state, indicating that its
    /// resolution process has started. It updates the blob's status and moves it from the
    /// 'added' queue to the 'pending' queue.
    ///
    /// Flushes state to the blockstore.
    pub fn set_blob_pending<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        hash: B256,
        size: u64,
        id: SubscriptionId,
        source: B256,
    ) -> Result<(), ActorError> {
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
        self.blobs.pending.upsert(
            store,
            hash,
            BlobSource::new(subscriber, id, source),
            blob.size,
        )?;

        // Remove entire blob entry from the added queue
        self.blobs.added.remove_entry(store, &hash, blob.size)?;

        // Save blob
        self.blobs
            .save_tracked(blobs.set_and_flush_tracked(&hash, blob)?);
        Ok(())
    }

    /// Finalizes a blob's resolution process with a success or failure status.
    ///
    /// This method completes the blob resolution process by setting its final status
    /// (resolved or failed). For failed blobs, it handles refunding of credits and capacity
    /// reclamation as needed. The method also removes the blob from the pending queue.
    ///
    /// Flushes state to the blockstore.
    pub fn finalize_blob<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        params: FinalizeBlobStateParams,
    ) -> Result<(), ActorError> {
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
                let refund_credits = self.get_storage_cost(cutoff - sub.added, &size);
                let correction_credits = if cutoff > group_expiry {
                    self.get_storage_cost(cutoff - group_expiry, &size)
                } else {
                    Credit::zero()
                };
                self.refund_caller(&mut caller, &refund_credits, &correction_credits);
            }

            // If there's no new group expiry, all subscriptions have failed.
            let reclaim_capacity = if new_group_expiry.is_none() { size } else { 0 };

            // Release credits considering other subscriptions may still be pending.
            let reclaim_credits = if last_debit_epoch < group_expiry {
                self.get_storage_cost(
                    group_expiry
                        - new_group_expiry.map_or(last_debit_epoch, |e| e.max(last_debit_epoch)),
                    &size,
                )
            } else {
                Credit::zero()
            };
            self.release_capacity_for_caller(
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
        self.blobs.pending.remove_source(
            store,
            params.hash,
            BlobSource::new(subscriber, params.id, sub.source),
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

    /// Deletes a blob subscription or the entire blob if it has no remaining subscriptions.
    ///
    /// This method handles the process of deleting a blob subscription for a specific caller,
    /// which may include:
    /// - Removing the caller's subscription from the blob's subscriber list
    /// - Refunding unused storage credits to the subscriber
    /// - Releasing committed capacity from the subscriber's account
    /// - Removing the blob entirely if no subscriptions remain
    /// - Cleaning up related queue entries and indexes
    ///
    /// Flushes state to the blockstore.
    pub fn delete_blob<BS: Blockstore>(
        &mut self,
        store: &BS,
        caller: Address,
        sponsor: Option<Address>,
        params: DeleteBlobStateParams,
    ) -> Result<(bool, u64), ActorError> {
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
                self.debit_caller(&mut caller, debit_epoch);
            } else if last_debit_epoch != debit_epoch {
                // The account was debited after this blob's expiry
                // Return over-debited credit
                let return_credits =
                    self.get_storage_cost(last_debit_epoch - group_expiry, &blob.size);
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
                        self.get_storage_cost(group_expiry - reclaim_duration_start, &size)
                    } else {
                        Credit::zero()
                    }
                })
                .unwrap_or_default();
            self.release_capacity_for_caller(
                &mut caller,
                reclaim_subnet_capacity,
                reclaim_caller_capacity,
                &reclaim_credits,
            );
        }

        // Update expiry index
        self.blobs.expiries.update(
            store,
            caller.subscriber_address(),
            params.hash,
            &params.id,
            vec![ExpiryUpdate::Remove(sub.expiry)],
        )?;

        // Remove the source from the added queue
        self.blobs.added.remove_source(
            store,
            params.hash,
            BlobSource::new(caller.subscriber_address(), params.id.clone(), sub.source),
            size,
        )?;

        // Remove the source from the pending queue
        self.blobs.pending.remove_source(
            store,
            params.hash,
            BlobSource::new(caller.subscriber_address(), params.id.clone(), sub.source),
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
    ///
    /// Returns the number of subscriptions processed and the next key to continue iteration.
    /// If `starting_hash` is `None`, iteration starts from the beginning.
    /// If `limit` is `None`, all subscriptions are processed.
    /// If `limit` is not `None`, iteration stops after examining `limit` blobs.
    ///
    /// Flushes state to the blockstore.
    pub fn trim_blob_expiries<BS: Blockstore>(
        &mut self,
        config: &RecallConfig,
        store: &BS,
        subscriber: Address,
        current_epoch: ChainEpoch,
        starting_hash: Option<B256>,
        limit: Option<u32>,
    ) -> Result<(u32, Option<B256>, Vec<B256>), ActorError> {
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

        // Walk blobs
        let (_, next_key) = blobs.for_each_ranged(
            starting_key.as_ref(),
            limit.map(|l| l as usize),
            |hash, blob| -> Result<bool, ActorError> {
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
                Ok(true)
            },
        )?;

        Ok((processed, next_key, deleted_blobs))
    }

    /// Returns an error if the subnet storage is at capacity.
    pub(crate) fn ensure_capacity(&self, capacity: u64) -> Result<(), ActorError> {
        if self.capacity_available(capacity).is_zero() {
            return Err(ActorError::forbidden(
                "subnet has reached storage capacity".into(),
            ));
        }
        Ok(())
    }

    /// Return available capacity as a difference between `blob_capacity_total` and `capacity_used`.
    pub(crate) fn capacity_available(&self, blob_capacity_total: u64) -> u64 {
        // Prevent underflow. We only care if free capacity is > 0 anyway.
        blob_capacity_total.saturating_sub(self.blobs.bytes_size)
    }

    /// Returns the [`Credit`] storage cost for the given duration and size.
    pub(crate) fn get_storage_cost(&self, duration: i64, size: &u64) -> Credit {
        Credit::from_whole(duration * BigInt::from(*size))
    }

    /// Returns the current [`Credit`] debit amount based on the caller's current capacity used
    /// and the given duration.
    pub(crate) fn get_debit_for_caller<BS: Blockstore>(
        &self,
        caller: &Caller<BS>,
        epoch: ChainEpoch,
    ) -> Credit {
        let debit_duration = epoch.saturating_sub(caller.subscriber().last_debit_epoch);
        Credit::from_whole(BigInt::from(caller.subscriber().capacity_used) * debit_duration)
    }

    /// Returns an account's current max allowed blob TTL by address.
    pub(crate) fn get_account_max_ttl<BS: Blockstore>(
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
