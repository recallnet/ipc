// SPDX-License-Identifier: MIT
//! Actor address util

use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use crate::{CommandLineHandler, GlobalArguments};

pub(crate) struct IdToFVMAddr;

#[async_trait]
impl CommandLineHandler for IdToFVMAddr {
    type Arguments = IdToFVMAddrArgs;

    async fn handle(_global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        let fvm = Address::new_id(arguments.id);
        log::info!("fvm address: {}", fvm.to_string());
        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Get Ethereum address for an FVM address")]
pub(crate) struct IdToFVMAddrArgs {
    #[arg(long, help = "FVM address to convert to Eth address")]
    pub id: u64,
}
