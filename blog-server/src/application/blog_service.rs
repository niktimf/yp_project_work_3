use std::sync::Arc;

use crate::data::PostgresPostRepository;
use crate::domain::{CreatePostCommand, DomainError, Post, UpdatePostCommand};

pub struct BlogService {
    post_repository: Arc<PostgresPostRepository>,
}

impl BlogService {
    pub const fn new(post_repository: Arc<PostgresPostRepository>) -> Self {
        Self { post_repository }
    }

    pub async fn create_post(
        &self,
        author_id: i64,
        command: CreatePostCommand,
    ) -> Result<Post, DomainError> {
        self.post_repository
            .create(&command.title, &command.content, author_id)
            .await
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, DomainError> {
        self.post_repository
            .find_by_id(id)
            .await?
            .ok_or(DomainError::PostNotFound)
    }

    pub async fn update_post(
        &self,
        id: i64,
        author_id: i64,
        command: UpdatePostCommand,
    ) -> Result<Post, DomainError> {
        // Try to update - one query in happy path
        if let Some(post) = self
            .post_repository
            .update_by_author(id, author_id, &command.title, &command.content)
            .await?
        {
            return Ok(post);
        }

        // Failed - check why (only on error path)
        if self.post_repository.find_by_id(id).await?.is_some() {
            Err(DomainError::Forbidden)
        } else {
            Err(DomainError::PostNotFound)
        }
    }

    pub async fn delete_post(
        &self,
        id: i64,
        author_id: i64,
    ) -> Result<(), DomainError> {
        // Try to delete - one query in happy path
        if self.post_repository.delete_by_author(id, author_id).await? {
            return Ok(());
        }

        // Failed - check why (only on error path)
        if self.post_repository.find_by_id(id).await?.is_some() {
            Err(DomainError::Forbidden)
        } else {
            Err(DomainError::PostNotFound)
        }
    }

    pub async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Post>, i64), DomainError> {
        let posts = self.post_repository.list(limit, offset).await?;
        let total = self.post_repository.count().await?;
        Ok((posts, total))
    }
}
