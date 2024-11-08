
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract SelfDestruct {
    address private owner;

    constructor() {
        owner = msg.sender;
    }

    function unprotected() public {
        selfdestruct(payable(msg.sender));
    }

    function unprotected2() external {
        suicide(owner);
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Not the owner");
        _;
    }
}
