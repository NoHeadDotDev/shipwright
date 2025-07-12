# Counter Example with Hot Reload

This example demonstrates the Shipwright LiveView counter with **hot reload** support.

## Features

- **Real-time updates**: Server-rendered HTML with WebSocket communication
- **Hot reload**: Template changes update instantly without restart (development only)
- **Type-safe**: End-to-end type safety from server to client
- **Modern UI**: Styled buttons with hover effects

## Running the Example

```bash
# Run in development mode (with hot reload)
cargo run

# Server will start on:
# - Main app: http://localhost:3000
# - Hot reload: ws://localhost:3001 (automatic)
```

## Testing Hot Reload

1. Start the server with `cargo run`
2. Open http://localhost:3000 in your browser
3. Open the developer console to see hot reload messages
4. Edit the template in `src/main.rs` (e.g., change button colors or text)
5. Save the file and watch the page update instantly!

## Hot Reload Features

- **Template tracking**: Monitors `html!` macro usage for changes
- **Instant updates**: Changes appear without server restart
- **Development only**: Hot reload is disabled in release builds
- **Browser sync**: Automatically reloads the page when templates change

## Example Changes to Try

In the `render()` function, try changing:

```rust
// Change button colors
style="background: #ff6b6b"  // Red -> try #4ecdc4 (teal)
style="background: #51cf66"  // Green -> try #ffa726 (orange)

// Change text
"🚀 Shipwright LiveView Counter"  // -> "⚡ Hot Reload Demo"

// Add new elements
<p>"Current time: " { chrono::Utc::now().format("%H:%M:%S") }</p>
```

Save the file and watch the changes appear instantly in your browser!

## How It Works

1. **File Watcher**: Monitors `.rs` files for changes containing `html!` macros
2. **Template Parser**: Extracts and analyzes template changes
3. **WebSocket Server**: Broadcasts updates to connected browsers on port 3001
4. **Client Script**: Receives updates and triggers page reload

The hot reload system is designed to be lightweight and only active during development.