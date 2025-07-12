# Shipwright LiveView Hot Reload - Runtime Integration

This document describes the complete runtime integration system for Shipwright LiveView hot reload, which enables seamless development experience with live template updates while preserving application state.

## Overview

The runtime integration consists of several interconnected systems that work together to provide a smooth hot reload experience:

1. **Enhanced Template Registry** - Tracks and manages LiveView templates at runtime
2. **DOM Diffing Engine** - Generates minimal DOM updates using morphdom-style algorithms
3. **Component Re-rendering System** - Updates LiveView components while preserving state
4. **LiveView Integration Layer** - Bridges hot reload with the main LiveView framework
5. **Error Recovery System** - Handles failures gracefully with multiple fallback strategies
6. **Comprehensive Logging** - Provides detailed debugging and performance insights
7. **JavaScript Client** - Applies DOM patches efficiently in the browser
8. **State Serialization** - Preserves and restores component state across updates

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  File Watcher   │───▶│  Template Parser │───▶│ Template Cache  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   JS Client     │◀───│  WebSocket       │◀───│ Hot Reload      │
│   (Browser)     │    │  Server          │    │ Server          │
└─────────────────┘    └──────────────────┘    └─────────────────┘
        │                                                │
        ▼                                                ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  DOM Patcher    │    │  State Manager   │    │ Template        │
│                 │    │                  │    │ Registry        │
└─────────────────┘    └──────────────────┘    └─────────────────┘
        │                        │                      │
        ▼                        ▼                      ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  LiveView       │    │  Error Recovery  │    │ Logging System  │
│  Components     │    │  System          │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Core Components

### 1. Enhanced Template Registry (`runtime.rs`)

The template registry provides live tracking of templates and component instances:

```rust
use shipwright_liveview_hotreload::runtime::{TemplateRegistry, RegisteredTemplate, ComponentInstance};

// Register a template
let template = RegisteredTemplate {
    id: template_id,
    component_type: "MyComponent".to_string(),
    function_name: "render".to_string(),
    content_hash: None,
    current_html: None,
    dynamic_parts: vec![],
    preserved_state: None,
};

let registry = TemplateRegistry::global();
let mut registry_guard = registry.write().await;
registry_guard.register(template);
```

Key features:
- Global singleton for template tracking
- Component instance management with state preservation
- Hot reload update handling with error recovery
- Integration with DOM diffing system

### 2. DOM Diffing Engine (`dom_diff.rs`)

Advanced DOM diffing system that generates minimal update operations:

```rust
use shipwright_liveview_hotreload::dom_diff::{HotReloadDomDiffer, HotReloadDiffOptions};

let mut differ = HotReloadDomDiffer::new();

// Register component boundary for targeted updates
let boundary = ComponentBoundary {
    component_id: "my-component".to_string(),
    root_selector: "#my-component".to_string(),
    isolated: true,
    children: vec![],
    state_data: None,
};
differ.register_component_boundary(boundary);

// Generate optimized patch
let patch = differ.generate_hot_reload_patch(
    old_html,
    new_html,
    Some("my-component")
)?;
```

Features:
- Morphdom-compatible diffing algorithm
- Targeted component updates
- State preservation during DOM changes
- Efficient patch generation with caching

### 3. LiveView Integration (`liveview_integration.rs`)

Seamless integration with the shipwright-liveview framework:

```rust
use shipwright_liveview_hotreload::liveview_integration::{
    LiveViewHotReloadManager, ActiveLiveView, LiveViewHotReloadConfig
};

// Initialize the manager
let config = LiveViewHotReloadConfig::default();
let manager = LiveViewHotReloadManager::new(config);

// Register a LiveView instance
let view = ActiveLiveView::new(
    "instance-1".to_string(),
    template_id,
    initial_html,
    "MyComponent".to_string(),
);
manager.register_liveview_instance(view).await?;

// Update component when template changes
let results = manager.update_liveview_component(&template_update).await?;
```

Features:
- LiveView component tracking and management
- State preservation across updates
- Event-driven update system
- Integration with error recovery

### 4. Main Integration System (`integration.rs`)

The main orchestrator that connects all systems:

```rust
use shipwright_liveview_hotreload::integration::{
    HotReloadIntegration, HotReloadIntegrationConfig
};

// Initialize the complete system
let config = HotReloadIntegrationConfig::default();
let integration = HotReloadIntegration::new(config).await?;

// Start the system
integration.start().await?;

// Register LiveView components
integration.register_liveview(
    "instance-1".to_string(),
    template_id,
    initial_html,
    "MyComponent".to_string(),
).await?;
```

Features:
- Complete system orchestration
- WebSocket handling for client communication
- Axum integration helpers
- Comprehensive statistics and monitoring

### 5. Error Recovery (`error_recovery.rs`)

Robust error handling with multiple recovery strategies:

```rust
use shipwright_liveview_hotreload::error_recovery::{
    ErrorRecoverySystem, ErrorBoundary, RecoveryStrategy
};

// Create error recovery system
let config = ErrorRecoveryConfig::default();
let recovery_system = ErrorRecoverySystem::new(config);

// Register recovery strategies
recovery_system.add_recovery_strategy(Arc::new(
    RetryStrategy::new(3, Duration::from_millis(500))
)).await;

// Handle errors
let outcome = recovery_system.handle_error(error, context).await;
```

Features:
- Circuit breaker pattern for cascading failures
- Multiple recovery strategies (retry, fallback, refresh)
- Error boundaries for component isolation
- Comprehensive error tracking and statistics

### 6. Logging and Debugging (`logging.rs`)

Advanced logging system with real-time debugging:

```rust
use shipwright_liveview_hotreload::logging::{
    HotReloadLogger, LoggingConfig, LogEventType, LogLevel
};

// Initialize logging
let config = LoggingConfig::default();
let logger = HotReloadLogger::new(config);
logger.init_logging()?;

// Log events
let event_id = logger.log_event(
    LogEventType::ComponentUpdate,
    LogLevel::Info,
    "Component updated successfully".to_string(),
    Some(context_data),
    Some(template_id),
    Some(duration),
).await;

// Debug commands
let result = logger.handle_debug_command(DebugCommand::StartSession {
    templates: vec![template_id],
    components: vec!["MyComponent".to_string()],
}).await?;
```

Features:
- Structured logging with event types and metadata
- Performance metrics tracking
- Interactive debugging with breakpoints
- Real-time event streaming
- Multiple output formats (JSON, text, pretty)

### 7. JavaScript Client Integration

Enhanced browser-side DOM patching:

```javascript
import { HotReloadClient, initHotReload } from './hot-reload-client.js';

// Initialize hot reload
const client = initHotReload('ws://localhost:3001/ws', {
    toastEnabled: true,
    showIndicator: true,
    enableDebugShortcuts: true
});

// The client automatically handles:
// - Structured DOM patches
// - State preservation and restoration
// - Error recovery and fallbacks
// - User feedback via toast notifications
```

Features:
- Structured DOM patch application
- Advanced state preservation (forms, scroll, focus)
- Visual feedback system with toast notifications
- Debug shortcuts and visual indicators
- Automatic reconnection with exponential backoff

### 8. State Serialization (`state_serialization.rs`)

Comprehensive state management across hot reloads:

```rust
use shipwright_liveview_hotreload::state_serialization::{
    StateManager, JsonStateSerializer, StateManagerConfig
};

// Initialize state manager
let config = StateManagerConfig::default();
let state_manager = StateManager::new(config);

// Register serializers
state_manager.register_serializer::<MyComponentState>(
    Box::new(JsonStateSerializer::<MyComponentState>::new())
).await;

// Preserve state
state_manager.preserve_state(
    "instance-1".to_string(),
    template_id,
    "MyComponent".to_string(),
    &component_state,
).await?;

// Restore state
let restored_state: Option<MyComponentState> = 
    state_manager.restore_state("instance-1").await?;
```

Features:
- Type-safe state serialization
- Multiple serialization formats (JSON, MessagePack)
- Compression and integrity validation
- Automatic cleanup and memory management
- Built-in serializers for common types

## Usage Guide

### Basic Setup

1. **Add to your Cargo.toml:**
```toml
[dependencies]
shipwright-liveview-hotreload = { path = "path/to/hot-reload-crate" }
```

2. **Initialize in your main application:**
```rust
use shipwright_liveview_hotreload::integration::{HotReloadIntegration, HotReloadIntegrationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize hot reload in development
    #[cfg(debug_assertions)]
    {
        let config = HotReloadIntegrationConfig {
            server_port: 3001,
            watch_directory: "src".to_string(),
            ..Default::default()
        };
        
        let integration = HotReloadIntegration::new(config).await?;
        
        // Start in background
        tokio::spawn(async move {
            if let Err(e) = integration.start().await {
                eprintln!("Hot reload error: {}", e);
            }
        });
    }
    
    // Your normal application setup
    let app = Router::new()
        .route("/", get(index));
    
    // Add hot reload routes in development
    #[cfg(debug_assertions)]
    let app = shipwright_liveview_hotreload::enable_hot_reload!(app, config);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

3. **Include the JavaScript client in your HTML:**
```html
<!DOCTYPE html>
<html>
<head>
    <title>My App</title>
</head>
<body>
    <!-- Your LiveView content -->
    
    <!-- Hot reload script (development only) -->
    <script>
        if (window.location.hostname === 'localhost') {
            import('./hot-reload-client.js').then(module => {
                module.initHotReload('ws://localhost:3001/ws');
            });
        }
    </script>
</body>
</html>
```

### LiveView Component Integration

```rust
use shipwright_liveview::{Html, LiveView, html};
use shipwright_liveview_hotreload::{hot_reload_liveview, preserve_component_state};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CounterState {
    count: i32,
    message: String,
}

impl LiveView for Counter {
    type Message = CounterMsg;
    
    fn mount(&mut self, _uri: Uri, _headers: &HeaderMap, handle: ViewHandle<Self::Message>) {
        // Register for hot reload
        let template_id = TemplateId::new(
            std::path::PathBuf::from(file!()),
            line!() as u32,
            column!() as u32,
        );
        
        hot_reload_liveview!(
            "counter-instance",
            template_id,
            self.render().to_string(),
            "Counter"
        );
    }
    
    fn render(&self) -> Html<Self::Message> {
        html! {
            <div data-live-view-id="counter-instance" data-template-id="counter">
                <h1>"Counter: " {self.count}</h1>
                <button axm-click={CounterMsg::Increment}>"+"</button>
                <button axm-click={CounterMsg::Decrement}>"-"</button>
                <p>{&self.message}</p>
            </div>
        }
    }
    
    // State is automatically preserved and restored during hot reload
}
```

## Development Features

### Debug Shortcuts

When hot reload is active, several keyboard shortcuts are available:

- **Ctrl+Shift+R**: Force reconnect to hot reload server
- **Ctrl+Shift+H**: Show hot reload statistics in console
- **Ctrl+Shift+T**: Toggle toast notifications

### Visual Indicators

The system provides visual feedback:

- **Connection indicator**: Shows hot reload status in bottom-right corner
- **Toast notifications**: Non-intrusive updates about hot reload operations
- **Console logging**: Detailed information about updates and errors

### Performance Monitoring

Built-in performance tracking includes:

- Update latency measurement
- DOM patch size tracking
- State preservation timing
- Error rate monitoring
- Component update frequency

## Configuration Options

### Server Configuration
```rust
HotReloadIntegrationConfig {
    server_host: "localhost".to_string(),
    server_port: 3001,
    watch_directory: "src".to_string(),
    watch_patterns: vec!["**/*.rs".to_string()],
    debug_mode: true,
    liveview_config: LiveViewHotReloadConfig {
        preserve_state: true,
        preserve_form_state: true,
        preserve_scroll: true,
        preserve_focus: true,
        state_preservation_timeout: 1000,
    },
}
```

### Client Configuration
```javascript
initHotReload('ws://localhost:3001/ws', {
    toastEnabled: true,
    showIndicator: true,
    enableDebugShortcuts: true
});
```

## Error Handling

The system provides multiple layers of error handling:

1. **Graceful degradation**: If hot reload fails, falls back to full page refresh
2. **Error boundaries**: Component-level isolation prevents cascading failures
3. **Circuit breakers**: Automatic protection against repeated failures
4. **Recovery strategies**: Multiple retry and fallback mechanisms
5. **User feedback**: Clear error messages and recovery suggestions

## Performance Considerations

- **Minimal DOM updates**: Only changed elements are updated
- **State preservation**: Efficient serialization with compression
- **Caching**: Template and diff caching for improved performance
- **Batching**: Multiple updates are batched for efficiency
- **Memory management**: Automatic cleanup of old states and cached data

## Troubleshooting

### Common Issues

1. **Hot reload not connecting**: Check that the server is running on the correct port
2. **State not preserved**: Ensure your component state implements `Serialize` and `Deserialize`
3. **Updates not applying**: Verify that templates have the correct data attributes
4. **Performance issues**: Check the debug console for timing information

### Debug Information

Use the debug shortcuts or check the browser console for detailed information about:
- Connection status
- Update operations
- Error messages
- Performance metrics
- State preservation details

## Future Enhancements

Planned improvements include:

- **CSS hot reload**: Live updating of stylesheets
- **Asset hot reload**: Live updating of images and other assets
- **Component tree visualization**: Debug view of component hierarchy
- **Time-travel debugging**: Ability to step through state changes
- **Performance profiler**: Detailed performance analysis tools
- **Custom recovery strategies**: User-defined error recovery logic

## Contributing

To contribute to the hot reload system:

1. **Add tests**: All new features should include comprehensive tests
2. **Update documentation**: Keep this README and code comments up to date
3. **Performance testing**: Ensure changes don't degrade performance
4. **Error handling**: Include proper error handling and recovery
5. **Logging**: Add appropriate logging for debugging

The hot reload system is designed to be extensible and maintainable, with clear separation of concerns and comprehensive error handling.