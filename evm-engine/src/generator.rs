use crate::crypto;
use crate::matcher::{self, Position};
use serde::Serialize;

#[derive(Serialize)]
pub struct VanityResult {
    pub private_key: String,
    pub address: String,
}

/// Decode a hex string (no 0x prefix) into bytes.
/// Panics on odd-length or invalid hex. Only called once before the loop.
fn decode_hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("invalid hex in pattern"))
        .collect()
}

pub fn generate_vanity(prefix: &str, suffix: &str, position: Position) -> VanityResult {
    // Decode pattern hex strings to byte slices once, outside the loop.
    let prefix_bytes = if !prefix.is_empty() {
        decode_hex(prefix)
    } else {
        vec![]
    };
    let suffix_bytes = if !suffix.is_empty() {
        decode_hex(suffix)
    } else {
        vec![]
    };

    loop {
        let (address_bytes, private_key_bytes) = crypto::generate_keypair_raw();

        let matched = match position {
            Position::Prefix => matcher::matches_bytes(&address_bytes, &prefix_bytes, Position::Prefix),
            Position::Suffix => matcher::matches_bytes(&address_bytes, &suffix_bytes, Position::Suffix),
            Position::Combine => {
                matcher::matches_bytes(&address_bytes, &prefix_bytes, Position::Prefix)
                    && matcher::matches_bytes(&address_bytes, &suffix_bytes, Position::Suffix)
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
