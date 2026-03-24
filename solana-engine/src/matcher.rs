#[derive(Clone, Copy)]
pub enum Position {
    Prefix,
    Suffix,
    Combine,
}

pub fn matches(address: &str, prefix: &str, suffix: &str, position: Position) -> bool {
    match position {
        Position::Suffix  => address.ends_with(suffix),
        Position::Prefix  => address.starts_with(prefix),
        Position::Combine => address.starts_with(prefix) && address.ends_with(suffix),
    }
}