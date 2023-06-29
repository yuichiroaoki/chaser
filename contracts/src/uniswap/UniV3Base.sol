// SPDX-License-Identifier: MIT
pragma solidity >=0.7.0;

contract UniV3Base {
    /// @dev The minimum value that can be returned from #getSqrtRatioAtTick. Equivalent to getSqrtRatioAtTick(MIN_TICK)
    uint160 private constant MIN_SQRT_RATIO = 4295128739;
    /// @dev The maximum value that can be returned from #getSqrtRatioAtTick. Equivalent to getSqrtRatioAtTick(MAX_TICK)
    uint160 private constant MAX_SQRT_RATIO = 1461446703485210103287273052203988822378723970342;

    function getSqrtPriceLimitX96(bool zeroForOne) internal pure returns (uint160) {
        return zeroForOne ? MIN_SQRT_RATIO + 1 : MAX_SQRT_RATIO - 1;
    }

    function deltaToAmount(int256 amount0Delta, int256 amount1Delta)
        internal
        pure
        returns (uint256 amountIn, uint256 amountOut)
    {
        (amountIn, amountOut) = amount0Delta > 0
            ? (uint256(amount0Delta), uint256(-amount1Delta))
            : (uint256(amount1Delta), uint256(-amount0Delta));
    }

    function getTickSpacing(uint24 fee) internal pure returns (int24) {
        if (fee == 500) {
            return 10;
        } else if (fee == 3000) {
            return 60;
        } else if (fee == 100) {
            return 1;
        } else if (fee == 10000) {
            return 200;
        } else {
            revert("fee not supported");
        }
    }
}
