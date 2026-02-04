pub mod error;
pub mod grpc_client;
pub mod http_client;

pub use error::BlogClientError;
pub use grpc_client::GrpcBlogClient;
pub use http_client::HttpBlogClient;

// Generated protobuf code — allow clippy lints that cannot be fixed in auto-generated tonic/prost output
#[allow(clippy::missing_errors_doc, clippy::derive_partial_eq_without_eq)]
pub mod proto {
    tonic::include_proto!("blog");
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Transport type for the client
#[derive(Debug, Clone)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

/// Post data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub author_username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Paginated list of posts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostsList {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Unified blog client that can use either HTTP or gRPC transport
pub struct BlogClient {
    inner: ClientImpl,
}

enum ClientImpl {
    Http(HttpBlogClient),
    Grpc(GrpcBlogClient),
}

impl BlogClient {
    /// Create a new client with the specified transport.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the connection to the server fails.
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        let inner = match transport {
            Transport::Http(base_url) => {
                ClientImpl::Http(HttpBlogClient::new(&base_url))
            }
            Transport::Grpc(endpoint) => {
                ClientImpl::Grpc(GrpcBlogClient::new(&endpoint).await?)
            }
        };
        Ok(Self { inner })
    }

    /// Set the JWT token for authenticated requests
    pub fn set_token(&mut self, token: String) {
        match &mut self.inner {
            ClientImpl::Http(c) => c.set_token(token),
            ClientImpl::Grpc(c) => c.set_token(token),
        }
    }

    /// Get the current JWT token
    pub fn get_token(&self) -> Option<&str> {
        match &self.inner {
            ClientImpl::Http(c) => c.get_token(),
            ClientImpl::Grpc(c) => c.get_token(),
        }
    }

    /// Clear the JWT token
    pub fn clear_token(&mut self) {
        match &mut self.inner {
            ClientImpl::Http(c) => c.clear_token(),
            ClientImpl::Grpc(c) => c.clear_token(),
        }
    }

    /// Register a new user.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the registration request fails.
    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = match &mut self.inner {
            ClientImpl::Http(c) => {
                c.register(username, email, password).await?
            }
            ClientImpl::Grpc(c) => {
                c.register(username, email, password).await?
            }
        };

        self.set_token(response.token.clone());
        Ok(response)
    }

    /// Login with email and password.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the login request fails.
    pub async fn login(
        &mut self,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = match &mut self.inner {
            ClientImpl::Http(c) => c.login(email, password).await?,
            ClientImpl::Grpc(c) => c.login(email, password).await?,
        };

        self.set_token(response.token.clone());
        Ok(response)
    }

    /// Create a new post (requires authentication).
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the request fails or authentication is missing.
    pub async fn create_post(
        &mut self,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        match &mut self.inner {
            ClientImpl::Http(c) => c.create_post(title, content).await,
            ClientImpl::Grpc(c) => c.create_post(title, content).await,
        }
    }

    /// Get a post by ID.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the request fails or the post is not found.
    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        match &mut self.inner {
            ClientImpl::Http(c) => c.get_post(id).await,
            ClientImpl::Grpc(c) => c.get_post(id).await,
        }
    }

    /// Update a post (requires authentication).
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the request fails or authentication is missing.
    pub async fn update_post(
        &mut self,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        match &mut self.inner {
            ClientImpl::Http(c) => c.update_post(id, title, content).await,
            ClientImpl::Grpc(c) => c.update_post(id, title, content).await,
        }
    }

    /// Delete a post (requires authentication).
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the request fails or authentication is missing.
    pub async fn delete_post(
        &mut self,
        id: i64,
    ) -> Result<(), BlogClientError> {
        match &mut self.inner {
            ClientImpl::Http(c) => c.delete_post(id).await,
            ClientImpl::Grpc(c) => c.delete_post(id).await,
        }
    }

    /// List posts with pagination.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the request fails.
    pub async fn list_posts(
        &mut self,
        limit: i64,
        offset: i64,
    ) -> Result<PostsList, BlogClientError> {
        match &mut self.inner {
            ClientImpl::Http(c) => c.list_posts(limit, offset).await,
            ClientImpl::Grpc(c) => c.list_posts(limit, offset).await,
        }
    }
}
