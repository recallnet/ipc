// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::sync::LazyLock;

use fvm::kernel::{ExecutionError, Result, SyscallError};
use fvm::syscalls::Context;
use fvm_shared::error::ErrorNumber;
use iroh_blobs::Hash;
use iroh_manager::IrohManager;
use recall_kernel_ops::RecallOps;
use tokio::spawn;

pub const MODULE_NAME: &str = "recall";
pub const HASHRM_SYSCALL_FUNCTION_NAME: &str = "hash_rm";

const ENV_IROH_PATH: &str = "IROH_SYSCALL_PATH";
static IROH_INSTANCE: LazyLock<Option<IrohManager>> = LazyLock::new(|| {
    std::env::var(ENV_IROH_PATH)
        .ok()
        .map(|path| IrohManager::new(path))
});

fn hash_source(bytes: &[u8]) -> Result<[u8; 32]> {
    bytes
        .try_into()
        .map_err(|e| ExecutionError::Syscall(SyscallError::new(ErrorNumber::IllegalArgument, e)))
}

pub fn hash_rm(context: Context<'_, impl RecallOps>, hash_offset: u32) -> Result<()> {
    let hash_bytes = context.memory.try_slice(hash_offset, 32)?;
    let hash = Hash::from_bytes(hash_source(hash_bytes)?);

    // Don't block the chain with this.
    spawn(async move {
        let Some(iroh) = &*IROH_INSTANCE else {
            tracing::error!("iroh instance has not been configured");
            return;
        };
        let iroh_client = match iroh.blobs_client().await {
            Ok(client) => client,
            Err(e) => {
                tracing::error!(hash = ?hash, error = e.to_string(), "failed to initialize Iroh client");
                return;
            }
        };
        // Deleting the tag will trigger deletion of the blob if it was the last reference.
        // TODO: this needs to be tagged with a "user id"
        let tag = iroh_blobs::Tag(format!("stored-seq-{hash}").into());
        match iroh_client.tags().delete(tag.clone()).await {
            Ok(_) => tracing::debug!(tag = ?tag, hash = ?hash, "removed content from Iroh"),
            Err(e) => {
                tracing::warn!(tag = ?tag, hash = ?hash, error = e.to_string(), "deleting tag from Iroh failed");
            }
        }
    });
    Ok(())
}
