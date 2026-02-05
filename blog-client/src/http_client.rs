use std::time::Duration;

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};

use crate::error::BlogClientError;
use crate::{AuthResponse, Post, PostsList, User};

#[derive(Debug, Serialize)]
struct RegisterRequest<'a> {
    username: &'a str,
    email: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct CreatePostRequest<'a> {
    title: &'a str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
struct UpdatePostRequest<'a> {
    title: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ApiAuthResponse {
    token: String,
    user: ApiUser,
}

#[derive(Debug, Deserialize)]
struct ApiUser {
    id: i64,
    username: String,
    email: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct ApiPost {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    author_username: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct ApiPostsList {
    posts: Vec<ApiPost>,
    total: i64,
    limit: i64,
    offset: i64,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    error: String,
}

impl From<ApiAuthResponse> for AuthResponse {
    fn from(api: ApiAuthResponse) -> Self {
        Self {
            token: api.token,
            user: User {
                id: api.user.id,
                username: api.user.username,
                email: api.user.email,
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &api.user.created_at,
                )
                .map_or_else(
                    |_| chrono::Utc::now(),
                    |dt| dt.with_timezone(&chrono::Utc),
                ),
            },
        }
    }
}

impl From<ApiPost> for Post {
    fn from(api: ApiPost) -> Self {
        Self {
            id: api.id,
            title: api.title,
            content: api.content,
            author_id: api.author_id,
            author_username: api.author_username,
            created_at: chrono::DateTime::parse_from_rfc3339(&api.created_at)
                .map_or_else(
                    |_| chrono::Utc::now(),
                    |dt| dt.with_timezone(&chrono::Utc),
                ),
            updated_at: chrono::DateTime::parse_from_rfc3339(&api.updated_at)
                .map_or_else(
                    |_| chrono::Utc::now(),
                    |dt| dt.with_timezone(&chrono::Utc),
                ),
        }
    }
}

pub struct HttpBlogClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl HttpBlogClient {
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Self::DEFAULT_TIMEOUT)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.base_url, path)
    }

    async fn handle_error_response(
        &self,
        response: reqwest::Response,
    ) -> BlogClientError {
        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return BlogClientError::NotFound;
        }

        if status == reqwest::StatusCode::UNAUTHORIZED {
            let msg = response
                .json::<ApiError>()
                .await
                .map_or_else(|_| "Unauthorized".to_string(), |e| e.error);
            return BlogClientError::Unauthorized(msg);
        }

        let msg = response
            .json::<ApiError>()
            .await
            .map_or_else(|_| format!("HTTP error: {status}"), |e| e.error);

        BlogClientError::InvalidRequest(msg)
    }

    /// Register a new user.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the HTTP request fails or the server returns an error.
    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .post(self.url("/auth/register"))
            .json(&RegisterRequest {
                username,
                email,
                password,
            })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let api_response: ApiAuthResponse = response.json().await?;
        Ok(api_response.into())
    }

    /// Login with username and password.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the HTTP request fails or credentials are invalid.
    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .post(self.url("/auth/login"))
            .json(&LoginRequest { username, password })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let api_response: ApiAuthResponse = response.json().await?;
        Ok(api_response.into())
    }

    /// Create a new post.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if no token is set, the HTTP request fails, or the server returns an error.
    pub async fn create_post(
        &self,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let token = self.token.as_ref().ok_or(BlogClientError::NoToken)?;

        let response = self
            .client
            .post(self.url("/posts"))
            .bearer_auth(token)
            .json(&CreatePostRequest { title, content })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let api_post: ApiPost = response.json().await?;
        Ok(api_post.into())
    }

    /// Get a post by ID.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the HTTP request fails or the post is not found.
    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .get(self.url(&format!("/posts/{id}")))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let api_post: ApiPost = response.json().await?;
        Ok(api_post.into())
    }

    /// Update an existing post.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if no token is set, the HTTP request fails, or the server returns an error.
    pub async fn update_post(
        &self,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let token = self.token.as_ref().ok_or(BlogClientError::NoToken)?;

        let response = self
            .client
            .put(self.url(&format!("/posts/{id}")))
            .bearer_auth(token)
            .json(&UpdatePostRequest { title, content })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let api_post: ApiPost = response.json().await?;
        Ok(api_post.into())
    }

    /// Delete a post by ID.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if no token is set, the HTTP request fails, or the server returns an error.
    pub async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        let token = self.token.as_ref().ok_or(BlogClientError::NoToken)?;

        let response = self
            .client
            .delete(self.url(&format!("/posts/{id}")))
            .bearer_auth(token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        Ok(())
    }

    /// List posts with pagination.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the HTTP request fails or the server returns an error.
    pub async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<PostsList, BlogClientError> {
        let response: Response = self
            .client
            .get(self.url("/posts"))
            .query(&[("limit", limit), ("offset", offset)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let api_list: ApiPostsList = response.json().await?;
        Ok(PostsList {
            posts: api_list.posts.into_iter().map(Post::from).collect(),
            total: api_list.total,
            limit: api_list.limit,
            offset: api_list.offset,
        })
    }
}
