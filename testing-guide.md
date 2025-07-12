# Comprehensive Testing Guide: Fantoccini, Faker, and Rust Testing Patterns

## Table of Contents
1. [Project Structure](#project-structure)
2. [Unit Testing with Faker](#unit-testing-with-faker)
3. [Integration Testing Structure](#integration-testing-structure)
4. [Browser Testing with Fantoccini](#browser-testing-with-fantoccini)
5. [Multi-Crate Workspace Testing](#multi-crate-workspace-testing)
6. [Best Practices](#best-practices)

## Project Structure

Here's a recommended structure for a Rust web application with comprehensive testing:

```
my-web-app/
├── Cargo.toml                # Workspace root
├── crates/
│   ├── api/                  # API server crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── handlers/
│   │   └── tests/            # Integration tests
│   │       └── api_tests.rs
│   ├── core/                 # Core business logic
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── tests/
│   ├── web/                  # Frontend server
│   │   ├── Cargo.toml
│   │   └── src/
│   └── test-utils/           # Shared test utilities
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
└── tests/                    # E2E browser tests
    └── browser_tests.rs

```

## Unit Testing with Faker

### 1. Setting up Faker with Dummy Derive

First, add dependencies to your `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
validator = { version = "0.18", features = ["derive"] }

[dev-dependencies]
fake = { version = "2.9", features = ["derive"] }
```

### 2. Using Dummy Derive with cfg_attr

```rust
// src/models/user.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct User {
    #[cfg_attr(test, dummy(faker = "1000..9999"))]
    pub id: u32,
    
    #[cfg_attr(test, dummy(faker = "fake::faker::name::en::FirstName()"))]
    #[validate(length(min = 1, max = 50))]
    pub first_name: String,
    
    #[cfg_attr(test, dummy(faker = "fake::faker::name::en::LastName()"))]
    #[validate(length(min = 1, max = 50))]
    pub last_name: String,
    
    #[cfg_attr(test, dummy(faker = "fake::faker::internet::en::SafeEmail()"))]
    #[validate(email)]
    pub email: String,
    
    #[cfg_attr(test, dummy(faker = "18..100"))]
    pub age: u8,
    
    #[cfg_attr(test, dummy(faker = "fake::faker::boolean::en::Boolean(75)"))]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct CreateUserRequest {
    #[cfg_attr(test, dummy(faker = "fake::faker::name::en::FirstName()"))]
    pub first_name: String,
    
    #[cfg_attr(test, dummy(faker = "fake::faker::name::en::LastName()"))]
    pub last_name: String,
    
    #[cfg_attr(test, dummy(faker = "fake::faker::internet::en::SafeEmail()"))]
    pub email: String,
    
    #[cfg_attr(test, dummy(faker = "fake::faker::internet::en::Password(10..20)"))]
    pub password: String,
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use fake::{Fake, Faker};

    #[test]
    fn test_user_creation() {
        let user: User = Faker.fake();
        assert!(user.validate().is_ok());
        assert!(user.age >= 18 && user.age <= 100);
        assert!(user.id >= 1000 && user.id <= 9999);
    }

    #[test]
    fn test_create_user_request() {
        let request: CreateUserRequest = Faker.fake();
        assert!(!request.first_name.is_empty());
        assert!(!request.last_name.is_empty());
        assert!(request.password.len() >= 10);
    }
}
```

### 3. Complex Test Data Generation

```rust
// src/models/order.rs
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct Order {
    #[cfg_attr(test, dummy(faker = "10000..99999"))]
    pub id: u32,
    
    #[cfg_attr(test, dummy(faker = "1000..9999"))]
    pub user_id: u32,
    
    #[cfg_attr(test, dummy(faker = "OrderStatus::random()"))]
    pub status: OrderStatus,
    
    #[cfg_attr(test, dummy(faker = "1..10"))]
    pub items: Vec<OrderItem>,
    
    #[cfg_attr(test, dummy(expr = "calculate_total(&items)"))]
    pub total: Decimal,
    
    #[cfg_attr(test, dummy(faker = "fake::faker::chrono::en::DateTimeBetween(
        chrono::Utc::now() - chrono::Duration::days(30),
        chrono::Utc::now()
    )"))]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct OrderItem {
    #[cfg_attr(test, dummy(faker = "fake::faker::lorem::en::Word()"))]
    pub product_name: String,
    
    #[cfg_attr(test, dummy(faker = "1..5"))]
    pub quantity: u32,
    
    #[cfg_attr(test, dummy(faker = "10.0..500.0"))]
    pub price: f64,
}

#[derive(Debug, Clone)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

#[cfg(test)]
impl OrderStatus {
    fn random() -> Self {
        use fake::faker::number::en::NumberWithFormat;
        let n: u8 = NumberWithFormat("^").fake();
        match n % 5 {
            0 => OrderStatus::Pending,
            1 => OrderStatus::Processing,
            2 => OrderStatus::Shipped,
            3 => OrderStatus::Delivered,
            _ => OrderStatus::Cancelled,
        }
    }
}

#[cfg(test)]
fn calculate_total(items: &[OrderItem]) -> Decimal {
    items.iter()
        .map(|item| Decimal::from_f64_retain(item.price).unwrap() * Decimal::from(item.quantity))
        .sum()
}
```

## Integration Testing Structure

### 1. API Integration Tests

```rust
// crates/api/tests/user_api_tests.rs
use api::test_helpers::{spawn_app, TestApp};
use fake::{Fake, Faker};
use reqwest::StatusCode;

#[tokio::test]
async fn test_create_user_endpoint() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    
    let user_data = CreateUserRequest {
        first_name: fake::faker::name::en::FirstName().fake(),
        last_name: fake::faker::name::en::LastName().fake(),
        email: fake::faker::internet::en::SafeEmail().fake(),
        password: fake::faker::internet::en::Password(10..20).fake(),
    };

    // Act
    let response = client
        .post(&format!("{}/users", app.address))
        .json(&user_data)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let created_user: User = response.json().await.expect("Failed to parse response");
    assert_eq!(created_user.email, user_data.email);
    assert_eq!(created_user.first_name, user_data.first_name);
}

#[tokio::test]
async fn test_get_user_by_id() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Create a user first
    let user_data: CreateUserRequest = Faker.fake();
    let create_response = client
        .post(&format!("{}/users", app.address))
        .json(&user_data)
        .send()
        .await
        .expect("Failed to create user");
    
    let created_user: User = create_response.json().await.expect("Failed to parse user");
    
    // Get the user
    let get_response = client
        .get(&format!("{}/users/{}", app.address, created_user.id))
        .send()
        .await
        .expect("Failed to get user");
    
    assert_eq!(get_response.status(), StatusCode::OK);
    
    let retrieved_user: User = get_response.json().await.expect("Failed to parse user");
    assert_eq!(retrieved_user.id, created_user.id);
}
```

### 2. Test Helpers Module

```rust
// crates/test-utils/src/lib.rs
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    
    let connection_pool = configure_database(&configuration.database).await;

    let server = api::startup::run(listener, connection_pool.clone())
        .expect("Failed to bind address");
    
    let _ = tokio::spawn(server);
    
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
        
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
        
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
```

## Browser Testing with Fantoccini

### 1. Basic Browser Test Setup

```rust
// tests/browser_tests.rs
use fantoccini::{ClientBuilder, Locator};
use fake::{Fake, Faker};

#[tokio::test]
async fn test_user_registration_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Start the web driver (ensure geckodriver or chromedriver is running)
    let client = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await?;

    // Generate test data
    let first_name: String = fake::faker::name::en::FirstName().fake();
    let last_name: String = fake::faker::name::en::LastName().fake();
    let email: String = fake::faker::internet::en::SafeEmail().fake();
    let password: String = fake::faker::internet::en::Password(10..20).fake();

    // Navigate to registration page
    client.goto("http://localhost:3000/register").await?;

    // Fill out the form
    client
        .find(Locator::Css("input[name='first_name']"))
        .await?
        .send_keys(&first_name)
        .await?;
        
    client
        .find(Locator::Css("input[name='last_name']"))
        .await?
        .send_keys(&last_name)
        .await?;
        
    client
        .find(Locator::Css("input[name='email']"))
        .await?
        .send_keys(&email)
        .await?;
        
    client
        .find(Locator::Css("input[name='password']"))
        .await?
        .send_keys(&password)
        .await?;

    // Submit the form
    client
        .find(Locator::Css("button[type='submit']"))
        .await?
        .click()
        .await?;

    // Wait for success message
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Verify success
    let success_message = client
        .find(Locator::Css(".success-message"))
        .await?
        .text()
        .await?;
        
    assert!(success_message.contains("Registration successful"));

    client.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_login_flow() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await?;

    // First create a user via API
    let test_user = create_test_user().await?;

    // Navigate to login
    client.goto("http://localhost:3000/login").await?;

    // Login
    client
        .find(Locator::Css("input[name='email']"))
        .await?
        .send_keys(&test_user.email)
        .await?;
        
    client
        .find(Locator::Css("input[name='password']"))
        .await?
        .send_keys(&test_user.password)
        .await?;
        
    client
        .find(Locator::Css("button[type='submit']"))
        .await?
        .click()
        .await?;

    // Verify redirect to dashboard
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    let current_url = client.current_url().await?;
    assert!(current_url.as_ref().contains("/dashboard"));

    client.close().await?;
    Ok(())
}
```

### 2. Advanced Browser Testing with Page Objects

```rust
// tests/support/page_objects.rs
use fantoccini::{Client, Locator, Element};

pub struct LoginPage {
    client: Client,
}

impl LoginPage {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn goto(&self) -> Result<(), fantoccini::error::CmdError> {
        self.client.goto("http://localhost:3000/login").await
    }

    pub async fn fill_email(&self, email: &str) -> Result<(), fantoccini::error::CmdError> {
        self.client
            .find(Locator::Css("input[name='email']"))
            .await?
            .send_keys(email)
            .await
    }

    pub async fn fill_password(&self, password: &str) -> Result<(), fantoccini::error::CmdError> {
        self.client
            .find(Locator::Css("input[name='password']"))
            .await?
            .send_keys(password)
            .await
    }

    pub async fn submit(&self) -> Result<(), fantoccini::error::CmdError> {
        self.client
            .find(Locator::Css("button[type='submit']"))
            .await?
            .click()
            .await
    }

    pub async fn get_error_message(&self) -> Result<Option<String>, fantoccini::error::CmdError> {
        match self.client.find(Locator::Css(".error-message")).await {
            Ok(element) => Ok(Some(element.text().await?)),
            Err(_) => Ok(None),
        }
    }
}

// Usage in tests
#[tokio::test]
async fn test_login_with_page_object() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await?;

    let login_page = LoginPage::new(client);
    
    login_page.goto().await?;
    login_page.fill_email("test@example.com").await?;
    login_page.fill_password("wrong_password").await?;
    login_page.submit().await?;
    
    let error = login_page.get_error_message().await?;
    assert!(error.is_some());
    assert!(error.unwrap().contains("Invalid credentials"));

    Ok(())
}
```

## Multi-Crate Workspace Testing

### 1. Workspace Configuration

```toml
# Cargo.toml (root)
[workspace]
members = [
    "crates/api",
    "crates/core",
    "crates/web",
    "crates/test-utils",
]
resolver = "2"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
fake = { version = "2.9", features = ["derive"] }

# Development dependencies
[workspace.dev-dependencies]
criterion = "0.5"
```

### 2. Crate-Specific Testing

```toml
# crates/core/Cargo.toml
[package]
name = "core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
sqlx = { workspace = true }

[dev-dependencies]
fake = { workspace = true }
test-utils = { path = "../test-utils" }
```

### 3. Cross-Crate Integration Tests

```rust
// crates/api/tests/cross_crate_tests.rs
use core::services::UserService;
use api::handlers::UserHandler;
use test_utils::TestDatabase;

#[tokio::test]
async fn test_user_service_integration() {
    let db = TestDatabase::new().await;
    let user_service = UserService::new(db.pool.clone());
    let handler = UserHandler::new(user_service);
    
    // Test that API handler correctly uses core service
    let create_request: CreateUserRequest = Faker.fake();
    let result = handler.create_user(create_request).await;
    
    assert!(result.is_ok());
}
```

## Best Practices

### 1. Test Organization

```rust
// src/lib.rs
pub mod models;
pub mod services;
pub mod handlers;

#[cfg(test)]
mod tests {
    use super::*;
    
    mod unit_tests {
        use super::*;
        
        #[test]
        fn test_internal_logic() {
            // Unit tests for internal functions
        }
    }
    
    mod integration_tests {
        use super::*;
        
        #[test]
        fn test_public_api() {
            // Tests for public API
        }
    }
}
```

### 2. Test Data Builder Pattern

```rust
// tests/support/builders.rs
use fake::{Fake, Faker};

pub struct UserBuilder {
    first_name: String,
    last_name: String,
    email: String,
    age: u8,
}

impl Default for UserBuilder {
    fn default() -> Self {
        Self {
            first_name: fake::faker::name::en::FirstName().fake(),
            last_name: fake::faker::name::en::LastName().fake(),
            email: fake::faker::internet::en::SafeEmail().fake(),
            age: (18..80).fake(),
        }
    }
}

impl UserBuilder {
    pub fn with_email(mut self, email: String) -> Self {
        self.email = email;
        self
    }
    
    pub fn with_age(mut self, age: u8) -> Self {
        self.age = age;
        self
    }
    
    pub fn build(self) -> User {
        User {
            id: Faker.fake(),
            first_name: self.first_name,
            last_name: self.last_name,
            email: self.email,
            age: self.age,
            is_active: true,
        }
    }
}

// Usage
#[test]
fn test_with_builder() {
    let user = UserBuilder::default()
        .with_email("specific@example.com".to_string())
        .with_age(25)
        .build();
        
    assert_eq!(user.email, "specific@example.com");
    assert_eq!(user.age, 25);
}
```

### 3. Test Fixtures and Setup

```rust
// tests/support/fixtures.rs
use once_cell::sync::Lazy;
use std::sync::Arc;

pub struct TestFixtures {
    pub db_pool: PgPool,
    pub http_client: reqwest::Client,
    pub test_server_url: String,
}

static TEST_FIXTURES: Lazy<Arc<TestFixtures>> = Lazy::new(|| {
    Arc::new(
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(setup_test_fixtures())
    )
});

async fn setup_test_fixtures() -> TestFixtures {
    // Setup database
    let db_pool = setup_test_database().await;
    
    // Start test server
    let test_server_url = start_test_server(db_pool.clone()).await;
    
    // Create HTTP client
    let http_client = reqwest::Client::new();
    
    TestFixtures {
        db_pool,
        http_client,
        test_server_url,
    }
}

pub fn get_test_fixtures() -> Arc<TestFixtures> {
    TEST_FIXTURES.clone()
}
```

### 4. Async Test Helpers

```rust
// tests/support/async_helpers.rs
use tokio::time::{sleep, Duration, timeout};

pub async fn wait_for_condition<F, Fut>(
    condition: F,
    max_wait: Duration,
    check_interval: Duration,
) -> Result<(), String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + max_wait;
    
    while tokio::time::Instant::now() < deadline {
        if condition().await {
            return Ok(());
        }
        sleep(check_interval).await;
    }
    
    Err("Condition not met within timeout".to_string())
}

// Usage
#[tokio::test]
async fn test_async_operation() {
    perform_async_operation().await;
    
    wait_for_condition(
        || async { check_if_ready().await },
        Duration::from_secs(10),
        Duration::from_millis(100),
    )
    .await
    .expect("Operation did not complete in time");
}
```

### 5. Running Tests

```bash
# Run all tests in workspace
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run tests for specific crate
cargo test -p core

# Run browser tests (ensure WebDriver is running)
cargo test --test browser_tests

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel with specific thread count
cargo test -- --test-threads=4

# Use nextest for better test output and performance
cargo nextest run
```

## Summary

This comprehensive testing setup provides:

1. **Faker/Dummy Integration**: Automatic test data generation with type safety
2. **Clear Test Organization**: Separation of unit, integration, and E2E tests
3. **Browser Testing**: Full user flow testing with Fantoccini
4. **Workspace Support**: Testing across multiple crates with shared utilities
5. **Best Practices**: Builders, fixtures, and async helpers for maintainable tests

Key principles:
- Use unit tests for testing individual components in isolation
- Use integration tests for testing public APIs and module interactions
- Use browser tests for critical user flows
- Generate realistic test data with faker
- Keep tests fast, isolated, and deterministic
- Use shared test utilities to reduce duplication