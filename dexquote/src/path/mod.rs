use ethers::types::Address;
mod price;
use neo4rs::{query, Graph, Path};
pub use price::get_amount_out_from_path;

use crate::utils::address_str;

#[derive(Clone, Debug)]
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
