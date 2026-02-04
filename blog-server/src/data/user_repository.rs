use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::{DomainError, Password, User};

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &Password,
    ) -> Result<User, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            r"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, created_at
            ",
        )
        .bind(username)
        .bind(email)
        .bind(password_hash.as_ref())
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    #[allow(dead_code)]
    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            r"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    pub async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            r"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE email = $1
            ",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    #[allow(dead_code)]
    pub async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            r"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE username = $1
            ",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: i64,
    username: String,
    email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self::new(
            row.id,
            row.username,
            row.email,
            Password::from_hash(row.password_hash),
            row.created_at,
        )
    }
}
