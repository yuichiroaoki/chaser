use dexquote::path::{get_possible_paths, PoolInfo};
use ethers::types::Address;
use neo4rs::Graph;
use rocket::serde::json::Json;

const NEO4J_URI: &str = "bolt://localhost:7687";
const NEO4J_PASSWORD: &str = "testtest";

#[get("/<token_in>/<token_out>")]
pub async fn get_path(token_in: String, token_out: String) -> Json<Vec<Vec<PoolInfo>>> {
    let graph = Graph::new(NEO4J_URI, "neo4j", NEO4J_PASSWORD)
        .await
        .unwrap();
    let token_in = token_in.parse::<Address>().unwrap();
    let token_out = token_out.parse::<Address>().unwrap();
    match get_possible_paths(&graph, token_in, token_out, 1, 5, "Arb").await {
        Some(paths) => Json(paths),
        None => Json(Vec::new()),
    }
}
