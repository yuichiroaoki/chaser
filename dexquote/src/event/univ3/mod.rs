use std::sync::Arc;

use crate::db::{get_pool, univ3};
use cfmms::pool::Pool;
use ethers::abi::AbiDecode;
use ethers::prelude::*;
mod decode;
mod tick_bitmap;
mod ticks;
pub use decode::*;
use tracing::{instrument, warn};
use uniswap_v3_math::{liquidity_math, tick_math};

pub struct UniV3SwapEvent {
    pub amount0: I256,
    pub amount1: I256,
    pub sqrt_price_x96: U256,
    pub liquidity: u128,
    pub tick: i32,
}

pub struct UniV3MintEvent {
    pub amount: u128,
    pub amount0: U256,
    pub amount1: U256,
}

// ref. https://github.com/Uniswap/v3-core/blob/main/contracts/interfaces/pool/IUniswapV3PoolEvents.sol
pub const UNIV3_SWAP_EVENT_SIG: &str = "Swap(address,address,int256,int256,uint160,uint128,int24)";
pub const UNIV3_MINT_EVENT_SIG: &str = "Mint(address,address,int24,int24,uint128,uint256,uint256)";
pub const UNIV3_BURN_EVENT_SIG: &str = "Burn(address,int24,int24,uint128,uint256,uint256)";

pub fn update_with_swap_event(
    redis_url: &str,
    chain_id: u64,
    pool_address: Address,
    log_data: &Bytes,
) {
    let redis_client = redis::Client::open(redis_url).unwrap();
    let univ3_event = match decode_swap_event(log_data) {
        Ok(event) => event,
        Err(e) => {
            warn!("failed to decode swap event: {:?}", e);
            return;
        }
    };
    // UniswapV3
    match univ3::update_pool(
        &redis_client,
        chain_id,
        pool_address,
        univ3_event.liquidity,
        univ3_event.sqrt_price_x96,
        univ3_event.tick,
    ) {
        Ok(_) => {}
        Err(e) => {
            warn!("failed to update pool: {:?}", e);
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct LiquidityUpdateParams {
    pub pool_address: Address,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity_delta: i128,
}

#[instrument]
pub async fn update_with_liquidity_event<M: Middleware + 'static>(
    redis_url: &str,
    chain_id: u64,
    pool_address: Address,
    log: &Log,
    is_mint: bool,
    middleware: Arc<M>,
) {
    let redis_client = redis::Client::open(redis_url).unwrap();
    let tick_lower = i32::decode(log.topics[2]).unwrap();
    let tick_upper = i32::decode(log.topics[3]).unwrap();
    let event_data = match is_mint {
        true => decode_mint_event(&log.data),
        false => decode_burn_event(&log.data),
    };
    let event_data = match event_data {
        Ok(event_data) => event_data,
        Err(e) => {
            warn!("failed to decode liquidity event: {:?}", e);
            return;
        }
    };
    let liquidity_delta = if is_mint {
        event_data.amount as i128
    } else {
        -(event_data.amount as i128)
    };
    let params = LiquidityUpdateParams {
        pool_address,
        tick_lower,
        tick_upper,
        liquidity_delta,
    };
    modify_position(&redis_client, chain_id, params, middleware).await;
}

// UniswapV3
#[instrument]
async fn modify_position<M: Middleware + 'static>(
    redis_client: &redis::Client,
    chain_id: u64,
    params: LiquidityUpdateParams,
    middleware: Arc<M>,
) {
    match check_ticks(params.tick_lower, params.tick_upper) {
        Ok(_) => {}
        Err(e) => {
            warn!("invalid ticks: {:?}", e);
            return;
        }
    }

    let pool_info = match get_pool(redis_client, chain_id, params.pool_address) {
        Ok(pool_info) => match pool_info {
            Some(pool_info) => match pool_info {
                Pool::UniswapV3(pool) => pool,
                _ => {
                    warn!("pool is not UniswapV3");
                    return;
                }
            },
            None => {
                warn!("pool not found");
                return;
            }
        },
        Err(e) => {
            warn!("{:?}", e);
            return;
        }
    };

    match update_position(
        redis_client,
        chain_id,
        params,
        pool_info.tick_spacing,
        middleware,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            warn!("failed to update position: {:?}", e);
            warn!(params.tick_lower, params.tick_upper, params.liquidity_delta);
        }
    };
    if params.liquidity_delta != 0 {
        if pool_info.tick < params.tick_lower {
            // current tick is below the passed range; liquidity can only become in range by crossing from left to
            // right, when we'll need _more_ token0 (it's becoming more valuable) so user must provide it
        } else if pool_info.tick < params.tick_upper {
            // current tick is inside the passed range
            match liquidity_math::add_delta(pool_info.liquidity, params.liquidity_delta) {
                Ok(new_liquiidty) => {
                    match univ3::update_liquidity(
                        redis_client,
                        chain_id,
                        params.pool_address,
                        new_liquiidty,
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("failed to update liquidity: {:?}", e);
                        }
                    };
                }
                Err(e) => {
                    warn!("failed to update liquidity: {:?}", e);
                    warn!(pool_info.liquidity, params.liquidity_delta);
                    warn!(pool_info.tick, params.tick_lower, params.tick_upper);
                }
            }
        } else {
            // current tick is above the passed range; liquidity can only become in range by crossing from right to
            // left, when we'll need _more_ token1 (it's becoming more valuable) so user must provide itj
        }
    }
}

/// @dev Common checks for valid tick inputs.
fn check_ticks(tick_lower: i32, tick_upper: i32) -> Result<(), Box<dyn std::error::Error>> {
    if tick_lower >= tick_upper {
        return Err("TLU".into());
    }
    if tick_lower < tick_math::MIN_TICK {
        return Err("TLM".into());
    }
    if tick_upper > tick_math::MAX_TICK {
        return Err("TUM".into());
    }
    Ok(())
}

async fn update_position<M: Middleware + 'static>(
    redis_client: &redis::Client,
    chain_id: u64,
    params: LiquidityUpdateParams,
    tick_spacing: i32,
    middleware: Arc<M>,
) -> Result<(), Box<dyn std::error::Error>> {
    // _updatePosition
    let mut flipped_lower = false;
    let mut flipped_upper = false;
    if params.liquidity_delta != 0 {
        // tick.lower ticks.update
        flipped_lower = ticks::update(
            redis_client,
            chain_id,
            params.pool_address,
            params.tick_lower,
            params.liquidity_delta,
            false,
            middleware.clone(),
        )
        .await?;

        // tick.upper ticks.update
        flipped_upper = ticks::update(
            redis_client,
            chain_id,
            params.pool_address,
            params.tick_upper,
            params.liquidity_delta,
            true,
            middleware.clone(),
        )
        .await?;

        // update bitmap
        if flipped_lower {
            tick_bitmap::flip_tick(
                redis_client,
                chain_id,
                params.pool_address,
                params.tick_lower,
                tick_spacing,
                middleware.clone(),
            )
            .await?;
        }
        if flipped_upper {
            tick_bitmap::flip_tick(
                redis_client,
                chain_id,
                params.pool_address,
                params.tick_upper,
                tick_spacing,
                middleware.clone(),
            )
            .await?;
        }
    }

    // clear any tick data that is no longer needed
    if params.liquidity_delta < 0 {
        if flipped_lower {
            ticks::clear(
                redis_client,
                chain_id,
                params.pool_address,
                params.tick_lower,
            );
        }
        if flipped_upper {
            ticks::clear(
                redis_client,
                chain_id,
                params.pool_address,
                params.tick_upper,
            );
        }
    }
    Ok(())
}
