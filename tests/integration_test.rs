use engine::crypto;
use engine::matcher;

#[test]
fn test_match_logic() {
    let addr = "0x1234567890abcdef1234567890abcdef12345678";
    
    // Prefix match
    assert!(matcher::matches(addr, "1234", false));
    assert!(matcher::matches(addr, "12345", false));
    assert!(!matcher::matches(addr, "abcd", false));
    
    // Suffix match
    assert!(matcher::matches(addr, "5678", true));
    assert!(matcher::matches(addr, "abcdef12345678", true));
    assert!(!matcher::matches(addr, "1234", true));
    
    // Case insensitive matching
    let mixed_addr = "0xAbCdEf78901234567890123456789012345678";
    assert!(matcher::matches(mixed_addr, "abcdef", false));
    assert!(matcher::matches(mixed_addr, "345678", true));
    
    // Without prefix 0x
    let raw_addr = "1234567890abcdef1234567890abcdef12345678";
    assert!(matcher::matches(raw_addr, "1234", false));
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
    // Note: Since `crypto::generate_keypair` internally calls getrandom,
    // we cannot test fixed inputs easily without refactoring the random seed injection.
    // We will verify length and format constraints.
    // Address format should be lowercase hex characters for everything after `0x`.
    let (_, addr) = crypto::generate_keypair();
    let hex_part = &addr[2..];
    for c in hex_part.chars() {
        assert!(c.is_digit(16));
        assert!(!c.is_uppercase()); // Hex encode string produces lowercase
    }
}
