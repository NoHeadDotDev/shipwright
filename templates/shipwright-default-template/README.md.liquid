# {{project-name}}

A Shipwright-powered Rust web application with hot reload support.

## Project Structure

```
{{project-name}}/
├── {{project-name}}-server/     # Main web server
{%- if project_type == "full-stack" or project_type == "liveview-app" %}
├── {{project-name}}-frontend/   # Frontend assets and build
{%- endif %}
{%- if use_liveview %}
├── {{project-name}}-liveview/   # LiveView components
{%- endif %}
├── {{project-name}}-shared/     # Shared code between crates
├── Cargo.toml                   # Workspace configuration
└── Shipwright.toml             # Shipwright configuration
```

## Getting Started

### Prerequisites

- Rust (latest stable)
- Shipwright CLI
{%- if project_type == "full-stack" or project_type == "liveview-app" %}
- Node.js and npm
{%- endif %}
{%- if database == "postgres" %}
- PostgreSQL
{%- elif database == "sqlite" %}
- SQLite
{%- elif database == "mysql" %}
- MySQL
{%- endif %}

### Development

1. Install dependencies:
   ```bash
   cargo build
{%- if project_type == "full-stack" or project_type == "liveview-app" %}
   cd {{project-name}}-frontend && npm install
{%- endif %}
   ```

2. Start the development server with hot reload:
   ```bash
   shipwright dev
   ```

3. Open your browser to `http://localhost:{{port}}`

### Production

Build for production:
```bash
shipwright build --release
```

Run the production server:
```bash
shipwright serve --release
```

## Configuration

Edit `Shipwright.toml` to customize:
- Server port and host
- Hot reload settings
- Watch paths and patterns
{%- if project_type == "full-stack" or project_type == "liveview-app" %}
- Frontend build configuration
{%- endif %}
{%- if database != "none" %}
- Database connection
{%- endif %}

## Features

- **Framework**: {{framework}}
{%- if hot_reload %}
- **Hot Reload**: Automatic browser refresh on code changes
{%- endif %}
{%- if use_liveview %}
- **LiveView**: Server-rendered interactive components
{%- endif %}
{%- if project_type == "full-stack" or project_type == "liveview-app" %}
- **Frontend**: {{frontend_framework}} build tooling
{%- endif %}
{%- if database != "none" %}
- **Database**: {{database}} with SQLx
{%- endif %}

## Project Layout

### Server (`{{project-name}}-server`)
The main web server handling HTTP requests and WebSocket connections.

{%- if use_liveview %}
### LiveView (`{{project-name}}-liveview`)
Server-rendered interactive components using Shipwright LiveView.
{%- endif %}

{%- if project_type == "full-stack" or project_type == "liveview-app" %}
### Frontend (`{{project-name}}-frontend`)
Frontend assets, styles, and client-side JavaScript/TypeScript.
{%- endif %}

### Shared (`{{project-name}}-shared`)
Common types, utilities, and business logic shared between crates.

## Environment Variables

- `RUST_LOG`: Set logging level (default: `info`)
- `PORT`: Override server port (default: `{{port}}`)
{%- if database != "none" %}
- `DATABASE_URL`: Database connection string
{%- endif %}

## License

This project is licensed under the MIT OR Apache-2.0 license.