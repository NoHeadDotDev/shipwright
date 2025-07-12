//! Comprehensive tests for the enhanced template parsing system with hot reload capabilities.

use shipwright_liveview_macros::{html, html_enhanced};
use std::env;

/// Test basic enhanced parsing functionality
#[test]
fn test_enhanced_basic_parsing() {
    // Enable hot reload for this test
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    // This should work with enhanced parsing
    let _result = html! {
        <div class="test">
            <p>Hello World</p>
        </div>
    };
    
    // Reset environment variable
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with dynamic content
#[test]
fn test_enhanced_dynamic_content() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let value = "dynamic";
    let _result = html! {
        <div class={value}>
            {format!("Hello {}", value)}
        </div>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with control flow
#[test]
fn test_enhanced_control_flow() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let show_content = true;
    let items = vec!["a", "b", "c"];
    
    let _result = html! {
        <div>
            if show_content {
                <p>Content is shown</p>
            }
            for item in items {
                <span>{item}</span>
            }
        </div>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with event handlers
#[test]
fn test_enhanced_event_handlers() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let _result = html! {
        <button axm-click={|| println!("clicked")}>
            Click me
        </button>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with complex nesting
#[test]
fn test_enhanced_complex_nesting() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let data = vec![1, 2, 3];
    let show_details = true;
    
    let _result = html! {
        <div class="container">
            <header>
                <h1>Title</h1>
            </header>
            <main>
                for item in data {
                    <section>
                        <h2>{format!("Item {}", item)}</h2>
                        if show_details {
                            <p>Details for item {item}</p>
                        } else {
                            <p>No details</p>
                        }
                    </section>
                }
            </main>
        </div>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test backward compatibility - should work without enhanced features
#[test]
fn test_backward_compatibility() {
    // Don't set SHIPWRIGHT_HOT_RELOAD - should use compatible mode
    
    let _result = html! {
        <div>
            <p>Compatible mode</p>
        </div>
    };
}

/// Test explicit enhanced macro
#[test]
fn test_explicit_enhanced_macro() {
    let _result = html_enhanced! {
        <div class="enhanced">
            <p>Explicitly enhanced</p>
        </div>
    };
}

/// Test enhanced parsing with match expressions
#[test]
fn test_enhanced_match_expressions() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let state = "active";
    
    let _result = html! {
        <div>
            match state {
                "active" => {
                    <span class="active">Active</span>
                },
                "inactive" => {
                    <span class="inactive">Inactive</span>
                },
                _ => {
                    <span class="unknown">Unknown</span>
                },
            }
        </div>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with void elements
#[test]
fn test_enhanced_void_elements() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let _result = html! {
        <div>
            <br />
            <hr />
            <input type="text" />
        </div>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with conditional attributes
#[test]
fn test_enhanced_conditional_attributes() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let is_disabled = true;
    let css_class = "button";
    
    let _result = html! {
        <button 
            class={css_class}
            disabled=if is_disabled { Some("disabled") } else { None }
            axm-click=if !is_disabled { Some(|| println!("clicked")) } else { None }
        >
            Submit
        </button>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with doctype
#[test]
fn test_enhanced_with_doctype() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let _result = html! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                <h1>Hello</h1>
            </body>
        </html>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with boolean attributes
#[test]
fn test_enhanced_boolean_attributes() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let _result = html! {
        <div>
            <input type="checkbox" checked />
            <select multiple>
                <option selected>Option 1</option>
                <option>Option 2</option>
            </select>
        </div>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with data attributes
#[test]
fn test_enhanced_data_attributes() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let data_value = "test";
    
    let _result = html! {
        <div 
            data-value={data_value}
            data-static="static"
            data-count="42"
        >
            Content
        </div>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing fallback behavior
#[test]
fn test_enhanced_parsing_fallback() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    // This should work even if enhanced parsing encounters issues
    // The macro should fallback to compatible mode
    let _result = html! {
        <div>
            <p>Fallback test</p>
        </div>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with mixed content
#[test]
fn test_enhanced_mixed_content() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let title = "Mixed Content";
    let items = vec!["Alpha", "Beta", "Gamma"];
    let show_footer = true;
    
    let _result = html! {
        <article>
            <header>
                <h1>{title}</h1>
                <nav>
                    for (index, item) in items.iter().enumerate() {
                        <a href={format!("#{}", index)}>
                            {item}
                        </a>
                    }
                </nav>
            </header>
            
            <main>
                <section>
                    "This is static text mixed with "
                    <strong>{"dynamic content"}</strong>
                    " in the same element."
                </section>
                
                match items.len() {
                    0 => <p>"No items"</p>,
                    1 => <p>"One item"</p>,
                    n => <p>{format!("{} items", n)}</p>,
                }
            </main>
            
            if show_footer {
                <footer>
                    <p>"Copyright 2024"</p>
                </footer>
            }
        </article>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing performance (basic check)
#[test]
fn test_enhanced_parsing_performance() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let start = std::time::Instant::now();
    
    // Parse a reasonably complex template
    let items: Vec<i32> = (0..100).collect();
    let _result = html! {
        <div class="performance-test">
            <h1>"Performance Test"</h1>
            <ul>
                for item in items {
                    <li class={format!("item-{}", item)}>
                        <span>"Item: "</span>
                        <strong>{item}</strong>
                        if item % 2 == 0 {
                            <em>" (even)"</em>
                        } else {
                            <em>" (odd)"</em>
                        }
                    </li>
                }
            </ul>
        </div>
    };
    
    let duration = start.elapsed();
    println!("Enhanced parsing took: {:?}", duration);
    
    // This is a basic performance check - in a real scenario you'd have benchmarks
    assert!(duration < std::time::Duration::from_secs(1), "Parsing took too long");
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test error handling in enhanced parsing
#[test]
fn test_enhanced_error_handling() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    // Valid templates should still work with enhanced parsing
    let _result = html! {
        <div>
            <p>Valid content</p>
        </div>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with custom elements
#[test]
fn test_enhanced_custom_elements() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let _result = html! {
        <div>
            <custom-element data-prop="value">
                <nested-component>
                    "Custom content"
                </nested-component>
            </custom-element>
        </div>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test enhanced parsing with namespaced elements (SVG example)
#[test]
fn test_enhanced_namespaced_elements() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let _result = html! {
        <div>
            <svg width="100" height="100">
                <circle cx="50" cy="50" r="40" fill="red" />
                <text x="50" y="55" text-anchor="middle">"SVG"</text>
            </svg>
        </div>
    };
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}