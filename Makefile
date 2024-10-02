# Each major sub-repository in this monorepo has their own Makefiles;
# instead of making an even more compilicated common one, let's delegate to them.

default:
	cd contracts && make gen
	cargo build --locked --release
	./target/release/ipc-cli --version
	./target/release/fendermint --version

SUBTREES := fendermint ipc ipld/resolver contracts

test: $(patsubst %, test/%, $(SUBTREES))

# Using `cd` instead of `-C` so $(PWD) is correct.
test/%:
	cd $* && make test

lint/%:
	cd $* && make lint || echo "$* lint failed"

license:
	./scripts/add_license.sh

lint: license $(patsubst %, lint/%, $(SUBTREES))

install:
	cd fendermint && make install && cargo install iroh-cli

config-devnet:
	./scripts/setup.sh

run-devnet-iroh:
	iroh --rpc-addr 127.0.0.1:4919 start

run-devnet-fendermint:
	./scripts/run_fendermint.sh

run-devnet-cometbft:
	./scripts/run_cometbft.sh

run-devnet-objects:
	./scripts/run_objects.sh

run-devnet-evm:
	fendermint eth run

run-localnet:
	./scripts/deploy_subnet/deploy.sh localnet
