use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use iroh_manager::IrohManager;
use n0_future::TryStreamExt;
use tracing::info;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Data directory for iroh storage
    #[clap(short, long, default_value = "./iroh-data")]
    data_dir: PathBuf,

    /// IP address to bind to
    #[clap(short, long, default_value = "0.0.0.0")]
    ip: IpAddr,

    /// Port to listen on for RPC
    #[clap(short, long, default_value = "4919")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    info!("Starting Iroh Manager on {}:{}", args.ip, args.port);
    let rpc_addr = SocketAddr::new(args.ip, args.port);
    
    // // Convert IP to appropriate socket address format
    // let (v4_addr, v6_addr) = match args.ip {
    //     IpAddr::V4(addr) => (Some(addr), None),
    //     IpAddr::V6(addr) => (None, Some(addr)),
    // };

    let dir = tempfile::tempdir()?;

    // Create and start the manager
    let manager = IrohManager::new(None, None, dir.path(), Some(rpc_addr)).await?;
    
    info!("Iroh Manager started and listening at {}", manager.rpc_addr());

    let tags: Vec<_> = (0..10).map(|i| format!("tag-{i}")).collect();

        for tag in &tags {
            manager.blobs_client()
                .add_bytes_named(format!("content-for-{tag}"), tag.as_bytes())
                .await?;
        }

        let existing_tags: Vec<_> = manager
            .blobs_client()
            .tags()
            .list()
            .await?
            .try_collect()
            .await?;

    // Keep the server running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Iroh Manager");

    Ok(())
}
