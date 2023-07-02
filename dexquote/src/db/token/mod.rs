use super::get_pool_key;
use ethers::{
    prelude::{abigen, ContractError},
    providers::Middleware,
    types::Address,
};
use redis::RedisResult;
use std::{collections::HashMap, sync::Arc};

abigen!(
    IERC20,
    r#"[
        symbol() external view returns (string memory)
        decimals() external view returns (uint8)
    ]"#,
);

pub struct TokenInfo {
    pub address: Address,
    pub symbol: String,
    pub decimals: u32,
    pub scam: u32,
}

pub fn get_token(
    client: &redis::Client,
    chain_id: u64,
    token_address: Address,
) -> RedisResult<Option<TokenInfo>> {
    let mut con = client.get_connection()?;
    let key = get_pool_key(token_address, chain_id);
    let target_data: HashMap<String, String> =
        redis::cmd("HGETALL").arg(key).query(&mut con).unwrap();
    if target_data.is_empty() {
        return Ok(None);
    }
    let symbol = target_data
        .get("symbol")
        .unwrap_or(&"".to_string())
        .to_string();
    let decimals = target_data
        .get("decimals")
        .unwrap_or(&"18".to_string())
        .parse()
        .unwrap();
    let scam = target_data
        .get("scam")
        .unwrap_or(&"2".to_string())
        .parse()
        .unwrap();
    Ok(Some(TokenInfo {
        address: token_address,
        symbol,
        decimals,
        scam,
    }))
}

pub fn add_token(client: &redis::Client, chain_id: u64, token_info: TokenInfo) {
    let mut con = client.get_connection().unwrap();
    let key = get_pool_key(token_info.address, chain_id);
    let _: () = redis::cmd("HSET")
        .arg(key)
        .arg("symbol")
        .arg(token_info.symbol)
        .arg("decimals")
        .arg(token_info.decimals)
        .arg("scam")
        .arg(token_info.scam)
        .query(&mut con)
        .unwrap();
}

pub fn update_scam(
    client: &redis::Client,
    chain_id: u64,
    token_address: Address,
    scam: u32,
) -> RedisResult<()> {
    let mut con = client.get_connection().unwrap();
    let key = get_pool_key(token_address, chain_id);
    redis::cmd("HSET")
        .arg(key)
        .arg("scam")
        .arg(scam)
        .query(&mut con)
}

pub async fn add_token_from_provider<M: Middleware + 'static>(
    client: &redis::Client,
    chain_id: u64,
    middleware: Arc<M>,
    token_address: Address,
    scam: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut con = client.get_connection().unwrap();
    let symbol = erc20_address_to_symbol(middleware.clone(), token_address).await?;
    let decimals = erc20_address_to_decimals(middleware, token_address).await?;
    let key = get_pool_key(token_address, chain_id);
    redis::cmd("HSET")
        .arg(key)
        .arg("symbol")
        .arg(symbol)
        .arg("decimals")
        .arg(decimals)
        .arg("scam")
        .arg(scam)
        .query(&mut con)?;
    Ok(())
}

pub async fn erc20_address_to_symbol<M: Middleware + 'static>(
    middleware: Arc<M>,
    address: Address,
) -> Result<String, ContractError<M>> {
    let token = IERC20::new(address, middleware);
    token.symbol().call().await
}

pub async fn erc20_address_to_decimals<M: Middleware + 'static>(
    middleware: Arc<M>,
    address: Address,
) -> Result<u8, ContractError<M>> {
    let token = IERC20::new(address, middleware);
    token.decimals().call().await
}
