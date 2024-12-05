// Copyright 2022-2024 Textile, Inc.
// SPDX-License-Identifier: Apache-2.0, MIT
mod anvil;
mod contracts;
mod cometbft;
mod ethapi;
mod fendermint;
mod iroh;
mod node;
mod objects;
mod relayer;
mod util;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use regex::Regex;
use std::path::Path;

use anvil::{
    start_anvil,
    setup_anvil_keys,
    ANVIL_PRIVATE_KEYS,
    ANVIL_PUBLIC_KEYS
};
use cometbft::init_cometbft;
use contracts::{
    build_contracts,
    buy_credits,
    create_subnet,
    deploy_libraries,
    deploy_gater,
    deploy_gateway,
    deploy_registry,
    deploy_supply_source,
    fund_wallet_token,
    join_subnet,
    setup_gater,
    ContractMap
};
use node::{create_node, NodeConfig, PortMap};
use relayer::start_relayer;
use util::{
    get_ipc_dir,
    init_node_dir,
    log_level_print,
    print_logo,
    pipe_sub_command,
    setup_subnet_config,
    sleep_thirty,
    sleep_three,
    PipeSubCommandArgs
};

const ANVIL_CHAIN_ID: &str = "31337";
// note: the subnet id always has the same value so putting it here
// TODO: we can probably get this from the subnet create command so we can use this on pre-existing nets
const SUBNET_ADDRESS: &str = "t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly";
// Note: to get subnet eth addr do `./target/debug/ipc-cli util f4-to-eth-addr --addr "t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly"`
const SUBNET_ETH_ADDRESS: &str = "0x56639db16ac50a89228026e42a316b30179a5376";

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
        #[arg(long, default_value_t = 0)]
        start_node: u8
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
        Some(Commands::Local { nodes , start_node}) => {
            let mut node_numbers = vec!();
            for i in *start_node..(*nodes + *start_node) {
                node_numbers.push(i);
                if i > 9 {
                    panic!(
                        "there are only 10 anvil accounts available for validators, you are trying to use account {:?}",
                        nodes + start_node + 1
                    );
                }
            }
            localnet(&log_level, node_numbers);
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
    sleep_three(log_level);
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

    sleep_three(log_level);
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

// TODO: this doesn't build or install anything. we are just starting the network here. we should have the ability to build somehow
fn localnet(log_level: &LogLevel, nodes: Vec<u8>) {
    let subnet_id: &str = &format!("/r{ANVIL_CHAIN_ID}/{SUBNET_ADDRESS}");
    let network_name: &str = &format!("r{ANVIL_CHAIN_ID}-{SUBNET_ADDRESS}");

    let node_count = nodes.len();

    let ipc_config_dir = get_ipc_dir();
    let current_dir = std::env::current_dir().unwrap();
    let repo_root_dir = Path::new(current_dir.to_str().unwrap()); // "/Users/jwagner/Workspaces/textile/github/ipc"

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

    sleep_three(log_level);

    print("starting with clean .ipc dir", &LogLevel::Debug);

    init_node_dir(&ipc_config_dir, &ipc_config_dir.join(network_name), repo_root_dir, nodes.clone());

    build_contracts(repo_root_dir, log_level);

    let (anvil_stdout, anvil_stderr) = start_anvil(log_level);
    setup_anvil_keys(nodes.clone(), &ipc_config_dir, log_level);

    let mut contracts = deploy_libraries(repo_root_dir, log_level);

    let gateway_address = deploy_gateway(&contracts, repo_root_dir, log_level);
    contracts.gateway = gateway_address;

    let registry_address = deploy_registry(&contracts, repo_root_dir, log_level);
    contracts.registry = registry_address;

    let supply_source_address = deploy_supply_source(PARENT_ENDPOINT, repo_root_dir, log_level);
    contracts.supply_source_address = supply_source_address;

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
            &contracts.supply_source_address,
            PARENT_ENDPOINT
        );
        print(&format!("token funding result {:?}", wallet_out), &LogLevel::Debug);
    }

    setup_subnet_config(
        repo_root_dir,
        ipc_config_dir.as_path(),
        &contracts,
        PARENT_ENDPOINT,
        &format!("http://127.0.0.1:{:?}", ETHAPI_PORTS[nodes[0] as usize]),
        ANVIL_CHAIN_ID,
        subnet_id,
        log_level
    );
    let gater_address = deploy_gater(PARENT_ENDPOINT, repo_root_dir, log_level);
    contracts.gater = gater_address;
    println!("gater addresssss:   {:?}", &contracts.gater);
    create_subnet(&contracts.supply_source_address, &contracts.gater, log_level);

    setup_gater(
        PARENT_ENDPOINT,
        &contracts.gater,
        SUBNET_ETH_ADDRESS,
        nodes.clone(),
        repo_root_dir,
        log_level
    );

    // initialize cometbft config before starting nodes
    init_cometbft(nodes.clone(), &ipc_config_dir.join(network_name), log_level);

    let mut node_outs = vec!();
    print(&format!("starting {:?} validatore nodes", node_count), &LogLevel::Info);
    for i in nodes.into_iter() {
        let port_index = i as usize;
        let node_config = NodeConfig {
            log_level,
            ipc_config_dir: &ipc_config_dir,
            contracts: &contracts,
            node_number: i,
            node_path: &ipc_config_dir.join(network_name).join(format!("validator-{:?}", i)).join(format!("validator-{:?}", i)),
            ports: &PortMap {
                abci: ABCI_PORTS[port_index],
                cometbft_p2p: CMT_P2P_PORTS[port_index],
                cometbft_rpc: CMT_RPC_PORTS[port_index],
                ethapi: ETHAPI_PORTS[port_index],
                fm_resolver: FM_RESOLVER_PORTS[port_index],
                iroh: IROH_RPC_PORTS[port_index],
                objects: OBJECTS_PORTS[port_index]
            }
        };
        let out = create_node(node_config);
        node_outs.push(out);
        // give the node time to finish setting up
        sleep_three(log_level);

        join_subnet(PARENT_ENDPOINT, SUBNET_ETH_ADDRESS, subnet_id, &contracts.supply_source_address, i, log_level);
    }

    // give the network time to finish setting up
    sleep_thirty(log_level);

    start_relayer(&ipc_config_dir, subnet_id, log_level);
    buy_credits(&format!("http://localhost:{:?}", ETHAPI_PORTS[0]), subnet_id, log_level);

    print_logo(&start_label, log_level);

    // Note: Rust `Command` starts subprocesses that live only as long as the parent
    //  process.  This means `ctrl+c` will kill everything, but we also need to keep
    //  the subprocesses running after the parent is finished starting the network.
    //  The join calls below ensure the parent does not exit when this function finishes.
    anvil_stdout.join().unwrap();
    anvil_stderr.join().unwrap();

    for out in node_outs.into_iter() {
        out.iroh.0.join().unwrap();
        out.iroh.1.join().unwrap();
        out.cometbft.0.join().unwrap();
        out.cometbft.1.join().unwrap();
        out.fendermint.0.join().unwrap();
        out.fendermint.1.join().unwrap();
        out.evm.0.join().unwrap();
        out.evm.1.join().unwrap();
        out.objects.0.join().unwrap();
        out.objects.1.join().unwrap();
    }

    println!("fatal error, network died");
}
