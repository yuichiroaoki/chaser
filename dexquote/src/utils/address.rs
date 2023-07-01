use ethers::{types::Address, utils::hex};

pub fn address_str(address: Address) -> String {
    hex::encode(address.as_bytes())
}
