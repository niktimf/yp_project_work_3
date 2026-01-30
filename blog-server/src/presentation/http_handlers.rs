use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::application::{AuthService, BlogService};
use crate::domain::{
    CreatePostCommand, DomainError, LoginCommand, RegisterCommand,
    UpdatePostCommand,
};
use crate::infrastructure::ServerConfig;
use crate::presentation::dto::{
    AuthResponseDto, CreatePostDto, LoginDto, PostDto, PostsListDto,
    RegisterDto, UpdatePostDto, UserDto,
};
use crate::presentation::middleware::AuthenticatedUser;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub blog_service: Arc<BlogService>,
}

// Error response
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// Convert DomainError to HTTP response
impl IntoResponse for DomainError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            DomainError::UserNotFound => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            DomainError::UserAlreadyExists => {
                (StatusCode::CONFLICT, self.to_string())
            }
            DomainError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            DomainError::PostNotFound => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            DomainError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            DomainError::ValidationError(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

// ============ Auth Handlers ============

pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterDto>,
) -> Result<impl IntoResponse, DomainError> {
    let command = RegisterCommand {
        username: dto.username,
        email: dto.email,
        password: dto.password,
    };

    let result = state.auth_service.register(command).await?;

    let response = AuthResponseDto {
        token: result.token,
        user: UserDto::from(&result.user),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> Result<impl IntoResponse, DomainError> {
    let command = LoginCommand {
        email: dto.email,
        password: dto.password,
    };

    let result = state.auth_service.login(command).await?;

    let response = AuthResponseDto {
        token: result.token,
        user: UserDto::from(&result.user),
    };

    Ok((StatusCode::OK, Json(response)))
}

// ============ Post Handlers ============

pub async fn create_post(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreatePostDto>,
) -> Result<impl IntoResponse, DomainError> {
    let command = CreatePostCommand {
        title: dto.title,
        content: dto.content,
    };

    let post = state
        .blog_service
        .create_post(user.user_id, command)
        .await?;

    Ok((StatusCode::CREATED, Json(PostDto::from(post))))
}

pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, DomainError> {
    let post = state.blog_service.get_post(id).await?;
    Ok((StatusCode::OK, Json(PostDto::from(post))))
}

pub async fn update_post(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(dto): Json<UpdatePostDto>,
) -> Result<impl IntoResponse, DomainError> {
    let command = UpdatePostCommand {
        title: dto.title,
        content: dto.content,
    };

    let post = state
        .blog_service
        .update_post(id, user.user_id, command)
        .await?;

    Ok((StatusCode::OK, Json(PostDto::from(post))))
}

pub async fn delete_post(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, DomainError> {
    state.blog_service.delete_post(id, user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ListPostsQuery {
    pub limit: i64,
    pub offset: i64,
}

impl Default for ListPostsQuery {
    fn default() -> Self {
        Self {
            limit: 10,
            offset: 0,
        }
    }
}

pub async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<ListPostsQuery>,
) -> Result<impl IntoResponse, DomainError> {
    let (posts, total) = state
        .blog_service
        .list_posts(query.limit, query.offset)
        .await?;

    let response = PostsListDto {
        posts: posts.into_iter().map(PostDto::from).collect(),
        total,
        limit: query.limit,
        offset: query.offset,
    };

    Ok((StatusCode::OK, Json(response)))
}

// ============ Health Check ============

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub timestamp: String,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

// ============ Router ============

pub fn router(state: AppState, config: ServerConfig) -> Router {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(config.rate_limit_per_second)
            .burst_size(config.rate_limit_burst)
            .finish()
            .expect("Failed to build rate limit config"),
    );

    let auth_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login));

    let posts_routes = Router::new()
        .route("/", get(list_posts))
        .route("/", post(create_post))
        .route("/{id}", get(get_post))
        .route("/{id}", put(update_post))
        .route("/{id}", delete(delete_post));

    let api_v1 = Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth_routes)
        .nest("/posts", posts_routes)
        .with_state(state);

    Router::new()
        .nest("/api/v1", api_v1)
        .layer(GovernorLayer::new(governor_conf))
        .layer(TraceLayer::new_for_http())
}
