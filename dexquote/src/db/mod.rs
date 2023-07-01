use std::collections::HashMap;

use cfmms::pool::Pool;
use ethers::types::Address;
use redis::RedisResult;

use crate::utils::address_str;

mod univ2;
mod univ3;

/// Get pool key for redis
pub fn get_pool_key(pool_address: Address, chain_id: u64) -> String {
    format!("{}:{}", chain_id, address_str(pool_address))
}

pub fn get_dex_pools(client: &redis::Client, chain_id: u64, dex_string: &str) -> Vec<Address> {
    let mut con = client.get_connection().unwrap();
    let key = format!("{}:{}", chain_id, dex_string);
    let pools: Vec<String> = redis::cmd("SMEMBERS").arg(key).query(&mut con).unwrap();
    pools.iter().map(|x| x.parse().unwrap()).collect()
}

pub fn add_dex_pool(
    client: &redis::Client,
    chain_id: u64,
    dex_string: &str,
    pool_address: Address,
) {
    let mut con = client.get_connection().unwrap();
    let key = format!("{}:{}", chain_id, dex_string);
    let _: () = redis::cmd("SADD")
        .arg(key)
        .arg(address_str(pool_address))
        .query(&mut con)
        .unwrap();
}

pub fn remove_dex_pool(
    client: &redis::Client,
    chain_id: u64,
    dex_string: &str,
    pool_address: Address,
) -> RedisResult<()> {
    let mut con = client.get_connection()?;
    let key = format!("{}:{}", chain_id, dex_string);
    redis::cmd("SREM")
        .arg(key)
        .arg(address_str(pool_address))
        .query(&mut con)
}

pub fn delete_pool(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
) -> RedisResult<()> {
    let mut con = client.get_connection()?;
    let key = get_pool_key(pool_address, chain_id);
    redis::cmd("DEL").arg(key).query(&mut con)
}

pub fn get_pool_hashmap(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
) -> RedisResult<HashMap<String, String>> {
    let mut con = client.get_connection()?;
    let key = get_pool_key(pool_address, chain_id);
    let target_data: HashMap<String, String> =
        redis::cmd("HGETALL").arg(key).query(&mut con).unwrap();
    Ok(target_data)
}

pub fn add_pool(client: &redis::Client, chain_id: u64, pool: Pool) {
    match pool {
        Pool::UniswapV3(pool) => {
            add_dex_pool(client, chain_id, "UNIV3", pool.address);
            univ3::add_pool(client, chain_id, pool);
        }
        Pool::UniswapV2(pool) => {
            add_dex_pool(client, chain_id, "UNIV2", pool.address);
            univ2::add_pool(client, chain_id, pool);
        }
    }
}

pub fn get_pool(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
) -> RedisResult<Option<Pool>> {
    let mut con = client.get_connection()?;
    let key = get_pool_key(pool_address, chain_id);
    let target_data: HashMap<String, String> =
        redis::cmd("HGETALL").arg(key).query(&mut con).unwrap();
    let dex = target_data.get("dex").unwrap();
    match dex.as_str() {
        "UNIV3" => {
            let pool = univ3::get_pool(client, chain_id, pool_address)?;
            Ok(pool)
        }
        "UNIV2" => {
            let pool = univ2::get_pool(client, chain_id, pool_address)?;
            Ok(pool)
        }
        _ => Ok(None),
    }
}
