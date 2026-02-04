use std::sync::Arc;

use crate::data::PostgresUserRepository;
use crate::domain::{
    AuthResult, DomainError, LoginCommand, Password, RegisterCommand,
};
use crate::infrastructure::JwtService;

pub struct AuthService {
    user_repository: Arc<PostgresUserRepository>,
    jwt_service: Arc<JwtService>,
}

impl AuthService {
    pub const fn new(
        user_repository: Arc<PostgresUserRepository>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            user_repository,
            jwt_service,
        }
    }

    pub async fn register(
        &self,
        command: RegisterCommand,
    ) -> Result<AuthResult, DomainError> {
        // Hash password
        let password_hash = Password::hash(&command.password)?;

        // Create user - DB will reject duplicates via UNIQUE constraints
        // Error code 23505 is converted to UserAlreadyExists in From<sqlx::Error>
        let user = self
            .user_repository
            .create(&command.username, &command.email, &password_hash)
            .await?;

        // Generate token
        let token = self.jwt_service.generate_token(user.id, &user.username)?;

        Ok(AuthResult { token, user })
    }

    pub async fn login(
        &self,
        command: LoginCommand,
    ) -> Result<AuthResult, DomainError> {
        // Find user by email
        let user = self
            .user_repository
            .find_by_email(&command.email)
            .await?
            .ok_or(DomainError::InvalidCredentials)?;

        // Verify password
        if !user.password_hash.verify(&command.password) {
            return Err(DomainError::InvalidCredentials);
        }

        // Generate token
        let token = self.jwt_service.generate_token(user.id, &user.username)?;

        Ok(AuthResult { token, user })
    }
}
