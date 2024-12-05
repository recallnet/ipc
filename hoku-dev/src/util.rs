// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use colored::{ColoredString, Colorize};
use regex::Regex;
use toml_edit::{DocumentMut, value};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::thread::JoinHandle;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::fs::{read_to_string, write, File};
use serde::Serialize;
use chrono::{DateTime, Local};

use crate::contracts::ContractMap;
use crate::LogLevel;
use crate::anvil::{ANVIL_PRIVATE_KEYS, ANVIL_PUBLIC_KEYS};

pub const THIRTY_SECONDS: Duration = Duration::from_secs(30);
pub const THREE_SECONDS: Duration = Duration::from_secs(3);
pub const NODE_PREFIX: &str = "validator-";

#[derive(Serialize)]
struct Keystore {
    address: String,
    private_key: String,
}

pub fn sleep_thirty(log_level: &LogLevel) {
    let time_now: DateTime<Local> = Local::now();
    log_level_print(
        &String::from("STARTUP").white().bold(),
        &format!("[{:?}] - wait 30 seconds", time_now),
        log_level,&LogLevel::Info,
        &vec![]
    );
    thread::sleep(THIRTY_SECONDS);
}
pub fn sleep_three(log_level: &LogLevel) {
    let time_now: DateTime<Local> = Local::now();
    log_level_print(&String::from("STARTUP").white().bold(),&format!("[{:?}] - wait 3 seconds", time_now), log_level,&LogLevel::Info, &vec![]);
    thread::sleep(THREE_SECONDS);
}

#[rustfmt::skip]
pub fn print_logo(label: &ColoredString, log_level: &LogLevel) {
    let print = |message: &str, message_level: &LogLevel| {
        log_level_print(label, message, log_level, message_level, &vec![]);
    };

    print("", &LogLevel::Quiet);
    print("               hoku started", &LogLevel::Quiet);
    print("--------------------------------------------------", &LogLevel::Quiet);
    print("", &LogLevel::Quiet);
    print("   \\\\\\\\\\\\\\\\\\\\\\              \\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("   \\\\\\\\\\\\\\\\\\\\\\\\\\            \\\\\\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("   \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\          \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("     \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\          \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("       \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\          \\\\\\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("         \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\          \\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("           \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("             \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("   \\\\\\\\\\\\\\\\\\\\\\\\           \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("   \\\\\\\\\\\\\\\\\\\\\\\\\\\\           \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("   \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\           \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("     \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\           \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("       \\\\\\\\\\\\\\\\\\\\\\\\\\\\             \\\\\\\\\\\\\\\\\\\\\\\\\\", &LogLevel::Quiet);
    print("         \\\\\\\\\\\\\\\\\\\\\\\\               \\\\\\\\\\\\\\\\\\\\\\ ", &LogLevel::Quiet);
    print("", &LogLevel::Quiet);
    print("--------------------------------------------------", &LogLevel::Quiet);
    print("", &LogLevel::Quiet);
}

// Helper that both allows suppressing logs based on a user defined logging
// prefference and a set of regex filters that remove extraneous messages
pub fn log_level_print(
    label: &ColoredString,
    words: &str,
    log_level: &LogLevel,
    message_level: &LogLevel,
    filters: &Vec<Result<Regex, regex::Error>>,
) {
    if matches!(log_level, LogLevel::Debug) {
        // if logging is set to debug we don't apply the filters
        filtered_print(label, words, &vec![]);
        return;
    }
    if log_level >= message_level {
        filtered_print(label, words, filters);
    }
}

pub fn filtered_print(
    label: &ColoredString,
    words: &str,
    filters: &Vec<Result<Regex, regex::Error>>,
) {
    let mut suppress = false;
    for i in filters {
        let filter = i.as_ref().unwrap();
        if suppress {
            break;
        }
        if filter.is_match(words) {
            suppress = true;
        }
    }
    if !suppress {
        println!("[{}] {}", label, words);
    }
}

pub struct PipeSubCommandArgs<'a> {
    pub title: &'a ColoredString,
    pub cmd: &'a str,
    pub args: Vec<&'a str>,
    pub envs: Option<Vec<Vec<&'a str>>>,
    pub current_dir: Option<&'a str>,
    pub out_filters: Vec<Result<Regex, regex::Error>>,
    pub err_filters: Vec<Result<Regex, regex::Error>>,
    pub log_level: &'a LogLevel,
}

pub fn pipe_sub_command(args: PipeSubCommandArgs) -> (JoinHandle<()>, JoinHandle<()>) {
    let lblout = args.title.clone();
    let lblerr = args.title.clone();

    let mut cmd: &mut Command = &mut Command::new(args.cmd);

    if let Some(envs) = args.envs {
        for env in &envs {
            if env.len() == 2 {
                cmd = cmd.env(env[0], env[1]);
            }
        }
    }
    if let Some(current_dir) = args.current_dir {
        cmd.current_dir(current_dir);
    }

    let mut command_out = cmd
        .args(args.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("could not run command {:?}: Error: {:?}", cmd, err));
    let stdout = command_out
        .stdout
        .take()
        .unwrap_or_else(|| panic!("could not take {} stdout", lblout));
    let stderr = command_out
        .stderr
        .take()
        .unwrap_or_else(|| panic!("could not take {} stderr", lblerr));

    let out_log = *args.log_level;
    let stdout_thread = thread::spawn(move || {
        let stdout_lines = BufReader::new(stdout).lines();
        for line in stdout_lines {
            let line = line.unwrap();
            log_level_print(&lblout, &line, &out_log, &LogLevel::Info, &args.out_filters);
        }
    });

    // TODO: all of our make tasks pipe info through stderr, figure out why
    let err_log = *args.log_level;
    let stderr_thread = thread::spawn(move || {
        let stderr_lines = BufReader::new(stderr).lines();
        for line in stderr_lines {
            let line = line.unwrap();
            log_level_print(&lblerr, &line, &err_log, &LogLevel::Info, &args.err_filters);
        }
    });

    (stdout_thread, stderr_thread)
}

// TODO: this should take the log_level. if debug, print the paths all these commands use
pub fn init_node_dir(ipc_config_dir: &Path, network_dir: &Path, repo_root_dir: &Path, nodes: Vec<u8>) {
    Command::new("rm")
        .args(["-rf", ipc_config_dir.to_str().unwrap()])
        .output()
        .expect("failed to rm .ipc");
    Command::new("mkdir")
        .args([ipc_config_dir.to_str().unwrap()])
        .output()
        .expect("failed to mkdir .ipc");

    for i in nodes.into_iter() {
        let validator_dir = network_dir.join(format!("validator-{:?}", i)).join(format!("validator-{:?}", i));
        Command::new("mkdir")
            .args(["-p", validator_dir.join("cometbft").join("config").to_str().unwrap()])
            .output()
            .expect("failed to create cometbft dir");
        Command::new("mkdir")
            .args(["-p", validator_dir.join("cometbft").join("data").to_str().unwrap()])
            .output()
            .expect("failed to create cometbft dir");

        Command::new("mkdir")
            .args([
                "-p",
                validator_dir
                    .join("fendermint")
                    .join("data")
                    .to_str()
                    .unwrap()
            ])
            .output()
            .expect("failed to create fendermint data dir");
        let fm_config_dir = validator_dir
            .join("fendermint")
            .join("config");
        Command::new("mkdir")
            .args([
                "-p",
                fm_config_dir
                    .to_str()
                    .unwrap()
            ])
            .output()
            .expect("failed to create fendermint config dir");
        let fm_keys_dir = validator_dir
            .join("fendermint")
            .join("keys");
        Command::new("mkdir")
            .args([
                "-p",
                fm_keys_dir
                    .to_str()
                    .unwrap()
            ])
            .output()
            .expect("failed to create fendermint keys dir");

        // copy config and keys
        Command::new("cp")
            .args([
                repo_root_dir.join("hoku-dev")
                    .join("config")
                    .join("fendermint")
                    .join("default.toml")
                    .to_str()
                    .unwrap(),
                fm_config_dir
                    .join("default.toml")
                    .to_str()
                    .unwrap()
            ])
            .output()
            .expect("failed to create fendermint config dir");

        Command::new("mkdir")
            .args(["-p", validator_dir.join("iroh").to_str().unwrap()])
            .output()
            .expect("failed to create iroh dir");
        Command::new("mkdir")
            .args([
                "-p",
                validator_dir.join("iroh").join("blobs").to_str().unwrap()
            ])
            .output()
            .expect("failed to create iroh blobs dir");

        let iroh_config_dir = validator_dir
            .join("iroh")
            .join("config");
        Command::new("mkdir")
            .args([
                "-p",
                iroh_config_dir.to_str().unwrap()
            ])
            .output()
            .expect("failed to create iroh blobs dir");

        // copy iroh config into this nodes directory
        Command::new("cp")
            .args([
                repo_root_dir.join("hoku-dev")
                    .join("config")
                    .join("iroh")
                    .join("iroh.config.toml")
                    .to_str()
                    .unwrap(),
                iroh_config_dir
                    .join("iroh.config.toml")
                    .to_str()
                    .unwrap()
            ])
            .output()
            .expect("failed to copy iroh config");

        let default_keys_dir = repo_root_dir
            .join("hoku-dev")
            .join("config")
            .join("keys");
        // TODO: convert the default anvil accounts to this format
        Command::new("cp")
            .args([
                default_keys_dir.join(format!("validator_key_{:?}.sk", i)).to_str().unwrap(),
                fm_keys_dir
                    .join("validator_key.sk")
                    .to_str()
                    .unwrap()
            ])
            .output()
            .expect("failed to copy key file");
        Command::new("cp")
            .args([
                default_keys_dir.join(format!("network_key_{:?}.sk", i)).to_str().unwrap(),
                fm_keys_dir
                    .join("network.sk")
                    .to_str()
                    .unwrap()
            ])
            .output()
            .expect("failed to copy key file");
    }
}

pub fn get_rust_log_level(log_level: &LogLevel) -> &str {
    match log_level {
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Quiet => "error",
        LogLevel::Silent => ""
    }
}

pub fn get_ipc_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap();
    Path::new(&home).join(".ipc")
}

pub fn create_keystore(filepath: &Path) {
    let file = File::create(filepath).unwrap();
    let mut keys: Vec<Keystore> = Vec::new();

    for i in 0..(ANVIL_PUBLIC_KEYS.len()) {
        let key = Keystore {
            address: ANVIL_PUBLIC_KEYS[i].to_string(),
            private_key: ANVIL_PRIVATE_KEYS[i].to_string(),
        };
        keys.push(key);
    }

    serde_json::to_writer_pretty(&file, &keys).unwrap();
}

// copy subnet config files
pub fn setup_subnet_config(
    repo_root_dir: &Path,
    config_folder: &Path,
    contracts: &ContractMap,
    parent_rpc_url: &str,
    child_rpc_url: &str,
    parent_chain_id: &str,
    subnet_id: &str,
    log_level: &LogLevel
) {
    // put ipc config in ~/.ipc/
    Command::new("cp")
        .args([
            repo_root_dir.join("hoku-dev").join("config").join("config.toml").to_str().unwrap(),
            config_folder.to_str().unwrap(),
        ])
        .output()
        .expect("failed to copy ipc config");

    setup_ipc_config_file(&config_folder.join("config.toml"), contracts, parent_rpc_url, child_rpc_url, parent_chain_id, subnet_id);

    // put relayer config in ~/.ipc/
    Command::new("cp")
        .args([
            repo_root_dir.join("hoku-dev").join("config").join("relayer.config.toml").to_str().unwrap(),
            config_folder.to_str().unwrap(),
        ])
        .output()
        .expect("failed to copy relayer config");

    setup_ipc_config_file(&config_folder.join("relayer.config.toml"), contracts, parent_rpc_url, child_rpc_url, parent_chain_id, subnet_id);
    create_keystore(&config_folder.join("evm_keystore.json"));

    log_level_print(
        &String::from("STARTUP").white().bold(),
        "subnet and relayer config files setup",
        log_level,
        &LogLevel::Info,
        &vec!()
    );
}

fn setup_ipc_config_file(
    config_filepath: &Path,
    contracts: &ContractMap,
    parent_rpc_url: &str,
    child_rpc_url: &str,
    parent_chain_id: &str,
    subnet_id: &str
) {

    // we use our default cometbft config.toml file, but we need to update to use the config for this network
    let config_file = read_to_string(config_filepath).expect("could not modify ipc config");
    let mut conf_doc = config_file.parse::<DocumentMut>().expect("invalid ipc config document");

    // TODO: setup separte ports for each node's metrics
    conf_doc["subnets"][0]["id"] = value(format!("/r{parent_chain_id}"));
    conf_doc["subnets"][0]["config"]["provider_http"] = value(parent_rpc_url);
    conf_doc["subnets"][0]["config"]["registry_addr"] = value(&contracts.registry);
    conf_doc["subnets"][0]["config"]["gateway_addr"] = value(&contracts.gateway);
    conf_doc["subnets"][1]["id"] = value(subnet_id);
    conf_doc["subnets"][1]["config"]["provider_http"] = value(child_rpc_url);

    write(config_filepath, conf_doc.to_string()).expect("could not write to ipc config file");
}

pub fn get_forge_deployed_address(deploy_json_file: &Path) -> String {
    let data = read_to_string(deploy_json_file).expect("Unable to read file");
    let json: serde_json::Value = serde_json::from_str(&data).expect("JSON was not well-formatted");

    let address = String::from(json["returns"]["0"]["value"].as_str().unwrap());
    address
}

pub fn get_hardhat_deployed_address(deploy_json_file: &Path) -> String {
    let data = read_to_string(deploy_json_file).expect("Unable to read file");
    let json: serde_json::Value = serde_json::from_str(&data).expect("JSON was not well-formatted");

    let address = String::from(json["address"].as_str().unwrap());
    address
}
