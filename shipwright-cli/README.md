# Shipwright CLI

A standalone command-line interface for creating and managing Phoenix-style Rust web applications. Shipwright provides rapid development with real-time LiveView updates, database integration, and modern development workflows.

## Installation

```bash
# Install from crates.io (coming soon)
cargo install shipwright-cli

# Or build from source
git clone https://github.com/NoHeadDotDev/shipwright
cd shipwright/shipwright-cli
cargo build --release
```

## Features

- **🚀 Project Generation**: Create new projects from GitHub templates  
- **⚡ Hot Reload Development**: Automatic recompilation and browser refresh
- **🔧 Production Ready**: Optimized builds and deployment configurations
- **📊 Database Integration**: SQLx with migrations and entity management
- **🎨 LiveView Support**: Real-time UI updates without JavaScript frameworks
- **🏗️ Modern Stack**: Axum 0.8, SQLx, thiserror/eyre error handling

## Quick Start

```bash
# Create a new project
shipwright new my-app

# Enter the project directory
cd my-app

# Start development server
shipwright dev
```

Your application will be available at `http://localhost:3000` with hot reload enabled!

## Commands

### `shipwright new <name>`
Create a new Shipwright project from templates hosted on GitHub.

```bash
shipwright new my-app                           # Use default template
shipwright new my-app --template shipwright     # Use full framework template  
shipwright new my-app --template username/repo  # Use custom GitHub template
shipwright new my-app --directory /path/to/dir  # Custom output directory
```

**Available Templates:**
- `default`: Complete project with LiveView, database, and modern Rust stack
- `shipwright`: Full framework template with all components

### `shipwright dev`
Start the development server with hot reload functionality.

```bash
shipwright dev              # Start on default port (3000)
shipwright dev --port 8080  # Start on custom port
shipwright dev --no-watch   # Disable file watching
```

### `shipwright serve`
Start a production server for serving built applications.

```bash
shipwright serve                    # Serve on default port (8080)
shipwright serve --port 3000       # Serve on custom port
shipwright serve --release          # Use release build
```

### `shipwright build`
Build the application for production.

```bash
shipwright build              # Debug build
shipwright build --release   # Release build
```

## Project Structure

Generated projects follow this structure:

```
my-app/
├── Cargo.toml              # Workspace configuration
├── my-app-config/          # Configuration management crate  
├── my-app-db/              # Database models and migrations
├── my-app-web/             # Web server with Axum and LiveView
├── migrations/             # Database migration files
├── static/                 # Static assets (CSS, JS, images)
├── Shipwright.toml         # Framework configuration
└── README.md               # Project documentation
```

## Configuration

Projects include a `Shipwright.toml` configuration file:

```toml
[app]
name = "my-app"
port = 3000

[database]
url = "sqlite:my-app.db"
auto_migrate = true

[hot_reload]
enabled = true
port = 3001
watch_paths = ["src", "assets"]

[build]
release = false
features = []
```

## Templates

The CLI fetches templates from GitHub, making it fully standalone. Templates include:

- **Modern Rust Web Stack**: Axum 0.8.4, SQLx, Tokio
- **LiveView Integration**: Real-time UI updates
- **Database Setup**: Migrations, entities, repository patterns
- **Asset Pipeline**: CSS/JS processing and optimization
- **Testing Framework**: Unit, integration, and browser tests
- **Production Deployment**: Docker, CI/CD configurations

## Development

The CLI automatically detects workspace structure and provides:
- Hot reload with file watching
- Database migration management  
- Asset compilation and serving
- Comprehensive error handling
- Development vs production configurations

## Examples

```bash
# Create and run a new project
shipwright new blog-app
cd blog-app
shipwright dev

# Production deployment
shipwright build --release
shipwright serve --release --port 80

# Use custom template
shipwright new my-api --template https://github.com/username/api-template
```

## Contributing

Contributions are welcome! Please see our [contributing guidelines](../CONTRIBUTING.md) for details.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.