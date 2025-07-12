# Shipwright LiveView Macros

This crate provides the `view!` macro for creating reactive HTML templates with RSX syntax in the Shipwright LiveView framework.

## Features

- **RSX Syntax**: Write HTML-like templates with embedded Rust expressions
- **Components**: First-class support for reusable components
- **Event Handlers**: Type-safe event handler registration
- **Control Flow**: Built-in support for conditionals and loops
- **Dynamic Attributes**: Reactive attribute and class bindings
- **Compile-Time Optimization**: Separates static and dynamic parts at compile time
- **Location Tracking**: Tracks template locations using `file!()`, `line!()`, `column!()`

## Usage

```rust
use shipwright_liveview_macros::view;

let my_view = view! {
    <div class="container">
        <h1>{"Hello, World!"}</h1>
        <button on:click={handle_click}>{"Click me"}</button>
        {if show_content {
            <p>{"Dynamic content here"}</p>
        }}
    </div>
};
```

## Syntax Guide

### Basic HTML Elements

```rust
view! {
    <div class="my-class" id="my-id">
        <p>{"Static text"}</p>
        <span>{dynamic_value}</span>
    </div>
}
```

### Components

Components are identified by uppercase first letters:

```rust
view! {
    <MyComponent prop={value} another_prop={42} />
    <Button label={"Click"} on_click={handler} />
}
```

### Event Handlers

Use the `on:` prefix for events:

```rust
view! {
    <button on:click={handle_click}>{"Click"}</button>
    <input on:change={handle_input} on:keydown={handle_key} />
}
```

### Conditional Rendering

```rust
view! {
    {if condition {
        <div>{"True branch"}</div>
    } else {
        <div>{"False branch"}</div>
    }}
}
```

### Loops

```rust
view! {
    <ul>
        {for item in items {
            <li>{item.name}</li>
        }}
    </ul>
}
```

### Dynamic Attributes

```rust
view! {
    // Dynamic attribute value
    <input value={current_value} />
    
    // Conditional classes
    <div class:active={is_active} class:disabled={!enabled}>
        {"Content"}
    </div>
}
```

## How It Works

The `view!` macro performs the following at compile time:

1. **Parsing**: Converts RSX syntax into an AST
2. **Analysis**: Separates static HTML from dynamic parts
3. **Code Generation**: 
   - Generates static HTML template strings
   - Creates update functions for dynamic parts
   - Registers event handlers with unique IDs
   - Tracks template locations for hot reloading

The output is a `View` struct containing:
- `template_id`: Unique identifier based on file location
- `static_template`: Pre-rendered HTML with placeholders
- `update_fn`: Function to update dynamic parts
- `event_handlers`: List of registered event handlers

## Implementation Details

The macro uses several optimization techniques:

- **Static Extraction**: HTML that doesn't change is rendered once
- **Targeted Updates**: Only dynamic parts trigger DOM updates
- **Event Delegation**: Events are handled efficiently through delegation
- **Template Caching**: Templates are identified by source location

## Error Messages

The macro provides helpful error messages for common mistakes:

```rust
// Error: Mismatched closing tag
view! {
    <div>
        <span>{"Content"}</div>  // Error: expected </span>
    </span>
}

// Error: Invalid attribute syntax
view! {
    <div class={dynamic class}>  // Error: expected string literal or {expression}
}
```

## Performance Considerations

- Static parts are extracted at compile time
- Dynamic expressions are evaluated only when needed
- Event handlers are registered once during initialization
- Minimal runtime overhead for template rendering

## Future Enhancements

Planned features for future versions:

- Fragment support (`<>...</>`)
- Spread attributes (`{...props}`)
- Slot/children support for components
- Better error recovery and diagnostics
- Integration with IDE tooling