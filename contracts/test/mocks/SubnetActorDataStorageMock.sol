// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {LibDataStorage} from "../../contracts/subnet/LibDataStorage.sol";
import {SubnetActorModifiers} from "../../contracts/lib/LibSubnetActorStorage.sol";
import {ValidatorInfo, ValidatorSet} from "../../contracts/structs/Subnet.sol";
import {LibValidatorSet} from "../../contracts/lib/LibStaking.sol";

contract SubnetActorDataStorageMock is SubnetActorModifiers {
    using LibValidatorSet for ValidatorSet;

    constructor(
        address v1,
        address v2,
        uint256 totalConfirmedStorage,
        ValidatorInfo memory validator1,
        ValidatorInfo memory validator2
    ) {
        // Set individual validator entries in the mapping
        s.validatorSet.validators[v1] = validator1;
        s.validatorSet.validators[v2] = validator2;

        // Set the totalConfirmedStorage field
        s.validatorSet.totalConfirmedStorage = totalConfirmedStorage;
    }

    function getTotalConfirmedStorage() external view returns (uint256) {
        return LibDataStorage.getTotalConfirmedStorage();
    }

    function getTotalValidatorStorage(address validator) external view returns (uint256) {
        return LibDataStorage.getTotalValidatorStorage(validator);
    }

    function getTotalValidatorConfirmedStorage(address validator) external view returns (uint256) {
        return LibDataStorage.getTotalValidatorConfirmedStorage(validator);
    }

    function commitStorage(address validator, uint256 newStorage) external {
        LibDataStorage.commitStorage(validator, newStorage);
    }

    function commitStorageWithConfirm(address validator, uint256 newStorage) external {
        LibDataStorage.commitStorageWithConfirm(validator, newStorage);
    }

    function recordStorageWithdraw(address validatorAddress1, uint256 amountToWithdraw) external {
        s.validatorSet.recordStorageWithdraw(validatorAddress1, amountToWithdraw);
    }

    function withdrawStorage(address validator, uint256 amount) external {
        LibDataStorage.withdrawStorage(validator, amount);
    }

    function confirmStorageWithdraw(address validatorAddress1, uint256 amountToWithdraw) external {
        s.validatorSet.confirmStorageWithdraw(validatorAddress1, amountToWithdraw);
    }

    function withdrawStorageWithConfirm(address validator, uint256 amount) external {
        LibDataStorage.withdrawStorageWithConfirm(validator, amount);
    }
}
