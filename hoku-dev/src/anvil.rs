// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
use colored::Colorize;
use std::fs::write;
use std::path::Path;
use std::thread::JoinHandle;
use regex::Regex;

use crate::util::{log_level_print, pipe_sub_command, get_rust_log_level, PipeSubCommandArgs};
use crate::LogLevel;

pub const ANVIL_PRIVATE_KEYS: [&str; 10] = [
    "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    "5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
    "7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6",
    "47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a",
    "8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba",
    "92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e",
    "4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356",
    "dbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97",
    "2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
];
pub const ANVIL_PUBLIC_KEYS: [&str; 10] = [
    "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
    "0x70997970c51812dc3a010c7d01b50e0d17dc79c8",
    "0x3c44cdddb6a900fa2b585dd299e03d12fa4293bc",
    "0x90f79bf6eb2c4f870365e785982e1f101e93b906",
    "0x15d34aaf54267db7d7c367839aaf71a00a2c6a65",
    "0x9965507d1a55bcc2695c58ba16fb37d819b0a4dc",
    "0x976ea74026e726554db657fa54763abd0c3a0aa9",
    "0x14dc79964da2c08b23698b3d3cc7ca32193d9955",
    "0x23618e81e3f5cdf7f54c3d65f7fbc0abf5b21e8f",
    "0xa0ee7a142d267c1f36714e4a8f75612f20a79720",
];

pub fn start_anvil(log_level: &LogLevel) -> (JoinHandle<()>, JoinHandle<()>) {
    let rust_log = get_rust_log_level(log_level);

    let anvil_msg_filters = vec![
        Regex::new("eth_blockNumber"),
        Regex::new("eth_getTransactionReceipt"),
    ];
    let anvil_out = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("ANVIL").blue().bold(),
        cmd: "anvil",
        // note: you can use `["--block-time", "10"].to_vec(),` here to simulate a public net but startup is much slower
        args: vec![],
        envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
        current_dir: None,
        out_filters: anvil_msg_filters.clone(),
        err_filters: anvil_msg_filters.clone(),
        log_level,
    });

    log_level_print(&String::from("STARTUP").white().bold(),"anvil started", log_level,&LogLevel::Quiet, &vec![]);

    anvil_out
}

pub fn setup_anvil_keys(nodes: Vec<u8>, ipc_dir: &Path, log_level: &LogLevel) {
    // put validate private keys where the create_node func can find them
    for i in nodes.into_iter() {
        let filepath = ipc_dir.join(format!("validator_{i}.sk"));
        let private_key = ANVIL_PRIVATE_KEYS[i as usize];

        write(filepath, private_key).unwrap();
    }
    log_level_print(
        &String::from("STARTUP").white().bold(),
        "finished setting up validator private keys",
        log_level,
        &LogLevel::Info,
        &vec![]
    );

}
