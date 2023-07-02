use super::{get_pool_hashmap, get_pool_key};
use crate::utils::address_str;
use cfmms::pool::{Pool, UniswapV2Pool};
use ethers::prelude::*;
use redis::RedisResult;
use std::collections::HashMap;

pub fn get_pool(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
) -> RedisResult<Option<Pool>> {
    let target_data = get_pool_hashmap(client, chain_id, pool_address)?;
    if target_data.is_empty() {
        return Ok(None);
    }
    Ok(hashmap_to_univ2(pool_address, target_data))
}

fn hashmap_to_univ2(pool_address: Address, target_data: HashMap<String, String>) -> Option<Pool> {
    let fee = target_data.get("fee")?.parse().unwrap();
    let token_a = target_data.get("token0")?.parse().unwrap();
    let token_a_decimals = target_data.get("token0_decimals")?.parse().unwrap();
    let token_b = target_data.get("token1")?.parse().unwrap();
    let token_b_decimals = target_data.get("token1_decimals")?.parse().unwrap();
    let reserve_0 = target_data.get("reserve0")?.parse().unwrap();
    let reserve_1 = target_data.get("reserve1")?.parse().unwrap();
    Some(Pool::UniswapV2(UniswapV2Pool {
        address: pool_address,
        fee,
        token_a,
        token_a_decimals,
        token_b,
        token_b_decimals,
        reserve_0,
        reserve_1,
    }))
}

pub fn add_pool(client: &redis::Client, chain_id: u64, pool: UniswapV2Pool) -> RedisResult<()> {
    let mut con = client.get_connection()?;
    let key = get_pool_key(pool.address, chain_id);
    redis::cmd("HSET")
        .arg(key)
        .arg("fee")
        .arg(pool.fee)
        .arg("token0")
        .arg(address_str(pool.token_a))
        .arg("token0_decimals")
        .arg(pool.token_a_decimals)
        .arg("token1")
        .arg(address_str(pool.token_b))
        .arg("token1_decimals")
        .arg(pool.token_b_decimals)
        .arg("reserve0")
        .arg(pool.reserve_0.to_string())
        .arg("reserve1")
        .arg(pool.reserve_1.to_string())
        .arg("dex")
        .arg("UNIV2")
        .query(&mut con)
}

pub fn update_pool(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
    reserve0: u128,
    reserve1: u128,
) -> RedisResult<()> {
    let mut con = client.get_connection()?;
    let key = get_pool_key(pool_address, chain_id);
    redis::cmd("HSET")
        .arg(key)
        .arg("reserve0")
        .arg(reserve0.to_string())
        .arg("reserve1")
        .arg(reserve1.to_string())
        .query(&mut con)
}
