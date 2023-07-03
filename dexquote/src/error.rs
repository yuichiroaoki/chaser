use ethers::types::Address;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DexQuoteError {
    #[error("redis error: {0:?}")]
    RedisError(#[from] redis::RedisError),
    #[error("pool not found: {0:?}")]
    PoolNotFound(Address),
    #[error("pool id not found: {0}")]
    PoolIdNotFound(String),
    #[error("{0}")]
    GetPriceError(String),
    #[error("{0}")]
    MathError(String),
    #[error("invalid fee: {0}")]
    InvalidFee(u32),
    #[error("invalid dex: {0}")]
    InvalidDex(String),
}
