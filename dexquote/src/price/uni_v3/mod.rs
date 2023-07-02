use ethers::core::types::{Address, I256, U256};
mod custom;
pub use custom::*;
mod tick_bitmap;

use crate::{error::DexQuoteError, types::DexQuoteResult};

pub async fn get_price(
    redis_url: &str,
    chain_id: u64,
    json_rpc_url_or_ipc_path: String,
    pool_address: Address,
    zero_for_one: bool,
    amount_in: U256,
) -> DexQuoteResult<U256> {
    let pool_state = match custom::PoolState::init(
        redis_url,
        chain_id,
        json_rpc_url_or_ipc_path,
        pool_address,
    ) {
        Ok(pool_state) => match pool_state {
            Some(pool_state) => pool_state,
            None => {
                return Err(DexQuoteError::PoolNotFound(pool_address));
            }
        },
        Err(e) => {
            return Err(e);
        }
    };
    let amoount_specified = I256::from_raw(amount_in);
    let (amount_out, _, _, _) = match pool_state.get_price(amoount_specified, zero_for_one).await {
        Ok(result) => result,
        Err(e) => {
            return Err(DexQuoteError::GetPriceError(e.to_string()));
        }
    };
    Ok(amount_out)
}

#[cfg(not(feature = "ipc"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::provider::get_provider;
    use crate::db::univ3::{add_pool, add_tick_bitmap_from_tick};
    use cfmms::pool::UniswapV3Pool;
    use compute_univ3_address::uni_v3::get_pool_address;
    use ethers::{prelude::abigen, utils::parse_units};
    use std::sync::Arc;

    const USDC_STR: &str = "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8";
    const WETH_STR: &str = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
    const REDIS_URL: &str = "redis://:testtest@127.0.0.1:30073/";
    const CHAIN_ID: u64 = 42161;

    abigen!(
            Quoter,
        r#"[
        function quoteExactInputSingle(address tokenIn, address tokenOut,uint24 fee, uint256 amountIn, uint160 sqrtPriceLimitX96) external returns (uint256 amountOut)
    ]"#;
    );

    async fn get_price_from_quoter(
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        fee: u32,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let alchemy_api_key =
            std::env::var("ALCHEMY_API_KEY").expect("Could not get ALCHEMY_API_KEY");
        let provider = get_provider(&format!(
            "https://arb-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key.as_str()
        ))
        .unwrap();
        let client = Arc::new(provider);

        let quoter_address = "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6".parse::<Address>()?;
        let contract = Quoter::new(quoter_address, client);
        let sqrt_price_limit_x96 = U256::from_dec_str("0").unwrap();
        let amount_out = contract
            .quote_exact_input_single(token_in, token_out, fee, amount_in, sqrt_price_limit_x96)
            .call()
            .await?;
        Ok(amount_out)
    }

    #[tokio::test]
    async fn test_get_price_usdc_weth() {
        let alchemy_api_key =
            std::env::var("ALCHEMY_API_KEY").expect("Could not get ALCHEMY_API_KEY");
        let json_rpc_url = format!(
            "https://arb-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key.as_str()
        );
        let provider = get_provider(&json_rpc_url).unwrap();
        let middleware = Arc::new(provider);

        let token_in = USDC_STR.parse::<Address>().unwrap();
        let token_out = WETH_STR.parse::<Address>().unwrap();
        let zero_for_one = token_in < token_out;
        let amount_in: U256 = parse_units("4000", 6).unwrap().into();
        let fee = 500;
        let pool_address = get_pool_address(token_in, token_out, fee);

        let pool_info = UniswapV3Pool::new_from_address(pool_address, middleware.clone())
            .await
            .unwrap();
        let redis_client = redis::Client::open(REDIS_URL).unwrap();
        let tick = pool_info.tick;
        add_pool(&redis_client, CHAIN_ID, pool_info).unwrap();
        add_tick_bitmap_from_tick(
            &redis_client,
            CHAIN_ID,
            pool_address,
            tick,
            fee,
            &json_rpc_url,
        )
        .await
        .unwrap();

        let amount_out = super::get_price(
            REDIS_URL,
            CHAIN_ID,
            json_rpc_url,
            pool_address,
            zero_for_one,
            amount_in,
        )
        .await
        .unwrap();
        assert_eq!((amount_out > U256::zero()), true);
        let amount_out_quoter = get_price_from_quoter(token_in, token_out, amount_in, fee)
            .await
            .unwrap();
        assert_eq!(amount_out, amount_out_quoter);
    }

    #[tokio::test]
    async fn test_get_price_usdc_weth_3000() {
        let alchemy_api_key =
            std::env::var("ALCHEMY_API_KEY").expect("Could not get ALCHEMY_API_KEY");
        let json_rpc_url = format!(
            "https://arb-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key.as_str()
        );
        let provider = get_provider(&json_rpc_url).unwrap();
        let middleware = Arc::new(provider);

        let token_in = USDC_STR.parse::<Address>().unwrap();
        let token_out = WETH_STR.parse::<Address>().unwrap();
        let zero_for_one = token_in < token_out;
        let amount_in: U256 = parse_units("4000", 6).unwrap().into();
        let fee = 3000;
        let pool_address = get_pool_address(token_in, token_out, fee);
        let pool_info = UniswapV3Pool::new_from_address(pool_address, middleware.clone())
            .await
            .unwrap();
        let redis_client = redis::Client::open(REDIS_URL).unwrap();
        let tick = pool_info.tick;
        add_pool(&redis_client, CHAIN_ID, pool_info).unwrap();
        add_tick_bitmap_from_tick(
            &redis_client,
            CHAIN_ID,
            pool_address,
            tick,
            fee,
            &json_rpc_url,
        )
        .await
        .unwrap();

        let amount_out = super::get_price(
            REDIS_URL,
            CHAIN_ID,
            json_rpc_url,
            pool_address,
            zero_for_one,
            amount_in,
        )
        .await
        .unwrap();
        assert_eq!((amount_out > U256::zero()), true);
        let amount_out_quoter = get_price_from_quoter(token_in, token_out, amount_in, fee)
            .await
            .unwrap();
        assert_eq!(amount_out, amount_out_quoter);
    }
}
