// https://github.com/Uniswap/smart-order-router/blob/main/src/util/chains.ts

use ethers::types::Address;
use serde_derive::Deserialize;

pub fn id_to_network_name(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "mainnet",
        10 => "optimism-mainnet",
        56 => "bnb-mainnet",
        137 => "polygon-mainnet",
        42161 => "arbitrum-mainnet",
        42220 => "celo-mainnet",
        43114 => "avalanche-mainnet",
        _ => "unknown",
    }
}

#[derive(Debug, Clone)]
pub struct SubgraphPool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: u32,
    pub liquidity: u128,
    pub tvl_eth: f64,
    pub tvl_usd: f64,
}

impl SubgraphPool {
    pub fn involves_token(&self, token: Address) -> bool {
        self.token0 == token || self.token1 == token
    }
}

#[derive(Deserialize, Debug)]
struct SubgraphToken {
    id: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct V3SubgraphPool {
    id: String,
    feeTier: String,
    liquidity: String,
    token0: SubgraphToken,
    token1: SubgraphToken,
    tvlETH: f64,
    tvlUSD: f64,
}

async fn get_v3_subgraph_pools(chain_id: u64) -> Vec<V3SubgraphPool> {
    let chain_name = id_to_network_name(chain_id);
    let uri =
        format!("https://cloudflare-ipfs.com/ipns/api.uniswap.org/v1/pools/v3/{chain_name}.json");
    // send request and parse response as json
    let resp: Vec<V3SubgraphPool> = reqwest::get(&uri)
        .await
        .expect("Failed to send request")
        .json()
        .await
        .expect("Failed to parse response");
    resp
}

pub async fn get_subgraph_poools(chain_id: u64) -> Vec<SubgraphPool> {
    let v3_pools = get_v3_subgraph_pools(chain_id).await;
    let mut pools = Vec::new();
    for pool in v3_pools {
        let pool = SubgraphPool {
            address: pool.id.parse().unwrap(),
            token0: pool.token0.id.parse().unwrap(),
            token1: pool.token1.id.parse().unwrap(),
            fee: pool.feeTier.parse().unwrap(),
            liquidity: pool.liquidity.parse().unwrap(),
            tvl_eth: pool.tvlETH,
            tvl_usd: pool.tvlUSD,
        };
        pools.push(pool);
    }
    pools
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_subgraph_pools_mainnet() {
        let mainnet_subgraph_pools = get_subgraph_poools(1).await;
        assert!(mainnet_subgraph_pools.len() > 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_subgraph_pools_optimism() {
        let optimism_subgraph_pools = get_subgraph_poools(10).await;
        assert!(optimism_subgraph_pools.len() > 0);
    }

    #[tokio::test]
    async fn test_get_subgraph_pools_bsc() {
        let bsc_subgraph_pools = get_subgraph_poools(56).await;
        assert!(bsc_subgraph_pools.len() > 0);
    }

    #[tokio::test]
    async fn test_get_subgraph_pools_polygon() {
        let polygon_subgraph_pools = get_subgraph_poools(137).await;
        assert!(polygon_subgraph_pools.len() > 0);
    }

    #[tokio::test]
    async fn test_get_subgraph_pools_arbitrum() {
        let arbitrum_subgraph_pools = get_subgraph_poools(42161).await;
        assert!(arbitrum_subgraph_pools.len() > 0);
    }

    #[tokio::test]
    async fn test_get_subgraph_pools_celo() {
        let celo_subgraph_pools = get_subgraph_poools(42220).await;
        assert!(celo_subgraph_pools.len() > 0);
    }

    #[tokio::test]
    async fn test_get_subgraph_pools_avalanche() {
        let avalanche_subgraph_pools = get_subgraph_poools(43114).await;
        assert!(avalanche_subgraph_pools.len() > 0);
    }
}
