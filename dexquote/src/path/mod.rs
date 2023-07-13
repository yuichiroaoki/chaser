use ethers::types::Address;
mod price;
use neo4rs::{query, Graph, Path};
pub use price::get_amount_out_from_path;
use serde_derive::{Deserialize, Serialize};

use crate::{subgraph::SubgraphPool, utils::address_str};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolInfo {
    pub address: Address,
    pub token_in: Address,
    pub token_out: Address,
}

pub async fn get_possible_paths(
    graph: &Graph,
    token_in: Address,
    token_out: Address,
    hop: u64,
    path_result_limit: u64,
    chain_label: &str,
) -> Option<Vec<Vec<PoolInfo>>> {
    let query_string = format!(
        "Match p=(:{chain_label} {{address: $token0_address}})-[r*..{hop}]-(:{chain_label} {{address: $token1_address}})
      RETURN p LIMIT {path_result_limit}"
    );
    let mut result = graph
        .execute(
            query(&query_string)
                .param("token0_address", address_str(token_in))
                .param("token1_address", address_str(token_out)),
        )
        .await
        .unwrap();

    let mut paths = Vec::new();
    while let Ok(Some(row)) = result.next().await {
        let mut route = Vec::new();
        let path: Path = row.get("p").unwrap();
        let nodes = path.nodes();
        let edges = path.rels();
        for (i, edge) in edges.iter().enumerate() {
            let token_in: String = nodes[i].get("address").unwrap();
            let token_out: String = nodes[i + 1].get("address").unwrap();
            let address: String = edge.get("address").unwrap();
            let pool_address = address.parse::<Address>().unwrap();
            let pool = PoolInfo {
                address: pool_address,
                token_in: token_in.parse().unwrap(),
                token_out: token_out.parse().unwrap(),
            };
            route.push(pool);
        }
        paths.push(route);
    }
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

pub struct ComputeRoutes {
    token_in: Address,
    token_out: Address,
    pools: Vec<SubgraphPool>,
    max_hops: u8,
    pools_used: Vec<bool>,
    routes: Vec<SubgraphRoute>,
}

#[derive(Clone, Debug)]
pub struct SubgraphRoute {
    pub token_in: Address,
    pub token_out: Address,
    pub pools: Vec<SubgraphPool>,
}

impl ComputeRoutes {
    pub fn new(
        token_in: Address,
        token_out: Address,
        pools: Vec<SubgraphPool>,
        max_hops: u8,
    ) -> Self {
        let pool_len = pools.len();
        ComputeRoutes {
            token_in,
            token_out,
            pools,
            max_hops,
            pools_used: vec![false; pool_len],
            routes: Vec::new(),
        }
    }

    pub fn get_routes(&self) -> Vec<SubgraphRoute> {
        self.routes.clone()
    }

    // https://github.com/Uniswap/smart-order-router/blob/main/src/routers/alpha-router/functions/compute-all-routes.ts
    pub fn compute_all_univ3_routes(&mut self) {
        self.compute_routes(Vec::new(), None);
    }

    fn compute_routes(
        &mut self,
        current_route: Vec<SubgraphPool>,
        _previous_token_out: Option<Address>,
    ) {
        if current_route.len() > self.max_hops.into() {
            return;
        }

        if !current_route.is_empty()
            && current_route[current_route.len() - 1].involves_token(self.token_out)
        {
            self.routes.push(SubgraphRoute {
                pools: current_route,
                token_in: self.token_in,
                token_out: self.token_out,
            });
            return;
        }

        for i in 0..self.pools.len() {
            if self.pools_used[i] {
                continue;
            }

            let cur_pool = &self.pools[i];
            let previous_token_out = if let Some(previous_token_out) = _previous_token_out {
                previous_token_out
            } else {
                self.token_in
            };

            if !cur_pool.involves_token(previous_token_out) {
                continue;
            }

            let current_token_out = if cur_pool.token0 == previous_token_out {
                cur_pool.token1
            } else {
                cur_pool.token0
            };

            let mut _current_route = current_route.clone();
            _current_route.push(cur_pool.clone());
            self.pools_used[i] = true;
            self.compute_routes(_current_route, Some(current_token_out));
            self.pools_used[i] = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::subgraph;

    use super::*;

    const USDC_STR: &str = "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8";
    const WETH_STR: &str = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";

    #[tokio::test]
    async fn test_compute_all_univ3_routes() {
        let chain_id = 42161;
        let subgraph_pools = subgraph::get_subgraph_poools(chain_id).await;
        let first_ten_pools = subgraph_pools[0..300].to_vec();
        let mut compute_route = ComputeRoutes::new(
            USDC_STR.parse().unwrap(),
            WETH_STR.parse().unwrap(),
            first_ten_pools,
            2,
        );

        compute_route.compute_all_univ3_routes();
        let routes = compute_route.get_routes();
        for route in routes {
            let token_in = route.token_in;
            let token_out = route.token_out;
            assert_eq!(token_in, USDC_STR.parse().unwrap());
            assert_eq!(token_out, WETH_STR.parse().unwrap());
            let hop = route.pools.len();
            assert_eq!(hop <= 2, true);
            if hop == 1 {
                assert_eq!(route.pools[0].involves_token(token_in), true);
                assert_eq!(route.pools[0].involves_token(token_out), true);
            } else if hop == 2 {
                assert_eq!(route.pools[0].involves_token(token_in), true);
                assert_eq!(route.pools[1].involves_token(token_out), true);
            } else {
                panic!("hop should be less than 3");
            }
        }
    }
}
