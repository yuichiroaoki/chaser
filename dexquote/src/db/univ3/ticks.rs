use super::get_pool_ticks_key;
use super::UniV3Pool;
use ethers::abi::AbiDecode;
use ethers::abi::AbiEncode;
use ethers::{core::types::Address, prelude::*};
use redis::RedisResult;
use std::collections::HashMap;
use std::sync::Arc;

pub async fn get_ticks_from_provider<M: Middleware + 'static>(
    pool_address: Address,
    tick: i32,
    middleware: Arc<M>,
) -> Result<(u128, i128), ContractError<M>> {
    let contract = UniV3Pool::new(pool_address, middleware);
    let (
        liquidity_gross,
        liquidity_net,
        _fee_growth_outside0_x128,
        _fee_growth_outside1_x128,
        _tick_cumulative_outside,
        _seconds_per_liquidity_outside_x128,
        _seconds_outside,
        _initialized,
    ) = contract.ticks(tick).call().await?;
    Ok((liquidity_gross, liquidity_net))
}

pub fn get_ticks(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
    tick: i32,
) -> RedisResult<Option<(u128, i128)>> {
    let mut con = client.get_connection()?;
    let key = get_pool_ticks_key(pool_address, chain_id, tick);
    let target_data: HashMap<String, String> =
        redis::cmd("HGETALL").arg(key).query(&mut con).unwrap();
    if target_data.is_empty() {
        return Ok(None);
    }
    let liquidity_net = i128::decode_hex(target_data.get("liquidity_net").unwrap()).unwrap();
    let liquidity_gross = u128::decode_hex(target_data.get("liquidity_gross").unwrap()).unwrap();
    Ok(Some((liquidity_gross, liquidity_net)))
}

pub fn delete_ticks(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
    tick: i32,
) -> RedisResult<()> {
    let mut con = client.get_connection()?;
    let key = get_pool_ticks_key(pool_address, chain_id, tick);
    redis::cmd("DEL").arg(key).query(&mut con)
}

pub fn update_ticks(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
    tick: i32,
    liquidity_gross: u128,
    liquidity_net: i128,
) -> RedisResult<()> {
    let mut con = client.get_connection()?;
    let key = get_pool_ticks_key(pool_address, chain_id, tick);
    redis::cmd("HSET")
        .arg(key)
        .arg("liquidity_gross")
        .arg(liquidity_gross.encode_hex())
        .arg("liquidity_net")
        .arg(liquidity_net.encode_hex())
        .query(&mut con)
}
