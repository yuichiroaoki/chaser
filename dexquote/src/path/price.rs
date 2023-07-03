use super::PoolInfo;
use crate::price;
use ethers::types::U256;

pub async fn get_amount_out_from_path(
    redis_url: &str,
    chain_id: u64,
    json_rpc_url: &str,
    amount_in: U256,
    path: &[PoolInfo],
) -> Result<U256, Box<dyn std::error::Error>> {
    let mut estimated_amount_out = amount_in;
    for route in path {
        estimated_amount_out = price::get_price(
            redis_url,
            chain_id,
            json_rpc_url.to_string(),
            route.address,
            route.token_in,
            route.token_out,
            estimated_amount_out,
        )
        .await?;
    }
    Ok(estimated_amount_out)
}
