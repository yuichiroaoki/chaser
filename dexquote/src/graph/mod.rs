use ethers::types::Address;
use neo4rs::{query, Graph, Node, Relation};

use crate::utils::address_str;

pub async fn check_if_pool_already_exists(
    graph: &Graph,
    token_label: &str,
    pool_address: Address,
) -> bool {
    let query_string = format!(
        "MATCH (:{token_label})-[r:Pool {{address: $address}}]-(:{token_label})
      RETURN r",
    );
    let mut result = graph
        .execute(query(&query_string).param("address", address_str(pool_address)))
        .await
        .unwrap();

    // if the pool exists, return
    if let Ok(Some(_)) = result.next().await {
        return true;
    }

    false
}

pub async fn add_token_pair_to_neo4j(graph: &Graph, token_label: &str, tokens: [Address; 2]) {
    let (token0_str, token1_str) = (address_str(tokens[0]), address_str(tokens[1]));

    // Since token nodes have unique constraints, you can't create the same node twice
    let query_string = format!(
        "CREATE (token0:{token_label} {{address: $token_address0}}), (token1:{token_label} {{address: $token_address1}})
        RETURN token0, token1",
    );
    let mut result = graph
        .execute(
            query(&query_string)
                .param("token_address0", token0_str.clone())
                .param("token_address1", token1_str.clone()),
        )
        .await
        .unwrap();

    while let Ok(Some(row)) = result.next().await {
        let token0_node: Node = row.get("token0").unwrap();
        let token1_node: Node = row.get("token1").unwrap();
        let token0_address: String = token0_node.get("address").unwrap();
        let token1_address: String = token1_node.get("address").unwrap();
        assert_eq!(token0_address, token0_str);
        assert_eq!(token1_address, token1_str);
    }
}

pub async fn delete_pool(
    graph: &Graph,
    token_label: &str,
    pool_address: Address,
) {
    let query_string = format!(
        "MATCH (:{token_label})-[r:Pool {{address: $address}}]-(:{token_label}) DELETE r",
    );
    let mut result = graph
        .execute(query(&query_string).param("address", address_str(pool_address)))
        .await
        .unwrap();

    while let Ok(Some(row)) = result.next().await {
        println!("{:?}", row);
    }
}

pub async fn add_pool_to_neo4j(
    graph: &Graph,
    token_label: &str,
    pool_address: Address,
    token0: Address,
    token1: Address,
) {
    if check_if_pool_already_exists(graph, token_label, pool_address).await {
        return;
    }

    let query_string = format!(
        "MATCH (a:{token_label}), (b:{token_label})
		WHERE a.address = $token0_address AND b.address = $token1_address 
		CREATE (a)-[r:Pool {{ address: $address }}]->(b) 
		RETURN r",
    );
    let mut result = graph
        .execute(
            query(&query_string)
                .param("token0_address", address_str(token0))
                .param("token1_address", address_str(token1))
                .param("address", address_str(pool_address)),
        )
        .await
        .unwrap();

    while let Ok(Some(row)) = result.next().await {
        let edge: Relation = row.get("r").unwrap();
        let address: String = edge.get("address").unwrap();
        assert_eq!(address, address_str(pool_address));
    }
}
