use std::sync::Arc;
use {{crate_name}}_config::Config;
use {{crate_name}}_db::DatabasePool;

/// Application state that's shared across all request handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db_pool: DatabasePool,
}

impl AppState {
    /// Create a new AppState instance
    pub fn new(config: Config, db_pool: DatabasePool) -> Self {
        Self {
            config: Arc::new(config),
            db_pool,
        }
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get a reference to the database pool
    pub fn db_pool(&self) -> &DatabasePool {
        &self.db_pool
    }
}