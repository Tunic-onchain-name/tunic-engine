use wasm_bindgen::prelude::*;
use crate::matcher::Position;

pub mod crypto;
pub mod generator;
pub mod matcher;

#[wasm_bindgen]
pub fn generate_vanity(prefix: &str, suffix: &str, position_str: &str) -> JsValue {
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
