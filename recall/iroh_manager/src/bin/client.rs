use std::net::ToSocketAddrs;
use std::net::SocketAddr;

use anyhow::Result;
use clap::Parser;
use iroh_blobs::rpc::proto::RpcService;
use iroh_manager::BlobsClient;
use n0_future::StreamExt;
use quic_rpc::client::QuinnConnector;
use tracing::info;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Server address to connect to
    #[clap(short, long, default_value = "127.0.0.1:4919")]
    server: String,

    /// Operation to perform: list, add, delete
    #[clap(short, long, default_value = "list")]
    operation: String,

    /// Tag to use for add/delete operations
    #[clap(short, long)]
    tag: Option<String>,

    /// Content to add (for add operation)
    #[clap(short, long)]
    content: Option<String>,
}



/// Connect to the given rpc listening on this address, with this key.
pub async fn connect(remote_addr: SocketAddr) -> Result<BlobsClient> {
    info!("iroh RPC connecting to {}", remote_addr);
    let bind_addr: SocketAddr = "127.0.0.1:0".parse()?;
    let client = quic_rpc::transport::quinn::make_insecure_client_endpoint(bind_addr)?;
    let client = QuinnConnector::<RpcService>::new(client, remote_addr, "localhost".to_string());
    let client = quic_rpc::RpcClient::<RpcService, _>::new(client);
    let client = iroh_blobs::rpc::client::blobs::Client::new(client);
    Ok(client.boxed())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let server_addrs: Vec<_> = args.server.to_socket_addrs().unwrap().collect();

    for addr in &server_addrs {
        info!("Found server address: {}", addr);
    }
    let server_addr = server_addrs.get(0).unwrap().clone();

    info!("Connecting to Iroh Manager at {}", args.server);
    let client = connect(server_addr).await?;

    match args.operation.as_str() {
        "list" => {
            info!("Listing all tags");
            let tags: Vec<_> = client.tags().list().await?.try_collect().await?;

            if tags.is_empty() {
                println!("No tags found");
            } else {
                println!("Found {} tags:", tags.len());
                for tag in tags {
                    println!("- {:?}", tag);
                }
            }
        }
        "add" => {
            let tag = args
                .tag
                .ok_or_else(|| anyhow::anyhow!("Tag is required for add operation"))?;
            let content = args
                .content
                .ok_or_else(|| anyhow::anyhow!("Content is required for add operation"))?;

            info!("Adding content with tag: {}", tag);
            client.add_bytes_named(content, tag.as_bytes()).await?;
            println!("Content added successfully with tag: {}", tag);
        }
        "delete" => {
            let tag = args
                .tag
                .ok_or_else(|| anyhow::anyhow!("Tag is required for delete operation"))?;

            info!("Deleting tag: {}", tag);
            client.tags().delete(tag).await?;
            println!("Tag deleted successfully");
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown operation: {}", args.operation));
        }
    }

    tokio::signal::ctrl_c().await?;

    Ok(())
}
