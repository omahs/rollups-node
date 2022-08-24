// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Input Box
pragma solidity ^0.8.13;

import {CanonicalMachine} from "../common/CanonicalMachine.sol";

contract InputBox {
    using CanonicalMachine for CanonicalMachine.Log2Size;

    mapping(address => bytes32[]) public inputBoxes;

    event InputAdded(address indexed dapp, address sender, bytes input);

    function addInput(address _dapp, bytes calldata _input)
        public
        returns (bytes32)
    {
        bytes32[] storage inputBox = inputBoxes[_dapp];

        bytes32 inputHash = computeInputHash(
            msg.sender,
            block.number,
            block.timestamp,
            _input,
            inputBox.length
        );

        // add input to correct inbox
        inputBox.push(inputHash);

        // block.number and timestamp can be retrieved by the event metadata itself
        emit InputAdded(_dapp, msg.sender, _input);

        return inputHash;
    }

    function computeInputHash(
        address sender,
        uint256 blockNumber,
        uint256 blockTimestamp,
        bytes calldata input,
        uint256 inputIndex
    ) internal pure returns (bytes32) {
        // TODO guarantee that unwrapping is worth the gas cost
        require(
            input.length <=
                (1 << CanonicalMachine.INPUT_MAX_LOG2_SIZE.uint64OfSize()),
            "input len: [0,driveSize]"
        );

        bytes32 keccakMetadata = keccak256(
            abi.encode(
                sender,
                blockNumber,
                blockTimestamp,
                0, //TODO decide how to deal with epoch index
                inputIndex // input index
            )
        );

        bytes32 keccakInput = keccak256(input);

        return keccak256(abi.encode(keccakMetadata, keccakInput));
    }
}
