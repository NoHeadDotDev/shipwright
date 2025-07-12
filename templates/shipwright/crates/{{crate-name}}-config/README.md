# {{project-name}} Configuration

This crate provides robust configuration management for {{project-name}} with support for multiple environments, environment variable overrides, and comprehensive validation.

## Features

- **Environment-specific configurations** (development, production, test)
- **Environment variable overrides** for sensitive data
- **Configuration validation** with detailed error messages
- **Hot-reloading** support in development (with `watch` feature)
- **Runtime environment detection** (Docker, Kubernetes, CI/CD)
- **Configuration backup and restore** utilities
- **Structured logging** support
- **Feature flags** and custom settings

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
{{crate_name}}-config = { path = "../{{crate_name}}-config" }
```

### Basic Usage

```rust
use {{crate_name}}_config::{Config, Environment, get_config};

// Load configuration for current environment
let config = Config::load_current()?;

// Or load for specific environment
let config = Config::load(Environment::Production)?;

// Access configuration values
println!("Database URL: {}", config.database.url);
println!("Server address: {}", config.server_address());

// Use global configuration instance
let global_config = get_config();
println!("App name: {}", global_config.app.name);
```

## Configuration Files

The system looks for configuration files in the following locations (in order):

1. `./config/{environment}.toml`
2. `../config/{environment}.toml`
3. `../../config/{environment}.toml`
4. `~/.config/{{crate_name}}/{environment}.toml`

### Environment Files

- `development.toml` - Development settings (default)
- `production.toml` - Production settings
- `test.toml` - Test settings

## Environment Variables

Set the environment using:

```bash
export {{upper_case_name}}_ENV=production
```

### Configuration Overrides

You can override any configuration value using environment variables:

```bash
# Database configuration
export {{upper_case_name}}_DATABASE_URL="postgresql://localhost/myapp"

# Server configuration
export {{upper_case_name}}_SERVER_PORT=3000
export {{upper_case_name}}_SERVER_HOST="0.0.0.0"

# Logging configuration
export {{upper_case_name}}_LOG_LEVEL=debug

# Security configuration
export {{upper_case_name}}_JWT_SECRET="your-secure-secret"
```

## Configuration Sections

### Database Configuration

```toml
[database]
url = "sqlite:app.db"
max_connections = 10
min_connections = 1
connect_timeout = 30
max_lifetime = 3600
log_queries = false
auto_migrate = true
```

### Server Configuration

```toml
[server]
host = "127.0.0.1"
port = 8080
cors_enabled = true
cors_origins = ["*"]
request_timeout = 30
max_body_size = 10485760  # 10MB
compression = true
static_dir = "static"
dev_mode = false
```

### Logging Configuration

```toml
[logging]
level = "info"
format = "json"
file_enabled = true
file_path = "logs/app.log"
max_file_size = 104857600  # 100MB
max_files = 10
console_enabled = true
structured = true
```

### Security Configuration

```toml
[security]
jwt_secret = "your-secret-key"
jwt_expiration = 3600
https_redirect = false
secure_cookies = false
session_timeout = 7200
rate_limiting = false
rate_limit_rpm = 60
```

### Application Configuration

```toml
[app]
name = "{{project-name}}"
version = "0.1.0"
debug = false

[app.features]
experimental_features = false
debug_toolbar = false

[app.custom]
api_base_url = "https://api.example.com"
upload_dir = "/var/uploads"
```

## Features

### Feature Flags

Enable feature flags in your configuration:

```toml
[app.features]
new_ui = true
beta_features = false
```

Check features in code:

```rust
if config.is_feature_enabled("new_ui") {
    // Use new UI
}
```

### Custom Settings

Add custom application-specific settings:

```toml
[app.custom]
api_timeout = "30"
cache_ttl = "3600"
```

Access in code:

```rust
let timeout = config.get_custom_setting("api_timeout");
```

### Configuration Validation

The system automatically validates:

- Database URL format and connectivity
- Server address and port validity
- Log level values
- Required security settings in production
- File and directory paths

### Runtime Environment Detection

```rust
use {{crate_name}}_config::utils::env::{is_docker, is_kubernetes, get_runtime_info};

if is_docker() {
    println!("Running in Docker");
}

let runtime = get_runtime_info();
println!("Platform: {} on {}", runtime.platform, runtime.hostname);
```

### Configuration Watching (Development)

Enable the `watch` feature for hot-reloading:

```toml
[dependencies]
{{crate_name}}-config = { path = "../{{crate_name}}-config", features = ["watch"] }
```

```rust
use {{crate_name}}_config::utils::watch::watch_config_files;

watch_config_files(|| {
    println!("Configuration changed, reloading...");
    // Reload your application configuration
})?;
```

## Production Deployment

### Required Environment Variables

For production deployment, ensure these environment variables are set:

```bash
{{upper_case_name}}_ENV=production
{{upper_case_name}}_DATABASE_URL="postgresql://user:pass@host/db"
{{upper_case_name}}_JWT_SECRET="your-very-secure-secret-key"
```

### Security Checklist

- [ ] Set secure JWT secret via environment variable
- [ ] Configure proper CORS origins
- [ ] Enable HTTPS redirect
- [ ] Set secure cookie flags
- [ ] Configure rate limiting
- [ ] Review log levels and disable debug logging
- [ ] Set up proper file permissions for config files

## Testing

The test configuration uses in-memory SQLite and minimal logging for fast test execution:

```bash
{{upper_case_name}}_ENV=test cargo test
```

## Backup and Migration

```rust
use {{crate_name}}_config::utils::backup::{backup_config, restore_config};

// Create backup
let backup_file = backup_config(&config)?;

// Restore from backup
let restored_config = restore_config(&backup_file)?;
```

## Error Handling

The configuration system provides detailed error messages:

```rust
use {{crate_name}}_config::{Config, ConfigError};

match Config::load_current() {
    Ok(config) => println!("Config loaded successfully"),
    Err(ConfigError::FileNotFound { path }) => {
        eprintln!("Config file not found: {}", path);
    }
    Err(ConfigError::ValidationError { message }) => {
        eprintln!("Invalid configuration: {}", message);
    }
    Err(e) => eprintln!("Configuration error: {}", e),
}
```