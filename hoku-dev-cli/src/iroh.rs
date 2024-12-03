// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
use toml_edit::{DocumentMut, value};
use clap::{Parser, Subcommand, ValueEnum};
use colored::{ColoredString, Colorize};
use regex::Regex;
use serde::Serialize;
use std::fs::{write, File, read_to_string};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::thread::sleep;
use std::thread::JoinHandle;
use std::time;

use crate::util::{log_level_print, pipe_sub_command, get_rust_log_level, THIRTY_SECONDS, PipeSubCommandArgs};
use crate::{LogLevel, IROH_RPC_PORTS};

pub fn setup_iroh_config(iroh_dir: &Path) {
    // we have a default cometbft config.toml file, and we need to update to use the config for this node
    let iroh_config_filepath = iroh_dir.join("config").join("iroh.config.toml");

    let config_file = read_to_string(&iroh_config_filepath).expect("could not modify iroh config");
    let mut conf_doc = config_file.parse::<DocumentMut>().expect("invalid document");

    // TODO: not sure if iroh config files let us control anything useful, i.e. listening ports
//    conf_doc["metrics_addr"] = value(format!("127.0.0.1:{:?}", node_iroh_metrics_port));
//
//    // If this is NOT the first iroh node, update the conf differently
//    if node_number > 0 {
//        conf_doc["p2p"]["seeds"] = value(format!("tcp://127.0.0.1:{:?}", CMT_RPC_PORTS[0]));
//    }

    write(&iroh_config_filepath, conf_doc.to_string()).expect("could not write to iroh config file");
}

pub fn start_iroh(iroh_dir: &PathBuf, rpc_address: &str, label: &ColoredString, log_level: &LogLevel) -> (JoinHandle<()>, JoinHandle<()>) {
    let rust_log = get_rust_log_level(log_level);
    let print = |message: &str, message_level: &LogLevel| {
        log_level_print(label, message, log_level, message_level, &vec![]);
    };
    //TODO: iroh metrics enabled by adding `metrics_addr = "0.0.0.0:{what ever IROH_METRICS_PORT is for this node}"` to config
    // "iroh-start",
    // TODO: iroh recommends starting nodes using the rust SDK. It might be a lot of work, but look into doing this here
    let iroh_out = pipe_sub_command(PipeSubCommandArgs {
        title: label,
        cmd: "iroh",
        args: [
            "--rpc-addr",
            rpc_address,
            "start",
        ]
        .to_vec(),
        envs: Some([
            // TODO: not sure how iroh cli configures logging
            ["RUST_LOG", rust_log].to_vec(),
            [
                "IROH_DATA_DIR",
                iroh_dir.join("data").to_str().unwrap(),
            ].to_vec(),
            [
                "IROH_CONFIG_DIR",
                iroh_dir.join("config").to_str().unwrap(),
            ].to_vec()
        ].to_vec()),
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level,
    });

    // "iroh-wait"
    // TODO: not sure we need all these 30 second waits.
    print("wait 30 seconds", &LogLevel::Info);
    thread::sleep(THIRTY_SECONDS);
    print("", &LogLevel::Info);

    iroh_out
}
