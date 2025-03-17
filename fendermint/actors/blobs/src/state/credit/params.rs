// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::Credit;
use fvm_shared::{clock::ChainEpoch, econ::TokenAmount};

/// Params for committing capacity.
#[derive(Debug)]
pub struct CommitCapacityParams {
    /// Commitment size for subnet,
    /// which may be less than caller if the data already exists in the subnet.
    pub subnet_size: u64,
    /// Commitment size for caller.
    pub caller_size: u64,
    /// Commitment cost
    pub cost: Credit,
    /// Token amount available to commitment.
    pub value: TokenAmount,
    /// Commitment chain epoch.
    pub epoch: ChainEpoch,
}
