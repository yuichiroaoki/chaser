use dexquote::path::PoolInfo;
use rocket::http::Status;
use rocket::local::blocking::Client;

use crate::quote::Route;

#[test]
fn test_get_path() {
    let token_in = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
    let token_out = "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8";

    let client = Client::tracked(super::rocket()).unwrap();
    let uri = format!("/path/{}/{}", token_in, token_out);
    let response = client.get(uri).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let paths = response.into_json::<Vec<Vec<PoolInfo>>>();
    assert!(paths.is_some());
    println!("{:#?}", paths.unwrap());
}

#[test]
fn test_quote_prices() {
    let token_in = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
    let token_out = "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8";

    let client = Client::tracked(super::rocket()).unwrap();
    let amount_in = "1000000000000";
    let uri = format!("/quote/{}/{}/{}", token_in, token_out, amount_in);
    let response = client.get(uri).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let paths = response.into_json::<Vec<Route>>();
    assert!(paths.is_some());
    println!("{:#?}", paths.unwrap());
}

#[test]
fn test_health_check() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/healthcheck").dispatch();
    assert_eq!(response.into_string(), Some("OK".into()));
}
