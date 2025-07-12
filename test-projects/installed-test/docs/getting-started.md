# Getting Started with {{project-name}}

This guide will help you get {{project-name}} up and running on your local development environment.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (1.70 or later) - [Install Rust](https://rustup.rs/)
- **Database** - Choose one:
  {%- if database == "postgres" %}
  - PostgreSQL (12 or later) - [Install PostgreSQL](https://www.postgresql.org/download/)
  {%- elif database == "sqlite" %}
  - SQLite (3.35 or later) - Usually pre-installed on most systems
  {%- elif database == "mysql" %}
  - MySQL (8.0 or later) - [Install MySQL](https://dev.mysql.com/downloads/)
  {%- elif database == "none" %}
  - No database required for this configuration
  {%- endif %}
- **Node.js** (16 or later) - [Install Node.js](https://nodejs.org/) (for frontend assets)
- **Git** - [Install Git](https://git-scm.com/downloads)

## Installation

### 1. Clone the Repository

```bash
git clone <your-repository-url> {{project-name}}
cd {{project-name}}
```

### 2. Environment Setup

Copy the example environment file and configure it:

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```env
# Server Configuration
HOST=127.0.0.1
PORT={{port}}
ENVIRONMENT=development

{%- if database != "none" %}
# Database Configuration
{%- if database == "postgres" %}
DATABASE_URL=postgres://username:password@localhost/{{project_name_snake_case}}_dev
{%- elif database == "sqlite" %}
DATABASE_URL=sqlite:{{project_name_snake_case}}.db
{%- elif database == "mysql" %}
DATABASE_URL=mysql://username:password@localhost/{{project_name_snake_case}}_dev
{%- endif %}
{%- endif %}

# Logging
RUST_LOG={{crate_name}}=debug,tower_http=debug

{%- if hot_reload %}
# Hot Reload (development only)
HOT_RELOAD_ENABLED=true
{%- endif %}
```

{%- if database != "none" %}
### 3. Database Setup

{%- if database == "postgres" %}
Create a PostgreSQL database:

```bash
# Using psql
createdb {{project_name_snake_case}}_dev

# Or using SQL
psql -c "CREATE DATABASE {{project_name_snake_case}}_dev;"
```
{%- elif database == "mysql" %}
Create a MySQL database:

```bash
# Using mysql command line
mysql -u root -p -e "CREATE DATABASE {{project_name_snake_case}}_dev;"
```
{%- elif database == "sqlite" %}
SQLite databases are created automatically when the application runs.
{%- endif %}

Run database migrations:

```bash
cargo run --bin {{project-name}}-server -- migrate
```

Or if you have sqlx-cli installed:

```bash
sqlx migrate run
```
{%- endif %}

### 4. Install Dependencies

Install Rust dependencies:

```bash
cargo build
```

{%- if project_type == "full-stack" or project_type == "liveview-app" %}
Install frontend dependencies:

```bash
cd frontend
npm install
cd ..
```
{%- endif %}

### 5. Run the Application

Start the development server:

```bash
cargo run --bin {{project-name}}-server
```

{%- if hot_reload %}
With hot reload enabled, the server will automatically restart when you make changes to your Rust code.
{%- endif %}

The application will be available at:
- **Web Interface**: http://localhost:{{port}}
- **Health Check**: http://localhost:{{port}}/health
{%- if use_liveview %}
- **LiveView Demo**: http://localhost:{{port}}/live
{%- endif %}

## Development Workflow

### 1. Running Tests

Run the test suite:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests
cargo test --test integration_tests
```

### 2. Code Formatting

Format your code:

```bash
cargo fmt
```

### 3. Linting

Run Clippy for additional linting:

```bash
cargo clippy
```

### 4. Database Operations

{%- if database != "none" %}
Create a new migration:

```bash
sqlx migrate add migration_name
```

Run migrations:

```bash
sqlx migrate run
```

Revert last migration:

```bash
sqlx migrate revert
```
{%- endif %}

## Project Structure

```
{{project-name}}/
├── Cargo.toml                   # Workspace configuration
├── .env                         # Environment variables
├── .env.example                 # Environment template
├── README.md                    # Project overview
│
├── {{project-name}}-server/     # Main web server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # Application entry point
│       └── controllers/        # HTTP request handlers
│           ├── mod.rs
│           ├── health.rs       # Health check endpoints
│           └── users.rs        # User management
│
├── {{project-name}}-shared/     # Shared types and utilities
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs              # Common types and functions
│
{%- if use_liveview %}
├── {{project-name}}-liveview/   # LiveView components
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # LiveView exports
│       ├── components/         # Reusable components
│       ├── pages/              # Page components
│       └── router.rs           # LiveView routing
│
{%- endif %}
├── docs/                        # Documentation
├── migrations/                  # Database migrations
├── static/                      # Static assets (CSS, JS, images)
├── tests/                       # Integration tests
└── examples/                    # Code examples
```

## Configuration

The application uses environment-based configuration. Key settings include:

- `HOST` - Server bind address (default: 127.0.0.1)
- `PORT` - Server port (default: {{port}})
- `ENVIRONMENT` - Runtime environment (development/production)
{%- if database != "none" %}
- `DATABASE_URL` - Database connection string
{%- endif %}
- `RUST_LOG` - Logging configuration

For detailed configuration options, see the [Configuration Guide](./configuration.md).

## Next Steps

Now that you have {{project-name}} running:

1. **Explore the API** - Check out the [API Reference](./api-reference.md)
{%- if use_liveview %}
2. **Build LiveView Components** - Read the [LiveView Guide](./liveview.md)
{%- endif %}
3. **Understand the Architecture** - See the [Architecture Overview](./architecture.md)
4. **Deploy to Production** - Follow the [Deployment Guide](./deployment.md)

## Troubleshooting

If you encounter issues:

1. Check that all prerequisites are installed
2. Verify your environment configuration in `.env`
{%- if database != "none" %}
3. Ensure the database is running and accessible
4. Try recreating the database and running migrations again
{%- endif %}
5. Check the logs for error messages
6. Consult the [Troubleshooting Guide](./troubleshooting.md)

## Getting Help

- Read the [documentation](./README.md)
- Check [existing issues](https://github.com/your-org/{{project-name}}/issues)
- Create a [new issue](https://github.com/your-org/{{project-name}}/issues/new) if needed

Welcome to {{project-name}}! 🚀