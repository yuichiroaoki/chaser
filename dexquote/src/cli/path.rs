use dexquote::path::{get_amount_out_from_path, get_possible_paths, PoolInfo};
use ethers::types::{Address, U256};
use neo4rs::Graph;

use crate::config;

#[derive(Debug)]
struct Route {
    pub _info: Vec<PoolInfo>,
    pub estimated_amount_out: U256,
}

async fn get_paths(
    token_in: String,
    token_out: String,
    hop: u64,
    path_result_limit: u64,
    config_name: String,
) -> Vec<Vec<PoolInfo>> {
    let conf = config::get_config(config_name);
    let graph = Graph::new(conf.neo4j_uri, "neo4j", conf.neo4j_pass)
        .await
        .unwrap();
    let token_in = token_in.parse::<Address>().unwrap();
    let token_out = token_out.parse::<Address>().unwrap();
    get_possible_paths(
        &graph,
        token_in,
        token_out,
        hop,
        path_result_limit,
        conf.chain_label.as_str(),
    )
    .await
    .unwrap()
}

pub async fn show_paths(
    token_in: String,
    token_out: String,
    hop: u64,
    path_result_limit: u64,
    config_name: String,
) {
    let paths = get_paths(token_in, token_out, hop, path_result_limit, config_name).await;
    println!("{:#?}", paths);
}

pub async fn show_best_prices(
    token_in: String,
    token_out: String,
    hop: u64,
    path_result_limit: u64,
    amount_in: String,
    config_name: String,
) {
    let paths = get_paths(
        token_in,
        token_out,
        hop,
        path_result_limit,
        config_name.clone(),
    )
    .await;
    let amount_in = U256::from_dec_str(amount_in.as_str()).unwrap();
    let conf = config::get_config(config_name);
    let chain_id = 42161;
    let mut routes = Vec::new();
    for path in paths {
        let estimated_amount_out = get_amount_out_from_path(
            &conf.redis_url,
            chain_id,
            conf.json_rpc_url.as_str(),
            amount_in,
            path.as_slice(),
        )
        .await
        .unwrap();
        let route = Route {
            _info: path,
            estimated_amount_out,
        };
        routes.push(route);
    }

    // sort by estimated_amount_out
    routes.sort_by(|a, b| b.estimated_amount_out.cmp(&a.estimated_amount_out));
    println!("{:#?}", routes);
}
