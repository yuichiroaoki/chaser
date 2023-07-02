use cfmms::checkpoint;
use dexquote::db::add_pool;
use ethers::providers::{Http, Middleware, Provider};
use std::{error::Error, sync::Arc, time::Instant};
use tracing::info;

use crate::config;

pub async fn import_pool(
    config_name: String,
    checkpoint_path: String,
    sync: bool,
) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let conf = config::get_config(config_name);
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
    let num_pools = pools.len();
    for pool in pools {
        add_pool(&redis_client, chain_id, pool)?;
    }

    info!("Imported {} pools in {:?}", num_pools, start.elapsed());

    Ok(())
}
