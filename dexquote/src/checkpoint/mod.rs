use cfmms::{
    checkpoint::generate_checkpoint_with_throttle,
    dex::{Dex, DexVariant},
};
use ethers::{
    providers::{Http, Provider},
    types::H160,
};
use std::{error::Error, str::FromStr, sync::Arc, time::Instant};
use tracing::{info, instrument};

use crate::config;

#[instrument]
pub async fn create_checkpoint(
    config_name: String,
    checkpoint_path: String,
) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let conf = config::get_config(config_name);
    let provider = Arc::new(Provider::<Http>::try_from(&conf.json_rpc_url).unwrap());

    let dexes = vec![
        // Sushiswap
        Dex::new(
            H160::from_str("0xc35DADB65012eC5796536bD9864eD8773aBc74C4").unwrap(),
            DexVariant::UniswapV2,
            70,
            Some(300),
        ),
        // UniswapV3
        Dex::new(
            H160::from_str("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap(),
            DexVariant::UniswapV3,
            35,
            None,
        ),
    ];

    generate_checkpoint_with_throttle(dexes, provider, 100000, 5, &checkpoint_path).await?;

    info!("Created a checkpoint in {:?}", start.elapsed());

    Ok(())
}
