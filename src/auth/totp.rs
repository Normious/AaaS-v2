use totp_rs::{Algorithm, Secret, TOTP};

use crate::error::{AppError, AppResult};

const ISSUER: &str = "AaaS";

pub fn generate_secret() -> String {
    Secret::generate_secret().to_encoded().to_string()
}

pub fn build_totp(secret_base32: &str, account_name: &str) -> AppResult<TOTP> {
    let secret = Secret::Encoded(secret_base32.to_string())
        .to_bytes()
        .map_err(|e| AppError::Internal(format!("TOTP secret: {e}")))?;

    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret,
        Some(ISSUER.to_string()),
        account_name.to_string(),
    )
    .map_err(|e| AppError::Internal(format!("TOTP build: {e}")))
}

/// Generate an otpauth:// URL for QR code generation.
pub fn otpauth_url(secret_base32: &str, account_name: &str) -> AppResult<String> {
    let totp = build_totp(secret_base32, account_name)?;
    Ok(totp.get_url())
}

/// Verify a 6-digit code from the user.
pub fn verify_code(secret_base32: &str, account_name: &str, code: &str) -> AppResult<bool> {
    let totp = build_totp(secret_base32, account_name)?;
    Ok(totp.check_current(code).unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_roundtrip() {
        let secret = generate_secret();
        let url = otpauth_url(&secret, "test@example.com").unwrap();
        assert!(url.starts_with("otpauth://totp/"));
    }
}
