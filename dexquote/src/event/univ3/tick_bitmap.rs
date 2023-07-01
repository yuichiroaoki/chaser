use std::sync::Arc;

use crate::db::univ3::{get_tick_bitmap, get_tick_bitmap_from_provider, update_tick_bitmap};
use ethers::prelude::*;
use uniswap_v3_math::tick_bitmap::position;

pub async fn flip_tick<M: Middleware + 'static>(
    redis_client: &redis::Client,
    chain_id: u64,
    pool_address: H160,
    tick: i32,
    tick_spacing: i32,
    middleware: Arc<M>,
) -> Result<(), Box<dyn std::error::Error>> {
    // require(tick % tickSpacing == 0); // ensure that the tick is spaced
    // (int16 wordPos, uint8 bitPos) = position(tick / tickSpacing);
    // uint256 mask = 1 << bitPos;
    // self[wordPos] ^= mask;
    assert!(tick % tick_spacing == 0);
    let (word_pos, bit_pos) = position(tick / tick_spacing);
    let mask = U256::one() << bit_pos;

    let current_word = get_tick_bitmap(redis_client, chain_id, pool_address, word_pos)?;
    match current_word {
        Some(current_word) => {
            let new_word = current_word ^ mask;
            update_tick_bitmap(redis_client, chain_id, pool_address, word_pos, new_word)?;
        }
        None => {
            let word = get_tick_bitmap_from_provider(pool_address, word_pos, middleware).await?;
            update_tick_bitmap(redis_client, chain_id, pool_address, word_pos, word)?;
        }
    };
    Ok(())
}
