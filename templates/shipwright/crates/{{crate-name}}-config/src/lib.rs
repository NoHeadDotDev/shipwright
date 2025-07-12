//! Configuration management for {{project-name}}
//!
//! This crate provides a robust configuration system that supports:
//! - Environment-specific configuration files (development.toml, production.toml, test.toml)
//! - Environment variable overrides
//! - Configuration validation
//! - Default values and fallbacks
//!
//! # Usage
//!
//! ```rust
//! use {{crate_name}}_config::{Config, Environment, ConfigError};
//!
//! // Load configuration for the current environment
//! let config = Config::load(Environment::Development)?;
//!
//! // Access configuration values
//! println!("Database URL: {}", config.database.url);
//! println!("Server port: {}", config.server.port);
//! ```

pub mod utils;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Failed to read configuration file: {source}")]
    ReadError { source: std::io::Error },
    
    #[error("Failed to parse configuration: {source}")]
    ParseError { source: toml::de::Error },
    
    #[error("Invalid configuration: {message}")]
    ValidationError { message: String },
    
    #[error("Environment variable error: {source}")]
    EnvError { source: env::VarError },
    
    #[error("Path error: {message}")]
    PathError { message: String },
}

/// Application environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Environment {
    /// Get the current environment from the `{{upper_case_name}}_ENV` environment variable
    pub fn current() -> Self {
        match env::var("{{upper_case_name}}_ENV") {
            Ok(env_str) => match env_str.to_lowercase().as_str() {
                "production" | "prod" => Environment::Production,
                "test" | "testing" => Environment::Test,
                _ => Environment::Development,
            },
            Err(_) => Environment::Development,
        }
    }
    
    /// Get the configuration file name for this environment
    pub fn config_filename(&self) -> &'static str {
        match self {
            Environment::Development => "development.toml",
            Environment::Production => "production.toml",
            Environment::Test => "test.toml",
        }
    }
    
    /// Check if this is a production environment
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }
    
    /// Check if this is a development environment
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }
    
    /// Check if this is a test environment
    pub fn is_test(&self) -> bool {
        matches!(self, Environment::Test)
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Production => write!(f, "production"),
            Environment::Test => write!(f, "test"),
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Maximum connection lifetime in seconds
    pub max_lifetime: u64,
    /// Enable SQL query logging
    pub log_queries: bool,
    /// Enable database migrations on startup
    pub auto_migrate: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:{{crate_name}}.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: 30,
            max_lifetime: 3600,
            log_queries: false,
            auto_migrate: true,
        }
    }
}

/// Web server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable CORS
    pub cors_enabled: bool,
    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Maximum request body size in bytes
    pub max_body_size: usize,
    /// Enable gzip compression
    pub compression: bool,
    /// Static files directory
    pub static_dir: Option<String>,
    /// Enable development mode features
    pub dev_mode: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_enabled: true,
            cors_origins: vec!["*".to_string()],
            request_timeout: 30,
            max_body_size: 1024 * 1024 * 10, // 10MB
            compression: true,
            static_dir: None,
            dev_mode: false,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (json, pretty, compact)
    pub format: String,
    /// Enable file logging
    pub file_enabled: bool,
    /// Log file path
    pub file_path: Option<String>,
    /// Maximum log file size in bytes
    pub max_file_size: u64,
    /// Number of log files to keep
    pub max_files: u32,
    /// Enable console logging
    pub console_enabled: bool,
    /// Enable structured logging
    pub structured: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            file_enabled: false,
            file_path: None,
            max_file_size: 1024 * 1024 * 100, // 100MB
            max_files: 10,
            console_enabled: true,
            structured: false,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// JWT secret key
    pub jwt_secret: String,
    /// JWT expiration time in seconds
    pub jwt_expiration: u64,
    /// Enable HTTPS redirect
    pub https_redirect: bool,
    /// Enable secure cookies
    pub secure_cookies: bool,
    /// Session timeout in seconds
    pub session_timeout: u64,
    /// Enable rate limiting
    pub rate_limiting: bool,
    /// Rate limit: requests per minute
    pub rate_limit_rpm: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-secret-key-change-me".to_string(),
            jwt_expiration: 3600 * 24, // 24 hours
            https_redirect: false,
            secure_cookies: false,
            session_timeout: 3600 * 2, // 2 hours
            rate_limiting: false,
            rate_limit_rpm: 60,
        }
    }
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Current environment
    pub environment: Environment,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Server configuration
    pub server: ServerConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Application-specific settings
    pub app: AppConfig,
}

/// Application-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Debug mode
    pub debug: bool,
    /// Feature flags
    pub features: std::collections::HashMap<String, bool>,
    /// Custom settings
    pub custom: std::collections::HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "{{project-name}}".to_string(),
            version: "0.1.0".to_string(),
            debug: false,
            features: std::collections::HashMap::new(),
            custom: std::collections::HashMap::new(),
        }
    }
}

impl Config {
    /// Load configuration for the specified environment
    pub fn load(environment: Environment) -> Result<Self, ConfigError> {
        info!("Loading configuration for environment: {}", environment);
        
        let mut config = Self::default_for_environment(environment);
        
        // Find and load configuration file
        let config_path = Self::find_config_file(environment)?;
        debug!("Loading configuration from: {}", config_path.display());
        
        let content = fs::read_to_string(&config_path)
            .map_err(|e| ConfigError::ReadError { source: e })?;
        
        // Parse TOML configuration
        let file_config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError { source: e })?;
        
        // Merge file configuration with defaults
        config.merge_with(file_config);
        
        // Apply environment variable overrides
        config.apply_env_overrides()?;
        
        // Validate configuration
        config.validate()?;
        
        info!("Configuration loaded successfully");
        Ok(config)
    }
    
    /// Load configuration for the current environment
    pub fn load_current() -> Result<Self, ConfigError> {
        Self::load(Environment::current())
    }
    
    /// Create default configuration for the specified environment
    fn default_for_environment(environment: Environment) -> Self {
        let mut config = Self::default();
        config.environment = environment;
        
        // Apply environment-specific defaults
        match environment {
            Environment::Development => {
                config.server.dev_mode = true;
                config.logging.level = "debug".to_string();
                config.app.debug = true;
            }
            Environment::Production => {
                config.server.dev_mode = false;
                config.logging.level = "info".to_string();
                config.logging.structured = true;
                config.security.https_redirect = true;
                config.security.secure_cookies = true;
                config.app.debug = false;
            }
            Environment::Test => {
                config.server.port = 0; // Use random port for tests
                config.logging.level = "warn".to_string();
                config.database.url = ":memory:".to_string();
                config.app.debug = true;
            }
        }
        
        config
    }
    
    /// Find the configuration file for the specified environment
    fn find_config_file(environment: Environment) -> Result<PathBuf, ConfigError> {
        let filename = environment.config_filename();
        
        // Search paths in order of preference
        let search_paths = [
            // Current directory
            PathBuf::from("config").join(filename),
            // Parent directory
            PathBuf::from("../config").join(filename),
            // Project root
            PathBuf::from("../../config").join(filename),
            // Home directory
            dirs::home_dir()
                .map(|home| home.join(".config").join("{{crate_name}}").join(filename))
                .unwrap_or_else(|| PathBuf::from(filename)),
        ];
        
        for path in &search_paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }
        
        Err(ConfigError::FileNotFound {
            path: filename.to_string(),
        })
    }
    
    /// Merge this configuration with another configuration
    fn merge_with(&mut self, other: Config) {
        // Note: This is a simplified merge - in practice, you might want
        // more sophisticated merging logic
        self.database = other.database;
        self.server = other.server;
        self.logging = other.logging;
        self.security = other.security;
        self.app = other.app;
    }
    
    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        // Database overrides
        if let Ok(url) = env::var("{{upper_case_name}}_DATABASE_URL") {
            self.database.url = url;
        }
        
        // Server overrides
        if let Ok(port) = env::var("{{upper_case_name}}_SERVER_PORT") {
            self.server.port = port.parse().unwrap_or(self.server.port);
        }
        
        if let Ok(host) = env::var("{{upper_case_name}}_SERVER_HOST") {
            self.server.host = host;
        }
        
        // Logging overrides
        if let Ok(level) = env::var("{{upper_case_name}}_LOG_LEVEL") {
            self.logging.level = level;
        }
        
        // Security overrides
        if let Ok(secret) = env::var("{{upper_case_name}}_JWT_SECRET") {
            self.security.jwt_secret = secret;
        }
        
        Ok(())
    }
    
    /// Validate the configuration
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate database configuration
        if self.database.url.is_empty() {
            return Err(ConfigError::ValidationError {
                message: "Database URL cannot be empty".to_string(),
            });
        }
        
        if self.database.max_connections == 0 {
            return Err(ConfigError::ValidationError {
                message: "Database max_connections must be greater than 0".to_string(),
            });
        }
        
        // Validate server configuration
        if self.server.port == 0 && !self.environment.is_test() {
            return Err(ConfigError::ValidationError {
                message: "Server port cannot be 0 in non-test environments".to_string(),
            });
        }
        
        // Validate logging configuration
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::ValidationError {
                message: format!("Invalid log level: {}", self.logging.level),
            });
        }
        
        // Validate security configuration
        if self.environment.is_production() && self.security.jwt_secret == "your-secret-key-change-me" {
            return Err(ConfigError::ValidationError {
                message: "JWT secret must be changed in production".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Get the database URL with environment variable override
    pub fn database_url(&self) -> String {
        env::var("{{upper_case_name}}_DATABASE_URL").unwrap_or_else(|_| self.database.url.clone())
    }
    
    /// Get the server bind address
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
    
    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.app.features.get(feature).copied().unwrap_or(false)
    }
    
    /// Get a custom setting
    pub fn get_custom_setting(&self, key: &str) -> Option<&String> {
        self.app.custom.get(key)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            environment: Environment::Development,
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
            app: AppConfig::default(),
        }
    }
}

/// Global configuration instance
static CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::load_current().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {}", e);
        Config::default()
    })
});

/// Get the global configuration instance
pub fn get_config() -> &'static Config {
    &CONFIG
}

/// Initialize the global configuration with a specific environment
pub fn init_config(environment: Environment) -> Result<(), ConfigError> {
    // Force re-evaluation of the lazy static
    std::env::set_var("{{upper_case_name}}_ENV", environment.to_string());
    let _config = Config::load(environment)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_environment_current() {
        // Test default environment
        std::env::remove_var("{{upper_case_name}}_ENV");
        assert_eq!(Environment::current(), Environment::Development);
        
        // Test production environment
        std::env::set_var("{{upper_case_name}}_ENV", "production");
        assert_eq!(Environment::current(), Environment::Production);
        
        // Test test environment
        std::env::set_var("{{upper_case_name}}_ENV", "test");
        assert_eq!(Environment::current(), Environment::Test);
        
        // Clean up
        std::env::remove_var("{{upper_case_name}}_ENV");
    }
    
    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.environment, Environment::Development);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Invalid database URL should fail
        config.database.url = "".to_string();
        assert!(config.validate().is_err());
        
        // Invalid max_connections should fail
        config.database.url = "sqlite:test.db".to_string();
        config.database.max_connections = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("config");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_file = config_dir.join("development.toml");
        let config_content = r#"
[database]
url = "sqlite:test.db"
max_connections = 5

[server]
port = 3000
host = "0.0.0.0"

[logging]
level = "debug"

[security]
jwt_secret = "test-secret"

[app]
name = "Test App"
debug = true
"#;
        
        fs::write(&config_file, config_content).unwrap();
        
        // Change to temp directory to test config loading
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let config = Config::load(Environment::Development).unwrap();
        
        assert_eq!(config.database.url, "sqlite:test.db");
        assert_eq!(config.database.max_connections, 5);
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.security.jwt_secret, "test-secret");
        assert_eq!(config.app.name, "Test App");
        assert!(config.app.debug);
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
    
    #[test]
    fn test_env_overrides() {
        let mut config = Config::default();
        
        // Set environment variables
        std::env::set_var("{{upper_case_name}}_DATABASE_URL", "postgres://localhost/test");
        std::env::set_var("{{upper_case_name}}_SERVER_PORT", "9000");
        std::env::set_var("{{upper_case_name}}_LOG_LEVEL", "error");
        
        config.apply_env_overrides().unwrap();
        
        assert_eq!(config.database.url, "postgres://localhost/test");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.logging.level, "error");
        
        // Clean up
        std::env::remove_var("{{upper_case_name}}_DATABASE_URL");
        std::env::remove_var("{{upper_case_name}}_SERVER_PORT");
        std::env::remove_var("{{upper_case_name}}_LOG_LEVEL");
    }
}