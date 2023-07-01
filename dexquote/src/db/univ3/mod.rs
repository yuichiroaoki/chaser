use super::{get_pool_hashmap, get_pool_key};
use crate::utils::address_str;
use cfmms::pool::{Pool, UniswapV3Pool};
use ethers::{
    abi::{AbiDecode, AbiEncode},
    prelude::*,
};
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
    Ok(hashmap_to_univ3(pool_address, target_data))
}

fn hashmap_to_univ3(pool_address: Address, target_data: HashMap<String, String>) -> Option<Pool> {
    let fee = target_data.get("fee")?.parse().unwrap();
    let token_a = target_data.get("token0")?.parse().unwrap();
    let token_a_decimals = target_data.get("token0_decimals")?.parse().unwrap();
    let token_b = target_data.get("token1")?.parse().unwrap();
    let token_b_decimals = target_data.get("token1_decimals")?.parse().unwrap();
    let liquidity = target_data.get("liquidity")?.parse().unwrap();
    let sqrt_price = U256::decode_hex(target_data.get("sqrt_price")?).unwrap();
    let tick = target_data.get("tick")?.parse().unwrap();
    let tick_spacing = target_data.get("tick_spacing")?.parse().unwrap();
    let liquidity_net = target_data.get("liquidity_net")?.parse().unwrap();
    Some(Pool::UniswapV3(UniswapV3Pool {
        address: pool_address,
        fee,
        token_a,
        token_a_decimals,
        token_b,
        token_b_decimals,
        liquidity,
        sqrt_price,
        tick,
        tick_spacing,
        liquidity_net,
    }))
}

pub fn add_pool(client: &redis::Client, chain_id: u64, pool: UniswapV3Pool) {
    let mut con = client.get_connection().unwrap();
    let key = get_pool_key(pool.address, chain_id);
    let _: () = redis::cmd("HSET")
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
        .arg("liquidity")
        .arg(pool.liquidity.to_string())
        .arg("sqrt_price")
        .arg(pool.sqrt_price.encode_hex())
        .arg("tick")
        .arg(pool.tick)
        .arg("tick_spacing")
        .arg(pool.tick_spacing)
        .arg("liquidity_net")
        .arg(pool.liquidity_net.to_string())
        .arg("dex")
        .arg("UNIV3")
        .query(&mut con)
        .unwrap();
}
