// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "../src/uniswap/interfaces/IUniversalRouter.sol";
import "../src/interfaces/IERC20.sol";
import "../src/uniswap/interfaces/IUniswapV2Router.sol";
import "../src/uniswap/interfaces/ISwapRouter.sol";
import "../src/Bot.sol";

contract UniversalRouterTest is Test {
    IUniversalRouter immutable router = IUniversalRouter(0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD);
    IERC20 weth = IERC20(0x82aF49447D8a07e3bd95BD0d56f35241523fBab1);
    IERC20 xirtam = IERC20(0xe73394F6a157A0Fa656Da2b73BbEDA85c38dfDeC);
    address immutable backrunBot1 = 0x0000900e00070d8090169000D2B090B67f0c1050;
    address immutable backrunBot2 = 0x9c1bD19B3A6820fe6B4f0AeaaAE7e786D44A3c82;
    IUniswapV2Router immutable sushiRouter = IUniswapV2Router(0x1b02dA8Cb0d097eB8D57A175b88c7D8b47997506);
    ISwapRouter immutable swapRouter = ISwapRouter(0xE592427A0AEce92De3Edee1F18E0157C05861564);
    address immutable user = 0x36528721ee15c46f2d24Fb6bfc5b580029749c5a;

    address univ3WethXirtam10000 = 0xcDb3a8ade333fB408dB9dCF4326C70b1c3229bB5;

    /**
     * trigger tx using UniversalRouter
     * 52,500 XIRTAM - UniV3 1% -> WETH
     * 31,195.5367336892724526 XIRTAM - UniV3 0.3% -> WETH
     *
     * ETH received: 0.021535251082800023 ETH
     *
     * ref:
     * https://arbiscan.io/tx/0xf655ab9e4f21121476abc0c8b26c382e042c9b8fb1b18638fef839962301e076
     */
    function trigger() internal {
        uint256 initBalance = user.balance;
        uint256 initTokenBalance = xirtam.balanceOf(user);

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

        uint256 tokenAmountIn = initTokenBalance - xirtam.balanceOf(user);
        console.log("amount in", tokenAmountIn);
        uint256 amountOut = user.balance - initBalance;
        console.log("amount out", amountOut);
    }

    /**
     * backrun #1
     * WETH - UniV3 ETH/XIRTAM 1% -> XIRTAM
     * XIRTAM - Sushi 0.3% -> WETH
     *
     * Profit: 0.01227623956324444 WETH
     *
     * ref:
     * https://arbiscan.io/tx/0xbcf2e46c15110cb8ca020e227493e56e695b4501cebb5111e12c53d8d266ead0
     * https://explorer.phalcon.xyz/tx/arbitrum/0xbcf2e46c15110cb8ca020e227493e56e695b4501cebb5111e12c53d8d266ead0
     */
    function backrun1() internal {
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

    /**
     * backrun #2
     * WETH - UniV3 ETH/XIRTAM 0.3% -> XIRTAM
     * XIRTAM - Sushi 0.3% -> WETH
     *
     * Profit: 0.000425571747640877 WETH
     *
     * ref:
     * https://arbiscan.io/tx/0xef872b446b503f7b1ff079873902f01051f709889c38f2dc12348994bfb5b2bc
     * https://explorer.phalcon.xyz/tx/arbitrum/0xef872b446b503f7b1ff079873902f01051f709889c38f2dc12348994bfb5b2bc
     */
    function backrun2() internal {
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

    function swapOnSushi(address tokenIn, address tokenOut, uint256 amountIn) internal returns (uint256) {
        address[] memory path = new address[](2);
        path[0] = address(tokenIn);
        path[1] = address(tokenOut);
        uint256 initBalance = user.balance;
        vm.prank(user);
        xirtam.approve(address(sushiRouter), amountIn);

        vm.prank(user);
        sushiRouter.swapExactTokensForETHSupportingFeeOnTransferTokens(amountIn, 1, path, user, 1687563343);

        uint256 amountOut = user.balance - initBalance;
        return amountOut;
    }

    function swapOnUniV310000(address tokenIn, address tokenOut, uint256 amountIn) internal returns (uint256) {
        vm.prank(user);
        xirtam.approve(address(swapRouter), amountIn);

        vm.prank(user);
        uint256 amountOut = swapRouter.exactInputSingle(
            ISwapRouter.ExactInputSingleParams({
                tokenIn: tokenIn,
                tokenOut: tokenOut,
                fee: 10000,
                recipient: user,
                deadline: 1687563343,
                amountIn: amountIn,
                amountOutMinimum: 1,
                sqrtPriceLimitX96: 0
            })
        );

        return amountOut;
    }

    /**
     * If user(0x36528721ee15c46f2d24Fb6bfc5b580029749c5a) chose to swap XIRTAM to ETH on Sushi,
     * he/she could have get 0.035107906712239745 ETH and this swap wouldn't have created an arbitrage opportunity.
     *
     * 83,695.5367336892724526 XIRTAM - Sushi 0.3% -> WETH
     *
     * ETH received: 0.035107906712239745 ETH
     */
    function testSushi() public {
        uint256 amountIn = 83695536733689272452600;
        // xirtam => ETH on Sushi
        uint256 amountOut = swapOnSushi(address(xirtam), address(weth), amountIn);
        console.log("ETH received:", amountOut);

        // xirtam => ETH on UniV3
        // ETH => xirtam on Sushi

        Bot bot = new Bot();
        uint256 backrunAmountIn = 69553673368922;
        vm.expectRevert();
        // WETH => xirtam on Sushi
        // xirtam => WETH on UniV3
        // there is no arbitrage opportunity and the transaction will be reverted
        bot.backrunOnUniV3Sushi(univ3WethXirtam10000, address(xirtam), address(weth), backrunAmountIn);
    }

    function testSushiAndUniV310000() public {
        uint256 amountIn = 83695536733689272452600;
        // xirtam => ETH on Sushi
        uint256 amountOut = swapOnSushi(address(xirtam), address(weth), 80000000000000000000000);
        amountOut += swapOnUniV310000(address(xirtam), address(weth), amountIn - 80000000000000000000000);
        console.log("ETH received:", amountOut);
    }
}
