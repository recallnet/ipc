// Copyright 2022-2024 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

// use fvm_ipld_hamt::{Hash, HashAlgorithm, HashedKey};
use fvm_ipld_hamt::{Hash, HashAlgorithm};
use fvm_sdk as fvm;
use fvm_shared::crypto::hash::SupportedHashes;
use std::hash::Hasher;

pub type HashedKey = [u8; 32];

#[derive(Default)]
struct RuntimeHasherWrapper(pub Vec<u8>);

/// This Hasher impl only intercepts key bytes. Is used only together with FvmHashSha256 below.
impl Hasher for RuntimeHasherWrapper {
    fn finish(&self) -> u64 {
        // u64 hash not used in hamt
        0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

#[derive(Default, Debug)]
pub struct FvmHashSha256;

impl HashAlgorithm for FvmHashSha256 {
    fn hash<X>(key: &X) -> HashedKey
    where
        X: Hash + ?Sized,
    {
        let mut rval_digest: HashedKey = Default::default();
        let mut hasher = RuntimeHasherWrapper::default();
        key.hash(&mut hasher);

        fvm::crypto::hash_into(SupportedHashes::Sha2_256, &hasher.0, &mut rval_digest);

        rval_digest
    }
}
