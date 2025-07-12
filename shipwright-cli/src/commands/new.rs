use clap::Args;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;
use crate::error::ShipwrightError;

/// Create a new Shipwright project from a template
#[derive(Args)]
pub struct NewCommand {
    /// Project name
    name: String,
    
    /// Template to use (can be a git URL, local path, or registered template name)
    #[arg(long, short, default_value = "default")]
    template: String,
    
    /// Directory to create the project in (defaults to project name)
    #[arg(long)]
    directory: Option<PathBuf>,
    
    /// Skip interactive prompts and use defaults
    #[arg(long)]
    defaults: bool,
    
    /// Force overwrite if directory already exists
    #[arg(long)]
    force: bool,
    
    /// Don't initialize a git repository
    #[arg(long)]
    no_git: bool,
    
    /// Additional template variables (can be specified multiple times)
    #[arg(long = "define", short = 'd', value_parser = parse_key_value)]
    variables: Vec<(String, String)>,
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid key=value: '{}'", s));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

impl NewCommand {
    pub async fn run(self) -> Result<(), ShipwrightError> {
        // Check if cargo-generate is installed
        let check_cmd = Command::new("cargo")
            .arg("generate")
            .arg("--version")
            .output();
            
        if check_cmd.is_err() || !check_cmd.unwrap().status.success() {
            return Err(ShipwrightError::BuildError(
                "cargo-generate is not installed. Install it with: cargo install cargo-generate".to_string()
            ));
        }
        
        info!("Creating new Shipwright project: {}", self.name);
        
        // Build cargo-generate command
        let mut cmd = Command::new("cargo");
        cmd.arg("generate");
        
        // Determine template source
        let template_path = if self.template.starts_with("http") || self.template.starts_with("git@") {
            // Direct git URL - use as-is for cargo generate
            return self.run_cargo_generate_with_git(&self.template).await;
        } else if self.template.contains('/') && !std::path::Path::new(&self.template).exists() {
            // GitHub shorthand (e.g., "username/repo") - use as git URL
            let git_url = format!("https://github.com/{}", self.template);
            return self.run_cargo_generate_with_git(&git_url).await;
        } else {
            // Local path or predefined template
            match self.template.as_str() {
                "default" => {
                    // Use the default Shipwright template from local templates directory
                    self.find_shipwright_root()?
                        .join("templates")
                        .join("shipwright-default-template")
                },
                "shipwright" => {
                    // Use the full shipwright template
                    self.find_shipwright_root()?
                        .join("templates")
                        .join("shipwright")
                },
                _ => {
                    // Assume it's a local path
                    let path = std::path::Path::new(&self.template);
                    if path.exists() {
                        path.to_path_buf()
                    } else {
                        // Try checking in templates directory
                        let templates_path = self.find_shipwright_root()?
                            .join("templates")
                            .join(&self.template);
                        
                        if templates_path.exists() {
                            templates_path
                        } else {
                            return Err(ShipwrightError::IoError(
                                format!("Template not found: {}. Available templates: default, shipwright", self.template)
                            ));
                        }
                    }
                }
            }
        };
        
        // For local templates, use the path directly
        cmd.arg("--path").arg(&template_path);
        cmd.arg("--name").arg(&self.name);
        
        // Set destination directory if specified
        if let Some(ref dest) = self.directory {
            cmd.arg("--destination").arg(dest);
        }
        
        // Add force flag if specified
        if self.force {
            cmd.arg("--force");
        }
        
        // Use silent mode if defaults requested
        if self.defaults {
            cmd.arg("--silent");
            cmd.arg("--allow-commands");
        }
        
        // Add custom variables
        for (key, value) in &self.variables {
            cmd.arg("--define").arg(format!("{}={}", key, value));
        }
        
        // Add VCS option if no-git specified
        if self.no_git {
            cmd.arg("--vcs").arg("none");
        }
        
        // Execute cargo-generate
        info!("Running cargo-generate with template: {}", template_path.display());
        let output = cmd.output()
            .map_err(|e| ShipwrightError::IoError(
                format!("Failed to run cargo-generate: {}", e)
            ))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ShipwrightError::BuildError(
                format!("cargo-generate failed: {}", stderr)
            ));
        }
        
        // Print success message
        let default_dir = PathBuf::from(&self.name);
        let project_dir = self.directory.as_ref().unwrap_or(&default_dir);
        println!("\n✅ Successfully created new Shipwright project: {}", self.name);
        println!("\n📝 Next steps:");
        println!("   cd {}", project_dir.display());
        println!("   shipwright dev");
        
        // Additional tips based on template
        if self.template.contains("full-stack") {
            println!("\n💡 Don't forget to:");
            println!("   - Install frontend dependencies: cd {}-frontend && npm install", self.name);
            println!("   - Set up your database if needed");
        }
        
        Ok(())
    }
    
    async fn run_cargo_generate_with_git(&self, git_url: &str) -> Result<(), ShipwrightError> {
        info!("Creating new Shipwright project: {}", self.name);
        
        // Build cargo-generate command for git templates
        let mut cmd = Command::new("cargo");
        cmd.arg("generate");
        cmd.arg("--git").arg(git_url);
        cmd.arg("--name").arg(&self.name);
        
        // Set destination directory if specified
        if let Some(ref dest) = self.directory {
            cmd.arg("--destination").arg(dest);
        }
        
        // Add force flag if specified
        if self.force {
            cmd.arg("--force");
        }
        
        // Use silent mode if defaults requested
        if self.defaults {
            cmd.arg("--silent");
            cmd.arg("--allow-commands");
        }
        
        // Add custom variables
        for (key, value) in &self.variables {
            cmd.arg("--define").arg(format!("{}={}", key, value));
        }
        
        // Add VCS option if no-git specified
        if self.no_git {
            cmd.arg("--vcs").arg("none");
        }
        
        // Execute cargo-generate
        info!("Running cargo-generate with git template: {}", git_url);
        let output = cmd.output()
            .map_err(|e| ShipwrightError::IoError(
                format!("Failed to run cargo-generate: {}", e)
            ))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ShipwrightError::BuildError(
                format!("cargo-generate failed: {}", stderr)
            ));
        }
        
        // Print success message
        let default_dir = PathBuf::from(&self.name);
        let project_dir = self.directory.as_ref().unwrap_or(&default_dir);
        println!("\n✅ Successfully created new Shipwright project: {}", self.name);
        println!("\n📝 Next steps:");
        println!("   cd {}", project_dir.display());
        println!("   shipwright dev");
        
        Ok(())
    }
    
    /// Find the Shipwright project root by looking for templates directory
    fn find_shipwright_root(&self) -> Result<PathBuf, ShipwrightError> {
        let mut current_dir = std::env::current_dir()
            .map_err(|e| ShipwrightError::IoError(format!("Failed to get current directory: {}", e)))?;
        
        // Look for templates directory in current dir and parent dirs
        loop {
            let templates_path = current_dir.join("templates");
            if templates_path.exists() && templates_path.is_dir() {
                return Ok(current_dir);
            }
            
            // Check if there's a Cargo.toml with shipwright-cli
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                    if content.contains("shipwright-cli") {
                        return Ok(current_dir);
                    }
                }
            }
            
            // Move to parent directory
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                break;
            }
        }
        
        // Fallback: assume we're in the shipwright repo itself
        let fallback_path = std::env::current_dir()
            .map_err(|e| ShipwrightError::IoError(format!("Failed to get current directory: {}", e)))?;
        
        // Check if we're already in the shipwright root
        if fallback_path.join("templates").exists() {
            Ok(fallback_path)
        } else {
            // Try going up one level (in case we're in shipwright-cli/)
            if let Some(parent) = fallback_path.parent() {
                if parent.join("templates").exists() {
                    return Ok(parent.to_path_buf());
                }
            }
            
            Err(ShipwrightError::IoError(
                "Could not find Shipwright project root with templates directory".to_string()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_key_value() {
        assert_eq!(parse_key_value("key=value").unwrap(), ("key".to_string(), "value".to_string()));
        assert_eq!(parse_key_value("port=3000").unwrap(), ("port".to_string(), "3000".to_string()));
        assert!(parse_key_value("invalid").is_err());
    }
}