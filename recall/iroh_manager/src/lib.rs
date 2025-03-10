// Copyright 2025 Recall Contributors
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::net_protocol::Blobs;
use tokio::sync::Mutex;

pub type IrohBlobsClient = iroh_blobs::rpc::client::blobs::MemClient;

#[derive(Clone, Debug)]
pub struct IrohManager {
    client: Arc<Mutex<Option<IrohNode>>>,
    /// Storage path for iroh-blobs
    storage_path: PathBuf,
}

#[derive(Debug, Clone)]
struct IrohNode {
    _router: Router,
    blobs: Blobs<iroh_blobs::store::fs::Store>,
}

impl IrohNode {
    async fn new(path: impl AsRef<Path>) -> Result<Self> {
        // TODO: preserve secret key
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let blobs = Blobs::persistent(path).await?.build(&endpoint);
        let router = Router::builder(endpoint)
            .accept(iroh_blobs::ALPN, blobs.clone())
            .spawn()
            .await?;

        Ok(Self {
            _router: router,
            blobs,
        })
    }
}

impl IrohManager {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            client: Default::default(),
            storage_path: path.as_ref().to_path_buf(),
        }
    }

    /// Retrives a blob client, and starts the node if it has not started yet.
    pub async fn blobs_client(&self) -> Result<IrohBlobsClient> {
        let mut client = self.client.lock().await;
        if client.is_none() {
            let new_client = IrohNode::new(&self.storage_path).await?;
            client.replace(new_client);
        }

        Ok(client
            .as_ref()
            .expect("just inserted")
            .blobs
            .client()
            .clone())
    }
}
