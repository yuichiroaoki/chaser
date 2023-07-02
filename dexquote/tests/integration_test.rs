use std::time::Instant;

use cfmms::checkpoint;
use dexquote::db::add_pool;
use indicatif::{ProgressBar, ProgressStyle};
use neo4rs::Graph;
use tracing::{info, warn};

const REDIS_URL: &str = "redis://:testtest@127.0.0.1:30073/";

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
