// Copyright 2021 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Validator Manager
pragma solidity >=0.7.0;

interface MockPortal {
    // Ether - deposits/withdrawal of ether
    // ERC20 - deposit/withdrawal of ERC20 compatible tokens
    enum Operation {EtherOp, ERC20Op}

    // Deposit - deposit from an L1 address to an L2 address
    // Transfer - transfer from one L2 address to another
    // Withdraw - withdraw from an L2 address to an L1 address
    enum Transaction {Deposit, Transfer, Withdraw}

    /// @notice deposits ether in portal contract and create ether in L2
    /// @param _L2receivers array with receivers addresses
    /// @param _amounts array of amounts of ether to be distributed
    /// @param _data information to be interpreted by L2
    /// @return hash of input generated by deposit
    /// @dev  receivers[i] receive amounts[i]
    function etherDeposit(
        address[] calldata _L2receivers,
        uint256[] calldata _amounts,
        bytes calldata _data
    ) external payable returns (bytes32);

    /// @notice deposits ERC20 in portal contract and create tokens in L2
    /// @param _ERC20 address of ERC20 token to be deposited
    /// @param _L1Sender address on L1 that authorized the transaction
    /// @param _L2receivers array with receivers addresses
    /// @param _amounts array of amounts of ether to be distributed
    /// @param _data information to be interpreted by L2
    /// @return hash of input generated by deposit
    /// @dev  receivers[i] receive amounts[i]
    function erc20Deposit(
        address _ERC20,
        address _L1Sender,
        address[] calldata _L2receivers,
        uint256[] calldata _amounts,
        bytes calldata _data
    ) external returns (bytes32);

    /// @notice executes a rollups voucher
    /// @param _data data with information necessary to execute voucher
    /// @return status of voucher execution
    /// @dev can only be called by the Output contract
    function executeRollupsVoucher(bytes calldata _data)
        external
        returns (bool);

    /// @notice emitted on Ether deposited
    event EtherDeposited(
        address[] _L2receivers,
        uint256[] _amounts,
        bytes _data
    );

    /// @notice emitted on ERC20 deposited
    event ERC20Deposited(
        address _ERC20,
        address _L1Sender,
        address[] _L2receivers,
        uint256[] _amounts,
        bytes _data
    );

    /// @notice emitted on Ether withdrawal
    event EtherWithdrawn(address payable _receiver, uint256 _amount);

    /// @notice emitted on ERC20 withdrawal
    event ERC20Withdrawn(
        address _ERC20,
        address payable _receiver,
        uint256 _amount
    );
}
