use crate::{error::DexQuoteError, types::DexQuoteResult};

// ref. https://github.com/Uniswap/v3-sdk/blob/main/src/constants.ts
pub fn get_tick_spacing(fee: u32) -> DexQuoteResult<i32> {
    let tick_spacing = match fee {
        100 => 1,
        500 => 10,
        3000 => 60,
        10000 => 200,
        _ => {
            return Err(DexQuoteError::InvalidFee(fee));
        }
    };
    Ok(tick_spacing)
}
