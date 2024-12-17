// Copyright 2022-2024 Protocol Labs
// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
use colored::Colorize;
use std::path::Path;
use std::process::Child;
use std::thread::JoinHandle;

use crate::cometbft::{start_cometbft, get_cmt_rpc};
use crate::ethapi::{start_ethapi, init_ethapi};
use crate::fendermint::start_fendermint;
use crate::iroh::{setup_iroh_config, start_iroh};
use crate::objects::{start_objects, init_objects};
use crate::util::{log_level_print, sleep_thirty};
use crate::{
    LogLevel,
    ContractMap
};

pub struct NodeHandles {
    pub iroh: (JoinHandle<()>, JoinHandle<()>, Child),
    pub cometbft: (JoinHandle<()>, JoinHandle<()>, Child),
    pub fendermint: (JoinHandle<()>, JoinHandle<()>, Child),
    pub evm: (JoinHandle<()>, JoinHandle<()>, Child),
    pub objects: (JoinHandle<()>, JoinHandle<()>, Child),
}

pub struct NodeConfig<'a> {
    pub node_path: &'a Path,
    pub ipc_config_dir: &'a Path,
    pub contracts:&'a  ContractMap,
    pub ports:&'a PortMap,
    pub node_number: u8,
    pub log_level: &'a LogLevel,
}

pub struct PortMap {
    pub abci: u16,
    pub cometbft_rpc: u16,
    pub cometbft_p2p: u16,
    pub ethapi: u16,
    pub fm_resolver: u16,
    pub iroh: u16,
    pub objects: u16
}

pub fn create_node(config: NodeConfig) -> NodeHandles {
    let NodeConfig {
        log_level,
        node_path,
        ipc_config_dir,
        contracts,
        node_number,
        ports
    } = config;

    let start_label = String::from("STARTUP").white().bold();
    let print = |message: &str, message_level: &LogLevel| {
        log_level_print(&start_label, message, log_level, message_level, &vec![]);
    };

    print(
        &format!("node path: {}", node_path.to_str().unwrap()),
        &LogLevel::Debug,
    );

    let iroh_dir = node_path.join("iroh");
    setup_iroh_config(&iroh_dir);
    let iroh_rpc_addr = format!("127.0.0.1:{:?}", ports.iroh);
    let iroh_out = start_iroh(&iroh_dir, &iroh_rpc_addr, &format!("IROH {:?}", node_number).magenta().bold(), log_level);

    // create config and start this nodes cometbft
    let cmt_dir = node_path.join("cometbft");
    let cmt_rpc_address = get_cmt_rpc(node_number);
    let cmt_rpc_url = &format!("http://{}", cmt_rpc_address);

    let cometbft_out = start_cometbft(
        &cmt_dir,
        &format!("COMETBFT {:?}", node_number).cyan().bold(),
        log_level
    );

    let fm_dir = node_path.join("fendermint");
    let label = format!("FENDERMINT {:?}", node_number).yellow().bold();
    let fm_resolver_port = format!("{:?}", ports.fm_resolver);

    init_objects(&fm_dir, node_number, log_level);
    init_ethapi(&fm_dir, node_number, log_level);

    let fendermint_out = start_fendermint(
        &fm_dir,
        &label,
        &format!("http://{}", &iroh_rpc_addr),
        cmt_rpc_url,
        &fm_resolver_port,
        log_level
    );
    sleep_thirty(log_level);
    let evm_out = start_ethapi(
        &fm_dir,
        &format!("ETHAPI_RPC {:?}", node_number).blue().on_yellow().bold(),
        cmt_rpc_url,
        log_level
    );
    sleep_thirty(log_level);
    let objects_out = start_objects(&fm_dir, &format!("OBJECTS {:?}", node_number).bright_green().bold(), log_level);
    sleep_thirty(log_level);

    print("", &LogLevel::Quiet);
    print(
        &format!("created subnet node {:?}", node_path),
        &LogLevel::Quiet,
    );
    print("", &LogLevel::Quiet);

    NodeHandles {
        iroh: iroh_out,
        cometbft: cometbft_out,
        fendermint: fendermint_out,
        evm: evm_out,
        objects: objects_out,
    }
}
