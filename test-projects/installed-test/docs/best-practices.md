# Best Practices and Patterns

This guide covers recommended patterns, conventions, and best practices for developing with the installed-test framework.

## Table of Contents

1. [Code Organization](#code-organization)
2. [Error Handling](#error-handling)
3. [Database Patterns](#database-patterns)
4. [API Design](#api-design)
5. [Security Best Practices](#security-best-practices)
6. [Performance Optimization](#performance-optimization)
7. [Testing Strategies](#testing-strategies)
8. [Logging and Monitoring](#logging-and-monitoring)
9. [Configuration Management](#configuration-management)
10. [Deployment Practices](#deployment-practices)

## Code Organization

### Project Structure

Follow the established crate structure for consistency:

```
installed-test/
├── installed-test-server/     # Web server and API endpoints
├── installed-test-shared/     # Common types and utilities
├── installed-test-liveview/   # LiveView components and pages
├── docs/                        # Documentation
├── migrations/                  # Database migrations
└── tests/                       # Integration tests
```

### Module Organization

**Controllers** - Keep controllers thin and focused:

```rust
// Good: Simple, focused controller
pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.user_service.create_user(request).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

// Avoid: Business logic in controllers
pub async fn create_user_bad(/*...*/) -> Result<impl IntoResponse, AppError> {
    // Don't put validation, database calls, etc. directly in controllers
    if request.username.is_empty() { /* ... */ }
    let user = sqlx::query!(/*...*/).fetch_one(&state.db).await?;
    // ... complex business logic
}
```

**Services** - Encapsulate business logic:

```rust
pub struct UserService {
    repository: UserRepository,
    email_service: EmailService,
}

impl UserService {
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, AppError> {
        // Validate input
        self.validate_user_request(&request).await?;
        
        // Check business rules
        self.check_user_limits().await?;
        
        // Create user
        let user = self.repository.create(request).await?;
        
        // Side effects
        self.email_service.send_welcome_email(&user).await?;
        
        Ok(user)
    }
}
```

### Naming Conventions

- **Types**: PascalCase (`UserService`, `CreateUserRequest`)
- **Functions**: snake_case (`create_user`, `validate_input`)
- **Constants**: SCREAMING_SNAKE_CASE (`MAX_FILE_SIZE`, `DEFAULT_TIMEOUT`)
- **Modules**: snake_case (`user_service`, `auth_middleware`)

## Error Handling

### Comprehensive Error Types

Define specific error types for different failure modes:

```rust
#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found: {id}")]
    NotFound { id: Uuid },
    
    #[error("Username already taken: {username}")]
    UsernameTaken { username: String },
    
    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },
    
    #[error("User limit exceeded: {current}/{max}")]
    LimitExceeded { current: u32, max: u32 },
}
```

### Error Conversion Patterns

Use `From` traits for automatic error conversion:

```rust
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound,
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                AppError::Conflict("Resource already exists".to_string())
            }
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}
```

### Result Handling

Always handle errors appropriately:

```rust
// Good: Explicit error handling
match user_service.create_user(request).await {
    Ok(user) => Ok(Json(user)),
    Err(UserError::UsernameTaken { username }) => {
        Err(AppError::Validation(format!("Username '{}' is already taken", username)))
    }
    Err(e) => Err(AppError::from(e)),
}

// Better: Use the ? operator with proper error conversion
let user = user_service.create_user(request).await?;
Ok(Json(user))
```

## Database Patterns

### Repository Pattern

Encapsulate data access in repository types:

```rust
pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT id, username, email, created_at, updated_at FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
    }
    
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT id, username, email, created_at, updated_at FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await
    }
}
```

### Query Optimization

- Use indexes for frequently queried columns
- Limit result sets with `LIMIT` and `OFFSET`
- Use `fetch_optional` for queries that might not return results
- Prefer `query_as!` macro for type safety

```rust
// Good: Paginated query with indexes
pub async fn list_users(
    &self,
    limit: u32,
    offset: u32,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, username, email, created_at, updated_at 
         FROM users 
         ORDER BY created_at DESC 
         LIMIT $1 OFFSET $2",
        limit as i64,
        offset as i64
    )
    .fetch_all(&self.pool)
    .await
}
```

### Migration Best Practices

- Make migrations idempotent
- Use transactions for complex migrations
- Add indexes in separate migrations
- Include rollback scripts

```sql
-- Good migration structure
BEGIN;

-- Create table
CREATE TABLE IF NOT EXISTS posts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    content TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add constraints
ALTER TABLE posts ADD CONSTRAINT posts_title_not_empty CHECK (length(title) > 0);

COMMIT;
```

## API Design

### RESTful Endpoints

Follow REST conventions for predictable APIs:

```
GET    /api/users          # List users
GET    /api/users/{id}     # Get specific user
POST   /api/users          # Create user
PUT    /api/users/{id}     # Update user
DELETE /api/users/{id}     # Delete user
```

### Response Consistency

Use consistent response formats:

```rust
// Success responses
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

// Error responses
#[derive(Serialize)]
pub struct ApiError {
    pub success: bool,
    pub error: String,
    pub message: String,
}

// Paginated responses
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}
```

### Input Validation

Validate all input data:

```rust
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    #[validate(regex = "^[a-zA-Z0-9_]+$")]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8))]
    pub password: Option<String>,
}
```

## Security Best Practices

### Authentication

Use secure authentication patterns:

```rust
// JWT with proper claims
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub user_id: Uuid,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

// Secure password hashing
pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::InternalError)
}
```

### Authorization

Implement role-based access control:

```rust
#[derive(Debug, PartialEq)]
pub enum Role {
    User,
    Admin,
    Moderator,
}

pub fn require_role(required: Role) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and_then(move |auth_header: Option<String>| {
            async move {
                let token = extract_token(auth_header)?;
                let claims = verify_token(token)?;
                let user_role = get_user_role(claims.user_id).await?;
                
                if user_role == required || user_role == Role::Admin {
                    Ok(())
                } else {
                    Err(warp::reject::custom(Unauthorized))
                }
            }
        })
}
```

### Input Sanitization

Always sanitize user input:

```rust
pub fn sanitize_html(input: &str) -> String {
    // Use a library like ammonia for HTML sanitization
    ammonia::clean(input)
}

pub fn validate_slug(slug: &str) -> Result<(), ValidationError> {
    if slug.chars().all(|c| c.is_alphanumeric() || c == '-') {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid slug format"))
    }
}
```

## Performance Optimization

### Database Performance

- Use connection pooling
- Implement query result caching
- Optimize expensive queries

```rust
// Connection pool configuration
let pool = SqlitePoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .max_lifetime(Duration::from_secs(1800))
    .idle_timeout(Duration::from_secs(600))
    .connect(&database_url)
    .await?;
```

### Caching Strategies

Implement appropriate caching:

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct CacheService {
    cache: RwLock<HashMap<String, (String, std::time::Instant)>>,
    ttl: Duration,
}

impl CacheService {
    pub async fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().await;
        if let Some((value, inserted_at)) = cache.get(key) {
            if inserted_at.elapsed() < self.ttl {
                Some(value.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub async fn set(&self, key: String, value: String) {
        let mut cache = self.cache.write().await;
        cache.insert(key, (value, std::time::Instant::now()));
    }
}
```

### Async Best Practices

- Use `tokio::spawn` for CPU-bound tasks
- Implement timeouts for external calls
- Use `select!` for concurrent operations

```rust
use tokio::time::{timeout, Duration};

pub async fn fetch_user_data(user_id: Uuid) -> Result<UserData, AppError> {
    let timeout_duration = Duration::from_secs(10);
    
    timeout(timeout_duration, async {
        // Concurrent requests
        let (profile, preferences, activity) = tokio::try_join!(
            fetch_user_profile(user_id),
            fetch_user_preferences(user_id),
            fetch_user_activity(user_id)
        )?;
        
        Ok(UserData {
            profile,
            preferences,
            activity,
        })
    })
    .await
    .map_err(|_| AppError::Timeout)?
}
```

## Testing Strategies

### Unit Tests

Test individual components in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_slugify() {
        assert_eq!(utils::slugify("Hello World!"), "hello-world");
        assert_eq!(utils::slugify(""), "");
        assert_eq!(utils::slugify("123-test"), "123-test");
    }
    
    #[tokio::test]
    async fn test_user_creation() {
        let service = UserService::new_test();
        let request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let user = service.create_user(request).await.unwrap();
        assert_eq!(user.username, "testuser");
    }
}
```

### Integration Tests

Test complete workflows:

```rust
#[tokio::test]
async fn test_user_registration_flow() {
    let app = test_app().await;
    
    // Create user
    let response = app
        .post("/api/users")
        .json(&CreateUserRequest {
            username: "newuser".to_string(),
            email: "new@example.com".to_string(),
        })
        .await;
    
    assert_eq!(response.status(), 201);
    
    let user: User = response.json().await;
    assert_eq!(user.username, "newuser");
    
    // Verify user exists
    let get_response = app.get(&format!("/api/users/{}", user.id)).await;
    assert_eq!(get_response.status(), 200);
}
```

### Test Utilities

Create helpers for common test scenarios:

```rust
pub async fn create_test_user() -> User {
    User {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub async fn setup_test_db() -> DbPool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}
```

## Logging and Monitoring

### Structured Logging

Use structured logging with context:

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, AppError> {
    info!(username = %request.username, "Creating new user");
    
    match self.repository.create(request).await {
        Ok(user) => {
            info!(user_id = %user.id, "User created successfully");
            Ok(user)
        }
        Err(e) => {
            error!(error = %e, "Failed to create user");
            Err(e.into())
        }
    }
}
```

### Health Checks

Implement comprehensive health checks:

```rust
pub async fn detailed_health_check(State(state): State<AppState>) -> impl IntoResponse {
    let mut health = HashMap::new();
    
    // Database health
    health.insert("database", check_database(&state.db).await);
    
    // External service health
    health.insert("email_service", check_email_service().await);
    
    // Disk space
    health.insert("disk_space", check_disk_space().await);
    
    let all_healthy = health.values().all(|&status| status);
    let status_code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (status_code, Json(health))
}
```

## Configuration Management

### Environment-based Configuration

Use environment variables for configuration:

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub smtp_url: Option<String>,
    pub max_file_size: usize,
    pub rate_limit: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            smtp_url: env::var("SMTP_URL").ok(),
            max_file_size: env::var("MAX_FILE_SIZE")
                .unwrap_or("10485760".to_string())
                .parse()?,
            rate_limit: env::var("RATE_LIMIT")
                .unwrap_or("100".to_string())
                .parse()?,
        })
    }
}
```

### Feature Flags

Implement feature flags for gradual rollouts:

```rust
#[derive(Debug, Clone)]
pub struct FeatureFlags {
    pub enable_new_feature: bool,
    pub enable_beta_ui: bool,
    pub maintenance_mode: bool,
}

impl FeatureFlags {
    pub fn from_env() -> Self {
        Self {
            enable_new_feature: env_bool("ENABLE_NEW_FEATURE", false),
            enable_beta_ui: env_bool("ENABLE_BETA_UI", false),
            maintenance_mode: env_bool("MAINTENANCE_MODE", false),
        }
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}
```

## Deployment Practices

### Docker Best Practices

Use multi-stage builds:

```dockerfile
# Build stage
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/installed-test-server /usr/local/bin/
CMD ["installed-test-server"]
```

### Health Checks

Implement proper health checks:

```yaml
# docker-compose.yml
services:
  app:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s
```

### Graceful Shutdown

Handle shutdown signals properly:

```rust
use tokio::signal;

pub async fn run_server(app: Router, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
```

Following these patterns and practices will help you build robust, maintainable, and scalable applications with the installed-test framework.