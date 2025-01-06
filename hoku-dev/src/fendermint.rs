// Copyright 2022-2024 Protocol Labs
// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
use std::fs::{write, read_to_string, File};
use std::path::Path;
use std::thread::JoinHandle;
use std::process::Child;
use toml_edit::{DocumentMut, value};
use colored::{ColoredString, Colorize};
use serde_json::Value;

use crate::util::{pipe_sub_command, get_rust_log_level, PipeSubCommandArgs};
use crate::{
    LogLevel, ABCI_PORTS
};

pub fn start_fendermint(
    fm_dir: &Path,
    label: &ColoredString,
    iroh_rpc_url: &str,
    cmt_rpc_url: &str,
    resolver_port: &str,
    log_level: &LogLevel
) -> (JoinHandle<()>, JoinHandle<()>, Child) {
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
            "--network",
            "testnet",
            "run",
            "--iroh-addr",
            iroh_rpc_url
        ].to_vec(),
        envs: Some(
            [
                // TODO: RUST_LOG and FM_LOG_LEVEL don't seem to affect logging
                ["RUST_LOG", rust_log].to_vec(),
                ["FM_LOG_LEVEL", &format!("{rust_log},fendermint={rust_log}")].to_vec(),
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

// initialize fendermint and generate the genesis files
pub fn init_fendermint(nodes: Vec<u8>, root_id: &str, chain_id: &str, nodes_dir: &Path, repo_root_dir: &Path, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);
    let genesis_file_path = nodes_dir.join("fm_genesis.json");
    let keys_dir = repo_root_dir.join("hoku-dev").join("config").join("keys");

    fm_copy_contracts(nodes.clone(), nodes_dir, repo_root_dir, log_level);
    genesis_create(&genesis_file_path, log_level);
    genesis_set_chain_id(chain_id, &genesis_file_path, log_level);
    genesis_add_accounts(&genesis_file_path, &keys_dir, log_level);
    genesis_add_validators(nodes.clone(), &genesis_file_path, &keys_dir, log_level);
    genesis_set_gateway(&genesis_file_path, root_id, log_level);
    genesis_seal(&genesis_file_path, nodes_dir, repo_root_dir, log_level);
    fm_update_config(nodes.clone(), nodes_dir, rust_log);

    fm_to_cmt(nodes.clone(), nodes_dir, log_level);

}

// TODO: for completeness I'm giving each fendermint a copy of the contract artifacts, but probably not necessary
fn fm_copy_contracts(nodes: Vec<u8>, nodes_dir: &Path, repo_root_dir: &Path, log_level: &LogLevel) {
    let contracts_dir = repo_root_dir.join("contracts").join("out").join("");
    let contracts_dir_str = contracts_dir.to_str().unwrap();

    for i in nodes.clone().into_iter() {
        let fm_dir = nodes_dir.join(format!("validator-{:?}", i)).join(format!("validator-{:?}", i)).join("fendermint");
        let (out, err, mut child) = pipe_sub_command(PipeSubCommandArgs {
            title: &format!("FM CONTRACTS COPY {:?}", i).blue(),
            cmd: "cp",
            args: [
                "-r",
                contracts_dir_str,
                fm_dir.join("contracts").to_str().unwrap()
            ].to_vec(),
            current_dir: None,
            envs: None,
            out_filters: vec!(),
            err_filters: vec!(),
            log_level
        });

        out.join().unwrap();
        err.join().unwrap();
        child.wait().unwrap();
    }
}

fn fm_update_config(nodes: Vec<u8>, nodes_dir: &Path, rust_log: &str) {
    // use the default fendermint config.toml file, but update specifically for each node in the network
    for i in nodes.clone().into_iter() {
        let fm_config_filepath = nodes_dir
            .join(format!("validator-{:?}", i))
            .join(format!("validator-{:?}", i))
            .join("fendermint")
            .join("config")
            .join("default.toml");
        let config_file = read_to_string(&fm_config_filepath).expect("could not read cometbft config file");
        let mut conf_doc = config_file.parse::<DocumentMut>().expect("invalid document");

        // TODO: setup separte ports for each node's metrics
        conf_doc["abci"]["listen"]["port"] = value(ABCI_PORTS[i as usize] as i64);
        conf_doc["tracing"]["console"]["level"] = value(rust_log);
        conf_doc["metrics"]["enabled"] = value(false);

        // TODO: we need to get snapshots working... there's a ticket for this
        //conf_doc["snapshots"]["enabled"] = value(true);

        write(&fm_config_filepath, conf_doc.to_string()).expect("could not write to cometbft config file");
    }
}

fn genesis_create(genesis_file_path: &Path, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);

    let (out, err, mut child) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("GENESIS CREATE").yellow().bold(),
        cmd: "./target/debug/fendermint",
        args: [
            "-n", "test",
            "genesis",
            "--genesis-file",
            genesis_file_path.to_str().unwrap(),
            "new",
            "--chain-name",
            "localnet",
            "--base-fee",
            "1000",
            "--timestamp",
            "1680101412",
            "--power-scale",
            "3"
        ].to_vec(),
        envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
        current_dir: None,
        out_filters: vec!(),
        err_filters: vec!(),
        log_level
    });

    out.join().unwrap();
    err.join().unwrap();
    child.wait().unwrap();
}

fn genesis_add_accounts(genesis_file_path: &Path, keys_dir: &Path, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);

    // add an account for each anvil key pair
    for i in 0..10 {
        let (out, err, mut child) = pipe_sub_command(PipeSubCommandArgs {
            title: &format!("GENESIS ADD ACCOUNT {:?}", i).yellow().bold(),
            cmd: "./target/debug/fendermint",
            args: [
                "-n", "test",
                "genesis",
                "--genesis-file",
                genesis_file_path.to_str().unwrap(),
                "add-account",
                "--public-key",
                keys_dir.join(format!("validator_key_{:?}.pk", i)).to_str().unwrap(),
                "--balance",
                "1000",
                "--kind",
                "ethereum"
            ].to_vec(),
            envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
            current_dir: None,
            out_filters: vec!(),
            err_filters: vec!(),
            log_level
        });

        out.join().unwrap();
        err.join().unwrap();
        child.wait().unwrap();
    }

}

fn genesis_add_validators(nodes: Vec<u8>, genesis_file_path: &Path, keys_dir: &Path, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);

    // Add validators to the Genesis file
    for i in nodes.into_iter() {
        let (out, err, mut child) = pipe_sub_command(PipeSubCommandArgs {
            title: &format!("GENESIS ADD VALIDATOR {:?}", i).bright_cyan().bold(),
            cmd: "./target/debug/fendermint",
            args: [
                "-n", "test",
                "genesis",
                "--genesis-file",
                genesis_file_path.to_str().unwrap(),
                "add-validator",
                "--public-key",
                keys_dir.join(format!("validator_key_{:?}.pk", i)).to_str().unwrap(),
                "--power",
                "1"
            ].to_vec(),
            envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
            current_dir: None,
            out_filters: vec!(),
            err_filters: vec!(),
            log_level
        });

        out.join().unwrap();
        err.join().unwrap();
        child.wait().unwrap();
    }
}

fn genesis_set_gateway(genesis_file_path: &Path, root_id: &str, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);

    let (out, err, mut child) = pipe_sub_command(PipeSubCommandArgs {
        title: &"GENESIS SETUP GATEWAY".to_string().bright_yellow().bold(),
        cmd: "./target/debug/fendermint",
        args: [
            "-n", "test",
            "genesis",
            "--genesis-file",
            genesis_file_path.to_str().unwrap(),
            "ipc",
            "gateway",
            "--subnet-id",
            root_id,
            "--bottom-up-check-period",
            "10",
            "--msg-fee",
            "1",
            "--majority-percentage",
            "65",
            "--active-validators-limit",
            "10"
        ].to_vec(),
        envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
        current_dir: None,
        out_filters: vec!(),
        err_filters: vec!(),
        log_level
    });

    out.join().unwrap();
    err.join().unwrap();
    child.wait().unwrap();
}

fn genesis_seal(genesis_file_path: &Path, nodes_dir: &Path, repo_root_dir: &Path, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);

    let (out, err, mut child) = pipe_sub_command(PipeSubCommandArgs {
        title: &"GENESIS SEAL".to_string().green().bold(),
        cmd: "./target/debug/fendermint",
        args: [
            "-n", "test",
            "genesis",
            "--genesis-file",
            genesis_file_path.to_str().unwrap(),
            "ipc",
            "seal-genesis",
            "--builtin-actors-path",
            repo_root_dir.join("fendermint").join("builtin-actors").join("output").join("bundle.car").to_str().unwrap(),
            "--custom-actors-path",
            repo_root_dir.join("fendermint").join("actors").join("output").join("custom_actors_bundle.car").to_str().unwrap(),
            "--artifacts-path",
            repo_root_dir.join("contracts").join("out").to_str().unwrap(),
            "--output-path",
            nodes_dir.join("fm_sealed_genesis.car").to_str().unwrap()
        ].to_vec(),
        envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
        current_dir: None,
        out_filters: vec!(),
        err_filters: vec!(),
        log_level
    });

    out.join().unwrap();
    err.join().unwrap();
    child.wait().unwrap();
}

fn fm_to_cmt(nodes: Vec<u8>, nodes_dir: &Path, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);

    let fm_genesis_path = nodes_dir.join("fm_genesis.json");
    let fm_genesis_path_str = fm_genesis_path.to_str().unwrap();
    let fm_sealed_genesis_path = nodes_dir.join("fm_sealed_genesis.car");
    let fm_sealed_genesis_path_str = fm_sealed_genesis_path.to_str().unwrap();

    // We need to merge the cometbft genesis validators with the genesis we are going to get from converting the fendermint genesis to cometbft.
    // This is needed (afaict) because the fendermint genesis has no knowledge of the cometbft validator details, e.g. keys, addresses, power, etc...
    let cmt_orig_genesis_path = nodes_dir
        .join("validator-0")
        .join("validator-0")
        .join("cometbft")
        .join("config")
        .join("genesis.json");

    let cmt_orig_genesis_str = read_to_string(&cmt_orig_genesis_path).expect("could not read cometbft genesis file");
    let cmt_orig_genesis: Value = serde_json::from_str(&cmt_orig_genesis_str).expect("unable to parse cometbft genesis JSON");
//
//    let fm_orig_genesis_str = read_to_string(&fm_genesis_path).expect("could not read fendermint genesis file");
//    let mut fm_orig_genesis: FmGenesis = serde_json::from_str(&fm_orig_genesis_str).expect("unable to parse fendermint genesis JSON");


//    cmt_genesis.validators =
//    println!("{}", serde_json::to_string_pretty(&config).expect("unable to serialize"));


    for i in nodes.into_iter() {
        let node_dir = nodes_dir.join(format!("validator-{:?}", i)).join(format!("validator-{:?}", i));
        let cmt_config_path = node_dir.join("cometbft").join("config");

        let (out, err, mut child) = pipe_sub_command(PipeSubCommandArgs {
            title: &format!("FM TO CMT KEY {:?}", i).bright_cyan().bold(),
            cmd: "./target/debug/fendermint",
            args: [
                "-n", "test",
                "key",
                "into-tendermint",
                "--secret-key",
                node_dir.join("fendermint").join("keys").join("validator_key.sk").to_str().unwrap(),
                "--out",
                cmt_config_path.join("priv_validator_key.json").to_str().unwrap()
            ].to_vec(),
            envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
            current_dir: None,
            out_filters: vec!(),
            err_filters: vec!(),
            log_level
        });

        out.join().unwrap();
        err.join().unwrap();
        child.wait().unwrap();

        let (out, err, mut child) = pipe_sub_command(PipeSubCommandArgs {
            title: &format!("FM TO CMT GENESIS {:?}", i).green().bold(),
            cmd: "./target/debug/fendermint",
            args: [
                "-n", "test",
                "genesis",
                "--genesis-file",
                fm_genesis_path_str,
                "into-tendermint",
                "--app-state",
                fm_sealed_genesis_path_str,
                "--out",
                cmt_config_path.join("genesis.json").to_str().unwrap()
            ].to_vec(),
            envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
            current_dir: None,
            out_filters: vec!(),
            err_filters: vec!(),
            log_level
        });

        out.join().unwrap();
        err.join().unwrap();
        child.wait().unwrap();

        let cmt_genesis_path = cmt_config_path.join("genesis.json");
        let cmt_genesis_str = read_to_string(&cmt_genesis_path).expect("could not read cometbft genesis file");
        let mut cmt_genesis: Value = serde_json::from_str(&cmt_genesis_str).expect("unable to parse cometbft genesis JSON");

        cmt_genesis["validators"] = cmt_orig_genesis["validators"].clone();
        cmt_genesis["consensus_params"]["validator"] = cmt_orig_genesis["consensus_params"]["validator"].clone();
        cmt_genesis["initial_height"] = Value::from("0");

        let cmt_genesis_file = File::create(&cmt_genesis_path).unwrap();
        serde_json::to_writer_pretty(cmt_genesis_file, &cmt_genesis).expect("could not write cometbft genesis");
    }
}

fn genesis_set_chain_id(chain_id: &str, genesis_file_path: &Path, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);
    // fendermint genesis --genesis-file <GENESIS_FILE> set-chain-id --chain-id <CHAIN_ID>
    let (out, err, mut child) = pipe_sub_command(PipeSubCommandArgs {
        title: &"SET CHAIN ID".to_string().magenta().on_yellow().bold(),
        cmd: "./target/debug/fendermint",
        args: [
            "genesis",
            "--genesis-file",
            genesis_file_path.to_str().unwrap(),
            "set-chain-id",
            "--chain-id",
            chain_id
        ].to_vec(),
        envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level
    });

    out.join().unwrap();
    err.join().unwrap();
    child.wait().unwrap();
}
