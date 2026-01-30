//! PKCE (Proof Key for Code Exchange) implementation for OAuth 2.0

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};

/// Generate a cryptographically random code verifier
///
/// The verifier is a 43-128 character random string using unreserved characters.
/// We use 64 bytes (86 base64url characters) for good security.
pub fn generate_code_verifier() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..64).map(|_| rng.gen()).collect();
    URL_SAFE_NO_PAD.encode(random_bytes)
}

/// Generate the code challenge from a code verifier
///
/// Uses SHA256 hash of the verifier, then base64url encodes it.
/// This implements the S256 challenge method.
pub fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

/// Generate a random state string for CSRF protection
pub fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    URL_SAFE_NO_PAD.encode(random_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code_verifier() {
        let verifier = generate_code_verifier();
        // Should be 86 characters (64 bytes base64url encoded without padding)
        assert!(verifier.len() >= 43 && verifier.len() <= 128);
        // Should only contain URL-safe characters
        assert!(verifier.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_generate_code_challenge() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = generate_code_challenge(verifier);
        // SHA256 hash of verifier, base64url encoded
        // This is a known test vector
        assert_eq!(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn test_generate_state() {
        let state = generate_state();
        // Should be 43 characters (32 bytes base64url encoded without padding)
        assert_eq!(state.len(), 43);
        // Should only contain URL-safe characters
        assert!(state.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_verifier_uniqueness() {
        let v1 = generate_code_verifier();
        let v2 = generate_code_verifier();
        assert_ne!(v1, v2, "Each verifier should be unique");
    }
}
