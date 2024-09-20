// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {ValidatorSet, Validator, StakingChangeLog} from "../structs/Subnet.sol";
import {LibSubnetActorStorage, SubnetActorStorage} from "./LibSubnetActorStorage.sol";
import {LibStakingChangeLog} from "./LibStakingChangeLog.sol";
import {LibValidatorSet} from "./LibStaking.sol";
import {WithdrawExceedingStorage} from "../errors/IPCErrors.sol";

library LibStorageStaking {
    using LibStakingChangeLog for StakingChangeLog;
    using LibValidatorSet for ValidatorSet;

    // =============== Getters =============

    /// @notice Getter for total storage committed by all validators in a subnet.
    function getTotalConfirmedStorage() internal view returns(uint256 totalStorage) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        totalStorage = s.validatorSet.totalConfirmedStorage;
    }

    /// @notice Gets the total storage committed by the validator.
    /// @param validator The address to check for storage amount.
    function totalValidatorStorage(address validator) internal view returns (uint256) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.validators[validator].totalStorage;
    }

    /// @notice Checks if the validator has committed storage before.
    /// @param validator The address to check for storage status.
    /// @return A boolean indicating whether the validator has committed storage.
    function hasStorage(address validator) internal view returns (bool) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        return s.validatorSet.validators[validator].totalStorage != 0;
    }

    /// @notice Commit the storage. 
    function commitStorage(address validator, uint256 totalStorage) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        s.changeSet.commitStorageRequest(validator, totalStorage);
        s.validatorSet.recordStorageDeposit(validator, totalStorage);
        require(validator != address(0) && totalStorage > 0,"Function not implemented yet");
    }

    /// @notice Confirm the deposit directly without going through the confirmation process
    function commitStorageWithConfirm(address validator, uint256 totalStorage) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // record deposit that updates the total commited storage
        s.validatorSet.recordStorageDeposit(validator, totalStorage);
        // confirm deposit that updates the confirmed commited storage
        s.validatorSet.confirmStorageDeposit(validator, totalStorage);

        // Because depositWithConfirm runs before this we know the address is already a validator we just need to include the storage in the struct
        if (!s.bootstrapped) { //TODO It's not the most gas-efficient operation to do this twice; evaluate the possibility of doing it only once.
            uint256 storageAmount = s.validatorSet.validators[validator].confirmedStorage;
            s.genesisValidators[s.genesisValidators.length - 1].storageAmount = storageAmount;
        }
    }

    /// @notice Validator reduces its total storage committed by amount.
    function recordStorageWithdraw(ValidatorSet storage validators, address validator, uint256 amount) internal {
        uint256 total = validators.validators[validator].totalStorage;
        if (total < amount) {
            revert WithdrawExceedingStorage();
        }

        validators.validators[validator].totalStorage = total - amount;
    }

    function confirmStorageWithdraw(ValidatorSet storage self, address validator, uint256 amount) internal {
        self.totalConfirmedStorage -= amount;
        uint256 confirmedStorage = self.validators[validator].confirmedStorage;
        uint256 totalStorage = self.validators[validator].totalStorage;
        // This call might happen after a call to LibStaking.withdrawWithConfirm deleting the validator
        if (confirmedStorage == 0 && totalStorage == 0 ) {
            return;
        }
        uint256 newStorage = confirmedStorage - amount;
        
        if (newStorage == 0 && totalStorage == 0) {
            delete self.validators[validator];
        } else {
            self.validators[validator].confirmedStorage = newStorage;
        }
    }

    /// @notice Confirm the storage withdraw directly without going through the confirmation process
    /// and releasing from the gateway.
    /// @dev only use for non-bootstrapped subnets
    function withdrawStorageWithConfirm(address validator, uint256 amount) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // record deposit that updates the total storage
        recordStorageWithdraw(s.validatorSet, validator, amount);
        // confirm deposit that updates the confirmed storage
        confirmStorageWithdraw(s.validatorSet, validator, amount);

    }

    /// @notice Withdraw the storage
    function withdrawStorage(address validator, uint256 amount) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        s.changeSet.withdrawStorageRequest(validator, amount);
        recordStorageWithdraw(s.validatorSet, validator, amount);
    }

}