// Copyright 2025 Recall Contributors
// SPDX-License-Identifier: Apache-2.0, MIT

pub type IrohBlobsClient = iroh_blobs::rpc::client::blobs::MemClient;

mod manager;
mod node;

pub use self::manager::IrohManager;
pub use self::node::IrohNode;
