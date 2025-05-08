# Subnet Scripts

* `set-up-subnet.nu` - setting up a subnet
  * `./set-up-subnet.nu init workdir1 testnet`
  * `./set-up-subnet.nu create-subnet workdir1`
  * `./set-up-subnet.nu deploy-subnet-contracts workdir1`
* `localnet.nu` - to start a localnet. Call `./localnet.nu run -h` to see help.

Both scripts require `nushell` `>=v0.101.0` ([github releases](https://github.com/nushell/nushell/releases)).

## Install nushell
If you do not have nushell installed, you can call `source ./set-up-nu.sh` that will download the required nushell version to `./.nu` and add it to your path.
