// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::store::GcConfig;
use iroh_blobs::util::fs::load_secret_key;
use tracing::info;

use crate::IrohBlobsClient;

/// Wrapper around and iroh `Endpoint` and the functionality
/// to handle blobs.
#[derive(Debug, Clone)]
pub struct IrohNode {
    router: Router,
    blobs: BlobsWrapper,
}

#[derive(Debug, Clone)]
enum BlobsWrapper {
    Mem(Blobs<iroh_blobs::store::mem::Store>),
    Fs(Blobs<iroh_blobs::store::fs::Store>),
}

impl BlobsWrapper {
    fn client(&self) -> &IrohBlobsClient {
        match self {
            BlobsWrapper::Mem(b) => b.client(),
            BlobsWrapper::Fs(b) => b.client(),
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

        Ok(Self {
            router,
            blobs: BlobsWrapper::Fs(blobs),
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
        Ok(Self {
            router,
            blobs: BlobsWrapper::Mem(blobs),
        })
    }

    /// Returns the [`Endpoint`] for this node.
    pub fn endpoint(&self) -> &Endpoint {
        self.router.endpoint()
    }

    /// Returns the blobs client, necessary to interact with the blobs API:
    pub fn blobs_client(&self) -> &IrohBlobsClient {
        self.blobs.client()
    }
}
