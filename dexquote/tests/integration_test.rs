use std::sync::Arc;
use std::time::Instant;

use cfmms::checkpoint;
use dexquote::path;
use dexquote::{constants::provider::get_provider, db::add_pool};
use ethers::types::{Address, U256};
use neo4rs::Graph;

const REDIS_URL: &str = "redis://:testtest@127.0.0.1:30073/";
const WETH_STR: &str = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
const USDC_STR: &str = "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8";
const NEO4J_URI: &str = "bolt://localhost:7687";
const NEO4J_USER: &str = "neo4j";

#[tokio::test]
async fn test_query_possible_path() {
    let start = Instant::now();

    let alchemy_api_key = std::env::var("ALCHEMY_API_KEY").expect("Could not get ALCHEMY_API_KEY");
    let json_rpc_url = format!(
        "https://arb-mainnet.g.alchemy.com/v2/{}",
        alchemy_api_key.as_str()
    );
    let provider = get_provider(&json_rpc_url).unwrap();
    let checkpoint_path = "fixtures/checkpoint.json";
    let redis_client = redis::Client::open(REDIS_URL).unwrap();
    let (_, pools) = checkpoint::sync_pools_from_checkpoint_with_throttle(
        &checkpoint_path,
        100000,
        5,
        Arc::new(provider),
    )
    .await
    .unwrap();
    let chain_id = 42161;
    let graph = Graph::new(NEO4J_URI, NEO4J_USER, "testtest").await.unwrap();

    let total_pool_num = pools.len();
    let mut err_count = 0;
    for pool in pools {
        match add_pool(&redis_client, chain_id, pool, &graph, "Arb").await {
            Ok(_) => {}
            Err(e) => {
                err_count += 1;
                println!("Error adding pool: {:?}", e);
            }
        };
    }

    let elapsed = start.elapsed();
    println!(
        "Imported {} pools in {} seconds with {} errors",
        total_pool_num - err_count,
        elapsed.as_secs(),
        err_count
    );

    let token_in: Address = WETH_STR.parse().unwrap();
    let token_out: Address = USDC_STR.parse().unwrap();
    let routes = path::get_possible_paths(&graph, token_in, token_out, 2, 10, "Arb")
        .await
        .unwrap();
    println!(
        "Queried possible paths in {} seconds",
        start.elapsed().as_secs()
    );

    assert_eq!(routes.len() > 0, true);
    let amount_in = U256::exp10(18);
    let estimated_amount_out =
        path::get_amount_out_from_path(REDIS_URL, chain_id, &json_rpc_url, amount_in, &routes[0])
            .await
            .unwrap();
    assert_eq!(estimated_amount_out > U256::zero(), true);
}
