use ethers::{types::H256, utils::keccak256};
use uniswap_v3_math::error::UniswapV3MathError;

pub fn get_event_sig(sig: &str) -> H256 {
    H256::from(keccak256(sig.as_bytes()))
}

pub fn before_add_delta(x: u128, y: i128) -> Result<u128, UniswapV3MathError> {
    if y < 0 {
        let z = x.overflowing_add(-y as u128);

        if z.0 < x {
            Err(UniswapV3MathError::LiquidityAdd)
        } else {
            Ok(z.0)
        }
    } else {
        let z = x.overflowing_sub(y as u128);
        if z.1 {
            Err(UniswapV3MathError::LiquiditySub)
        } else {
            Ok(z.0)
        }
    }
}
