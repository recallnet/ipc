// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetActorManagerFacet, LibStaking, LibStorageStaking} from "../../contracts/subnet/SubnetActorManagerFacet.sol";

contract SubnetActorManagerFacetMock is SubnetActorManagerFacet {
    function setActiveLimit(uint16 limit) external  {
        s.validatorSet.activeLimit = limit;
    }

    function setMinCollateral(uint16 amount) external  {
        s.minActivationCollateral = amount;
    }

    function setStorageCollateralRatio(uint16 amount) external  {
        s.tokensPerStorageRatio = amount;
    }

    function isValidator(address validator) external view returns (bool) {
        return LibStaking.isValidator(validator);
    }

    function hasStorage(address validator) external view returns (bool) {
        return LibStorageStaking.hasStorage(validator);
    }

    function getTotalStorage(address validator) external  view returns (uint256) {
        return LibStorageStaking.totalValidatorStorage(validator);
    }

    function getTotalConfirmedStorage(address validator) external  view returns (uint256) {
        return s.validatorSet.validators[validator].confirmedStorage;
    }

    function getSubnetTotalConfirmedStorage() external  view returns (uint256) {
        return LibStorageStaking.getTotalConfirmedStorage();
    }
}
