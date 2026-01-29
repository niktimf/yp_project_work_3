use chrono::{Duration, Utc};
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
};
use serde::{Deserialize, Serialize};

use crate::domain::DomainError;

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
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn generate_token(
        &self,
        user_id: i64,
        username: &str,
    ) -> Result<String, DomainError> {
        let now = Utc::now();
        let exp = now + Duration::hours(24);

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

    #[test]
    fn test_generate_and_verify_token() {
        let jwt_service =
            JwtService::new("test-secret-key-that-is-at-least-32-chars");

        let token = jwt_service.generate_token(1, "testuser").unwrap();
        let claims = jwt_service.verify_token(&token).unwrap();

        assert_eq!(claims.user_id, 1);
        assert_eq!(claims.username, "testuser");
    }

    #[test]
    fn test_invalid_token() {
        let jwt_service =
            JwtService::new("test-secret-key-that-is-at-least-32-chars");

        let result = jwt_service.verify_token("invalid-token");
        assert!(result.is_err());
    }
}
