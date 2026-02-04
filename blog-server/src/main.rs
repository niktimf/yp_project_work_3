mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::application::{AuthService, BlogService};
use crate::data::{PostgresPostRepository, PostgresUserRepository};
use crate::infrastructure::{
    Database, DatabaseConfig, FromEnv, JwtConfig, JwtService,
};
use crate::presentation::{
    AppState, BlogGrpcService, CorsConfig, PaginationConfig, ServerConfig,
    proto::blog_service_server::BlogServiceServer, router,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables first so .env values are available everywhere
    dotenvy::dotenv().ok();

    // Initialize tracing (reads RUST_LOG from env)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting blog server...");

    // Load configuration from environment
    let db_config = DatabaseConfig::from_env();
    let jwt_config = JwtConfig::from_env();
    let server_config = ServerConfig::from_env();
    let cors_config = CorsConfig::from_env();
    let pagination_config = PaginationConfig::from_env();

    // Create database connection
    tracing::info!("Connecting to database...");
    let database = Database::new(&db_config).await?;

    // Run migrations
    tracing::info!("Running migrations...");
    database.run_migrations().await?;

    let pool = database.pool().clone();

    // Initialize services
    let jwt_service = Arc::new(JwtService::new(&jwt_config));
    let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));
    let post_repository = Arc::new(PostgresPostRepository::new(pool.clone()));

    let auth_service =
        Arc::new(AuthService::new(user_repository, jwt_service.clone()));
    let blog_service = Arc::new(BlogService::new(post_repository));

    // Start HTTP and gRPC servers
    let http_handle = tokio::spawn(run_http_server(
        auth_service.clone(),
        blog_service.clone(),
        jwt_service.clone(),
        server_config,
        cors_config,
        pagination_config.clone(),
    ));

    let grpc_handle = tokio::spawn(run_grpc_server(
        auth_service,
        blog_service,
        jwt_service,
        server_config,
        pagination_config,
    ));

    // Wait for both servers
    tokio::select! {
        result = http_handle => {
            if let Err(e) = result {
                tracing::error!("HTTP server error: {}", e);
            }
        }
        result = grpc_handle => {
            if let Err(e) = result {
                tracing::error!("gRPC server error: {}", e);
            }
        }
    }

    Ok(())
}

async fn run_http_server(
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
    server_config: ServerConfig,
    cors_config: CorsConfig,
    pagination_config: PaginationConfig,
) -> Result<()> {
    use axum::Extension;
    use axum::http::{HeaderValue, Method};
    use std::net::SocketAddr;
    use std::time::Duration;
    use tower_http::cors::{AllowOrigin, Any, CorsLayer};

    let origins: Vec<HeaderValue> = cors_config
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .max_age(Duration::from_secs(cors_config.max_age_secs));

    let state = AppState {
        auth_service,
        blog_service,
        pagination_config,
    };

    let addr = server_config.http_addr();

    let app = router(state, server_config)
        .layer(Extension(jwt_service))
        .layer(cors);
    tracing::info!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn run_grpc_server(
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
    server_config: ServerConfig,
    pagination_config: PaginationConfig,
) -> Result<()> {
    use tonic::transport::Server;

    let addr = server_config.grpc_addr();
    tracing::info!("gRPC server listening on {}", addr);

    let grpc_service = BlogGrpcService::new(
        auth_service,
        blog_service,
        jwt_service,
        pagination_config,
    );

    Server::builder()
        .add_service(BlogServiceServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
