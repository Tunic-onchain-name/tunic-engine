use ed25519_dalek::SigningKey;
use bs58;

pub fn generate_keypair() -> (String, String, String) {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).expect("Failed to get random bytes");

    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();

    let mut full_secret = [0u8; 64];
    full_secret[0..32].copy_from_slice(&seed);
    full_secret[32..64].copy_from_slice(verifying_key.as_bytes());

    let address = bs58::encode(verifying_key.as_bytes()).into_string();
    let private_key_base58 = bs58::encode(&full_secret).into_string();
    let private_key_json = serde_json::to_string(&full_secret.to_vec()).unwrap();

    (private_key_base58, private_key_json, address)
}

/// Returns Base58 address (for matching) and raw 32-byte seed.
/// No private key encoding — designed for the hot brute-force loop.
pub fn generate_keypair_for_match() -> (String, [u8; 32]) {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).expect("Failed to get random bytes");

    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();

    let address = bs58::encode(verifying_key.as_bytes()).into_string();

    (address, seed)
}

/// Encodes raw 32-byte seed to Base58 private key and JSON array.
/// Called once after a match is confirmed.
pub fn encode_private_key(raw: &[u8; 32]) -> (String, String) {
    let signing_key = SigningKey::from_bytes(raw);
    let verifying_key = signing_key.verifying_key();

    let mut full_secret = [0u8; 64];
    full_secret[0..32].copy_from_slice(raw);
    full_secret[32..64].copy_from_slice(verifying_key.as_bytes());

    let private_key_base58 = bs58::encode(&full_secret).into_string();
    let private_key_json = serde_json::to_string(&full_secret.to_vec()).unwrap();

    (private_key_base58, private_key_json)
}