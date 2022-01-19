// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Ether Portal interface
pragma solidity >=0.7.0;

interface IEtherPortal {
    /// @notice deposit an amount of Ether in the portal and create Ether in L2
    /// @param _data information to be interpreted by L2
    /// @return hash of input generated by deposit
    function etherDeposit(bytes calldata _data)
        external
        payable
        returns (bytes32);

    /// @notice withdraw an amount of Ether from the portal
    /// @param _data data with withdrawal information
    /// @dev can only be called by the Rollups contract
    function etherWithdrawal(bytes calldata _data) external returns (bool);

    /// @notice emitted on Ether deposited
    event EtherDeposited(address _sender, uint256 _amount, bytes _data);

    /// @notice emitted on Ether withdrawal
    event EtherWithdrawn(address payable _receiver, uint256 _amount);
}
