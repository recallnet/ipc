// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Storage path for the objects iroh node
    #[arg(long, short, env = "IROH_OBJECTS_PATH")]
    pub iroh_objects_path: PathBuf,
    /// Storage path for the syscall iroh node
    #[arg(long, short, env = "IROH_SYSCALL_PATH")]
    pub iroh_syscall_path: PathBuf,
}
