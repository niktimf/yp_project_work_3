// Presentation layer - HTTP handlers, gRPC service, middleware

pub mod dto;
pub mod grpc_service;
pub mod http_handlers;
pub mod middleware;

pub use grpc_service::{BlogGrpcService, proto};
pub use http_handlers::{AppState, router};
