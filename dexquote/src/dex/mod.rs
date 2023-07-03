use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
pub enum Dex {
    UniswapV3,
    UniswapV2,
}

pub struct ParseDexError;

impl FromStr for Dex {
    type Err = ParseDexError;

    fn from_str(dex_str: &str) -> Result<Self, Self::Err> {
        if dex_str == "UNIV3" {
            Ok(Dex::UniswapV3)
        } else if dex_str == "UNIV2" {
            Ok(Dex::UniswapV2)
        } else {
            Err(ParseDexError)
        }
    }
}

impl Dex {
    pub fn as_str(&self) -> &'static str {
        match self {
            Dex::UniswapV3 => "UNIV3",
            Dex::UniswapV2 => "UNIV2",
        }
    }
}
