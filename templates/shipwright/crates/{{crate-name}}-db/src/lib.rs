//! Database layer for {{project-name}}
//!
//! This crate provides database connectivity and data access patterns.

use {{crate-name | replace(from="-", to="_")}}_config::Config;
use sqlx::{{database-type | title}}Pool;
use std::time::Duration;

pub type DbPool = {{database-type | title}}Pool;

/// Database connection manager
pub struct Database {
    pool: DbPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let pool = sqlx::{{database-type | title}}Pool::connect_with(
            sqlx::{{database-type | title}}ConnectOptions::new()
                .host(&config.database.url)
                .max_connections(config.database.max_connections)
                .idle_timeout(Duration::from_secs(30))
                .acquire_timeout(Duration::from_secs(3))
        )
        .await?;

        Ok(Self { pool })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    /// Run database migrations
    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }

    /// Check if the database is healthy
    pub async fn health_check(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// Common database error types
#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(#[from] sqlx::Error),
    
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

/// Database repository trait
pub trait Repository {
    type Entity;
    type Id;
    
    /// Find entity by ID
    async fn find_by_id(&self, id: Self::Id) -> anyhow::Result<Option<Self::Entity>>;
    
    /// Find all entities
    async fn find_all(&self) -> anyhow::Result<Vec<Self::Entity>>;
    
    /// Save entity
    async fn save(&self, entity: &Self::Entity) -> anyhow::Result<Self::Entity>;
    
    /// Delete entity by ID
    async fn delete(&self, id: Self::Id) -> anyhow::Result<bool>;
}