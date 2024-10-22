# Contract Deployments

Contract deployment involves a lot of setup and inputs as well as generating a lot of artifacts.

This document captures the steps to get the addresses of the deployed contracts, version of code and tools used for deployment in a docker container.



## Filecoin Calibration Testnet

### Ignition

docker_image=3boxben/ipc:ignition-contracts

List ipc contracts out artifacts:
```
docker run --rm  ${docker_image} ls -l ./contracts/out
```

Get the PARENT_GATEWAY_ADDRESS:
```
docker run --rm  ${docker_image} jq -r .address contracts/deployments/calibrationnet/GatewayDiamond.json
```

Get the PARENT_REGISTRY_ADDRESS:
```
docker run --rm  ${docker_image} jq -r .address contracts/deployments/calibrationnet/SubnetRegistryDiamond.json
```

Get the SUPPLY_SOURCE_ADDRESS:
```
docker run --rm  ${docker_image} jq -r '.transactions | .[] | select(.contractName == "ERC1967Proxy") | .contractAddress' ./hoku-contracts/broadcast/Hoku.s.sol/314159/run-latest.json
```

Get the VALIDATOR_GATER_ADDRESS:
```
docker run --rm  ${docker_image} jq -r '.transactions | .[] | select(.contractName == "ValidatorGater") | .contractAddress'  ./hoku-contracts/broadcast/ValidatorGater.s.sol/314159/run-latest.json
```

PARENT_GATEWAY_ADDRESS=0xaDfc2428D57c58c322CFFa7307b7e63402accd17
PARENT_REGISTRY_ADDRESS=0xDe4C2e132011CC3279484dFD1d2557c84517277a
SUPPLY_SOURCE_ADDRESS=0x9e947917d1812c2ab813704315f739f1d9f858a9
VALIDATOR_GATER_ADDRESS=0x907fa3b65907563d71eb9a7b4ac9321aaf53b67e
