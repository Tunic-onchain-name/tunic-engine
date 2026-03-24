use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

pub mod generator;
pub mod crypto;
pub mod matcher;

pub use crate::matcher::Position;


#[wasm_bindgen]
pub fn generate_vanity(prefix: &str, suffix: &str, position_str: &str) -> JsValue {
    let valid_base58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    
    if !prefix.chars().all(|c| valid_base58.contains(c)) || 
       !suffix.chars().all(|c| valid_base58.contains(c)) {
        return JsValue::from_str(r#"{"error":"Invalid Base58 pattern"}"#);
    }

    let position = match position_str {
        "prefix"  => Position::Prefix,
        "suffix"  => Position::Suffix,
        "combine" => Position::Combine,
        _         => Position::Suffix,
    };

    let result = generator::generate_vanity(prefix, suffix, position);
    JsValue::from_str(&serde_json::to_string(&result).unwrap())
}