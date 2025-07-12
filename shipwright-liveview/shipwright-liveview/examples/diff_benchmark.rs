//! Benchmark example for the DOM diffing module

use shipwright_liveview::diff::{diff_html, DiffOptions};
use shipwright_liveview::diff::optimizer::PatchOptimizer;
use std::time::Instant;

fn main() {
    println!("DOM Diffing Benchmark\n");
    
    // Benchmark 1: Simple text changes
    benchmark_text_changes();
    
    // Benchmark 2: Attribute changes
    benchmark_attribute_changes();
    
    // Benchmark 3: List operations
    benchmark_list_operations();
    
    // Benchmark 4: Complex document changes
    benchmark_complex_document();
    
    // Benchmark 5: Binary serialization
    benchmark_serialization();
}

fn benchmark_text_changes() {
    println!("=== Text Changes Benchmark ===");
    
    let from = generate_text_document(100);
    let to = generate_text_document_modified(100);
    
    let options = DiffOptions::default();
    let start = Instant::now();
    
    let patch = diff_html(&from, &to, &options).unwrap();
    let diff_time = start.elapsed();
    
    println!("Document size: {} chars", from.len());
    println!("Diff time: {:?}", diff_time);
    println!("Operations generated: {}", patch.ops.len());
    println!();
}

fn benchmark_attribute_changes() {
    println!("=== Attribute Changes Benchmark ===");
    
    let from = generate_attribute_document(50);
    let to = generate_attribute_document_modified(50);
    
    let options = DiffOptions::default();
    let start = Instant::now();
    
    let patch = diff_html(&from, &to, &options).unwrap();
    let diff_time = start.elapsed();
    
    let optimizer = PatchOptimizer::new();
    let optimized = optimizer.optimize(patch.clone());
    
    println!("Elements with attributes: 50");
    println!("Diff time: {:?}", diff_time);
    println!("Operations before optimization: {}", patch.ops.len());
    println!("Operations after optimization: {}", optimized.ops.len());
    println!();
}

fn benchmark_list_operations() {
    println!("=== List Operations Benchmark ===");
    
    // Without keys
    let from = generate_list(100, false);
    let to = generate_list_reordered(100, false);
    
    let options = DiffOptions::default();
    let start = Instant::now();
    let patch_no_keys = diff_html(&from, &to, &options).unwrap();
    let time_no_keys = start.elapsed();
    
    // With keys
    let from_keyed = generate_list(100, true);
    let to_keyed = generate_list_reordered(100, true);
    
    let mut keyed_options = DiffOptions::default();
    keyed_options.use_keys = true;
    
    let start = Instant::now();
    let patch_with_keys = diff_html(&from_keyed, &to_keyed, &keyed_options).unwrap();
    let time_with_keys = start.elapsed();
    
    println!("List items: 100");
    println!("Without keys - Time: {:?}, Operations: {}", time_no_keys, patch_no_keys.ops.len());
    println!("With keys - Time: {:?}, Operations: {}", time_with_keys, patch_with_keys.ops.len());
    println!();
}

fn benchmark_complex_document() {
    println!("=== Complex Document Benchmark ===");
    
    let from = generate_complex_document();
    let to = generate_complex_document_modified();
    
    let options = DiffOptions::default();
    let start = Instant::now();
    
    let patch = diff_html(&from, &to, &options).unwrap();
    let diff_time = start.elapsed();
    
    let optimizer = PatchOptimizer::new();
    let start = Instant::now();
    let optimized = optimizer.optimize(patch.clone());
    let optimize_time = start.elapsed();
    
    println!("Document size: {} chars", from.len());
    println!("Diff time: {:?}", diff_time);
    println!("Optimization time: {:?}", optimize_time);
    println!("Operations before optimization: {}", patch.ops.len());
    println!("Operations after optimization: {}", optimized.ops.len());
    println!();
}

fn benchmark_serialization() {
    println!("=== Serialization Benchmark ===");
    
    let from = generate_complex_document();
    let to = generate_complex_document_modified();
    
    let options = DiffOptions::default();
    let patch = diff_html(&from, &to, &options).unwrap();
    
    // JSON serialization
    let start = Instant::now();
    let json = serde_json::to_string(&patch).unwrap();
    let json_time = start.elapsed();
    
    // Binary serialization
    let start = Instant::now();
    let binary = patch.to_binary().unwrap();
    let binary_time = start.elapsed();
    
    // Deserialization
    let start = Instant::now();
    let _ = shipwright_liveview::diff::patch::Patch::from_binary(&binary).unwrap();
    let deserialize_time = start.elapsed();
    
    println!("Patch operations: {}", patch.ops.len());
    println!("JSON size: {} bytes, time: {:?}", json.len(), json_time);
    println!("Binary size: {} bytes, time: {:?}", binary.len(), binary_time);
    println!("Binary deserialization time: {:?}", deserialize_time);
    println!("Size reduction: {:.1}%", (1.0 - (binary.len() as f64 / json.len() as f64)) * 100.0);
}

// Helper functions to generate test documents

fn generate_text_document(paragraphs: usize) -> String {
    let mut html = String::from("<article>");
    for i in 0..paragraphs {
        html.push_str(&format!("<p>This is paragraph number {}. It contains some text content.</p>", i));
    }
    html.push_str("</article>");
    html
}

fn generate_text_document_modified(paragraphs: usize) -> String {
    let mut html = String::from("<article>");
    for i in 0..paragraphs {
        if i % 3 == 0 {
            html.push_str(&format!("<p>This is MODIFIED paragraph number {}. It has been updated.</p>", i));
        } else {
            html.push_str(&format!("<p>This is paragraph number {}. It contains some text content.</p>", i));
        }
    }
    html.push_str("</article>");
    html
}

fn generate_attribute_document(elements: usize) -> String {
    let mut html = String::from("<div>");
    for i in 0..elements {
        html.push_str(&format!(
            r#"<div class="item item-{}" id="elem-{}" data-index="{}">Element {}</div>"#,
            i, i, i, i
        ));
    }
    html.push_str("</div>");
    html
}

fn generate_attribute_document_modified(elements: usize) -> String {
    let mut html = String::from("<div>");
    for i in 0..elements {
        if i % 2 == 0 {
            html.push_str(&format!(
                r#"<div class="item item-{} modified" id="elem-{}" data-index="{}" data-modified="true">Element {} (modified)</div>"#,
                i, i, i, i
            ));
        } else {
            html.push_str(&format!(
                r#"<div class="item item-{}" id="elem-{}" data-index="{}">Element {}</div>"#,
                i, i, i, i
            ));
        }
    }
    html.push_str("</div>");
    html
}

fn generate_list(items: usize, with_keys: bool) -> String {
    let mut html = String::from("<ul>");
    for i in 0..items {
        if with_keys {
            html.push_str(&format!(r#"<li key="item-{}">Item {}</li>"#, i, i));
        } else {
            html.push_str(&format!("<li>Item {}</li>", i));
        }
    }
    html.push_str("</ul>");
    html
}

fn generate_list_reordered(items: usize, with_keys: bool) -> String {
    let mut html = String::from("<ul>");
    // Reverse order
    for i in (0..items).rev() {
        if with_keys {
            html.push_str(&format!(r#"<li key="item-{}">Item {}</li>"#, i, i));
        } else {
            html.push_str(&format!("<li>Item {}</li>", i));
        }
    }
    html.push_str("</ul>");
    html
}

fn generate_complex_document() -> String {
    r#"
    <div class="container">
        <header>
            <h1>Document Title</h1>
            <nav>
                <ul>
                    <li><a href="#section1">Section 1</a></li>
                    <li><a href="#section2">Section 2</a></li>
                    <li><a href="#section3">Section 3</a></li>
                </ul>
            </nav>
        </header>
        <main>
            <section id="section1">
                <h2>Section 1</h2>
                <p>This is the content of section 1.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                    <li>Item 3</li>
                </ul>
            </section>
            <section id="section2">
                <h2>Section 2</h2>
                <p>This is the content of section 2.</p>
                <table>
                    <tr><th>Name</th><th>Value</th></tr>
                    <tr><td>Alpha</td><td>100</td></tr>
                    <tr><td>Beta</td><td>200</td></tr>
                </table>
            </section>
        </main>
        <footer>
            <p>Copyright 2024</p>
        </footer>
    </div>
    "#.to_string()
}

fn generate_complex_document_modified() -> String {
    r#"
    <div class="container updated">
        <header>
            <h1>Updated Document Title</h1>
            <nav>
                <ul>
                    <li><a href="#section1">Section 1</a></li>
                    <li><a href="#section3">Section 3</a></li>
                    <li><a href="#section2">Section 2</a></li>
                    <li><a href="#section4">Section 4</a></li>
                </ul>
            </nav>
        </header>
        <main>
            <section id="section1">
                <h2>Section 1 - Modified</h2>
                <p>This is the updated content of section 1.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2 - Updated</li>
                    <li>Item 3</li>
                    <li>Item 4</li>
                </ul>
            </section>
            <section id="section2">
                <h2>Section 2</h2>
                <p>This is the content of section 2.</p>
                <table>
                    <tr><th>Name</th><th>Value</th><th>Status</th></tr>
                    <tr><td>Alpha</td><td>150</td><td>Active</td></tr>
                    <tr><td>Beta</td><td>200</td><td>Inactive</td></tr>
                    <tr><td>Gamma</td><td>300</td><td>Active</td></tr>
                </table>
            </section>
            <section id="section4">
                <h2>Section 4</h2>
                <p>This is a new section.</p>
            </section>
        </main>
        <footer>
            <p>Copyright 2024 - All rights reserved</p>
        </footer>
    </div>
    "#.to_string()
}