use k256::{ecdsa::SigningKey, SecretKey};
use tiny_keccak::{Hasher, Keccak};
use std::fmt::Write;

fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

pub fn generate_keypair() -> (String, String) {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).expect("Failed to get random bytes");

    let secret_key = SecretKey::from_slice(&seed).expect("Invalid secret key seed");
    let signing_key = SigningKey::from(secret_key.clone());
    let verifying_key = signing_key.verifying_key();

    let uncompressed_pubkey = verifying_key.to_encoded_point(false);
    let pubkey_bytes = uncompressed_pubkey.as_bytes();
    
    // Strip the 0x04 prefix (uncompressed point format)
    let pubkey_no_prefix = &pubkey_bytes[1..];
    
    // Keccak256 hash
    let mut hasher = Keccak::v256();
    hasher.update(pubkey_no_prefix);
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);

    // Take last 20 bytes
    let address_bytes = &hash[12..];
    let address = format!("0x{}", encode_hex(address_bytes));
    
    let private_key = format!("0x{}", encode_hex(&seed));

    (private_key, address)
}
