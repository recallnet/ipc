// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{convert::Infallible, net::ToSocketAddrs, num::ParseIntError};

use anyhow::anyhow;
use anyhow::Context;
use base64::{engine::general_purpose, Engine};
use bytes::Buf;
use entangler::Entangler;
use entangler_storage::iroh::IrohStorage as EntanglerIrohStorage;
use fendermint_actor_bucket::{GetParams, Object};
use fendermint_app_settings::objects::ObjectsSettings;
use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::message::GasParams;
use fendermint_rpc::QueryClient;
use fendermint_vm_message::query::FvmQueryHeight;
use fendermint_vm_message::signed::SignedMessage;
use futures_util::StreamExt;
use fvm_shared::chainid::ChainID;
use fvm_shared::{address::Address, econ::TokenAmount};
use iroh::blobs::Hash;
use iroh::client::blobs::{BlobStatus, ReadAtLen};
use iroh::net::NodeAddr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;
use warp::{
    filters::multipart::Part,
    http::{HeaderMap, HeaderValue, StatusCode},
    hyper::body::Body,
    path::Tail,
    Filter, Rejection, Reply,
};

use crate::cmd;
use crate::options::objects::{ObjectsArgs, ObjectsCommands};

const MAX_OBJECT_LENGTH: u64 = 1024 * 1024 * 1024;
/// The alpha parameter for alpha entanglement determines the number of parity blob to generate for the original blob.
const ENTANGLER_ALPHA: u8 = 3;
/// The s parameter for alpha entanglement determines the number of horizontal strands in the grid.
const ENTANGLER_S: u8 = 5;
/// The p parameter for alpha entanglement determines the number of helical strands in the grid.
const ENTANGLER_P: u8 = 5;

cmd! {
    ObjectsArgs(self, settings: ObjectsSettings) {
        match self.command.clone() {
            ObjectsCommands::Run { tendermint_url, iroh_addr} => {
                if settings.metrics.enabled {
                    info!("metrics enabled");
                    let registry = prometheus::Registry::new();
                    // Default process metrics are enabled by default.

                    info!(
                        listen_addr = settings.metrics.listen.to_string(),
                        "serving metrics"
                    );
                    let mut builder = prometheus_exporter::Builder::new(settings.metrics.listen.try_into()?);
                    builder.with_registry(registry);
                    let _ = builder.start().context("failed to start metrics server")?;
                } else {
                    info!("metrics disabled");
                }

                let client = FendermintClient::new_http(tendermint_url, None)?;

                let iroh_addr = iroh_addr
                    .to_socket_addrs()?
                    .next()
                    .ok_or(anyhow!("failed to convert iroh_addr to a socket address"))?;
                let iroh_client = iroh::client::Iroh::connect_addr(iroh_addr).await?;

                // Admin routes
                let health = warp::path!("health")
                    .and(warp::get()).and_then(handle_health);
                let node_addr = warp::path!("v1" / "node" )
                .and(warp::get())
                .and(with_iroh(iroh_client.clone()))
                .and_then(handle_node_addr);

                // Objects routes
                let objects_upload = warp::path!("v1" / "objects" )
                .and(warp::post())
                .and(with_client(client.clone()))
                .and(with_iroh(iroh_client.clone()))
                .and(warp::multipart::form().max_length(MAX_OBJECT_LENGTH))
                .and_then(handle_object_upload);

                let objects_download = warp::path!("v1" / "objects" / Address / ..)
                .and(warp::path::tail())
                .and(
                    warp::get().map(|| "GET".to_string()).or(warp::head().map(|| "HEAD".to_string())).unify()
                )
                .and(warp::header::optional::<String>("Range"))
                .and(warp::query::<HeightQuery>())
                .and(with_client(client.clone()))
                .and(with_iroh(iroh_client.clone()))
                .and_then(handle_object_download);

                let router = health
                    .or(node_addr)
                    .or(objects_upload)
                    .or(objects_download)
                    .with(warp::cors().allow_any_origin()
                        .allow_headers(vec!["Content-Type"])
                        .allow_methods(vec!["PUT", "DEL", "GET", "HEAD"]))
                    .recover(handle_rejection);

                if let Some(listen_addr) = settings.listen.to_socket_addrs()?.next() {
                    warp::serve(router).run(listen_addr).await;
                    Ok(())
                } else {
                    Err(anyhow!("failed to convert to a socket address"))
                }
            },
        }
    }
}

fn with_client(
    client: FendermintClient,
) -> impl Filter<Extract = (FendermintClient,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_iroh(
    client: iroh::client::Iroh,
) -> impl Filter<Extract = (iroh::client::Iroh,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

#[derive(Serialize, Deserialize)]
struct HeightQuery {
    pub height: Option<u64>,
}

#[derive(Debug, Error)]
enum ObjectsError {
    #[error("error parsing range header: `{0}`")]
    RangeHeaderParseError(ParseIntError),
    #[error("invalid range header")]
    RangeHeaderInvalid,
}

impl From<ParseIntError> for ObjectsError {
    fn from(err: ParseIntError) -> Self {
        ObjectsError::RangeHeaderParseError(err)
    }
}

struct ObjectParser {
    signed_msg: Option<SignedMessage>,
    chain_id: ChainID,
    hash: Option<Hash>,
    size: Option<u64>,
    source: Option<NodeAddr>,
}

impl Default for ObjectParser {
    fn default() -> Self {
        ObjectParser {
            signed_msg: None,
            chain_id: ChainID::from(0),
            hash: None,
            size: None,
            source: None,
        }
    }
}

impl ObjectParser {
    async fn read_part(&mut self, part: Part) -> anyhow::Result<Vec<u8>> {
        let value = part
            .stream()
            .fold(Vec::new(), |mut vec, data| async move {
                if let Ok(data) = data {
                    vec.extend_from_slice(data.chunk());
                }
                vec
            })
            .await;
        Ok(value)
    }

    async fn read_chain_id(&mut self, form_part: Part) -> anyhow::Result<()> {
        let value = self.read_part(form_part).await?;
        let text = String::from_utf8(value).map_err(|_| anyhow!("cannot parse chain id"))?;
        let int: u64 = text.parse().map_err(|_| anyhow!("cannot parse chain_id"))?;
        self.chain_id = ChainID::from(int);
        Ok(())
    }

    async fn read_msg(&mut self, form_part: Part) -> anyhow::Result<()> {
        let value = self.read_part(form_part).await?;
        let signed_msg = general_purpose::URL_SAFE
            .decode(value)
            .map_err(|e| anyhow!("Failed to decode b64 encoded message: {}", e))
            .and_then(|b64_decoded| {
                fvm_ipld_encoding::from_slice::<SignedMessage>(&b64_decoded)
                    .map_err(|e| anyhow!("Failed to deserialize signed message: {}", e))
            })?;
        self.signed_msg = Some(signed_msg);
        Ok(())
    }

    async fn read_hash(&mut self, form_part: Part) -> anyhow::Result<()> {
        let value = self.read_part(form_part).await?;
        let text = String::from_utf8(value).map_err(|_| anyhow!("cannot parse hash"))?;
        let hash: Hash = text.parse().map_err(|_| anyhow!("cannot parse hash"))?;
        self.hash = Some(hash);
        Ok(())
    }

    async fn read_size(&mut self, form_part: Part) -> anyhow::Result<()> {
        let value = self.read_part(form_part).await?;
        let text = String::from_utf8(value).map_err(|_| anyhow!("cannot parse size"))?;
        let size: u64 = text.parse().map_err(|_| anyhow!("cannot parse size"))?;
        self.size = Some(size);
        Ok(())
    }

    async fn read_source(&mut self, form_part: Part) -> anyhow::Result<()> {
        let value = self.read_part(form_part).await?;
        let text = String::from_utf8(value).map_err(|_| anyhow!("cannot parse source"))?;
        let source: NodeAddr =
            serde_json::from_str(&text).map_err(|_| anyhow!("cannot parse source"))?;
        self.source = Some(source);
        Ok(())
    }

    async fn read_form(mut form_parts: warp::multipart::FormData) -> anyhow::Result<Self> {
        let mut object_parser = ObjectParser::default();
        while let Some(part) = form_parts.next().await {
            let part = part.map_err(|e| {
                dbg!(e);
                anyhow!("cannot read form data")
            })?;
            match part.name() {
                "chain_id" => {
                    object_parser.read_chain_id(part).await?;
                }
                "msg" => {
                    object_parser.read_msg(part).await?;
                }
                "hash" => {
                    object_parser.read_hash(part).await?;
                }
                "size" => {
                    object_parser.read_size(part).await?;
                }
                "source" => {
                    object_parser.read_source(part).await?;
                }
                _ => {
                    return Err(anyhow!("unknown form field"));
                }
            }
        }
        Ok(object_parser)
    }
}

async fn handle_health() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::reply())
}

async fn handle_node_addr(iroh: iroh::client::Iroh) -> Result<impl Reply, Rejection> {
    let node_addr = iroh.net().node_addr().await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to get iroh node address info: {}", e),
        })
    })?;
    Ok(warp::reply::json(&node_addr))
}

#[derive(Serialize)]
struct UploadResponse {
    hash: String,
    metadata_hash: String,
}

async fn handle_object_upload<F: QueryClient>(
    client: F,
    iroh: iroh::client::Iroh,
    form_parts: warp::multipart::FormData,
) -> Result<impl Reply, Rejection> {
    let parser = ObjectParser::read_form(form_parts).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to read form: {}", e),
        })
    })?;

    // Verify the signature
    let signed_msg = match parser.signed_msg {
        Some(signed_msg) => signed_msg,
        None => {
            return Err(Rejection::from(BadRequest {
                message: "missing signed message".to_string(),
            }))
        }
    };
    signed_msg.verify(&parser.chain_id).map_err(|e| {
        Rejection::from(BadRequest {
            message: e.to_string(),
        })
    })?;

    // Ensure the sender has enough credits, and fetch the data through iroh
    let SignedMessage { message, .. } = signed_msg;
    ensure_bucket_exists(client, message.to)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("failed to connect with bucket: {}", e),
            })
        })?;

    let hash = match parser.hash {
        Some(hash) => hash,
        None => {
            return Err(Rejection::from(BadRequest {
                message: "missing hash in form".to_string(),
            }))
        }
    };
    let size = match parser.size {
        Some(size) => size,
        None => {
            return Err(Rejection::from(BadRequest {
                message: "missing size in form".to_string(),
            }))
        }
    };
    let source = match parser.source {
        Some(source) => source,
        None => {
            return Err(Rejection::from(BadRequest {
                message: "missing source in form".to_string(),
            }))
        }
    };

    // TODO: What to do with downloaded data if there's a failure below?
    let progress = iroh.blobs().download(hash, source).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to fetch blob {}: {}", hash, e),
        })
    })?;
    let outcome = progress.finish().await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to fetch blob {}: {}", hash, e),
        })
    })?;

    let ent = new_entangler(iroh).map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to create entangler: {}", e),
        })
    })?;
    let metadata_hash = ent.entangle_uploaded(hash.to_string()).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to entangle uploaded data: {}", e),
        })
    })?;

    tracing::info!(
        "downloaded blob {} in {:?} (size: {}; local_size: {}; downloaded_size: {}; metadata: {})",
        hash,
        outcome.stats.elapsed,
        size,
        outcome.local_size,
        outcome.downloaded_size,
        metadata_hash,
    );

    let response = UploadResponse {
        hash: hash.to_string(),
        metadata_hash,
    };
    Ok(warp::reply::json(&response))
}

fn new_entangler(
    iroh: iroh::client::Iroh,
) -> Result<Entangler<EntanglerIrohStorage>, entangler::Error> {
    Entangler::new(
        EntanglerIrohStorage::from_client(iroh),
        ENTANGLER_ALPHA,
        ENTANGLER_S,
        ENTANGLER_P,
    )
}

async fn ensure_bucket_exists<F: QueryClient>(client: F, to: Address) -> anyhow::Result<()> {
    let actor_state = client.actor_state(&to, FvmQueryHeight::Committed).await?;
    actor_state.value.ok_or(anyhow!("cannot find actor {to}"))?;
    Ok(())
}

fn get_range_params(range: String, size: u64) -> Result<(u64, u64), ObjectsError> {
    let range: Vec<String> = range
        .replace("bytes=", "")
        .split('-')
        .map(|n| n.to_string())
        .collect();
    if range.len() != 2 {
        return Err(ObjectsError::RangeHeaderInvalid);
    }
    let (start, end): (u64, u64) = match (!range[0].is_empty(), !range[1].is_empty()) {
        (true, true) => (range[0].parse::<u64>()?, range[1].parse::<u64>()?),
        (true, false) => (range[0].parse::<u64>()?, size - 1),
        (false, true) => {
            let last = range[1].parse::<u64>()?;
            if last > size {
                (0, size - 1)
            } else {
                (size - last, size - 1)
            }
        }
        (false, false) => (0, size - 1),
    };
    if start > end || end >= size {
        return Err(ObjectsError::RangeHeaderInvalid);
    }
    Ok((start, end))
}

pub(crate) struct ObjectRange {
    start: u64,
    end: u64,
    len: u64,
    size: u64,
    body: Body,
}

async fn handle_object_download<F: QueryClient + Send + Sync>(
    address: Address,
    tail: Tail,
    method: String,
    range: Option<String>,
    height_query: HeightQuery,
    client: F,
    iroh: iroh::client::Iroh,
) -> Result<impl Reply, Rejection> {
    let height = height_query
        .height
        .unwrap_or(FvmQueryHeight::Committed.into());
    let path = tail.as_str();
    let key: Vec<u8> = path.into();
    let maybe_object = os_get(client, address, GetParams(key), height)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("bucket get error: {}", e),
            })
        })?;

    match maybe_object {
        Some(object) => {
            let hash = Hash::from_bytes(object.hash.0);
            let status = iroh.blobs().status(hash).await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to read object: {} {}", hash, e),
                })
            })?;
            let BlobStatus::Complete { size } = status else {
                // TODO: handle partial state if the range is in that
                return Err(Rejection::from(BadRequest {
                    message: format!("object {} is not available", hash),
                }));
            };

            let object_range = match range {
                Some(range) => {
                    let (start, end) = get_range_params(range, size).unwrap();
                    let len = (end - start) + 1;
                    let reader = iroh
                        .blobs()
                        .read_at(hash, start, ReadAtLen::AtMost(len))
                        .await
                        .map_err(|e| {
                            Rejection::from(BadRequest {
                                message: format!("failed to fetch object: {} {}", hash, e),
                            })
                        })?;
                    let body = Body::wrap_stream(reader);
                    ObjectRange {
                        start,
                        end,
                        len,
                        size,
                        body,
                    }
                }
                None => {
                    let ent = new_entangler(iroh).map_err(|e| {
                        Rejection::from(BadRequest {
                            message: format!("failed to create entangler: {}", e),
                        })
                    })?;
                    let recovery_hash = Hash::from_bytes(object.recovery_hash.0);
                    let bytes = ent
                        .download(&hash.to_string(), Some(&recovery_hash.to_string()))
                        .await
                        .map_err(|e| {
                            Rejection::from(BadRequest {
                                message: format!("failed to download object: {} {}", hash, e),
                            })
                        })?;
                    let body = Body::from(bytes);
                    ObjectRange {
                        start: 0,
                        end: size - 1,
                        len: size,
                        size,
                        body,
                    }
                }
            };

            // If it is a HEAD request, we don't need to send the body,
            // but we still need to send the Content-Length header
            if method == "HEAD" {
                let mut response = warp::reply::Response::new(Body::empty());
                let mut header_map = HeaderMap::new();
                header_map.insert("Content-Length", HeaderValue::from(object_range.size));
                let headers = response.headers_mut();
                headers.extend(header_map);
                return Ok(response);
            }

            let mut response = warp::reply::Response::new(object_range.body);
            let mut header_map = HeaderMap::new();
            if object_range.len < object_range.size {
                *response.status_mut() = StatusCode::PARTIAL_CONTENT;
                header_map.insert(
                    "Content-Range",
                    HeaderValue::from_str(&format!(
                        "bytes {}-{}/{}",
                        object_range.start, object_range.end, object_range.len
                    ))
                    .unwrap(),
                );
            } else {
                header_map.insert("Accept-Ranges", HeaderValue::from_str("bytes").unwrap());
            }
            header_map.insert("Content-Length", HeaderValue::from(object_range.len));
            header_map.insert(
                "Content-Type",
                HeaderValue::from_str(
                    object
                        .metadata
                        .get("content-type")
                        .unwrap_or(&"application/octet-stream".to_string()),
                )
                .unwrap(),
            );

            let headers = response.headers_mut();
            headers.extend(header_map);

            Ok(response)
        }
        None => Err(Rejection::from(NotFound)),
    }
}

// Rejection handlers

#[derive(Clone, Debug)]
struct BadRequest {
    message: String,
}

impl warp::reject::Reject for BadRequest {}

#[derive(Debug)]
struct NotFound;

impl warp::reject::Reject for NotFound {}

#[derive(Clone, Debug, Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() || err.find::<NotFound>().is_some() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<BadRequest>() {
        let err = e.to_owned();
        (StatusCode::BAD_REQUEST, err.message)
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            "Payload too large".to_string(),
        )
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
    };

    let reply = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });
    let reply = warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*");
    Ok(warp::reply::with_status(reply, code))
}

// RPC methods

async fn os_get<F: QueryClient + Send + Sync>(
    mut client: F,
    address: Address,
    params: GetParams,
    height: u64,
) -> anyhow::Result<Option<Object>> {
    let gas_params = GasParams {
        gas_limit: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };
    let h = FvmQueryHeight::from(height);

    let return_data = client
        .os_get_call(address, params, TokenAmount::default(), gas_params, h)
        .await?;

    Ok(return_data)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::str::FromStr;

    use super::*;
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::core::rand::{rngs::StdRng, SeedableRng};
    use fendermint_actor_bucket::AddParams;
    use fendermint_rpc::FendermintClient;
    use fendermint_vm_message::conv::from_eth::to_fvm_address;
    use fvm_ipld_encoding::RawBytes;
    use iroh::blobs::Hash;
    use iroh::net::NodeAddr;
    use tendermint_rpc::{Method, MockClient, MockRequestMethodMatcher};

    // Used to mocking Actor State
    const ABCI_QUERY_RESPONSE_UPLOAD: &str = r#"{
        "jsonrpc": "2.0",
        "id": "",
        "result": {
         "response": {
             "code": 0,
             "log": "",
             "info": "",
             "index": "0",
             "key": "GGQ=",
             "value": "pWRjb2Rl2CpYJwABVaDkAiB4ZQKaqaSEiu8tIb2Ef7bIWOoxPeNkAEljZabMaAMlaGVzdGF0ZdgqWCcAAXGg5AIgRbDPwiDO7Ft8HGLE1Bk9OOTrpI6IFXKc51+cCrDkwcBoc2VxdWVuY2UAZ2JhbGFuY2VKADY1ya3F3qAAAHFkZWxlZ2F0ZWRfYWRkcmVzc1YECqf8cO8ArRpoWzmcqFtkasWDXCqx",
             "proof": null,
             "height": "8",
             "codespace": ""
           }
        }
     }"#;

    const ABCI_QUERY_RESPONSE_DOWNLOAD: &str = r#"{
        "jsonrpc": "2.0",
        "id": "",
        "result": {
         "response": {
            "code": 0,
            "log": "",
            "info": "",
            "index": "0",
            "key": "",
            "value": "mKASGE0YpBhjGGMYaRhkGFgYJAEYcBIYIBilGGgY5AQY2BjqGNoY7BiSGFoYwxi8GBsYNhglGJwPGG0YcAANGHIYnBjZGBgYxBhqGIMYbxhJGHYYVxhkGHMYaRh6GGUGGGgYchhlGHMYbxhsGHYYZRhkGPUYaBhtGGUYdBhhGGQYYRh0GGEYoRhlGF8YcxhpGHoYZRhhGDYYMBiiGNgYbhg6GEsKBxhtGGUYcxhzGGEYZxhlEg0KBBhmGHIYbxhtEgMYdBgwGDAYGAESGDEKAhh0GG8SGCkYdBgyGHcYdRhoGHEYNxh0GGEYMxgzGGUYdxgzGGMYMhhuGGQYbRhnGGIYbBg3GDcYNxh6GDUYeBg0GGQYbBh5GGYYbhhtGHkYbBhqGGkYaBhxGBgB",
            "proof": null,
            "height": "6017",
            "codespace": ""
            }
        }
     }"#;

    fn form_body(
        boundary: &str,
        serialized_signed_message_b64: &str,
        hash: Hash,
        source: NodeAddr,
        size: u64,
    ) -> Vec<u8> {
        let mut body = Vec::new();
        let form_data = format!(
            "\
            --{0}\r\n\
            content-disposition: form-data; name=\"chain_id\"\r\n\r\n\
            314159\r\n\
            --{0}\r\n\
            content-disposition: form-data; name=\"msg\"\r\n\r\n\
            {1}\r\n\
            --{0}\r\n\
            content-disposition: form-data; name=\"hash\"\r\n\r\n\
            {2}\r\n\
            --{0}\r\n\
            content-disposition: form-data; name=\"size\"\r\n\r\n\
            {3}\r\n\
            --{0}\r\n\
            content-disposition: form-data; name=\"source\"\r\n\r\n\
            {4}\r\n\
            --{0}--\r\n\
            ",
            boundary,
            serialized_signed_message_b64,
            hash,
            size,
            serde_json::to_string_pretty(&source).unwrap(),
        );
        body.extend_from_slice(form_data.as_bytes());

        dbg!(std::str::from_utf8(&body)).unwrap();
        body
    }

    async fn multipart_form(
        serialized_signed_message_b64: &str,
        hash: Hash,
        source: NodeAddr,
        size: u64,
    ) -> warp::multipart::FormData {
        let boundary = "--abcdef1234--";
        let body = form_body(boundary, serialized_signed_message_b64, hash, source, size);
        warp::test::request()
            .method("POST")
            .header("content-length", body.len())
            .header(
                "content-type",
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(body)
            .filter(&warp::multipart::form())
            .await
            .unwrap()
    }

    fn setup_logs() {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        use tracing_subscriber::EnvFilter;

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(tracing_subscriber::fmt::format().with_line_number(true))
                    .with_writer(std::io::stdout),
            )
            .with(EnvFilter::from_default_env())
            .try_init()
            .ok();
    }

    #[tokio::test]
    async fn test_handle_object_upload() {
        setup_logs();

        let matcher = MockRequestMethodMatcher::default().map(
            Method::AbciQuery,
            Ok(ABCI_QUERY_RESPONSE_UPLOAD.to_string()),
        );
        let client = FendermintClient::new(MockClient::new(matcher).0);
        let iroh = iroh::node::Node::memory().spawn().await.unwrap();

        // client iroh node
        let client_iroh = iroh::node::Node::memory().spawn().await.unwrap();
        let hash = client_iroh
            .blobs()
            .add_bytes(&b"hello world"[..])
            .await
            .unwrap()
            .hash;
        let client_node_addr = client_iroh.net().node_addr().await.unwrap();

        let iroh_storage = EntanglerIrohStorage::from_client(client_iroh.client().clone());
        let ent = Entangler::new(iroh_storage, 3, 5, 5).unwrap();
        let metadata_hash = ent.entangle_uploaded(hash.to_string()).await.unwrap();

        let iroh_metadata_hash = Hash::from_str(&metadata_hash.as_str()).unwrap();

        let store = Address::new_id(90);
        let key = b"key";
        let size = 11;
        let params = AddParams {
            source: fendermint_actor_blobs_shared::state::PublicKey(*iroh.node_id().as_bytes()),
            key: key.to_vec(),
            hash: fendermint_actor_blobs_shared::state::Hash(*hash.as_bytes()),
            size,
            recovery_hash: fendermint_actor_blobs_shared::state::Hash(
                *iroh_metadata_hash.as_bytes(),
            ),
            ttl: None,
            metadata: HashMap::new(),
            overwrite: true,
        };
        let params = RawBytes::serialize(params).unwrap();

        let sk = fendermint_crypto::SecretKey::random(&mut StdRng::from_entropy());
        let signing_key = SigningKey::from_slice(sk.serialize().as_ref()).unwrap();
        let from_address = ethers::core::utils::secret_key_to_address(&signing_key);
        let message = fvm_shared::message::Message {
            version: Default::default(),
            from: to_fvm_address(from_address),
            to: store,
            sequence: 0,
            value: TokenAmount::from_atto(0),
            method_num: fendermint_actor_bucket::Method::AddObject as u64,
            params,
            gas_limit: 3000000,
            gas_fee_cap: TokenAmount::from_atto(0),
            gas_premium: TokenAmount::from_atto(0),
        };
        let chain_id = fvm_shared::chainid::ChainID::from(314159);
        let signed = SignedMessage::new_secp256k1(message, &sk, &chain_id).unwrap();

        let serialized_signed_message = fvm_ipld_encoding::to_vec(&signed).unwrap();
        let serialized_signed_message_b64 =
            general_purpose::URL_SAFE.encode(&serialized_signed_message);

        let multipart_form =
            multipart_form(&serialized_signed_message_b64, hash, client_node_addr, size).await;

        let reply = handle_object_upload(client, iroh.client().clone(), multipart_form)
            .await
            .unwrap();
        let response = reply.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore]
    async fn test_handle_object_download_get() {
        let matcher = MockRequestMethodMatcher::default().map(
            Method::AbciQuery,
            Ok(ABCI_QUERY_RESPONSE_DOWNLOAD.to_string()),
        );
        let client = FendermintClient::new(MockClient::new(matcher).0);
        let iroh = iroh::node::Node::memory().spawn().await.unwrap();
        let _hash = iroh
            .blobs()
            .add_bytes(&b"hello world"[..])
            .await
            .unwrap()
            .hash;

        let result = handle_object_download(
            Address::new_actor("t2mnd5jkuvmsaf457ympnf3monalh3vothdd5njoy".as_bytes()),
            warp::test::request()
                .path("/foo/bar")
                .filter(&warp::path::tail())
                .await
                .unwrap(),
            "GET".to_string(),
            None,
            HeightQuery { height: Some(1) },
            client,
            iroh.client().clone(),
        )
        .await;
        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("Content-Type")
                .unwrap()
                .to_str()
                .unwrap(),
            "application/octet-stream"
        );
        let body = warp::hyper::body::to_bytes(response.into_body())
            .await
            .unwrap();
        assert_eq!(body, "hello world".as_bytes());
    }

    #[tokio::test]
    #[ignore]
    async fn test_handle_object_download_with_range() {
        let matcher = MockRequestMethodMatcher::default().map(
            Method::AbciQuery,
            Ok(ABCI_QUERY_RESPONSE_DOWNLOAD.to_string()),
        );
        let client = FendermintClient::new(MockClient::new(matcher).0);
        let iroh = iroh::node::Node::memory().spawn().await.unwrap();
        let _hash = iroh
            .blobs()
            .add_bytes(&b"hello world"[..])
            .await
            .unwrap()
            .hash;

        let result = handle_object_download(
            Address::new_actor("t2mnd5jkuvmsaf457ympnf3monalh3vothdd5njoy".as_bytes()),
            warp::test::request()
                .path("/foo/bar")
                .filter(&warp::path::tail())
                .await
                .unwrap(),
            "GET".to_string(),
            Some("bytes=0-4".to_string()),
            HeightQuery { height: Some(1) },
            client,
            iroh.client().clone(),
        )
        .await;
        assert!(result.is_ok(), "{:#?}", result.err());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        let body = warp::hyper::body::to_bytes(response.into_body())
            .await
            .unwrap();
        assert_eq!(body, "hello".as_bytes());
    }

    #[tokio::test]
    #[ignore]
    async fn test_handle_object_download_head() {
        let matcher = MockRequestMethodMatcher::default().map(
            Method::AbciQuery,
            Ok(ABCI_QUERY_RESPONSE_DOWNLOAD.to_string()),
        );
        let client = FendermintClient::new(MockClient::new(matcher).0);
        let iroh = iroh::node::Node::memory().spawn().await.unwrap();
        let _hash = iroh
            .blobs()
            .add_bytes(&b"hello world"[..])
            .await
            .unwrap()
            .hash;

        let result = handle_object_download(
            Address::new_actor("t2mnd5jkuvmsaf457ympnf3monalh3vothdd5njoy".as_bytes()),
            warp::test::request()
                .path("/foo/bar")
                .filter(&warp::path::tail())
                .await
                .unwrap(),
            "HEAD".to_string(),
            None,
            HeightQuery { height: Some(1) },
            client,
            iroh.client().clone(),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("Content-Length").unwrap(), "11");
    }
}
