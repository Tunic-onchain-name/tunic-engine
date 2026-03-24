use crate::crypto;
use crate::matcher::{self, Position};
use serde::Serialize;

#[derive(Serialize)]
pub struct VanityResult {
    pub private_key: String,
    pub address: String,
}

/// Decode a hex string into a vector of nibbles (0-15).
fn decode_to_nibbles(s: &str) -> Vec<u8> {
    s.chars()
        .map(|c| c.to_digit(16).expect("invalid hex in pattern") as u8)
        .collect()
}

pub fn generate_vanity(prefix: &str, suffix: &str, position: Position) -> VanityResult {
    let prefix_nibbles = decode_to_nibbles(prefix);
    let suffix_nibbles = decode_to_nibbles(suffix);

    loop {
        let (address_bytes, private_key_bytes) = crypto::generate_keypair_raw();

        let matched = match position {
            Position::Prefix => matcher::matches_nibbles(&address_bytes, &prefix_nibbles, Position::Prefix),
            Position::Suffix => matcher::matches_nibbles(&address_bytes, &suffix_nibbles, Position::Suffix),
            Position::Combine => {
                matcher::matches_nibbles(&address_bytes, &prefix_nibbles, Position::Prefix)
                    && matcher::matches_nibbles(&address_bytes, &suffix_nibbles, Position::Suffix)
            }
        };

        if matched {
            let (private_key, address) = crypto::encode_result(&address_bytes, &private_key_bytes);
            return VanityResult {
                private_key,
                address,
            };
        }
    }
}
