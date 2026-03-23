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
