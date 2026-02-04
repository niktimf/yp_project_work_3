use chrono::{Duration, Utc};
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
};
use serde::{Deserialize, Serialize};

use crate::domain::DomainError;

use super::config::{FromEnv, env_or, env_required};

impl From<jsonwebtoken::errors::Error> for DomainError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::JwtError(err.to_string())
    }
}

#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub token_expiry_hours: i64,
}

impl FromEnv for JwtConfig {
    fn from_env() -> Self {
        Self {
            secret: env_required("JWT_SECRET"),
            token_expiry_hours: env_or("JWT_TOKEN_EXPIRY_HOURS", 24),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    token_expiry_hours: i64,
}

impl JwtService {
    pub fn new(config: &JwtConfig) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(config.secret.as_bytes()),
            token_expiry_hours: config.token_expiry_hours,
        }
    }

    pub fn generate_token(
        &self,
        user_id: i64,
        username: &str,
    ) -> Result<String, DomainError> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.token_expiry_hours);

        let claims = Claims {
            user_id,
            username: username.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(DomainError::from)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, DomainError> {
        let token_data: TokenData<Claims> =
            decode(token, &self.decoding_key, &Validation::default())?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test-secret-key-that-is-at-least-32-chars".to_string(),
            token_expiry_hours: 24,
        }
    }

    #[test]
    fn test_generate_and_verify_token() {
        let jwt_service = JwtService::new(&test_config());

        let token = jwt_service.generate_token(1, "testuser").unwrap();
        let claims = jwt_service.verify_token(&token).unwrap();

        assert_eq!(claims.user_id, 1);
        assert_eq!(claims.username, "testuser");
    }

    #[test]
    fn test_invalid_token() {
        let jwt_service = JwtService::new(&test_config());

        let result = jwt_service.verify_token("invalid-token");
        assert!(result.is_err());
    }
}
