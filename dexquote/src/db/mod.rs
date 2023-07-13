use std::collections::HashMap;
pub mod token;
use cfmms::pool::Pool;
use ethers::types::Address;
use neo4rs::Graph;
use redis::RedisResult;

use crate::{
    graph::{add_pool_to_neo4j, add_token_pair_to_neo4j},
    subgraph::SubgraphPool,
    types::DexQuoteResult,
    utils::address_str,
};

pub mod univ2;
pub mod univ3;

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
) -> RedisResult<()> {
    let mut con = client.get_connection()?;
    let key = format!("{}:{}", chain_id, dex_string);
    redis::cmd("SADD")
        .arg(key)
        .arg(address_str(pool_address))
        .query(&mut con)
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

pub async fn add_pool(
    client: &redis::Client,
    chain_id: u64,
    pool: Pool,
    graph: &Graph,
    chain_label: &str,
) -> DexQuoteResult<()> {
    let (token0, token1);
    let pool_address = pool.address();
    match pool {
        Pool::UniswapV3(pool) => {
            add_dex_pool(client, chain_id, "UNIV3", pool_address)?;
            univ3::add_pool(client, chain_id, pool)?;
            (token0, token1) = (pool.token_a, pool.token_b)
        }
        Pool::UniswapV2(pool) => {
            add_dex_pool(client, chain_id, "UNIV2", pool_address)?;
            univ2::add_pool(client, chain_id, pool)?;
            (token0, token1) = (pool.token_a, pool.token_b)
        }
    }

    add_token_pair_to_neo4j(graph, chain_label, [token0, token1]).await;

    add_pool_to_neo4j(graph, chain_label, pool_address, token0, token1).await;
    Ok(())
}

pub async fn add_pool_from_subgraph(
    client: &redis::Client,
    chain_id: u64,
    pool: SubgraphPool,
    graph: &Graph,
    chain_label: &str,
) -> DexQuoteResult<()> {
    add_dex_pool(client, chain_id, "UNIV3", pool.address)?;
    univ3::add_pool_from_subgraph(client, chain_id, &pool)?;

    add_token_pair_to_neo4j(graph, chain_label, [pool.token0, pool.token1]).await;

    add_pool_to_neo4j(graph, chain_label, pool.address, pool.token0, pool.token1).await;
    Ok(())
}

pub fn get_pool(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
) -> DexQuoteResult<Option<Pool>> {
    let mut con = client.get_connection()?;
    let key = get_pool_key(pool_address, chain_id);
    let target_data: HashMap<String, String> =
        redis::cmd("HGETALL").arg(key).query(&mut con).unwrap();
    let dex = match target_data.get("dex") {
        Some(dex) => dex,
        None => {
            return Ok(None);
        }
    };
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
