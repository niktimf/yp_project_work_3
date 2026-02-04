use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub author_username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Post {
    pub const fn new(
        id: i64,
        title: String,
        content: String,
        author_id: i64,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            title,
            content,
            author_id,
            author_username: None,
            created_at,
            updated_at,
        }
    }

    pub fn with_author_username(mut self, username: String) -> Self {
        self.author_username = Some(username);
        self
    }
}

/// Domain command for creating a post
#[derive(Debug, Clone)]
pub struct CreatePostCommand {
    pub title: String,
    pub content: String,
}

/// Domain command for updating a post
#[derive(Debug, Clone)]
pub struct UpdatePostCommand {
    pub title: String,
    pub content: String,
}
