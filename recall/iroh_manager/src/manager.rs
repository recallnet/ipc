// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;
use tracing::info;

use crate::{IrohBlobsClient, IrohNode};

#[derive(Clone, Debug)]
pub struct IrohManager {
    client: Arc<Mutex<Option<IrohNode>>>,
    /// Storage path for iroh-blobs
    storage_path: PathBuf,
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
            info!("no active iroh node found, starting");
            let new_client = IrohNode::persistent(&self.storage_path).await?;
            client.replace(new_client);
        }

        Ok(client
            .as_ref()
            .expect("just inserted")
            .blobs_client()
            .clone())
    }
}
