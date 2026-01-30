pub mod error;
pub mod grpc_client;
pub mod http_client;

pub use error::BlogClientError;
pub use grpc_client::GrpcBlogClient;
pub use http_client::HttpBlogClient;

// Generated protobuf code
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
    transport: Transport,
    http_client: Option<HttpBlogClient>,
    grpc_client: Option<GrpcBlogClient>,
}

impl BlogClient {
    /// Create a new client with the specified transport
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        match &transport {
            Transport::Http(base_url) => {
                let http_client = HttpBlogClient::new(base_url);
                Ok(Self {
                    transport,
                    http_client: Some(http_client),
                    grpc_client: None,
                })
            }
            Transport::Grpc(endpoint) => {
                let grpc_client = GrpcBlogClient::new(endpoint).await?;
                Ok(Self {
                    transport,
                    http_client: None,
                    grpc_client: Some(grpc_client),
                })
            }
        }
    }

    /// Set the JWT token for authenticated requests
    pub fn set_token(&mut self, token: String) {
        if let Some(http) = &mut self.http_client {
            http.set_token(token.clone());
        }
        if let Some(grpc) = &mut self.grpc_client {
            grpc.set_token(token);
        }
    }

    /// Get the current JWT token
    pub fn get_token(&self) -> Option<&str> {
        if let Some(http) = &self.http_client {
            return http.get_token();
        }
        if let Some(grpc) = &self.grpc_client {
            return grpc.get_token();
        }
        None
    }

    /// Clear the JWT token
    pub fn clear_token(&mut self) {
        if let Some(http) = &mut self.http_client {
            http.clear_token();
        }
        if let Some(grpc) = &mut self.grpc_client {
            grpc.clear_token();
        }
    }

    /// Register a new user
    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .unwrap()
                    .register(username, email, password)
                    .await?
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .unwrap()
                    .register(username, email, password)
                    .await?
            }
        };

        self.set_token(response.token.clone());
        Ok(response)
    }

    /// Login with email and password
    pub async fn login(
        &mut self,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .unwrap()
                    .login(email, password)
                    .await?
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .unwrap()
                    .login(email, password)
                    .await?
            }
        };

        self.set_token(response.token.clone());
        Ok(response)
    }

    /// Create a new post (requires authentication)
    pub async fn create_post(
        &mut self,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .unwrap()
                    .create_post(title, content)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .unwrap()
                    .create_post(title, content)
                    .await
            }
        }
    }

    /// Get a post by ID
    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                self.http_client.as_ref().unwrap().get_post(id).await
            }
            Transport::Grpc(_) => {
                self.grpc_client.as_mut().unwrap().get_post(id).await
            }
        }
    }

    /// Update a post (requires authentication)
    pub async fn update_post(
        &mut self,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .unwrap()
                    .update_post(id, title, content)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .unwrap()
                    .update_post(id, title, content)
                    .await
            }
        }
    }

    /// Delete a post (requires authentication)
    pub async fn delete_post(
        &mut self,
        id: i64,
    ) -> Result<(), BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                self.http_client.as_ref().unwrap().delete_post(id).await
            }
            Transport::Grpc(_) => {
                self.grpc_client.as_mut().unwrap().delete_post(id).await
            }
        }
    }

    /// List posts with pagination
    pub async fn list_posts(
        &mut self,
        limit: i64,
        offset: i64,
    ) -> Result<PostsList, BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .unwrap()
                    .list_posts(limit, offset)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .unwrap()
                    .list_posts(limit, offset)
                    .await
            }
        }
    }
}
