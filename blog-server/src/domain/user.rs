use chrono::{DateTime, Utc};

use super::password::Password;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: Password,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        id: i64,
        username: String,
        email: String,
        password_hash: Password,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            password_hash,
            created_at,
        }
    }
}

/// Domain command for user registration
#[derive(Debug, Clone)]
pub struct RegisterCommand {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Domain command for user login
#[derive(Debug, Clone)]
pub struct LoginCommand {
    pub email: String,
    pub password: String,
}

/// Domain result for successful authentication
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub token: String,
    pub user: User,
}
