use std::collections::HashMap;

use crate::db::univ2::{get_pool, hashmap_to_univ2};
use crate::error::DexQuoteError;
use crate::types::DexQuoteResult;
use cfmms::pool::Pool;
use ethers::prelude::*;
use ethers::types::U256;

pub const UNIV2_BASIC_FEE: u32 = 9970;
pub const FEE_DENOMINATOR: u32 = 10000;

pub fn get_price(
    redis_url: &str,
    chain_id: u64,
    pool_address: Address,
    token_in: Address,
    amount_in: U256,
) -> DexQuoteResult<U256> {
    let redis_client = redis::Client::open(redis_url).unwrap();
    let pool_state = match get_pool(&redis_client, chain_id, pool_address) {
        Ok(pool_state) => match pool_state {
            Some(pool_state) => pool_state,
            None => {
                return Err(DexQuoteError::PoolNotFound(pool_address));
            }
        },
        Err(e) => {
            return Err(DexQuoteError::RedisError(e));
        }
    };
    if let Pool::UniswapV2(pool) = pool_state {
        return Ok(pool.simulate_swap(token_in, amount_in));
    }
    Err(DexQuoteError::PoolNotFound(pool_address))
}

pub fn get_price_with_hashmap(
    pool_address: Address,
    token_in: Address,
    amount_in: U256,
    target_data: HashMap<String, String>,
) -> DexQuoteResult<U256> {
    let pool_state = hashmap_to_univ2(pool_address, target_data);
    if let Some(Pool::UniswapV2(pool)) = pool_state {
        return Ok(pool.simulate_swap(token_in, amount_in));
    }
    Err(DexQuoteError::PoolNotFound(pool_address))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{constants::provider::get_provider, db::univ2::add_pool};

    use super::*;
    use cfmms::pool::UniswapV2Pool;
    use ethers::utils::parse_units;

    const USDC_STR: &str = "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8";
    const WETH_STR: &str = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
    const REDIS_URL: &str = "redis://:testtest@127.0.0.1:30073/";
    const CHAIN_ID: u64 = 42161;

    abigen!(
        IUniswapV2Router02,
        r#"[
        getAmountsOut(uint256 amountIn, address[] path) external view returns (uint256[] amounts)
    ]"#,
    );

    pub async fn get_price_from_router(
        token_in: Address,
        token_out: Address,
        amount_in: U256,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let alchemy_api_key =
            std::env::var("ALCHEMY_API_KEY").expect("Could not get ALCHEMY_API_KEY");
        let provider = get_provider(&format!(
            "https://arb-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key.as_str()
        ))?;
        let client = Arc::new(provider);

        let router_address: Address = "0x1b02dA8Cb0d097eB8D57A175b88c7D8b47997506"
            .parse()
            .unwrap();
        let contract = IUniswapV2Router02::new(router_address, client);
        let amounts = contract
            .get_amounts_out(amount_in, vec![token_in, token_out])
            .call()
            .await?;
        Ok(amounts[1])
    }

    #[tokio::test]
    async fn test_get_price_usdc_weth_sushi() {
        let alchemy_api_key =
            std::env::var("ALCHEMY_API_KEY").expect("Could not get ALCHEMY_API_KEY");
        let provider = get_provider(&format!(
            "https://arb-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key.as_str()
        ))
        .unwrap();
        let middleware = Arc::new(provider);

        let token_in = USDC_STR.parse::<Address>().unwrap();
        let token_out = WETH_STR.parse::<Address>().unwrap();
        let amount_in: U256 = parse_units("10000", 6).unwrap().into();
        let pool_address: Address = "0x905dfCD5649217c42684f23958568e533C711Aa3"
            .parse()
            .unwrap();
        let pool_info = UniswapV2Pool::new_from_address(pool_address, middleware.clone())
            .await
            .unwrap();
        let redis_client = redis::Client::open(REDIS_URL).unwrap();
        add_pool(&redis_client, CHAIN_ID, pool_info).unwrap();
        let amount_out =
            super::get_price(REDIS_URL, CHAIN_ID, pool_address, token_in, amount_in).unwrap();
        assert_eq!((amount_out > parse_units("0", 18).unwrap().into()), true);
        let amount_out_from_router = get_price_from_router(token_in, token_out, amount_in)
            .await
            .unwrap();
        assert_eq!(amount_out, amount_out_from_router);
    }
}
