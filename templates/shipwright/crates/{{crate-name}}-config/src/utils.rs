//! Configuration utilities and helpers

use crate::{Config, Environment, ConfigError};
use std::env;
use std::fs;
use std::path::Path;

/// Configuration file watcher for hot-reloading in development
#[cfg(feature = "watch")]
pub mod watch {
    use super::*;
    use notify::{Watcher, RecommendedWatcher, RecursiveMode, Result as NotifyResult};
    use std::sync::mpsc;
    use std::time::Duration;
    
    /// Watch configuration files for changes
    pub fn watch_config_files<F>(callback: F) -> NotifyResult<()>
    where
        F: Fn() + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        
        let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| {
            tx.send(res).unwrap();
        })?;
        
        watcher.watch("config", RecursiveMode::Recursive)?;
        
        for res in rx {
            match res {
                Ok(_) => {
                    tracing::info!("Configuration file changed, reloading...");
                    callback();
                }
                Err(e) => tracing::error!("Watch error: {:?}", e),
            }
        }
        
        Ok(())
    }
}

/// Environment detection utilities
pub mod env {
    use super::*;
    
    /// Detect if running in Docker
    pub fn is_docker() -> bool {
        Path::new("/.dockerenv").exists() || 
        env::var("DOCKER_CONTAINER").is_ok()
    }
    
    /// Detect if running in Kubernetes
    pub fn is_kubernetes() -> bool {
        env::var("KUBERNETES_SERVICE_HOST").is_ok()
    }
    
    /// Detect if running in CI/CD
    pub fn is_ci() -> bool {
        env::var("CI").is_ok() || 
        env::var("CONTINUOUS_INTEGRATION").is_ok() ||
        env::var("GITHUB_ACTIONS").is_ok() ||
        env::var("GITLAB_CI").is_ok() ||
        env::var("JENKINS_URL").is_ok()
    }
    
    /// Get the runtime environment info
    pub fn get_runtime_info() -> RuntimeInfo {
        RuntimeInfo {
            is_docker: is_docker(),
            is_kubernetes: is_kubernetes(),
            is_ci: is_ci(),
            hostname: gethostname::gethostname().to_string_lossy().to_string(),
            platform: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
        }
    }
    
    /// Runtime environment information
    #[derive(Debug, Clone)]
    pub struct RuntimeInfo {
        pub is_docker: bool,
        pub is_kubernetes: bool,
        pub is_ci: bool,
        pub hostname: String,
        pub platform: String,
        pub arch: String,
    }
}

/// Configuration validation utilities
pub mod validation {
    use super::*;
    use std::net::{IpAddr, SocketAddr};
    use url::Url;
    
    /// Validate a database URL
    pub fn validate_database_url(url: &str) -> Result<(), ConfigError> {
        if url.is_empty() {
            return Err(ConfigError::ValidationError {
                message: "Database URL cannot be empty".to_string(),
            });
        }
        
        // Basic URL validation
        if !url.starts_with("sqlite:") && !url.starts_with("postgresql:") && 
           !url.starts_with("mysql:") && !url.starts_with("postgres:") {
            return Err(ConfigError::ValidationError {
                message: format!("Unsupported database URL scheme: {}", url),
            });
        }
        
        Ok(())
    }
    
    /// Validate a server address
    pub fn validate_server_address(host: &str, port: u16) -> Result<(), ConfigError> {
        let addr = format!("{}:{}", host, port);
        
        addr.parse::<SocketAddr>()
            .map_err(|_| ConfigError::ValidationError {
                message: format!("Invalid server address: {}", addr),
            })?;
        
        Ok(())
    }
    
    /// Validate a URL
    pub fn validate_url(url: &str) -> Result<(), ConfigError> {
        Url::parse(url)
            .map_err(|_| ConfigError::ValidationError {
                message: format!("Invalid URL: {}", url),
            })?;
        
        Ok(())
    }
    
    /// Validate a file path exists
    pub fn validate_file_path(path: &str) -> Result<(), ConfigError> {
        if !Path::new(path).exists() {
            return Err(ConfigError::ValidationError {
                message: format!("File does not exist: {}", path),
            });
        }
        
        Ok(())
    }
    
    /// Validate a directory path exists
    pub fn validate_directory_path(path: &str) -> Result<(), ConfigError> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(ConfigError::ValidationError {
                message: format!("Directory does not exist: {}", path.display()),
            });
        }
        
        if !path.is_dir() {
            return Err(ConfigError::ValidationError {
                message: format!("Path is not a directory: {}", path.display()),
            });
        }
        
        Ok(())
    }
}

/// Configuration migration utilities
pub mod migration {
    use super::*;
    use serde_json::Value;
    
    /// Migrate configuration from old format to new format
    pub fn migrate_config(old_config: Value) -> Result<Config, ConfigError> {
        // This is a placeholder for configuration migration logic
        // In practice, you would implement version-specific migration logic here
        
        let config_str = serde_json::to_string(&old_config)
            .map_err(|e| ConfigError::ValidationError {
                message: format!("Failed to serialize config: {}", e),
            })?;
        
        // Convert JSON to TOML and then parse
        let toml_value: toml::Value = toml::from_str(&config_str)
            .map_err(|e| ConfigError::ParseError { source: e })?;
        
        let config: Config = toml_value.try_into()
            .map_err(|e| ConfigError::ValidationError {
                message: format!("Failed to migrate config: {}", e),
            })?;
        
        Ok(config)
    }
}

/// Configuration backup and restore utilities
pub mod backup {
    use super::*;
    use std::fs;
    use std::time::SystemTime;
    
    /// Create a backup of the current configuration
    pub fn backup_config(config: &Config) -> Result<String, ConfigError> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let backup_name = format!("config_backup_{}.toml", timestamp);
        let config_toml = toml::to_string_pretty(config)
            .map_err(|e| ConfigError::ValidationError {
                message: format!("Failed to serialize config: {}", e),
            })?;
        
        fs::write(&backup_name, config_toml)
            .map_err(|e| ConfigError::ReadError { source: e })?;
        
        Ok(backup_name)
    }
    
    /// Restore configuration from backup
    pub fn restore_config(backup_file: &str) -> Result<Config, ConfigError> {
        let content = fs::read_to_string(backup_file)
            .map_err(|e| ConfigError::ReadError { source: e })?;
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError { source: e })?;
        
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_database_url() {
        use validation::validate_database_url;
        
        assert!(validate_database_url("sqlite:test.db").is_ok());
        assert!(validate_database_url("postgresql://localhost/test").is_ok());
        assert!(validate_database_url("").is_err());
        assert!(validate_database_url("invalid://test").is_err());
    }
    
    #[test]
    fn test_validate_server_address() {
        use validation::validate_server_address;
        
        assert!(validate_server_address("127.0.0.1", 8080).is_ok());
        assert!(validate_server_address("0.0.0.0", 3000).is_ok());
        assert!(validate_server_address("invalid", 8080).is_err());
    }
    
    #[test]
    fn test_validate_url() {
        use validation::validate_url;
        
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("invalid-url").is_err());
    }
}