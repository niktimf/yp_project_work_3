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
    email: &'a str,
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
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
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
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&api.updated_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }
}

pub struct HttpBlogClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl HttpBlogClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
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
                .map(|e| e.error)
                .unwrap_or_else(|_| "Unauthorized".to_string());
            return BlogClientError::Unauthorized(msg);
        }

        let msg = response
            .json::<ApiError>()
            .await
            .map(|e| e.error)
            .unwrap_or_else(|_| format!("HTTP error: {status}"));

        BlogClientError::InvalidRequest(msg)
    }

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

    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = self
            .client
            .post(self.url("/auth/login"))
            .json(&LoginRequest { email, password })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_error_response(response).await);
        }

        let api_response: ApiAuthResponse = response.json().await?;
        Ok(api_response.into())
    }

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
