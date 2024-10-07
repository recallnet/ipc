// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Test} from "forge-std/Test.sol";
import "forge-std/console.sol";
import {SubnetActorDataStorageMock} from "../mocks/SubnetActorDataStorageMock.sol";
import {ValidatorInfo} from "../../contracts/structs/Subnet.sol";
import {SubnetActorStorage, LibSubnetActorStorage} from "../../contracts/lib/LibSubnetActorStorage.sol";
import {WithdrawExceedingStorage} from "../../contracts/errors/IPCErrors.sol";

contract LibDataStorageTest is Test {
    SubnetActorDataStorageMock private dataStorageFacet;
    address constant validatorAddress1 = address(0x1);
    address constant validatorAddress2 = address(0x2);
    uint256 constant validator1Storage = 100;
    uint256 constant validator2Storage = 200;
    ValidatorInfo validator1;
    ValidatorInfo validator2;

    function setUp() public {
        initializeValidatorsInfo();
        dataStorageFacet = new SubnetActorDataStorageMock(
            validatorAddress1,
            validatorAddress2,
            validator1Storage + validator2Storage,
            validator1,
            validator2
        );
    }

    function testGetTotalConfirmedStorage() public {
        uint256 totalStorage = dataStorageFacet.getTotalConfirmedStorage();
        assertEq(totalStorage, validator1Storage + validator2Storage);
    }

    function testTotalValidatorStorage() public {
        uint256 storage1 = dataStorageFacet.getTotalValidatorStorage(validatorAddress1);
        assertEq(storage1, validator1Storage); // set appStorage first

        uint256 storage2 = dataStorageFacet.getTotalValidatorStorage(validatorAddress2);
        assertEq(storage2, validator2Storage);

        uint256 storage3 = dataStorageFacet.getTotalValidatorStorage(address(0x3)); // Non validator
        assertEq(storage3, 0);
    }

    function testCommitStorage() public {
        uint256 newStorage = 500;
        // Before storage commit
        uint256 initialStorage = dataStorageFacet.getTotalValidatorStorage(validatorAddress1);
        assertEq(initialStorage, validator1Storage);

        // Commit storage
        dataStorageFacet.commitStorage(validatorAddress1, newStorage);

        // After storage commit
        uint256 committedStorage = dataStorageFacet.getTotalValidatorStorage(validatorAddress1);
        assertEq(committedStorage, validator1Storage + newStorage);
    }

    function testCommitStorageWithConfirm() public {
        uint256 newStorage = 1000;

        uint256 prevTotalStorage = dataStorageFacet.getTotalValidatorStorage(validatorAddress1);
        uint256 prevConfirmedStorage = dataStorageFacet.getTotalConfirmedStorage();

        uint256 total = prevTotalStorage + newStorage;
        // Commit and confirm storage for validatorAddress1
        dataStorageFacet.commitStorageWithConfirm(validatorAddress1, newStorage);

        // Check total and confirmed storage
        uint256 totalStorage = dataStorageFacet.getTotalValidatorStorage(validatorAddress1);
        uint256 confirmedStorage = dataStorageFacet.getTotalConfirmedStorage();

        assertEq(totalStorage, total);
        assertEq(confirmedStorage, prevConfirmedStorage + newStorage);
    }

    /// Test for recordWithdraw when the amount is less than or equal to totalStorage
    function testRecordWithdrawSuccess() public {
        uint256 amountToWithdraw = 50;
        uint256 initialStorage = dataStorageFacet.getTotalValidatorStorage(validatorAddress1);

        // Call recordWithdraw
        dataStorageFacet.recordStorageWithdraw(validatorAddress1, amountToWithdraw);

        // Check if totalStorage is reduced
        uint256 updatedStorage = dataStorageFacet.getTotalValidatorStorage(validatorAddress1);
        assertEq(updatedStorage, initialStorage - amountToWithdraw);
    }

    /// Test for recordWithdraw when the amount exceeds totalStorage (should revert)
    function testRecordWithdrawExceedsStorage() public {
        uint256 amountToWithdraw = 200; // Exceeding storage

        vm.expectRevert(WithdrawExceedingStorage.selector);
        dataStorageFacet.recordStorageWithdraw(validatorAddress1, amountToWithdraw);
    }

    /// Test for confirmWithdraw with normal storage reduction
    function testConfirmWithdrawSuccess() public {
        uint256 amountToWithdraw = 50;
        uint256 initialConfirmedStorage = dataStorageFacet.getTotalConfirmedStorage();
        uint256 validatorTotal = initialConfirmedStorage - amountToWithdraw;
        uint256 total = dataStorageFacet.getTotalConfirmedStorage() - amountToWithdraw;

        // Call confirmWithdraw
        dataStorageFacet.confirmStorageWithdraw(validatorAddress1, amountToWithdraw);

        // Check if confirmedStorage is reduced
        uint256 updatedConfirmedStorage = dataStorageFacet.getTotalConfirmedStorage();
        assertEq(updatedConfirmedStorage, validatorTotal);

        // Check if totalConfirmedStorage is reduced
        assertEq(dataStorageFacet.getTotalConfirmedStorage(), total);
    }

    /// Test for confirmStorageWithdraw when newStorage == 0 and totalStorage == 0 (deletion case)
    function testConfirmWithdrawDeleteValidator() public {
        uint256 totalConfirmed = dataStorageFacet.getTotalConfirmedStorage();

        // Withdraw full storage, so the validator should be deleted
        uint256 amountToWithdraw = dataStorageFacet.getTotalValidatorStorage(validatorAddress1);

        // Call withdrawStorageWithConfirm that internally calls the confirmStorageWithdraw
        dataStorageFacet.withdrawStorageWithConfirm(validatorAddress1, amountToWithdraw);

        // Check if the validator was deleted
        assertEq(dataStorageFacet.getTotalValidatorStorage(validatorAddress1), 0);
        assertEq(dataStorageFacet.getTotalValidatorConfirmedStorage(validatorAddress1), 0);
        assertEq(dataStorageFacet.getTotalConfirmedStorage(), totalConfirmed - amountToWithdraw);
    }

    /// Test for withdrawWithConfirm (record and confirm)
    function testWithdrawWithConfirm() public {
        uint256 amountToWithdraw = 50;
        uint256 totalConfirmedStorage = dataStorageFacet.getTotalConfirmedStorage();

        // Call withdrawWithConfirm
        dataStorageFacet.withdrawStorageWithConfirm(validatorAddress1, amountToWithdraw);

        // Check if totalStorage and confirmedStorage are reduced
        uint256 updatedTotalStorage = dataStorageFacet.getTotalValidatorStorage(validatorAddress1);
        uint256 updatedConfirmedStorage = dataStorageFacet.getTotalConfirmedStorage();

        assertEq(updatedTotalStorage, validator1Storage - amountToWithdraw);
        assertEq(updatedConfirmedStorage, totalConfirmedStorage - amountToWithdraw);
    }

    /// Test for withdraw (record withdrawal without confirmation)
    function testWithdraw() public {
        uint256 amountToWithdraw = 50;
        uint256 totalConfirmedStorage = dataStorageFacet.getTotalValidatorConfirmedStorage(validatorAddress1);

        // Call withdraw
        dataStorageFacet.withdrawStorage(validatorAddress1, amountToWithdraw);

        // Check if totalStorage is reduced (confirmation not yet done)
        uint256 updatedTotalStorage = dataStorageFacet.getTotalValidatorStorage(validatorAddress1);

        assertEq(updatedTotalStorage, validator1Storage - amountToWithdraw);
        // confirmedStorage should remain the same since withdraw was not confirmed
        assertEq(dataStorageFacet.getTotalValidatorConfirmedStorage(validatorAddress1), totalConfirmedStorage);
    }

    function initializeValidatorsInfo() public {
        uint256 weight = 700;
        // Initialize ValidatorInfo objects with empty metadata...for now
        validator1 = ValidatorInfo({
            federatedPower: 1000,
            confirmedCollateral: weight,
            totalCollateral: 800,
            metadata: "",
            totalStorageAmount: validator1Storage,
            confirmedStorageAmount: validator1Storage
        });

        validator2 = ValidatorInfo({
            federatedPower: 1500,
            confirmedCollateral: weight,
            totalCollateral: 1200,
            metadata: "",
            totalStorageAmount: validator2Storage,
            confirmedStorageAmount: validator2Storage
        });
    }
}
