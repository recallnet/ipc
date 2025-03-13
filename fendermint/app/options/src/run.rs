// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use clap::Args;
use iroh_base::NodeId;

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Storage path for the iroh node
    #[arg(long, env = "IROH_PATH")]
    pub iroh_path: PathBuf,
    /// Storage path for the syscall iroh node
    #[arg(long, env = "IROH_OBJECTS_NODE_ID")]
    pub iroh_objects_id: NodeId,
}
