use crate::error::DexQuoteError;

/// Library generic result type.
pub type DexQuoteResult<T> = Result<T, DexQuoteError>;
