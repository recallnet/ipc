// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//
// Forked from https://github.com/consensus-shipyard/libp2p-bitswap with assumed MIT license
// as per Cargo.toml: https://github.com/consensus-shipyard/libp2p-bitswap/blob/7dd9cececda3e4a8f6e14c200a4b457159d8db33/Cargo.toml#L7
//
// License headers added post-fork.
use crate::compat::other;
use libipld::cid::{Cid, Version};
use libipld::multihash::{Code, MultihashDigest};
use std::convert::TryFrom;
use std::io::Result;
use unsigned_varint::{decode as varint_decode, encode as varint_encode};

/// Prefix represents all metadata of a CID, without the actual content.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Prefix {
    /// The version of CID.
    pub version: Version,
    /// The codec of CID.
    pub codec: u64,
    /// The multihash type of CID.
    pub mh_type: u64,
    /// The multihash length of CID.
    pub mh_len: usize,
}

impl Prefix {
    /// Create a new prefix from encoded bytes.
    pub fn new(data: &[u8]) -> Result<Prefix> {
        let (raw_version, remain) = varint_decode::u64(data).map_err(other)?;
        let version = Version::try_from(raw_version).map_err(other)?;
        let (codec, remain) = varint_decode::u64(remain).map_err(other)?;
        let (mh_type, remain) = varint_decode::u64(remain).map_err(other)?;
        let (mh_len, _remain) = varint_decode::usize(remain).map_err(other)?;
        Ok(Prefix {
            version,
            codec,
            mh_type,
            mh_len,
        })
    }

    /// Convert the prefix to encoded bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(4);
        let mut buf = varint_encode::u64_buffer();
        let version = varint_encode::u64(self.version.into(), &mut buf);
        res.extend_from_slice(version);
        let mut buf = varint_encode::u64_buffer();
        let codec = varint_encode::u64(self.codec, &mut buf);
        res.extend_from_slice(codec);
        let mut buf = varint_encode::u64_buffer();
        let mh_type = varint_encode::u64(self.mh_type, &mut buf);
        res.extend_from_slice(mh_type);
        let mut buf = varint_encode::u64_buffer();
        let mh_len = varint_encode::u64(self.mh_len as u64, &mut buf);
        res.extend_from_slice(mh_len);
        res
    }

    /// Create a CID out of the prefix and some data that will be hashed
    pub fn to_cid(&self, data: &[u8]) -> Result<Cid> {
        let mh = Code::try_from(self.mh_type).map_err(other)?.digest(data);
        Cid::new(self.version, self.codec, mh).map_err(other)
    }
}

impl From<&Cid> for Prefix {
    fn from(cid: &Cid) -> Self {
        Self {
            version: cid.version(),
            codec: cid.codec(),
            mh_type: cid.hash().code(),
            mh_len: cid.hash().digest().len(),
        }
    }
}
