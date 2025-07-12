use clap::Args;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;
use crate::error::ShipwrightError;

/// Create a new Shipwright project from a template
/// 
/// This command generates a new Rust web application using Shipwright templates.
/// Templates are fetched from GitHub by default, making this CLI fully standalone.
/// 
/// Available built-in templates:
///   - default: Complete project with LiveView, database, and modern Rust stack
///   - shipwright: Full framework template with all components
/// 
/// Examples:
///   shipwright new my-app                           # Use default template
///   shipwright new my-app --template shipwright     # Use full template
///   shipwright new my-app --template username/repo  # Use custom GitHub template
#[derive(Args)]
pub struct NewCommand {
    /// Project name (will be used as directory name and in generated files)
    name: String,
    
    /// Template to use (default, shipwright, git URL, or GitHub repo)
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
        
        // Determine template source - prioritize remote templates for standalone CLI usage
        if self.template.starts_with("http") || self.template.starts_with("git@") {
            // Direct git URL - use as-is for cargo generate
            return self.run_cargo_generate_with_git(&self.template).await;
        } else if self.template.contains('/') && !std::path::Path::new(&self.template).exists() {
            // GitHub shorthand (e.g., "username/repo") - use as git URL
            let git_url = format!("https://github.com/{}", self.template);
            return self.run_cargo_generate_with_git(&git_url).await;
        } else {
            // Check for predefined templates - use remote GitHub templates by default
            match self.template.as_str() {
                "default" => {
                    // Try remote template first (standalone usage)
                    let git_url = "https://github.com/NoHeadDotDev/shipwright";
                    match self.run_cargo_generate_with_git_subfolder(git_url, "templates/shipwright-default-template", "templates").await {
                        Ok(_) => return Ok(()),
                        Err(_) => {
                            // Fallback to local template for development
                            if let Ok(templates_path) = self.try_find_local_template("shipwright-default-template") {
                                let mut cmd = Command::new("cargo");
                                cmd.arg("generate");
                                cmd.arg("--path").arg(&templates_path);
                                cmd.arg("--name").arg(&self.name);
                                return self.run_cargo_generate_command(cmd).await;
                            } else {
                                return Err(ShipwrightError::IoError(
                                    "Default template not found locally or remotely. Please check your internet connection or run from the shipwright project directory.".to_string()
                                ));
                            }
                        }
                    }
                },
                "shipwright" => {
                    // Try remote template first (standalone usage)
                    let git_url = "https://github.com/NoHeadDotDev/shipwright";
                    match self.run_cargo_generate_with_git_subfolder(git_url, "templates/shipwright", "templates").await {
                        Ok(_) => return Ok(()),
                        Err(_) => {
                            // Fallback to local template for development
                            if let Ok(templates_path) = self.try_find_local_template("shipwright") {
                                let mut cmd = Command::new("cargo");
                                cmd.arg("generate");
                                cmd.arg("--path").arg(&templates_path);
                                cmd.arg("--name").arg(&self.name);
                                return self.run_cargo_generate_command(cmd).await;
                            } else {
                                return Err(ShipwrightError::IoError(
                                    "Shipwright template not found locally or remotely. Please check your internet connection or run from the shipwright project directory.".to_string()
                                ));
                            }
                        }
                    }
                },
                _ => {
                    // Try local path first
                    let path = std::path::Path::new(&self.template);
                    if path.exists() {
                        // Use local template
                        let mut cmd = Command::new("cargo");
                        cmd.arg("generate");
                        cmd.arg("--path").arg(&path);
                        cmd.arg("--name").arg(&self.name);
                        return self.run_cargo_generate_command(cmd).await;
                    } else {
                        // Try to find local Shipwright development templates as fallback
                        if let Ok(templates_path) = self.try_find_local_template(&self.template) {
                            let mut cmd = Command::new("cargo");
                            cmd.arg("generate");
                            cmd.arg("--path").arg(&templates_path);
                            cmd.arg("--name").arg(&self.name);
                            return self.run_cargo_generate_command(cmd).await;
                        } else {
                            return Err(ShipwrightError::IoError(
                                format!("Template not found: {}. Available templates: default, shipwright, or specify a git URL", self.template)
                            ));
                        }
                    }
                }
            }
        }
        
        // This code should not be reached due to early returns above
        unreachable!()
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
        
        // Use silent mode to avoid terminal issues in CLI environments
        cmd.arg("--silent");
        cmd.arg("--allow-commands");
        
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
    
    async fn run_cargo_generate_with_git_subfolder(&self, git_url: &str, subfolder: &str, branch: &str) -> Result<(), ShipwrightError> {
        info!("Creating new Shipwright project: {}", self.name);
        
        // Build cargo-generate command for git templates with subfolder and branch
        let mut cmd = Command::new("cargo");
        cmd.arg("generate");
        cmd.arg("--git").arg(git_url);
        cmd.arg("--branch").arg(branch);
        cmd.arg("--subfolder").arg(subfolder);
        cmd.arg("--name").arg(&self.name);
        
        self.run_cargo_generate_command(cmd).await
    }
    
    async fn run_cargo_generate_command(&self, mut cmd: Command) -> Result<(), ShipwrightError> {
        // Set destination directory if specified
        if let Some(ref dest) = self.directory {
            cmd.arg("--destination").arg(dest);
        }
        
        // Add force flag if specified
        if self.force {
            cmd.arg("--force");
        }
        
        // Use silent mode to avoid terminal issues in CLI environments
        cmd.arg("--silent");
        cmd.arg("--allow-commands");
        
        // Add custom variables
        for (key, value) in &self.variables {
            cmd.arg("--define").arg(format!("{}={}", key, value));
        }
        
        // Add VCS option if no-git specified
        if self.no_git {
            cmd.arg("--vcs").arg("none");
        }
        
        // Execute cargo-generate
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
    
    fn try_find_local_template(&self, template_name: &str) -> Result<PathBuf, ShipwrightError> {
        // Only try to find local templates if we're in development mode
        if let Ok(shipwright_root) = self.find_shipwright_root() {
            let templates_path = shipwright_root.join("templates").join(template_name);
            if templates_path.exists() {
                return Ok(templates_path);
            }
        }
        Err(ShipwrightError::IoError("Local template not found".to_string()))
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