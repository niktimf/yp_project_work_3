use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::application::{AuthService, BlogService};
use crate::domain::{
    CreatePostCommand, DomainError, LoginCommand, RegisterCommand,
    UpdatePostCommand,
};
use crate::infrastructure::JwtService;

// Include generated protobuf code
pub mod proto {
    tonic::include_proto!("blog");
}

use proto::blog_service_server::BlogService as GrpcBlogService;
use proto::{
    AuthResponse, CreatePostRequest as GrpcCreatePostRequest,
    DeletePostRequest, DeleteResponse, GetPostRequest, ListPostsRequest,
    ListPostsResponse, LoginRequest as GrpcLoginRequest, Post as GrpcPost,
    PostResponse, RegisterRequest as GrpcRegisterRequest,
    UpdatePostRequest as GrpcUpdatePostRequest, User as GrpcUser,
};

pub struct BlogGrpcService {
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
}

impl BlogGrpcService {
    pub fn new(
        auth_service: Arc<AuthService>,
        blog_service: Arc<BlogService>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }

    fn extract_user_id<T>(&self, request: &Request<T>) -> Result<i64, Status> {
        let auth_header = request
            .metadata()
            .get("authorization")
            .ok_or_else(|| {
                Status::unauthenticated("Missing authorization header")
            })?
            .to_str()
            .map_err(|_| {
                Status::unauthenticated("Invalid authorization header")
            })?;

        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            Status::unauthenticated("Invalid authorization header format")
        })?;

        let claims = self.jwt_service.verify_token(token).map_err(|e| {
            Status::unauthenticated(format!("Invalid token: {e}"))
        })?;

        Ok(claims.user_id)
    }
}

impl From<DomainError> for Status {
    fn from(e: DomainError) -> Self {
        match &e {
            DomainError::UserAlreadyExists => Status::already_exists(e.to_string()),
            DomainError::InvalidCredentials => Status::unauthenticated(e.to_string()),
            DomainError::PostNotFound | DomainError::UserNotFound => {
                Status::not_found(e.to_string())
            }
            DomainError::Forbidden => Status::permission_denied(e.to_string()),
            DomainError::ValidationError(_) => Status::invalid_argument(e.to_string()),
            _ => Status::internal(e.to_string()),
        }
    }
}

#[tonic::async_trait]
impl GrpcBlogService for BlogGrpcService {
    async fn register(
        &self,
        request: Request<GrpcRegisterRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        let command = RegisterCommand {
            username: req.username,
            email: req.email,
            password: req.password,
        };

        let result = self
            .auth_service
            .register(command)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(AuthResponse {
            token: result.token,
            user: Some(GrpcUser {
                id: result.user.id.to_string(),
                username: result.user.username,
                email: result.user.email,
                created_at: result.user.created_at.to_rfc3339(),
            }),
        }))
    }

    async fn login(
        &self,
        request: Request<GrpcLoginRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        let command = LoginCommand {
            email: req.email,
            password: req.password,
        };

        let result = self
            .auth_service
            .login(command)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(AuthResponse {
            token: result.token,
            user: Some(GrpcUser {
                id: result.user.id.to_string(),
                username: result.user.username,
                email: result.user.email,
                created_at: result.user.created_at.to_rfc3339(),
            }),
        }))
    }

    async fn create_post(
        &self,
        request: Request<GrpcCreatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let user_id = self.extract_user_id(&request)?;
        let req = request.into_inner();

        let command = CreatePostCommand {
            title: req.title,
            content: req.content,
        };

        let post = self
            .blog_service
            .create_post(user_id, command)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(PostResponse {
            post: Some(GrpcPost {
                id: post.id.to_string(),
                title: post.title,
                content: post.content,
                author_id: post.author_id.to_string(),
                author_username: post.author_username.unwrap_or_default(),
                created_at: post.created_at.to_rfc3339(),
                updated_at: post.updated_at.to_rfc3339(),
            }),
        }))
    }

    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let req = request.into_inner();

        let post_id: i64 = req
            .post_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid post_id"))?;

        let post = self
            .blog_service
            .get_post(post_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(PostResponse {
            post: Some(GrpcPost {
                id: post.id.to_string(),
                title: post.title,
                content: post.content,
                author_id: post.author_id.to_string(),
                author_username: post.author_username.unwrap_or_default(),
                created_at: post.created_at.to_rfc3339(),
                updated_at: post.updated_at.to_rfc3339(),
            }),
        }))
    }

    async fn update_post(
        &self,
        request: Request<GrpcUpdatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let user_id = self.extract_user_id(&request)?;
        let req = request.into_inner();

        let post_id: i64 = req
            .post_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid post_id"))?;

        let command = UpdatePostCommand {
            title: req.title,
            content: req.content,
        };

        let post = self
            .blog_service
            .update_post(post_id, user_id, command)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(PostResponse {
            post: Some(GrpcPost {
                id: post.id.to_string(),
                title: post.title,
                content: post.content,
                author_id: post.author_id.to_string(),
                author_username: post.author_username.unwrap_or_default(),
                created_at: post.created_at.to_rfc3339(),
                updated_at: post.updated_at.to_rfc3339(),
            }),
        }))
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let user_id = self.extract_user_id(&request)?;
        let req = request.into_inner();

        let post_id: i64 = req
            .post_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid post_id"))?;

        self.blog_service
            .delete_post(post_id, user_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(DeleteResponse {
            success: true,
            message: "Post deleted successfully".to_string(),
        }))
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let req = request.into_inner();

        let page = req.page.max(1);
        let page_size = req.page_size.clamp(1, 100);
        let offset = i64::from((page - 1) * page_size);
        let limit = i64::from(page_size);

        let (posts, total) = self
            .blog_service
            .list_posts(limit, offset)
            .await
            .map_err(Status::from)?;

        let grpc_posts: Vec<GrpcPost> = posts
            .into_iter()
            .map(|post| GrpcPost {
                id: post.id.to_string(),
                title: post.title,
                content: post.content,
                author_id: post.author_id.to_string(),
                author_username: post.author_username.unwrap_or_default(),
                created_at: post.created_at.to_rfc3339(),
                updated_at: post.updated_at.to_rfc3339(),
            })
            .collect();

        Ok(Response::new(ListPostsResponse {
            posts: grpc_posts,
            total_count: total as i32,
            page,
            page_size,
        }))
    }
}
