// Copyright 2022-2024 Protocol Labs
// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
use toml_edit::{DocumentMut, value};
use colored::ColoredString;
use std::path::Path;
use std::process::Child;
use std::fs::{write, read_to_string};
use std::thread::JoinHandle;

use crate::util::{pipe_sub_command, get_rust_log_level, PipeSubCommandArgs};
use crate::{
    LogLevel,
    ETHAPI_PORTS
};

pub fn start_ethapi(
    fm_dir: &Path,
    label: &ColoredString,
    cmt_rpc_url: &str,
    log_level: &LogLevel
) -> (JoinHandle<()>, JoinHandle<()>, Child) {
    let rust_log = get_rust_log_level(log_level);

    // "start the etherium json-rpc facade",
    let ethapi_out = pipe_sub_command(PipeSubCommandArgs {
        title: label,
        cmd: "./target/debug/fendermint",
        envs: Some([
            ["RUST_LOG", rust_log].to_vec(),
            ["FM_NETWORK", "test"].to_vec()
        ].to_vec()),
        args: [
            "--home-dir",
            fm_dir.to_str().unwrap(),
            "--network",
            "testnet",
            "eth",
            "run",
            // The URL of the fendermint ABCI query RPC endpoint
            "--http-url",
            cmt_rpc_url,
            // how to set the listen port?
        ].to_vec(),
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level,
    });

    ethapi_out
}

pub fn init_ethapi(fm_dir: &Path, node_number: u8, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);
    // we use our default cometbft config.toml file, but we need to update to use the config for this network
    let fm_config_filepath = fm_dir.join("config").join("default.toml");
    let config_file = read_to_string(&fm_config_filepath).expect("could not modify cometbft config");
    let mut conf_doc = config_file.parse::<DocumentMut>().expect("invalid document");


    // TODO: setup separte ports for each node's metrics
    conf_doc["eth"]["listen"]["port"] = value(ETHAPI_PORTS[node_number as usize] as i64);
    conf_doc["eth"]["tracing"]["console"]["level"] = value(rust_log);
    conf_doc["eth"]["metrics"]["enabled"] = value(false);

    write(&fm_config_filepath, conf_doc.to_string()).expect("could not write to cometbft config file");
}
