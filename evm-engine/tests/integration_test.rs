use engine::crypto;
use engine::matcher::{self, Position};

// ---------------------------------------------------------------------------
// Existing string-based tests (fixed to match current matches() signature)
// ---------------------------------------------------------------------------

#[test]
fn test_match_logic() {
    let addr = "0x1234567890abcdef1234567890abcdef12345678";

    // Prefix match
    assert!(matcher::matches(addr, "1234", "", Position::Prefix));
    assert!(matcher::matches(addr, "12345", "", Position::Prefix));
    assert!(!matcher::matches(addr, "abcd", "", Position::Prefix));

    // Suffix match
    assert!(matcher::matches(addr, "", "5678", Position::Suffix));
    assert!(matcher::matches(addr, "", "abcdef12345678", Position::Suffix));
    assert!(!matcher::matches(addr, "", "1234", Position::Suffix));

    // Case insensitive matching
    let mixed_addr = "0xAbCdEf7890123456789012345678901234567890";
    assert!(matcher::matches(mixed_addr, "abcdef", "", Position::Prefix));
    assert!(matcher::matches(mixed_addr, "", "567890", Position::Suffix));

    // Without prefix 0x
    let raw_addr = "1234567890abcdef1234567890abcdef12345678";
    assert!(matcher::matches(raw_addr, "1234", "", Position::Prefix));
}

#[test]
fn test_keypair_generation() {
    let (priv_key, addr) = crypto::generate_keypair();

    assert!(priv_key.starts_with("0x"));
    assert_eq!(priv_key.len(), 66); // 0x + 64 hex chars

    assert!(addr.starts_with("0x"));
    assert_eq!(addr.len(), 42); // 0x + 40 hex chars
}

#[test]
fn test_keypair_determinism() {
    let (_, addr) = crypto::generate_keypair();
    let hex_part = &addr[2..];
    for c in hex_part.chars() {
        assert!(c.is_ascii_hexdigit());
        assert!(!c.is_uppercase());
    }
}

// ---------------------------------------------------------------------------
// New byte-level tests
// ---------------------------------------------------------------------------

#[test]
fn test_generate_keypair_raw_returns_20_byte_address() {
    let (address_bytes, private_key_bytes) = crypto::generate_keypair_raw();
    assert_eq!(address_bytes.len(), 20);
    assert_eq!(private_key_bytes.len(), 32);

    // Verify no accidental all-zero output
    assert!(address_bytes.iter().any(|&b| b != 0));
    assert!(private_key_bytes.iter().any(|&b| b != 0));
}

#[test]
fn test_matches_bytes_suffix() {
    let mut addr = [0u8; 20];
    // Set last 2 bytes to 0x35, 0x04
    addr[18] = 0x35;
    addr[19] = 0x04;

    let pattern = vec![0x3, 0x5, 0x0, 0x4];
    assert!(matcher::matches_nibbles(&addr, &pattern, Position::Suffix));

    let bad_pattern = vec![0x0, 0x4, 0x3, 0x5];
    assert!(!matcher::matches_nibbles(&addr, &bad_pattern, Position::Suffix));
}

#[test]
fn test_matches_bytes_prefix() {
    let mut addr = [0u8; 20];
    addr[0] = 0xab;
    addr[1] = 0xcd;

    let pattern = vec![0xa, 0xb, 0xc, 0xd];
    assert!(matcher::matches_nibbles(&addr, &pattern, Position::Prefix));

    let bad_pattern = vec![0xa, 0xb, 0xc, 0xe];
    assert!(!matcher::matches_nibbles(&addr, &bad_pattern, Position::Prefix));
}

#[test]
fn test_matches_bytes_combine() {
    let mut addr = [0u8; 20];
    // Same 2-byte pattern at both ends
    addr[0] = 0xaa;
    addr[1] = 0xbb;
    addr[18] = 0xaa;
    addr[19] = 0xbb;

    let pattern = vec![0xa, 0xa, 0xb, 0xb];
    assert!(matcher::matches_nibbles(&addr, &pattern, Position::Prefix) 
            && matcher::matches_nibbles(&addr, &pattern, Position::Suffix));

    // Break suffix
    addr[19] = 0xcc;
    assert!(!(matcher::matches_nibbles(&addr, &pattern, Position::Prefix) 
            && matcher::matches_nibbles(&addr, &pattern, Position::Suffix)));
}

#[test]
fn test_matches_bytes_edge_cases() {
    let addr = [0xff; 20];

    // Empty pattern returns false
    assert!(!matcher::matches_nibbles(&addr, &[], Position::Prefix));

    // Full 40-nibble pattern
    let full = vec![0xf; 40];
    assert!(matcher::matches_nibbles(&addr, &full, Position::Prefix));
    assert!(matcher::matches_nibbles(&addr, &full, Position::Suffix));

    // 41-nibble pattern (too long) returns false
    let too_long = vec![0xf; 41];
    assert!(!matcher::matches_nibbles(&addr, &too_long, Position::Prefix));
}

#[test]
fn test_encode_result_produces_valid_hex() {
    let address_bytes: [u8; 20] = [
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
        0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
    ];
    let private_key_bytes: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ];

    let (priv_key, addr) = crypto::encode_result(&address_bytes, &private_key_bytes);

    assert!(priv_key.starts_with("0x"));
    assert_eq!(priv_key.len(), 66);
    assert_eq!(addr, "0xdeadbeef00112233445566778899aabbccddeeff");

    // All hex chars must be lowercase
    for c in addr[2..].chars() {
        assert!(c.is_ascii_hexdigit());
        assert!(!c.is_uppercase());
    }
}
#[test]
fn test_hex_pattern_validation() {
    use engine::is_valid_hex_pattern;

    assert!(is_valid_hex_pattern("abcd"));
    assert!(is_valid_hex_pattern("123456"));
    assert!(is_valid_hex_pattern("632")); // Odd length now valid
    assert!(is_valid_hex_pattern("003")); // Odd length now valid
    assert!(is_valid_hex_pattern(""));
    
    // Invalid characters
    assert!(!is_valid_hex_pattern("ghjk"));
    
    // Too long
    assert!(!is_valid_hex_pattern(&"a".repeat(41)));
}

#[test]
fn test_matches_nibbles_basic() {
    let mut addr = [0u8; 20];
    // Address starts with 0x632...
    addr[0] = 0x63;
    addr[1] = 0x2A; // Next nibble is 2

    // Prefix 632
    let prefix = vec![6, 3, 2];
    assert!(matcher::matches_nibbles(&addr, &prefix, Position::Prefix));
    
    // Prefix 6321 (Incorrect)
    let prefix_bad = vec![6, 3, 2, 1];
    assert!(!matcher::matches_nibbles(&addr, &prefix_bad, Position::Prefix));

    // Suffix 003
    // Address ends with ...003
    addr[18] = 0x00;
    addr[19] = 0xA3; // Last nibble is 3, second to last is A
    // Wait, suffix "003" means the last 3 nibbles are 0, 0, 3.
    // Address ends in ...XYZ
    // In our address bytes: ... [addr 18] [addr 19]
    // Bytes are ... [Z W] [X Y] (hex)
    // "003" suffix means:
    // Nibble 39 (Y) = 3
    // Nibble 38 (X) = 0
    // Nibble 37 (W) = 0
    
    addr[18] = 0x50; // W=0
    addr[19] = 0x03; // X=0, Y=3
    
    let suffix = vec![0, 0, 3];
    assert!(matcher::matches_nibbles(&addr, &suffix, Position::Suffix));
}
