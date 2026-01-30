use std::str::FromStr;

pub trait FromEnv: Sized {
    fn from_env() -> Self;
}

/// Helper to read a value from environment variable with a default
pub fn env_or<T: FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Helper to read a required value from environment variable
pub fn env_required(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} must be set"))
}

#[derive(Clone)]
pub struct ServerConfig {
    pub rate_limit_per_second: u64,
    pub rate_limit_burst: u32,
}

impl FromEnv for ServerConfig {
    fn from_env() -> Self {
        Self {
            rate_limit_per_second: env_or("RATE_LIMIT_PER_SECOND", 10),
            rate_limit_burst: env_or("RATE_LIMIT_BURST", 20),
        }
    }
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

impl FromEnv for DatabaseConfig {
    fn from_env() -> Self {
        Self {
            url: env_required("DATABASE_URL"),
        }
    }
}

#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
}

impl FromEnv for JwtConfig {
    fn from_env() -> Self {
        Self {
            secret: env_required("JWT_SECRET"),
        }
    }
}
