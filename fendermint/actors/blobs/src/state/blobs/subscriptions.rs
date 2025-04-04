// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::str::from_utf8;

use fendermint_actor_blobs_shared::blobs::{Subscription, SubscriptionId};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::clock::ChainEpoch;
use recall_ipld::{hamt, hamt::map::TrackedFlushResult};

/// HAMT wrapper tracking blob [`Subscription`]s by subscription ID.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Subscriptions {
    pub root: hamt::Root<SubscriptionId, Subscription>,
    size: u64,
}

impl Subscriptions {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<SubscriptionId, Subscription>::new(store, "subscription_group")?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, SubscriptionId, Subscription>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(
        &mut self,
        tracked_flush_result: TrackedFlushResult<SubscriptionId, Subscription>,
    ) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    pub fn len(&self) -> u64 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns the current group max expiry and the group max expiry after adding the provided ID
    /// and new value.
    pub fn max_expiries<BS: Blockstore>(
        &self,
        store: &BS,
        target_id: &SubscriptionId,
        new_value: Option<ChainEpoch>,
    ) -> Result<(Option<ChainEpoch>, Option<ChainEpoch>), ActorError> {
        let mut max = None;
        let mut new_max = None;
        let subscriptions = self.hamt(store)?;
        for val in subscriptions.iter() {
            let (id, sub) = deserialize_iter_sub(val)?;
            if sub.failed {
                continue;
            }
            if sub.expiry > max.unwrap_or(0) {
                max = Some(sub.expiry);
            }
            let new_value = if &id == target_id {
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
        Ok((max, new_max))
    }

    /// Returns whether the provided ID corresponds to a subscription that has the minimum
    /// added epoch and the next minimum added epoch in the group.
    pub fn is_min_added<BS: Blockstore>(
        &self,
        store: &BS,
        trim_id: &SubscriptionId,
    ) -> anyhow::Result<(bool, Option<ChainEpoch>), ActorError> {
        let subscriptions = self.hamt(store)?;
        let trim = subscriptions
            .get(trim_id)?
            .ok_or(ActorError::not_found(format!(
                "subscription id {} not found",
                trim_id
            )))?;

        let mut next_min = None;
        for val in subscriptions.iter() {
            let (id, sub) = deserialize_iter_sub(val)?;
            if sub.failed || &id == trim_id {
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

fn deserialize_iter_sub<'a>(
    val: Result<(&hamt::BytesKey, &'a Subscription), hamt::Error>,
) -> Result<(SubscriptionId, &'a Subscription), ActorError> {
    let (id_bytes, sub) = val.map_err(|e| {
        ActorError::illegal_state(format!(
            "failed to deserialize subscription from iter: {}",
            e
        ))
    })?;
    let id = from_utf8(id_bytes).map_err(|e| {
        ActorError::illegal_state(format!(
            "failed to deserialize subscription ID from iter: {}",
            e
        ))
    })?;
    let subscription_id = SubscriptionId::new(id).map_err(|e| {
        ActorError::illegal_state(format!("failed to decode subscription ID from iter: {}", e))
    })?;
    Ok((subscription_id, sub))
}
