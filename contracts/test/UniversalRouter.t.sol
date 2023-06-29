// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "../src/uniswap/interfaces/IUniversalRouter.sol";
import "../src/interfaces/IERC20.sol";

contract UniversalRouterTest is Test {
    IUniversalRouter immutable router = IUniversalRouter(0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD);
    IERC20 weth = IERC20(0x82aF49447D8a07e3bd95BD0d56f35241523fBab1);
    address immutable backrunBot1 = 0x0000900e00070d8090169000D2B090B67f0c1050;
    address immutable backrunBot2 = 0x9c1bD19B3A6820fe6B4f0AeaaAE7e786D44A3c82;

    function trigger() internal {
        // https://arbiscan.io/tx/0xf655ab9e4f21121476abc0c8b26c382e042c9b8fb1b18638fef839962301e076
        address user = 0x36528721ee15c46f2d24Fb6bfc5b580029749c5a;

        uint256 initBalance = user.balance;
        console.log("Eth Balance", initBalance);

        string memory input0 =
            "0x000000000000000000000000e73394f6a157a0fa656da2b73bbeda85c38dfdec000000000000000000000000ffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000064bdb82600000000000000000000000000000000000000000000000000000000000000000000000000000000000000003fc91a3afd70395cd496c647d5a6cc9d4b2b7fad000000000000000000000000000000000000000000000000000000006496322e00000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000041a6f00d1e51f896ff3c759d7fa841b1a9bd081a05e07bfcddb4ae05acce3f54e7012503b94e1ba0d6a5c329a5d25e80f0e169277f263cfd8b3731d084047a629d1b00000000000000000000000000000000000000000000000000000000000000";
        string memory input1 =
            "0x0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000b1e07dc231427d00000000000000000000000000000000000000000000000000000001f01ef1ec16b9b00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002be73394f6a157a0fa656da2b73bbeda85c38dfdec000bb882af49447d8a07e3bd95bd0d56f35241523fbab1000000000000000000000000000000000000000000";
        string memory input2 =
            "0x0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000b1e07dc231427d000000000000000000000000000000000000000000000000000000029db962f78e27a00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002be73394f6a157a0fa656da2b73bbeda85c38dfdec00271082af49447d8a07e3bd95bd0d56f35241523fbab1000000000000000000000000000000000000000000";
        string memory input3 =
            "0x00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000048dd854e3a4e15";

        bytes[] memory inputs = new bytes[](4);
        inputs[0] = vm.parseBytes(input0);
        inputs[1] = vm.parseBytes(input1);
        inputs[2] = vm.parseBytes(input2);
        inputs[3] = vm.parseBytes(input3);

        string memory commandsStr = "0x0a00000c";
        bytes memory commands = vm.parseBytes(commandsStr);
        vm.prank(user);
        router.execute(commands, inputs, 1687563343);

        uint256 finalBalance = user.balance;
        console.log("Eth Balance", finalBalance);
    }

    function backrun1() internal {
        // backrun #1
        // https://arbiscan.io/tx/0xbcf2e46c15110cb8ca020e227493e56e695b4501cebb5111e12c53d8d266ead0
        address bot1 = 0xBAcda30A60230b0303d87c8A7232fE320754f339;
        address bot1Vault = 0xDAE6dd9AD7093EC747fBBA15e2592BaC9f12b881;
        uint256 wethBalance = weth.balanceOf(bot1Vault);
        string memory data1Str =
            "0x627dd56a000200000000000000000029bea8d60cc1ec82af49447d8a07e3bd95bd0d56f35241523fbab1e73394f6a157a0fa656da2b73bbeda85c38dfdec82af49447d8a07e3bd95bd0d56f35241523fbab1080014fc711972e37f62dc82406b5ff98e68e4fe654e3c00001873df54500e52343516c0110b4863afad77423b9c26f20000";
        bytes memory data1 = vm.parseBytes(data1Str);
        vm.prank(bot1);
        (bool success,) = backrunBot1.call{value: 0}(data1);
        require(success, "backrun #1 failed");
        uint256 profit1 = weth.balanceOf(bot1Vault) - wethBalance;
        console.log("Bot#1 Profit", profit1);
    }

    function backrun2() internal {
        // backrun #2
        // https://arbiscan.io/tx/0xef872b446b503f7b1ff079873902f01051f709889c38f2dc12348994bfb5b2bc
        address bot2 = 0x0312Bb0a7Df13d99c6e4A6d2aAcB58925bef8537;
        address bot2Vault = 0xDc6dD6f3CB363D02444B1612552297F46A5cdFBf;
        uint256 wethBalance = weth.balanceOf(bot2Vault);
        string memory data2Str =
            "0x627dd56a000200000000000000000029bea8d60cc1ec82af49447d8a07e3bd95bd0d56f35241523fbab1e73394f6a157a0fa656da2b73bbeda85c38dfdec82af49447d8a07e3bd95bd0d56f35241523fbab1080014cdb3a8ade333fb408db9dcf4326c70b1c3229bb500001873df54500e52343516c0110b4863afad77423b9c26f20000";
        bytes memory data2 = vm.parseBytes(data2Str);
        vm.prank(bot2);
        (bool success,) = backrunBot2.call{value: 0}(data2);
        require(success, "backrun #2 failed");
        uint256 profit2 = weth.balanceOf(bot2Vault) - wethBalance;
        console.log("Bot#2 Profit", profit2);
    }

    function testBackrun() public {
        trigger();
        backrun1();
        backrun2();
    }
}
