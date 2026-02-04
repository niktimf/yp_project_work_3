use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Post not found")]
    PostNotFound,

    #[error("Forbidden: you don't have permission to perform this action")]
    Forbidden,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Password hash error: {0}")]
    PasswordHashError(String),

    #[error("JWT error: {0}")]
    JwtError(String),
}

impl From<password_hash::Error> for DomainError {
    fn from(err: password_hash::Error) -> Self {
        Self::PasswordHashError(err.to_string())
    }
}
