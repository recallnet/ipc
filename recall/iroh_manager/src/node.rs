// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::rpc::proto::RpcService;
use iroh_blobs::store::GcConfig;
use iroh_blobs::util::fs::load_secret_key;
use quic_rpc::server::{ChannelTypes, RpcChannel, RpcServerError};
use tracing::info;

use crate::BlobsClient;

/// Wrapper around and iroh `Endpoint` and the functionality
/// to handle blobs.
#[derive(Debug, Clone)]
pub struct IrohNode {
    router: Router,
    pub(crate) blobs: BlobsWrapper,
}

#[derive(Debug, Clone)]
pub(crate) enum BlobsWrapper {
    Mem {
        blobs: Blobs<iroh_blobs::store::mem::Store>,
        client: BlobsClient,
    },
    Fs {
        blobs: Blobs<iroh_blobs::store::fs::Store>,
        client: BlobsClient,
    },
}

impl BlobsWrapper {
    fn client(&self) -> &BlobsClient {
        match self {
            BlobsWrapper::Mem { ref client, .. } => client,
            BlobsWrapper::Fs { ref client, .. } => client,
        }
    }

    pub(crate) async fn handle_rpc_request<C>(
        self,
        msg: iroh_blobs::rpc::proto::Request,
        chan: RpcChannel<RpcService, C>,
    ) -> std::result::Result<(), RpcServerError<C>>
    where
        C: ChannelTypes<RpcService>,
    {
        match self {
            BlobsWrapper::Mem { blobs, .. } => blobs.handle_rpc_request(msg, chan).await,
            BlobsWrapper::Fs { blobs, .. } => blobs.handle_rpc_request(msg, chan).await,
        }
    }
}

/// GC interval duration.
const GC_DURATION: Duration = Duration::from_secs(300);

impl IrohNode {
    /// Creates a new persistent iroh node in the specified location.
    pub async fn persistent(path: impl AsRef<Path>) -> Result<Self> {
        // TODO: enable metrics

        let root = path.as_ref();
        info!("creating persistent iroh node in {}", root.display());

        let blobs_path = root.join("blobs");
        let secret_key_path = root.join("iroh_key");

        tokio::fs::create_dir_all(&blobs_path).await?;
        let secret_key = load_secret_key(secret_key_path).await?;

        let endpoint = Endpoint::builder()
            .discovery_n0()
            .secret_key(secret_key)
            .bind()
            .await?;
        let blobs = Blobs::persistent(path).await?.build(&endpoint);
        blobs.start_gc(GcConfig {
            period: GC_DURATION,
            done_callback: None,
        })?;

        let router = Router::builder(endpoint)
            .accept(iroh_blobs::ALPN, blobs.clone())
            .spawn()
            .await?;

        let client = blobs.client().boxed();
        Ok(Self {
            router,
            blobs: BlobsWrapper::Fs { blobs, client },
        })
    }

    /// Creates a new in memory based iroh node.
    pub async fn memory() -> Result<Self> {
        info!("creating inmemory iroh node");
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let blobs = Blobs::memory().build(&endpoint);
        blobs.start_gc(GcConfig {
            period: GC_DURATION,
            done_callback: None,
        })?;

        let router = Router::builder(endpoint)
            .accept(iroh_blobs::ALPN, blobs.clone())
            .spawn()
            .await?;
        let client = blobs.client().boxed();
        Ok(Self {
            router,
            blobs: BlobsWrapper::Mem { blobs, client },
        })
    }

    /// Returns the [`Endpoint`] for this node.
    pub fn endpoint(&self) -> &Endpoint {
        self.router.endpoint()
    }

    /// Returns the blobs client, necessary to interact with the blobs API:
    pub fn blobs_client(&self) -> &BlobsClient {
        self.blobs.client()
    }
}
