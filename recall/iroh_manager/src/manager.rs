// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::{Path, PathBuf};

use anyhow::Result;
use n0_future::task::AbortOnDropHandle;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{IrohBlobsClient, IrohNode};

#[derive(Debug)]
pub struct IrohManager {
    client: IrohNode,
    _cleanup_task: AbortOnDropHandle<()>,
}

/// Append to this file to trigger deletions
pub const DEL_FILE: &str = "delete_me.log";

impl IrohManager {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let storage_path = path.as_ref().to_path_buf();
        let client = IrohNode::persistent(&storage_path).await?;
        let blobs_client = client.blobs_client().clone();

        let sp = storage_path.clone();
        let cleanup_task = tokio::task::spawn(async move {
            if let Err(err) = run_cleanup(sp, blobs_client).await {
                error!("iroh cleanup task failed: {:?}", err);
            }
        });

        Ok(Self {
            client,
            _cleanup_task: AbortOnDropHandle::new(cleanup_task),
        })
    }

    /// Retrives a blob client, and starts the node if it has not started yet.
    pub fn blobs_client(&self) -> &IrohBlobsClient {
        self.client.blobs_client()
    }
}

async fn run_cleanup(storage_path: PathBuf, blobs: IrohBlobsClient) -> Result<()> {
    let path = storage_path.join(DEL_FILE);
    info!("opening file for cleanup: {}", path.display());

    let mut file = tokio::fs::OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(&path)
        .await?;
    let mut pos = tokio::fs::metadata(&path).await?.len();
    let rt = Handle::try_current()?;

    let (mut watcher, mut rx) = async_watcher(rt)?;

    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    let tags = blobs.tags();
    // watch
    while let Some(res) = rx.recv().await {
        match res {
            Ok(_event) => {
                // ignore any event that didn't change the pos
                if file.metadata().await?.len() == pos {
                    continue;
                }

                // read from pos to end of file
                if pos > 0 {
                    file.seek(std::io::SeekFrom::Start(pos + 1)).await?;
                }

                // update pos to end of file
                pos = file.metadata().await?.len();

                let reader = BufReader::new(&mut file);
                let mut lines = reader.lines();
                loop {
                    let Ok(Some(line)) = lines.next_line().await else {
                        break;
                    };
                    debug!("found line: {:?}", line);
                    match tags.delete(line.as_bytes()).await {
                        Ok(()) => {
                            debug!("deleted tag {}", line);
                        }
                        Err(err) => {
                            warn!("failed to delete tag {}: {:?}", line, err);
                        }
                    }
                }
            }
            Err(error) => println!("{error:?}"),
        }
    }

    Ok(())
}

fn async_watcher(
    rt: Handle,
) -> notify::Result<(RecommendedWatcher, mpsc::Receiver<notify::Result<Event>>)> {
    let (tx, rx) = mpsc::channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            let tx = tx.clone();
            rt.block_on(async move {
                tx.send(res).await.ok();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use n0_future::StreamExt;
    use tokio::io::AsyncWriteExt;

    use super::*;

    #[tokio::test]
    async fn test_append_delete() -> Result<()> {
        tracing_subscriber::fmt().init();
        let dir = tempfile::tempdir()?;

        let iroh = IrohManager::new(dir.path()).await?;

        let tags: Vec<_> = (0..10).map(|i| format!("tag-{i}")).collect();

        for tag in &tags {
            iroh.blobs_client()
                .add_bytes_named(format!("content-for-{tag}"), tag.as_bytes())
                .await?;
        }

        let existing_tags: Vec<_> = iroh
            .blobs_client()
            .tags()
            .list()
            .await?
            .try_collect()
            .await?;
        assert_eq!(existing_tags.len(), 10);

        // create file and start appending to it
        let p = dir.path().to_path_buf();
        let t = tags.clone();
        let task = tokio::task::spawn(async move {
            let path = p.join(DEL_FILE);
            info!("writing to {}", path.display());
            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(path)
                .await?;

            for tag in t {
                file.write_all(tag.as_bytes()).await?;
                file.write_all(b"\n").await?;
            }

            anyhow::Ok(())
        });

        task.await??;

        // wait for the tags to be deleted
        tokio::time::sleep(Duration::from_secs(3)).await;

        let existing_tags: Vec<_> = iroh
            .blobs_client()
            .tags()
            .list()
            .await?
            .try_collect()
            .await?;
        dbg!(&existing_tags);
        assert_eq!(existing_tags.len(), 0);

        Ok(())
    }
}
