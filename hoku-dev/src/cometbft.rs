// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
use toml_edit::{DocumentMut, value};
use colored::{ColoredString, Colorize};
use regex::Regex;
use std::fs::{write, read_to_string};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::JoinHandle;

use crate::util::{log_level_print, pipe_sub_command, get_rust_log_level, sleep_thirty, NODE_PREFIX, PipeSubCommandArgs};
use crate::{LogLevel, CMT_RPC_PORTS, CMT_P2P_PORTS};

// for localnet this will generate the required genesis and config files for the given number of nodes
// parent_dir is the root of the localnet directory, e.g. ~/.ipc/NETWORK_NAME
pub fn init_cometbft(nodes: Vec<u8>, parent_dir: &PathBuf, log_level: &LogLevel) {
    // start by cleaning any existing tmp dir
    remove_tmp_dir();
    let tmp_dir = create_tmp_dir();

    let rust_log = get_rust_log_level(log_level);
    let (cometbft_stdout, cometbft_stderr) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("COMETBFT INIT").bright_green().bold(),
        cmd: "cometbft",
        envs: Some([
            ["RUST_LOG", rust_log].to_vec(),
        ].to_vec()),
        args: [
            "testnet",
            "--o",
            tmp_dir.to_str().unwrap(),
            "--v",
            &nodes.len().to_string(),
            "--hostname-prefix",
            NODE_PREFIX,
            "--node-dir-prefix",
            NODE_PREFIX
        ].to_vec(),
        out_filters: vec!(),
        err_filters: vec!(),
        current_dir: None,
        log_level
    });

    cometbft_stdout.join().unwrap();
    cometbft_stderr.join().unwrap();
    // copy the genesis, config, data, etc... resulting from above from the tmp dir to ~/.ipc/NETWORK_NAME/validator-*/validator-*/cometbft/
    for i in nodes.clone().into_iter() {
        move_config(
            i,
            &tmp_dir.join(format!("{NODE_PREFIX}{:?}", i)),
            parent_dir,
            log_level
        );
    }

    let persistent_peers = get_persistent_peers(nodes.clone(), parent_dir);
    // customize config to fit this network
    for i in nodes.into_iter() {
        set_config(
            i,
            &persistent_peers,
            parent_dir,
            log_level
        );
    }

    remove_tmp_dir();
}

pub fn start_cometbft(cmt_dir: &Path, label: &ColoredString, log_level: &LogLevel) -> (JoinHandle<()>, JoinHandle<()>) {
    // "cometbft-init",
    let cmt_home = String::from(cmt_dir.to_str().unwrap());
    let rust_log = get_rust_log_level(log_level);

    // "cometbft-start",
    let out = pipe_sub_command(PipeSubCommandArgs {
        title: label,
        cmd: "cometbft",
        args: [
            "start",
            "--proxy_app",
            "kvstore",
            "--home",
            &cmt_home,
            "--consensus.create_empty_blocks_interval",
            "15s"
        ]
        .to_vec(),
        envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
        current_dir: None,
        out_filters: vec![
            Regex::new("received proposal"),
            Regex::new("received complete proposal block"),
            Regex::new("finalizing commit of block"),
            Regex::new("executed block"),
            Regex::new("committed state"),
            Regex::new("indexed block exents"),
            Regex::new("Timed out"),
        ],
        err_filters: vec![],
        log_level,
    });

    // "cometbft-wait"
    sleep_thirty(log_level);

    out
}

fn create_tmp_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap();
    let tmp_dir = Path::new(&home).join(".cometbft-tmp");

    Command::new("mkdir")
        .args([
            tmp_dir.to_str().unwrap()
        ])
        .output()
        .expect("could not create cometbft temp directory");

    tmp_dir
}

fn remove_tmp_dir() {
    let home = std::env::var("HOME").unwrap();
    Command::new("rm")
        .args([
            "-rf",
            Path::new(&home).join(".cometbft-tmp").to_str().unwrap()
        ])
        .output()
        .expect("could not remove cometbft temp directory");
}

fn move_config(node_number: u8, from_dir: &PathBuf, root_dir: & PathBuf, log_level: &LogLevel) {
    let to_dir = root_dir.join(format!("{NODE_PREFIX}{:?}", node_number)).join(format!("{NODE_PREFIX}{:?}", node_number)).join("cometbft");
    // we have a default cometbft config.toml file, and we need to update to use the config for this node
    let cp_conf_out = Command::new("cp")
        .args([
            "-r",
            from_dir.join("config").join(".").to_str().unwrap(),
            to_dir.join("config").join("").to_str().unwrap()
        ])
        .output()
        .expect("failed to copy cometbft config");
    log_level_print(&format!("COMETBFT CONF {:?}", node_number).yellow().on_blue().bold(), &format!("{:?}", cp_conf_out), log_level, &LogLevel::Debug, &vec!());
    let cp_data_out = Command::new("cp")
        .args([
            "-r",
            from_dir.join("data").join(".").to_str().unwrap(),
            to_dir.join("data").join("").to_str().unwrap()
        ])
        .output()
        .expect("failed to copy cometbft config");
    log_level_print(&format!("COMETBFT CONF {:?}", node_number).blue().on_yellow().bold(), &format!("{:?}", cp_data_out), log_level, &LogLevel::Debug, &vec!());

}

// after all of the directories are setup and populated with default files, this updates cmt config files to fit the local network
fn set_config(node_number: u8, persistent_peers: &str, root_dir: & PathBuf, log_level: &LogLevel) {
    let cmt_dir = root_dir
        .join(format!("{NODE_PREFIX}{:?}", node_number))
        .join(format!("{NODE_PREFIX}{:?}", node_number))
        .join("cometbft");
    let cmt_config_filepath = cmt_dir.join("config").join("config.toml");
    let config_file = read_to_string(&cmt_config_filepath).expect("could not modify cometbft config");
    let mut conf_doc = config_file.parse::<DocumentMut>().expect("invalid document");

    // TODO: does doing this make sense? cmtbft is really noisy with debug logging
    conf_doc["log_level"] = value(match log_level {
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Quiet => "error",
        // note: the silent value differs from RUST_LOG
        LogLevel::Silent => "none"
    });
    conf_doc["proxy_app"] = value("kvstore");
    conf_doc["max_subscription_clients"] = value(10);
    // note: I don't know why this is so high, but leaveing it like the upstream localnet
    conf_doc["max_subscriptions_per_client"] = value(1000);
    // TCP or UNIX socket address for the RPC server to listen on
    conf_doc["rpc"]["laddr"] = value(format!("tcp://{}", get_cmt_rpc(node_number)));
    conf_doc["p2p"]["laddr"] = value(format!("tcp://127.0.0.1:{:?}", CMT_P2P_PORTS[node_number as usize]));
    conf_doc["p2p"]["persistent_peers"] = value(persistent_peers);

    write(&cmt_config_filepath, conf_doc.to_string()).expect("could not write to cometbft config file");
}

pub fn get_cmt_rpc(node_number: u8) -> String {
    format!("127.0.0.1:{:?}", CMT_RPC_PORTS[node_number as usize])
}

fn get_persistent_peers(nodes: Vec<u8>, root_dir: &PathBuf) -> String {
    let mut peer_str = String::from("");
    // note: the multi node cometbft start up is a little weird since  
    for i in 0..nodes.len() {
        let node_num = nodes[i];
        let home_dir = root_dir
            .join(format!("{NODE_PREFIX}{:?}", node_num))
            .join(format!("{NODE_PREFIX}{:?}", node_num))
            .join("cometbft");
        let node_id = get_cmt_node_id(&home_dir);
        if i == 0 {
            peer_str = format!("{node_id}@127.0.0.1:{:?}", CMT_P2P_PORTS[node_num as usize]);
        } else {
            peer_str = format!("{peer_str},{node_id}@127.0.0.1:{:?}", CMT_P2P_PORTS[node_num as usize]);
        }
    }

    // return immutable String
    peer_str
}

fn get_cmt_node_id(home_dir: &PathBuf) -> String {
    let out = Command::new("cometbft")
        .args([
            "--home",
            home_dir.to_str().unwrap(),
            "show-node-id"
        ])
        .output()
        .expect("could not get cometbft node id");

    let raw_id = String::from_utf8(out.stdout).unwrap();
    let node_id: String = raw_id.chars().filter(|c| !c.is_whitespace()).collect();

    node_id
}
