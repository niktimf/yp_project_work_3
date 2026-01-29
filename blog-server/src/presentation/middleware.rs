use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::sync::Arc;

use crate::infrastructure::JwtService;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub struct AuthError(pub String);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: self.0 }))
            .into_response()
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Get JWT service from extensions
        let jwt_service = parts
            .extensions
            .get::<Arc<JwtService>>()
            .ok_or_else(|| AuthError("JWT service not configured".to_string()))?
            .clone();

        // Get Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| {
                AuthError("Missing Authorization header".to_string())
            })?
            .to_str()
            .map_err(|_| {
                AuthError("Invalid Authorization header".to_string())
            })?;

        // Extract Bearer token
        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            AuthError("Invalid Authorization header format".to_string())
        })?;

        // Verify token
        let claims = jwt_service
            .verify_token(token)
            .map_err(|e| AuthError(format!("Invalid token: {}", e)))?;

        Ok(AuthenticatedUser {
            user_id: claims.user_id,
            username: claims.username,
        })
    }
}

// Optional auth extractor - doesn't fail if no token is present
#[derive(Debug, Clone)]
pub struct OptionalAuthenticatedUser(pub Option<AuthenticatedUser>);

impl<S> FromRequestParts<S> for OptionalAuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        match AuthenticatedUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuthenticatedUser(Some(user))),
            Err(_) => Ok(OptionalAuthenticatedUser(None)),
        }
    }
}
