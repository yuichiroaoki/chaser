use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub json_rpc_url: String,
    pub ws_rpc_url: String,
    pub redis_url: String,
    pub neo4j_uri: String,
    pub neo4j_pass: String,
    pub chain_label: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            json_rpc_url: "".into(),
            ws_rpc_url: "".into(),
            redis_url: "".into(),
            neo4j_uri: "".into(),
            neo4j_pass: "".into(),
            chain_label: "".into(),
        }
    }
}

const APP_NAME: &str = "dexquote";

pub fn get_config(config_name: String) -> Config {
    let cfg: Config = confy::load(APP_NAME, config_name.as_str()).unwrap();
    cfg
}
