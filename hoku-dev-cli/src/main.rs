// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
mod util;

use toml_edit::{DocumentMut, value};
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use regex::Regex;
use serde::Serialize;
use std::fs::{write, File, read_to_string};
use std::path::Path;
use std::process::{Command, Output};
use std::thread;
use std::thread::sleep;
use std::thread::JoinHandle;
use std::time;
use util::pipe_sub_command;

use crate::util::{log_level_print, print_logo, get_rust_log_level, PipeSubCommandArgs};

#[derive(Serialize)]
struct Keystore {
    address: String,
    private_key: String,
}

struct ContractMap<'a> {
    account_helper: &'a str,
    subnet_id_helper: &'a str,
    cross_msg_helper: &'a str,
    lib_staking: &'a str,
    lib_quorum: &'a str,
    gateway: &'a str,
    registry: &'a str,
    supply_source_address: &'a str,
    gater: &'a str
}

struct NodeConfig<'a> {
    node_name: &'a str,
    repo_root_dir: &'a Path,
    ipc_config_dir: &'a Path,
    contracts:&'a  ContractMap<'a>,
    node_number: u8,
    is_bootstrap: bool,
    log_level: &'a LogLevel,
}

// note: the subnet id always has the same value so putting it here
// TODO: we can probably get this from the subnet create command so we can use this on pre-existing nets
const SUBNET_ID: &str = "/r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly";
// Note: to get subnet eth addr do `./target/debug/ipc-cli util f4-to-eth-addr --addr "t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly"`
const SUBNET_ETH_ADDRESS: &str = "0x56639db16ac50a89228026e42a316b30179a5376";
const NETWORK_NAME: &str = "r31337-t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly";

const IROH_RPC_PORTS: [u16; 10] = [4919, 4920, 4921, 4922, 4923, 4924, 4925, 4926, 4927, 4928];
const CMT_RPC_PORTS: [u16; 10] = [26657, 26658, 26659, 26660, 26661, 26662, 26663, 26664, 26665, 26666];
const CMT_P2P_PORTS: [u16; 10] = [26756, 26757, 26758, 26759, 26760, 26761, 26762, 26763, 26764, 26765];
const FM_RESOLVER_PORTS: [u16; 10] = [26855, 26856, 26857, 26858, 26859, 26860, 26861, 26862, 26863, 26864];
const ETHAPI_PORTS: [u16; 10] = [8645, 8646, 8647, 8648, 8649, 8650, 8651, 8652, 8653, 8654];
const OBJECTS_PORTS: [u16; 10] = [8001, 8002, 8003, 8004, 8005, 8006, 8007, 8008, 8009, 8010];
const ABCI_PORTS: [u16; 10] = [26950, 26951, 26952, 26953, 26954, 26955, 26956, 26957, 26958, 26959];

//const FENDERMINT_METRICS_HOST_PORTS: [u16; 3] = [9184, 9185, 9186];
//const IROH_METRICS_HOST_PORTS: [u16; 3] = [9091, 9092, 9093];
//const PROMTAIL_AGENT_HOST_PORTS: [u16; 3] = [9080, 9081, 9082];
//const PROMETHEUS_HOST_PORT: u16 = 9090;
//const LOKI_HOST_PORT: u16 = 3100;
//const GRAFANA_HOST_PORT: u16 = 3000;
//const ANVIL_HOST_PORT: u16 = 8545;

const ANVIL_PRIVATE_KEYS: [&str; 10] = [
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
const ANVIL_PUBLIC_KEYS: [&str; 10] = [
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

const PARENT_ENDPOINT: &str = "http://localhost:8545";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The level of logging
    #[arg(short, long, default_value_t = LogLevel::Info, value_enum)]
    log: LogLevel,

    #[command(subcommand)]
    network: Option<Commands>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum LogLevel {
    Debug = 4,
    Info = 3,
    Quiet = 2,
    Silent = 1,
}

#[derive(Subcommand)]
enum Commands {
    Local {
        #[arg(short, long, default_value_t = 1)]
        nodes: u8,
    },
    Dev {},
}

fn main() {
    let args = Cli::parse();
    let log_level = args.log;

    match &args.network {
        Some(Commands::Dev {}) => {
            devnet(&log_level);
        }
        Some(Commands::Local { nodes }) => {
            localnet(&log_level, *nodes);
        }
        None => {
            panic!("no network given, network must be either `dev` or `local`");
        }
    }
}

fn devnet(log_level: &LogLevel) {
    // if do_setup {
    //     Command::new("make")
    //         .args("config-devnet")
    //         .outpu
    //         .expec//t("failed to setup devnet");
    // }
    let start_label = String::from("STARTUP").white().bold();

    let (iroh_stdout, iroh_stderr) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("IROH").magenta().bold(),
        cmd: "make",
        args: ["run-devnet-iroh"].to_vec(),
        envs: None,
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level,
    });

    // TODO: use channels to pass ready messages instead of sleep calls, maybe not worth the effort since waiting works?
    thread::sleep(time::Duration::from_secs(3));
    let (objects_stdout, objects_stderr) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("OBJECTS").green().bold(),
        cmd: "make",
        args: ["run-devnet-objects"].to_vec(),
        envs: None,
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level,
    });
    let fendermint_filters = vec![
        Regex::new(r"BlobsFinalityPendingBlobs\(0\)"),
        Regex::new(r"BlobsFinalityPendingBytes\(0\)"),
        Regex::new("size: 0, tx_count: 0"),
        Regex::new("BlockCommitted")
    ];
    let (fendermint_stdout, fendermint_stderr) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("FENDERMINT").bright_yellow().bold(),
        cmd: "make",
        args: ["run-devnet-fendermint"].to_vec(),
        envs: None,
        current_dir: None,
        out_filters: fendermint_filters.clone(),
        err_filters: fendermint_filters.clone(),
        log_level,
    });

    thread::sleep(time::Duration::from_secs(3));
    let (cometbft_stdout, cometbft_stderr) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("COMETBFT").cyan().bold(),
        cmd: "make",
        args: ["run-devnet-cometbft"].to_vec(),
        envs: None,
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
    let (evm_stdout, evm_stderr) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("EVM_RPC").bright_yellow().on_blue().bold(),
        cmd: "make",
        args: ["run-devnet-evm"].to_vec(),
        envs: None,
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level,
    });

    print_logo(&start_label, log_level);

    // Note: once we join iroh_stdout this will pause execution until iroh closes, i.e. until the network is shutdown
    iroh_stdout.join().unwrap();
    iroh_stderr.join().unwrap();
    objects_stdout.join().unwrap();
    objects_stderr.join().unwrap();
    fendermint_stdout.join().unwrap();
    fendermint_stderr.join().unwrap();
    cometbft_stdout.join().unwrap();
    cometbft_stderr.join().unwrap();
    evm_stdout.join().unwrap();
    evm_stderr.join().unwrap();
}

// TODO: this doesn't build or install anything. we are just starting the network here
fn localnet(log_level: &LogLevel, node_count: u8) {
    // note: since we are starting a pristine parent chain all contract addresses
    // are fixed for every deploy. Any changes to the order of setup interactions
    // will potentially change these values.
    // TODO: this is brittle and only works for localnet. when
    //      time allows we might parse command outputs to get values
    let contracts = ContractMap {
        account_helper: "0x5FbDB2315678afecb367f032d93F642f64180aa3",
        subnet_id_helper: "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512",
        cross_msg_helper: "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9",
        lib_staking: "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0",
        lib_quorum: "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9",
        gateway: "0x9A676e781A523b5d0C0e43731313A708CB607508",
        registry: "0x4ed7c70F96B99c776995fB64377f0d4aB3B0e1C1",
        supply_source_address: "0xa85233C63b9Ee964Add6F2cffe00Fd84eb32338f",
        gater: "0x851356ae760d987E095750cCeb3bC6014560891C"
    };

    let home = std::env::var("HOME").unwrap();
    let current_dir = std::env::current_dir().unwrap();
    let repo_root_dir = Path::new(current_dir.to_str().unwrap()); // "/Users/jwagner/Workspaces/textile/github/ipc"
    let ipc_config_dir = Path::new(&home).join(".ipc");

    let rust_log = get_rust_log_level(log_level);
    let start_label = String::from("STARTUP").white().bold();
    let print = |message: &str, message_level: &LogLevel| {
        log_level_print(&start_label, message, log_level, message_level, &vec![]);
    };
    print("", &LogLevel::Quiet);
    print(
        "--------------------------------------------------",
        &LogLevel::Quiet,
    );
    print("               hoku is starting", &LogLevel::Quiet);
    print(
        "--------------------------------------------------",
        &LogLevel::Quiet,
    );
    print("", &LogLevel::Quiet);
    let three_seconds = time::Duration::from_secs(3);
    thread::sleep(three_seconds);

    print("starting with clean .ipc dir", &LogLevel::Debug);

    Command::new("rm")
        .args(["-rf", &format!("{home}/.ipc")])
        .output()
        .expect("failed to rm .ipc");
    Command::new("mkdir")
        .args([&format!("{home}/.ipc")])
        .output()
        .expect("failed to mkdir .ipc");

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

    let anvil_msg_filters = vec![
        Regex::new("eth_blockNumber"),
        Regex::new("eth_getTransactionReceipt"),
    ];
    let (anvil_stdout, anvil_stderr) = pipe_sub_command(PipeSubCommandArgs {
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

    print("anvil started", &LogLevel::Quiet);

    // put validate private keys where the create_node func can find them
    for i in 0..node_count {
        let filepath = ipc_config_dir.join(format!("validator_{i}.sk"));
        let private_key = ANVIL_PRIVATE_KEYS[i as usize];

        write(filepath, private_key).unwrap();
    }
    print(
        "finished setting up validator private keys",
        &LogLevel::Info,
    );

    Command::new("rm")
        .args(["-rf", repo_root_dir.join("contracts").join("deployments").join("localnet").to_str().unwrap()])
        .output()
        .expect("failed to rm old localnet deployments");

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

    reg_out.join().unwrap();
    reg_err.join().unwrap();

    // TODO: sometimes this fails because we need to run `forge build`
    let (supply_out, supply_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("DEPLOY SUPPLY SOURCE").purple().bold(),
        cmd: "forge",
        envs: Some([
            ["RUST_LOG", rust_log].to_vec(),
            ["RPC_URL", PARENT_ENDPOINT].to_vec(),
            ["PRIVATE_KEY", &format!("0x{}", ANVIL_PRIVATE_KEYS[0])].to_vec(),
            ["CHAIN_ID", "31337"].to_vec(),
        ].to_vec()),
        current_dir: repo_root_dir.join("hoku-contracts").to_str(),
        args: [
            "script",
            "--private-key",
            ANVIL_PRIVATE_KEYS[0],
            "--rpc-url",
            PARENT_ENDPOINT,
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
    // fund each anvil wallet with supply token
    for i in 0..(ANVIL_PUBLIC_KEYS.len() - 1) {
        let public_key = ANVIL_PUBLIC_KEYS[i];
        print(
            &format!("funding address {} with supply token in parent", public_key),
            &LogLevel::Info,
        );
        let wallet_out = fund_wallet_token(
            public_key,
            ANVIL_PRIVATE_KEYS[0],
            "10100000000000000000000",
            contracts.supply_source_address,
        );
        print(&format!("token funding result {:?}", wallet_out), &LogLevel::Debug);
    }

    setup_config(repo_root_dir, ipc_config_dir.as_path());

    print("setup config files", &LogLevel::Info);
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
            &contracts.supply_source_address,
            "--validator-gater",
            &contracts.gater,
            "--collateral-source-kind",
            "erc20",
            "--collateral-source-address",
            &contracts.supply_source_address
        ].to_vec(),
        log_level,
        out_filters: vec![],
        err_filters: vec![]
    });

    subnet_out.join().unwrap();
    subnet_err.join().unwrap();

    deploy_gater(repo_root_dir, contracts.gater, node_count, log_level);

    let mut node_outs = vec!();
    print(&format!("starting {:?} validatore nodes", node_count), &LogLevel::Info);
    for i in 0..node_count {
        let node_config = NodeConfig {
            log_level,
            node_name: &format!("validator-{i}"),
            repo_root_dir,
            ipc_config_dir: &ipc_config_dir,
            contracts: &contracts,
            node_number: i,
            is_bootstrap: matches!(i, 0),
        };
        let out = create_node(node_config);
        node_outs.push(out);
        // give the node time to finish setting up
        thread::sleep(three_seconds);

        join_subnet(contracts.supply_source_address, i, log_level)
    }

    // give the network time to finish setting up
    print("wait 30 seconds", &LogLevel::Info);
    let thirty_seconds = time::Duration::from_secs(30);
    thread::sleep(thirty_seconds);
    print("", &LogLevel::Info);

    start_relayer(&ipc_config_dir, log_level);

    buy_credits(log_level);

    print_logo(&start_label, log_level);

    // Note: Rust `Command` starts subprocesses that live only as long as the parent
    //  process.  However calling join on a subprocess will keep the parent from ending
    //  by waiting for the subprocess to finish.  We join anvil here simply because
    //  it is convenient and it will never finish.
    anvil_stdout.join().unwrap();
    anvil_stderr.join().unwrap();
    println!("network died");
}

fn deploy_gater(repo_root_dir: &Path, gater_contract: &str, node_count: u8, log_level: &LogLevel) {
    let (gater_out, gater_err) = pipe_sub_command(PipeSubCommandArgs {
        title: &String::from("DEPLOY VALIDATOR GATER").bright_green().bold(),
        cmd: "forge",
        args: [
            "script",
            "script/ValidatorGater.s.sol",
            "--private-key",
            ANVIL_PRIVATE_KEYS[0],
            "--rpc-url",
            PARENT_ENDPOINT,
            "--tc",
            "DeployScript",
            "--sig",
            "run(string)",
            "local",
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

    // Approve each validator to stake
    let gater_label = String::from("APPROVE VALIDATOR GATE").bright_green().bold();
    for i in 0..node_count {
        let out = approve_validator_gate(
            // public key of validator being approved
            ANVIL_PUBLIC_KEYS[i as usize],
            // validator-0 is owner, only they can approve
            ANVIL_PRIVATE_KEYS[0],
            gater_contract
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
            PARENT_ENDPOINT,
            "--timeout",
            "120",
            gater_contract,
            "setSubnet((uint64,address[]))",
            &format!("(31337, [{}])", SUBNET_ETH_ADDRESS)
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

fn join_subnet(supply_source_address: &str, node_number: u8, log_level: &LogLevel) {
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
            PARENT_ENDPOINT,
            "--timeout",
            "120",
            supply_source_address,
            "approve(address,uint256)",
            SUBNET_ETH_ADDRESS,
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
            SUBNET_ID,
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

struct NodeHandles {
    iroh: (JoinHandle<()>, JoinHandle<()>),
    cometbft: (JoinHandle<()>, JoinHandle<()>),
    fendermint: (JoinHandle<()>, JoinHandle<()>),
    evm: (JoinHandle<()>, JoinHandle<()>),
    objects: (JoinHandle<()>, JoinHandle<()>),
}
fn create_node(config: NodeConfig) -> NodeHandles {
    let NodeConfig {
        log_level,
        node_name,
        repo_root_dir,
        ipc_config_dir,
        contracts,
        node_number,
        is_bootstrap,
    } = config;

    let rust_log = get_rust_log_level(log_level);
    let start_label = String::from("STARTUP").white().bold();
    let print = |message: &str, message_level: &LogLevel| {
        log_level_print(&start_label, message, log_level, message_level, &vec![]);
    };

    let node_root_path = ipc_config_dir.join(NETWORK_NAME).join(node_name);

    print(
        &format!("init node dir {:?}", node_root_path.to_str().unwrap()),
        &LogLevel::Debug,
    );
    init_node_dir(&node_root_path, &repo_root_dir, &node_number);

    let node_path = node_root_path.join(node_name);
    let cmt_dir = node_path.join("cometbft");
    // let env_file = node_root_path.join(".env");

    // TODO: this isn't needed anymore
//    let cmt_config_file = if is_bootstrap {
//        "bootstrap_config.toml"
//    } else {
//        "config.toml"
//    };
    let fm_dir = node_path.join("fendermint");
    let iroh_config_dir = node_path.join("iroh").join("config");

    setup_iroh_config(&iroh_config_dir, node_number);

    //TODO: iroh metrics enabled by adding `metrics_addr = "0.0.0.0:{what ever IROH_METRICS_PORT is for this node}"` to config
    // "iroh-start",
    let iroh_port = IROH_RPC_PORTS[node_number as usize];
    let iroh_rpc_addr = format!("127.0.0.1:{iroh_port}");
    // TODO: iroh recommends starting nodes using the rust SDK. It might be a lot of work, but look into doing this here
    let iroh_out = pipe_sub_command(PipeSubCommandArgs {
        title: &format!("IROH {:?}", node_number).magenta().bold(),
        cmd: "iroh",
        args: [
            "--rpc-addr",
            &iroh_rpc_addr,
            "start",
        ]
        .to_vec(),
        envs: Some([
            // TODO: not sure how iroh cli configures logging
            ["RUST_LOG", rust_log].to_vec(),
            [
                "IROH_DATA_DIR",
                node_path.join("iroh").join("data").to_str().unwrap(),
            ].to_vec(),
            [
                "IROH_CONFIG_DIR",
                node_path.join("iroh").join("config").to_str().unwrap(),
            ].to_vec()
        ].to_vec()),
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level,
    });

    // "iroh-wait"
    // TODO: not sure we need these 30 second waits.
    print("wait 30 seconds", &LogLevel::Info);
    let thirty_seconds = time::Duration::from_secs(30);
    thread::sleep(thirty_seconds);
    print("", &LogLevel::Info);

    let cmt_rpc_port = CMT_RPC_PORTS[node_number as usize];
    let cmt_rpc_url = format!("http://127.0.0.1:{:?}", cmt_rpc_port);
    let cmt_tcp_url = format!("tcp://127.0.0.1:{:?}", cmt_rpc_port);
    // create config and start this nodes cometbft
    let cometbft_out = start_cometbft(
        &cmt_dir,
        &repo_root_dir.join("hoku-dev-cli").join("config").join("cometbft").join("config.toml"),
        &cmt_tcp_url,
        node_number,
        log_level
    );

    // "cometbft-wait"
    print("wait 30 seconds", &LogLevel::Info);
    thread::sleep(thirty_seconds);
    print("", &LogLevel::Info);

    // TODO: do we need a bootstrap?
//    let bootstrap_peer_id = "16Uiu2HAmH5dzPoWL2wtQQGaPnwJZZn988xDAxbpsYu8nBhfyCrfH";

    // Bootstrap node started. Node id 20ee1f8f1c39602c5f7bf8e161c85a6eaf194c86, peer id 16Uiu2HAmH5dzPoWL2wtQQGaPnwJZZn988xDAxbpsYu8nBhfyCrfH
    // Bootstrap node endpoint: 20ee1f8f1c39602c5f7bf8e161c85a6eaf194c86@validator-0-cometbft:26656
    // Bootstrap resolver endpoint: /dns/validator-0-fendermint/tcp/26655/p2p/16Uiu2HAmH5dzPoWL2wtQQGaPnwJZZn988xDAxbpsYu8nBhfyCrfH
    print(
        &format!("node path: {}", node_path.to_str().unwrap()),
        &LogLevel::Debug,
    );

    let fendermint_out = start_fendermint(&fm_dir, node_number, &format!("http://{}", iroh_rpc_addr), &cmt_rpc_url, log_level);

    // TODO: I don't know why we have to wait, maybe remove this and see?
    print("wait 30 seconds", &LogLevel::Debug);
    thread::sleep(thirty_seconds);
    print("", &LogLevel::Debug);

    // "start the etherium json-rpc facade",
    let evm_out = pipe_sub_command(PipeSubCommandArgs {
        title: &format!("EVM_RPC {:?}", node_number).blue().on_yellow().bold(),
        cmd: "./target/debug/fendermint",
        envs: Some([
            ["RUST_LOG", rust_log].to_vec(),
        ].to_vec()),
        args: [
            "--home-dir",
            fm_dir.to_str().unwrap(),
            "eth",
            "run",
            // The URL of the Tendermint node's RPC endpoint
            "--http-url",
            &cmt_rpc_url,
            // how to set the listen port?
        ].to_vec(),
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level,
    });

    thread::sleep(thirty_seconds);

    // "objects-start",
    let objects_out = pipe_sub_command(PipeSubCommandArgs {
        title: &format!("OBJECTS {:?}", node_number).green().bold(),
        cmd: "./target/debug/fendermint",
        args: [
            "--home-dir",
            fm_dir.to_str().unwrap(),
            "objects",
            "run"
            // TODO: need to pass in port number for this objects node
        ].to_vec(),
        envs: Some([
            ["RUST_LOG", rust_log].to_vec(),
            ["FM_NETWORK", "test"].to_vec()
        ].to_vec()),
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level,
    });

    thread::sleep(thirty_seconds);

    print("", &LogLevel::Quiet);
    print(
        &format!("created subnet node {}", node_name),
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

fn start_cometbft(cmt_dir: &Path, default_config_path: &Path, cmt_tcp_url: &str, node_number: u8, log_level: &LogLevel) -> (JoinHandle<()>, JoinHandle<()>) {
    // "cometbft-init",
    let cmt_home = String::from(cmt_dir.to_str().unwrap());
    let rust_log = get_rust_log_level(log_level);

    // TODO: we need to use the `cometbft testnet` command to start a network, maybe?
    let (cometbft_stdout, cometbft_stderr) = pipe_sub_command(PipeSubCommandArgs {
        title: &format!("COMETBFT INIT {:?}", node_number).bright_green().bold(),
        cmd: "cometbft",
        envs: Some([
            ["RUST_LOG", rust_log].to_vec(),
            ["CMT_PROXY_APP", "kvstore"].to_vec(),
            ["CMT_RPC_MAX_SUBSCRIPTION_CLIENTS", "10"].to_vec(),
            ["CMT_RPC_MAX_SUBSCRIPTIONS_PER_CLIENT", "1000"].to_vec()
        ].to_vec()),
        args: [
            "init",
            "--home",
            &cmt_home
        ].to_vec(),
        out_filters: vec!(),
        err_filters: vec!(),
        current_dir: None,
        log_level
    });

    cometbft_stdout.join().unwrap();
    cometbft_stderr.join().unwrap();

    // Note: the config that results from the `cometbft init` command above is overwritten by this function
    setup_cmt_config(cmt_dir, cmt_tcp_url, node_number, default_config_path, log_level);

    // "cometbft-start",
    pipe_sub_command(PipeSubCommandArgs {
        title: &format!("COMETBFT {:?}", node_number).cyan().bold(),
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
    })
}

fn start_fendermint(fm_dir: &Path, node_number: u8, iroh_rpc_url: &str, cmt_rpc_url: &str, log_level: &LogLevel) -> (JoinHandle<()>, JoinHandle<()>) {
    let rust_log = get_rust_log_level(log_level);
    let validator_key_path = fm_dir
        .join("keys")
        .join("validator_key.sk");
    // "fendermint-start-validator",
    setup_fm_config(fm_dir, node_number);
    let fm_resolver_port = FM_RESOLVER_PORTS[node_number as usize];
    pipe_sub_command(PipeSubCommandArgs {
        title: &format!("FENDERMINT {:?}", node_number).yellow().bold(),
        cmd: "./target/debug/fendermint",
        args: [
            "--home-dir",
            fm_dir.to_str().unwrap(),
            "run",
            "--iroh-addr",
            iroh_rpc_url
        ].to_vec(),
        envs: Some(
            [
                ["RUST_LOG", rust_log].to_vec(),
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
                    &format!("/ip4/127.0.0.1/tcp/{fm_resolver_port}"),
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

fn setup_fm_config(fm_dir: &Path, node_number: u8) {
    // we use our default cometbft config.toml file, but we need to update to use the config for this network
    let fm_config_filepath = fm_dir.join("config").join("default.toml");
    let config_file = read_to_string(&fm_config_filepath).expect("could not modify cometbft config");
    let mut conf_doc = config_file.parse::<DocumentMut>().expect("invalid document");

    conf_doc["abci"]["listen"]["port"] = value(ABCI_PORTS[node_number as usize] as i64);
    conf_doc["eth"]["listen"]["port"] = value(ETHAPI_PORTS[node_number as usize] as i64);
    conf_doc["objects"]["listen"]["port"] = value(OBJECTS_PORTS[node_number as usize] as i64);

    // TODO: setup separte ports for each node's metrics
    conf_doc["metrics"]["enabled"] = value(false);
    conf_doc["eth"]["metrics"]["enabled"] = value(false);
    conf_doc["objects"]["metrics"]["enabled"] = value(false);

    // TODO: we need to get snapshots working... there's a ticket for this
    //conf_doc["snapshots"]["enabled"] = value(true);

    write(&fm_config_filepath, conf_doc.to_string()).expect("could not write to cometbft config file");
}

fn setup_cmt_config(cmt_dir: &Path, cmt_rpc_url: &str, node_number: u8, default_config_path: &Path, log_level: &LogLevel) {
    // we have a default cometbft config.toml file, and we need to update to use the config for this node
    let cmt_config_filepath = cmt_dir.join("config").join("config.toml");
    let cp_conf_out = Command::new("cp")
        .args([
            default_config_path.to_str().unwrap(),
            cmt_config_filepath.to_str().unwrap()
        ])
        .output()
        .expect("failed to copy cometbft config");
    log_level_print(&format!("COMETBFT CONF {:?}", node_number).yellow().on_blue().bold(), &format!("{:?}", cp_conf_out), log_level, &LogLevel::Debug, &vec!());

    let config_file = read_to_string(&cmt_config_filepath).expect("could not modify cometbft config");
    let mut conf_doc = config_file.parse::<DocumentMut>().expect("invalid document");

    conf_doc["moniker"] = value(format!("cometbft_node_{:?}", node_number));
    // TODO: does doing this make sense? cmtbft is really noisy with debug logging
    conf_doc["log_level"] = value(match log_level {
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Quiet => "error",
        LogLevel::Silent => "none"
    });
    // TCP or UNIX socket address for the RPC server to listen on
    conf_doc["rpc"]["laddr"] = value(cmt_rpc_url);

    conf_doc["p2p"]["laddr"] = value(format!("tcp://0.0.0.0:{:?}", CMT_P2P_PORTS[node_number as usize]));
    // If this is not the first cometbft node, add the first cometbft node as a seed node
    if node_number > 0 {
        conf_doc["p2p"]["seeds"] = value(format!("tcp://127.0.0.1:{:?}", CMT_RPC_PORTS[0]));
    }

    write(&cmt_config_filepath, conf_doc.to_string()).expect("could not write to cometbft config file");
}

fn setup_iroh_config(iroh_config_dir: &Path, node_number: u8) {
    // we have a default cometbft config.toml file, and we need to update to use the config for this node
    let iroh_config_filepath = iroh_config_dir.join("iroh.config.toml");

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

fn create_keystore(filepath: &Path) {
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

fn approve_validator_gate(
    public_key: &str,
    private_key: &str,
    validator_gater_address: &str
) -> Output {
    let out = Command::new("cast")
        .args([
            "send",
            "--private-key",
            private_key,
            "--rpc-url",
            PARENT_ENDPOINT,
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
    return out;
}

// Start relayer
// note: this command mutates the order of keys in the evm_keystore.json file. to
// keep the accounts consistent for usage (e.g., logging accounts, using
// validator keys, etc.), we temporarily copy the file and then restore it.
fn start_relayer(ipc_config_dir: &Path, log_level: &LogLevel) -> (JoinHandle<()>, JoinHandle<()>) {

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
            SUBNET_ID,
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
    sleep(time::Duration::from_secs(3));

    Command::new("cp").args([
        ipc_config_dir.join("copy.evm_keystore.json"),
        ipc_config_dir.join("evm_keystore.json")
    ]).output().unwrap();

    relayer_out
}

fn buy_credits(log_level: &LogLevel) {
    let token_amount = "10000000000000000000000";

    let rust_log = get_rust_log_level(log_level);
    for i in 0..ANVIL_PUBLIC_KEYS.len() {
        let addr = ANVIL_PUBLIC_KEYS[i];

        let (stdout, stderr) = pipe_sub_command(PipeSubCommandArgs {
            title: &format!("FUND WITH TOKEN {:?}", i).bright_purple().bold(),
            cmd: "./target/debug/ipc-cli",
            envs: Some([["RUST_LOG", rust_log].to_vec()].to_vec()),
            current_dir: None,
            args: [
                "cross-msg",
                "fund-with-token",
                "--subnet",
                SUBNET_ID,
                "--from",
                addr,
                "--to",
                addr,
                "--approve",
                token_amount
            ].to_vec(),
            out_filters: vec![],
            err_filters: vec![],
            log_level
        });

        stdout.join().unwrap();
        stderr.join().unwrap();
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
                &format!("http://localhost:{:?}", ETHAPI_PORTS[0]),
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
        thread::sleep(time::Duration::from_secs(2));
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

fn fund_wallet_token(
    public_key: &str,
    private_key: &str,
    token_amount: &str,
    supply_source_address: &str,
) -> Output {
    //cast send --private-key "$pk" --rpc-url "$rpc_url" --timeout 120 "$SUPPLY_SOURCE_ADDRESS" "mint(address,uint256)" "$addr" "$token_amount"
    Command::new("cast")
        .args([
            "send",
            "--private-key",
            private_key,
            "--rpc-url",
            PARENT_ENDPOINT,
            supply_source_address,
            "mint(address,uint256)",
            public_key,
            token_amount,
        ])
        .output()
        .expect("failed to fund wallet with source token")
}

// TODO: this should take the log_level. if debug, print the paths all these commands use
fn init_node_dir(dir: &Path, repo_root_dir: &Path, node_number: &u8) {
    Command::new("rm")
        .args(["-rf", dir.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("failed to remove dir {:?}: Error: {:?}", dir, err));
    Command::new("mkdir")
        .args(["-p", dir.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("failed to create {:?}: Error: {:?}", dir, err));

    Command::new("touch")
        .args([dir.join(".env").to_str().unwrap()])
        .output()
        .expect("failed to create .env file");

    let validator_dir = dir.join(format!("validator-{:?}", node_number));
    Command::new("mkdir")
        .args([validator_dir.to_str().unwrap()])
        .output()
        .expect("failed to make validator dir");

    Command::new("mkdir")
        .args(["-p", validator_dir.join("cometbft").to_str().unwrap()])
        .output()
        .expect("failed to create cometbft dir");

    Command::new("mkdir")
        .args(["-p", validator_dir.join("fendermint").to_str().unwrap()])
        .output()
        .expect("failed to create fendermint dir");
    Command::new("mkdir")
        .args([
            "-p",
            validator_dir
                .join("fendermint")
                .join("data")
                .to_str()
                .unwrap(),
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
                .unwrap(),
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
                .unwrap(),
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
                .unwrap(),
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
            validator_dir.join("iroh").join("blobs").to_str().unwrap(),
        ])
        .output()
        .expect("failed to create iroh blobs dir");

    let iroh_config_dir = validator_dir
        .join("iroh")
        .join("config");
    Command::new("mkdir")
        .args([
            "-p",
            iroh_config_dir.to_str().unwrap(),
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
                .unwrap(),
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
            default_keys_dir.join(format!("validator_key_{:?}.sk", &node_number)).to_str().unwrap(),
            fm_keys_dir
                .join("validator_key.sk")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("failed to copy key file");
    Command::new("cp")
        .args([
            default_keys_dir.join(format!("network_key_{:?}.sk", &node_number)).to_str().unwrap(),
            fm_keys_dir
                .join("network.sk")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("failed to copy key file");

}

// copy subnet config files
fn setup_config(repo_root_dir: &Path, config_folder: &Path) {
    // put ipc config in ~/.ipc/
    Command::new("cp")
        .args([
            repo_root_dir.join("hoku-dev-cli").join("config").join("config.toml").to_str().unwrap(),
            config_folder.to_str().unwrap(),
        ])
        .output()
        .expect("failed to copy ipc config");
    // put relayer config in ~/.ipc/
    Command::new("cp")
        .args([
            repo_root_dir.join("hoku-dev-cli").join("config").join("relayer.config.toml").to_str().unwrap(),
            config_folder.to_str().unwrap(),
        ])
        .output()
        .expect("failed to copy relayer config");

    create_keystore(&config_folder.join("evm_keystore.json"));
}
