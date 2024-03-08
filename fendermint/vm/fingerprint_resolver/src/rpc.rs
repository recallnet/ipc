// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::pool::ResolveQueue;
use async_stm::{atomically, queues::TQueueLike};
use fvm_shared::address::Address;

pub struct RPCResolver {
    queue: ResolveQueue,
    /// owner of the pool (validator's own address if node is validating)
    owner: Address,
}

impl RPCResolver {
    pub fn new(queue: ResolveQueue, owner: Address) -> Self {
        Self { queue, owner }
    }

    /// Start taking tasks from the resolver pool and resolving them using the IPLD Resolver.
    pub async fn run(self) {
        loop {
            let task = atomically(|| {
                let task = self.queue.read()?;
                Ok(task)
            })
            .await;

            let owner = self.owner.clone();
            let proposer = task.proposer_address.clone();
            let proposed_at_height = task.proposed_at_height.clone();
            let fingerprint = task.fingerprint.clone();

            // It his task pool belongs the proposer of the task, then it should
            // POST the `fingerprint` on the destination chain(s).
            // Otherwise, it should get the fingerprint from the destination chains
            // for the `proposed_at_height` and verify it against `fingerprint`.
            //
            //
            // Dummy resolution. Sleeps for a few seconds and then resolve the task.
            //
            // In a real implementation, this will be replaced by RPC calls to
            // EVM chains to set the fingerprint on the given contract.
            if owner == proposer {
                tracing::info!("posting fingerprint {:?}", fingerprint);
                tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;
                atomically(|| task.set_resolved()).await;
                todo!("RPC call to post fingerprint")
            } else {
                tracing::info!(
                    "verifying fingerprint {:?}, at height {:?}",
                    fingerprint,
                    proposed_at_height
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                atomically(|| task.set_resolved()).await;
                todo!("RPC call to get fingerprint and verify")
            }
        }
    }
}
