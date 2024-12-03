// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
use toml_edit::{DocumentMut, value};
use clap::{Parser, Subcommand, ValueEnum};
use colored::ColoredString;
use std::fs::{write, File, read_to_string};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread::JoinHandle;

use crate::util::{log_level_print, pipe_sub_command, get_rust_log_level, PipeSubCommandArgs};
use crate::{
    LogLevel,
    ABCI_PORTS,
    ETHAPI_PORTS,
    OBJECTS_PORTS
};


pub fn start_fendermint(
    fm_dir: &Path,
    label: &ColoredString,
    iroh_rpc_url: &str,
    cmt_rpc_url: &str,
    resolver_port: &str,
    log_level: &LogLevel
) -> (JoinHandle<()>, JoinHandle<()>) {
    let rust_log = get_rust_log_level(log_level);
    let validator_key_path = fm_dir
        .join("keys")
        .join("validator_key.sk");

    // "fendermint-start-validator",
    pipe_sub_command(PipeSubCommandArgs {
        title: label,
        cmd: "./target/debug/fendermint",
        args: [
            "--home-dir",
            fm_dir.to_str().unwrap(),
            "run",
            "--iroh-addr",
            iroh_rpc_url
        ].to_vec(),
        envs: Some(
            [
                ["RUST_LOG", rust_log].to_vec(),
                ["TENDERMINT_RPC_URL", cmt_rpc_url].to_vec(),
                ["FM_NETWORK", "test"].to_vec(),
                ["FM_TRACING_CONSOLE_LEVEL", rust_log].to_vec(),
                [
                    "FM_VALIDATOR_KEY__PATH",
                    validator_key_path.to_str().unwrap(),
                ]
                .to_vec(),
                ["FM_VALIDATOR_KEY__KIND", "regular"].to_vec(),
                [
                    "FM_RESOLVER__CONNECTION__LISTEN_ADDR",
                    &format!("/ip4/127.0.0.1/tcp/{resolver_port}"),
                ]
                .to_vec(),
            ]
            .to_vec(),
        ),
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level
    })
}

pub fn init_fendermint(fm_dir: &Path, node_number: u8) {
    // we use our default cometbft config.toml file, but we need to update to use the config for this network
    let fm_config_filepath = fm_dir.join("config").join("default.toml");
    let config_file = read_to_string(&fm_config_filepath).expect("could not modify cometbft config");
    let mut conf_doc = config_file.parse::<DocumentMut>().expect("invalid document");

    conf_doc["abci"]["listen"]["port"] = value(ABCI_PORTS[node_number as usize] as i64);
    conf_doc["eth"]["listen"]["port"] = value(ETHAPI_PORTS[node_number as usize] as i64);
    conf_doc["objects"]["listen"]["port"] = value(OBJECTS_PORTS[node_number as usize] as i64);

    // TODO: setup separte ports for each node's metrics
    conf_doc["metrics"]["enabled"] = value(false);
    conf_doc["eth"]["metrics"]["enabled"] = value(false);
    conf_doc["objects"]["metrics"]["enabled"] = value(false);

    // TODO: we need to get snapshots working... there's a ticket for this
    //conf_doc["snapshots"]["enabled"] = value(true);

    write(&fm_config_filepath, conf_doc.to_string()).expect("could not write to cometbft config file");
}
