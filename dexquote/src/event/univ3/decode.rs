use super::{UniV3MintEvent, UniV3SwapEvent};
use ethers::{
    abi::ethabi,
    types::{Bytes, I256},
};

pub fn decode_swap_event(log_data: &Bytes) -> Result<UniV3SwapEvent, ethabi::Error> {
    let decoded_data = ethabi::decode(
        &[
            ethabi::ParamType::Int(256),
            ethabi::ParamType::Int(256),
            ethabi::ParamType::Uint(160),
            ethabi::ParamType::Uint(128),
            ethabi::ParamType::Int(24),
        ],
        log_data,
    )?;
    let mut decoded = decoded_data.into_iter();
    let amount0 = I256::from_raw(decoded.next().unwrap().into_int().unwrap());
    let amount1 = I256::from_raw(decoded.next().unwrap().into_int().unwrap());
    let sqrt_price_x96 = decoded.next().unwrap().into_uint().unwrap();
    let liquidity = decoded.next().unwrap().into_uint().unwrap().as_u128();
    let tick = I256::from_raw(decoded.next().unwrap().into_int().unwrap()).as_i32();
    Ok(UniV3SwapEvent {
        amount0,
        amount1,
        sqrt_price_x96,
        liquidity,
        tick,
    })
}

pub fn decode_mint_event(log_data: &Bytes) -> Result<UniV3MintEvent, ethabi::Error> {
    let decoded_data = ethabi::decode(
        &[
            ethabi::ParamType::Address,
            ethabi::ParamType::Uint(128),
            ethabi::ParamType::Uint(256),
            ethabi::ParamType::Uint(256),
        ],
        log_data,
    )?;
    let mut decoded = decoded_data.into_iter();
    decoded.next();
    let amount = decoded.next().unwrap().into_uint().unwrap().as_u128();
    let amount0 = decoded.next().unwrap().into_uint().unwrap();
    let amount1 = decoded.next().unwrap().into_uint().unwrap();
    Ok(UniV3MintEvent {
        amount,
        amount0,
        amount1,
    })
}

pub fn decode_burn_event(log_data: &Bytes) -> Result<UniV3MintEvent, ethabi::Error> {
    let decoded_data = ethabi::decode(
        &[
            ethabi::ParamType::Uint(128),
            ethabi::ParamType::Uint(256),
            ethabi::ParamType::Uint(256),
        ],
        log_data,
    )?;
    let mut decoded = decoded_data.into_iter();
    let amount = decoded.next().unwrap().into_uint().unwrap().as_u128();
    let amount0 = decoded.next().unwrap().into_uint().unwrap();
    let amount1 = decoded.next().unwrap().into_uint().unwrap();
    Ok(UniV3MintEvent {
        amount,
        amount0,
        amount1,
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethers::types::U256;

    use super::*;

    #[test]
    fn test_decode_swap_event_univ3() {
        let log_data = Bytes::from_str("0x000000000000000000000000000000000000000000000004bc383746a93b3165ffffffffffffffffffffffffffffffffffffffffffffffca3b3880a2a34393c900000000000000000000000000000000000000035fe33d63a8900a892110d31200000000000000000000000000000000000000000001d080f958306fa0b3922e0000000000000000000000000000000000000000000000000000000000005f06").unwrap();
        let event_data = decode_swap_event(&log_data).unwrap();
        assert_eq!(
            event_data.amount0,
            I256::from_dec_str("87349627349290922341").unwrap()
        );
        assert_eq!(
            event_data.amount1,
            I256::from_dec_str("-991856877897370070071").unwrap()
        );
    }

    #[test]
    fn test_decode_swap_event_alg() {
        let log_data = Bytes::from_str("0xfffffffffffffffffffffffffffffffffffffffffffffffffffb20edc9a253d40000000000000000000000000000000000000000000000001e2b00b77d4d8b500000000000000000000000000000000000000027cea3d4b98d30c252eda9edab00000000000000000000000000000000000000000000057d9c78a9913e31c4830000000000000000000000000000000000000000000000000000000000011fd4").unwrap();
        let event_data = decode_swap_event(&log_data).unwrap();
        assert_eq!(
            event_data.amount0,
            I256::from_dec_str("-1371169221356588").unwrap()
        );
        assert_eq!(
            event_data.amount1,
            I256::from_dec_str("2173832033217645392").unwrap()
        );
        assert_eq!(
            event_data.sqrt_price_x96,
            U256::from_dec_str("3153850309552619302081708813739").unwrap()
        );
        assert_eq!(event_data.liquidity, 25928950371670320858243);
        assert_eq!(event_data.tick, 73684);
    }

    #[test]
    fn test_decode_mint_event() {
        let log_data = Bytes::from_str("0x000000000000000000000000c36442b4a4522e871399cd717abdd847ab11fe8800000000000000000000000000000000000000000000000000eca5e1816a2e68000000000000000000000000000000000000000000000002af3915adb62db2200000000000000000000000000000000000000000000000000000000450041b1d").unwrap();
        let event_data = decode_mint_event(&log_data).unwrap();
        assert_eq!(event_data.amount, 66610482461159016);
        assert_eq!(
            event_data.amount0,
            U256::from_dec_str("49519635013558972960").unwrap()
        );
        assert_eq!(
            event_data.amount1,
            U256::from_dec_str("18522315549").unwrap()
        );
    }

    #[test]
    fn test_decode_burn_event() {
        // https://arbiscan.io/tx/0x763c9ac33dfb2df31f58b377030b203a38a4c60d27036d626f5af884255a47e7#eventlogẅ
        let log_data = Bytes::from_str("0x00000000000000000000000000000000000000000000000000ab1aeb98c6cb6d00000000000000000000000000000000000000000000007147e0e0b5a74e24ab0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let event_data = decode_burn_event(&log_data).unwrap();
        assert_eq!(event_data.amount, 48161820200323949);
        assert_eq!(
            event_data.amount0,
            U256::from_dec_str("2089661466971456021675").unwrap()
        );
        assert_eq!(event_data.amount1, U256::from_dec_str("0").unwrap());
    }
}
