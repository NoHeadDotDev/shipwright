use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use crate::error::ShipwrightError;

/// Main configuration structure for Shipwright.toml
/// 
/// Example Shipwright.toml:
/// ```toml
/// [application]
/// name = "my-app"
/// version = "0.1.0"
/// default_platform = "web"
/// 
/// [web]
/// title = "My Shipwright App"
/// favicon = "public/favicon.ico"
/// base_path = "/"
/// 
/// [build]
/// target_dir = "target"
/// out_dir = "dist"
/// 
/// [serve]
/// host = "localhost"
/// port = 8080
/// 
/// [hot_reload]
/// enabled = true
/// watch_paths = ["src", "assets"]
/// ignore_paths = ["target", "dist"]
/// 
/// [workspace]
/// members = ["app", "shared"]
/// exclude = ["examples"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub application: ApplicationConfig,
    pub web: Option<WebConfig>,
    pub build: Option<BuildConfig>,
    pub serve: Option<ServeConfig>,
    pub hot_reload: Option<HotReloadConfig>,
    pub workspace: Option<WorkspaceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: Option<String>,
    pub default_platform: Option<String>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub title: Option<String>,
    pub favicon: Option<PathBuf>,
    pub base_path: Option<String>,
    pub index_template: Option<PathBuf>,
    pub public_dir: Option<PathBuf>,
    pub assets_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub target_dir: Option<PathBuf>,
    pub out_dir: Option<PathBuf>,
    pub release: Option<bool>,
    pub features: Option<Vec<String>>,
    pub target: Option<String>,
    pub environment: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServeConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub static_dir: Option<PathBuf>,
    pub cors: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadConfig {
    pub enabled: Option<bool>,
    pub watch_paths: Option<Vec<PathBuf>>,
    pub ignore_paths: Option<Vec<PathBuf>>,
    pub poll_interval: Option<u64>,
    pub debounce_ms: Option<u64>,
    pub reload_css: Option<bool>,
    pub reload_js: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub members: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub default_members: Option<Vec<String>>,
}

impl Config {
    /// Load configuration from Shipwright.toml file
    pub fn load(path: &Path) -> Result<Self, ShipwrightError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| ShipwrightError::TomlError(e.to_string()))?;
        Ok(config)
    }

    /// Save configuration to Shipwright.toml file
    pub fn save(&self, path: &Path) -> Result<(), ShipwrightError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ShipwrightError::TomlError(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the web configuration with defaults
    pub fn web(&self) -> WebConfig {
        self.web.clone().unwrap_or_default()
    }

    /// Get the build configuration with defaults
    pub fn build(&self) -> BuildConfig {
        self.build.clone().unwrap_or_default()
    }

    /// Get the serve configuration with defaults
    pub fn serve(&self) -> ServeConfig {
        self.serve.clone().unwrap_or_default()
    }

    /// Get the hot reload configuration with defaults
    pub fn hot_reload(&self) -> HotReloadConfig {
        self.hot_reload.clone().unwrap_or_default()
    }

    /// Get the workspace configuration with defaults
    pub fn workspace(&self) -> WorkspaceConfig {
        self.workspace.clone().unwrap_or_default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            application: ApplicationConfig {
                name: "shipwright-app".to_string(),
                version: Some("0.1.0".to_string()),
                default_platform: Some("web".to_string()),
                authors: None,
                description: None,
            },
            web: None,
            build: None,
            serve: None,
            hot_reload: None,
            workspace: None,
        }
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            title: Some("Shipwright App".to_string()),
            favicon: Some(PathBuf::from("public/favicon.ico")),
            base_path: Some("/".to_string()),
            index_template: Some(PathBuf::from("index.html")),
            public_dir: Some(PathBuf::from("public")),
            assets_dir: Some(PathBuf::from("assets")),
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target_dir: Some(PathBuf::from("target")),
            out_dir: Some(PathBuf::from("dist")),
            release: Some(false),
            features: None,
            target: None,
            environment: None,
        }
    }
}

impl Default for ServeConfig {
    fn default() -> Self {
        Self {
            host: Some("localhost".to_string()),
            port: Some(8080),
            static_dir: Some(PathBuf::from("dist")),
            cors: Some(true),
        }
    }
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled: Some(true),
            watch_paths: Some(vec![
                PathBuf::from("src"),
                PathBuf::from("assets"),
                PathBuf::from("public"),
            ]),
            ignore_paths: Some(vec![
                PathBuf::from("target"),
                PathBuf::from("dist"),
                PathBuf::from(".git"),
                PathBuf::from("node_modules"),
            ]),
            poll_interval: Some(1000),
            debounce_ms: Some(300),
            reload_css: Some(true),
            reload_js: Some(true),
        }
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            members: None,
            exclude: None,
            default_members: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.application.name, "shipwright-app");
        assert_eq!(config.application.version, Some("0.1.0".to_string()));
        assert_eq!(config.application.default_platform, Some("web".to_string()));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(config.application.name, deserialized.application.name);
    }

    #[test]
    fn test_config_file_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config::default();
        
        // Test save
        config.save(temp_file.path()).unwrap();
        
        // Test load
        let loaded_config = Config::load(temp_file.path()).unwrap();
        assert_eq!(config.application.name, loaded_config.application.name);
    }

    #[test]
    fn test_missing_config_file() {
        let non_existent_path = Path::new("non_existent_config.toml");
        let config = Config::load(non_existent_path).unwrap();
        assert_eq!(config.application.name, "shipwright-app");
    }
}