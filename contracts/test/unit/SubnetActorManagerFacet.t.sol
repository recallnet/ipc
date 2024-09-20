// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "../../contracts/lib/LibStaking.sol";
import {TestUtils} from "../helpers/TestUtils.sol";
import {SubnetActorManagerFacetMock} from "../mocks/SubnetActorManagerFacetMock.sol";

contract SubnetActorManagerFacetTest is Test {

    SubnetActorManagerFacetMock internal subnetActorManagerFacet;
    uint256 constant storageCommintment = 10;
    address walletAddr = vm.addr(1);
    bytes uncompressedKey;
    bytes metadata;

    function setUp() public {
        uncompressedKey = abi.encodePacked(bytes1(0x04), TestUtils.derivePubKeyBytes(1));
        metadata = abi.encodePacked(TestUtils.addStorageToPK(uncompressedKey));
        
        subnetActorManagerFacet = new SubnetActorManagerFacetMock();
        subnetActorManagerFacet.setActiveLimit(10);
        subnetActorManagerFacet.setMinCollateral(100);
        subnetActorManagerFacet.setStorageCollateralRatio(1);
        vm.deal(walletAddr, 10 ether);
    }

    function testSetStorageOnJoin() public {
        vm.startPrank(walletAddr);
        vm.expectRevert();//Not enough collateral per storage
        subnetActorManagerFacet.join{value: 1}(metadata); 
        
        // Expect no revert on this call
        subnetActorManagerFacet.join{value: 10}(metadata); // Call join function with valid collateral and metadata
        vm.stopPrank();
        // Check that the validator has joined
        assertTrue(subnetActorManagerFacet.isValidator(walletAddr), "Validator did not join successfully");
        assertTrue(subnetActorManagerFacet.hasStorage(walletAddr));
        
        assertEq(subnetActorManagerFacet.getTotalStorage(walletAddr), storageCommintment);
        assertEq(subnetActorManagerFacet.getTotalConfirmedStorage(walletAddr), storageCommintment);
    }

    function testSetStorageOnStakeStorage() public {
        (uint256 validatorTotalStorage, uint256 validatorConfirmedStorage, uint256 totalConfirmedStorage ) = getStorageValues();

        // Must revert if validator have not joined the subnet
        vm.expectRevert();
        subnetActorManagerFacet.stakeStorage(storageCommintment);


        vm.startPrank(walletAddr);
        subnetActorManagerFacet.join{value: 10}(metadata); // Call join before staking
        vm.expectRevert();
        subnetActorManagerFacet.stakeStorage{value: 1}(storageCommintment);// Not enough collateral

        subnetActorManagerFacet.stakeStorage{value: 10}(storageCommintment);
        vm.stopPrank();
        
        assertGt(subnetActorManagerFacet.getTotalStorage(walletAddr), validatorTotalStorage);
        assertGt(subnetActorManagerFacet.getTotalConfirmedStorage(walletAddr), validatorConfirmedStorage);
        assertGt(subnetActorManagerFacet.getSubnetTotalConfirmedStorage(), totalConfirmedStorage);
    }

    function testSetStorageOnLeave() public {
        // Should not allow to leave if address never joined
        vm.expectRevert();
        subnetActorManagerFacet.leave();

        vm.startPrank(walletAddr);
        subnetActorManagerFacet.join{value: 10}(metadata); // Call join before leaving
        // Save current storage state
        (, , uint256 totalConfirmedStorage) = getStorageValues();
        subnetActorManagerFacet.leave();
        vm.stopPrank();

        (uint256 newValidatorTotalStorage, uint256 newValidatorConfirmedStorage, uint256 newTotalConfirmedStorage ) = getStorageValues();
        console.log(totalConfirmedStorage,newValidatorTotalStorage, newValidatorConfirmedStorage, newTotalConfirmedStorage);
        assertEq(newValidatorTotalStorage, 0);
        assertEq(newValidatorConfirmedStorage, 0);
        assertEq(newTotalConfirmedStorage, 0);
    }

    function testSetStorageOnUnstakeStorage() public {
        vm.startPrank(walletAddr);

        subnetActorManagerFacet.join{value: 10}(metadata); // Call join before unstaking
        subnetActorManagerFacet.stakeStorage{value: 10}(storageCommintment);

        (uint256 validatorTotalStorage, , uint256 totalConfirmedStorage ) = getStorageValues();
        uint256 amount = storageCommintment;

        vm.expectRevert();
        subnetActorManagerFacet.unstakeStorage(0);// Cannot unstake 0

        vm.expectRevert();
        subnetActorManagerFacet.unstakeStorage(validatorTotalStorage + 1);// Cannot exceed total storage

        subnetActorManagerFacet.unstakeStorage(amount);
        vm.stopPrank();

        (uint256 newValidatorTotalStorage, uint256 newValidatorConfirmedStorage, uint256 newTotalConfirmedStorage ) = getStorageValues();

        assertEq(newValidatorTotalStorage, validatorTotalStorage - amount);
        assertEq(newValidatorConfirmedStorage, validatorTotalStorage - amount);
        assertEq(newTotalConfirmedStorage, totalConfirmedStorage - amount);
    }

    function testEnforceStorageCollateralOnUnstake() public {
        vm.startPrank(walletAddr);

        subnetActorManagerFacet.join{value: 10}(metadata); // Call join before unstaking

        (, uint256 totalConfirmedStorage,) = getStorageValues();

        vm.expectRevert();
        subnetActorManagerFacet.unstake(totalConfirmedStorage - 1);

        vm.stopPrank();
    }

    function getStorageValues() internal view returns(uint256, uint256, uint256) {
        uint256 validatorTotalStorage = subnetActorManagerFacet.getTotalStorage(walletAddr);
        uint256 validatorConfirmedStorage = subnetActorManagerFacet.getTotalConfirmedStorage(walletAddr);
        uint256 totalConfirmedStorage = subnetActorManagerFacet.getSubnetTotalConfirmedStorage();

        return (validatorTotalStorage, validatorConfirmedStorage, totalConfirmedStorage);
    }

}