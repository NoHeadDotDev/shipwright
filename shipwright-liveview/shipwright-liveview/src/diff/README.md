# DOM Diffing Module

This module implements a morphdom-like algorithm for efficient DOM diffing in Rust. It works directly with HTML strings/fragments to generate minimal update instructions that can be applied to the DOM.

## Features

- **Morphdom Algorithm**: A Rust port of the popular morphdom algorithm for efficient DOM updates
- **HTML String Processing**: Works directly with HTML strings without requiring a full DOM implementation
- **Minimal Updates**: Generates only the necessary operations to transform one HTML tree to another
- **Optimizations for Common Patterns**:
  - List updates (add/remove/reorder) with key-based diffing
  - Attribute toggles and batch updates
  - Text content changes
  - Component boundary tracking
- **Efficient Patch Representation**:
  - Compact binary format for wire transfer
  - Streaming support
  - Compression-friendly structure

## Usage

```rust
use shipwright_liveview::diff::{diff_html, DiffOptions};

// Diff two HTML strings
let from = "<div>Hello World</div>";
let to = "<div>Hello Rust</div>";

let options = DiffOptions::default();
let patch = diff_html(from, to, &options).unwrap();

// The patch contains operations to transform 'from' to 'to'
for op in &patch.ops {
    println!("Operation: {:?}", op);
}
```

## Key-based List Diffing

For efficient list updates, enable key-based diffing:

```rust
let mut options = DiffOptions::default();
options.use_keys = true;

let from = r#"
    <ul>
        <li key="a">Item A</li>
        <li key="b">Item B</li>
    </ul>
"#;

let to = r#"
    <ul>
        <li key="b">Item B</li>
        <li key="a">Item A</li>
    </ul>
"#;

let patch = diff_html(from, to, &options).unwrap();
// Will generate efficient move operations instead of replace operations
```

## Binary Serialization

Patches can be serialized to a compact binary format:

```rust
// Serialize
let binary = patch.to_binary().unwrap();

// Deserialize
let restored = Patch::from_binary(&binary).unwrap();
```

## Architecture

The module is organized into several components:

- **`morphdom.rs`**: Core diffing algorithm implementation
- **`parser.rs`**: HTML parser for converting strings to tree structures
- **`patch.rs`**: Patch representation and serialization
- **`optimizer.rs`**: Advanced optimization strategies for patches

## Performance Considerations

1. **Key-based Diffing**: Use keys for lists that frequently change order
2. **Component Boundaries**: Mark component boundaries with data attributes for better optimization
3. **Batch Operations**: The optimizer will automatically batch similar operations
4. **Binary Format**: Use binary serialization for network transfer to reduce payload size

## Future Enhancements

- Support for Shadow DOM boundaries
- Custom element handling
- Incremental parsing for large documents
- WebAssembly bindings for client-side usage