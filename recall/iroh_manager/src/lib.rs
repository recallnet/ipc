// Copyright 2025 Recall Contributors
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::hashseq::HashSeq;
use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::rpc::client::blobs::BlobStatus;
use iroh_blobs::Hash;
use num_traits::Zero;
use tokio::sync::Mutex;

pub type IrohBlobsClient = iroh_blobs::rpc::client::blobs::MemClient;

/// Helper for managing Iroh connections.
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

/// Returns the user blob hash and size from the hash sequence.
/// The user blob hash is the first hash in the sequence.
pub async fn get_blob_hash_and_size(
    iroh: &IrohBlobsClient,
    seq_hash: Hash,
) -> Result<(Hash, u64), anyhow::Error> {
    // Get the hash sequence status (it needs to be available)
    let status = iroh.status(seq_hash).await.map_err(|e| {
        anyhow!(
            "failed to get status for hash sequence object: {} {}",
            seq_hash,
            e
        )
    })?;
    let BlobStatus::Complete { size } = status else {
        return Err(anyhow!(
            "hash sequence object {} is not available",
            seq_hash
        ));
    };
    if size.is_zero() {
        return Err(anyhow!("hash sequence object {} has zero size", seq_hash));
    }

    // Read the bytes and create a hash sequence
    let res = iroh
        .read_to_bytes(seq_hash)
        .await
        .map_err(|e| anyhow!("failed to read hash sequence object: {} {}", seq_hash, e))?;
    let hash_seq = HashSeq::try_from(res)
        .map_err(|e| anyhow!("failed to parse hash sequence object: {} {}", seq_hash, e))?;

    // Get the user blob status at index 0 (it needs to be available)
    let blob_hash = hash_seq.get(0).ok_or_else(|| {
        anyhow!(
            "failed to get hash with index 0 from hash sequence object: {}",
            seq_hash
        )
    })?;
    let status = iroh
        .status(blob_hash)
        .await
        .map_err(|e| anyhow!("failed to read object: {} {}", blob_hash, e))?;

    // Finally, get the size from the status
    let BlobStatus::Complete { size } = status else {
        return Err(anyhow!("object {} is not available", blob_hash));
    };

    Ok((blob_hash, size))
}
