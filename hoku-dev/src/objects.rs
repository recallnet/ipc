// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
use toml_edit::{DocumentMut, value};
use colored::ColoredString;
use std::fs::{write, read_to_string};
use std::path::Path;
use std::thread::JoinHandle;

use crate::util::{pipe_sub_command, get_rust_log_level, PipeSubCommandArgs};
use crate::{
    LogLevel,
    OBJECTS_PORTS
};

pub fn start_objects(
    fm_dir: &Path,
    label: &ColoredString,
    log_level: &LogLevel
) -> (JoinHandle<()>, JoinHandle<()>) {
    let rust_log = get_rust_log_level(log_level);

    // "objects-start",
    let objects_out = pipe_sub_command(PipeSubCommandArgs {
        title: label,
        cmd: "./target/debug/fendermint",
        args: [
            "--home-dir",
            fm_dir.to_str().unwrap(),
            "--network",
            "testnet",
            "objects",
            "run"
        ].to_vec(),
        envs: Some([
            // TODO: I don't think the RUST_LOG env does anything in the fendermint cli
            ["RUST_LOG", rust_log].to_vec(),
            ["FM_NETWORK", "test"].to_vec()
        ].to_vec()),
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level,
    });

    objects_out
}

pub fn init_objects(fm_dir: &Path, node_number: u8, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);
    // we use our default cometbft config.toml file, but we need to update to use the config for this network
    let fm_config_filepath = fm_dir.join("config").join("default.toml");
    let config_file = read_to_string(&fm_config_filepath).expect("could not modify cometbft config");
    let mut conf_doc = config_file.parse::<DocumentMut>().expect("invalid document");

    // TODO: setup separte ports for each node's object metrics
    conf_doc["objects"]["listen"]["port"] = value(OBJECTS_PORTS[node_number as usize] as i64);
    conf_doc["objects"]["tracing"]["console"]["level"] = value(rust_log);
    conf_doc["objects"]["metrics"]["enabled"] = value(false);

    write(&fm_config_filepath, conf_doc.to_string()).expect("could not write to cometbft config file");
}
