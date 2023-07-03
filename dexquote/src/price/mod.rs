use std::str::FromStr;

use ethers::types::{Address, U256};

use crate::{db::get_pool_hashmap, dex::Dex, error::DexQuoteError, types::DexQuoteResult};

pub mod uni_v2;
pub mod uni_v3;

pub async fn get_price(
    redis_url: &str,
    chain_id: u64,
    json_rpc_url: String,
    pool_address: Address,
    token_in: Address,
    token_out: Address,
    amount_in: U256,
) -> DexQuoteResult<U256> {
    let redis_client = redis::Client::open(redis_url)?;
    let target_data = get_pool_hashmap(&redis_client, chain_id, pool_address)?;
    if target_data.is_empty() {
        return Err(DexQuoteError::PoolNotFound(pool_address));
    }
    let dex_string = target_data.get("dex").unwrap();
    let dex = match Dex::from_str(dex_string) {
        Ok(dex) => dex,
        Err(_) => {
            return Err(DexQuoteError::InvalidDex(dex_string.to_string()));
        }
    };
    let zero_for_one = token_in < token_out;
    match dex {
        Dex::UniswapV3 => {
            uni_v3::get_price_with_hashmap(
                redis_url,
                chain_id,
                json_rpc_url,
                pool_address,
                zero_for_one,
                amount_in,
                target_data,
            )
            .await
        }
        Dex::UniswapV2 => {
            uni_v2::get_price_with_hashmap(pool_address, token_in, amount_in, target_data)
        }
    }
}
