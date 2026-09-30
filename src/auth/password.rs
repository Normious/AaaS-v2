use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};

use crate::config::Config;
use crate::error::{AppError, AppResult};

pub fn hash_password(password: &str, config: &Config) -> AppResult<String> {
    let params = Params::new(
        config.argon2_memory_kib,
        config.argon2_iterations,
        config.argon2_parallelism,
        None,
    )
    .map_err(|e| AppError::Internal(format!("Argon2 params: {e}")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let salt = SaltString::generate(&mut OsRng);

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("Hash error: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed =
        PasswordHash::new(hash).map_err(|e| AppError::Internal(format!("Parse hash: {e}")))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// Timing-attack mitigation: always do comparable work, even if user doesn't exist.
pub fn dummy_verify(password: &str) {
    // ponytail: fixed hash keeps timing roughly constant; upgrade to per-request random salt if measured otherwise.
    const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG";
    let _ = verify_password(password, DUMMY_HASH);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        let mut cfg = Config::from_env();
        cfg.argon2_memory_kib = 8192;
        cfg.argon2_iterations = 1;
        cfg
    }

    #[test]
    fn hash_and_verify() {
        let cfg = test_config();
        let hash = hash_password("correct horse battery staple", &cfg).unwrap();
        assert!(verify_password("correct horse battery staple", &hash).unwrap());
        assert!(!verify_password("wrong password", &hash).unwrap());
    }
}
