use ethers::types::U256;

const MIN_SQRT_RATIO: &str = "4295128739";
const MAX_SQRT_RATIO: &str = "1461446703485210103287273052203988822378723970342";

pub fn get_sqrt_price_limit_x96(zero_for_one: bool) -> U256 {
    if zero_for_one {
        U256::from_dec_str(MIN_SQRT_RATIO).unwrap() + U256::one()
    } else {
        U256::from_dec_str(MAX_SQRT_RATIO).unwrap() - U256::one()
    }
}
