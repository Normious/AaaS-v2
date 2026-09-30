use rand::RngCore;
use sha2::{Digest, Sha256};

/// Generate a cryptographically secure random token (URL-safe).
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64_url(&bytes)
}

/// Generate a short numeric code (for TOTP backup codes).
pub fn generate_numeric_code(digits: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut code = String::with_capacity(digits);
    for _ in 0..digits {
        code.push(char::from(b'0' + (rng.next_u32() % 10) as u8));
    }
    code
}

/// SHA-256 hash of a token, returned as hex.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn base64_url(bytes: &[u8]) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_url_safe_and_unique() {
        let a = generate_token();
        let b = generate_token();
        assert_ne!(a, b);
        assert!(!a.contains(['+', '/', '=']));
        assert_eq!(hash_token(&a).len(), 64);
    }
}
