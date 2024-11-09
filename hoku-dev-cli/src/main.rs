mod util;

use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;
use std::fs::File;
use std::fs::write;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::thread::JoinHandle;
use std::time;
use serde::Serialize;
use colored::Colorize;
use util::pipe_sub_command;

#[derive(Serialize)]
struct Keystore {
    address: String,
    private_key: String
}

struct ContractMap {
    account_helper: String,
    subnet_id_helper: String,
    cross_msg_helper: String,
    lib_staking: String,
    lib_quorum: String,
    gateway: String,
    registry: String,
    supply_source_address: String
}

struct NodeConfig<'a> {
    node_name: &'a str,
    repo_root_dir: &'a Path,
    ipc_config_folder: &'a str,
    contracts: ContractMap,
    node_number: u8,
    is_bootstrap: bool,
    log_level: &'a LogLevel,
}

// note: the subnet id always has the same value, we need it
//   before it's created for use int the docker network name
const SUBNET_ID: &str = "/r31337/t410f6dl55afbyjbpupdtrmedyqrnmxdmpk7rxuduafq";
const NETWORK_NAME: &str = "r31337-t410f6dl55afbyjbpupdtrmedyqrnmxdmpk7rxuduafq";

// TODO: this will be a command arg, with max value of 10
const VALIDATOR_COUNT: u8 = 4;

// TODO: all these values assume that there is 3 nodes, we want to take a count as a command flag param
const CMT_P2P_HOST_PORTS: [u16; 3] = [26656, 26756, 26856];
const CMT_RPC_HOST_PORTS: [u16; 3] = [26657, 26757, 26857];
const ETHAPI_HOST_PORTS: [u16; 3] = [8645, 8745, 8845];
const RESOLVER_HOST_PORTS: [u16; 3] = [26655, 26755, 26855];
const OBJECTS_HOST_PORTS: [u16; 3] = [8001, 8002, 8003];
const IROH_RPC_HOST_PORTS: [u16; 3] = [4921, 4922, 4923];
const FENDERMINT_METRICS_HOST_PORTS: [u16; 3] = [9184, 9185, 9186];
const IROH_METRICS_HOST_PORTS: [u16; 3] = [9091, 9092, 9093];
const PROMTAIL_AGENT_HOST_PORTS: [u16; 3] = [9080, 9081, 9082];
const PROMETHEUS_HOST_PORT: u16 = 9090;
const LOKI_HOST_PORT: u16 = 3100;
const GRAFANA_HOST_PORT: u16 = 3000;
const ANVIL_HOST_PORT: u16 = 8545;
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
    "2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6"
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
    "0xa0ee7a142d267c1f36714e4a8f75612f20a79720"
];

const PARENT_ENDPOINT: &str = "http://localhost:8545";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The level of logging
    #[arg(default_value_t = LogLevel::Info, value_enum)]
    log: LogLevel,

    #[command(subcommand)]
    network: Option<Commands>
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum LogLevel {
    Info,
    Debug,
    Silent
}

#[derive(Subcommand)]
enum Commands {
    Local {
        #[arg(short, long)]
        nodes: Option<u8>
    },
    Dev {

    }
}

fn main() {
    let args = Cli::parse();
    let log_level = args.log;

    match &args.network {
        Some(Commands::Dev {}) => {
            devnet(&log_level);
        }
        Some(Commands::Local { nodes }) => {
            localnet(&log_level);
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

    let (iroh_stdout, iroh_stderr) = pipe_sub_command(
        String::from("IROH").magenta().bold(),
        "make",
        ["run-devnet-iroh"].to_vec(),
        None,
        None,
        vec![],
        vec![],
        log_level
    );

    // TODO: use channels to pass ready messages instead of sleep calls, maybe not worth the effort since waiting works?
    thread::sleep(time::Duration::from_secs(3));
    let (objects_stdout, objects_stderr) = pipe_sub_command(
        String::from("OBJECTS").green().bold(),
        "make",
        ["run-devnet-objects"].to_vec(),
        None,
        None,
        vec![],
        vec![],
        log_level
    );
    let (fendermint_stdout, fendermint_stderr) = pipe_sub_command(
        String::from("FENDERMINT").bright_yellow().bold(),
        "make",
        ["run-devnet-fendermint"].to_vec(),
        None,
        None,
        vec![],
        vec![],
        log_level
    );

    thread::sleep(time::Duration::from_secs(3));
    let (cometbft_stdout, cometbft_stderr) = pipe_sub_command(
        String::from("COMETBFT").cyan().bold(),
        "make",
        ["run-devnet-cometbft"].to_vec(),
        None,
        None,
        vec![],
        vec![],
        log_level
    );
    let (evm_stdout, evm_stderr) = pipe_sub_command(
        String::from("EVM_RPC").red().bold(),
        "make",
        ["run-devnet-evm"].to_vec(),
        None,
        None,
        vec![],
        vec![],
        log_level
    );

    iroh_stdout.join().unwrap();
    iroh_stderr.join().unwrap();
    thread::sleep(time::Duration::from_millis(500));
    objects_stdout.join().unwrap();
    objects_stderr.join().unwrap();
    thread::sleep(time::Duration::from_millis(500));
    fendermint_stdout.join().unwrap();
    fendermint_stderr.join().unwrap();
    thread::sleep(time::Duration::from_millis(500));
    cometbft_stdout.join().unwrap();
    cometbft_stderr.join().unwrap();
    thread::sleep(time::Duration::from_millis(500));
    evm_stdout.join().unwrap();
    evm_stderr.join().unwrap();
}

// TODO: this doesn't build or install anything. we are just starting the network here
fn localnet(log_level: &LogLevel) {
    // TODO: show startup message, and convert plain printlns to use [STARTUP] tag

    let filtered_list = vec![
        Regex::new("eth_blockNumber"),
        Regex::new("eth_getTransactionReceipt")
    ];
    // note: since we are starting a pristine parent chain all contract addresses
    // are fixed for every deploy so no need to parse command outputs to get values
    let contracts = ContractMap {
        account_helper: "0x5FbDB2315678afecb367f032d93F642f64180aa3".to_string(),
        subnet_id_helper: "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0".to_string(),
        cross_msg_helper: "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9".to_string(),
        lib_staking: "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512".to_string(),
        lib_quorum: "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9".to_string(),
        gateway: "0x9A676e781A523b5d0C0e43731313A708CB607508".to_string(),
        registry: "0xc5a5C42992dECbae36851359345FE25997F5C42d".to_string(),
        supply_source_address: "0xE6E340D132b5f46d1e472DebcD681B2aBc16e57E".to_string()
    };

    let home = std::env::var("HOME").unwrap();
    let current_dir = std::env::current_dir().unwrap();
    let repo_root_dir = Path::new(current_dir.to_str().unwrap()); // "/Users/jwagner/Workspaces/textile/github/ipc"
    let ipc_config_dir = Path::new(&home).join(".ipc");
    let ipc_config_folder = ipc_config_dir.to_str().unwrap(); // "/Users/jwagner/.ipc"

    println!("");
    println!("--------------------------------------------------");
    println!("               hoku is starting");
    println!("--------------------------------------------------");
    println!("");
    let three_seconds = time::Duration::from_secs(3);
    thread::sleep(three_seconds);

    if matches!(log_level, LogLevel::Debug) {
        println!("starting with clean .ipc dir");
    }

    Command::new("rm")
        .args([
            "-rf",
            &format!("{home}/.ipc")
        ])
        .output()
        .expect("failed to rm .ipc");
    Command::new("mkdir")
        .args([&format!("{home}/.ipc")])
        .output()
        .expect("failed to mkdir .ipc");

    let (anvil_stdout, anvil_stderr) = pipe_sub_command(
        String::from("ANVIL").blue().bold(),
        "anvil",
        ["--block-time", "10"].to_vec(),
        None,
        None,
        filtered_list.clone(),
        filtered_list.clone(),
        log_level
    );

    println!("[{}] anvil started", String::from("STARTUP").white().bold());

    // put validate private keys where the create_node func can find them
    for i in 0..(VALIDATOR_COUNT - 1) {
        let filepath = Path::new(ipc_config_folder).join(&format!("validator_{i}.sk"));
        let private_key = ANVIL_PRIVATE_KEYS[i as usize];

        write(filepath, private_key).unwrap();
    }

    println!("[{}] finished setting up validator private keys", String::from("STARTUP").white().bold());


    let lib_deploy_out = Command::new("npx")
        .current_dir(repo_root_dir.join("contracts"))
        .env("RPC_URL", "http://127.0.0.1:8545")
        .env("PRIVATE_KEY", ANVIL_PRIVATE_KEYS[0])
        .env("CHAIN_ID", "31337")
        .args([
            "hardhat",
            "deploy-libraries",
            "--network",
            "localnet"
        ])
        .output()
        .expect("failed to run hardhat deploy-libraries");

    println!("[{}] library contracts deployed", String::from("STARTUP").white().bold());

    if matches!(log_level, LogLevel::Debug) {
        println!("");
        println!("{:?}", lib_deploy_out);
        println!("");
    }

    let gateway_deploy_out = Command::new("npx")
        .current_dir(repo_root_dir.join("contracts"))
        .env("RPC_URL", "http://127.0.0.1:8545")
        .env("PRIVATE_KEY", ANVIL_PRIVATE_KEYS[0])
        .env("CHAIN_ID", "31337")
        .env("ACCOUNT_HELPER", &contracts.account_helper)
        .env("SUBNET_ID_HELPER", &contracts.subnet_id_helper)
        .env("CROSS_MSG_HELPER", &contracts.cross_msg_helper)
        .env("LIB_STAKING", &contracts.lib_staking)
        .env("LIB_QUORUM", &contracts.lib_quorum)
        .args([
            "hardhat",
            "deploy-gateway",
            "--network",
            "localnet"
        ])
        .output()
        .expect("failed to run hardhat deploy-gateway-local");

    if !matches!(log_level, LogLevel::Silent) {
        println!("[{}] gateway contracts deployed", String::from("STARTUP").white().bold());
    }
    if matches!(log_level, LogLevel::Debug) {
        println!("");
        println!("{:?}", gateway_deploy_out);
        println!("");
    }

    let registry_deploy_out = Command::new("npx")
        .current_dir(repo_root_dir.join("contracts"))
        .env("RPC_URL", "http://127.0.0.1:8545")
        .env("PRIVATE_KEY", ANVIL_PRIVATE_KEYS[0])
        .env("CHAIN_ID", "31337")
        .env("REGISTRY_CREATION_PRIVILEGES", "unrestricted")
        .env("ACCOUNT_HELPER", &contracts.account_helper)
        .env("SUBNET_ID_HELPER", &contracts.subnet_id_helper)
        .env("CROSS_MSG_HELPER", &contracts.cross_msg_helper)
        .env("LIB_STAKING", &contracts.lib_staking)
        .env("LIB_QUORUM", &contracts.lib_quorum)
        .env("GATEWAY_ADDRESS", &contracts.gateway)
        .args([
            "hardhat",
            "deploy-registry",
            "--network",
            "localnet"
        ])
        .output()
        .expect("failed to run hardhat deploy-subnet-registry-local");

    println!("[{}] registry contracts deployed", String::from("STARTUP").white().bold());
    if matches!(log_level, LogLevel::Debug) {
        println!("");
        println!("{:?}", registry_deploy_out);
        println!("");
    }

    // need to run clean or we hit upgradeable saftey validation errors resulting
    // from contracts with the same name
    Command::new("forge")
        .args(["clean"])
        .output()
        .expect("failed to forge clean");

    // TODO: sometimes this fails because we need to run `forge clean && forge build`
    let supply_source_token_out = Command::new("forge")
        .current_dir(repo_root_dir.join("hoku-contracts"))
        .env("RPC_URL", PARENT_ENDPOINT)
        .env("PRIVATE_KEY", &format!("0x{}",ANVIL_PRIVATE_KEYS[0]))
        .env("CHAIN_ID", "31337")
        .args([
            "script",
            "script/Hoku.s.sol",
            "--tc",
            "DeployScript",
            "0",
            "--sig",
            "run(uint8)",
            "--rpc-url",
            PARENT_ENDPOINT,
            "--broadcast",
            "-vv"
        ])
        .output()
        .expect("failed to run forge to deploy huko supply source token");

    println!("[{}] supply source deployed", String::from("STARTUP").white().bold());
    if matches!(log_level, LogLevel::Debug) {
        println!("");
        println!("{:?}", supply_source_token_out);
        println!("");
    }

    // fund each anvil wallet with supply token
    for i in 0..(ANVIL_PUBLIC_KEYS.len() - 1) {
        fund_wallet_token(
            ANVIL_PUBLIC_KEYS[i as usize],
            ANVIL_PRIVATE_KEYS[i as usize],
            "10100000000000000000000",
            &contracts.supply_source_address
        );
    }

    setup_config(repo_root_dir, ipc_config_folder);

    println!("[{}] setup config files", String::from("STARTUP").white().bold());

    let create_subnet_out = Command::new("ipc-cli")
        .args([
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
            "3",
            "--permission-mode",
            "federated",
            "--supply-source-kind",
            "erc20",
            "--supply-source-address",
            &contracts.supply_source_address
        ])
        .output()
        .expect("failed to create subnet");

    println!("[{}] created ipc subnet", String::from("STARTUP").white().bold());
    if matches!(log_level, LogLevel::Debug) {
        println!("");
        println!("{:?}", create_subnet_out);
        println!("");
    }

    let node_0_config = NodeConfig {
        log_level: &log_level,
        node_name: "validator-0",
        repo_root_dir: repo_root_dir,
        ipc_config_folder: ipc_config_folder,
        contracts: contracts,
        node_number: 0,
        is_bootstrap: true
    };

    let node_0_outs = create_node(node_0_config);

    // TODO: take command flag param to set number of nodes
    // create_node("validator-1", false);
    // create_node("validator-2", false);

    // echo "$bootstrap_output"
    // bootstrap_node_id=$(echo "$bootstrap_output" | sed -n '/CometBFT node ID:/ {n;p;}' | tr -d "[:blank:]")
    // bootstrap_peer_id=$(echo "$bootstrap_output" | sed -n '/IPLD Resolver Multiaddress:/ {n;p;}' | tr -d "[:blank:]" | sed 's/.*\/p2p\///')
    // echo "Bootstrap node started. Node id ${bootstrap_node_id}, peer id ${bootstrap_peer_id}"

    // bootstrap_node_endpoint=${bootstrap_node_id}@validator-0-cometbft:${CMT_P2P_HOST_PORTS[0]}
    // echo "Bootstrap node endpoint: ${bootstrap_node_endpoint}"
    // bootstrap_resolver_endpoint="/dns/validator-0-fendermint/tcp/${RESOLVER_HOST_PORTS[0]}/p2p/${bootstrap_peer_id}"
    // echo "Bootstrap resolver endpoint: ${bootstrap_resolver_endpoint}"
    println!("");
    println!("               hoku started");
    println!("--------------------------------------------------");
    println!("");
    println!("   \\\\\\\\\\\\\\\\\\\\\\              \\\\\\\\\\\\\\\\\\\\\\\\");
    println!("   \\\\\\\\\\\\\\\\\\\\\\\\\\            \\\\\\\\\\\\\\\\\\\\\\\\\\\\");
    println!("   \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\          \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\");
    println!("     \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\          \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\");
    println!("       \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\          \\\\\\\\\\\\\\\\\\\\\\\\\\\\");
    println!("         \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\          \\\\\\\\\\\\\\\\\\\\\\\\");
    println!("           \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\");
    println!("             \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\");
    println!("   \\\\\\\\\\\\\\\\\\\\\\\\           \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\");
    println!("   \\\\\\\\\\\\\\\\\\\\\\\\\\\\           \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\");
    println!("   \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\           \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\");
    println!("     \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\           \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\");
    println!("       \\\\\\\\\\\\\\\\\\\\\\\\\\\\             \\\\\\\\\\\\\\\\\\\\\\\\\\");
    println!("         \\\\\\\\\\\\\\\\\\\\\\\\               \\\\\\\\\\\\\\\\\\\\\\");
    println!("");
    println!("--------------------------------------------------");
    println!("");

    anvil_stdout.join().unwrap();
    anvil_stderr.join().unwrap();

    let tup_0 = node_0_outs.0;
    tup_0.0.join().unwrap();
    tup_0.1.join().unwrap();

    let tup_1 = node_0_outs.1;
    tup_1.0.join().unwrap();
    tup_1.1.join().unwrap();

    let tup_2 = node_0_outs.2;
    tup_2.0.join().unwrap();
    tup_2.1.join().unwrap();

    let tup_3 = node_0_outs.3;
    tup_3.0.join().unwrap();
    tup_3.1.join().unwrap();

    let tup_4 = node_0_outs.4;
    tup_4.0.join().unwrap();
    tup_4.1.join().unwrap();

    // TODO: at the end print contract addresses and other useful info
}

fn create_node(
    config: NodeConfig,
) -> (
    (JoinHandle<()>, JoinHandle<()>),
    (JoinHandle<()>, JoinHandle<()>),
    (JoinHandle<()>, JoinHandle<()>),
    (JoinHandle<()>, JoinHandle<()>),
    (JoinHandle<()>, JoinHandle<()>),
) {
    let NodeConfig {
        log_level,
        node_name,
        repo_root_dir,
        ipc_config_folder,
        contracts,
        node_number,
        is_bootstrap
    } = config;

    let ipc_config_path = Path::new(ipc_config_folder);
    let node_root_path = ipc_config_path.join(NETWORK_NAME).join(node_name);
    init_node_dir(&node_root_path, &node_number);
    let node_path = node_root_path.join(node_name);
    let cmt_dir = node_path.join("cometbft");
    // let env_file = node_root_path.join(".env");
    let cmt_config_file = if is_bootstrap { "bootstrap_config.toml" } else { "config.toml" };
    let cmt_config_dir = repo_root_dir.join("hoku-dev-cli").join("config").join("cometbft");
    let keys_dir = repo_root_dir.join("hoku-dev-cli").join("config").join("keys");
//    let fm_dir = node_path.join("fendermint");
//    let iroh_dir = node_path.join("iroh");
//    let iroh_config_dir = repo_root_dir.join("hoku-dev-cli").join("config").join("iroh");


// TODO: where do all these keys come from?  We don't need to be copying them.
    Command::new("cp")
    .args([
        keys_dir.join("validator_key.sk").to_str().unwrap(),
        node_path.join("keys").join("validator_key.sk").to_str().unwrap()
    ])
    .output()
    .expect("failed to copy key file");

    Command::new("cp")
    .args([
        keys_dir.join("validator_key.pk").to_str().unwrap(),
        node_path.join("keys").join("validator_key.pk").to_str().unwrap()
    ])
    .output()
    .expect("failed to copy key file");

    Command::new("cp")
    .args([
        keys_dir.join("network_key.sk").to_str().unwrap(),
        node_path.join("keys").join("network_key.sk").to_str().unwrap()
    ])
    .output()
    .expect("failed to copy key file");

    Command::new("cp")
    .args([
        keys_dir.join("network_key.pk").to_str().unwrap(),
        node_path.join("keys").join("network_key.pk").to_str().unwrap()
    ])
    .output()
    .expect("failed to copy key file");

    // copy cometbft config into this validator's directory
    let copy_cmt_conf = Command::new("cp")
        .args([
            cmt_config_dir.join(cmt_config_file).to_str().unwrap(),
            cmt_dir.join("config.toml").to_str().unwrap()
        ])
        .output()
        .expect("failed to copy cometbft bootstrap config");

    if matches!(log_level, LogLevel::Debug) {
        println!("copied cometbft config");
        println!("{:?}", copy_cmt_conf);
        println!("");
    }

    // "iroh-start",
    let iroh_out = pipe_sub_command(
        String::from("IROH").magenta().bold(),
        "iroh",
        [
            "--rpc-addr",
            "127.0.0.1:4919",
            "--config",
            ipc_config_path.join("iroh.config.toml").to_str().unwrap(),
            "start"
        ].to_vec(),
        Some([
            [
                "IROH_DATA_DIR",
                node_path.join("iroh").join("data").to_str().unwrap()
            ].to_vec()
        ].to_vec()),
        None,
        vec![],
        vec![],
        log_level
    );

    // "iroh-wait",
    println!("wait 30 seconds");
    let thirty_seconds = time::Duration::from_secs(30);
    thread::sleep(thirty_seconds);

    println!("");

    // "cometbft-init",
    let cometbft_init_out = Command::new("cometbft")
        .env("CMT_PROXY_APP", &format!("tcp://127.0.0.1:26658"))
        .env("CMT_RPC_MAX_SUBSCRIPTION_CLIENTS", "10")
        .env("CMT_RPC_MAX_SUBSCRIPTIONS_PER_CLIENT", "1000")
        .args([
            "init",
            "--home",
            cmt_dir.to_str().unwrap()
        ])
        .output()
        .expect("failed to init cometbft");

    println!("initialized cometbft");
    println!("{:?}", cometbft_init_out);
    println!("");

    // "cometbft-start",
    let cometbft_out = pipe_sub_command(
        String::from("COMETBFT").cyan().bold(),
        "cometbft",
        [
            "node",
            "--proxy_app",
            "kvstore",
            "--home",
            cmt_dir.to_str().unwrap()
        ].to_vec(),
        None,
        None,
        vec![
            Regex::new("received proposal"),
            Regex::new("received complete proposal block"),
            Regex::new("finalizing commit of block"),
            Regex::new("executed block"),
            Regex::new("committed state"),
            Regex::new("indexed block exents"),
            Regex::new("Timed out")
        ],
        vec![],
        log_level
    );

    // "cometbft-wait",
    println!("wait 30 seconds");
    thread::sleep(thirty_seconds);
    println!("");

    // TODO: how to get bootstrap peer id?
    let bootstrap_peer_id = "16Uiu2HAmH5dzPoWL2wtQQGaPnwJZZn988xDAxbpsYu8nBhfyCrfH";

    // Bootstrap node started. Node id 20ee1f8f1c39602c5f7bf8e161c85a6eaf194c86, peer id 16Uiu2HAmH5dzPoWL2wtQQGaPnwJZZn988xDAxbpsYu8nBhfyCrfH
    // Bootstrap node endpoint: 20ee1f8f1c39602c5f7bf8e161c85a6eaf194c86@validator-0-cometbft:26656
    // Bootstrap resolver endpoint: /dns/validator-0-fendermint/tcp/26655/p2p/16Uiu2HAmH5dzPoWL2wtQQGaPnwJZZn988xDAxbpsYu8nBhfyCrfH
    println!("node path: {}", node_path.to_str().unwrap());
    // "fendermint-start-validator",
    let fendermint_out = pipe_sub_command(
        String::from("FENDERMINT").yellow().bold(),
        "./target/release/fendermint",
        ["run"].to_vec(),
        Some([
            ["FM_NETWORK", "test"].to_vec(),
            ["FM_TRACING_CONSOLE_LEVEL", "info"].to_vec(),
            [
                "FM_VALIDATOR_KEY__PATH",
                node_path.join("keys").join("validator_key.sk").to_str().unwrap()
            ].to_vec(),
            ["FM_VALIDATOR_KEY__KIND", "regular"].to_vec(),
            ["FM_RESOLVER__CONNECTION__LISTEN_ADDR", "/ip4/127.0.0.1/tcp/3001"].to_vec()
        ].to_vec()),
        repo_root_dir.to_str(),
        vec![],
        vec![],
        log_level
    );

    println!("wait 30 seconds");
    thread::sleep(thirty_seconds);
    println!("");

    // "start the etherium json-rpc facade",
    let evm_out = pipe_sub_command(
        String::from("EVM_RPC").red().bold(),
        "./target/release/fendermint",
        [
            "eth",
            "run",
            // "--http-url",
            // "put what ever the cometbft(tendermint) url is in here"
        ].to_vec(),
        None,
        None,
        vec!(),
        vec!(),
        log_level
    );

    // "objects-start",
    let objects_out = pipe_sub_command(
        String::from("OBJECTS").green().bold(),
        "./target/release/fendermint",
        ["objects", "run"].to_vec(),
        Some([["FM_NETWORK", "test"].to_vec()].to_vec()),
        None,
        vec!(),
        vec!(),
        log_level
    );

    // "promtail-start",
//    let prom_out = Command::new("docker")
//        .args([
//            "run",
//            "-d",
//            "--name",
//            promtail_container_name,
//            "--network",
//            //NETWORK_NAME,
//            "host",
//            "--publish",
//            &format!("127.0.0.1:{}:9080", PROMTAIL_AGENT_HOST_PORTS[node_number as usize]),
//            "--volume",
//            "/var/run/docker.sock:/var/run/docker.sock",
//            "--volume",
//            &format!("{ipc_config_folder}/promtail-config.yaml:/etc/promtail/promtail-config.yaml"),
//            "--volume",
//            &format!("{:?}/data/logs:/var/log/fendermint/", fm_dir),
//            "--volume",
//            &format!("{:?}/logs:/var/log/iroh/", iroh_dir),
//            PROMTAIL_DOCKER_IMAGE,
//            &format!("--client.external-labels=host=$(hostname),node={node_name}"),
//            &format!("--config.file=/etc/promtail/promtail-config.yaml"),
//            "--client.url=http://loki:3100/loki/api/v1/push"
//        ])
//        .output()
//        .expect(&format!("failed to start promtail {:?}", node_number));
//
//    println!("promtail started");
//    println!("{:?}", prom_out);
//    println!("");

    println!("created subnet node {node_name}");
    println!("");

    return (iroh_out, cometbft_out, fendermint_out, evm_out, objects_out);
}

fn create_keystore(filepath: &str) {
    let file = File::create(filepath).unwrap();
    let mut keys: Vec<Keystore> = Vec::new();

    for i in 0..(ANVIL_PUBLIC_KEYS.len() - 1) {
        let key = Keystore {
            address: ANVIL_PUBLIC_KEYS[i].to_string(),
            private_key: ANVIL_PRIVATE_KEYS[i].to_string()
        };
        keys.push(key);
    }

    serde_json::to_writer_pretty(&file, &keys).unwrap();
}

fn fund_wallet_token(public_key: &str, private_key: &str, token_amount: &str, supply_source_address: &String) {
    println!("[{}] funding address {} with supply token", String::from("STARTUP").white().bold(), public_key);
    Command::new("cast")
        .args([
            "send",
            supply_source_address,
            "mint(address,uint256)",
            public_key,
            token_amount,
            "--rpc-url",
            PARENT_ENDPOINT,
            "--private-key",
            private_key
        ])
        .output()
        .expect("failed to fund wallet with source token");
}

fn init_node_dir(dir: &Path, node_number: &u8) {

    println!("init node dir {:?}", dir.to_str().unwrap());

    Command::new("rm")
        .args(["-rf", dir.to_str().unwrap()])
        .output()
        .expect(&format!("failed to remove dir {:?}", dir));
    Command::new("mkdir")
        .args(["-p", dir.to_str().unwrap()])
        .output()
        .expect(&format!("failed to create {:?}", dir));

    Command::new("touch")
        .args([
            dir.join(".env").to_str().unwrap()
        ])
        .output()
        .expect("failed to create .env file");

    let validator_dir = dir.join(format!("validator-{:?}", node_number));
    Command::new("mkdir")
        .args([
            validator_dir.to_str().unwrap()
        ])
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
        .args(["-p", validator_dir.join("fendermint").join("data").to_str().unwrap()])
        .output()
        .expect("failed to create fendermint data dir");

    Command::new("mkdir")
        .args(["-p", validator_dir.join("iroh").to_str().unwrap()])
        .output()
        .expect("failed to create iroh dir");
    Command::new("mkdir")
        .args(["-p", validator_dir.join("iroh").join("blobs").to_str().unwrap()])
        .output()
        .expect("failed to create iroh blobs dir");

    Command::new("mkdir")
        .args(["-p", validator_dir.join("keys").to_str().unwrap()])
        .output()
        .expect("failed to create keys dir");
}

fn setup_config(repo_root_dir: &Path, config_folder: &str) {

    // copy config files
    Command::new("cp")
        .args([
            &format!("{}/scripts/deploy_subnet/.ipc-local/config.toml",
            repo_root_dir.display()),
            config_folder
        ])
        .output()
        .expect("failed to copy ipc config");
    Command::new("cp")
        .args([
            &format!("{}/infra/prometheus/prometheus.yaml",
            repo_root_dir.display()),
            config_folder
        ])
        .output()
        .expect("failed to copy prometheus config");
    Command::new("cp")
        .args([
            &format!("{}/infra/loki/loki-config.yaml",
            repo_root_dir.display()),
            config_folder
        ])
        .output()
        .expect("failed to copy loki config");
    Command::new("cp")
        .args([
            &format!("{}/infra/promtail/promtail-config.yaml",
            repo_root_dir.display()),
            config_folder
        ])
        .output()
        .expect("failed to copy promtail config");
    Command::new("cp")
        .args([
            &format!("{}/infra/iroh/iroh.config.toml",
            repo_root_dir.display()),
            config_folder
        ])
        .output()
        .expect("failed to copy iroh config");

    create_keystore(&format!("{config_folder}/evm_keystore.json"));
}