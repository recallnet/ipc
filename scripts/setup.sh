#!/bin/sh
set -eu

GOPATH="${GOPATH:-$HOME/go}"
# If `FM_NETWORK=test` is set, then use a relative path to `builtin-actors` vs.
# pulling from the remote repo
if [ "$FM_NETWORK" = "test" ]; then
    BUILTIN_ACTORS_PATH="../builtin-actors"
else
    BUILTIN_ACTORS_PATH="builtin-actors"
fi

# Create a new Genesis file
rm -rf test-network
mkdir test-network
fendermint genesis --genesis-file test-network/genesis.json new --chain-name test --base-fee 1000 --timestamp 1680101412 --power-scale 0

# Create some keys
mkdir test-network/keys
for NAME in bob charlie dave; do
  fendermint key gen --out-dir test-network/keys --name $NAME;
done

## Also include all standard Hardhat accounts
temp_dir=$(mktemp -d)
hardhat_accounts=(
  "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
  "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
  "5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"
  "7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6"
  "47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a"
  "8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba"
  "92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e"
  "4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356"
  "dbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97"
  "2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6"
  "f214f2b2cd398c806f84e317254e0f0b801d0643303237d97a22a48e01628897"
  "701b615bbdfb9de65240bc28bd21bbc0d996645a3dd57e7b12bc2bdf6f192c82"
  "a267530f49f8280200edf313ee7af6b827f2a8bce2897751d06a843f644967b1"
  "47c99abed3324a2707c28affff1267e45918ec8c3f20b8aa892e8b065d2942dd"
  "c526ee95bf44d8fc405a158bb884d9d1238d99f0612e9f33d006bb0789009aaa"
  "8166f546bab6da521a8369cab06c5d2b9e46670292d85c875ee9ec20e84ffb61"
  "ea6c44ac03bff858b476bba40716402b03e41b8e97e276d1baec7c37d42484a0"
  "689af8efa8c651a91ad287602527f3af2fe9f6501a7ac4b061667b5a93e037fd"
  "de9be858da4a475276426320d5e9262ecfc3ba460bfac56360bfa6c4c28b4ee0"
  "df57089febbacf7ba0bc227dafbffa9fc08a93fdc68e1e42411a14efcf23656e"
)
## Create files for each account, labeled as `account_0`, `account_1`, etc.
for i in "${!hardhat_accounts[@]}"; do
  echo "${hardhat_accounts[$i]}" | tr -d '\n' > "$temp_dir/account_$i.sk"
done
## Iterate therough each Hardhat file and create Fendermint compatible keypairs
for account in $temp_dir/*.sk; do
  base_name=$(basename "$account" .sk)
  fendermint key from-eth --out-dir test-network/keys --secret-key "$account" --name "$base_name"
done
rm -rf "$temp_dir"

## Use fixed address for alice to make dev w/ ADM cli less painful
echo "HDI9SU0dBp/kyJE1Ch7GkcQhbBdBigyzx1M7FDvSuBI=" | tr -d '\n' > test-network/keys/alice.sk
echo "Ayh506Z/KRZnDgtarffTZQympqQ8A4hfwse1gK9t0NJi" | tr -d '\n' > test-network/keys/alice.pk


# Add accounts to the Genesis file
## A stand-alone account
fendermint genesis --genesis-file test-network/genesis.json add-account --public-key test-network/keys/alice.pk --balance 1000 --kind ethereum
## A multi-sig account
fendermint genesis --genesis-file test-network/genesis.json add-multisig --public-key test-network/keys/bob.pk --public-key test-network/keys/charlie.pk --public-key test-network/keys/dave.pk --threshold 2 --vesting-start 0 --vesting-duration 1000000 --balance 30
## Hardhat accounts
for i in "${!hardhat_accounts[@]}"; do
  fendermint genesis --genesis-file test-network/genesis.json add-account --public-key test-network/keys/account_$i.pk --balance 1000 --kind ethereum
done

# Add validators to the Genesis file
fendermint genesis --genesis-file test-network/genesis.json add-validator --public-key test-network/keys/bob.pk --power 1

# Add ipc to the Genesis file
fendermint genesis --genesis-file test-network/genesis.json ipc gateway --subnet-id /r31415926 --bottom-up-check-period 10 --msg-fee 1 --majority-percentage 65

# Configure Tendermint
rm -rf "$HOME/.cometbft"
"$GOPATH/bin/cometbft" init

## Convert the Genesis file
mv "$HOME/.cometbft/config/genesis.json" "$HOME/.cometbft/config/genesis.json.orig"
fendermint genesis --genesis-file test-network/genesis.json into-tendermint --out "$HOME/.cometbft/config/genesis.json"
## Convert the private key
mv "$HOME/.cometbft/config/priv_validator_key.json" "$HOME/.cometbft/config/priv_validator_key.json.orig"
fendermint key into-tendermint --secret-key test-network/keys/bob.sk --out "$HOME/.cometbft/config/priv_validator_key.json"

## Setup data directory and copy default app config
rm -rf "$HOME/.fendermint"
mkdir -p "$HOME/.fendermint/data"
cp -r ./fendermint/app/config "$HOME/.fendermint/config"

## Generate a network key for the IPLD resolver
mkdir -p "$HOME/.fendermint/keys"
fendermint key gen --out-dir "$HOME/.fendermint/keys" --name network

## Copy validator keys
cp test-network/keys/bob.pk "$HOME/.fendermint/keys/validator.pk"
cp test-network/keys/bob.sk "$HOME/.fendermint/keys/validator.sk"

## Copy IPC contracts
mkdir -p "$HOME/.fendermint/contracts"
cp -r ./contracts/out/* "$HOME/.fendermint/contracts"

# Build actors
(cd $BUILTIN_ACTORS_PATH && make bundle-mainnet)
mkdir -p fendermint/builtin-actors/output
cp $BUILTIN_ACTORS_PATH/output/builtin-actors-mainnet.car fendermint/builtin-actors/output/bundle.car
cp fendermint/builtin-actors/output/bundle.car "$HOME/.fendermint/bundle.car"
cargo build --release -p fendermint_actors
cp fendermint/actors/output/custom_actors_bundle.car "$HOME/.fendermint/custom_actors_bundle.car"
