// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {ReentrancyGuard} from "./LibReentrancyGuard.sol";
import {NotEnoughCollateralForStorageAmount, CannotReleaseZero, NotValidator, NotEnoughStorageCommitment, InsufficientStorage} from "../errors/IPCErrors.sol";
import {LibSubnetActor} from "./LibSubnetActor.sol";
import {LibValidatorSet, LibStaking} from "./LibStaking.sol";
import {ValidatorSet, Asset, StakingChangeLog} from "../structs/Subnet.sol";
import {AssetHelper} from "./AssetHelper.sol";
import {LibStakingChangeLog} from "./LibStakingChangeLog.sol";
import {SubnetActorStorage, LibSubnetActorStorage} from "./LibSubnetActorStorage.sol";

library LibDataStorage {
    using LibValidatorSet for ValidatorSet;
    using AssetHelper for Asset;
    using LibStakingChangeLog for StakingChangeLog;

    // =============== Getters =============

    /// @notice Gets the total storage committed by the validator.
    /// @param validator The address to check for storage amount.
    function totalValidatorStorage(address validator) public view returns (uint256) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.validators[validator].totalStorageAmount;
    }

    function getTotalValidatorConfirmedStorage(address validator) external view returns (uint256) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        return s.validatorSet.validators[validator].confirmedStorageAmount;
    }

    /// @notice Getter for total storage committed by all validators in a subnet.
    function getTotalConfirmedStorage() external view returns (uint256 totalStorage) {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        totalStorage = s.validatorSet.totalConfirmedStorage;
    }

    // =============== Operations =============

    function processJoin(uint256 storageCommitment, bool bootstrapped) external {
        processCommitStorage(msg.sender, storageCommitment, bootstrapped);
    }

    /// @notice Enforces that remaining collateral is enough for the storage commited
    /// @dev Reverts if the collateral is not in enough for the storage amount
    /// @param newCollateral The new validator's collateral
    function validateUnstake(uint256 newCollateral) external view {
        uint256 totalStorage = totalValidatorStorage(msg.sender);

        enforceStorageCollateralValidation(newCollateral, totalStorage);
    }

    function processStorageStake(uint256 storageAmount, uint256 stakeAmount, Asset storage collateralSource) external {
        if (stakeAmount > 0) collateralSource.lock(stakeAmount);
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        stakeStorage(storageAmount, stakeAmount, s.bootstrapped);
    }

    function processStorageUnStake(uint256 storageAmount, bool includeCollateral) external {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        // disabling validator changes for federated validation subnets
        LibSubnetActor.enforceCollateralValidation();

        if (storageAmount == 0) {
            revert CannotReleaseZero();
        }

        uint256 totalStorage = totalValidatorStorage(msg.sender);

        hasEnoughStorage(totalStorage);
        if (totalStorage <= storageAmount) {
            revert InsufficientStorage();
        }

        if (!s.bootstrapped) {
            withdrawStorageWithConfirm(msg.sender, storageAmount, s.validatorSet);
        } else {
            withdrawStorage(msg.sender, storageAmount, s.changeSet, s.validatorSet);
        }

        if (includeCollateral) {
            //uint256 collateral = storageAmount * s.tokenStorageRatio;
            //processUnstake(msg.sender, collateral, bootstrapped, s.collateralSource); TODO make normal unstake process
        }
    }

    /// @notice Confirm the deposit directly without going through the confirmation process
    function commitStorageWithConfirm(address validator, uint256 totalStorage) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        hasEnoughStorage(totalStorage);

        // record deposit that updates the total commited storage
        s.validatorSet.recordStorageDeposit(validator, totalStorage);
        // confirm deposit that updates the confirmed commited storage
        s.validatorSet.confirmStorageDeposit(validator, totalStorage);
    }

    /// @notice Commit the storage.
    function commitStorage(address validator, uint256 totalStorage) internal {
        hasEnoughStorage(totalStorage);

        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        s.changeSet.commitStorageRequest(validator, totalStorage);
        s.validatorSet.recordStorageDeposit(validator, totalStorage);
    }

    function stakeStorage(uint256 storageAmount, uint256 stakeAmount, bool bootstrapped) internal {
        LibSubnetActor.enforceCollateralValidation();
        uint256 collateral = stakeAmount + LibStaking.totalValidatorCollateral(msg.sender);
        uint256 totalStorage = storageAmount + totalValidatorStorage(msg.sender);
        enforceStorageCollateralValidation(stakeAmount + collateral, totalStorage + storageAmount);

        if (stakeAmount > 0) {
            if (!bootstrapped) {
                LibStaking.depositWithConfirm(msg.sender, stakeAmount);
                LibSubnetActor.bootstrapSubnetIfNeeded();
            } else {
                LibStaking.deposit(msg.sender, stakeAmount);
            }
        }

        processCommitStorage(msg.sender, storageAmount, bootstrapped);
    }

    /// @notice Confirm the storage withdraw directly without going through the confirmation process
    /// and releasing from the gateway.
    /// @dev only use for non-bootstrapped subnets
    function withdrawStorageWithConfirm(address validator, uint256 amount, ValidatorSet storage validatorSet) internal {
        // record deposit that updates the total storage
        validatorSet.recordStorageWithdraw(validator, amount);
        // confirm deposit that updates the confirmed storage
        validatorSet.confirmStorageWithdraw(validator, amount);
    }

    /// @notice Withdraw the storage
    function withdrawStorage(address validator, uint256 amount, StakingChangeLog storage changeSet, ValidatorSet storage validatorSet) internal {
        changeSet.withdrawStorageRequest(validator, amount);
        validatorSet.recordStorageWithdraw(validator, amount);
    }

    // ====== Helpers ========

    /// @notice Retuns true if validator has storage
    /// @dev reverts if storage equals 0
    function hasEnoughStorage(uint256 storageAmount) private pure {
        if (storageAmount == 0) {
            revert NotEnoughStorageCommitment();
        }
    }

    /// @notice Ensures that the provided collateral is enough for the committed storage.
    /// @dev Reverts if the collateral is not in enough for the storage amount
    function enforceStorageCollateralValidation(uint256 collateral, uint256 storageAmount) private view {
        hasEnoughStorage(storageAmount);
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 requiredCollateral = storageAmount * s.tokenStorageRatio;

        if (collateral < requiredCollateral) {
            revert NotEnoughCollateralForStorageAmount();
        }
        return;
    }

    function processCommitStorage(address sender, uint256 storageCommitment, bool bootstrapped) private {
        if (!bootstrapped) {
            commitStorageWithConfirm(sender, storageCommitment);
        } else {
            commitStorage(sender, storageCommitment);
        }
    }
}
