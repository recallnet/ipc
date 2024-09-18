// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "../../src/lib/LibStorageStaking.sol";
import "../../src/lib/LibSubnetActorStorage.sol";
import "../../src/lib/LibStaking.sol";
import "../../src/lib/LibStakingChangeLog.sol";
import "../../src/structs/Subnet.sol"; // Import required structs like ValidatorSet, Validator, etc.

contract LibStorageStakingTest is Test {
    using LibStorageStaking for ValidatorSet;
    using LibStorageStaking for SubnetActorStorage;
    
    ValidatorInfo validator1;
    ValidatorInfo validator2;
    Validator [2]validators;

    address constant validatorAddress1 = address(0x1);
    address constant validatorAddress2 = address(0x2);
    uint256 validator1Storage = 100;
    uint256 validator2Storage = 200;
    uint256 totalValidatorsStorage = validator1Storage + validator1Storage;
    //TODO make this set up match others in project
    function setUp() public {
        // Initialize Validator structs and ValidatorSet.
        initializeValidatorsInfo(validatorAddress1, validatorAddress2);
        SubnetActorStorage storage s =  LibSubnetActorStorage.appStorage();
        //Setting manually
        s.genesisValidators.push(validators[0]);
        s.genesisValidators.push(validators[1]);
        // Set individual validator entries in the mapping
        s.validatorSet.validators[validatorAddress1] = validator1;
        s.validatorSet.validators[validatorAddress2] = validator2;
        
        // Set the totalConfirmedStorage field
        s.validatorSet.totalConfirmedStorage = totalValidatorsStorage;
    }


    function initializeValidatorsInfo(address v1, address v2) public {
        uint256 weight = 700;
        // Initialize ValidatorInfo objects with empty metadata...for now
        validator1 = ValidatorInfo({
            federatedPower: 1000,
            confirmedCollateral: weight,
            totalCollateral: 800,
            metadata: "",  // TODO
            totalStorage: validator1Storage,
            confirmedStorage: validator1Storage
        });

        validators[0] = Validator(weight,v1,"",validator1Storage);

        validator2 = ValidatorInfo({
            federatedPower: 1500,
            confirmedCollateral: weight,
            totalCollateral: 1200,
            metadata: "",  // TODO
            totalStorage: validator2Storage,
            confirmedStorage: validator2Storage
        });
        validators[1] = Validator(weight,v2,"",validator2Storage);
    }

    function testGetTotalConfirmedStorage() public {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 totalStorage = LibStorageStaking.getTotalConfirmedStorage(s.validatorSet);
        assertEq(totalStorage, totalValidatorsStorage);
    }

    function testTotalValidatorStorage() public {
        uint256 storage1 = LibStorageStaking.totalValidatorStorage(validatorAddress1);
        assertEq(storage1, validator1Storage);// set appStorage first

        uint256 storage2 = LibStorageStaking.totalValidatorStorage(validatorAddress2);
        assertEq(storage2, validator2Storage);
    }

    function testHasStorage() public {
        bool hasStorage1 = LibStorageStaking.hasStorage(validatorAddress1);
        assertTrue(hasStorage1);

        bool hasStorage2 = LibStorageStaking.hasStorage(validatorAddress2);
        assertTrue(hasStorage2);

        bool hasStorage3 = LibStorageStaking.hasStorage(address(0x3)); // Non-existent validator
        assertFalse(hasStorage3);
    }

    function testCommitStorage() public {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 newStorage = 500;
        // Before storage commit
        uint256 initialStorage = s.validatorSet.validators[validatorAddress1].totalStorage;
        assertEq(initialStorage, validator1Storage);

        // Commit storage
        LibStorageStaking.commitStorage(validatorAddress1, newStorage);

        // After storage commit
        uint256 committedStorage = s.validatorSet.validators[validatorAddress1].totalStorage;
        assertEq(committedStorage, validator1Storage + newStorage);
    }

    function testCommitStorageWithConfirm() public {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 newStorage = 1000;
        uint256 total = validator1Storage + newStorage;
        // Commit and confirm storage for validatorAddress1
        LibStorageStaking.commitStorageWithConfirm(validatorAddress1, newStorage);

        // Check total and confirmed storage
        uint256 totalStorage = s.validatorSet.validators[validatorAddress1].totalStorage;
        uint256 confirmedStorage = s.validatorSet.validators[validatorAddress1].confirmedStorage;
        assertEq(totalStorage, total);
        assertEq(confirmedStorage, total);
    }

    /// Test for recordWithdraw when the amount is less than or equal to totalStorage
    function testRecordWithdrawSuccess() public {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 amountToWithdraw = 50;
        uint256 initialStorage = s.validatorSet.validators[validatorAddress1].totalStorage;

        // Call recordWithdraw
        s.validatorSet.recordStorageWithdraw(validatorAddress1, amountToWithdraw);

        // Check if totalStorage is reduced
        uint256 updatedStorage = s.validatorSet.validators[validatorAddress1].totalStorage;
        assertEq(updatedStorage, initialStorage - amountToWithdraw);
    }

    /// Test for recordWithdraw when the amount exceeds totalStorage (should revert)
    function testRecordWithdrawExceedsStorage() public {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 amountToWithdraw = 200; // Exceeding storage

        vm.expectRevert(WithdrawExceedingStorage.selector);
        s.validatorSet.recordStorageWithdraw(validatorAddress1, amountToWithdraw);
    }

    /// Test for confirmWithdraw with normal storage reduction
    function testConfirmWithdrawSuccess() public {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 amountToWithdraw = 50;
        uint256 initialConfirmedStorage = s.validatorSet.validators[validatorAddress1].confirmedStorage;
        uint256 validatorTotal = initialConfirmedStorage - amountToWithdraw;
        uint256 total = s.validatorSet.totalConfirmedStorage - amountToWithdraw;
        
        // Call confirmWithdraw
        s.validatorSet.confirmStorageWithdraw(validatorAddress1, amountToWithdraw);

        // Check if confirmedStorage is reduced
        uint256 updatedConfirmedStorage = s.validatorSet.validators[validatorAddress1].confirmedStorage;
        assertEq(updatedConfirmedStorage, validatorTotal);
        
        // Check if totalConfirmedStorage is reduced
        assertEq(s.validatorSet.totalConfirmedStorage, total);
    }

    /// Test for confirmStorageWithdraw when newStorage == 0 and totalStorage == 0 (deletion case)
    function testConfirmWithdrawDeleteValidator() public {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 totalConfirmed = s.validatorSet.totalConfirmedStorage;

        // Withdraw full storage, so the validator should be deleted
        uint256 amountToWithdraw = s.validatorSet.validators[validatorAddress1].totalStorage;
        
        // Call withdrawStorageWithConfirm that internally calls the confirmStorageWithdraw
        LibStorageStaking.withdrawStorageWithConfirm(validatorAddress1, amountToWithdraw);

        // Check if the validator was deleted
        assertEq(s.validatorSet.validators[validatorAddress1].totalStorage, 0);
        assertEq(s.validatorSet.validators[validatorAddress1].confirmedStorage, 0);
        assertEq(s.validatorSet.totalConfirmedStorage, totalConfirmed - amountToWithdraw);
    }

   /// Test for withdrawWithConfirm (record and confirm)
    function testWithdrawWithConfirm() public {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 amountToWithdraw = 50;
        uint256 totalConfirmedStorage = s.validatorSet.validators[validatorAddress1].confirmedStorage;

        // Call withdrawWithConfirm
        LibStorageStaking.withdrawStorageWithConfirm(validatorAddress1, amountToWithdraw);

        // Check if totalStorage and confirmedStorage are reduced
        uint256 updatedTotalStorage = s.validatorSet.validators[validatorAddress1].totalStorage;
        uint256 updatedConfirmedStorage = s.validatorSet.validators[validatorAddress1].confirmedStorage;

        assertEq(updatedTotalStorage, validator1Storage - amountToWithdraw);
        assertEq(updatedConfirmedStorage, totalConfirmedStorage - amountToWithdraw);
    }

    /// Test for withdraw (record withdrawal without confirmation)
    function testWithdraw() public {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 amountToWithdraw = 50;
        uint256 totalConfirmedStorage = s.validatorSet.validators[validatorAddress1].confirmedStorage;
        
        // Call withdraw
        LibStorageStaking.withdrawStorage(validatorAddress1, amountToWithdraw);
        
        // Check if totalStorage is reduced (confirmation not yet done)
        uint256 updatedTotalStorage = s.validatorSet.validators[validatorAddress1].totalStorage;
        assertEq(updatedTotalStorage, validator1Storage - amountToWithdraw);
        // confirmedStorage should remain the same since withdraw was not confirmed
        assertEq(s.validatorSet.validators[validatorAddress1].confirmedStorage, totalConfirmedStorage);
    }
}