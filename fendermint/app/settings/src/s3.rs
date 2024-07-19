// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::SocketAddress;
use serde::Deserialize;
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct S3Settings {
    pub listen: SocketAddress,
}
