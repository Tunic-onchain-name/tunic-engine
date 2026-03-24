use wasm_bindgen::prelude::*;
use crate::matcher::Position;

pub mod crypto;
pub mod generator;
pub mod matcher;

pub fn is_valid_hex_pattern(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit()) && s.len() <= 40
}

#[wasm_bindgen]
pub fn generate_vanity(prefix: &str, suffix: &str, position_str: &str) -> JsValue {
    if !is_valid_hex_pattern(prefix) || !is_valid_hex_pattern(suffix) {
        return JsValue::from_str(r#"{"error":"Invalid hex pattern (0-9, a-f, max 40 chars)"}"#);
    }

    let position = match position_str {
        "prefix" => Position::Prefix,
        "suffix" => Position::Suffix,
        "combine" => Position::Combine,
        _ => Position::Suffix,
    };
    
    let result = generator::generate_vanity(prefix, suffix, position);
    let json_str = serde_json::to_string(&result).unwrap();
    JsValue::from_str(&json_str)
}
