// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
use colored::Colorize;
use std::path::Path;
use std::process::Command;
use std::thread::JoinHandle;

use crate::anvil::ANVIL_PUBLIC_KEYS;
use crate::util::{pipe_sub_command, get_rust_log_level, sleep_three, PipeSubCommandArgs};
use crate::LogLevel;

// Start relayer
// note: this command mutates the order of keys in the evm_keystore.json file. to
// keep the accounts consistent for usage (e.g., logging accounts, using
// validator keys, etc.), we temporarily copy the file and then restore it.
pub fn start_relayer(ipc_config_dir: &Path, subnet_id: &str, log_level: &LogLevel) -> (JoinHandle<()>, JoinHandle<()>) {

    Command::new("cp").args([
        ipc_config_dir.join("evm_keystore.json"),
        ipc_config_dir.join("copy.evm_keystore.json")
    ]).output().unwrap();

    let relayer_out = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("RELAYER").bright_yellow().bold(),
        cmd: "./target/debug/ipc-cli",
        args: [
            "--config-path",
            ipc_config_dir.join("relayer.config.toml").to_str().unwrap(),
            "checkpoint",
            "relayer",
            "--submitter",
            ANVIL_PUBLIC_KEYS[0],
            "--max-parallelism",
            "1",
            "--checkpoint-interval-sec",
            "15",
            "--subnet",
            subnet_id,
            // in case we setup metrics, the command looks like something like this
            //--metrics-address 0.0.0.0(should this be localhost?):9184
        ].to_vec(),
        envs: Some([
            ["RUST_LOG", get_rust_log_level(log_level)].to_vec(),
            ["IPC_CLI_CONFIG_PATH", ipc_config_dir.join("relayer.config.toml").to_str().unwrap()].to_vec()
        ].to_vec()),
        current_dir: None,
        out_filters: vec!(),
        err_filters: vec!(),
        log_level
    });

    sleep_three(log_level);

    Command::new("cp").args([
        ipc_config_dir.join("copy.evm_keystore.json"),
        ipc_config_dir.join("evm_keystore.json")
    ]).output().unwrap();

    relayer_out
}
