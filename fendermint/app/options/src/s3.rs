// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use adm_sdk::network::Network as SdkNetwork;
use clap::{Args, Subcommand, ValueEnum};

#[derive(Args, Debug)]
pub struct S3Args {
    #[command(subcommand)]
    pub command: S3Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum S3Commands {
    Run {
        /// Network presets for subnet and RPC URLs.
        #[arg(short, long, value_enum, default_value_t = Network::Testnet, env = "SDK_NETWORK")]
        network: Network,

        /// Access key used for authentication.
        #[arg(long, env = "S3_ACCESS_KEY")]
        access_key: Option<String>,

        /// Secret key used for authentication.
        #[arg(long, env = "S3_SECRET_KEY")]
        secret_key: Option<String>,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Network {
    /// Network presets for mainnet.
    Mainnet,
    /// Network presets for Calibration (default pre-mainnet).
    Testnet,
    /// Network presets for a local three-node network.
    Localnet,
    /// Network presets for local development.
    Devnet,
}

impl Network {
    pub fn get(&self) -> SdkNetwork {
        match self {
            Network::Mainnet => SdkNetwork::Mainnet,
            Network::Testnet => SdkNetwork::Testnet,
            Network::Localnet => SdkNetwork::Localnet,
            Network::Devnet => SdkNetwork::Devnet,
        }
    }
}
