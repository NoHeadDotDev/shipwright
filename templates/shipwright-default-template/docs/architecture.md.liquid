# Architecture Overview

This document provides an overview of the {{project-name}} application architecture, explaining how the different components work together to create a modern, scalable web application.

## Table of Contents

1. [High-Level Architecture](#high-level-architecture)
2. [Crate Structure](#crate-structure)
3. [Request Flow](#request-flow)
4. [Data Layer](#data-layer)
5. [Business Logic](#business-logic)
6. [Presentation Layer](#presentation-layer)
7. [Security Architecture](#security-architecture)
8. [Performance Considerations](#performance-considerations)

## High-Level Architecture

{{project-name}} follows a layered architecture pattern with clear separation of concerns:

```
┌─────────────────────────────────────────────────────┐
│                    Client Layer                     │
├─────────────────────────────────────────────────────┤
│                 Presentation Layer                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │   Static    │  │   REST API  │  │  LiveView   │  │
│  │   Assets    │  │ Controllers │  │ Components  │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  │
├─────────────────────────────────────────────────────┤
│                 Business Logic Layer                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │  Services   │  │   Models    │  │ Validation  │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  │
├─────────────────────────────────────────────────────┤
│                    Data Layer                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │  Database   │  │    Cache    │  │  External   │  │
│  │   (SQLx)    │  │             │  │    APIs     │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Crate Structure

The application is organized into multiple crates for modularity and reusability:

### {{project-name}}-server

The main web server crate containing:

- **HTTP Server**: {%- if framework == "axum" %}Axum{%- elif framework == "actix-web" %}Actix Web{%- elif framework == "rocket" %}Rocket{%- endif %} web framework setup
- **Controllers**: HTTP request handlers and route definitions
- **Middleware**: Authentication, logging, CORS, etc.
- **Configuration**: Environment-based configuration management

```rust
// src/main.rs - Application entry point
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::init();
    
    // Load configuration
    let config = Config::from_env()?;
    
    // Setup database connection
    {%- if database != "none" %}
    let db = setup_database(&config).await?;
    {%- endif %}
    
    // Build application state
    let app_state = AppState {
        config: config.clone(),
        {%- if database != "none" %}
        db,
        {%- endif %}
    };
    
    // Create router with middleware
    let app = create_router(app_state);
    
    // Start server
    start_server(app, &config).await
}
```

### {{project-name}}-shared

Common types and utilities shared across crates:

- **Models**: Data structures and domain types
- **Error Types**: Application-wide error handling
- **Utilities**: Helper functions and common logic
- **Configuration**: Shared configuration types

```rust
// src/lib.rs - Shared types and functions
pub mod error;
pub mod models;
pub mod utils;
pub mod config;

// Re-export commonly used types
pub use error::AppError;
pub use config::Config;
```

{%- if use_liveview %}
### {{project-name}}-liveview

LiveView components and real-time UI logic:

- **Components**: Reusable UI components
- **Pages**: Full-page LiveView applications
- **Router**: LiveView-specific routing
- **State Management**: Component state and message handling

```rust
// src/lib.rs - LiveView exports
pub mod components;
pub mod pages;
pub mod router;

// Re-export LiveView functionality
pub use shipwright_liveview::{LiveView, Html, Updated, EventData};
pub use shipwright_liveview_macros::html;
```
{%- endif %}

## Request Flow

### HTTP API Requests

```mermaid
sequenceDiagram
    participant Client
    participant Server
    participant Controller
    participant Service
    participant Database

    Client->>Server: HTTP Request
    Server->>Controller: Route to handler
    Controller->>Service: Business logic call
    Service->>Database: Data operation
    Database-->>Service: Result
    Service-->>Controller: Response data
    Controller-->>Server: HTTP Response
    Server-->>Client: JSON/HTML Response
```

### LiveView Interactions

```mermaid
sequenceDiagram
    participant Browser
    participant LiveView
    participant Component
    participant Database

    Browser->>LiveView: Initial page load
    LiveView-->>Browser: Rendered HTML + WebSocket
    Browser->>LiveView: User interaction (event)
    LiveView->>Component: Update message
    Component->>Database: Data change (if needed)
    Database-->>Component: Updated data
    Component-->>LiveView: New state
    LiveView-->>Browser: DOM patch
```

## Data Layer

### Database Architecture

{%- if database != "none" %}
The application uses {{database | capitalize}} as the primary database with SQLx for type-safe SQL queries:

```rust
// Database connection pool
pub type DbPool = sqlx::{%- if database == "postgres" %}PgPool{%- elif database == "sqlite" %}SqlitePool{%- elif database == "mysql" %}MySqlPool{%- endif %};

// Repository pattern for data access
pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub async fn create(&self, user: CreateUserRequest) -> Result<User, AppError> {
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (id, username, email) VALUES ($1, $2, $3) RETURNING *",
            Uuid::new_v4(),
            user.username,
            user.email
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
}
```
{%- endif %}

### Migration Strategy

Database schema changes are managed through SQL migrations:

```
migrations/
├── 001_initial_setup.sql      # Core tables and indexes
├── 002_seed_data.sql          # Initial data
└── 003_add_user_preferences.sql # Feature additions
```

### Data Models

Domain models are defined in the shared crate:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
}
```

## Business Logic

### Service Layer

Business logic is encapsulated in service modules:

```rust
pub struct UserService {
    repository: UserRepository,
    email_service: EmailService,
}

impl UserService {
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, AppError> {
        // Validate input
        request.validate()?;
        
        // Check if user already exists
        if self.repository.find_by_email(&request.email).await?.is_some() {
            return Err(AppError::UserAlreadyExists);
        }
        
        // Create user
        let user = self.repository.create(request).await?;
        
        // Send welcome email
        self.email_service.send_welcome_email(&user).await?;
        
        Ok(user)
    }
}
```

### Error Handling

Centralized error handling with custom error types:

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("User already exists")]
    UserAlreadyExists,
}

// Convert to HTTP responses
{%- if framework == "axum" %}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AppError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "Invalid input"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };
        
        (status, Json(json!({"error": message}))).into_response()
    }
}
{%- endif %}
```

## Presentation Layer

### REST API Controllers

Controllers handle HTTP requests and responses:

```rust
pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.user_service.create_user(request).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.user_service.get_user(id).await?;
    Ok(Json(user))
}
```

{%- if use_liveview %}
### LiveView Components

Interactive components with real-time updates:

```rust
#[derive(Clone)]
pub struct UserProfile {
    user: User,
    editing: bool,
    form_data: UserFormData,
}

impl LiveView for UserProfile {
    type Message = ProfileMessage;
    
    fn update(mut self, msg: Self::Message, data: Option<EventData>) -> Updated<Self> {
        match msg {
            ProfileMessage::StartEditing => {
                self.editing = true;
                self.form_data = UserFormData::from(&self.user);
            }
            ProfileMessage::SaveChanges => {
                // Validate and save changes
                if self.form_data.is_valid() {
                    self.user.update_from(&self.form_data);
                    self.editing = false;
                }
            }
            ProfileMessage::CancelEditing => {
                self.editing = false;
            }
        }
        Updated::new(self)
    }
    
    fn render(&self) -> Html<Self::Message> {
        if self.editing {
            self.render_edit_form()
        } else {
            self.render_profile_view()
        }
    }
}
```
{%- endif %}

## Security Architecture

### Authentication & Authorization

```rust
// JWT-based authentication middleware
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));
    
    if let Some(token) = auth_header {
        match verify_jwt(token, &state.config.jwt_secret) {
            Ok(claims) => {
                request.extensions_mut().insert(claims);
                Ok(next.run(request).await)
            }
            Err(_) => Err(AppError::Unauthorized),
        }
    } else {
        Err(AppError::Unauthorized)
    }
}
```

### Input Validation

```rust
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    
    #[validate(length(min = 1, max = 5000))]
    pub content: String,
    
    #[validate(url)]
    pub featured_image: Option<String>,
}

// Automatic validation in controllers
pub async fn create_post(
    auth: AuthenticatedUser,
    Validated(Json(request)): Validated<Json<CreatePostRequest>>,
) -> Result<impl IntoResponse, AppError> {
    // Request is automatically validated
    let post = create_post_for_user(auth.user_id, request).await?;
    Ok((StatusCode::CREATED, Json(post)))
}
```

### CORS and Security Headers

```rust
let app = Router::new()
    .route("/api/users", post(create_user))
    .layer(
        CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([AUTHORIZATION, CONTENT_TYPE])
    )
    .layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(TimeoutLayer::new(Duration::from_secs(30)))
            .layer(middleware::from_fn(security_headers_middleware))
    );
```

## Performance Considerations

### Database Optimization

- **Connection Pooling**: Reuse database connections
- **Query Optimization**: Use indexes and efficient queries
- **Prepared Statements**: SQLx provides compile-time query checking

```rust
// Efficient pagination
pub async fn list_users(
    pool: &DbPool,
    page: u32,
    per_page: u32,
) -> Result<Vec<User>, sqlx::Error> {
    let offset = (page - 1) * per_page;
    
    sqlx::query_as!(
        User,
        "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        per_page as i64,
        offset as i64
    )
    .fetch_all(pool)
    .await
}
```

### Caching Strategy

- **Response Caching**: Cache frequently accessed data
- **Session Storage**: Efficient session management
- **Static Asset Caching**: Long-term caching for assets

```rust
// In-memory cache for session data
use dashmap::DashMap;

pub struct SessionCache {
    sessions: DashMap<String, UserSession>,
}

impl SessionCache {
    pub fn get(&self, token: &str) -> Option<UserSession> {
        self.sessions.get(token).map(|entry| entry.clone())
    }
    
    pub fn insert(&self, token: String, session: UserSession) {
        self.sessions.insert(token, session);
    }
}
```

### Async Performance

- **Non-blocking I/O**: All I/O operations are async
- **Request Concurrency**: Handle multiple requests concurrently
- **Background Tasks**: Process long-running tasks asynchronously

```rust
// Background task for email sending
async fn process_email_queue(email_service: EmailService) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        if let Ok(emails) = email_service.get_pending_emails().await {
            for email in emails {
                if let Err(e) = email_service.send_email(email).await {
                    error!("Failed to send email: {}", e);
                }
            }
        }
    }
}
```

## Monitoring and Observability

### Structured Logging

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(pool))]
pub async fn create_user(
    pool: &DbPool,
    request: CreateUserRequest,
) -> Result<User, AppError> {
    info!("Creating user with username: {}", request.username);
    
    match user_repository.create(request).await {
        Ok(user) => {
            info!("User created successfully with ID: {}", user.id);
            Ok(user)
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            Err(e)
        }
    }
}
```

### Health Checks

```rust
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let db_healthy = check_database_health(&state.db).await;
    
    let status = if db_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (status, Json(json!({
        "status": if db_healthy { "healthy" } else { "unhealthy" },
        "database": db_healthy,
        "timestamp": Utc::now(),
    })))
}
```

This architecture provides a solid foundation for building scalable, maintainable web applications with Rust and the Shipwright framework. The clear separation of concerns makes the codebase easy to understand, test, and extend.