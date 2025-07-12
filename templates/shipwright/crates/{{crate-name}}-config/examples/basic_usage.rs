//! Basic usage example for {{crate_name}}-config
//!
//! This example demonstrates how to load and use configuration
//! in your application.

use {{crate_name}}_config::{Config, Environment, ConfigError, get_config};

fn main() -> Result<(), ConfigError> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("=== {{project-name}} Configuration Example ===\n");

    // Example 1: Load configuration for current environment
    println!("1. Loading configuration for current environment...");
    let config = Config::load_current()?;
    print_config_summary(&config);

    // Example 2: Load configuration for specific environment
    println!("\n2. Loading development configuration...");
    let dev_config = Config::load(Environment::Development)?;
    print_config_summary(&dev_config);

    // Example 3: Using global configuration
    println!("\n3. Using global configuration instance...");
    let global_config = get_config();
    println!("Global config app name: {}", global_config.app.name);

    // Example 4: Feature flags
    println!("\n4. Feature flags:");
    if config.is_feature_enabled("debug_toolbar") {
        println!("✓ Debug toolbar is enabled");
    } else {
        println!("✗ Debug toolbar is disabled");
    }

    // Example 5: Custom settings
    println!("\n5. Custom settings:");
    if let Some(api_url) = config.get_custom_setting("api_base_url") {
        println!("API Base URL: {}", api_url);
    }

    // Example 6: Environment detection
    println!("\n6. Environment detection:");
    println!("Current environment: {}", config.environment);
    println!("Is production: {}", config.environment.is_production());
    println!("Is development: {}", config.environment.is_development());

    // Example 7: Database and server info
    println!("\n7. Connection info:");
    println!("Database URL: {}", config.database_url());
    println!("Server will bind to: {}", config.server_address());

    Ok(())
}

fn print_config_summary(config: &Config) {
    println!("  Environment: {}", config.environment);
    println!("  Database: {} (max {} connections)", 
             config.database.url, config.database.max_connections);
    println!("  Server: {}:{}", config.server.host, config.server.port);
    println!("  Log level: {}", config.logging.level);
    println!("  Debug mode: {}", config.app.debug);
}