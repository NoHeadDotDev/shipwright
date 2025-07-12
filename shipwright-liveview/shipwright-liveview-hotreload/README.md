# Shipwright LiveView Hot Reload

Development-time hot reload infrastructure for Shipwright LiveView templates.

## Features

- **File Watching**: Monitors `.rs` files for changes to `view!` or `html!` macros
- **Template Parsing**: Extracts and analyzes templates from Rust source files
- **WebSocket Server**: Serves hot reload updates to connected clients
- **Template Caching**: Efficient caching system with versioning
- **CLI Tool**: Easy-to-use command-line interface

## Installation

Add to your `Cargo.toml` during development:

```toml
[dev-dependencies]
shipwright-liveview-hotreload = { path = "../path/to/shipwright-liveview-hotreload" }
```

## Usage

### Starting the Hot Reload Server

```bash
# Start with default settings (port 3001, watch current directory)
shipwright-hotreload start

# Custom configuration
shipwright-hotreload start --port 3002 --watch src --watch templates --log-level debug

# Show help
shipwright-hotreload --help
```

### Integration with Your Application

#### Rust Side

In your application, connect to the hot reload server in development mode:

```rust
#[cfg(debug_assertions)]
use shipwright_liveview_hotreload::runtime::init_hot_reload;

#[tokio::main]
async fn main() {
    // Initialize hot reload in development
    #[cfg(debug_assertions)]
    init_hot_reload("ws://localhost:3001/ws".to_string()).await.ok();
    
    // Rest of your application...
}
```

#### JavaScript Side

Include the hot reload client in your frontend:

```javascript
import { initHotReload } from 'shipwright-liveview-hotreload/client/hot-reload-client.js';

// Initialize hot reload in development
if (process.env.NODE_ENV === 'development') {
  const client = initHotReload('ws://localhost:3001/ws');
  
  // Handle template updates
  client.onUpdate((update) => {
    console.log('Template updated:', update);
    // Your update handling logic here
  });
}
```

## Architecture

### Components

1. **File Watcher**: Uses `notify` to detect changes in Rust files
2. **Template Parser**: Extracts templates using `syn` for AST parsing
3. **WebSocket Server**: Built with Axum for real-time updates
4. **Template Cache**: In-memory cache with TTL and versioning
5. **Runtime Integration**: Allows templates to register themselves

### Protocol

The hot reload protocol uses JSON messages over WebSocket:

```json
// Connected message
{
  "type": "connected",
  "version": "1.0.0"
}

// Template update
{
  "type": "template_updated",
  "id": {
    "file": "/path/to/file.rs",
    "line": 42,
    "column": 8
  },
  "hash": "abc123...",
  "html": "<div>Updated template</div>",
  "dynamic_parts": [...]
}
```

### Template Identification

Templates are uniquely identified by:
- File path
- Line number
- Column number

This allows precise updates without full page reloads.

## Development

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

### Example

See the `examples/` directory for integration examples.

## Performance

- Debounced file watching (100ms)
- Efficient template caching
- Minimal WebSocket messages
- Automatic cache cleanup

## Limitations

- Development only - not for production use
- Requires debug assertions enabled
- Only supports Rust files with `.rs` extension
- Template syntax must be valid for parsing

## Future Enhancements

- [ ] Support for external template files
- [ ] Template validation
- [ ] Browser extension for better integration
- [ ] Hot reload for CSS/assets
- [ ] Template error overlay