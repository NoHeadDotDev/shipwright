# Shipwright CLI

The official command-line interface for the Shipwright enhanced hot reload system.

## Features

- **Development Server**: Hot reload with near-instant template updates
- **Production Server**: Optimized static file serving
- **Build System**: Comprehensive build pipeline with asset processing
- **Workspace Support**: Multi-crate project detection and handling
- **Configuration**: Flexible TOML-based configuration system

## Installation

From source:
```bash
cargo install --path .
```

## Usage

### Development Mode

Start a development server with hot reload:

```bash
# Start on default port (8080)
shipwright dev

# Custom port and host
shipwright dev --port 3000 --host 0.0.0.0

# Disable hot reload
shipwright dev --no-hot-reload

# Enable specific features
shipwright dev --features "ssr,hydration"

# Open browser automatically
shipwright dev --open
```

### Production Mode

Start a production server:

```bash
# Start production server
shipwright serve

# Custom configuration
shipwright serve --port 8080 --static-dir dist

# Enable compression and CORS
shipwright serve --gzip --cors
```

### Building

Build your application for production:

```bash
# Debug build
shipwright build

# Release build with optimizations
shipwright build --release

# Specific target (e.g., WebAssembly)
shipwright build --target wasm32-unknown-unknown

# Clean before building
shipwright build --clean --release

# Enable features
shipwright build --features "ssr,csr"
```

## Configuration

Shipwright uses a `Shipwright.toml` configuration file. See `Shipwright.toml.example` for a complete example.

### Basic Configuration

```toml
[application]
name = "my-app"
version = "0.1.0"
default_platform = "web"

[serve]
host = "localhost"
port = 8080

[hot_reload]
enabled = true
watch_paths = ["src", "assets"]
ignore_paths = ["target", "dist"]
```

### Workspace Configuration

For multi-crate projects:

```toml
[workspace]
members = ["app", "shared"]
exclude = ["examples"]
default_members = ["app"]
```

## Command Reference

### Global Options

- `--log-level <LEVEL>`: Set logging level (trace, debug, info, warn, error)
- `--config <PATH>`: Path to Shipwright.toml config file
- `--cwd <PATH>`: Working directory

### Dev Command

Options:
- `--host, -H <HOST>`: Host to bind to
- `--port, -p <PORT>`: Port to bind to
- `--no-hot-reload`: Disable hot reload
- `--release`: Use release mode
- `--features <FEATURES>`: Comma-separated list of features
- `--open`: Open browser automatically
- `--package <PACKAGE>`: Target package (for workspaces)

### Serve Command

Options:
- `--host, -H <HOST>`: Host to bind to
- `--port, -p <PORT>`: Port to bind to
- `--static-dir <DIR>`: Static files directory
- `--release`: Ensure release build
- `--cors`: Enable CORS headers
- `--gzip`: Enable gzip compression
- `--package <PACKAGE>`: Target package (for workspaces)

### Build Command

Options:
- `--release`: Enable release mode optimizations
- `--target <TARGET>`: Build target (e.g., wasm32-unknown-unknown)
- `--features <FEATURES>`: Comma-separated list of features
- `--out-dir <DIR>`: Output directory
- `--target-dir <DIR>`: Cargo target directory
- `--package <PACKAGE>`: Target package (for workspaces)
- `--clean`: Clean before building
- `--verbose, -v`: Verbose output

## Hot Reload System

The hot reload system watches for file changes and automatically:

1. Rebuilds the Rust code when `.rs` files change
2. Processes assets when asset files change
3. Updates the browser without full page reload (when possible)
4. Provides real-time build feedback

### Watched Files

By default, these file types trigger rebuilds:
- `.rs` (Rust source files)
- `.toml` (Configuration files)
- `.html` (Template files)
- `.css` (Stylesheets)
- `.js` (JavaScript files)

### Configuration

```toml
[hot_reload]
enabled = true
watch_paths = ["src", "assets", "public"]
ignore_paths = ["target", "dist", ".git"]
debounce_ms = 300
poll_interval = 1000
reload_css = true
reload_js = true
```

## Workspace Support

Shipwright automatically detects Cargo workspaces and can:

- Build specific packages within a workspace
- Watch all workspace members for changes
- Handle dependency graphs between workspace members
- Provide build environment variables

### Workspace Detection

The CLI automatically detects:
- Single-crate projects
- Multi-crate workspaces
- Target package based on current directory
- Dependency relationships between packages

## Environment Variables

The CLI sets these environment variables during builds:

- `SHIPWRIGHT_WORKSPACE_ROOT`: Path to workspace root
- `CARGO_TARGET_DIR`: Target directory for builds
- Custom variables from `[build.environment]` config section

## Examples

### Basic Web Application

```bash
# Initialize new project
cargo new my-app
cd my-app

# Create Shipwright.toml
cp Shipwright.toml.example Shipwright.toml

# Start development
shipwright dev --open
```

### Multi-crate Workspace

```bash
# Create workspace
mkdir my-workspace
cd my-workspace

# Create Cargo.toml for workspace
cat > Cargo.toml << EOF
[workspace]
members = ["app", "shared"]
EOF

# Create packages
cargo new app
cargo new shared --lib

# Configure Shipwright.toml for workspace
cat > Shipwright.toml << EOF
[application]
name = "my-workspace"

[workspace]
members = ["app", "shared"]
default_members = ["app"]
EOF

# Build specific package
shipwright build --package app --release
```

## Troubleshooting

### Common Issues

1. **Port already in use**
   ```bash
   shipwright dev --port 3001
   ```

2. **Build failures**
   ```bash
   shipwright build --verbose --clean
   ```

3. **Hot reload not working**
   - Check file permissions
   - Verify watch paths in config
   - Check ignore patterns

### Debug Mode

Enable verbose logging:
```bash
shipwright --log-level debug dev
```

## Contributing

See the main Shipwright repository for contribution guidelines.

## License

MIT OR Apache-2.0