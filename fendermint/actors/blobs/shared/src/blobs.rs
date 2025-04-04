// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;

use fvm_shared::address::Address;

mod blob;
mod params;
mod status;
mod subscribers;
mod subscription;
mod subscription_group;

pub use blob::*;
pub use params::*;
pub use status::*;
pub use subscribers::*;
pub use subscription::*;
pub use subscription_group::*;

use crate::bytes::B256;

/// The return type used when fetching "added" or "pending" blobs.
pub type BlobRequest = (B256, u64, HashSet<(Address, SubscriptionId, B256)>);
