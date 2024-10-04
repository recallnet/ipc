// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {ValidatorSet, Validator, StakingChangeLog} from "../structs/Subnet.sol";
import {LibSubnetActorStorage, SubnetActorStorage} from "./LibSubnetActorStorage.sol";
import {LibSubnetActor} from "./LibSubnetActor.sol";
import {LibStakingChangeLog} from "./LibStakingChangeLog.sol";
import {LibValidatorSet, LibStaking} from "./LibStaking.sol";
import {NotEnoughStorageCommitment} from "../errors/IPCErrors.sol";

library LibStorageStaking {
    using LibStakingChangeLog for StakingChangeLog;
    using LibValidatorSet for ValidatorSet;

    /// @notice Commit the storage. 
    function commitStorage(address validator, uint256 totalStorage) external {
        if (totalStorage == 0) {
            revert NotEnoughStorageCommitment();
        }
        
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        s.changeSet.commitStorageRequest(validator, totalStorage);
        s.validatorSet.recordStorageDeposit(validator, totalStorage);
    }

    /// @notice Confirm the deposit directly without going through the confirmation process
    function commitStorageWithConfirm(address validator, uint256 totalStorage) external {
        if (totalStorage == 0) {
            revert NotEnoughStorageCommitment();
        }
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // record deposit that updates the total commited storage
        s.validatorSet.recordStorageDeposit(validator, totalStorage);
        // confirm deposit that updates the confirmed commited storage
        s.validatorSet.confirmStorageDeposit(validator, totalStorage);

        // Because depositWithConfirm runs before this we know the address is already a validator we just need to include the storage in the struct
        //TODO It's not the most gas-efficient operation to do this twice; evaluate the possibility of doing it only once.
        uint256 storageAmount = s.validatorSet.validators[validator].confirmedStorage;
        s.genesisValidators[s.genesisValidators.length - 1].storageAmount = storageAmount;
    }

    /// @notice Confirm the storage withdraw directly without going through the confirmation process
    /// and releasing from the gateway.
    /// @dev only use for non-bootstrapped subnets
    function withdrawStorageWithConfirm(address validator, uint256 amount) external {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // record deposit that updates the total storage
        s.validatorSet.recordStorageWithdraw(validator, amount);
        // confirm deposit that updates the confirmed storage
        s.validatorSet.confirmStorageWithdraw(validator, amount);

    }

    /// @notice Withdraw the storage
    function withdrawStorage(address validator, uint256 amount) external {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        s.changeSet.withdrawStorageRequest(validator, amount);
        s.validatorSet.recordStorageWithdraw(validator, amount);
    }

}

library LibStorageStakingOps {
    function stakeStorage(uint256 amount, uint256 value, bool bootstrapped) external {
        LibSubnetActor.enforceCollateralValidation();

        uint256 collateral = value + LibStaking.totalValidatorCollateral(msg.sender);
        uint256 totalStorage = amount + LibStorageStakingGetters.totalValidatorStorage(msg.sender);
        LibSubnetActor.enforceStorageCollateralValidation(msg.value + collateral, totalStorage + amount);

        if (!bootstrapped) {
            LibStorageStaking.commitStorageWithConfirm(msg.sender, amount);
        } else {
            LibStorageStaking.commitStorage(msg.sender, amount);
        }   
    }

    // No unstakeStorage included since solidity compilation exceeds size limit
}

library LibStorageStakingGetters {
    /// @notice Getter for total storage committed by all validators in a subnet.
    function getTotalConfirmedStorage() external view returns (uint256 totalStorage) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        totalStorage = s.validatorSet.totalConfirmedStorage;
    }

    /// @notice Gets the total storage committed by the validator.
    /// @param validator The address to check for storage amount.
    function totalValidatorStorage(address validator) external view returns (uint256) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.validators[validator].totalStorage;
    }

    /// @notice Checks if the validator has committed storage before.
    /// @param validator The address to check for storage status.
    /// @return A boolean indicating whether the validator has committed storage.
    function hasStorage(address validator) external view returns (bool) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.validators[validator].totalStorage != 0;
    }
}