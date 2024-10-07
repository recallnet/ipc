use std::str::FromStr;
use anyhow::anyhow;
use async_trait::async_trait;
use clap::Args;
use ipc_api::subnet_id::SubnetID;

use crate::{CommandLineHandler, GlobalArguments, f64_to_token_amount, get_ipc_provider, require_fil_addr_from_str};

/// The command to stake in a subnet from validator
pub struct StakeSubnet;

#[async_trait]
impl CommandLineHandler for StakeSubnet {
    type Arguments = StakeSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("join subnet with args: {:?}", arguments);

        let mut provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let from = match &arguments.from {
            Some(address) => Some(require_fil_addr_from_str(address)?),
            None => None,
        };
        provider
            .stake(subnet, from, f64_to_token_amount(arguments.collateral)?)
            .await
    }
}

#[derive(Debug, Args)]
#[command(name = "stake", about = "Add collateral to an already joined subnet")]
pub struct StakeSubnetArgs {
    #[arg(long, help = "The address that stakes in the subnet")]
    pub from: Option<String>,
    #[arg(long, help = "The subnet to add collateral to")]
    pub subnet: String,
    #[arg(
        long,
        help = "The collateral to stake in the subnet (in whole FIL units)"
    )]
    pub collateral: f64,
}

/// The command to unstake in a subnet from validator
pub struct UnstakeSubnet;

#[async_trait]
impl CommandLineHandler for UnstakeSubnet {
    type Arguments = UnstakeSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("join subnet with args: {:?}", arguments);

        let mut provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let from = match &arguments.from {
            Some(address) => Some(require_fil_addr_from_str(address)?),
            None => None,
        };
        provider
            .unstake(subnet, from, f64_to_token_amount(arguments.collateral)?)
            .await
    }
}

#[derive(Debug, Args)]
#[command(
    name = "unstake",
    about = "Remove collateral to an already joined subnet"
)]
pub struct UnstakeSubnetArgs {
    #[arg(long, help = "The address that unstakes in the subnet")]
    pub from: Option<String>,
    #[arg(long, help = "The subnet to release collateral from")]
    pub subnet: String,
    #[arg(
        long,
        help = "The collateral to unstake from the subnet (in whole FIL units)"
    )]
    pub collateral: f64,
}

/// Stake more storage to the subnet
pub struct StakeStorageSubnet;

#[async_trait]
impl CommandLineHandler for StakeStorageSubnet {
    type Arguments = StakeStorageSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        todo!()
    }
}

#[derive(Debug, Args)]
#[command(name = "stake-storage", about = "Commit more storage to an already joined subnet")]
pub struct StakeStorageSubnetArgs {
    #[arg(long, help = "The address that stakes in the subnet")]
    pub from: Option<String>,
    #[arg(long, help = "The subnet to add collateral to")]
    pub subnet: String,
    #[arg(
        long,
        help = "Storage amount to commit to in the subnet (in GiBs)"
    )]
    pub storage_amount: u64,
}

pub struct UnstakeStorageSubnet;

#[async_trait]
impl CommandLineHandler for UnstakeStorageSubnet {
    type Arguments = UnstakeStorageSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        todo!()
    }
}

#[derive(Debug, Args)]
#[command(name = "unstake-storage", about = "Uncommit some storage from a subnet")]
pub struct UnstakeStorageSubnetArgs {
    #[arg(long, help = "The address that stakes in the subnet")]
    pub from: Option<String>,
    #[arg(long, help = "The subnet to add collateral to")]
    pub subnet: String,
    #[arg(
        long,
        help = "Storage amount to remove from the subnet (in GiBs)"
    )]
    pub storage_amount: u64,
}
