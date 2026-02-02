use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use super::config::{FromEnv, env_or, env_required};

#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl FromEnv for DatabaseConfig {
    fn from_env() -> Self {
        Self {
            url: env_required("DATABASE_URL"),
            max_connections: env_or("DATABASE_MAX_CONNECTIONS", 5),
        }
    }
}

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn run_migrations(
        &self,
    ) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
