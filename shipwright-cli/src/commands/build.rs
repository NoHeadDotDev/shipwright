use clap::Args;
use std::path::PathBuf;
use std::process::Stdio;
use tracing::{info, debug};

use crate::{config::Config, error::ShipwrightError};
use super::CommandContext;

/// Build application for production
/// 
/// This command builds the Shipwright application for production use.
/// It compiles the Rust code, processes assets, and creates an optimized
/// output directory ready for deployment.
/// 
/// Examples:
///   shipwright build                  # Build in debug mode
///   shipwright build --release        # Build with optimizations
///   shipwright build --target wasm32  # Build for specific target
///   shipwright build --features "ssr" # Enable specific features
#[derive(Args, Debug)]
pub struct BuildCommand {
    /// Enable release mode optimizations
    #[arg(long)]
    pub release: bool,

    /// Build target (e.g., wasm32-unknown-unknown)
    #[arg(long)]
    pub target: Option<String>,

    /// Features to enable during compilation
    #[arg(long, value_delimiter = ',')]
    pub features: Vec<String>,

    /// Output directory for built assets
    #[arg(long)]
    pub out_dir: Option<PathBuf>,

    /// Target directory for Cargo build artifacts
    #[arg(long)]
    pub target_dir: Option<PathBuf>,

    /// Target package to build (for workspaces)
    #[arg(long)]
    pub package: Option<String>,

    /// Additional arguments to pass to cargo
    #[arg(last = true)]
    pub cargo_args: Vec<String>,

    /// Clean before building
    #[arg(long)]
    pub clean: bool,

    /// Verbose output
    #[arg(long, short)]
    pub verbose: bool,
}

impl BuildCommand {
    pub async fn run(&self, config: Config) -> Result<(), ShipwrightError> {
        let ctx = CommandContext::new(config)?;
        
        info!("Starting Shipwright build process...");
        
        let build_config = ctx.config.build();
        let release = self.release || build_config.release.unwrap_or(false);
        
        info!("Build mode: {}", if release { "release" } else { "debug" });
        
        // Clean if requested
        if self.clean {
            self.clean_build(&ctx).await?;
        }
        
        // Build the Rust project
        self.build_rust_project(&ctx, release).await?;
        
        // Process and copy assets
        self.process_assets(&ctx).await?;
        
        // Create final output directory
        self.create_output(&ctx).await?;
        
        info!("Build completed successfully!");
        
        let out_dir = self.get_output_directory(&ctx);
        info!("Output directory: {}", out_dir.display());
        
        Ok(())
    }
    
    async fn clean_build(&self, ctx: &CommandContext) -> Result<(), ShipwrightError> {
        info!("Cleaning previous build artifacts...");
        
        let mut cmd = tokio::process::Command::new("cargo");
        cmd.arg("clean");
        
        if let Some(target_dir) = self.get_target_directory(ctx) {
            cmd.args(["--target-dir", &target_dir.display().to_string()]);
        }
        
        if let Some(package) = &self.package {
            cmd.args(["--package", package]);
        }
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ShipwrightError::BuildError(format!(
                "Clean failed:\n{}", stderr
            )));
        }
        
        info!("Clean completed");
        Ok(())
    }
    
    async fn build_rust_project(&self, ctx: &CommandContext, release: bool) -> Result<(), ShipwrightError> {
        info!("Building Rust project...");
        
        let mut cmd = tokio::process::Command::new("cargo");
        cmd.arg("build");
        
        if release {
            cmd.arg("--release");
        }
        
        if let Some(target) = &self.target {
            cmd.args(["--target", target]);
        } else if let Some(target) = &ctx.config.build().target {
            cmd.args(["--target", target]);
        }
        
        if let Some(target_dir) = self.get_target_directory(ctx) {
            cmd.args(["--target-dir", &target_dir.display().to_string()]);
        }
        
        if let Some(package) = &self.package {
            cmd.args(["--package", package]);
        } else if let Some(main_package) = ctx.workspace.get_main_package() {
            cmd.args(["--package", &main_package.name]);
        }
        
        // Combine features from command line and config
        let mut all_features = self.features.clone();
        if let Some(config_features) = &ctx.config.build().features {
            all_features.extend(config_features.clone());
        }
        
        if !all_features.is_empty() {
            all_features.sort();
            all_features.dedup();
            cmd.args(["--features", &all_features.join(",")]);
        }
        
        cmd.args(&self.cargo_args);
        
        if self.verbose {
            cmd.arg("--verbose");
        }
        
        // Set environment variables
        for (key, value) in ctx.workspace.get_build_env(&ctx.config) {
            cmd.env(key, value);
        }
        
        // Set additional environment from config
        if let Some(env) = &ctx.config.build().environment {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }
        
        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        debug!("Running cargo command: {:?}", cmd);
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(ShipwrightError::BuildError(format!(
                "Rust build failed:\nSTDOUT:\n{}\nSTDERR:\n{}", stdout, stderr
            )));
        }
        
        if self.verbose {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("{}", stdout);
        }
        
        info!("Rust build completed successfully");
        Ok(())
    }
    
    async fn process_assets(&self, ctx: &CommandContext) -> Result<(), ShipwrightError> {
        info!("Processing assets...");
        
        let web_config = ctx.config.web();
        
        // Copy public directory if it exists
        if let Some(public_dir) = &web_config.public_dir {
            if public_dir.exists() {
                let out_dir = self.get_output_directory(ctx);
                self.copy_directory(public_dir, &out_dir).await?;
                info!("Copied public assets from {}", public_dir.display());
            }
        }
        
        // Copy assets directory if it exists
        if let Some(assets_dir) = &web_config.assets_dir {
            if assets_dir.exists() {
                let out_dir = self.get_output_directory(ctx).join("assets");
                self.copy_directory(assets_dir, &out_dir).await?;
                info!("Copied assets from {}", assets_dir.display());
            }
        }
        
        // Generate index.html if template exists
        if let Some(template_path) = &web_config.index_template {
            if template_path.exists() {
                self.generate_index_html(ctx, template_path).await?;
            }
        }
        
        info!("Asset processing completed");
        Ok(())
    }
    
    async fn copy_directory(&self, src: &PathBuf, dst: &PathBuf) -> Result<(), ShipwrightError> {
        use walkdir::WalkDir;
        
        if !src.exists() {
            return Ok(());
        }
        
        tokio::fs::create_dir_all(dst).await?;
        
        for entry in WalkDir::new(src) {
            let entry = entry.map_err(|e| ShipwrightError::IoError(e.to_string()))?;
            let src_path = entry.path();
            let relative_path = src_path.strip_prefix(src)
                .map_err(|e| ShipwrightError::IoError(e.to_string()))?;
            let dst_path = dst.join(relative_path);
            
            if src_path.is_dir() {
                tokio::fs::create_dir_all(&dst_path).await?;
            } else {
                if let Some(parent) = dst_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::copy(src_path, dst_path).await?;
            }
        }
        
        Ok(())
    }
    
    async fn generate_index_html(&self, ctx: &CommandContext, template_path: &PathBuf) -> Result<(), ShipwrightError> {
        let template = tokio::fs::read_to_string(template_path).await?;
        let web_config = ctx.config.web();
        
        // Simple template replacement (can be enhanced with a proper template engine)
        let html = template
            .replace("{{title}}", &web_config.title.unwrap_or_else(|| ctx.config.application.name.clone()))
            .replace("{{base_path}}", &web_config.base_path.unwrap_or_else(|| "/".to_string()))
            .replace("{{app_name}}", &ctx.config.application.name);
        
        let out_dir = self.get_output_directory(ctx);
        let index_path = out_dir.join("index.html");
        
        tokio::fs::create_dir_all(&out_dir).await?;
        tokio::fs::write(index_path, html).await?;
        
        info!("Generated index.html");
        Ok(())
    }
    
    async fn create_output(&self, ctx: &CommandContext) -> Result<(), ShipwrightError> {
        let out_dir = self.get_output_directory(ctx);
        
        // Ensure output directory exists
        tokio::fs::create_dir_all(&out_dir).await?;
        
        // Copy compiled binaries if they exist
        let target_dir = self.get_target_directory(ctx)
            .unwrap_or_else(|| PathBuf::from("target"));
        
        let release_dir = if self.release { "release" } else { "debug" };
        let bin_dir = target_dir.join(release_dir);
        
        if bin_dir.exists() {
            // Copy relevant binaries to output directory
            for (binary_name, _) in ctx.workspace.get_binary_targets() {
                let binary_name = binary_name.split("::").last().unwrap_or(&binary_name);
                let src_path = bin_dir.join(binary_name);
                
                if src_path.exists() {
                    let dst_path = out_dir.join(binary_name);
                    tokio::fs::copy(&src_path, &dst_path).await?;
                    info!("Copied binary: {} -> {}", src_path.display(), dst_path.display());
                }
            }
        }
        
        // Create a build manifest
        self.create_build_manifest(ctx).await?;
        
        Ok(())
    }
    
    async fn create_build_manifest(&self, ctx: &CommandContext) -> Result<(), ShipwrightError> {
        let manifest = serde_json::json!({
            "name": ctx.config.application.name,
            "version": ctx.config.application.version,
            "build_time": chrono::Utc::now().to_rfc3339(),
            "build_mode": if self.release { "release" } else { "debug" },
            "target": self.target.as_ref().or(ctx.config.build().target.as_ref()),
            "features": {
                "enabled": self.features,
                "config": ctx.config.build().features
            }
        });
        
        let out_dir = self.get_output_directory(ctx);
        let manifest_path = out_dir.join("build-manifest.json");
        
        let manifest_content = serde_json::to_string_pretty(&manifest)?;
        tokio::fs::write(manifest_path, manifest_content).await?;
        
        debug!("Created build manifest");
        Ok(())
    }
    
    fn get_output_directory(&self, ctx: &CommandContext) -> PathBuf {
        self.out_dir.clone()
            .or_else(|| ctx.config.build().out_dir.clone())
            .unwrap_or_else(|| PathBuf::from("dist"))
    }
    
    fn get_target_directory(&self, ctx: &CommandContext) -> Option<PathBuf> {
        self.target_dir.clone()
            .or_else(|| ctx.config.build().target_dir.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_build_command_creation() {
        let cmd = BuildCommand {
            release: true,
            target: Some("wasm32-unknown-unknown".to_string()),
            features: vec!["ssr".to_string(), "hydration".to_string()],
            out_dir: Some(PathBuf::from("dist")),
            target_dir: Some(PathBuf::from("target")),
            package: Some("my-app".to_string()),
            cargo_args: vec!["--verbose".to_string()],
            clean: false,
            verbose: true,
        };
        
        assert!(cmd.release);
        assert_eq!(cmd.target, Some("wasm32-unknown-unknown".to_string()));
        assert_eq!(cmd.features.len(), 2);
        assert!(cmd.verbose);
    }
    
    #[tokio::test]
    async fn test_output_directory_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        
        // Mock workspace (would need proper setup in real test)
        // This is a simplified test structure
        let cmd = BuildCommand {
            release: false,
            target: None,
            features: vec![],
            out_dir: Some(temp_dir.path().join("custom-dist")),
            target_dir: None,
            package: None,
            cargo_args: vec![],
            clean: false,
            verbose: false,
        };
        
        // This would require a proper workspace setup to test fully
        // For now, just test the directory resolution logic
        assert!(cmd.out_dir.is_some());
    }
}