// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Join subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use ipc_api::subnet_id::SubnetID;
use num_traits::Zero;
use std::{fmt::Debug, str::FromStr};

use crate::{
    f64_to_token_amount, get_ipc_provider, require_fil_addr_from_str, CommandLineHandler,
    GlobalArguments,
};

/// The command to join a subnet
pub struct JoinSubnet;

#[async_trait]
impl CommandLineHandler for JoinSubnet {
    type Arguments = JoinSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("join subnet with args: {:?}", arguments);

        let mut provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let from = match &arguments.from {
            Some(address) => Some(require_fil_addr_from_str(address)?),
            None => None,
        };
        if let Some(initial_balance) = arguments.initial_balance.filter(|x| !x.is_zero()) {
            log::info!("pre-funding address with {initial_balance}");
            provider
                .pre_fund(subnet.clone(), from, f64_to_token_amount(initial_balance)?)
                .await?;
        }
        let storage_amount = arguments.storage_amount;
        let epoch = provider
            .join_subnet(
                subnet,
                from,
                f64_to_token_amount(arguments.collateral)?,
                storage_amount,
            )
            .await?;
        println!("joined at epoch: {epoch}");

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(name = "join", about = "Join a subnet")]
pub struct JoinSubnetArgs {
    #[arg(long, help = "The address that joins the subnet")]
    pub from: Option<String>,
    #[arg(long, help = "The subnet to join")]
    pub subnet: String,
    #[arg(
        long,
        help = "The collateral to stake in the subnet (in whole FIL units)"
    )]
    pub collateral: f64,
    #[arg(long, help = "Storage amount to commit to in the subnet (in GiBs)")]
    pub storage_amount: u128,
    #[arg(
        long,
        help = "Optionally add an initial balance to the validator in genesis in the subnet"
    )]
    pub initial_balance: Option<f64>,
}
