# {{crate_name}}-web

The web server crate for the {{crate_name}} application, built with Axum and Shipwright LiveView.

## Features

- **Modern Web Framework**: Built on Axum 0.8.4 with full async/await support
- **LiveView Integration**: Real-time UI updates with Shipwright LiveView
- **Hot Reload**: Development server with automatic reloading on file changes
- **Modular Architecture**: Clean separation of concerns with organized modules
- **Static Asset Serving**: Built-in static file serving from the `assets/` directory
- **Comprehensive Middleware**: Request logging, CORS, compression, and timeout handling
- **Type-Safe APIs**: JSON APIs with strong typing and validation

## Project Structure

```
src/
├── main.rs              # Application entry point
├── app.rs               # Application setup and middleware configuration
├── state.rs             # Shared application state
├── routes/              # Route definitions
│   ├── mod.rs
│   ├── api.rs          # REST API routes
│   ├── health.rs       # Health check endpoint
│   └── live_view.rs    # LiveView routes
├── controllers/         # Request handlers
│   ├── mod.rs
│   ├── api/            # API controllers
│   │   ├── mod.rs
│   │   ├── users.rs    # User management endpoints
│   │   └── version.rs  # Version information
│   └── live_view/      # LiveView controllers
│       ├── mod.rs
│       ├── home.rs     # Home page LiveView
│       ├── counter.rs  # Interactive counter example
│       └── dashboard.rs # Dashboard with live stats
└── middleware/          # Custom middleware
    ├── mod.rs
    ├── request_id.rs   # Request ID generation
    └── logging.rs      # Request logging
```

## Routes

### API Routes (`/api/`)
- `GET /api/users` - List all users
- `POST /api/users` - Create a new user
- `GET /api/users/{id}` - Get user by ID
- `PUT /api/users/{id}` - Update user
- `DELETE /api/users/{id}` - Delete user
- `GET /api/version` - Get application version info

### LiveView Routes (`/live/`)
- `GET /live/` - Home page with LiveView
- `GET /live/counter` - Interactive counter example
- `GET /live/dashboard` - Real-time dashboard

### Static Routes
- `GET /assets/*` - Static assets (CSS, images, etc.)
- `GET /bundle.js` - Shipwright LiveView JavaScript bundle
- `GET /health` - Health check endpoint

## Development

### Running the Server

```bash
cargo run
```

The server will start on `http://localhost:3000` by default. You can change the port by setting the `PORT` environment variable.

### Hot Reload

In development mode, the server automatically starts a hot reload server on `ws://localhost:3001` that watches for file changes and triggers browser reloads.

### Environment Variables

- `PORT` - Server port (default: 3000)
- `RUST_LOG` - Logging level (default: `{{crate_name}}_web=debug,tower_http=debug`)

## Adding New Routes

### API Routes

1. Add the route definition in `src/routes/api.rs`
2. Create the controller function in `src/controllers/api/`
3. Add the module export in `src/controllers/api/mod.rs`

Example:
```rust
// In src/routes/api.rs
.route("/posts", get(api::posts::list_posts))

// In src/controllers/api/posts.rs
pub async fn list_posts(State(state): State<AppState>) -> Json<Vec<Post>> {
    // Implementation
}
```

### LiveView Routes

1. Add the route definition in `src/routes/live_view.rs`
2. Create the LiveView controller in `src/controllers/live_view/`
3. Implement the `LiveView` trait for your component

Example:
```rust
// In src/routes/live_view.rs
.route("/blog", get(live_view::blog::blog_page))

// In src/controllers/live_view/blog.rs
pub async fn blog_page(live: LiveViewUpgrade) -> impl IntoResponse {
    let view = BlogPage::new();
    live.response(move |embed| {
        html! { /* Your HTML template */ }
    })
}
```

## Static Assets

Place static files in the `assets/` directory. They will be served at `/assets/*`.

Example:
- `assets/styles.css` → `http://localhost:3000/assets/styles.css`
- `assets/images/logo.png` → `http://localhost:3000/assets/images/logo.png`

## Middleware

The application includes several middleware layers:

1. **Tracing** - HTTP request tracing
2. **Compression** - GZIP compression for responses  
3. **Timeout** - 30-second request timeout
4. **CORS** - Cross-origin resource sharing
5. **Request ID** - Unique ID for each request
6. **Logging** - Structured request/response logging

## Dependencies

The web crate depends on the following workspace crates:
- `{{crate_name}}-config` - Configuration management
- `{{crate_name}}-db` - Database operations

## Testing

Run tests with:

```bash
cargo test
```

Integration tests are located in the `tests/` directory.