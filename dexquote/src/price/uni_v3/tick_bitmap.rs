use crate::db::univ3::get_tick_bitmap_and_update_if_necessary;

use ethers::prelude::*;
use uniswap_v3_math::bit_math;
use uniswap_v3_math::error::UniswapV3MathError;

//Returns next and initialized. This function calls the node to get the word at the word_pos.
//current_word is the current word in the TickBitmap of the pool based on `tick`. TickBitmap[word_pos] = current_word
//Where word_pos is the 256 bit offset of the ticks word_pos.. word_pos := tick >> 8
pub async fn next_initialized_tick_within_one_word(
    redis_client: &redis::Client,
    chain_id: u64,
    json_rpc_url_or_ipc_path: &str,
    pool_address: H160,
    tick: i32,
    tick_spacing: i32,
    lte: bool,
) -> Result<(i32, bool), UniswapV3MathError> {
    let compressed = if tick < 0 && tick % tick_spacing != 0 {
        (tick / tick_spacing) - 1
    } else {
        tick / tick_spacing
    };

    if lte {
        let (word_pos, bit_pos) = position(compressed);
        let mask = (U256::one() << bit_pos) - 1 + (U256::one() << bit_pos);

        // get the word from redis if it exists, otherwise get it from the node
        let word = match get_tick_bitmap_and_update_if_necessary(
            redis_client,
            chain_id,
            pool_address,
            word_pos,
            json_rpc_url_or_ipc_path,
        )
        .await
        {
            Ok(word) => word,
            Err(e) => return Err(UniswapV3MathError::MiddlewareError(e.to_string())),
        };

        let masked = word & mask;

        let initialized = !masked.is_zero();

        let next = if initialized {
            (compressed
                - (bit_pos
                    .overflowing_sub(bit_math::most_significant_bit(masked)?)
                    .0) as i32)
                * tick_spacing
        } else {
            (compressed - bit_pos as i32) * tick_spacing
        };

        Ok((next, initialized))
    } else {
        let (word_pos, bit_pos) = position(compressed + 1);
        let mask = !((U256::one() << bit_pos) - U256::one());

        // get the word from redis if it exists, otherwise get it from the node
        let word = match get_tick_bitmap_and_update_if_necessary(
            redis_client,
            chain_id,
            pool_address,
            word_pos,
            json_rpc_url_or_ipc_path,
        )
        .await
        {
            Ok(word) => word,
            Err(e) => return Err(UniswapV3MathError::MiddlewareError(e.to_string())),
        };

        let masked = word & mask;
        let initialized = !masked.is_zero();

        let next = if initialized {
            (compressed
                + 1
                + (bit_math::least_significant_bit(masked)?
                    .overflowing_sub(bit_pos)
                    .0) as i32)
                * tick_spacing
        } else {
            (compressed + 1 + ((0xFF - bit_pos) as i32)) * tick_spacing
        };

        Ok((next, initialized))
    }
}

// returns (int16 wordPos, uint8 bitPos)
pub fn position(tick: i32) -> (i16, u8) {
    ((tick >> 8) as i16, (tick % 256) as u8)
}
