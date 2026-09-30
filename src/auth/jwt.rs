use chrono::Utc;
use ed25519_dalek::{pkcs8::DecodePrivateKey, SigningKey};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub sid: String,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub scope: Vec<String>,
}

pub struct JwtIssuer {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    audience: String,
    access_ttl: i64,
}

impl JwtIssuer {
    pub fn from_config(config: &Config) -> AppResult<Self> {
        let private_pem = std::fs::read_to_string(&config.jwt_private_key_path)
            .map_err(|e| AppError::Internal(format!("Read JWT private key: {e}")))?;
        let public_pem = std::fs::read_to_string(&config.jwt_public_key_path)
            .map_err(|e| AppError::Internal(format!("Read JWT public key: {e}")))?;

        // Validate the private key is a valid Ed25519 key
        let _ = SigningKey::from_pkcs8_pem(&private_pem)
            .map_err(|e| AppError::Internal(format!("Parse private key: {e}")))?;

        Ok(Self {
            encoding_key: EncodingKey::from_ed_pem(private_pem.as_bytes())
                .map_err(|e| AppError::Internal(format!("Encoding key: {e}")))?,
            decoding_key: DecodingKey::from_ed_pem(public_pem.as_bytes())
                .map_err(|e| AppError::Internal(format!("Decoding key: {e}")))?,
            issuer: config.jwt_issuer.clone(),
            audience: config.jwt_audience.clone(),
            access_ttl: config.jwt_access_ttl,
        })
    }

    pub fn issue_access_token(
        &self,
        user_id: &str,
        session_id: &str,
        scope: Vec<String>,
    ) -> AppResult<String> {
        let now = Utc::now().timestamp();

        let claims = AccessClaims {
            sub: user_id.to_string(),
            sid: session_id.to_string(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            exp: now + self.access_ttl,
            iat: now,
            jti: Uuid::new_v4().to_string(),
            scope,
        };

        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some("aaas-v2-ed25519".into());

        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(format!("JWT encode: {e}")))
    }

    pub fn verify_access_token(&self, token: &str) -> AppResult<AccessClaims> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.leeway = 5;

        let token_data = decode::<AccessClaims>(token, &self.decoding_key, &validation)
            .map_err(|_| AppError::Unauthorized)?;

        Ok(token_data.claims)
    }
}
