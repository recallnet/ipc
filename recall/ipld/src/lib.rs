// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_hamt::Sha256;

pub mod amt;
pub mod hamt;

type Hasher = Sha256;
