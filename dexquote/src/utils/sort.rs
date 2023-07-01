use ethers::prelude::*;

pub fn sort_tokens(token_a: Address, token_b: Address) -> (Address, Address) {
    if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    }
}

#[test]
fn test_sort_tokens() {
    let usdc = "0x2791bca1f2de4661ed88a30c99a7a9449aa84174"
        .parse::<Address>()
        .unwrap();
    let weth = "0x7ceb23fd6bc0add59e62ac25578270cff1b9f619"
        .parse::<Address>()
        .unwrap();
    assert!(sort_tokens(usdc, weth).0 == usdc);
    assert!(sort_tokens(usdc, weth).1 == weth);
}
