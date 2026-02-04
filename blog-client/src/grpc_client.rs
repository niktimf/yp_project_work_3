use tonic::Request;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;

use crate::error::BlogClientError;
use crate::proto::blog_service_client::BlogServiceClient;
use crate::proto::{
    CreatePostRequest, DeletePostRequest, GetPostRequest, ListPostsRequest,
    LoginRequest, RegisterRequest, UpdatePostRequest,
};
use crate::{AuthResponse, Post, PostsList, User};

pub struct GrpcBlogClient {
    client: BlogServiceClient<Channel>,
    token: Option<String>,
}

impl GrpcBlogClient {
    /// Create a new gRPC blog client.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if connection to the gRPC endpoint fails.
    pub async fn new(endpoint: &str) -> Result<Self, BlogClientError> {
        let client = BlogServiceClient::connect(endpoint.to_string()).await?;
        Ok(Self {
            client,
            token: None,
        })
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

    fn create_request<T>(&self, message: T) -> Request<T> {
        let mut request = Request::new(message);

        if let Some(token) = &self.token
            && let Ok(value) =
                format!("Bearer {token}").parse::<MetadataValue<_>>()
        {
            request.metadata_mut().insert("authorization", value);
        }

        request
    }

    /// Register a new user.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the gRPC call fails or the response is missing user data.
    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let request = Request::new(RegisterRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        });

        let response = self.client.register(request).await?.into_inner();

        let user = response.user.ok_or_else(|| {
            BlogClientError::InvalidRequest(
                "Missing user in response".to_string(),
            )
        })?;

        Ok(AuthResponse {
            token: response.token,
            user: User {
                id: user.id.parse().unwrap_or(0),
                username: user.username,
                email: user.email,
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &user.created_at,
                )
                .map_or_else(
                    |_| chrono::Utc::now(),
                    |dt| dt.with_timezone(&chrono::Utc),
                ),
            },
        })
    }

    /// Login with email and password.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the gRPC call fails or the response is missing user data.
    pub async fn login(
        &mut self,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let request = Request::new(LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        });

        let response = self.client.login(request).await?.into_inner();

        let user = response.user.ok_or_else(|| {
            BlogClientError::InvalidRequest(
                "Missing user in response".to_string(),
            )
        })?;

        Ok(AuthResponse {
            token: response.token,
            user: User {
                id: user.id.parse().unwrap_or(0),
                username: user.username,
                email: user.email,
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &user.created_at,
                )
                .map_or_else(
                    |_| chrono::Utc::now(),
                    |dt| dt.with_timezone(&chrono::Utc),
                ),
            },
        })
    }

    /// Create a new post.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the gRPC call fails or the response is missing post data.
    pub async fn create_post(
        &mut self,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let request = self.create_request(CreatePostRequest {
            title: title.to_string(),
            content: content.to_string(),
        });

        let response = self.client.create_post(request).await?.into_inner();

        let post = response.post.ok_or_else(|| {
            BlogClientError::InvalidRequest(
                "Missing post in response".to_string(),
            )
        })?;

        Ok(grpc_post_to_post(post))
    }

    /// Get a post by ID.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the gRPC call fails or the response is missing post data.
    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        let request = Request::new(GetPostRequest {
            post_id: id.to_string(),
        });

        let response = self.client.get_post(request).await?.into_inner();

        let post = response.post.ok_or_else(|| {
            BlogClientError::InvalidRequest(
                "Missing post in response".to_string(),
            )
        })?;

        Ok(grpc_post_to_post(post))
    }

    /// Update an existing post.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the gRPC call fails or the response is missing post data.
    pub async fn update_post(
        &mut self,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let request = self.create_request(UpdatePostRequest {
            post_id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
        });

        let response = self.client.update_post(request).await?.into_inner();

        let post = response.post.ok_or_else(|| {
            BlogClientError::InvalidRequest(
                "Missing post in response".to_string(),
            )
        })?;

        Ok(grpc_post_to_post(post))
    }

    /// Delete a post by ID.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the gRPC call fails.
    pub async fn delete_post(
        &mut self,
        id: i64,
    ) -> Result<(), BlogClientError> {
        let request = self.create_request(DeletePostRequest {
            post_id: id.to_string(),
        });

        self.client.delete_post(request).await?;
        Ok(())
    }

    /// List posts with pagination.
    ///
    /// # Errors
    ///
    /// Returns `BlogClientError` if the gRPC call fails.
    pub async fn list_posts(
        &mut self,
        limit: i64,
        offset: i64,
    ) -> Result<PostsList, BlogClientError> {
        let page = i32::try_from(offset / limit).unwrap_or(i32::MAX) + 1;
        let page_size = i32::try_from(limit).unwrap_or(i32::MAX);

        let request = Request::new(ListPostsRequest {
            page,
            page_size,
            author_id: None,
        });

        let response = self.client.list_posts(request).await?.into_inner();

        let posts: Vec<Post> =
            response.posts.into_iter().map(grpc_post_to_post).collect();

        Ok(PostsList {
            posts,
            total: response.total_count,
            limit,
            offset,
        })
    }
}

fn grpc_post_to_post(post: crate::proto::Post) -> Post {
    Post {
        id: post.id.parse().unwrap_or(0),
        title: post.title,
        content: post.content,
        author_id: post.author_id.parse().unwrap_or(0),
        author_username: if post.author_username.is_empty() {
            None
        } else {
            Some(post.author_username)
        },
        created_at: chrono::DateTime::parse_from_rfc3339(&post.created_at)
            .map_or_else(
                |_| chrono::Utc::now(),
                |dt| dt.with_timezone(&chrono::Utc),
            ),
        updated_at: chrono::DateTime::parse_from_rfc3339(&post.updated_at)
            .map_or_else(
                |_| chrono::Utc::now(),
                |dt| dt.with_timezone(&chrono::Utc),
            ),
    }
}
