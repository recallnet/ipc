// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
use colored::Colorize;
use std::path::Path;
use std::process::{Command, Output};


use crate::anvil::{
    ANVIL_PRIVATE_KEYS, ANVIL_PUBLIC_KEYS
};
use crate::util::{
    log_level_print,
    pipe_sub_command,
    get_rust_log_level,
    sleep_three,
    get_forge_deployed_address,
    get_hardhat_deployed_address,
    PipeSubCommandArgs
};
use crate::LogLevel;

pub struct ContractMap {
    pub account_helper: String,
    pub subnet_id_helper: String,
    pub cross_msg_helper: String,
    pub lib_staking: String,
    pub lib_quorum: String,
    pub gateway: String,
    pub registry: String,
    pub supply_source_address: String,
    pub gater: String
}

pub fn build_contracts(repo_root_dir: &Path, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);
    // need to run clean or we hit upgradeable saftey validation errors resulting
    // from contracts with the same name
    let (clean_out, clean_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("FORGE CLEAN").bright_green().bold(),
        cmd: "forge",
        envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
        current_dir: repo_root_dir.join("hoku-contracts").to_str(),
        args: ["clean"].to_vec(),
        log_level,
        out_filters: vec![],
        err_filters: vec![]
    });

    clean_out.join().unwrap();
    clean_err.join().unwrap();

    let (build_out, build_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("FORGE BUILD").bright_yellow().bold(),
        cmd: "forge",
        // TODO: I don't know for sure which subprocesses use this env, but passing it in just in case
        envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
        current_dir: repo_root_dir.join("hoku-contracts").to_str(),
        args: ["build"].to_vec(),
        log_level,
        out_filters: vec![],
        err_filters: vec![]
    });

    build_out.join().unwrap();
    build_err.join().unwrap();

    // remove old deployment artifacts
    Command::new("rm")
        .args(["-rf", repo_root_dir.join("contracts").join("deployments").join("localnet").to_str().unwrap()])
        .output()
        .expect("failed to rm old localnet deployments");

}

pub fn deploy_libraries(repo_root_dir: &Path, log_level: &LogLevel) -> ContractMap {
    let (lib_out, lib_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("DEPLOY LIBRARIES").magenta().bold(),
        cmd: "npx",
        envs: Some([
            ["RPC_URL", "http://127.0.0.1:8545"].to_vec(),
            ["PRIVATE_KEY", ANVIL_PRIVATE_KEYS[0]].to_vec(),
            ["CHAIN_ID", "31337"].to_vec()
        ].to_vec()),
        current_dir: repo_root_dir.join("contracts").to_str(),
        args: ["hardhat", "deploy-libraries", "--network", "localnet"].to_vec(),
        log_level,
        out_filters: vec![],
        err_filters: vec![]
    });

    // We have to wait for the Libs to be deployed since other contracts use them
    lib_out.join().unwrap();
    lib_err.join().unwrap();

    ContractMap {
        account_helper: get_hardhat_deployed_address(
            &repo_root_dir
            .join("contracts")
            .join("deployments")
            .join("localnet")
            .join("AccountHelper.json")
        ),
        subnet_id_helper: get_hardhat_deployed_address(
            &repo_root_dir
            .join("contracts")
            .join("deployments")
            .join("localnet")
            .join("AccountHelper.json")
        ),
        cross_msg_helper: get_hardhat_deployed_address(
            &repo_root_dir
            .join("contracts")
            .join("deployments")
            .join("localnet")
            .join("CrossMsgHelper.json")
        ),
        lib_staking: get_hardhat_deployed_address(
            &repo_root_dir
            .join("contracts")
            .join("deployments")
            .join("localnet")
            .join("LibStaking.json")
        ),
        lib_quorum: get_hardhat_deployed_address(
            &repo_root_dir
            .join("contracts")
            .join("deployments")
            .join("localnet")
            .join("LibQuorum.json")
        ),
        gateway: String::from(""),
        registry: String::from(""),
        supply_source_address: String::from(""),
        gater: String::from(""),
    }
}

pub fn deploy_gater(anvil_rpc_url: &str, repo_root_dir: &Path, log_level: &LogLevel) -> String {
    let (gater_out, gater_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("DEPLOY VALIDATOR GATER").bright_green().bold(),
        cmd: "forge",
        args: [
            "script",
            "script/ValidatorGater.s.sol",
            "--private-key",
            ANVIL_PRIVATE_KEYS[0],
            "--rpc-url",
            anvil_rpc_url,
            "--tc",
            "DeployScript",
            "--sig",
            "run()",
            "--broadcast",
            "--timeout",
            "120",
            "-g",
            "130",
            "-vv"
        ].to_vec(),
        envs: None,
        current_dir: repo_root_dir.join("hoku-contracts").to_str(),
        out_filters: vec![],
        err_filters: vec![],
        log_level
    });

    gater_out.join().unwrap();
    gater_err.join().unwrap();

    get_forge_deployed_address(
        &repo_root_dir
        .join("hoku-contracts")
        .join("broadcast")
        .join("ValidatorGater.s.sol")
        .join("31337")
        .join("run-latest.json")
    )
}

pub fn setup_gater(
    anvil_rpc_url: &str,
    gater_contract: &str,
    subnet_eth_address: &str,
    nodes: Vec<u8>,
    repo_root_dir: &Path,
    log_level: &LogLevel
) {

    // Approve each validator to stake
    let gater_label = String::from("APPROVE VALIDATOR GATE").bright_green().bold();
    for i in nodes.into_iter() {
        let out = approve_validator_gate(
            // public key of validator being approved
            ANVIL_PUBLIC_KEYS[i as usize],
            // validator-0 is owner, only they can approve
            ANVIL_PRIVATE_KEYS[0],
            gater_contract,
            anvil_rpc_url
        );
        log_level_print(&gater_label, &format!("{:?}", out), log_level, &LogLevel::Info, &vec![]);
    }

    let (subnet_gate_out, subnet_gate_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("VALIDATOR GATER SET SUBNET").bright_green().bold(),
        cmd: "cast",
        args: [
            "send",
            "--private-key",
            ANVIL_PRIVATE_KEYS[0],
            "--rpc-url",
            anvil_rpc_url,
            "--timeout",
            "120",
            gater_contract,
            "setSubnet((uint64,address[]))",
            &format!("(31337, [{}])", subnet_eth_address)
        ].to_vec(),
        envs: None,
        current_dir: repo_root_dir.join("hoku-contracts").to_str(),
        out_filters: vec![],
        err_filters: vec![],
        log_level
    });

    subnet_gate_out.join().unwrap();
    subnet_gate_err.join().unwrap();

}

fn approve_validator_gate(
    public_key: &str,
    private_key: &str,
    validator_gater_address: &str,
    anvil_rpc_url: &str
) -> Output {
    let out = Command::new("cast")
        .args([
            "send",
            "--private-key",
            private_key,
            "--rpc-url",
            anvil_rpc_url,
            "--timeout",
            "120",
            validator_gater_address,
            "approve(address,uint256,uint256)",
            public_key,
            "1000000000000000000",
            "100000000000000000000"
        ])
        .output()
        .expect("failed to fund wallet with source token");
    out
}

pub fn deploy_gateway(contracts: &ContractMap, repo_root_dir: &Path, log_level: &LogLevel) -> String {

    let (gate_out, gate_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("DEPLOY GATEWAY").blue().on_yellow().bold(),
        cmd: "npx",
        envs: Some([
            ["RPC_URL", "http://127.0.0.1:8545"].to_vec(),
            ["PRIVATE_KEY", ANVIL_PRIVATE_KEYS[0]].to_vec(),
            ["CHAIN_ID", "31337"].to_vec(),
            ["ACCOUNT_HELPER", &contracts.account_helper].to_vec(),
            ["SUBNET_ID_HELPER", &contracts.subnet_id_helper].to_vec(),
            ["CROSS_MSG_HELPER", &contracts.cross_msg_helper].to_vec(),
            ["LIB_STAKING", &contracts.lib_staking].to_vec(),
            ["LIB_QUORUM", &contracts.lib_quorum].to_vec()
        ].to_vec()),
        current_dir: repo_root_dir.join("contracts").to_str(),
        args: ["hardhat", "deploy-gateway", "--network", "localnet"].to_vec(),
        log_level,
        out_filters: vec![],
        err_filters: vec![]
    });

    // TODO: calling join ensures that contract deployment happens in series, which is slow
    //  but required for some contracts that depend on previous deployments. At some point
    //  we could optimise running some of these in parallel?
    gate_out.join().unwrap();
    gate_err.join().unwrap();

    get_hardhat_deployed_address(
        &repo_root_dir
        .join("contracts")
        .join("deployments")
        .join("localnet")
        .join("GatewayDiamond.json")
    )
}

pub fn deploy_registry(contracts: &ContractMap, repo_root_dir: &Path, log_level: &LogLevel) -> String {

    let (reg_out, reg_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("DEPLOY REGISTRY").cyan().bold(),
        cmd: "npx",
        envs: Some([
            ["RPC_URL", "http://127.0.0.1:8545"].to_vec(),
            ["PRIVATE_KEY", ANVIL_PRIVATE_KEYS[0]].to_vec(),
            ["CHAIN_ID", "31337"].to_vec(),
            ["REGISTRY_CREATION_PRIVILEGES", "unrestricted"].to_vec(),
            ["ACCOUNT_HELPER", &contracts.account_helper].to_vec(),
            ["SUBNET_ID_HELPER", &contracts.subnet_id_helper].to_vec(),
            ["CROSS_MSG_HELPER", &contracts.cross_msg_helper].to_vec(),
            ["LIB_STAKING", &contracts.lib_staking].to_vec(),
            ["LIB_QUORUM", &contracts.lib_quorum].to_vec(),
            ["GATEWAY_ADDRESS", &contracts.gateway].to_vec()
        ].to_vec()),
        current_dir: repo_root_dir.join("contracts").to_str(),
        args: ["hardhat", "deploy-registry", "--network", "localnet"].to_vec(),
        log_level,
        out_filters: vec![],
        err_filters: vec![]
    });

    // TODO: calling join ensures that contract deployment happens in series, which is slow
    reg_out.join().unwrap();
    reg_err.join().unwrap();

    get_hardhat_deployed_address(
        &repo_root_dir
        .join("contracts")
        .join("deployments")
        .join("localnet")
        .join("SubnetRegistryDiamond.json")
    )
}

pub fn deploy_supply_source(anvil_rpc_url: &str, repo_root_dir: &Path, log_level: &LogLevel) -> String {
    let rust_log = get_rust_log_level(log_level);

    let (supply_out, supply_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("DEPLOY SUPPLY SOURCE").purple().bold(),
        cmd: "forge",
        envs: Some([
            ["RUST_LOG", rust_log].to_vec(),
            ["RPC_URL", anvil_rpc_url].to_vec(),
            ["PRIVATE_KEY", &format!("0x{}", ANVIL_PRIVATE_KEYS[0])].to_vec(),
            ["CHAIN_ID", "31337"].to_vec(),
        ].to_vec()),
        current_dir: repo_root_dir.join("hoku-contracts").to_str(),
        args: [
            "script",
            "--private-key",
            ANVIL_PRIVATE_KEYS[0],
            "--rpc-url",
            anvil_rpc_url,
            "--tc",
            "DeployScript",
            "--sig",
            "run(string)",
            "--broadcast",
            "--timeout",
            "120",
            "-vv",
            "script/Hoku.s.sol",
            "testnet",
        ].to_vec(),
        log_level,
        out_filters: vec![],
        err_filters: vec![]
    });

    supply_out.join().unwrap();
    supply_err.join().unwrap();

    get_forge_deployed_address(
        &repo_root_dir
        .join("hoku-contracts")
        .join("broadcast")
        .join("Hoku.s.sol")
        .join("31337")
        .join("run-latest.json")
    )
}

pub fn create_subnet(supply_source_address: &str, gater_address: &str, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);
    // note: the config used to run this command comes from `~/.ipc/config.toml`, which is copied from `hoku-dev/config/config.toml` during startup
    let (subnet_out, subnet_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("CREATE SUBNET").yellow().bold(),
        cmd: "./target/debug/ipc-cli",
        envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
        current_dir: None,
        args: [
            "subnet",
            "create",
            "--from",
            ANVIL_PUBLIC_KEYS[0],
            "--parent",
            "/r31337",
            "--min-validators",
            "1",
            "--min-validator-stake",
            "1",
            "--bottomup-check-period",
            "10",
            "--active-validators-limit",
            "10",
            "--permission-mode",
            "collateral",
            "--supply-source-kind",
            "erc20",
            "--supply-source-address",
            supply_source_address,
            "--validator-gater",
            gater_address,
            "--collateral-source-kind",
            "erc20",
            "--collateral-source-address",
            supply_source_address
        ].to_vec(),
        log_level,
        out_filters: vec![],
        err_filters: vec![]
    });

    subnet_out.join().unwrap();
    subnet_err.join().unwrap();

    
}

pub fn join_subnet(anvil_rpc_url: &str, subnet_eth_address: &str, subnet_id: &str, supply_source_address: &str, node_number: u8, log_level: &LogLevel) {
    let rust_log = get_rust_log_level(log_level);

  // Approve subnet contract to lock up to 10 HOKU from collateral contract (which is also the supply source contract)
    let (lock_collateral_out, lock_collateral_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &format!("APPROVE {:?}", node_number).bright_magenta().bold(),
        cmd: "cast",
        args: [
            "send",
            "--private-key",
            ANVIL_PRIVATE_KEYS[node_number as usize],
            "--rpc-url",
            anvil_rpc_url,
            "--timeout",
            "120",
            supply_source_address,
            "approve(address,uint256)",
            subnet_eth_address,
            "10000000000000000000"
        ].to_vec(),
        envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level
    });

    lock_collateral_out.join().unwrap();
    lock_collateral_err.join().unwrap();

    // Join and stake 10 HOKU
    //ipc-cli subnet join --from "${wallet_addresses[i]}" --subnet "$subnet_id" --collateral 10
    let (join_subnet_out, join_subnet_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &format!("JOIN SUBNET {:?}", node_number).magenta().on_yellow().bold(),
        cmd: "./target/debug/ipc-cli",
        args: [
            "subnet",
            "join",
            "--from",
            ANVIL_PUBLIC_KEYS[node_number as usize],
            "--subnet",
            subnet_id,
            "--collateral",
            "10"
        ].to_vec(),
        envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level
    });

    join_subnet_out.join().unwrap();
    join_subnet_err.join().unwrap();
}

pub fn buy_credits(subnet_eth_rpc_url: &str, subnet_id: &str, log_level: &LogLevel) {
    let token_amount = "10000000000000000000000";
    let rust_log = get_rust_log_level(log_level);
    for i in 0..ANVIL_PUBLIC_KEYS.len() {
        let addr = ANVIL_PUBLIC_KEYS[i];

        // send tokens from the parent erc20 contract to the subnet erc20 token
        let (stdout, stderr) = pipe_sub_command(PipeSubCommandArgs {
            title: &format!("FUND WITH TOKEN {:?}", i).bright_purple().bold(),
            cmd: "./target/debug/ipc-cli",
            envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
            current_dir: None,
            args: [
                "cross-msg",
                "fund-with-token",
                "--subnet",
                subnet_id,
                "--from",
                addr,
                "--to",
                addr,
                "--approve", // approve gateway before funding
                token_amount
            ].to_vec(),
            out_filters: vec![],
            err_filters: vec![],
            log_level
        });

        stdout.join().unwrap();
        stderr.join().unwrap();

        // let the transactions process
        sleep_three(log_level);
    }

    let label = String::from("CHECK TOKEN").purple().bold();
    let print = |message: &str, message_level: &LogLevel| {
        log_level_print(&label, message, log_level, message_level, &vec![]);
    };

    let mut stop = false;
    loop {
        print("checking credit deposit", &LogLevel::Info);

        let addr = ANVIL_PUBLIC_KEYS[9];
        let output = Command::new("cast")
            .args([
                "balance",
                "--rpc-url",
                subnet_eth_rpc_url,
                "--ether",
                addr
            ])
            .output()
            .expect("Failed to execute command");
        println!("cast balance out: {:?}", output);
        let balance = String::from_utf8(output.stdout).expect("Invalid UTF-8 sequence");
        print(&format!("balance: {balance}"), &LogLevel::Info);

        //if balance.trim().parse::<u64>().unwrap() != 0 {
        if stop {
            break;
        }
        stop = true;
        sleep_three(log_level);
    }

    let label = String::from("BUY CREDIT").yellow().bold();
//    let print = |message: &str, message_level: &LogLevel| {
//        log_level_print(&label, message, log_level, message_level, &vec![]);
//    };
    let credit_amount = "5000";

    for i in 0..ANVIL_PRIVATE_KEYS.len() {
        let private_key = ANVIL_PRIVATE_KEYS[i];

        let (stdout, stderr) = pipe_sub_command(PipeSubCommandArgs {
            title: &label,
            cmd: "hoku",
            args: [
                "credit",
                "buy",
                credit_amount
            ].to_vec(),
            envs: Some([
                ["HOKU_PRIVATE_KEY", private_key].to_vec(),
                ["HOKU_NETWORK", "localnet"].to_vec()
            ].to_vec()),
            current_dir: None,
            out_filters: vec![],
            err_filters: vec![],
            log_level
        });

        stdout.join().unwrap();
        stderr.join().unwrap();
    }

}

// mint the supply token on the parent chain
pub fn fund_wallet_token(
    public_key: &str,
    private_key: &str,
    token_amount: &str,
    supply_source_address: &str,
    parent_rpc_url: &str
) -> Output {
    //cast send --private-key "$pk" --rpc-url "$rpc_url" --timeout 120 "$SUPPLY_SOURCE_ADDRESS" "mint(address,uint256)" "$addr" "$token_amount"
    Command::new("cast")
        .args([
            "send",
            "--private-key",
            private_key,
            "--rpc-url",
            parent_rpc_url,
            supply_source_address,
            "mint(address,uint256)",
            public_key,
            token_amount,
        ])
        .output()
        .expect("failed to fund wallet with source token")
}
