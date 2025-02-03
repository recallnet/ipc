// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fil_actors_runtime::ActorError;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, MethodNum};
use log::info;
use sha2::{Digest, Sha256};

use crate::shared::{ReadRequest, ReadRequestStatus, RequestId};
use fendermint_actor_blobs_shared::state::Hash;

const MAX_READ_REQUEST_LEN: u32 = 1024 * 1024; // 1MB

/// The state represents all read requests.
#[derive(Debug, Default, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// Map of read requests by request ID.
    pub read_requests: HashMap<RequestId, ReadRequest>,
    /// Counter to sequence the requests.
    pub request_id_counter: RequestId,
}

impl State {
    pub fn open_read_request(
        &mut self,
        blob_hash: Hash,
        offset: u32,
        len: u32,
        callback_addr: Address,
        callback_method: u64,
    ) -> Result<u64, ActorError> {
        // Validate length is not greater than the maximum allowed
        if len > MAX_READ_REQUEST_LEN {
            return Err(ActorError::illegal_argument(format!(
                "read request length {} exceeds maximum allowed {}",
                len, MAX_READ_REQUEST_LEN
            )));
        }
        let request_id = self.next_request_id();
        let read_request = ReadRequest {
            blob_hash,
            offset,
            len,
            callback_addr,
            callback_method,
            status: ReadRequestStatus::Open,
        };
        info!("opening a read request onchain: {:?}", request_id);
        // will overrite a previous request with the same hash
        self.read_requests.insert(request_id, read_request);
        Ok(request_id)
    }

    pub fn close_read_request(&mut self, request_id: RequestId) -> Result<(), ActorError> {
        if self.get_read_request_status(request_id).is_none() {
            return Err(ActorError::not_found(
                "cannot close read request, it does not exist".to_string(),
            ));
        }
        // remove the closed request
        self.read_requests.remove(&request_id);
        Ok(())
    }

    pub fn get_open_read_requests(
        &self,
        size: u32,
    ) -> Vec<(RequestId, Hash, u32, u32, Address, MethodNum)> {
        self.read_requests
            .iter()
            .filter(|(_, request)| matches!(request.status, ReadRequestStatus::Open))
            .take(size as usize)
            .map(|element| {
                (
                    *element.0,
                    element.1.blob_hash,
                    element.1.offset,
                    element.1.len,
                    element.1.callback_addr,
                    element.1.callback_method,
                )
            })
            .collect::<Vec<_>>()
    }

    pub fn get_read_request_status(&self, id: RequestId) -> Option<ReadRequestStatus> {
        self.get_read_request(id).map(|r| r.status)
    }

    /// As we index requests by number, it makes sense to return a request by its id.
    pub fn get_read_request(&self, id: RequestId) -> Option<ReadRequest> {
        self.read_requests.get(&id).map(|r| r.clone())
    }

    /// Set a read request status to pending.
    pub fn set_read_request_pending(&mut self, id: RequestId) -> Result<(), ActorError> {
        let request = self
            .read_requests
            .get_mut(&id)
            .ok_or_else(|| ActorError::not_found(format!("read request {} not found", id)))?;

        if !matches!(request.status, ReadRequestStatus::Open) {
            return Err(ActorError::illegal_state(format!(
                "read request {} is not in open state",
                id
            )));
        }

        request.status = ReadRequestStatus::Pending;
        Ok(())
    }

    fn next_request_id(&mut self) -> RequestId {
        self.request_id_counter += 1;
        self.request_id_counter
    }
}

// FIXME SU Delete this
pub(crate) fn pad_to_32_bytes(input: &[u8]) -> [u8; 32] {
    let mut padded = [0u8; 32];
    let start = 32_usize.saturating_sub(input.len());
    padded[start..].copy_from_slice(&input[..input.len().min(32)]);
    padded
}
