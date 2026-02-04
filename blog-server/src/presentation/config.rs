use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::infrastructure::config::{FromEnv, env_or, env_required};

#[derive(Clone, Copy)]
pub struct ServerConfig {
    pub http_host: IpAddr,
    pub http_port: u16,
    pub grpc_host: IpAddr,
    pub grpc_port: u16,
    pub rate_limit_per_second: u64,
    pub rate_limit_burst: u32,
}

impl ServerConfig {
    pub const fn http_addr(&self) -> SocketAddr {
        SocketAddr::new(self.http_host, self.http_port)
    }

    pub const fn grpc_addr(&self) -> SocketAddr {
        SocketAddr::new(self.grpc_host, self.grpc_port)
    }
}

impl FromEnv for ServerConfig {
    fn from_env() -> Self {
        let default_host = IpAddr::V4(Ipv4Addr::UNSPECIFIED);

        Self {
            http_host: env_or("HTTP_HOST", default_host),
            http_port: env_or("HTTP_PORT", 3000),
            grpc_host: env_or("GRPC_HOST", default_host),
            grpc_port: env_or("GRPC_PORT", 50051),
            rate_limit_per_second: env_or("RATE_LIMIT_PER_SECOND", 10),
            rate_limit_burst: env_or("RATE_LIMIT_BURST", 20),
        }
    }
}

#[derive(Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub max_age_secs: u64,
}

impl FromEnv for CorsConfig {
    fn from_env() -> Self {
        let origins = env_required("CORS_ALLOWED_ORIGINS")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            allowed_origins: origins,
            max_age_secs: env_or("CORS_MAX_AGE", 3600),
        }
    }
}

#[derive(Clone)]
pub struct PaginationConfig {
    pub default_limit: i64,
    pub max_limit: i64,
}

impl FromEnv for PaginationConfig {
    fn from_env() -> Self {
        Self {
            default_limit: env_or("PAGINATION_DEFAULT_LIMIT", 10),
            max_limit: env_or("PAGINATION_MAX_LIMIT", 100),
        }
    }
}
