use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::{DomainError, Post};

pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        title: &str,
        content: &str,
        author_id: i64,
    ) -> Result<Post, DomainError> {
        let row = sqlx::query_as::<_, PostRow>(
            r"
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, content, author_id, created_at, updated_at
            ",
        )
        .bind(title)
        .bind(content)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<Post>, DomainError> {
        let row = sqlx::query_as::<_, PostWithAuthorRow>(
            r"
            SELECT p.id, p.title, p.content, p.author_id, u.username as author_username, p.created_at, p.updated_at
            FROM posts p
            JOIN users u ON p.author_id = u.id
            WHERE p.id = $1
            ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    /// Updates post only if it belongs to the author.
    /// Returns None if post not found or doesn't belong to author.
    pub async fn update_by_author(
        &self,
        id: i64,
        author_id: i64,
        title: &str,
        content: &str,
    ) -> Result<Option<Post>, DomainError> {
        let row = sqlx::query_as::<_, PostRow>(
            r"
            UPDATE posts
            SET title = $3, content = $4, updated_at = NOW()
            WHERE id = $1 AND author_id = $2
            RETURNING id, title, content, author_id, created_at, updated_at
            ",
        )
        .bind(id)
        .bind(author_id)
        .bind(title)
        .bind(content)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    /// Deletes post only if it belongs to the author.
    /// Returns true if deleted, false if not found or doesn't belong to author.
    pub async fn delete_by_author(
        &self,
        id: i64,
        author_id: i64,
    ) -> Result<bool, DomainError> {
        let result =
            sqlx::query("DELETE FROM posts WHERE id = $1 AND author_id = $2")
                .bind(id)
                .bind(author_id)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Post>, DomainError> {
        let rows = sqlx::query_as::<_, PostWithAuthorRow>(
            r"
            SELECT p.id, p.title, p.content, p.author_id, u.username as author_username, p.created_at, p.updated_at
            FROM posts p
            JOIN users u ON p.author_id = u.id
            ORDER BY p.created_at DESC
            LIMIT $1 OFFSET $2
            ",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn count(&self) -> Result<i64, DomainError> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.0)
    }
}

#[derive(sqlx::FromRow)]
struct PostRow {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PostRow> for Post {
    fn from(row: PostRow) -> Self {
        Self::new(
            row.id,
            row.title,
            row.content,
            row.author_id,
            row.created_at,
            row.updated_at,
        )
    }
}

#[derive(sqlx::FromRow)]
struct PostWithAuthorRow {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    author_username: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PostWithAuthorRow> for Post {
    fn from(row: PostWithAuthorRow) -> Self {
        Self::new(
            row.id,
            row.title,
            row.content,
            row.author_id,
            row.created_at,
            row.updated_at,
        )
        .with_author_username(row.author_username)
    }
}
