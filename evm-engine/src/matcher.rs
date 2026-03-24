#[derive(Clone, Copy)]
pub enum Position {
    Prefix,
    Suffix,
    Combine,
}

pub fn matches(address: &str, prefix: &str, suffix: &str, position: Position) -> bool {
    let address_lower = address.to_lowercase();
    let prefix_lower = prefix.to_lowercase();
    let suffix_lower = suffix.to_lowercase();
    
    let address_no_prefix = if address_lower.starts_with("0x") {
        &address_lower[2..]
    } else {
        &address_lower
    };

    match position {
        Position::Suffix => address_lower.ends_with(&suffix_lower),
        Position::Prefix => address_no_prefix.starts_with(&prefix_lower),
        Position::Combine => address_no_prefix.starts_with(&prefix_lower) && address_lower.ends_with(&suffix_lower),
    }
}

/// Nibble-level pattern match against raw address bytes.
/// Each hex character in the pattern is 1 nibble (4 bits).
pub fn matches_nibbles(address_bytes: &[u8; 20], pattern_nibbles: &[u8], position: Position) -> bool {
    let len = pattern_nibbles.len();
    if len == 0 || len > 40 {
        return false;
    }

    match position {
        Position::Prefix => {
            for i in 0..len {
                let addr_byte = address_bytes[i / 2];
                let addr_nibble = if i % 2 == 0 {
                    addr_byte >> 4
                } else {
                    addr_byte & 0x0F
                };
                if addr_nibble != pattern_nibbles[i] {
                    return false;
                }
            }
            true
        }
        Position::Suffix => {
            for i in 0..len {
                // Suffix matching works backwards from the end of the address.
                // pattern_nibbles[0] is the FIRST char of the suffix string.
                // For suffix "abc", pattern_nibbles is [a, b, c].
                // We want address to end with "abc".
                // Last nibble of address (index 39) must be 'c' (pattern_nibbles[2]).
                // Second to last (38) must be 'b' (pattern_nibbles[1]).
                let addr_nibble_idx = 40 - len + i;
                let addr_byte = address_bytes[addr_nibble_idx / 2];
                let addr_nibble = if addr_nibble_idx % 2 == 0 {
                    addr_byte >> 4
                } else {
                    addr_byte & 0x0F
                };
                if addr_nibble != pattern_nibbles[i] {
                    return false;
                }
            }
            true
        }
        Position::Combine => {
            // This expects separate prefix/suffix checks which should be handled by the caller
            // or we could split pattern_nibbles. But generator.rs already handles this.
            false 
        }
    }
}
