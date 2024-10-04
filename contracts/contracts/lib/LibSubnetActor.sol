// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {VALIDATOR_SECP256K1_PUBLIC_KEY_LENGTH} from "../constants/Constants.sol";
import {ERR_PERMISSIONED_AND_BOOTSTRAPPED, ERR_VALIDATOR_JOINED} from "../errors/IPCErrors.sol";
import {NotEnoughGenesisValidators, DuplicatedGenesisValidator, NotOwnerOfPublicKey, MethodNotAllowed, NotEnoughCollateralForStorageAmount, CollateralIsZero, InvalidPublicKeyLength, NotValidator} from "../errors/IPCErrors.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {IValidatorGater} from "../interfaces/IValidatorGater.sol";
import {Validator, ValidatorSet, PermissionMode, SubnetID, Asset} from "../structs/Subnet.sol";
import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {LibStaking} from "../lib/LibStaking.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {LibSubnetActorStorage, SubnetActorStorage} from "./LibSubnetActorStorage.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {AssetHelper} from "../lib/AssetHelper.sol";

library LibSubnetActor {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using AssetHelper for Asset;

    event SubnetBootstrapped(Validator[]);

    /// @notice Ensures that the subnet is operating under Collateral-based permission mode.
    /// @dev Reverts if the subnet is not in Collateral mode.
    function enforceCollateralValidation() internal view {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        if (s.validatorSet.permissionMode != PermissionMode.Collateral) {
            revert MethodNotAllowed(ERR_PERMISSIONED_AND_BOOTSTRAPPED);
        }
        return;
    }

    /// @notice Ensures that the provided collateral is enough for the committed storage.
    /// @dev Reverts if the collateral is not in enough for the storage amount
    function enforceStorageCollateralValidation(uint256 collateral, uint256 storageAmount) internal view {
        if (!LibStaking.isValidator(msg.sender)) {
            revert NotValidator(msg.sender);
        }
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        uint256 requiredCollateral = storageAmount * s.tokensPerStorageRatio;
        
        if (storageAmount > 0 && collateral < requiredCollateral) {
            revert NotEnoughCollateralForStorageAmount();
        }
        return;
    }

    /// @notice Sets the token per storage unit amount.
    function setStorageCollateral(uint256 amount) internal {    
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        s.tokensPerStorageRatio = amount;
    }

    /// @notice Ensures that the subnet is operating under Federated permission mode.
    /// @dev Reverts if the subnet is not in Federated mode.
    function enforceFederatedValidation() internal view {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        if (s.validatorSet.permissionMode != PermissionMode.Federated) {
            revert MethodNotAllowed(ERR_PERMISSIONED_AND_BOOTSTRAPPED);
        }
        return;
    }

    /// @notice Performs validator gating, i.e. checks if the validator power update is actually allowed.
    function gateValidatorPowerDelta(address validator, uint256 oldPower, uint256 newPower) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // zero address means no gating needed
        if (s.validatorGater == address(0)) {
            return;
        }

        SubnetID memory id = s.parentId.createSubnetId(address(this));
        IValidatorGater(s.validatorGater).interceptPowerDelta(id, validator, oldPower, newPower);
    }

    /// @notice Performs validator gating, i.e. checks if the validator power update is actually allowed.
    function gateValidatorNewPowers(address[] calldata validators, uint256[] calldata newPowers) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        // zero address means no gating needed
        if (s.validatorGater == address(0)) {
            return;
        }

        SubnetID memory subnet = s.parentId.createSubnetId(address(this));
        IValidatorGater gater = IValidatorGater(s.validatorGater);
        uint256 length = validators.length;

        for (uint256 i; i < length; ) {
            uint256 oldPower = LibStaking.getPower(validators[i]);
            gater.interceptPowerDelta(subnet, validators[i], oldPower, newPowers[i]);

            unchecked {
                i++;
            }
        }
    }

    /// @dev This function is used to bootstrap the subnet,
    ///     if its total collateral is greater than minimum activation collateral.
    function bootstrapSubnetIfNeeded() internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        uint256 totalCollateral = LibStaking.getTotalConfirmedCollateral();

        if (totalCollateral >= s.minActivationCollateral) {
            if (LibStaking.totalActiveValidators() >= s.minValidators) {
                s.bootstrapped = true;
                emit SubnetBootstrapped(s.genesisValidators);

                // register adding the genesis circulating supply (if it exists)
                registerInGateway(totalCollateral);
            }
        }
    }

    /// @notice Converts a 65-byte public key to its corresponding address.
    /// @param publicKey The 65-byte public key to be converted.
    /// @return The address derived from the given public key.
    function publicKeyToAddress(bytes calldata publicKey) internal pure returns (address) {
        assert(publicKey.length == VALIDATOR_SECP256K1_PUBLIC_KEY_LENGTH);
        bytes32 hashed = keccak256(publicKey[1:]);
        return address(uint160(uint256(hashed)));
    }

    /// @notice method that allows the contract owner to set the validators' federated power before.
    /// @notice subnet has already been bootstrapped.
    /// @param validators The list of validators' addresses.
    /// @param publicKeys The list of validators' public keys.
    /// @param powers The list of power values of the validators.
    function preBootstrapSetFederatedPower(
        address[] calldata validators,
        bytes[] calldata publicKeys,
        uint256[] calldata powers
    ) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        uint256 length = validators.length;

        if (length <= s.minValidators) {
            revert NotEnoughGenesisValidators();
        }

        gateValidatorNewPowers(validators, powers);

        for (uint256 i; i < length; ) {
            // check addresses
            address convertedAddress = publicKeyToAddress(publicKeys[i]);
            if (convertedAddress != validators[i]) {
                revert NotOwnerOfPublicKey();
            }

            // performing deduplication
            // validator should have no power when first added
            if (LibStaking.getPower(validators[i]) > 0) {
                revert DuplicatedGenesisValidator();
            }

            LibStaking.setMetadataWithConfirm(validators[i], publicKeys[i]);
            LibStaking.setFederatedPowerWithConfirm(validators[i], powers[i]);

            s.genesisValidators.push(Validator({addr: validators[i], weight: powers[i], metadata: publicKeys[i], storageAmount: 0}));

            unchecked {
                ++i;
            }
        }

        s.bootstrapped = true;
        emit SubnetBootstrapped(s.genesisValidators);

        // register adding the genesis circulating supply (if it exists)
        registerInGateway(0);
    }

    function registerInGateway(uint256 collateral) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        uint256 genesisCircSupply = s.genesisCircSupply;

        uint256 msgValue = 0;
        msgValue += s.supplySource.makeAvailable(s.ipcGatewayAddr, genesisCircSupply);
        msgValue += s.collateralSource.makeAvailable(s.ipcGatewayAddr, collateral);

        IGateway(s.ipcGatewayAddr).register{value: msgValue}(genesisCircSupply, collateral);
    }

    /// @notice method that allows the contract owner to set the validators' federated power after
    /// @dev subnet has already been bootstrapped.
    /// @param validators The list of validators' addresses.
    /// @param publicKeys The list of validators' public keys.
    /// @param powers The list of power values of the validators.
    function postBootstrapSetFederatedPower(
        address[] calldata validators,
        bytes[] calldata publicKeys,
        uint256[] calldata powers
    ) internal {
        uint256 length = validators.length;

        gateValidatorNewPowers(validators, powers);

        for (uint256 i; i < length; ) {
            // check addresses
            address convertedAddress = publicKeyToAddress(publicKeys[i]);
            if (convertedAddress != validators[i]) {
                revert NotOwnerOfPublicKey();
            }

            // no need to do deduplication as set directly set the power, there wont be any addition of
            // federated power.
            LibStaking.setFederatedPower({validator: validators[i], metadata: publicKeys[i], amount: powers[i]});

            unchecked {
                ++i;
            }
        }
    }

    /// @notice Removes an address from the initial balance keys.
    /// @param addr The address to be removed from the genesis balance keys.
    function rmAddressFromBalanceKey(address addr) internal {
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();

        uint256 length = s.genesisBalanceKeys.length;
        for (uint256 i; i < length; ) {
            if (s.genesisBalanceKeys[i] == addr) {
                s.genesisBalanceKeys[i] = s.genesisBalanceKeys[length - 1];
                s.genesisBalanceKeys.pop();
                // exit after removing the key
                break;
            }
            unchecked {
                ++i;
            }
        }
    }

    /// @notice Ensures that the provided parameters are valid to join the subnet
    function enforceJoinValidation(        
        bytes calldata publicKey,
        uint256 amount) internal {
        // Adding this check to prevent new validators from joining
        // after the subnet has been bootstrapped, if the subnet mode is not Collateral.
        SubnetActorStorage storage s = LibSubnetActorStorage.appStorage();
        if (s.bootstrapped) {
            enforceCollateralValidation();
        }

        if (amount == 0) {
            revert CollateralIsZero();
        }

        if (LibStaking.isValidator(msg.sender)) {
            revert MethodNotAllowed(ERR_VALIDATOR_JOINED);
        }

        if (publicKey.length != VALIDATOR_SECP256K1_PUBLIC_KEY_LENGTH) {
            // Taking 65 bytes because the FVM libraries have some assertions checking it, it's more convenient.
            revert InvalidPublicKeyLength();
        }

        address convertedAddress = publicKeyToAddress(publicKey);
        if (convertedAddress != msg.sender) {
            revert NotOwnerOfPublicKey();
        }

        gateValidatorPowerDelta(msg.sender, 0, amount);
    }
}