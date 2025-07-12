# Cargo Generate Integration Guide for Shipwright CLI

## Overview

This guide explains how to integrate cargo-generate templates into the Shipwright CLI tool to enable users to scaffold new projects with pre-configured hot-reload support, LiveView integration, and workspace structures.

## How cargo-generate Works

cargo-generate is a developer tool that creates new Rust projects from pre-existing git repository templates. It uses:
- **Liquid templating language** for file content substitution
- **Placeholders** that get replaced with user-provided or default values
- **Configuration files** (`cargo-generate.toml`) to define template behavior
- **Hooks** (Rhai scripts) for pre/post-generation logic

## Template Structure

A typical cargo-generate template has this structure:

```
my-template/
├── cargo-generate.toml          # Template configuration
├── Cargo.toml.liquid           # Root workspace Cargo.toml
├── Shipwright.toml.liquid      # Shipwright configuration
├── README.md.liquid            # Project documentation
├── .gitignore
├── {{project-name}}-server/    # Main server crate
│   ├── Cargo.toml.liquid
│   └── src/
│       └── main.rs.liquid
├── {{project-name}}-frontend/  # Frontend crate
│   ├── Cargo.toml.liquid
│   └── src/
│       └── lib.rs.liquid
└── hooks/
    ├── pre-generate.rhai       # Pre-generation script
    └── post-generate.rhai      # Post-generation script
```

## Placeholders

### Built-in Placeholders
- `{{project-name}}` - The project name (automatically converted to appropriate case)
- `{{crate_name}}` - Rust-safe crate name (snake_case)
- `{{authors}}` - Author information from git config
- `{{os-arch}}` - Target OS and architecture

### Custom Placeholders
Define in `cargo-generate.toml`:

```toml
[placeholders.framework]
type = "string"
prompt = "Which web framework?"
choices = ["axum", "actix-web", "rocket", "warp"]
default = "axum"

[placeholders.use_liveview]
type = "bool"
prompt = "Include LiveView support?"
default = true

[placeholders.database]
type = "string"
prompt = "Database setup?"
choices = ["none", "postgres", "sqlite", "mysql"]
default = "none"
```

## Creating a Shipwright Template

### 1. Basic cargo-generate.toml

```toml
[template]
cargo_generate_version = ">=0.19.0"
ignore = [".git", "target", "Cargo.lock"]

[placeholders.project_type]
type = "string"
prompt = "What type of Shipwright project?"
choices = ["full-stack", "api-only", "liveview-app"]
default = "full-stack"

[placeholders.port]
type = "string"
prompt = "Default port for development server?"
default = "3000"
regex = "^[0-9]{4,5}$"

[placeholders.hot_reload]
type = "bool"
prompt = "Enable hot reload?"
default = true

[placeholders.tailwind]
type = "bool"
prompt = "Include Tailwind CSS?"
default = true

[conditional.'project_type == "full-stack" || project_type == "liveview-app"'.placeholders.frontend_framework]
type = "string"
prompt = "Frontend build tool?"
choices = ["vite", "webpack", "esbuild", "none"]
default = "vite"

[hooks]
pre = ["scripts/pre-generate.rhai"]
post = ["scripts/post-generate.rhai"]
```

### 2. Template Files

#### Cargo.toml.liquid (workspace root)
```toml
[workspace]
members = [
    "{{project-name}}-server",
{%- if project_type == "full-stack" or project_type == "liveview-app" %}
    "{{project-name}}-frontend",
{%- endif %}
{%- if use_liveview %}
    "{{project-name}}-liveview",
{%- endif %}
]
resolver = "2"

[workspace.package]
version = "0.1.0"
authors = ["{{authors}}"]
edition = "2021"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.7"
serde = { version = "1", features = ["derive"] }
{%- if use_liveview %}
shipwright-liveview = { path = "../shipwright/shipwright-liveview" }
{%- endif %}
```

#### Shipwright.toml.liquid
```toml
# Shipwright configuration for {{project-name}}

[server]
port = {{port}}
host = "127.0.0.1"

[build]
watch_paths = ["src", "templates", "assets"]
exclude_paths = ["target", "node_modules"]

{%- if hot_reload %}
[hot_reload]
enabled = true
ws_port = {{ port | plus: 1000 }}
client_script = "/hot-reload-client.js"
{%- endif %}

{%- if project_type == "full-stack" or project_type == "liveview-app" %}
[frontend]
{%- if frontend_framework == "vite" %}
command = "npm run dev"
build_command = "npm run build"
dist_dir = "dist"
{%- elif frontend_framework == "webpack" %}
command = "npm run serve"
build_command = "npm run build"
dist_dir = "dist"
{%- endif %}
{%- endif %}

{%- if tailwind %}
[css]
framework = "tailwind"
input = "styles/app.css"
output = "static/app.css"
{%- endif %}
```

#### {{project-name}}-server/src/main.rs.liquid
```rust
use axum::{Router, routing::get};
{%- if hot_reload %}
use shipwright_liveview_hotreload::HotReloadLayer;
{%- endif %}
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "{{crate_name}}=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build application
    let app = Router::new()
        .route("/", get(hello_world))
        {%- if hot_reload %}
        .layer(HotReloadLayer::new())
        {%- endif %};

    // Run server
    let addr = SocketAddr::from(([127, 0, 0, 1], {{port}}));
    info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> &'static str {
    "Hello, {{project-name}}!"
}
```

### 3. Hook Scripts

#### scripts/post-generate.rhai
```rhai
let project_name = variable::get("project-name");
let use_liveview = variable::get("use_liveview");
let frontend_framework = variable::get("frontend_framework");

// Initialize git repository
system::command("git", ["init"]);
system::command("git", ["add", "."]);

// Install frontend dependencies if needed
if frontend_framework != "none" {
    file::write("package.json", r#"{
  "name": "{{project-name}}-frontend",
  "version": "0.1.0",
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  },
  "devDependencies": {
    "vite": "^5.0.0"
  }
}"#);
    
    system::command("npm", ["install"]);
}

// Create initial directories
file::create_dir("static");
file::create_dir("templates");

print("✅ Project " + project_name + " created successfully!");
print("📝 Next steps:");
print("  1. cd " + project_name);
print("  2. shipwright dev");
```

## Integrating with Shipwright CLI

Add a new command to create projects:

### 1. Add the new command to main.rs

```rust
#[derive(Subcommand)]
enum Commands {
    /// Start development server with hot reload
    Dev(DevCommand),
    /// Start production server
    Serve(ServeCommand),
    /// Build application for production
    Build(BuildCommand),
    /// Create a new Shipwright project
    New(NewCommand),
}
```

### 2. Create commands/new.rs

```rust
use clap::Args;
use std::path::PathBuf;
use std::process::Command;
use crate::error::ShipwrightError;
use crate::config::Config;

#[derive(Args)]
pub struct NewCommand {
    /// Project name
    name: String,
    
    /// Template to use (can be a git URL or a registered template name)
    #[arg(long, short, default_value = "shipwright/template-default")]
    template: String,
    
    /// Directory to create the project in
    #[arg(long, short)]
    dir: Option<PathBuf>,
    
    /// Skip interactive prompts and use defaults
    #[arg(long)]
    defaults: bool,
}

impl NewCommand {
    pub async fn run(self) -> Result<(), ShipwrightError> {
        let mut cmd = Command::new("cargo");
        cmd.arg("generate");
        
        // Add template
        cmd.arg("--git");
        cmd.arg(&self.template);
        
        // Add name
        cmd.arg("--name");
        cmd.arg(&self.name);
        
        // Add directory if specified
        if let Some(dir) = &self.dir {
            cmd.arg("--destination");
            cmd.arg(dir);
        }
        
        // Use defaults if requested
        if self.defaults {
            cmd.arg("--silent");
        }
        
        // Execute cargo-generate
        let output = cmd.output()
            .map_err(|e| ShipwrightError::IoError(
                format!("Failed to run cargo-generate: {}. Is it installed?", e)
            ))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ShipwrightError::CommandError(
                format!("cargo-generate failed: {}", stderr)
            ));
        }
        
        println!("✅ Created new Shipwright project: {}", self.name);
        println!("📝 Next steps:");
        println!("   cd {}", self.name);
        println!("   shipwright dev");
        
        Ok(())
    }
}
```

## Template Best Practices

1. **Naming Conventions**
   - Use lowercase for placeholder names
   - Use descriptive names for placeholders
   - Follow Rust naming conventions in generated code

2. **File Organization**
   - Keep `.liquid` extension for templated files
   - Use `{{project-name}}` in directory names for multi-crate workspaces
   - Include comprehensive `.gitignore`

3. **Placeholder Design**
   - Provide sensible defaults
   - Use choices for limited options
   - Add regex validation for user input
   - Group related placeholders conditionally

4. **Documentation**
   - Include README.md.liquid with project-specific instructions
   - Document all placeholders in cargo-generate.toml
   - Add comments in template files

5. **Hooks**
   - Use pre-hooks for validation
   - Use post-hooks for initialization (git, npm install, etc.)
   - Keep hooks simple and fast

## Example Multi-Crate Workspace Template

```
shipwright-workspace-template/
├── cargo-generate.toml
├── Cargo.toml.liquid
├── Shipwright.toml.liquid
├── README.md.liquid
├── .gitignore
├── {{project-name}}-server/
│   ├── Cargo.toml.liquid
│   └── src/
│       ├── main.rs.liquid
│       ├── handlers/
│       │   └── mod.rs.liquid
│       └── middleware/
│           └── mod.rs.liquid
├── {{project-name}}-frontend/
│   ├── Cargo.toml.liquid
│   ├── package.json.liquid
│   ├── vite.config.js.liquid
│   └── src/
│       ├── main.ts.liquid
│       └── App.vue.liquid
├── {{project-name}}-shared/
│   ├── Cargo.toml.liquid
│   └── src/
│       └── lib.rs.liquid
└── {{project-name}}-liveview/
    ├── Cargo.toml.liquid
    └── src/
        ├── lib.rs.liquid
        └── components/
            └── mod.rs.liquid
```

## Publishing Templates

1. Create a GitHub repository with your template
2. Add the `cargo-generate` topic to your repository
3. Include comprehensive documentation
4. Test the template thoroughly
5. Consider versioning your templates with git tags

## Integration with cargo-generate API

For programmatic template generation:

```rust
use cargo_generate::{generate, GenerateArgs};

fn create_project_programmatically() -> Result<(), Box<dyn std::error::Error>> {
    let args = GenerateArgs {
        template_path: Some("https://github.com/shipwright/templates".into()),
        name: Some("my-new-project".into()),
        force: false,
        verbose: true,
        template_values_file: None,
        silent: false,
        config: None,
        vcs: Some(Vcs::Git),
        branch: None,
        tag: None,
        // ... other options
    };
    
    generate(args)?;
    Ok(())
}
```

This guide provides a comprehensive foundation for integrating cargo-generate templates with the Shipwright CLI, enabling users to quickly scaffold new projects with hot-reload support and LiveView integration.