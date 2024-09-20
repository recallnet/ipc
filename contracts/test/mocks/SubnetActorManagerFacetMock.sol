// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {LibSubnetActorStorage, SubnetActorStorage} from "../../src/lib/LibSubnetActorStorage.sol";
import {SubnetActorManagerFacet, LibStaking, LibStorageStaking, ValidatorSet} from "../../src/subnet/SubnetActorManagerFacet.sol";

contract SubnetActorManagerFacetMock is SubnetActorManagerFacet {
    function setActiveLimit(uint16 limit) external {
        s.validatorSet.activeLimit = limit;
    }

    function setMinCollateral(uint16 amount) external {
        s.minActivationCollateral = amount;
    }

    function setStorageCollateralRatio(uint16 amount) external {
        s.tokensPerStorageRatio = amount;
    }

    function isValidator(address validator) public view returns (bool) {
        return LibStaking.isValidator(validator);
    }

    function hasStorage(address validator) public view returns (bool) {
        return LibStorageStaking.hasStorage(validator);
    }

    function getTotalStorage(address validator) public view returns (uint256) {
        return LibStorageStaking.totalValidatorStorage(validator);
    }

    function getTotalConfirmedStorage(address validator) public view returns (uint256) {
        return s.validatorSet.validators[validator].confirmedStorage;
    }

    function getSubnetTotalConfirmedStorage() public view returns (uint256) {
        return LibStorageStaking.getTotalConfirmedStorage();
    }
}
