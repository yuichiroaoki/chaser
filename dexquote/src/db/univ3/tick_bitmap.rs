use super::get_pool_tick_bitmap_key;
use super::UniV3Pool;
use ethers::{
    abi::{AbiDecode, AbiEncode},
    core::types::Address,
    prelude::*,
};
use redis::RedisResult;
use std::sync::Arc;

pub async fn get_tick_bitmap_from_provider<M: Middleware + 'static>(
    pool_address: Address,
    word_pos: i16,
    middleware: Arc<M>,
) -> Result<U256, ContractError<M>> {
    UniV3Pool::new(pool_address, middleware)
        .tick_bitmap(word_pos)
        .call()
        .await
}

pub fn get_tick_bitmap(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
    word_pos: i16,
) -> RedisResult<Option<U256>> {
    let mut con = client.get_connection()?;
    let key = get_pool_tick_bitmap_key(pool_address, chain_id, word_pos);
    let word: Option<String> = redis::cmd("GET").arg(key).query(&mut con).unwrap();
    Ok(word.map(|word| U256::decode_hex(word).unwrap()))
}

pub fn update_tick_bitmap(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
    word_pos: i16,
    word: U256,
) -> RedisResult<()> {
    let mut con = client.get_connection()?;
    let key = get_pool_tick_bitmap_key(pool_address, chain_id, word_pos);
    redis::cmd("SET")
        .arg(key)
        .arg(word.encode_hex())
        .query(&mut con)
}
