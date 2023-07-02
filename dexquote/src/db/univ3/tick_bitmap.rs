use crate::constants::provider::get_provider;
use crate::constants::tick_spacing::get_tick_spacing;

use super::get_pool_tick_bitmap_key;
use super::UniV3Pool;
use ethers::{
    abi::{AbiDecode, AbiEncode},
    core::types::Address,
    prelude::*,
};
use redis::RedisResult;
use std::sync::Arc;
use uniswap_v3_math::tick_bitmap;

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

// get tickBitmap from redis if it exist, otherwise get it from the node and update redis
pub async fn get_tick_bitmap_and_update_if_necessary(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
    word_pos: i16,
    json_rpc_url: &str,
) -> Result<U256, Box<dyn std::error::Error>> {
    match get_tick_bitmap(client, chain_id, pool_address, word_pos) {
        Ok(word) => match word {
            Some(word) => Ok(word),
            None => {
                let provider = get_provider(json_rpc_url)?;
                let middleware = Arc::new(provider);
                let word =
                    get_tick_bitmap_from_provider(pool_address, word_pos, middleware).await?;
                update_tick_bitmap(client, chain_id, pool_address, word_pos, word)?;
                Ok(word)
            }
        },
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn add_tick_bitmap_from_tick(
    client: &redis::Client,
    chain_id: u64,
    pool_address: Address,
    tick: i32,
    fee: u32,
    json_rpc_url_or_ipc_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tick_spacing = get_tick_spacing(fee)?;
    let compressed = if tick < 0 && tick % tick_spacing != 0 {
        (tick / tick_spacing) - 1
    } else {
        tick / tick_spacing
    };

    // zero_for_one = true
    let (word_pos, _bit_pos) = tick_bitmap::position(compressed);
    let _word = get_tick_bitmap_and_update_if_necessary(
        client,
        chain_id,
        pool_address,
        word_pos,
        json_rpc_url_or_ipc_path,
    )
    .await?;

    // zero_for_one = false
    let (word_pos, _bit_pos) = tick_bitmap::position(compressed + 1);
    let _word = get_tick_bitmap_and_update_if_necessary(
        client,
        chain_id,
        pool_address,
        word_pos,
        json_rpc_url_or_ipc_path,
    )
    .await?;

    Ok(())
}
