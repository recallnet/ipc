// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
pub struct RunArgs {
    #[arg(long, short, env = "IROH_OBJECTS_PATH")]
    pub iroh_path: PathBuf,
}
