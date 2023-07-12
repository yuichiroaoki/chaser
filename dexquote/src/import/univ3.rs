use dexquote::{
    db::{add_pool_from_subgraph, get_pool},
    subgraph,
};
use ethers::providers::{Http, Middleware, Provider};
use indicatif::{ProgressBar, ProgressStyle};
use neo4rs::Graph;
use std::{error::Error, sync::Arc, time::Instant};
use tracing::{info, warn};

use crate::config;

pub async fn import_pool(config_name: String) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let conf = config::get_config(config_name);
    let graph = Graph::new(conf.neo4j_uri, "neo4j", conf.neo4j_pass).await?;

    let redis_client = redis::Client::open(conf.redis_url).unwrap();

    let provider = Arc::new(Provider::<Http>::try_from(&conf.json_rpc_url).unwrap());
    let chain_id = provider.get_chainid().await?.as_u64();

    let pools = subgraph::get_subgraph_poools(chain_id).await;

    let total_pool_num = pools.len();
    let mut err_count = 0;
    let mut already_imported = 0;
    let pb = ProgressBar::new(total_pool_num as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .expect("Error when setting progress bar style")
            .progress_chars("=> "),
    );
    pb.set_prefix("Importing");
    for pool in pools {
        let pool_on_redis = get_pool(&redis_client, chain_id, pool.address);

        if pool_on_redis.is_err() {
            err_count += 1;
            pb.inc(1);
            continue;
        };

        if let Ok(Some(_pool_on_redis)) = pool_on_redis {
            already_imported += 1;
            pb.inc(1);
            continue;
        };

        match add_pool_from_subgraph(&redis_client, chain_id, pool, &graph, &conf.chain_label).await
        {
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
        total_pool_num,
        already_imported,
        err_count,
        "Imported {} pools in {} seconds",
        total_pool_num - err_count - already_imported,
        elapsed.as_secs(),
    );

    Ok(())
}
