// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "./uniswap/interfaces/IUniversalRouter.sol";
import "./interfaces/IERC20.sol";
import "./uniswap/interfaces/IUniswapV2Router.sol";
import "./uniswap/interfaces/ISwapRouter.sol";
import "./uniswap/interfaces/IUniswapV3SwapCallback.sol";
import "./uniswap/interfaces/IUniswapV3Pool.sol";
import "./uniswap/UniV3Base.sol";

error Fail(string reason);

/**
 * Example bot contract
 */
contract Bot is IUniswapV3SwapCallback, UniV3Base, Test {
    IUniswapV2Router immutable sushiRouter = IUniswapV2Router(0x1b02dA8Cb0d097eB8D57A175b88c7D8b47997506);
    ISwapRouter immutable swapRouter = ISwapRouter(0xE592427A0AEce92De3Edee1F18E0157C05861564);

    function uniswapV3SwapCallback(int256 amount0Delta, int256 amount1Delta, bytes calldata data) external override {
        (address tokenIn, address tokenOut) = abi.decode(data, (address, address));
        (uint256 amountIn, uint256 amountOut) = deltaToAmount(amount0Delta, amount1Delta);
        uint256 amountOutOnSushi = swapOnSushi(tokenOut, tokenIn, amountOut);
		console.log("sushi: %s", amountOutOnSushi);
		console.log("univ3: %s", amountIn);
        if (amountIn > amountOutOnSushi) {
            revert Fail("Not profitable");
        }
        IERC20(tokenIn).transfer(msg.sender, amountIn);
    }

    function backrunOnUniV3Sushi(address uniV3Pool, address tokenIn, address tokenOut, uint256 amountIn) external {
		bool zeroForOne = tokenIn < tokenOut;
        IUniswapV3Pool(uniV3Pool).swap(
            address(this), zeroForOne, int256(amountIn), getSqrtPriceLimitX96(zeroForOne), abi.encode(tokenIn, tokenOut)
        );
    }

    function swapOnSushi(address tokenIn, address tokenOut, uint256 amountIn) internal returns (uint256) {
        uint256 initAmount = IERC20(tokenOut).balanceOf(address(this));
        address[] memory path = new address[](2);
        path[0] = address(tokenIn);
        path[1] = address(tokenOut);
		IERC20(tokenIn).approve(address(sushiRouter), amountIn);
        sushiRouter.swapExactTokensForTokensSupportingFeeOnTransferTokens(
            amountIn, 1, path, address(this), block.timestamp
        );
        return IERC20(tokenOut).balanceOf(address(this)) - initAmount;
    }
}
