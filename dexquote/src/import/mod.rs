use cfmms::checkpoint;
use dexquote::db::{add_pool, get_pool};
use ethers::providers::{Http, Middleware, Provider};
use indicatif::{ProgressBar, ProgressStyle};
use neo4rs::Graph;
use std::{error::Error, sync::Arc, time::Instant};
use tracing::{info, warn};

use crate::config;

pub async fn import_pool(
    config_name: String,
    checkpoint_path: String,
    sync: bool,
) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let conf = config::get_config(config_name);
    let graph = Graph::new(conf.neo4j_uri, "neo4j", conf.neo4j_pass).await?;

    let redis_client = redis::Client::open(conf.redis_url).unwrap();

    let provider = Arc::new(Provider::<Http>::try_from(&conf.json_rpc_url).unwrap());
    let chain_id = provider.get_chainid().await?.as_u64();

    let pools;
    if sync {
        (_, pools) = checkpoint::sync_pools_from_checkpoint_with_throttle(
            &checkpoint_path,
            100000,
            5,
            provider,
        )
        .await?;
    } else {
        (_, pools, _) = checkpoint::deconstruct_checkpoint(&checkpoint_path);
    }

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
        let pool_address = pool.address();
        let pool_on_redis = get_pool(&redis_client, chain_id, pool_address)?;
        if let Some(_pool_on_redis) = pool_on_redis {
            continue;
        };
        match add_pool(&redis_client, chain_id, pool, &graph, &conf.chain_label).await {
            Ok(_) => {}
            Err(e) => {
                err_count += 1;
                warn!("Error adding pool: {:?}", e);
            }
        };
        pb.inc(1);
    }

    pb.finish_and_clear();
    let elapsed = start.elapsed();
    info!(
        "Imported {} pools in {} seconds with {} errors",
        total_pool_num - err_count,
        elapsed.as_secs(),
        err_count
    );

    Ok(())
}
