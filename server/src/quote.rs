use dexquote::path::{get_amount_out_from_path, get_possible_paths, PoolInfo};
use ethers::types::{Address, U256};
use neo4rs::Graph;
use rocket::serde::{json::Json, Deserialize, Serialize};

const NEO4J_URI: &str = "bolt://localhost:7687";
const NEO4J_PASSWORD: &str = "testtest";
const REDIS_URL: &str = "redis://:testtest@127.0.0.1:30073/";

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Route {
    pub path: Vec<PoolInfo>,
    pub estimated_amount_out: String,
}

#[get("/<token_in>/<token_out>/<amount_in>")]
pub async fn quote_prices(
    token_in: String,
    token_out: String,
    amount_in: String,
) -> Json<Vec<Route>> {
    let graph = Graph::new(NEO4J_URI, "neo4j", NEO4J_PASSWORD)
        .await
        .unwrap();
    let token_in = token_in.parse::<Address>().unwrap();
    let token_out = token_out.parse::<Address>().unwrap();
    let amount_in = U256::from_dec_str(&amount_in).unwrap();
    let paths = match get_possible_paths(&graph, token_in, token_out, 2, 5, "Arb").await {
        Some(paths) => paths,
        None => return Json(Vec::new()),
    };
    let chain_id = 42161;
    let alchemy_api_key = std::env::var("ALCHEMY_API_KEY").expect("Could not get ALCHEMY_API_KEY");
    let json_rpc_url = format!(
        "https://arb-mainnet.g.alchemy.com/v2/{}",
        alchemy_api_key.as_str()
    );
    let mut routes = Vec::new();
    for path in paths {
        let estimated_amount_out = get_amount_out_from_path(
            REDIS_URL,
            chain_id,
            &json_rpc_url,
            amount_in,
            path.as_slice(),
        )
        .await
        .unwrap();
        let route = Route {
            path,
            estimated_amount_out: estimated_amount_out.to_string(),
        };
        routes.push(route);
    }

    // sort by estimated_amount_out
    routes.sort_by(|a, b| b.estimated_amount_out.cmp(&a.estimated_amount_out));
    Json(routes)
}
