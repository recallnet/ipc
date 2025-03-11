# Each major sub-repository in this monorepo has their own Makefiles;
# instead of making an even more compilicated common one, let's delegate to them.

default:
	cd contracts && make gen
	cargo build --locked --release
	./target/release/ipc-cli --version
	./target/release/fendermint --version

SUBTREES_RUST := fendermint ipc ipld/resolver recall
SUBTREES_CONTRACTS := contracts
SUBTREES_ALL := $(SUBTREES_RUST) $(SUBTREES_CONTRACTS)

test: test-rust test-contracts

test-rust: $(patsubst %, test/%, $(SUBTREES_RUST))

test-contracts: $(patsubst %, test/%, $(SUBTREES_CONTRACTS))

# Using `cd` instead of `-C` so $(PWD) is correct.
test/%:
	cd $* && make test

lint/%:
	cd $* && make lint || { echo "$* lint failed"; exit 1; }

license:
	./scripts/add_license.sh

lint: license $(patsubst %, lint/%, $(SUBTREES_ALL))

## Recall

config-devnet:
	PATH="./target/release:$(PATH)" \
	./scripts/setup.sh

run-devnet-fendermint:
	rm -rf ~/.fendermint/data/rocksdb
	FM_NETWORK=test \
	FM_TRACING__CONSOLE__LEVEL=info,fendermint=debug,recall_executor=debug \
	FM_VALIDATOR_KEY__PATH=keys/validator.sk \
	FM_VALIDATOR_KEY__KIND=regular \
	FM_RESOLVER__CONNECTION__LISTEN_ADDR=/ip4/127.0.0.1/tcp/3001 \
	./target/release/fendermint run --iroh-path iroh-resolver-path # TODO: what path

run-devnet-cometbft:
	cometbft unsafe-reset-all
	cometbft start

run-devnet-objects:
	FM_NETWORK=test FM_OBJECTS__TRACING__CONSOLE__LEVEL=debug ./target/release/fendermint objects run

run-devnet-evm:
	FM_ETH__TRACING__CONSOLE__LEVEL=debug ./target/release/fendermint eth run

run-localnet:
	./scripts/deploy_subnet/deploy.sh localnet

stop-localnet:
	./scripts/deploy_subnet/stop_local.sh
