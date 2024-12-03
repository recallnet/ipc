// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
mod util;
mod cometbft;
mod iroh;
mod fendermint;

use toml_edit::{DocumentMut, value};
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use regex::Regex;
use serde::Serialize;
use std::fs::{write, File, read_to_string};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::thread::sleep;
use std::thread::JoinHandle;
use std::time;
use util::pipe_sub_command;
use crate::cometbft::{init_cometbft, start_cometbft, get_cmt_rpc};
use crate::iroh::{setup_iroh_config, start_iroh};
use crate::fendermint::{start_fendermint, init_fendermint};
use crate::util::{init_node_dir, log_level_print, print_logo, get_rust_log_level, THIRTY_SECONDS, PipeSubCommandArgs};

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
        gater: "0xf5059a5D33d5853360D16C683c16e67980206f36"
    };

    let ipc_config_dir = get_ipc_dir();
    let current_dir = std::env::current_dir().unwrap();
    let repo_root_dir = Path::new(current_dir.to_str().unwrap()); // "/Users/jwagner/Workspaces/textile/github/ipc"

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

    init_node_dir(&ipc_config_dir, repo_root_dir, node_count);
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
    for i in 0..(ANVIL_PUBLIC_KEYS.len()) {
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

    init_cometbft(node_count, &ipc_config_dir.join(NETWORK_NAME), log_level);

    let mut node_outs = vec!();
    print(&format!("starting {:?} validatore nodes", node_count), &LogLevel::Info);
    for i in 0..node_count {
        let node_config = NodeConfig {
            log_level,
            node_name: &format!("validator-{i}"),
            repo_root_dir,
            ipc_config_dir: &ipc_config_dir,
            contracts: &contracts,
            node_number: i
        };
        let out = create_node(node_config);
        node_outs.push(out);
        // give the node time to finish setting up
        thread::sleep(three_seconds);

        join_subnet(contracts.supply_source_address, i, log_level)
    }

    // give the network time to finish setting up
    print("wait 30 seconds", &LogLevel::Info);
    thread::sleep(THIRTY_SECONDS);
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
//    TODO: cannot index into &NodeHandles
//    for out in node_outs.iter() {
//        out[0].join().unwrap();
//        out[1].join().unwrap();
//    }

    println!("network died");
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
    } = config;

    let rust_log = get_rust_log_level(log_level);
    let start_label = String::from("STARTUP").white().bold();
    let print = |message: &str, message_level: &LogLevel| {
        log_level_print(&start_label, message, log_level, message_level, &vec![]);
    };

    let node_root_path = ipc_config_dir.join(NETWORK_NAME).join(node_name);

    let node_path = node_root_path.join(node_name);
    print(
        &format!("node path: {}", node_path.to_str().unwrap()),
        &LogLevel::Debug,
    );

    let iroh_dir = node_path.join("iroh");
    setup_iroh_config(&iroh_dir);
    let iroh_port = IROH_RPC_PORTS[node_number as usize];
    let iroh_rpc_addr = format!("127.0.0.1:{iroh_port}");
    let iroh_out = start_iroh(&iroh_dir, &iroh_rpc_addr, &format!("IROH {:?}", node_number).magenta().bold(), log_level);

    // create config and start this nodes cometbft
    let cmt_dir = node_path.join("cometbft");
    let cmt_rpc_url = get_cmt_rpc(node_number);
    let cometbft_out = start_cometbft(
        &cmt_dir,
        &format!("COMETBFT {:?}", node_number).cyan().bold(),
        log_level
    );

    // TODO: do we need a bootstrap?
//    let bootstrap_peer_id = "16Uiu2HAmH5dzPoWL2wtQQGaPnwJZZn988xDAxbpsYu8nBhfyCrfH";

    // Bootstrap node started. Node id 20ee1f8f1c39602c5f7bf8e161c85a6eaf194c86, peer id 16Uiu2HAmH5dzPoWL2wtQQGaPnwJZZn988xDAxbpsYu8nBhfyCrfH
    // Bootstrap node endpoint: 20ee1f8f1c39602c5f7bf8e161c85a6eaf194c86@validator-0-cometbft:26656
    // Bootstrap resolver endpoint: /dns/validator-0-fendermint/tcp/26655/p2p/16Uiu2HAmH5dzPoWL2wtQQGaPnwJZZn988xDAxbpsYu8nBhfyCrfH

    let fm_dir = node_path.join("fendermint");
    let label = format!("FENDERMINT {:?}", node_number).yellow().bold();
    let fm_resolver_port = format!("{:?}", FM_RESOLVER_PORTS[node_number as usize]);
    init_fendermint(&fm_dir, node_number);
    let fendermint_out = start_fendermint(
        &fm_dir,
        &label,
        &format!("http://{}", &iroh_rpc_addr),
    &format!("http://{}", cmt_rpc_url),
        &fm_resolver_port,
        log_level
    );

    // TODO: I don't know why we have to wait, maybe remove this and see?
    print("wait 30 seconds", &LogLevel::Debug);
    thread::sleep(THIRTY_SECONDS);
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
            &format!("http://127.0.0.1:{:?}", CMT_RPC_PORTS[node_number as usize]),
            // how to set the listen port?
        ].to_vec(),
        current_dir: None,
        out_filters: vec![],
        err_filters: vec![],
        log_level,
    });

    thread::sleep(THIRTY_SECONDS);

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

    thread::sleep(THIRTY_SECONDS);

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

fn get_ipc_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap();
    Path::new(&home).join(".ipc")
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
    let three_seconds = time::Duration::from_secs(3);
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

        // let the transactions process
        thread::sleep(three_seconds);
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
