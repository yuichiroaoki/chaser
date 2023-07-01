// ref. https://github.com/Uniswap/v3-sdk/blob/main/src/constants.ts
pub fn get_tick_spacing(fee: u32) -> i32 {
    match fee {
        100 => 1,
        500 => 10,
        3000 => 60,
        10000 => 200,
        _ => panic!("invalid fee"),
    }
}
