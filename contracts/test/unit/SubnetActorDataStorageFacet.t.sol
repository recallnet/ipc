// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Test} from "forge-std/Test.sol";
import {SubnetActorDataStorageMock} from "../mocks/SubnetActorDataStorageMock.sol";
import {ValidatorInfo} from "../../contracts/structs/Subnet.sol";
import {SubnetActorStorage, LibSubnetActorStorage} from "../../contracts/lib/LibSubnetActorStorage.sol";

contract SubnetActorDataStorageFacetTest is Test {
    SubnetActorDataStorageMock private dataStorageFacet;
    address constant validatorAddress1 = address(0x1);
    address constant validatorAddress2 = address(0x2);
    uint256 constant validator1Storage = 100;
    uint256 constant validator2Storage = 200;
    ValidatorInfo validator1;
    ValidatorInfo validator2;

   /* function setUp() public {
        dataStorageFacet = new SubnetActorDataStorageFacet();
        // Initialize Validator structs and ValidatorSet.
        initializeValidatorsInfo();
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // Set individual validator entries in the mapping
        s.validatorSet.validators[validatorAddress1] = validator1;
        s.validatorSet.validators[validatorAddress2] = validator2;

        // Set the totalConfirmedStorage field
        s.validatorSet.totalConfirmedStorage = validator1Storage + validator2Storage;
    }

    function testGetTotalConfirmedStorage() public {
        uint256 totalStorage = dataStorageFacet.getTotalConfirmedStorage();
        assertEq(totalStorage, totalValidatorsStorage);
    }

    function testTotalValidatorStorage() public {
        uint256 storage1 = dataStorageFacet.getTotalValidatorStorage(validatorAddress1);
        assertEq(storage1, validator1Storage); // set appStorage first

        uint256 storage2 = dataStorageFacet.getTotalValidatorStorage(validatorAddress2);
        assertEq(storage2, validator2Storage);
    }

    function testHasStorage() public {
        bool hasStorage1 = dataStorageFacet.hasStorage(validatorAddress1);
        assertTrue(hasStorage1);

        bool hasStorage2 = dataStorageFacet.hasStorage(validatorAddress2);
        assertTrue(hasStorage2);

        bool hasStorage3 = dataStorageFacet.hasStorage(address(0x3)); // Non-existent validator
        assertFalse(hasStorage3);
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
*/
}
