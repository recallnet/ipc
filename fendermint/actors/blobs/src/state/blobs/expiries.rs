// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::fmt::Display;

use fendermint_actor_blobs_shared::{blobs::SubscriptionId, bytes::B256};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{tuple::*, RawBytes};
use fvm_shared::{address::Address, clock::ChainEpoch};
use log::debug;
use recall_ipld::{
    amt::{self, vec::TrackedFlushResult},
    hamt::{self, MapKey},
};

/// Key used to namespace subscriptions in the expiry index.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct ExpiryKey {
    /// Key hash.
    pub hash: B256,
    /// Key subscription ID.
    pub id: SubscriptionId,
}

impl Display for ExpiryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExpiryKey(hash: {}, id: {})", self.hash, self.id)
    }
}

impl MapKey for ExpiryKey {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        let raw_bytes = RawBytes::from(b.to_vec());
        fil_actors_runtime::cbor::deserialize(&raw_bytes, "ExpiryKey")
            .map_err(|e| format!("Failed to deserialize ExpiryKey {}", e))
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let raw_bytes = fil_actors_runtime::cbor::serialize(self, "ExpiryKey")
            .map_err(|e| format!("Failed to serialize ExpiryKey {}", e))?;
        Ok(raw_bytes.to_vec())
    }
}

impl ExpiryKey {
    /// Create a new expiry key.
    pub fn new(hash: B256, id: &SubscriptionId) -> Self {
        Self {
            hash,
            id: id.clone(),
        }
    }
}

/// Type used as the root of [`Expiries`].
type ExpiriesRoot = hamt::Root<Address, hamt::Root<ExpiryKey, ()>>;

/// AMT wrapper for expiry index state.
#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct Expiries {
    /// The AMT root.
    pub root: amt::Root<ExpiriesRoot>,
    /// Index marker for pagination.
    /// When present, iteration starts from this index.
    /// Otherwise, iteration begins from the first entry.
    /// Used for efficient traversal during blob expiration.
    next_index: Option<u64>,
}

impl Expiries {
    /// Returns a new expiry collection.
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = amt::Root::<ExpiriesRoot>::new(store)?;
        Ok(Self {
            root,
            next_index: None,
        })
    }

    /// Returns the underlying [`amt::vec::Amt`].
    pub fn amt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<amt::vec::Amt<'a, BS, ExpiriesRoot>, ActorError> {
        self.root.amt(store)
    }

    /// Saves the state from the [`TrackedFlushResult`].
    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<ExpiriesRoot>) {
        self.root = tracked_flush_result.root;
    }

    /// The size of the collection.
    pub fn len<BS: Blockstore>(&self, store: BS) -> Result<u64, ActorError> {
        Ok(self.root.amt(store)?.count())
    }

    /// Iterates the collection up to the given epoch.
    pub fn foreach_up_to_epoch<BS: Blockstore, F>(
        &mut self,
        store: BS,
        epoch: ChainEpoch,
        batch_size: Option<u64>,
        mut f: F,
    ) -> Result<(), ActorError>
    where
        F: FnMut(ChainEpoch, Address, ExpiryKey) -> Result<(), ActorError>,
    {
        let expiries = self.amt(&store)?;

        debug!(
            "walking blobs up to epoch {} (next_index: {:?})",
            epoch, self.next_index
        );

        let (_, next_idx) = expiries.for_each_while_ranged(
            self.next_index,
            batch_size,
            |index, per_chain_epoch_root| {
                if index > epoch as u64 {
                    return Ok(false);
                }
                let per_chain_epoch_hamt = per_chain_epoch_root.hamt(&store, 0)?;
                per_chain_epoch_hamt.for_each(|address, per_address_root| {
                    let per_address_hamt = per_address_root.hamt(&store, 0)?;
                    per_address_hamt.for_each(|expiry_key, _| f(index as i64, address, expiry_key))
                })?;
                Ok(true)
            },
        )?;
        self.next_index = next_idx.filter(|&idx| idx <= epoch as u64);

        debug!("walked blobs (next_index: {:?})", self.next_index,);

        Ok(())
    }

    /// Updates the collection by applying the list of [`ExpiryUpdate`]s.
    pub fn update<BS: Blockstore>(
        &mut self,
        store: BS,
        subscriber: Address,
        hash: B256,
        id: &SubscriptionId,
        updates: Vec<ExpiryUpdate>,
    ) -> Result<(), ActorError> {
        if updates.is_empty() {
            return Ok(());
        }

        let mut expiries = self.amt(&store)?;
        for update in updates {
            match update {
                ExpiryUpdate::Add(chain_epoch) => {
                    // You cannot do get_or_create here: it expects value, we give it Result<Option<Value>>
                    let per_chain_epoch_root =
                        if let Some(per_chain_epoch_root) = expiries.get(chain_epoch as u64)? {
                            per_chain_epoch_root
                        } else {
                            hamt::Root::<Address, hamt::Root<ExpiryKey, ()>>::new(
                                &store,
                                &Expiries::store_name_per_root(chain_epoch),
                            )?
                        };
                    // The size does not matter
                    let mut per_chain_epoch_hamt = per_chain_epoch_root.hamt(&store, 1)?;
                    // You cannot do get_or_create here: it expects value, we give it Result<Option<Value>>
                    let per_address_root =
                        if let Some(per_address_root) = per_chain_epoch_hamt.get(&subscriber)? {
                            per_address_root
                        } else {
                            hamt::Root::<ExpiryKey, ()>::new(
                                &store,
                                &Expiries::store_name_per_address(chain_epoch, &subscriber),
                            )?
                        };
                    let mut per_address_hamt = per_address_root.hamt(&store, 1)?; // The size does not matter here
                    let expiry_key = ExpiryKey::new(hash, id);
                    let per_address_root = per_address_hamt.set_and_flush(&expiry_key, ())?;
                    let per_chain_epoch_root =
                        per_chain_epoch_hamt.set_and_flush(&subscriber, per_address_root)?;
                    self.save_tracked(
                        expiries.set_and_flush_tracked(chain_epoch as u64, per_chain_epoch_root)?,
                    );
                }
                ExpiryUpdate::Remove(chain_epoch) => {
                    if let Some(mut per_chain_epoch_root) = expiries.get(chain_epoch as u64)? {
                        let mut per_chain_epoch_hamt = per_chain_epoch_root.hamt(&store, 1)?; // The size does not matter here
                        if let Some(mut per_address_root) = per_chain_epoch_hamt.get(&subscriber)? {
                            let mut per_address_hamt = per_address_root.hamt(&store, 1)?; // The size does not matter here
                            let expiry_key = ExpiryKey::new(hash, id);
                            (per_address_root, _) =
                                per_address_hamt.delete_and_flush(&expiry_key)?;
                            if per_address_hamt.is_empty() {
                                (per_chain_epoch_root, _) =
                                    per_chain_epoch_hamt.delete_and_flush(&subscriber)?;
                            } else {
                                per_chain_epoch_root = per_chain_epoch_hamt
                                    .set_and_flush(&subscriber, per_address_root)?;
                            }
                        }
                        if per_chain_epoch_hamt.is_empty() {
                            self.save_tracked(
                                expiries.delete_and_flush_tracked(chain_epoch as u64)?,
                            );
                        } else {
                            self.save_tracked(
                                expiries.set_and_flush_tracked(
                                    chain_epoch as u64,
                                    per_chain_epoch_root,
                                )?,
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Returns the store display name.
    fn store_name() -> String {
        "expiries".to_string()
    }

    /// Returns the store display name for a root.
    fn store_name_per_root(chain_epoch: ChainEpoch) -> String {
        format!("{}.{}", Expiries::store_name(), chain_epoch)
    }

    /// Returns the store display name for an address.
    fn store_name_per_address(chain_epoch: ChainEpoch, address: &Address) -> String {
        format!("{}.{}", Expiries::store_name_per_root(chain_epoch), address)
    }
}

/// Helper enum for expiry updates.
pub enum ExpiryUpdate {
    /// Entry to add.
    Add(ChainEpoch),
    /// Entry to remove.
    Remove(ChainEpoch),
}

#[cfg(test)]
mod tests {
    use super::*;

    use fendermint_actor_blobs_testing::{new_address, new_hash};
    use fvm_ipld_blockstore::MemoryBlockstore;

    #[test]
    fn test_expiries_foreach_up_to_epoch() {
        let store = MemoryBlockstore::default();
        let mut state = Expiries::new(&store).unwrap();

        let addr = new_address();
        let mut hashes = vec![];
        for i in 1..=100 {
            let (hash, _) = new_hash(1024);
            state
                .update(
                    &store,
                    addr,
                    hash,
                    &SubscriptionId::default(),
                    vec![ExpiryUpdate::Add(i)],
                )
                .unwrap();
            hashes.push(hash);
        }
        assert_eq!(state.len(&store).unwrap(), 100);

        let mut range = vec![];
        state
            .foreach_up_to_epoch(&store, 10, None, |chain_epoch, _, _| {
                range.push(chain_epoch);
                Ok(())
            })
            .unwrap();
        assert_eq!(range.len(), 10);

        // Remove an element to test against a sparse state
        let remove_epoch = 5;
        let hash = hashes[remove_epoch - 1];
        state
            .update(
                &store,
                addr,
                hash,
                &SubscriptionId::default(),
                vec![ExpiryUpdate::Remove(remove_epoch as ChainEpoch)],
            )
            .unwrap();
        assert_eq!(state.len(&store).unwrap(), 99);

        let mut range = vec![];
        state
            .foreach_up_to_epoch(&store, 10, None, |chain_epoch, _, _| {
                range.push(chain_epoch);
                Ok(())
            })
            .unwrap();
        assert_eq!(range.len(), 9);
    }

    #[test]
    fn test_expiries_pagination() {
        let store = MemoryBlockstore::default();
        let mut state = Expiries::new(&store).unwrap();
        let addr = new_address();

        // Create expiries at epochs 1,2,4,7,8,10
        for i in &[1, 2, 4, 7, 8, 10] {
            let (hash, _) = new_hash(1024);
            state
                .update(
                    &store,
                    addr,
                    hash,
                    &SubscriptionId::default(),
                    vec![ExpiryUpdate::Add(*i as ChainEpoch)],
                )
                .unwrap();
        }

        // Process with batch size 2
        let mut processed = vec![];
        let mut done = false;
        while !done {
            state
                .foreach_up_to_epoch(&store, 10, Some(2), |epoch, _, _| {
                    processed.push(epoch);
                    Ok(())
                })
                .unwrap();
            done = state.next_index.is_none();
        }

        // Should get all epochs in order, despite gaps
        assert_eq!(processed, vec![1, 2, 4, 7, 8, 10]);
    }

    #[test]
    fn test_expiries_pagination_with_mutations() {
        let store = MemoryBlockstore::default();
        let mut state = Expiries::new(&store).unwrap();
        let addr = new_address();
        let current_epoch = 100;

        // Initial set: 110,120,130,140,150
        let mut hashes = vec![];
        for ttl in (10..=50).step_by(10) {
            let (hash, _) = new_hash(1024);
            state
                .update(
                    &store,
                    addr,
                    hash,
                    &SubscriptionId::default(),
                    vec![ExpiryUpdate::Add(current_epoch + ttl)],
                )
                .unwrap();
            hashes.push(hash);
        }

        let mut processed = vec![];

        // Process first batch (110,120)
        state
            .foreach_up_to_epoch(&store, 150, Some(2), |epoch, _, _| {
                processed.push(epoch);
                Ok(())
            })
            .unwrap();
        assert_eq!(processed, vec![110, 120]);

        // Add new expiry at 135
        let (hash, _) = new_hash(1024);
        state
            .update(
                &store,
                addr,
                hash,
                &SubscriptionId::default(),
                vec![ExpiryUpdate::Add(current_epoch + 35)],
            )
            .unwrap();

        // Remove expiry at 140
        let hash = hashes[3];
        state
            .update(
                &store,
                addr,
                hash,
                &SubscriptionId::default(),
                vec![ExpiryUpdate::Remove(current_epoch + 40)],
            )
            .unwrap();

        // Process remaining epochs
        while state.next_index.is_some() {
            state
                .foreach_up_to_epoch(&store, 150, Some(2), |epoch, _, _| {
                    processed.push(epoch);
                    Ok(())
                })
                .unwrap();
        }

        // Should get all expiries in order, with 140 removed and 135 added
        assert_eq!(processed, vec![110, 120, 130, 135, 150]);
    }

    #[test]
    fn test_expiries_pagination_with_expiry_update() {
        let store = MemoryBlockstore::default();
        let mut state = Expiries::new(&store).unwrap();
        let addr = new_address();
        let current_epoch = 100;

        // Initial set: add blobs with ttl 10,20,30,40,50
        let mut hashes = vec![];
        for ttl in (10..=50).step_by(10) {
            let (hash, _) = new_hash(1024);
            let expiry = current_epoch + ttl;
            state
                .update(
                    &store,
                    addr,
                    hash,
                    &SubscriptionId::default(),
                    vec![ExpiryUpdate::Add(expiry)],
                )
                .unwrap();
            hashes.push(hash);
        }

        let mut processed = vec![];

        // Process the first two expiries (110,120)
        state
            .foreach_up_to_epoch(&store, 150, Some(2), |epoch, _, _| {
                processed.push(epoch);
                Ok(())
            })
            .unwrap();
        assert_eq!(processed, vec![110, 120]);

        // Extend the expiry of the blob at 130 to 145 (can only extend, not reduce)
        let hash = hashes[2]; // blob with ttl 30
        state
            .update(
                &store,
                addr,
                hash,
                &SubscriptionId::default(),
                vec![
                    ExpiryUpdate::Remove(current_epoch + 30), // remove 130
                    ExpiryUpdate::Add(current_epoch + 45),    // add 145 (extended)
                ],
            )
            .unwrap();

        // Process remaining epochs - should see updated expiry
        while state.next_index.is_some() {
            state
                .foreach_up_to_epoch(&store, 150, Some(2), |epoch, _, _| {
                    processed.push(epoch);
                    Ok(())
                })
                .unwrap();
        }

        // Should get all expiries in chronological order, with 130 replaced by 145
        assert_eq!(processed, vec![110, 120, 140, 145, 150]);
    }

    #[test]
    fn test_expiries_pagination_with_multiple_subscribers() {
        let store = MemoryBlockstore::default();
        let mut state = Expiries::new(&store).unwrap();
        let addr1 = new_address();
        let addr2 = new_address();

        // Add multiple blobs expiring at the same epochs
        // addr1: two blobs expiring at 110, one at 120
        // addr2: one blob expiring at 110, two at 130
        let mut entries = vec![];

        // addr1's blobs
        for _ in 0..2 {
            let (hash, _) = new_hash(1024);
            state
                .update(
                    &store,
                    addr1,
                    hash,
                    &SubscriptionId::default(),
                    vec![ExpiryUpdate::Add(110)],
                )
                .unwrap();
            entries.push((110, addr1, hash));
        }
        let (hash, _) = new_hash(1024);
        state
            .update(
                &store,
                addr1,
                hash,
                &SubscriptionId::default(),
                vec![ExpiryUpdate::Add(120)],
            )
            .unwrap();
        entries.push((120, addr1, hash));

        // addr2's blobs
        let (hash, _) = new_hash(1024);
        state
            .update(
                &store,
                addr2,
                hash,
                &SubscriptionId::default(),
                vec![ExpiryUpdate::Add(110)],
            )
            .unwrap();
        entries.push((110, addr2, hash));

        for _ in 0..2 {
            let (hash, _) = new_hash(1024);
            state
                .update(
                    &store,
                    addr2,
                    hash,
                    &SubscriptionId::default(),
                    vec![ExpiryUpdate::Add(130)],
                )
                .unwrap();
            entries.push((130, addr2, hash));
        }

        let mut processed = vec![];
        let mut done = false;

        // Process all entries with batch size 2
        while !done {
            state
                .foreach_up_to_epoch(&store, 150, Some(2), |epoch, subscriber, key| {
                    processed.push((epoch, subscriber, key.hash));
                    Ok(())
                })
                .unwrap();
            done = state.next_index.is_none();
        }

        // Should get all entries, with multiple entries per epoch
        assert_eq!(processed.len(), 6); // Total number of blob expiries

        // Verify we got all entries at epoch 110
        let epoch_110 = processed.iter().filter(|(e, _, _)| *e == 110).count();
        assert_eq!(epoch_110, 3); // 2 from addr1, 1 from addr2

        // Verify we got all entries at epoch 130
        let epoch_130 = processed.iter().filter(|(e, _, _)| *e == 130).count();
        assert_eq!(epoch_130, 2); // Both from addr2
    }
}
