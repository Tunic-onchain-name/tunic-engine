use crate::crypto;
use crate::matcher::{self, Position};
use serde::Serialize;

#[derive(Serialize)]
pub struct VanityResult {
    pub private_key: String,
    pub address: String,
}

pub fn generate_vanity(prefix: &str, suffix: &str, position: Position) -> VanityResult {
    loop {
        let (private_key, address) = crypto::generate_keypair();
        if matcher::matches(&address, prefix, suffix, position) {
            return VanityResult {
                private_key,
                address,
            };
        }
    }
}
