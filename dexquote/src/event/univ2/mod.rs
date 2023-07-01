use crate::db::univ2::update_pool;
use ethers::abi::ethabi;
use ethers::prelude::*;

pub const UNIV2_SYNC_EVENT_SIG: &str = "Sync(uint112,uint112)";
pub const UNIV2_SWAP_EVENT_SIG: &str = "Swap(address,uint256,uint256,uint256,uint256,address)";

pub struct UniV2SyncEvent {
    pub reserve0: u128,
    pub reserve1: u128,
}

/// Decode a swap event from a log data
/// Returns (amount_in, amount_out, zero_for_one)
pub fn decode_swap_event(log_data: &Bytes) -> Result<(U256, U256, bool), ethabi::Error> {
    let decoded_data = ethabi::decode(
        &[
            ethabi::ParamType::Uint(256),
            ethabi::ParamType::Uint(256),
            ethabi::ParamType::Uint(256),
            ethabi::ParamType::Uint(256),
        ],
        log_data,
    )?;
    let mut decoded = decoded_data.into_iter();
    let amount0_in = decoded.next().unwrap().into_uint().unwrap();
    let amount1_in = decoded.next().unwrap().into_uint().unwrap();
    let amount0_out = decoded.next().unwrap().into_uint().unwrap();
    let amount1_out = decoded.next().unwrap().into_uint().unwrap();
    if amount0_in != U256::zero() {
        Ok((amount0_in, amount1_out, true))
    } else {
        Ok((amount1_in, amount0_out, false))
    }
}

pub fn decode_sync_event(log_data: &Bytes) -> Result<UniV2SyncEvent, ethabi::Error> {
    let decoded_data = ethabi::decode(
        &[ethabi::ParamType::Uint(112), ethabi::ParamType::Uint(112)],
        log_data,
    )?;
    let mut decoded = decoded_data.into_iter();
    let reserve0 = decoded.next().unwrap().into_uint().unwrap();
    let reserve1 = decoded.next().unwrap().into_uint().unwrap();
    Ok(UniV2SyncEvent {
        reserve0: reserve0.as_u128(),
        reserve1: reserve1.as_u128(),
    })
}

pub fn decode_velodrome_sync_event(log_data: &Bytes) -> Result<UniV2SyncEvent, ethabi::Error> {
    let decoded_data = ethabi::decode(
        &[ethabi::ParamType::Uint(256), ethabi::ParamType::Uint(256)],
        log_data,
    )?;
    let mut decoded = decoded_data.into_iter();
    let reserve0 = decoded.next().unwrap().into_uint().unwrap();
    let reserve1 = decoded.next().unwrap().into_uint().unwrap();
    Ok(UniV2SyncEvent {
        reserve0: reserve0.as_u128(),
        reserve1: reserve1.as_u128(),
    })
}

pub fn update_with_sync_event(
    redis_url: &str,
    chain_id: u64,
    pool_address: Address,
    log_data: &Bytes,
) {
    let redis_client = redis::Client::open(redis_url).unwrap();
    let univ2_event = match decode_sync_event(log_data) {
        Ok(event) => event,
        Err(e) => {
            println!("failed to decode swap event: {:?}", e);
            return;
        }
    };
    match update_pool(
        &redis_client,
        chain_id,
        pool_address,
        univ2_event.reserve0,
        univ2_event.reserve1,
    ) {
        Ok(_) => {}
        Err(e) => {
            println!("failed to update pool: {:?}", e);
        }
    };
}

pub fn update_with_velodrome_sync_event(
    redis_url: &str,
    chain_id: u64,
    pool_address: Address,
    log_data: &Bytes,
) {
    let redis_client = redis::Client::open(redis_url).unwrap();
    let univ2_event = match decode_velodrome_sync_event(log_data) {
        Ok(event) => event,
        Err(e) => {
            println!("failed to decode swap event: {:?}", e);
            return;
        }
    };
    match update_pool(
        &redis_client,
        chain_id,
        pool_address,
        univ2_event.reserve0,
        univ2_event.reserve1,
    ) {
        Ok(_) => {}
        Err(e) => {
            println!("failed to update pool: {:?}", e);
        }
    };
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_decode_sync_event_univ2() {
        let log_data = Bytes::from_str("0x000000000000000000000000000000000000000000000091185185b8b0c6056d00000000000000000000000000000000000000000000000000000466159bd113").unwrap();
        let event = decode_sync_event(&log_data).unwrap();
        assert_eq!(event.reserve0, 2676530219446195062125);
        assert_eq!(event.reserve1, 4836495708435);
    }

    #[test]
    fn test_decode_swap_event_univ2() {
        let log_data = Bytes::from_str("0x00000000000000000000000000000000000000000000000000056aa8c74b77ee0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000017f084a07abdf3f6").unwrap();
        let (amount_in, amount_out, zero_for_one) = decode_swap_event(&log_data).unwrap();
        assert_eq!(amount_in, U256::from_dec_str("1524648014215150").unwrap());
        assert_eq!(
            amount_out,
            U256::from_dec_str("1725024482071802870").unwrap()
        );
        assert_eq!(zero_for_one, true);
    }

    #[test]
    fn test_decode_velodrome_sync() {
        let log_data = Bytes::from_str("0x000000000000000000000000000000000000000000005dcf8bbab14a3978f8ba0000000000000000000000000000000000000000000000000000006e8e678f2f").unwrap();
        let sync_event = decode_velodrome_sync_event(&log_data).unwrap();
        assert_eq!(sync_event.reserve0, 443008627484984172148922);
        assert_eq!(sync_event.reserve1, 474835554095);
    }
}
