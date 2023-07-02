use dexquote::constants::provider::get_provider;
use dexquote::db::get_dex_pools;
use dexquote::event::get_event_sig;
use dexquote::event::univ2::{update_with_sync_event, UNIV2_SYNC_EVENT_SIG};
use dexquote::event::univ3::{
    update_with_liquidity_event, update_with_swap_event, UNIV3_BURN_EVENT_SIG,
    UNIV3_MINT_EVENT_SIG, UNIV3_SWAP_EVENT_SIG,
};
use ethers::prelude::*;
use std::sync::Arc;
use tracing::info;

use crossbeam_channel::unbounded;

use crate::config::{self, Config};

fn get_filter(config_name: String, chain_id: u64) -> Filter {
    let conf = config::get_config(config_name);
    let redis_client = redis::Client::open(conf.redis_url).unwrap();

    let univ3_pools = get_dex_pools(&redis_client, chain_id, "UNIV3");
    let univ2_pools = get_dex_pools(&redis_client, chain_id, "UNIV2");
    let all_pools = [univ3_pools, univ2_pools];
    let all_pools: Vec<Address> = all_pools.iter().flatten().cloned().collect();
    Filter::new().address(all_pools)
}

pub async fn update_pool_states(
    threads: usize,
    config_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let conf = config::get_config(config_name.clone());
    let provider = get_ws_provider(conf.ws_rpc_url).await?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let client = Arc::new(provider);
    let swap_filter = get_filter(config_name.clone(), chain_id);

    // Create a channel to receive messages from the feed client
    let (sender, receiver) = unbounded();

    // Create a new thread that runs the background maintain loop
    tokio::spawn(async move {
        let mut stream = client.subscribe_logs(&swap_filter).await.unwrap();
        while let Some(log) = stream.next().await {
            sender.send(log).unwrap();
        }
    });

    for _ in 0..threads - 1 {
        let conf = config::get_config(config_name.clone());
        let receiver_clone = receiver.clone();

        tokio::spawn(async move {
            loop {
                let log = receiver_clone
                    .recv()
                    .expect("Failed to receive data from feed client");

                update_pool_state(log, &conf, chain_id).await;
            }
        });
    }

    // main thread
    let conf = config::get_config(config_name);
    let receiver_clone = receiver.clone();
    loop {
        let log = receiver_clone
            .recv()
            .expect("Failed to receive data from feed client");

        update_pool_state(log, &conf, chain_id).await;
    }
}

async fn update_pool_state(log: Log, conf: &Config, chain_id: u64) {
    let event_sig = log.topics[0];
    let provider = get_provider(&conf.json_rpc_url).unwrap();
    let middleware = Arc::new(provider);
    match log.transaction_hash {
        Some(tx_hash) => {
            info!("tx_hash: {:?}", tx_hash);
        }
        None => return,
    }
    let pool_address = log.address;
    if event_sig == get_event_sig(UNIV3_SWAP_EVENT_SIG) {
        update_with_swap_event(&conf.redis_url, chain_id, pool_address, &log.data);
    } else if event_sig == get_event_sig(UNIV2_SYNC_EVENT_SIG) {
        update_with_sync_event(&conf.redis_url, chain_id, pool_address, &log.data);
    } else if event_sig == get_event_sig(UNIV3_MINT_EVENT_SIG) {
        update_with_liquidity_event(
            &conf.redis_url,
            chain_id,
            pool_address,
            &log,
            true,
            middleware,
        )
        .await
    } else if event_sig == get_event_sig(UNIV3_BURN_EVENT_SIG) {
        update_with_liquidity_event(
            &conf.redis_url,
            chain_id,
            pool_address,
            &log,
            false,
            middleware,
        )
        .await
    }
}

/// Return a Provider for the given Websocket URL
pub async fn get_ws_provider(
    ws_rpc_url: String,
) -> Result<Provider<Ws>, Box<dyn std::error::Error>> {
    let ws_provider = Provider::<Ws>::connect(ws_rpc_url).await?;
    Ok(ws_provider)
}
