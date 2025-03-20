// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::io::Write;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use fvm::kernel::{ExecutionError, Result, SyscallError};
use fvm::syscalls::Context;
use fvm_shared::error::ErrorNumber;
use iroh_blobs::Hash;
use iroh_manager::DEL_FILE;
use recall_kernel_ops::RecallOps;

pub const MODULE_NAME: &str = "recall";
pub const HASHRM_SYSCALL_FUNCTION_NAME: &str = "hash_rm";

const ENV_IROH_PATH: &str = "IROH_SYSCALL_PATH";

static IROH_DEL_FILE: LazyLock<Option<Mutex<std::fs::File>>> = LazyLock::new(|| {
    let iroh_path = PathBuf::from(std::env::var(ENV_IROH_PATH).ok()?);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(iroh_path.join(DEL_FILE))
        .ok()?;
    Some(Mutex::new(file))
});

fn hash_source(bytes: &[u8]) -> Result<[u8; 32]> {
    bytes
        .try_into()
        .map_err(|e| ExecutionError::Syscall(SyscallError::new(ErrorNumber::IllegalArgument, e)))
}

pub fn hash_rm(context: Context<'_, impl RecallOps>, hash_offset: u32) -> Result<()> {
    let hash_bytes = context.memory.try_slice(hash_offset, 32)?;
    let seq_hash = Hash::from_bytes(hash_source(hash_bytes)?);

    let Some(del_file) = &*IROH_DEL_FILE else {
        tracing::error!("iroh instance has not been configured");
        return Ok(());
    };
    let mut del_file = del_file.lock().expect("poisoned lock");

    // Appending the tag will trigger deletion of the blob if it was the last reference.
    if let Err(err) = writeln!(del_file, "stored-seq-{seq_hash}") {
        tracing::warn!(hash = ?seq_hash, error = err.to_string(), "deleting tag from Iroh failed");
    }

    Ok(())
}
