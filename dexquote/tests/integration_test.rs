use std::time::Instant;

use cfmms::checkpoint;
use dexquote::db::add_pool;
use dexquote::path;
use ethers::types::{Address, U256};
use indicatif::{ProgressBar, ProgressStyle};
use neo4rs::Graph;
use tracing::{info, warn};

const REDIS_URL: &str = "redis://:testtest@127.0.0.1:30073/";
const WETH_STR: &str = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
const USDC_STR: &str = "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8";

#[tokio::test]
async fn test_add_pool() {
    let start = Instant::now();

    let checkpoint_path = "fixtures/checkpoint.json";
    let redis_client = redis::Client::open(REDIS_URL).unwrap();
    let (_, pools, _) = checkpoint::deconstruct_checkpoint(&checkpoint_path);
    let chain_id = 42161;
    let graph = Graph::new("127.0.0.1:7687", "neo4j", "testtest")
        .await
        .unwrap();

    let total_pool_num = pools.len();
    let mut err_count = 0;
    let pb = ProgressBar::new(total_pool_num as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .expect("Error when setting progress bar style")
            .progress_chars("=> "),
    );
    pb.set_prefix("Importing");
    for pool in pools {
        match add_pool(&redis_client, chain_id, pool, &graph, "Arb").await {
            Ok(_) => {}
            Err(e) => {
                err_count += 1;
                warn!("Error adding pool: {:?}", e);
            }
        };
        pb.inc(1);
    }

    let elapsed = start.elapsed();
    pb.finish_and_clear();
    info!(
        "Imported {} pools in {} seconds with {} errors",
        total_pool_num - err_count,
        elapsed.as_secs(),
        err_count
    );
}

#[tokio::test]
async fn test_query_possible_path() {
    let start = Instant::now();
    let graph = Graph::new("127.0.0.1:7687", "neo4j", "testtest")
        .await
        .unwrap();

    let token_in: Address = WETH_STR.parse().unwrap();
    let token_out: Address = USDC_STR.parse().unwrap();
    let chain_id = 42161;
    let routes = path::get_possible_paths(&graph, token_in, token_out, 1, 10, "Arb")
        .await
        .unwrap();
    info!(
        "Queried possible paths in {} seconds",
        start.elapsed().as_secs()
    );

    assert_eq!(routes.len() > 0, true);
    let alchemy_api_key = std::env::var("ALCHEMY_API_KEY").expect("Could not get ALCHEMY_API_KEY");
    let json_rpc_url = format!(
        "https://arb-mainnet.g.alchemy.com/v2/{}",
        alchemy_api_key.as_str()
    );
    let amount_in = U256::exp10(18);
    let estimated_amount_out =
        path::get_amount_out_from_path(REDIS_URL, chain_id, &json_rpc_url, amount_in, &routes[0])
            .await
            .unwrap();
    assert_eq!(estimated_amount_out > U256::zero(), true);
}
