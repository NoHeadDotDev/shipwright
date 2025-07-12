# {{project-name}}

{{description}}

## Overview

This project was generated using the Shipwright framework template. It provides a robust foundation for building web applications with Rust.

## Architecture

This workspace contains the following crates:

{% if use-config -%}
- **{{crate-name}}-config**: Configuration management and settings
{% endif -%}
{% if use-database -%}
- **{{crate-name}}-db**: Database layer and data access
{% endif -%}
{% if use-web -%}
- **{{crate-name}}-web**: Web server and HTTP handlers
{% endif -%}

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Cargo
{% if use-database -%}
{% if database-type == "postgres" -%}
- PostgreSQL server
{% elsif database-type == "mysql" -%}
- MySQL server
{% elsif database-type == "sqlite" -%}
- SQLite (included with Rust)
{% endif -%}
{% endif -%}

### Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd {{project-name}}
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

{% if use-database -%}
4. Set up the database:
   ```bash
   # Create database and run migrations
   cargo run --bin setup-db
   ```
{% endif -%}

{% if use-web -%}
5. Start the web server:
   ```bash
   cargo run --bin {{crate-name}}-web
   ```
{% endif -%}

## Configuration

{% if use-config -%}
Configuration is managed through the `{{crate-name}}-config` crate. See the configuration documentation for available options.
{% else -%}
Configuration is handled through environment variables and command-line arguments.
{% endif -%}

## Development

### Running in Development Mode

```bash
cargo run
```

### Building for Production

```bash
cargo build --release
```

## License

This project is licensed under the {{license}} license.

## Generated with Shipwright

This project was generated using the [Shipwright](https://github.com/your-org/shipwright) framework template.