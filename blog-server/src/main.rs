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
use crate::infrastructure::{JwtService, create_pool, run_migrations};
use crate::presentation::{
    AppState, BlogGrpcService, create_router,
    proto::blog_service_server::BlogServiceServer,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting blog server...");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get configuration from environment
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret =
        std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    // Create database pool
    tracing::info!("Connecting to database...");
    let pool = create_pool(&database_url).await?;

    // Run migrations
    tracing::info!("Running migrations...");
    run_migrations(&pool).await?;

    // Initialize services
    let jwt_service = Arc::new(JwtService::new(&jwt_secret));
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
    ));

    let grpc_handle =
        tokio::spawn(run_grpc_server(auth_service, blog_service, jwt_service));

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
) -> Result<()> {
    use axum::Extension;
    use std::net::SocketAddr;
    use tower_http::cors::{Any, CorsLayer};
    use tower_http::trace::TraceLayer;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState {
        auth_service,
        blog_service,
    };

    let app = create_router(state)
        .layer(Extension(jwt_service))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn run_grpc_server(
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
) -> Result<()> {
    use std::net::SocketAddr;
    use tonic::transport::Server;

    let addr: SocketAddr = "0.0.0.0:50051".parse()?;
    tracing::info!("gRPC server listening on {}", addr);

    let grpc_service =
        BlogGrpcService::new(auth_service, blog_service, jwt_service);

    Server::builder()
        .add_service(BlogServiceServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
