use std::sync::Arc;

use crate::{
    db::univ3::{delete_ticks, get_ticks, get_ticks_from_provider, update_ticks},
    event::utils::before_add_delta,
};
use ethers::prelude::*;
use uniswap_v3_math::liquidity_math;

pub async fn update<M: Middleware + 'static>(
    redis_client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
    tick: i32,
    liquidity_delta: i128,
    upper: bool,
    middleware: Arc<M>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let liquidity_gross_after;
    let new_liquidity_net;
    let flipped;

    let ticks_on_redis = match get_ticks(redis_client, chain_id, pool_address, tick) {
        Ok(ticks) => ticks,
        Err(e) => {
            return Err(Box::new(e));
        }
    };
    if ticks_on_redis.is_some() {
        let (liquidity_gross_before, liquidity_net) = ticks_on_redis.unwrap();
        match liquidity_math::add_delta(liquidity_gross_before, liquidity_delta) {
            Ok(liquidity_gross) => {
                // require(liquidityGrossAfter <= maxLiquidity, 'LO');
                liquidity_gross_after = liquidity_gross;
                flipped = get_flipped(liquidity_gross_before, liquidity_gross_after);
                new_liquidity_net = if upper {
                    liquidity_net - liquidity_delta
                } else {
                    liquidity_net + liquidity_delta
                };
            }
            Err(e) => {
                println!("{:?}", e);

                (liquidity_gross_after, new_liquidity_net, flipped) =
                    get_liquidity_net_gross_flipped_from_provider(
                        pool_address,
                        tick,
                        liquidity_delta,
                        middleware,
                    )
                    .await?;
            }
        };
    } else {
        (liquidity_gross_after, new_liquidity_net, flipped) =
            get_liquidity_net_gross_flipped_from_provider(
                pool_address,
                tick,
                liquidity_delta,
                middleware,
            )
            .await?;
    }

    update_ticks(
        redis_client,
        chain_id,
        pool_address,
        tick,
        liquidity_gross_after,
        new_liquidity_net,
    )?;
    Ok(flipped)
}

async fn get_liquidity_net_gross_flipped_from_provider<M: Middleware + 'static>(
    pool_address: Address,
    tick: i32,
    liquidity_delta: i128,
    middleware: Arc<M>,
) -> Result<(u128, i128, bool), Box<dyn std::error::Error>> {
    let (liquidity_gross_after, new_liquidity_net) =
        get_ticks_from_provider(pool_address, tick, middleware).await?;
    let liquidity_gross_before = before_add_delta(liquidity_gross_after, liquidity_delta)?;

    let flipped = get_flipped(liquidity_gross_before, liquidity_gross_after);
    Ok((liquidity_gross_after, new_liquidity_net, flipped))
}

fn get_flipped(liquidity_gross_before: u128, liquidity_gross_after: u128) -> bool {
    (liquidity_gross_after == 0) != (liquidity_gross_before == 0)
}

pub fn clear(redis_client: &redis::Client, chain_id: u64, pool_address: Address, tick: i32) {
    match delete_ticks(redis_client, chain_id, pool_address, tick) {
        Ok(_) => {}
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
