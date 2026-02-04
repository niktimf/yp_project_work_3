// Data layer - repositories and database interactions

pub mod post_repository;
pub mod user_repository;

pub use post_repository::PostgresPostRepository;
pub use user_repository::PostgresUserRepository;

use crate::domain::DomainError;

impl From<sqlx::Error> for DomainError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => Self::UserNotFound,
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    // PostgreSQL unique violation
                    if code == "23505" {
                        return Self::UserAlreadyExists;
                    }
                }
                Self::DatabaseError(err.to_string())
            }
            _ => Self::DatabaseError(err.to_string()),
        }
    }
}
