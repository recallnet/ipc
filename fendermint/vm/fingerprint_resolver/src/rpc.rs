// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::pool::ResolveQueue;
use async_stm::{atomically, queues::TQueueLike};

pub struct RPCResolver {
    queue: ResolveQueue,
}

impl RPCResolver {
    pub fn new(queue: ResolveQueue) -> Self {
        Self { queue }
    }

    /// Start taking tasks from the resolver pool and resolving them using the IPLD Resolver.
    pub async fn run(self) {
        loop {
            let task = atomically(|| {
                let task = self.queue.read()?;
                Ok(task)
            })
            .await;

            // Dummy resolution. Sleeps for 30 seconds and then resolve the task.
            //
            // In a real implementation, this will be replaced by RPC calls to
            // EVM chains to set the fingerprint on the given contract.
            //
            // Ideally, it should also make a SNARK proof that takes as input the
            // the eth_storage_proof and the fingerprint (both public inputs)
            // This way validator can just verify the SNARK without having to
            // make an RPC call to the EVM chain.
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            tracing::info!(cid = ?task.cid(), "resolving fingerprinting task");
            atomically(|| task.set_resolved()).await;
        }
    }
}
