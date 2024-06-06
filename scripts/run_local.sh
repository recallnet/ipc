#!/bin/bash

set -e

# ------------------------------------------------------------
# COMMAND SETUP & HELPERS
# ------------------------------------------------------------

# Dicatate local behavior in this and `setup.sh` scripts
export FM_NETWORK=test

# Ports
FM_PORT=26658
CMT_PORT=26657
IPFS_PORT=5001
PROXY_PORT=8001
EVM_PORT=8545
HOST=127.0.0.1
PROXY_URL="http://$HOST:$PROXY_PORT"
IPC_CONFIG_FOLDER=test-network

# Polling settings
MAX_ATTEMPTS=10
INTERVAL=1

# Optionally, pass "build=true" to build & install Fendermint, or pass
# "silent=false" to log verbosely from Fendermint, CometBFT, and proxy
# API. Example: `./scripts/run_local.sh build=true silent=false`
BUILD="false"
SILENCE_LOG="true"
for arg in "$@"
do
    case $arg in
        build=*)
        BUILD="${arg#*=}"
        ;;
        silent=*)
        SILENCE_LOG="${arg#*=}"
        ;;
        *)
        # Unknown 
        ;;
    esac
done

# Run commands based on the SILENCE_LOG variable
run_cmd() {
  if [ "$SILENCE_LOG" = "true" ]; then
    "$@" &> /dev/null
  else
    "$@" 
  fi
}

# Kill process and its children upon receiving SIGINT
kill_with_children() {
    local pid=$1
    if kill -0 $pid >/dev/null 2>&1; then
        local children=$(pgrep -P $pid)
        for child in $children; do
            kill_with_children $child
        done
        kill $pid
    fi
}
cleanup() {
    kill_with_children $pid1 # Fendermint
    kill_with_children $pid2 # CometBFT
    [ ! -z "${pid3+x}" ] && kill_with_children $pid3 # An IPFS daemon might run independent of this script
    kill_with_children $pid4 # Proxy API
    exit
}
trap cleanup SIGINT

# Function to wait for a port to be open, with optional `path` to check for HTTP
# 200 response (used for proxy API)
check_status() {
    local port=$1
    local max_attempts=${2:-1} # Default to 1 attempts
    local interval=${3:-0} # Default to 0s interval
    local path=${4:-}  # Empty path by default
    local host=$HOST

    for ((attempt=1; attempt<=max_attempts; attempt++))
    do
        if [ -z "$path" ]; then
            # No path provided, check if the port is open
            nc -z $host $port >/dev/null 2>&1
            if [ $? -eq 0 ]; then
                return 0
            fi
        else
            # Path provided, check if the URL is accessible
            url="http://$host:$port$path"
            http_status=$(curl -s -o /dev/null -w "%{http_code}" $url)
    
            if [ "$http_status" -eq 200 ]; then
               return 0
            fi
        fi

        sleep $interval
    done

    return 1
}

# Process a secret key file and output the public key (for logging)
process_keyfile() {
    local sk_file=$1
    local temp_dir=$2
    local name=$(basename "$sk_file" .sk)
    local secret_key=$(cat "$sk_file" | base64 -d | xxd -p -c 1000000)
    local public_key=$(fendermint key into-eth --secret-key "$sk_file" --name tmp -o "$temp_dir" && (cat $(find "$temp_dir" -name "*.addr")))
    
    echo "$name:  "
    echo "  Public: 0x$public_key"
    echo "  Private: $secret_key"
}

# Check `ipfs daemon` is running, else, start it
echo $'\nNode setup\n=========='
echo "Initializing local network (Fendermint, CometBFT, IPFS, EVM, proxy)..."
if [ -f "${HOME}/.ipfs/api" ]; then
    ipfs_is_running="true"
else
    ipfs_is_running="false"
fi

# Check if ports are in use
ports_in_use=()
check_and_log_port() {
    local port=$1
    if check_status $port; then
        ports_in_use+=($port)
    fi
}
check_and_log_port $FM_PORT
check_and_log_port $CMT_PORT
if [ "$ipfs_is_running" = "false" ]; then
    check_and_log_port $IPFS_PORT
fi
check_and_log_port $PROXY_PORT
check_and_log_port $EVM_PORT

# If any ports are in use, exit
if [ ${#ports_in_use[@]} -gt 0 ]; then
    echo "The following ports are in use: ${ports_in_use[*]}"
    # Get PIDs of processes using the ports
    pids_in_use=()
    for port in "${ports_in_use[@]}"; do
        pid=$(lsof -t -i:$port)
        if [ ! -z "$pid" ]; then
            pids_in_use+=($pid)
        fi
    done
    # Remove duplicates
    pids_in_use=($(echo "${pids_in_use[@]}" | tr ' ' '\n' | sort -u | tr '\n' ' '))
    echo "PIDs: ${pids_in_use[*]}"
    echo "Run this to kill processes: kill -9 ${pids_in_use[*]}"
    exit 1
fi

# ------------------------------------------------------------
# BUILD & START NETWORK
# ------------------------------------------------------------

# Check if Fendermint needs to be built & installed
if [ "$BUILD" = "true" ]; then
  run_cmd cargo install --path fendermint/app
else
  echo "Skipping Fendermint build & installation step..."
fi

# Start Fendermint, CometBFT, and proxy API after running setup script, which
# builds contracts and seeds the network with test accounts
run_cmd ./scripts/setup.sh

# Wait for Fendermint to be ready before proceeding
run_cmd ./scripts/run_fendermint.sh &
pid1=$!
if check_status $FM_PORT $MAX_ATTEMPTS $INTERVAL; then
    echo "Fendermint is ready on port '$FM_PORT'"
else
    echo "Fendermint failed to start...exiting"
    exit 1
fi

# Wait for CometBFT to be ready before proceeding
run_cmd ./scripts/run_cometbft.sh &
pid2=$!
if check_status $CMT_PORT $MAX_ATTEMPTS $INTERVAL; then
    echo "CometBFT is ready on port '$CMT_PORT'"
else
    echo "CometBFT failed to start...exiting"
    exit 1
fi

# Start IPFS if it is not running already on the default port
if [ $ipfs_is_running = "false" ]; then
    run_cmd ipfs daemon &
  pid3=$!
  if check_status $IPFS_PORT $MAX_ATTEMPTS $INTERVAL; then
      echo "IPFS is ready on port '$IPFS_PORT'"
  else
      echo "IPFS failed to start...exiting"
      exit 1
  fi
else
    echo "IPFS is already running on port '$IPFS_PORT'"
fi

# Wait for EVM node to be ready before proceeding
run_cmd ./scripts/run_evm.sh &
pid2=$!
if check_status $EVM_PORT $MAX_ATTEMPTS $INTERVAL; then
    echo "EVM API is ready on port '$EVM_PORT'"
else
    echo "EVM API failed to start...exiting"
    exit 1
fi

# Wait for proxy readiness via "/health" endpoint
run_cmd ./scripts/run_proxy.sh &
pid4=$!
if check_status $PROXY_PORT $MAX_ATTEMPTS $INTERVAL "/health"; then
    echo "Proxy API is ready on port '$PROXY_PORT'"
else
    echo "Proxy API failed to start...exiting"
    exit 1
fi

# ------------------------------------------------------------
# CREATE MACHINES & LOG ENVIROMENT INFO
# ------------------------------------------------------------


# Log accounts by iterating through each .sk file in the keys directory, sort
# them, and process them (note: standard Hardhat accounts are included)
echo $'\nAccounts\n========'
temp_dir=$(mktemp -d)
accounts=()
for sk_file in $(find "$IPC_CONFIG_FOLDER/keys" -name "*.sk" | sort -V); do
    result=$(process_keyfile "$sk_file" "$temp_dir")
    output+=("$result")
done
printf "%s\n" "${output[@]}"
rm -rf "$temp_dir"

# Create the object store and accumulator
echo $'\nDeployed machines\n================='
# Capture otuput `robust_address` and `actor_id` of curl command
read os_robust_address os_actor_id < <(curl -s -X POST -H 'X-ADM-BroadcastMode: commit' $PROXY_URL/v1/machines/objectstores | jq -r '.data | "\(.robust_address) \(.actor_id)"')
read acc_robust_address acc_actor_id < <(curl -s -X POST -H 'X-ADM-BroadcastMode: commit' $PROXY_URL/v1/machines/accumulators | jq -r '.data | "\(.robust_address) \(.actor_id)"')

echo "Object store:"
echo "  Addresss: $os_robust_address"
echo "  Machine ID: $os_actor_id"
echo "Accumulator:"
echo "  Addresss: $acc_robust_address"
echo "  Machine ID: $acc_actor_id"

# Node APIs
echo $'\nNode APIs\n========='
echo "Fendermint: http://$HOST:$FM_PORT"
echo "CometBFT: http://$HOST:$CMT_PORT"
echo "EVM: http://$HOST:$EVM_PORT"
echo "Proxy: $PROXY_URL"

# Subnet & chain ID
echo $'\nChain info\n=========='
echo "Subnet ID: /r314159/t410f726d2jv6uj4mpkcbgg5ndlpp3l7dd5rlcpgzkoi" # hard coded
chain_id_hex=$(curl -s --location --request POST 'http://localhost:8545/' --header 'Content-Type: application/json' --data-raw '{ "jsonrpc":"2.0", "method":"eth_chainId", "params":[], "id":1 }' | jq -r '.result')
chain_id=$(printf "%d" $chain_id_hex)
echo "Chain ID: $chain_id ($chain_id_hex)"

echo $'\nNetwork is ready to accept requests!'

wait $pid1
wait $pid2
if [ ! -z "${pid3+x}" ]; then
    wait $pid3
fi
wait $pid4
