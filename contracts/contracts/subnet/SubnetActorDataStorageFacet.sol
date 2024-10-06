// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {Pausable} from "../lib/LibPausable.sol";

contract SubnetActorDataStorageFacet is SubnetActorModifiers, ReentrancyGuard, Pausable {

    function processJoin() external {}
    function processUnstake() external {}
    function processStorageStake() external {}
    function processStorageUnStake() external {}

}