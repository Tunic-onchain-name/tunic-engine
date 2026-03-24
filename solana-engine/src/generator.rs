use crate::crypto;
use crate::matcher::{self, Position};
use serde::Serialize;

#[derive(Serialize)]
pub struct VanityResult {
    pub private_key: String,
    pub private_key_json: String,
    pub address: String,
}

pub fn generate_vanity(prefix: &str, suffix: &str, position: Position) -> VanityResult {
    loop {
        let (address, raw_seed) = crypto::generate_keypair_for_match();

        if matcher::matches(&address, prefix, suffix, position) {
            let (private_key, private_key_json) = crypto::encode_private_key(&raw_seed);
            return VanityResult {
                private_key,
                private_key_json,
                address,
            };
        }
    }
}
