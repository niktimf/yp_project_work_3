use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{Post, User};

// ============ Request DTOs ============

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterDto {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePostDto {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePostDto {
    pub title: String,
    pub content: String,
}

// ============ Response DTOs ============

#[derive(Debug, Clone, Serialize)]
pub struct UserDto {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

impl From<&User> for UserDto {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponseDto {
    pub token: String,
    pub user: UserDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostDto {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub author_username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Post> for PostDto {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            author_username: post.author_username,
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}

impl From<&Post> for PostDto {
    fn from(post: &Post) -> Self {
        Self {
            id: post.id,
            title: post.title.clone(),
            content: post.content.clone(),
            author_id: post.author_id,
            author_username: post.author_username.clone(),
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PostsListDto {
    pub posts: Vec<PostDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
