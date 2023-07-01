use ethers::providers::{Http, Provider};

pub fn get_provider(json_rpc_url: &str) -> Result<Provider<Http>, Box<dyn std::error::Error>> {
    let provider =
        Provider::<Http>::try_from(json_rpc_url).expect("could not instantiate HTTP Provider");
    Ok(provider)
}
