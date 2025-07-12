use cargo_metadata::{MetadataCommand, Package, Metadata};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::error::ShipwrightError;
use crate::config::Config;

/// Workspace detection and management for Shipwright projects
#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub metadata: Metadata,
    pub target_package: Option<Package>,
    pub is_workspace: bool,
    pub members: Vec<Package>,
}

impl Workspace {
    /// Detect and load workspace information from the current directory
    pub fn detect(cwd: Option<&Path>) -> Result<Self, ShipwrightError> {
        let working_dir = cwd.unwrap_or_else(|| Path::new("."));
        
        debug!("Detecting workspace from: {}", working_dir.display());

        let mut cmd = MetadataCommand::new();
        cmd.current_dir(working_dir);
        
        let metadata = cmd.exec()?;
        
        let is_workspace = metadata.workspace_members.len() > 1;
        let root = metadata.workspace_root.clone();
        
        info!("Workspace detected: {} (root: {})", 
              if is_workspace { "multi-crate" } else { "single-crate" }, 
              root.as_str());

        // Get all workspace members
        let mut members = Vec::new();
        for member_id in &metadata.workspace_members {
            if let Some(package) = metadata.packages.iter().find(|p| &p.id == member_id) {
                members.push(package.clone());
            }
        }

        // Try to detect the target package (the one being built)
        let target_package = Self::detect_target_package(&metadata, working_dir)?;
        
        if let Some(ref package) = target_package {
            info!("Target package: {}", package.name);
        }

        Ok(Workspace {
            root: root.into_std_path_buf(),
            metadata,
            target_package,
            is_workspace,
            members,
        })
    }

    /// Detect the target package based on the current working directory
    fn detect_target_package(metadata: &Metadata, working_dir: &Path) -> Result<Option<Package>, ShipwrightError> {
        let current_dir = working_dir.canonicalize()?;
        
        // Find the package that contains the current working directory
        for package in &metadata.packages {
            let package_dir = package.manifest_path.parent().unwrap().as_std_path();
            if current_dir.starts_with(package_dir) {
                return Ok(Some(package.clone()));
            }
        }
        
        // If no specific package found, use the first workspace member
        if !metadata.workspace_members.is_empty() {
            let first_member = &metadata.workspace_members[0];
            if let Some(package) = metadata.packages.iter().find(|p| &p.id == first_member) {
                return Ok(Some(package.clone()));
            }
        }
        
        Ok(None)
    }

    /// Get all binary targets in the workspace
    pub fn get_binary_targets(&self) -> Vec<(String, PathBuf)> {
        let mut binaries = Vec::new();
        
        for package in &self.members {
            for target in &package.targets {
                if target.kind.contains(&"bin".to_string()) {
                    binaries.push((
                        format!("{}::{}", package.name, target.name),
                        target.src_path.clone().into_std_path_buf(),
                    ));
                }
            }
        }
        
        binaries
    }

    /// Get all library targets in the workspace
    pub fn get_library_targets(&self) -> Vec<(String, PathBuf)> {
        let mut libraries = Vec::new();
        
        for package in &self.members {
            for target in &package.targets {
                if target.kind.contains(&"lib".to_string()) {
                    libraries.push((
                        package.name.clone(),
                        target.src_path.clone().into_std_path_buf(),
                    ));
                }
            }
        }
        
        libraries
    }

    /// Get watch paths for the workspace
    pub fn get_watch_paths(&self, config: &Config) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // Add configured watch paths
        if let Some(watch_paths) = &config.hot_reload().watch_paths {
            for path in watch_paths {
                if path.is_absolute() {
                    paths.push(path.clone());
                } else {
                    paths.push(self.root.join(path));
                }
            }
        }

        // Add source directories from all workspace members
        for package in &self.members {
            let package_dir = package.manifest_path.parent().unwrap().as_std_path();
            
            // Add src directory
            let src_dir = package_dir.join("src");
            if src_dir.exists() {
                paths.push(src_dir);
            }
            
            // Add assets directory if it exists
            let assets_dir = package_dir.join("assets");
            if assets_dir.exists() {
                paths.push(assets_dir);
            }
        }

        // Remove duplicates and sort
        paths.sort();
        paths.dedup();
        
        debug!("Watch paths: {:?}", paths);
        paths
    }

    /// Get ignore paths for the workspace
    pub fn get_ignore_paths(&self, config: &Config) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // Add configured ignore paths
        if let Some(ignore_paths) = &config.hot_reload().ignore_paths {
            for path in ignore_paths {
                if path.is_absolute() {
                    paths.push(path.clone());
                } else {
                    paths.push(self.root.join(path));
                }
            }
        }

        // Add common ignore paths
        paths.push(self.root.join("target"));
        paths.push(self.root.join("dist"));
        paths.push(self.root.join(".git"));
        paths.push(self.root.join("node_modules"));
        
        // Remove duplicates and sort
        paths.sort();
        paths.dedup();
        
        debug!("Ignore paths: {:?}", paths);
        paths
    }

    /// Get the main package for building
    pub fn get_main_package(&self) -> Option<&Package> {
        self.target_package.as_ref()
    }

    /// Get package by name
    pub fn get_package_by_name(&self, name: &str) -> Option<&Package> {
        self.members.iter().find(|p| p.name == name)
    }

    /// Get build environment variables
    pub fn get_build_env(&self, config: &Config) -> HashMap<String, String> {
        let mut env = HashMap::new();
        
        // Add workspace root
        env.insert("SHIPWRIGHT_WORKSPACE_ROOT".to_string(), self.root.display().to_string());
        
        // Add target directory
        if let Some(target_dir) = &config.build().target_dir {
            env.insert("CARGO_TARGET_DIR".to_string(), self.root.join(target_dir).display().to_string());
        }
        
        // Add custom environment variables from config
        if let Some(custom_env) = &config.build().environment {
            for (key, value) in custom_env {
                env.insert(key.clone(), value.clone());
            }
        }
        
        env
    }

    /// Check if the workspace contains a specific package
    pub fn contains_package(&self, name: &str) -> bool {
        self.members.iter().any(|p| p.name == name)
    }

    /// Get dependency graph for the workspace
    pub fn get_dependency_graph(&self) -> HashMap<String, Vec<String>> {
        let mut graph = HashMap::new();
        
        for package in &self.members {
            let mut deps = Vec::new();
            
            for dep in &package.dependencies {
                // Only include workspace dependencies
                if self.contains_package(&dep.name) {
                    deps.push(dep.name.clone());
                }
            }
            
            graph.insert(package.name.clone(), deps);
        }
        
        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_workspace() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path();
        
        // Create workspace Cargo.toml
        let workspace_toml = r#"
[workspace]
members = ["app", "shared"]

[workspace.dependencies]
serde = "1.0"
"#;
        fs::write(workspace_root.join("Cargo.toml"), workspace_toml).unwrap();
        
        // Create app package
        fs::create_dir(workspace_root.join("app")).unwrap();
        let app_toml = r#"
[package]
name = "app"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "app"
path = "src/main.rs"

[dependencies]
shared = { path = "../shared" }
"#;
        fs::write(workspace_root.join("app/Cargo.toml"), app_toml).unwrap();
        fs::create_dir(workspace_root.join("app/src")).unwrap();
        fs::write(workspace_root.join("app/src/main.rs"), "fn main() {}").unwrap();
        
        // Create shared package
        fs::create_dir(workspace_root.join("shared")).unwrap();
        let shared_toml = r#"
[package]
name = "shared"
version = "0.1.0"
edition = "2021"

[lib]
name = "shared"
path = "src/lib.rs"
"#;
        fs::write(workspace_root.join("shared/Cargo.toml"), shared_toml).unwrap();
        fs::create_dir(workspace_root.join("shared/src")).unwrap();
        fs::write(workspace_root.join("shared/src/lib.rs"), "").unwrap();
        
        temp_dir
    }

    #[test]
    fn test_workspace_detection() {
        let temp_dir = create_test_workspace();
        let workspace = Workspace::detect(Some(temp_dir.path())).unwrap();
        
        assert!(workspace.is_workspace);
        assert_eq!(workspace.members.len(), 2);
        assert!(workspace.contains_package("app"));
        assert!(workspace.contains_package("shared"));
    }

    #[test]
    fn test_binary_targets() {
        let temp_dir = create_test_workspace();
        let workspace = Workspace::detect(Some(temp_dir.path())).unwrap();
        
        let binaries = workspace.get_binary_targets();
        assert_eq!(binaries.len(), 1);
        assert_eq!(binaries[0].0, "app::app");
    }

    #[test]
    fn test_library_targets() {
        let temp_dir = create_test_workspace();
        let workspace = Workspace::detect(Some(temp_dir.path())).unwrap();
        
        let libraries = workspace.get_library_targets();
        assert_eq!(libraries.len(), 1);
        assert_eq!(libraries[0].0, "shared");
    }

    #[test]
    fn test_dependency_graph() {
        let temp_dir = create_test_workspace();
        let workspace = Workspace::detect(Some(temp_dir.path())).unwrap();
        
        let graph = workspace.get_dependency_graph();
        assert_eq!(graph.len(), 2);
        assert_eq!(graph["app"], vec!["shared"]);
        assert_eq!(graph["shared"], Vec::<String>::new());
    }
}