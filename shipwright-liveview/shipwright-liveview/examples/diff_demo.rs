//! Example demonstrating the DOM diffing module

use shipwright_liveview::diff::{diff_html, DiffOptions};

fn main() {
    // Example 1: Simple text change
    println!("=== Example 1: Simple text change ===");
    let from = "<div>Hello World</div>";
    let to = "<div>Hello Rust</div>";
    
    let options = DiffOptions::default();
    match diff_html(from, to, &options) {
        Ok(patch) => {
            println!("From: {}", from);
            println!("To: {}", to);
            println!("Patch operations: {:?}", patch.ops);
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    
    // Example 2: Attribute changes
    println!("=== Example 2: Attribute changes ===");
    let from = r#"<div class="old" id="test">Content</div>"#;
    let to = r#"<div class="new" id="test" data-value="42">Content</div>"#;
    
    match diff_html(from, to, &options) {
        Ok(patch) => {
            println!("From: {}", from);
            println!("To: {}", to);
            println!("Patch operations: {:?}", patch.ops);
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    
    // Example 3: List updates
    println!("=== Example 3: List updates ===");
    let from = r#"
        <ul>
            <li>Item 1</li>
            <li>Item 2</li>
        </ul>
    "#;
    let to = r#"
        <ul>
            <li>Item 1</li>
            <li>Item 2</li>
            <li>Item 3</li>
        </ul>
    "#;
    
    match diff_html(from, to, &options) {
        Ok(patch) => {
            println!("From: {}", from.trim());
            println!("To: {}", to.trim());
            println!("Patch operations: {:?}", patch.ops);
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    
    // Example 4: Keyed list reordering
    println!("=== Example 4: Keyed list reordering ===");
    let from = r#"
        <ul>
            <li key="a">Item A</li>
            <li key="b">Item B</li>
            <li key="c">Item C</li>
        </ul>
    "#;
    let to = r#"
        <ul>
            <li key="c">Item C</li>
            <li key="a">Item A</li>
            <li key="b">Item B</li>
        </ul>
    "#;
    
    let mut keyed_options = DiffOptions::default();
    keyed_options.use_keys = true;
    
    match diff_html(from, to, &keyed_options) {
        Ok(patch) => {
            println!("From: {}", from.trim());
            println!("To: {}", to.trim());
            println!("Patch operations: {:?}", patch.ops);
            println!();
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    
    // Example 5: Binary serialization
    println!("=== Example 5: Binary serialization ===");
    let from = "<div>Original</div>";
    let to = "<div class='updated'>Modified</div>";
    
    match diff_html(from, to, &options) {
        Ok(patch) => {
            // Serialize to binary
            match patch.to_binary() {
                Ok(binary) => {
                    println!("Original patch: {:?}", patch.ops);
                    println!("Binary size: {} bytes", binary.len());
                    
                    // Deserialize back
                    match shipwright_liveview::diff::patch::Patch::from_binary(&binary) {
                        Ok(restored) => {
                            println!("Restored patch: {:?}", restored.ops);
                            println!("Serialization successful!");
                        }
                        Err(e) => eprintln!("Deserialization error: {}", e),
                    }
                }
                Err(e) => eprintln!("Serialization error: {}", e),
            }
        }
        Err(e) => eprintln!("Diff error: {}", e),
    }
}