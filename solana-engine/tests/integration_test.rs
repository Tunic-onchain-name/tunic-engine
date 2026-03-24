#[cfg(test)]
mod tests {
    use engine::crypto;
    use engine::matcher::{self, Position};
    use engine::generator;

    const VALID_BASE58: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    // =========================================================
    // crypto.rs tests (existing)
    // =========================================================

    #[test]
    fn test_keypair_generates_valid_base58_address() {
        let (_, _, address) = crypto::generate_keypair();
        assert!(
            address.chars().all(|c| VALID_BASE58.contains(c)),
            "Address contains invalid Base58 character: {}",
            address
        );
    }

    #[test]
    fn test_keypair_address_length() {
        let (_, _, address) = crypto::generate_keypair();
        assert!(
            address.len() >= 32 && address.len() <= 44,
            "Address length out of expected range: {}",
            address.len()
        );
    }

    #[test]
    fn test_keypair_private_key_is_valid_base58() {
        let (private_key, _, _) = crypto::generate_keypair();
        assert!(
            private_key.chars().all(|c| VALID_BASE58.contains(c)),
            "Private key contains invalid Base58 character: {}",
            private_key
        );
    }

    #[test]
    fn test_keypair_uniqueness() {
        let (pk1, j1, addr1) = crypto::generate_keypair();
        let (pk2, j2, addr2) = crypto::generate_keypair();
        assert_ne!(pk1, pk2, "Two private keys should not be identical");
        assert_ne!(j1, j2, "Two JSON keys should not be identical");
        assert_ne!(addr1, addr2, "Two addresses should not be identical");
    }

    #[test]
    fn test_private_key_and_address_are_different() {
        let (private_key, _, address) = crypto::generate_keypair();
        assert_ne!(
            private_key, address,
            "Private key and address must not be identical"
        );
    }

    // =========================================================
    // crypto.rs tests (new: generate_keypair_for_match / encode_private_key)
    // =========================================================

    #[test]
    fn test_generate_keypair_for_match_returns_valid_base58_address() {
        let (address, _seed) = crypto::generate_keypair_for_match();
        assert!(
            address.chars().all(|c| VALID_BASE58.contains(c)),
            "Address contains invalid Base58 character: {}",
            address
        );
        assert!(
            address.len() >= 32 && address.len() <= 44,
            "Address length out of expected range: {}",
            address.len()
        );
    }

    #[test]
    fn test_generate_keypair_for_match_returns_32_byte_seed() {
        let (_address, seed) = crypto::generate_keypair_for_match();
        assert_eq!(seed.len(), 32);
        // Seed should not be all zeros (astronomically unlikely)
        assert!(seed.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_encode_private_key_produces_valid_base58() {
        let (_address, seed) = crypto::generate_keypair_for_match();
        let (private_key_base58, private_key_json) = crypto::encode_private_key(&seed);

        // Base58 validity
        assert!(
            private_key_base58.chars().all(|c| VALID_BASE58.contains(c)),
            "Encoded private key contains invalid Base58 character: {}",
            private_key_base58
        );

        // JSON array validity: should parse as Vec<u8> of length 64
        let parsed: Vec<u8> = serde_json::from_str(&private_key_json)
            .expect("private_key_json should be valid JSON");
        assert_eq!(parsed.len(), 64, "Full secret should be 64 bytes");
    }

    #[test]
    fn test_encode_private_key_matches_generate_keypair() {
        // Verify that generate_keypair_for_match + encode_private_key
        // produces equivalent results to generate_keypair for the same seed.
        // We cannot control the seed directly, but we can verify structural equivalence.
        let (_address, seed) = crypto::generate_keypair_for_match();
        let (pk_base58, pk_json) = crypto::encode_private_key(&seed);

        // private key base58 should decode to 64 bytes
        let decoded = bs58::decode(&pk_base58).into_vec().expect("valid Base58");
        assert_eq!(decoded.len(), 64);

        // First 32 bytes must match the seed
        assert_eq!(&decoded[..32], &seed);

        // JSON should also decode to 64 bytes with same first 32
        let json_decoded: Vec<u8> = serde_json::from_str(&pk_json).unwrap();
        assert_eq!(&json_decoded[..32], &seed);
    }

    // =========================================================
    // matcher.rs tests
    // =========================================================

    #[test]
    fn test_suffix_match() {
        assert!(matcher::matches("ABC123bagas", "", "bagas", Position::Suffix));
    }

    #[test]
    fn test_suffix_no_match() {
        assert!(!matcher::matches("ABC123bagas", "", "singgih", Position::Suffix));
    }

    #[test]
    fn test_prefix_match() {
        assert!(matcher::matches("bagasABC123XYZ", "bagas", "", Position::Prefix));
    }

    #[test]
    fn test_prefix_no_match() {
        assert!(!matcher::matches("bagasABC123XYZ", "singgih", "", Position::Prefix));
    }

    #[test]
    fn test_combine_match() {
        assert!(matcher::matches("bagasABC123bagas", "bagas", "bagas", Position::Combine));
    }

    #[test]
    fn test_combine_requires_both_ends() {
        assert!(!matcher::matches("XYZABCbagas", "bagas", "bagas", Position::Combine));
        assert!(!matcher::matches("bagasXYZABC", "bagas", "bagas", Position::Combine));
    }

    #[test]
    fn test_case_sensitive_suffix() {
        assert!(matcher::matches("ABC123Bagas", "", "Bagas", Position::Suffix));
        assert!(!matcher::matches("ABC123Bagas", "", "bagas", Position::Suffix));
    }

    #[test]
    fn test_case_sensitive_prefix() {
        assert!(matcher::matches("BagasABC123", "Bagas", "", Position::Prefix));
        assert!(!matcher::matches("BagasABC123", "bagas", "", Position::Prefix));
    }

    #[test]
    fn test_case_sensitive_combine() {
        assert!(matcher::matches("BagasABC123Bagas", "Bagas", "Bagas", Position::Combine));
        assert!(!matcher::matches("BagasABC123Bagas", "bagas", "bagas", Position::Combine));
    }

    #[test]
    fn test_empty_pattern_always_matches_suffix() {
        assert!(matcher::matches("SomeAddress123", "", "", Position::Suffix));
    }

    #[test]
    fn test_empty_pattern_always_matches_prefix() {
        assert!(matcher::matches("SomeAddress123", "", "", Position::Prefix));
    }

    #[test]
    fn test_pattern_longer_than_address() {
        assert!(!matcher::matches("abc", "abcdefghijklmno", "abcdefghijklmno", Position::Suffix));
        assert!(!matcher::matches("abc", "abcdefghijklmno", "abcdefghijklmno", Position::Prefix));
        assert!(!matcher::matches("abc", "abcdefghijklmno", "abcdefghijklmno", Position::Combine));
    }

    #[test]
    fn test_no_0x_prefix_in_solana_address() {
        let address = "bagas9ABCDEFGHJKLMNPQRSTUVWXYZbagas";
        assert!(matcher::matches(address, "bagas", "bagas", Position::Combine));
    }

    #[test]
    fn test_invalid_base58_chars_not_present() {
        let (_, _, address) = crypto::generate_keypair();
        assert!(!address.contains('0'), "Address should not contain '0'");
        assert!(!address.contains('O'), "Address should not contain 'O'");
        assert!(!address.contains('I'), "Address should not contain 'I'");
        assert!(!address.contains('l'), "Address should not contain 'l'");
    }

    // =========================================================
    // generator.rs tests
    // =========================================================

    #[test]
    fn test_generate_vanity_suffix() {
        let result = generator::generate_vanity("", "ab", Position::Suffix);
        assert!(
            result.address.ends_with("ab"),
            "Address should end with 'ab', got: {}",
            result.address
        );
    }

    #[test]
    fn test_generate_vanity_prefix() {
        let result = generator::generate_vanity("Ab", "", Position::Prefix);
        assert!(
            result.address.starts_with("Ab"),
            "Address should start with 'Ab', got: {}",
            result.address
        );
    }

    #[test]
    fn test_generate_vanity_combine() {
        let result = generator::generate_vanity("ab", "ab", Position::Combine);
        assert!(
            result.address.starts_with("ab") && result.address.ends_with("ab"),
            "Address should start and end with 'ab', got: {}",
            result.address
        );
    }

    #[test]
    fn test_generate_vanity_result_has_valid_address() {
        let result = generator::generate_vanity("", "a", Position::Suffix);
        assert!(
            result.address.chars().all(|c| VALID_BASE58.contains(c)),
            "Generated address contains invalid Base58 character: {}",
            result.address
        );
    }

    #[test]
    fn test_generate_vanity_result_has_valid_private_key() {
        let result = generator::generate_vanity("", "a", Position::Suffix);
        assert!(
            result.private_key.chars().all(|c| VALID_BASE58.contains(c)),
            "Generated private key contains invalid Base58 character: {}",
            result.private_key
        );
    }

    #[test]
    fn test_generate_vanity_result_has_valid_private_key_json() {
        let result = generator::generate_vanity("", "a", Position::Suffix);
        let parsed: Vec<u8> = serde_json::from_str(&result.private_key_json)
            .expect("private_key_json should be valid JSON");
        assert_eq!(parsed.len(), 64, "Full secret in JSON should be 64 bytes");
    }

    #[test]
    fn test_generate_vanity_address_and_private_key_differ() {
        let result = generator::generate_vanity("", "a", Position::Suffix);
        assert_ne!(
            result.address, result.private_key,
            "Address and private key must not be identical"
        );
    }

    #[test]
    fn test_generate_vanity_case_sensitive_pattern() {
        let result = generator::generate_vanity("", "AB", Position::Suffix);
        assert!(
            result.address.ends_with("AB"),
            "Address should end with 'AB' (uppercase), got: {}",
            result.address
        );
        assert!(
            !result.address.ends_with("ab"),
            "Address should not end with 'ab' (lowercase) when pattern is 'AB'"
        );
    }

    #[test]
    fn test_generate_vanity_address_matches_pattern_post_refactor() {
        // Verify the refactored loop still produces correct matches
        let result = generator::generate_vanity("B", "x", Position::Combine);
        assert!(
            result.address.starts_with("B"),
            "Address should start with 'B', got: {}",
            result.address
        );
        assert!(
            result.address.ends_with("x"),
            "Address should end with 'x', got: {}",
            result.address
        );
    }
}