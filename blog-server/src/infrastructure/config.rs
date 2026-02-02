use std::str::FromStr;

/// Trait for loading configuration from environment variables.
/// Each module defines its own config struct and implements this trait.
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
