use cfmms::{
    dex::{Dex, DexVariant},
    sync,
};
use dexquote::db::add_pool;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::H160,
};
use std::{error::Error, str::FromStr, sync::Arc};

use crate::config;

pub async fn import_pool(
    config_name: String,
    checkpoint_path: String,
) -> Result<(), Box<dyn Error>> {
    let conf = config::get_config(config_name);
    let redis_client = redis::Client::open(conf.redis_url).unwrap();

    let provider = Arc::new(Provider::<Http>::try_from(&conf.json_rpc_url).unwrap());
    let chain_id = provider.get_chainid().await?.as_u64();
    let dexes = vec![
        //Add UniswapV3
        Dex::new(
            H160::from_str("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap(),
            DexVariant::UniswapV3,
            12369621,
            None,
        ),
    ];

    // Sync pairs
    let result =
        sync::sync_pairs_with_throttle(dexes, 100000, provider, 5, Some(&checkpoint_path)).await?;
    for pool in result {
        add_pool(&redis_client, chain_id, pool);
    }

    Ok(())
}
