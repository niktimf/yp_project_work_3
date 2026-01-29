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

impl From<sqlx::Error> for DomainError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => DomainError::UserNotFound,
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    // PostgreSQL unique violation
                    if code == "23505" {
                        return DomainError::UserAlreadyExists;
                    }
                }
                DomainError::DatabaseError(err.to_string())
            }
            _ => DomainError::DatabaseError(err.to_string()),
        }
    }
}

impl From<password_hash::Error> for DomainError {
    fn from(err: password_hash::Error) -> Self {
        DomainError::PasswordHashError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for DomainError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        DomainError::JwtError(err.to_string())
    }
}
