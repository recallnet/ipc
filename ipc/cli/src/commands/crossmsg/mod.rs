// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::commands::crossmsg::fund::Fund;
use crate::commands::crossmsg::propagate::Propagate;
use crate::commands::crossmsg::release::Release;
use crate::commands::crossmsg::topdown_cross::{
    ListTopdownCrossMessages, ListTopdownCrossMessagesArgs,
};
use crate::{CommandLineHandler, GlobalArguments};
use fund::FundArgs;
use propagate::PropagateArgs;
use release::ReleaseArgs;

use clap::{Args, Subcommand};

pub mod fund;
pub mod propagate;
pub mod release;
mod topdown_cross;

#[derive(Debug, Args)]
#[command(name = "crossmsg", about = "cross network messages related commands")]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct CrossMsgsCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl CrossMsgsCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::Fund(args) => Fund::handle(global, args).await,
            Commands::Release(args) => Release::handle(global, args).await,
            Commands::Propagate(args) => Propagate::handle(global, args).await,
            Commands::ListTopdownCrossMsgs(args) => {
                ListTopdownCrossMessages::handle(global, args).await
            }
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Fund(FundArgs),
    Release(ReleaseArgs),
    Propagate(PropagateArgs),
    ListTopdownCrossMsgs(ListTopdownCrossMessagesArgs),
}
