// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use colored::ColoredString;
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::thread::JoinHandle;
use std::path::Path;
use std::time;

use crate::{LogLevel, NETWORK_NAME};

pub const THIRTY_SECONDS: time::Duration = time::Duration::from_secs(30);
pub const NODE_PREFIX: &str = "validator-";

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
pub fn init_node_dir(ipc_config_dir: &Path, repo_root_dir: &Path, node_count: u8) {
    Command::new("rm")
        .args(["-rf", ipc_config_dir.to_str().unwrap()])
        .output()
        .expect("failed to rm .ipc");
    Command::new("mkdir")
        .args([ipc_config_dir.to_str().unwrap()])
        .output()
        .expect("failed to mkdir .ipc");

    for i in 0..node_count {
        let validator_dir = ipc_config_dir.join(NETWORK_NAME).join(format!("validator-{:?}", i)).join(format!("validator-{:?}", i));
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
                repo_root_dir.join("hoku-dev-cli")
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
                repo_root_dir.join("hoku-dev-cli")
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
            .join("hoku-dev-cli")
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
