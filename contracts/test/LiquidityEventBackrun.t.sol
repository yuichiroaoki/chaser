// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "../src/uniswap/interfaces/IUniversalRouter.sol";
import "../src/interfaces/IERC20.sol";
import "../src/uniswap/interfaces/IUniswapV2Router.sol";
import "../src/uniswap/interfaces/ISwapRouter.sol";
import "../src/uniswap/interfaces/IUniswapV2Pair.sol";
import "../src/woofi/IWooPPV2.sol";
import "../src/Bot.sol";

contract LiquidityEventBackrunTest is Test {
    IUniversalRouter immutable router = IUniversalRouter(0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD);
    address USDC = 0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8;
    address WOO = 0xcAFcD85D8ca7Ad1e1C6F82F651fA15E33AEfD07b;
    IERC20 usdc = IERC20(USDC);
    address immutable backrunBot = 0x0000900e00070d8090169000D2B090B67f0c1050;
    IUniswapV2Router immutable sushiRouter = IUniswapV2Router(0x1b02dA8Cb0d097eB8D57A175b88c7D8b47997506);
    ISwapRouter immutable swapRouter = ISwapRouter(0xE592427A0AEce92De3Edee1F18E0157C05861564);
    address immutable user = 0x0CF47073093Ec2f3C28f7b2A03A9fC345D6af678;
    IWooPPV2 immutable wooPPV2 = IWooPPV2(0xeFF23B4bE1091b53205E35f3AfCD9C7182bf3062);

    address sushiUsdcWooPool = 0xa9556426bdf92d02C613f2FD2e09943Fe6760395;
    uint256 immutable totalAmountIn = 83695536733689272452600;
    uint112 usdcReserve = 1537;
    uint112 wooReserve = 3971929858367134;

    /**
     * check initial liquidity amount before trigger tx
     * USDC: 1537 
     * WOO: 3971929858367134
     */
    function testLiquidityAmount() public {
        (uint112 reserve0, uint112 reserve1,) = IUniswapV2Pair(sushiUsdcWooPool).getReserves();
        (uint112 _usdcReserve, uint112 _wooReserve) = USDC < WOO ? (reserve0, reserve1) : (reserve1, reserve0);
        assertEq(_usdcReserve, usdcReserve);
        assertEq(_wooReserve, wooReserve);
        console.log("USDC", usdcReserve);
        console.log("WOO", wooReserve);
    }

    /**
     * trigger tx using SushiSwap router
     * Add
     * 266.287527571450924494 WOO
     * 103.044098 USDC
     *
     * ref:
     * https://arbiscan.io/tx/0x795114e6f654bd61649c3aa83d698b7add2340a7940f466ed4ec1bfb873d9d95
     */
    function trigger() internal {
        vm.prank(user);
        // user added liquidity with the same ratio as the pool
        // However, the initial token ratio was not correct
        uint256 amountADesired = 266287527571450924494; // WOO
        uint256 amountBDesired = 1537 * amountADesired / wooReserve; // USDC 
        assertEq(amountBDesired, 103044098);
        uint256 amountAMin = 264956089933593669871;
        uint256 amountBMin = 102528877;
        sushiRouter.addLiquidity(WOO, USDC, amountADesired, amountBDesired, amountAMin, amountBMin, user, 1686348252);
    }

    /**
     * better trigger tx using SushiSwap router
     * given that a good liquidity ratio on UniV2 pools is where the market value of the two tokens 
     * roughly are equal, we add the same market value of the two tokens to the pool
     * get the market value of WOO by querying on WoofiV2
     * 
     * Add
     * 266.287527571450924494 WOO (the same as the actual amount added)
     * 54.565978 USDC
     *
     */
    function betterTrigger() internal {
        // WOO
        uint256 amountADesired = 266287527571450924494;

        uint256 nextWooReserve = wooReserve + amountADesired;

        // how much nextWooReserve worth in USDC
        uint256 amountOut = wooPPV2.query(WOO, USDC, nextWooReserve);
        console.log("amountOut", amountOut);
        uint256 amountBDesired = amountOut - usdcReserve;
        // USDC
        console.log("amountBDesired", amountBDesired);
        vm.prank(user);
        sushiRouter.addLiquidity(WOO, USDC, amountADesired, amountBDesired, 1, 1, user, 1686348252);
    }

    function _uniV2Price(
        bool zeroForOne, // tokenIn < tokenOut
        address pool,
        uint256 amountIn
    ) internal view returns (uint256 amountOut) {
        (uint256 reserveIn, uint256 reserveOut) = getReserveInOut(
            zeroForOne,
            pool
        );
        require(
            reserveIn > 0 && reserveOut > 0,
            "UniswapV2Library: INSUFFICIENT_LIQUIDITY"
        );
        uint256 amountInWithFee = amountIn * 997;
        uint256 numerator = amountInWithFee * reserveOut;
        uint256 denominator = reserveIn * 1000 + amountInWithFee;
        return numerator / denominator;
    }

    function getReserveInOut(
        bool zeroForOne,
        address pool
    ) internal view returns (uint256 reserveIn, uint256 reserveOut) {
        (uint112 reserve0, uint112 reserve1, ) = IUniswapV2Pair(pool)
            .getReserves();
        (reserveIn, reserveOut) = zeroForOne
            ? (reserve0, reserve1)
            : (reserve1, reserve0);
    }

    /**
     * backrun
     * borrow 22.219075 USDC with Balancer flashloan
     * 22.219075 USDC - [WoofiV2] -> 107.779627732739178391 WOO
     * 107.779627732739178391 WOO - [Sushi USDC/WOO] -> 29.626667 USDC
     *
     * Profit: 7.407592 USDC
     *
     * ref:
     * https://arbiscan.io/tx/0x7e84c5153f513a0b49c667f1d6fbdb8602efdeae23cab7fd255d16f3314f7214
     */
    function backrun() internal {
        address bot = 0x9e5dA3FC9A9871D7F6d8E06e2ad56Ebc00DFDe3A;
        address botVault = 0xDAE6dd9AD7093EC747fBBA15e2592BaC9f12b881;
        uint256 usdcBalance = usdc.balanceOf(botVault);
        string memory dataStr =
            "0x627dd56a010200000000000000000000000001530943ff970a61a04b1ca14834a43f5de4533ebddb5cc8cafcd85d8ca7ad1e1c6f82f651fa15e33aefd07bff970a61a04b1ca14834a43f5de4533ebddb5cc81e0028ff970a61a04b1ca14834a43f5de4533ebddb5cc8cafcd85d8ca7ad1e1c6f82f651fa15e33aefd07b000018a9556426bdf92d02c613f2fd2e09943fe676039526f20000000000000000000000000000";
        bytes memory data = vm.parseBytes(dataStr);
        vm.prank(bot);
        (bool success,) = backrunBot.call{value: 0}(data);
        require(success, "backrun failed");
        uint256 profit = usdc.balanceOf(botVault) - usdcBalance;
        console.log("Bot Profit", profit);
    }

    /**
    * simulate the actual trigger and backrun tx
     */
    function testBackrun() public {
        trigger();
        backrun();
    }


    /**
    * simulate the better trigger and actual backrun tx
     * Still the bot makes 1.379824 USDC profit
     */
    function testBetterLiquidityAdd() public {
        betterTrigger();
        backrun();
    }
}
