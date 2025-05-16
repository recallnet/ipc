#!/usr/bin/env nu

use lib/localnet.nu
use lib/state-engine.nu
use lib/local-files.nu
use lib/steps.nu
use lib/parent-chain.nu
use lib/util.nu
use lib/subnet.nu

const anvil0_pk = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

# See subcommands
def main [] {}

def "main stop" [] {
  docker ps --format json | lines | each {from json} | where Names =~ $"localnet-" | each {docker rm -f $in.ID}
}

def "main run" [
  --fendermint-image: string = "fendermint",
  --workdir: string = "./localnet-data",
  --node-count: int = 2, # how many nodes to run
  --dc-repo: string = "https://github.com/recallnet/recall-docker-compose.git", # recall-docker-compose repo to clone
  --dc-branch: string = "main", # recall-docker-compose branch
  --rebuild-fendermint-image, # rebuild local fendermint image if --fendermint-image=fendermint, no effect otherwise
  --reset, # delete previous data
  ] {

  if $reset {
    reset $workdir
  }

  let all_node_indexes = (0..($node_count - 1) | each {$in})
  let additional_node_indexes = ($all_node_indexes | skip 1)

  let build_fendermint_image = (if $rebuild_fendermint_image and $fendermint_image == "fendermint" {[
    { name: "build_fendermint_image" fn: { local-files build-fendermint-image } }
  ]} else [])
  let bootstrap_additional_nodes = ($additional_node_indexes | each { |ix| [
    { name: $"localnet_node($ix)_create_wallet" fn: { util create-wallet $"validator($ix)"} }
    ...(steps prepare-validator $"validator($ix)" 2e18)
    { name: $"node($ix)_get_funds_on_subnet" fn: { subnet send-funds $env.state.validator0 ($env.state | get $"validator($ix)") 5e18} }

    { name: $"localnet_run_node($ix)" fn: {localnet run-localnet-node $ix $dc_repo $dc_branch --bootstrap}}
    { name: $"localnet_node($ix)_wait_for_cometbft" fn: { localnet wait-for-cometbft $ix }}

    { name: $"localnet_node($ix)_wait_for_sync" fn: { localnet wait-for-sync $ix }}
    ...(steps join-subnet $"validator($ix)" 2)
  ]} | flatten)

  let run_full_nodes = ($all_node_indexes | each { |ix| [
    { name: $"localnet_run_node($ix)_full" fn: {localnet run-localnet-node $ix $dc_repo $dc_branch}}
  ]} | flatten)

  let get_funds_step = {parent-chain send-funds $env.state.supply_source_owner 100e18 --from-private-key $anvil0_pk}
  let set_up_contract_owner_steps = [
    { name: "subnet_contract_create_wallet" fn: { util create-wallet "subnet_contract_owner"} }
    { name: "subnet_contract_get_funds_on_subnet" fn: { subnet send-funds $env.state.faucet_owner $env.state.subnet_contract_owner 100e18} }
    { name: "set_subnet_contract_owner_ref" fn: { do $env.state.update { subnet_contract_owner_ref: "subnet_contract_owner" } } }
  ]

  let steps = [
    { name: "localnet_init" fn: { localnet init-state $workdir $fendermint_image}}
    { name: "update_submodules" fn: { git submodule update --init --recursive }}
    ...$build_fendermint_image
    { name: "localnet_start_anvil" fn: {localnet run-anvil }}
    ...(steps get-create-subnet-steps $get_funds_step)
    { name: "localnet_run_node0_bootstrap" fn: {localnet run-localnet-node 0 $dc_repo $dc_branch --bootstrap}}
    { name: "localnet_node0_wait_for_cometbft" fn: { localnet wait-for-cometbft 0 }}

    ...$bootstrap_additional_nodes
    ...(steps get-deploy-subnet-contracts-steps $set_up_contract_owner_steps)

    ...$run_full_nodes
  ]

  state-engine run $workdir $steps --log-prefix "localnet"
}

def "main create-docker-image" [
  --workdir: string = "./localnet-data",
  --fendermint-image: string = "fendermint",
  --rebuild-fendermint-image, # rebuild local fendermint image if --fendermint-image=fendermint, no effect otherwise
  --node-count: int = 2, # how many nodes to run
  --dc-repo: string = "https://github.com/recallnet/recall-docker-compose.git", # recall-docker-compose repo to clone
  --dc-branch: string = "main", # recall-docker-compose branch
  --local-image-tag: string, # build a local image with the given tag
  --push-multi-arch-tags: string, # a comma separated list of tags. If not empty, build a multi-arch image and push it to the registry.
  --reset, # delete previous data
  ] {
  if $reset { reset $workdir }

  let shutdown_steps = (($node_count - 1)..0 | each { |ix| { name: $"docker_image_stop_localnet_($ix)" fn: {localnet stop-node $ix}} })

  let steps = [
    { name: "docker_image_start_localnet" fn: {(main run
      --fendermint-image $fendermint_image
      --workdir $workdir
      --node-count $node_count
      --dc-repo $dc_repo
      --dc-branch $dc_branch
      --rebuild-fendermint-image=$rebuild_fendermint_image
    )} }
    ...$shutdown_steps
    { name: "docker_image_stop_anvil" fn: {localnet stop-anvil}}
    { name: "docker_image_build" fn: {localnet build-dind-image $local_image_tag $push_multi_arch_tags }}
  ]

  state-engine run $workdir $steps --log-prefix "create-docker-image"
}

def reset [workdir: string] {
    print "resetting..."
    docker ps --format json | lines | each {from json} | where Names =~ $"localnet-" | each {docker rm -f $in.ID}
    rm -rf $workdir
}

# Kill all containers of the node
def "main kill-node" [
  ix: int, # Index of the node to kill
  ] {
  docker ps --format json | lines | each {from json} | where Names =~ $"localnet-node-($ix)" | each {docker rm -f $in.ID}
}

def "main reset-node" [
  ix: int, # Index of the node to reset
  ] {
  main kill-node $ix
  cd $"localnet-data/node-($ix)"
  rm -r workdir
  ./init-workdir
  cd ./workdir
  docker compose up -d
}

# Get funds on subnet
def "main get-funds" [address: string, amount: float] {
  let state = (open ./localnet-data/state.yml)
  cast send --private-key $state.faucet_owner.private_key -r http://localhost:8645 --value $amount $address
}
