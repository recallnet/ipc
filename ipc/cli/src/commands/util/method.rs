// SPDX-License-Identifier: MIT
//! Actor address util

use async_trait::async_trait;
use clap::Args;
use cruet::Inflector;
use crate::{CommandLineHandler, GlobalArguments};

pub(crate) struct Method;

#[async_trait]
impl CommandLineHandler for Method {
    type Arguments = MethodArgs;

    async fn handle(_global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        let fvm = hash_method(&arguments.id.to_pascal_case());
        log::info!("fvm method: {}", fvm);
        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Get method number of fvm actor method")]
pub(crate) struct MethodArgs {
    #[arg(long, help = "FVM method name to convert to method number")]
    pub id: String,
}

struct Blake2bHasher {}
impl frc42_dispatch::hash::Hasher for Blake2bHasher {
    fn hash(&self, bytes: &[u8]) -> Vec<u8> {
        blake2b_simd::blake2b(bytes).as_bytes().to_vec()
    }
}

fn hash_method(name: &str) -> u64 {
    let resolver = frc42_dispatch::hash::MethodResolver::new(
        Blake2bHasher {}
    );
    resolver.method_number(&name.to_pascal_case()).unwrap()
}
