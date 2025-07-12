# Cargo Generate Examples and Integration

## Real-World cargo-generate Template Examples

### 1. Simple API Template

```toml
# cargo-generate.toml
[template]
cargo_generate_version = ">=0.19.0"

[placeholders.api_framework]
type = "string"
prompt = "Which API framework?"
choices = ["actix-web", "axum", "rocket"]
default = "axum"

[placeholders.auth_type]
type = "string"
prompt = "Authentication type?"
choices = ["none", "jwt", "oauth", "basic"]
default = "jwt"
```

### 2. Microservice Template with Multiple Services

```toml
[placeholders.services]
type = "array"
prompt = "Which services to include?"
choices = ["api", "auth", "worker", "admin"]
default = ["api", "auth"]

[conditional.'services.contains("worker")'.placeholders.queue]
type = "string"
prompt = "Queue system for worker?"
choices = ["redis", "rabbitmq", "kafka"]
default = "redis"
```

### 3. Well-Structured Template Examples from GitHub

#### Rust Web App Template (inspired by shuttle-hq/shuttle)
```
rust-webapp-template/
├── cargo-generate.toml
├── Cargo.toml.liquid
├── src/
│   ├── main.rs.liquid
│   ├── routes/
│   │   ├── mod.rs.liquid
│   │   └── health.rs
│   └── middleware/
│       └── mod.rs.liquid
├── templates/
│   └── index.html.liquid
└── .github/
    └── workflows/
        └── ci.yml.liquid
```

#### CLI Tool Template (inspired by clap-rs patterns)
```
cli-tool-template/
├── cargo-generate.toml
├── Cargo.toml.liquid
├── src/
│   ├── main.rs.liquid
│   ├── cli.rs.liquid
│   └── commands/
│       ├── mod.rs.liquid
│       └── {{command_name}}.rs.liquid
└── tests/
    └── integration_test.rs.liquid
```

## Advanced cargo-generate Features

### 1. Using Rhai Scripts for Complex Logic

```rhai
// scripts/validate.rhai
let project_name = variable::get("project-name");
let reserved_names = ["test", "cargo", "rust", "std"];

if reserved_names.contains(project_name) {
    abort("Project name '" + project_name + "' is reserved");
}

// Ensure project name follows Rust naming conventions
if !project_name.chars().all(|c| c.is_lowercase() || c == '-' || c == '_') {
    abort("Project name must be lowercase with only '-' or '_' separators");
}
```

### 2. Conditional File Inclusion

```toml
[conditional.'auth_type == "jwt"']
include = ["src/auth/jwt.rs.liquid", "src/middleware/auth.rs.liquid"]

[conditional.'database == "none"']
exclude = ["src/db/", "migrations/"]
```

### 3. Complex Workspace Templates

```toml
[placeholders.workspace_structure]
type = "string"
prompt = "Workspace structure?"
choices = ["monorepo", "microservices", "library-with-examples"]
default = "monorepo"

[conditional.'workspace_structure == "microservices"'.placeholders.num_services]
type = "string"
prompt = "Number of initial services?"
default = "3"
regex = "^[1-9][0-9]?$"
```

## Integration Patterns for CLI Tools

### 1. Direct Integration (Embedding cargo-generate)

```rust
use std::process::Command;
use std::env;

pub fn create_from_template(
    template_url: &str,
    project_name: &str,
    variables: &[(String, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("generate")
        .arg("--git").arg(template_url)
        .arg("--name").arg(project_name);
    
    // Add custom variables
    for (key, value) in variables {
        cmd.arg("--define").arg(format!("{}={}", key, value));
    }
    
    let output = cmd.output()?;
    
    if !output.status.success() {
        return Err(format!("Failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    Ok(())
}
```

### 2. Template Discovery System

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateRegistry {
    templates: HashMap<String, TemplateInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub url: String,
    pub tags: Vec<String>,
    pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub default: Option<String>,
    pub choices: Option<Vec<String>>,
}

impl TemplateRegistry {
    pub async fn fetch_from_github(org: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Fetch template registry from GitHub
        let url = format!("https://raw.githubusercontent.com/{}/templates/main/registry.json", org);
        let response = reqwest::get(&url).await?;
        let registry: TemplateRegistry = response.json().await?;
        Ok(registry)
    }
    
    pub fn list_templates(&self) -> Vec<&TemplateInfo> {
        self.templates.values().collect()
    }
    
    pub fn find_by_tags(&self, tags: &[String]) -> Vec<&TemplateInfo> {
        self.templates.values()
            .filter(|t| tags.iter().any(|tag| t.tags.contains(tag)))
            .collect()
    }
}
```

### 3. Interactive Template Selection

```rust
use dialoguer::{theme::ColorfulTheme, Select, Input, Confirm};

pub fn interactive_template_setup() -> Result<TemplateConfig, Box<dyn std::error::Error>> {
    // Select template type
    let template_types = vec!["Web API", "CLI Tool", "Library", "Full Stack App"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What type of project?")
        .items(&template_types)
        .default(0)
        .interact()?;
    
    // Get project name
    let project_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Project name")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                Ok(())
            } else {
                Err("Project name can only contain alphanumeric characters, '-', or '_'")
            }
        })
        .interact_text()?;
    
    // Additional options based on template type
    let mut variables = vec![];
    
    if selection == 0 { // Web API
        let frameworks = vec!["Axum", "Actix-web", "Rocket"];
        let framework = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select framework")
            .items(&frameworks)
            .interact()?;
        
        variables.push(("framework".to_string(), frameworks[framework].to_lowercase()));
        
        let use_db = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Include database support?")
            .interact()?;
        
        if use_db {
            let databases = vec!["PostgreSQL", "SQLite", "MySQL"];
            let db = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select database")
                .items(&databases)
                .interact()?;
            
            variables.push(("database".to_string(), databases[db].to_lowercase()));
        }
    }
    
    Ok(TemplateConfig {
        template_type: template_types[selection].to_string(),
        project_name,
        variables,
    })
}

#[derive(Debug)]
pub struct TemplateConfig {
    pub template_type: String,
    pub project_name: String,
    pub variables: Vec<(String, String)>,
}
```

## Best Practices for Template Authors

### 1. Documentation Template

```markdown
# {{project-name}}

{{description}}

## Quick Start

```bash
# Development
cargo run

# Tests
cargo test

# Production build
cargo build --release
```

## Configuration

{%- if has_config %}
Edit `config.toml` to configure:
- Server settings
- Database connection
{%- if features contains "auth" %}
- Authentication providers
{%- endif %}
{%- endif %}

## Project Structure

```
{{project-name}}/
├── src/
│   ├── main.rs
{%- for module in modules %}
│   ├── {{module}}/
{%- endfor %}
│   └── lib.rs
├── tests/
└── Cargo.toml
```
```

### 2. Testing Your Templates

```bash
# Test template generation
cargo generate --git file:///path/to/template --name test-project

# Test with different variable combinations
cargo generate --git file:///path/to/template \
    --name test-api \
    --define framework=axum \
    --define database=postgres

# Automated testing script
#!/bin/bash
templates=("minimal" "full-stack" "api-only")
for template in "${templates[@]}"; do
    cargo generate --git ./templates/$template \
        --name test-$template \
        --silent \
        --destination /tmp/test-$template
    
    cd /tmp/test-$template
    cargo check || exit 1
    cargo test || exit 1
    cd -
    rm -rf /tmp/test-$template
done
```

### 3. Version Compatibility

```toml
[template]
cargo_generate_version = ">=0.19.0"

[placeholders.rust_version]
type = "string"
prompt = "Minimum Rust version?"
choices = ["1.70", "1.75", "stable", "nightly"]
default = "1.75"

[conditional.'rust_version == "nightly"']
include = ["src/nightly_features.rs.liquid"]
```

## Common Patterns and Solutions

### 1. Multi-Language Projects

```toml
[placeholders.include_frontend]
type = "bool"
prompt = "Include frontend?"
default = true

[conditional.'include_frontend'.placeholders.frontend_lang]
type = "string"
prompt = "Frontend language?"
choices = ["typescript", "javascript", "rust-wasm"]
default = "typescript"

[hooks]
post = ["scripts/setup-frontend.rhai"]
```

### 2. CI/CD Pipeline Generation

```liquid
# .github/workflows/ci.yml.liquid
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
    
    {%- if database != "none" %}
    - name: Setup database
      run: |
        {%- if database == "postgres" %}
        docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=test postgres
        {%- elif database == "mysql" %}
        docker run -d -p 3306:3306 -e MYSQL_ROOT_PASSWORD=test mysql
        {%- endif %}
    {%- endif %}
    
    - run: cargo test --all-features
```

### 3. License Selection

```toml
[placeholders.license]
type = "string"
prompt = "License?"
choices = ["MIT", "Apache-2.0", "MIT OR Apache-2.0", "GPL-3.0", "Proprietary"]
default = "MIT OR Apache-2.0"
```

This comprehensive guide provides practical examples and patterns for creating and using cargo-generate templates effectively with your Shipwright CLI tool.