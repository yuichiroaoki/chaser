use super::tick_bitmap;
use crate::constants::{sqrt_p::get_sqrt_price_limit_x96, tick_spacing::get_tick_spacing};
use crate::db::univ3::get_pool;
use crate::db::univ3::get_ticks_and_update_if_necessary;
use crate::types::DexQuoteResult;
use cfmms::pool::Pool;
use ethers::core::types::{Address, I256, U256};
use uniswap_v3_math::{liquidity_math, swap_math, tick_math};

#[derive(Clone, Copy, Debug)]
struct Slot0 {
    // the current price
    sqrt_price_x96: U256,
    // the current tick
    tick: i32,
    fee_protocol: u8,
}

pub struct PoolState {
    redis_client: redis::Client,
    chain_id: u64,
    json_rpc_url: String,
    pub token0: Address,
    pub token1: Address,
    pool_address: Address,
    slot0: Slot0,
    liquidity: u128,
    tick_spacing: i32,
    pub fee: u32,
}

struct SwapCache {
    liquidity_start: u128,
    // block_timestamp: U256,
    fee_protocol: u8,
}

// the top level state of the swap, the results of which are recorded in storage at the end
pub struct SwapState {
    // the amount remaining to be swapped in/out of the input/output asset
    amount_specified_remaining: I256,
    // the amount already swapped out/in of the output/input asset
    amount_calculated: I256,
    // current sqrt(price)
    sqrt_price_x96: U256,
    // the tick associated with the current price
    tick: i32,
    // the current liquidity in range
    liquidity: u128,
}

fn get_state(
    amount_specified: I256,
    liquidity: u128,
    sqrt_price_limit_x96: U256,
    tick: i32,
) -> SwapState {
    SwapState {
        amount_specified_remaining: amount_specified,
        amount_calculated: 0.into(),
        sqrt_price_x96: sqrt_price_limit_x96,
        // sqrt_price_x96: slot0Start.sqrt_price_x96,
        tick,
        liquidity,
    }
}

struct StepComputations {
    // the price at the beginning of the step
    sqrt_price_start_x96: U256,
    // the next tick to swap to from the current tick in the swap direction
    tick_next: i32,
    // whether tick_next is initialized or not
    initialized: bool,
    // sqrt(price) for the next tick (1/0)
    sqrt_price_next_x96: U256,
    // how much is being swapped in in this step
    amount_in: U256,
    // how much is being swapped out
    amount_out: U256,
    // how much fee is being paid in
    fee_amount: U256,
}

impl PoolState {
    pub fn init(
        redis_url: &str,
        chain_id: u64,
        json_rpc_url: String,
        pool_address: Address,
    ) -> DexQuoteResult<Option<Self>> {
        let redis_client = redis::Client::open(redis_url).unwrap();
        let pool_state = get_pool(&redis_client, chain_id, pool_address)?;
        match pool_state {
            Some(Pool::UniswapV3(pool_state)) => {
                let fee = pool_state.fee;
                let tick_spacing = get_tick_spacing(fee)?;
                let slot0 = Slot0 {
                    sqrt_price_x96: pool_state.sqrt_price,
                    tick: pool_state.tick,
                    fee_protocol: 0,
                };
                let token0 = pool_state.token_a;
                let token1 = pool_state.token_b;
                Ok(Some(Self {
                    redis_client,
                    chain_id,
                    json_rpc_url,
                    token0,
                    token1,
                    pool_address,
                    liquidity: pool_state.liquidity,
                    slot0,
                    tick_spacing,
                    fee,
                }))
            }
            _ => Ok(None),
        }
    }

    fn delta_to_amount(&self, amount0: I256, amount1: I256) -> U256 {
        if amount0 > I256::zero() {
            U256::try_from(-amount1).unwrap()
        } else {
            U256::try_from(-amount0).unwrap()
        }
    }

    pub async fn update_state(
        &mut self,
        amount_specified: I256,
        zero_for_one: bool,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let (amount_out, state, is_tick_changed, is_liquidity_changed) =
            self.get_price(amount_specified, zero_for_one).await?;
        // update pool state
        // ref. https://github.com/Uniswap/v3-core/blob/05c10bf6d547d6121622ac51c457f93775e1df09/contracts/UniswapV3Pool.sol#L734-L756
        if is_tick_changed {
            self.slot0.sqrt_price_x96 = state.sqrt_price_x96;
            self.slot0.tick = state.tick;
        }
        if is_liquidity_changed {
            self.liquidity = state.liquidity;
        }
        Ok(amount_out)
    }

    pub async fn get_price(
        &self,
        amount_specified: I256,
        zero_for_one: bool,
    ) -> Result<(U256, SwapState, bool, bool), Box<dyn std::error::Error>> {
        let sqrt_price_limit_x96 = get_sqrt_price_limit_x96(zero_for_one);
        let slot0_start = self.slot0;

        let cache = SwapCache {
            liquidity_start: self.liquidity,
            // blockTimestamp: _blockTimestamp(),
            fee_protocol: if zero_for_one {
                slot0_start.fee_protocol % 16
            } else {
                slot0_start.fee_protocol >> 4
            },
        };

        let mut state = get_state(
            amount_specified,
            self.liquidity,
            slot0_start.sqrt_price_x96,
            slot0_start.tick,
        );

        // continue swapping as long as we haven't used the entire input/output and haven't reached the price limit
        while state.amount_specified_remaining != I256::zero()
            && state.sqrt_price_x96 != sqrt_price_limit_x96
        {
            let mut step = StepComputations {
                sqrt_price_start_x96: state.sqrt_price_x96,
                tick_next: 0,
                initialized: false,
                sqrt_price_next_x96: 0.into(),
                amount_in: 0.into(),
                amount_out: 0.into(),
                fee_amount: 0.into(),
            };

            (step.tick_next, step.initialized) =
                tick_bitmap::next_initialized_tick_within_one_word(
                    &self.redis_client,
                    self.chain_id,
                    &self.json_rpc_url,
                    self.pool_address,
                    state.tick,
                    self.tick_spacing,
                    zero_for_one,
                )
                .await?;

            // ensure that we do not overshoot the min/max tick, as the tick bitmap is not aware of these bounds
            step.tick_next = step
                .tick_next
                .clamp(tick_math::MIN_TICK, tick_math::MAX_TICK);

            // get the price for the next tick
            step.sqrt_price_next_x96 = tick_math::get_sqrt_ratio_at_tick(step.tick_next)?;

            // compute values to swap to the target tick, price limit, or point where input/output amount is exhausted
            let sqrt_ratio_target_x96 = if zero_for_one {
                if step.sqrt_price_next_x96 < sqrt_price_limit_x96 {
                    sqrt_price_limit_x96
                } else {
                    step.sqrt_price_next_x96
                }
            } else if step.sqrt_price_next_x96 > sqrt_price_limit_x96 {
                sqrt_price_limit_x96
            } else {
                step.sqrt_price_next_x96
            };

            (
                state.sqrt_price_x96,
                step.amount_in,
                step.amount_out,
                step.fee_amount,
            ) = swap_math::compute_swap_step(
                state.sqrt_price_x96,
                sqrt_ratio_target_x96,
                state.liquidity,
                state.amount_specified_remaining,
                self.fee,
            )?;

            state.amount_specified_remaining -= I256::try_from(step.amount_in + step.fee_amount)?;
            state.amount_calculated -= I256::try_from(step.amount_out)?;
            if state.amount_specified_remaining == I256::zero() {
                break;
            };

            // if the protocol fee is on, calculate how much is owed, decrement fee_amount, and increment protocolFee
            if cache.fee_protocol > 0 {
                let delta = step.fee_amount / cache.fee_protocol;
                step.fee_amount -= delta;
                // state.protocolFee += uint128(delta);
            }

            // shift tick if we reached the next price
            if state.sqrt_price_x96 == step.sqrt_price_next_x96 {
                // if the tick is initialized, run the tick transition
                if step.initialized {
                    let mut liquidity_net = get_ticks_and_update_if_necessary(
                        &self.redis_client,
                        self.chain_id,
                        self.pool_address,
                        step.tick_next,
                        &self.json_rpc_url,
                    )
                    .await?;
                    // if we're moving leftward, we interpret liquidity_net as the opposite sign
                    // safe because liquidity_net cannot be type(int128).min
                    if zero_for_one {
                        liquidity_net = -liquidity_net;
                    }

                    state.liquidity = liquidity_math::add_delta(state.liquidity, liquidity_net)?;
                }

                state.tick = if zero_for_one {
                    step.tick_next - 1
                } else {
                    step.tick_next
                };
            } else if state.sqrt_price_x96 != step.sqrt_price_start_x96 {
                // recompute unless we're on a lower tick boundary (i.e. already transitioned ticks), and haven't moved
                state.tick = tick_math::get_tick_at_sqrt_ratio(state.sqrt_price_x96)?;
            }
        }
        let is_tick_changed = state.tick != slot0_start.tick;
        let is_liquidity_changed = cache.liquidity_start != state.liquidity;

        let (amount0, amount1) = if zero_for_one {
            (
                amount_specified - state.amount_specified_remaining,
                state.amount_calculated,
            )
        } else {
            (
                state.amount_calculated,
                amount_specified - state.amount_specified_remaining,
            )
        };
        let amount_out = self.delta_to_amount(amount0, amount1);
        Ok((amount_out, state, is_tick_changed, is_liquidity_changed))
    }
}
