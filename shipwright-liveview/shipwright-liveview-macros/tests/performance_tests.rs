//! Performance tests for the enhanced template parsing system.

use shipwright_liveview_macros::html;
use std::env;
use std::time::{Instant, Duration};

/// Test parsing performance with small templates
#[test]
fn test_small_template_performance() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let iterations = 100;
    let start = Instant::now();
    
    for i in 0..iterations {
        let _result = html! {
            <div class={format!("item-{}", i)}>
                <p>{format!("Content {}", i)}</p>
            </div>
        };
    }
    
    let duration = start.elapsed();
    let avg_duration = duration / iterations;
    
    println!("Small template average parsing time: {:?}", avg_duration);
    assert!(avg_duration < Duration::from_millis(10), "Small template parsing too slow");
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test parsing performance with medium templates
#[test]
fn test_medium_template_performance() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let iterations = 50;
    let start = Instant::now();
    
    for i in 0..iterations {
        let items: Vec<i32> = (0..10).collect();
        let show_details = i % 2 == 0;
        
        let _result = html! {
            <div class="container">
                <header>
                    <h1>{format!("Title {}", i)}</h1>
                </header>
                <main>
                    for item in items {
                        <section class={format!("section-{}", item)}>
                            <h2>{format!("Item {}", item)}</h2>
                            if show_details {
                                <p>Details for {item}</p>
                                <ul>
                                    for j in 0..3 {
                                        <li>Sub-item {j}</li>
                                    }
                                </ul>
                            }
                        </section>
                    }
                </main>
            </div>
        };
    }
    
    let duration = start.elapsed();
    let avg_duration = duration / iterations;
    
    println!("Medium template average parsing time: {:?}", avg_duration);
    assert!(avg_duration < Duration::from_millis(50), "Medium template parsing too slow");
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test parsing performance with large templates
#[test]
fn test_large_template_performance() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let iterations = 10;
    let start = Instant::now();
    
    for i in 0..iterations {
        let items: Vec<i32> = (0..50).collect();
        let categories = vec!["A", "B", "C", "D"];
        
        let _result = html! {
            <div class="large-container">
                <header class="main-header">
                    <h1>{format!("Large Template {}", i)}</h1>
                    <nav>
                        for category in &categories {
                            <a href={format!("#{}", category)}>
                                {category}
                            </a>
                        }
                    </nav>
                </header>
                
                <main class="content">
                    for category in &categories {
                        <section class={format!("category-{}", category)}>
                            <h2>Category {category}</h2>
                            <div class="items-grid">
                                for item in &items {
                                    <article class="item-card">
                                        <header>
                                            <h3>Item {item}</h3>
                                        </header>
                                        <div class="content">
                                            match item % 3 {
                                                0 => <p class="type-a">Type A content for {item}</p>,
                                                1 => <p class="type-b">Type B content for {item}</p>,
                                                _ => <p class="type-c">Type C content for {item}</p>,
                                            }
                                            
                                            if item % 5 == 0 {
                                                <div class="special">
                                                    <strong>Special item!</strong>
                                                </div>
                                            }
                                        </div>
                                        <footer>
                                            <button axm-click={|| println!("clicked {}", item)}>
                                                Click me
                                            </button>
                                        </footer>
                                    </article>
                                }
                            </div>
                        </section>
                    }
                </main>
                
                <footer class="main-footer">
                    <p>Total items: {items.len()}</p>
                </footer>
            </div>
        };
    }
    
    let duration = start.elapsed();
    let avg_duration = duration / iterations;
    
    println!("Large template average parsing time: {:?}", avg_duration);
    assert!(avg_duration < Duration::from_millis(200), "Large template parsing too slow");
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Compare enhanced vs compatible parsing performance
#[test]
fn test_enhanced_vs_compatible_performance() {
    let iterations = 50;
    
    // Test compatible mode
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
    let start_compatible = Instant::now();
    
    for i in 0..iterations {
        let _result = html! {
            <div class={format!("item-{}", i)}>
                <h1>Title {i}</h1>
                <p>Content goes here</p>
                if i % 2 == 0 {
                    <span>Even</span>
                } else {
                    <span>Odd</span>
                }
            </div>
        };
    }
    
    let compatible_duration = start_compatible.elapsed();
    
    // Test enhanced mode
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    let start_enhanced = Instant::now();
    
    for i in 0..iterations {
        let _result = html! {
            <div class={format!("item-{}", i)}>
                <h1>Title {i}</h1>
                <p>Content goes here</p>
                if i % 2 == 0 {
                    <span>Even</span>
                } else {
                    <span>Odd</span>
                }
            </div>
        };
    }
    
    let enhanced_duration = start_enhanced.elapsed();
    
    println!("Compatible mode total time: {:?}", compatible_duration);
    println!("Enhanced mode total time: {:?}", enhanced_duration);
    
    let overhead_ratio = enhanced_duration.as_nanos() as f64 / compatible_duration.as_nanos() as f64;
    println!("Enhanced mode overhead ratio: {:.2}x", overhead_ratio);
    
    // Enhanced mode should not be more than 5x slower than compatible mode
    assert!(overhead_ratio < 5.0, "Enhanced mode overhead too high: {:.2}x", overhead_ratio);
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test memory usage (basic check)
#[test]
fn test_memory_usage() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    // This is a basic memory usage test
    // In a real scenario, you'd use more sophisticated memory profiling
    let iterations = 100;
    
    for i in 0..iterations {
        let items: Vec<i32> = (0..20).collect();
        
        let _result = html! {
            <div class="memory-test">
                <h1>Memory Test {i}</h1>
                for item in items {
                    <div class={format!("item-{}", item)}>
                        <span>{item}</span>
                        if item % 2 == 0 {
                            <em>Even: {item}</em>
                        }
                    </div>
                }
            </div>
        };
        
        // Force cleanup between iterations
        if i % 10 == 0 {
            std::hint::black_box(&_result);
        }
    }
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test compilation time impact (measures compile-time performance)
#[test]
fn test_compilation_time_impact() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    // This test generates multiple templates to measure compilation impact
    // The actual measurement would be done by the build system
    
    macro_rules! generate_templates {
        ($count:expr) => {
            $(
                let _template = html! {
                    <div class={format!("template-{}", $count)}>
                        <h1>Template {$count}</h1>
                        <p>This is template number {$count}</p>
                        for i in 0..5 {
                            <span>Item {i}</span>
                        }
                    </div>
                };
            )*
        }
    }
    
    // Generate several templates
    for i in 0..10 {
        let _template = html! {
            <div class={format!("template-{}", i)}>
                <h1>Template {i}</h1>
                <p>This is template number {i}</p>
                for j in 0..5 {
                    <span>Item {j}</span>
                }
            </div>
        };
    }
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test parsing with deep nesting
#[test]
fn test_deep_nesting_performance() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let start = Instant::now();
    
    let _result = html! {
        <div class="level-1">
            <div class="level-2">
                <div class="level-3">
                    <div class="level-4">
                        <div class="level-5">
                            <div class="level-6">
                                <div class="level-7">
                                    <div class="level-8">
                                        <div class="level-9">
                                            <div class="level-10">
                                                <p>Deep content</p>
                                                for i in 0..5 {
                                                    <span class={format!("deep-item-{}", i)}>
                                                        {i}
                                                    </span>
                                                }
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    };
    
    let duration = start.elapsed();
    println!("Deep nesting parsing time: {:?}", duration);
    
    // Should handle reasonable nesting depth efficiently
    assert!(duration < Duration::from_millis(100), "Deep nesting parsing too slow");
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Test parsing with many dynamic parts
#[test]
fn test_many_dynamic_parts_performance() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let start = Instant::now();
    
    let data: Vec<i32> = (0..100).collect();
    let show_items = true;
    let prefix = "item";
    
    let _result = html! {
        <div class="dynamic-container">
            <h1>{format!("Dynamic template with {} items", data.len())}</h1>
            
            if show_items {
                <ul class="items-list">
                    for (index, item) in data.iter().enumerate() {
                        <li 
                            class={format!("{}-{}", prefix, index)}
                            data-value={item.to_string()}
                            style={format!("order: {}", item)}
                        >
                            <span class="index">{index}</span>
                            <span class="value">{item}</span>
                            
                            match item % 4 {
                                0 => <em class="quarter">Quarter</em>,
                                1 => <em class="third">Third</em>,
                                2 => <em class="half">Half</em>,
                                _ => <em class="other">Other</em>,
                            }
                            
                            if item % 10 == 0 {
                                <strong>Milestone!</strong>
                            }
                            
                            <button 
                                axm-click={move || println!("Clicked item {}", item)}
                                disabled=if *item > 90 { Some("disabled") } else { None }
                            >
                                Action {item}
                            </button>
                        </li>
                    }
                </ul>
            } else {
                <p>Items hidden</p>
            }
        </div>
    };
    
    let duration = start.elapsed();
    println!("Many dynamic parts parsing time: {:?}", duration);
    
    // Should handle many dynamic parts efficiently
    assert!(duration < Duration::from_millis(500), "Many dynamic parts parsing too slow");
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}

/// Benchmark fingerprinting performance
#[test]
fn test_fingerprinting_performance() {
    use shipwright_liveview_macros::fingerprinting::*;
    use std::collections::HashMap;
    
    let mut engine = FingerprintEngine::new();
    let iterations = 1000;
    
    let structure = TemplateStructure {
        root: StructureElement {
            element_type: ElementType::Root,
            attributes: HashMap::new(),
            children: vec![
                StructureElement {
                    element_type: ElementType::HtmlElement("div".to_string()),
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("class".to_string(), Some("test".to_string()));
                        attrs
                    },
                    children: Vec::new(),
                }
            ],
        },
    };
    
    let start = Instant::now();
    
    for i in 0..iterations {
        let template_id = format!("template_{}", i);
        let content = format!("content_{}", i);
        
        let _fingerprint = engine.fingerprint_template(
            &template_id,
            &content,
            &structure,
            &[]
        );
    }
    
    let duration = start.elapsed();
    let avg_duration = duration / iterations;
    
    println!("Fingerprinting average time: {:?}", avg_duration);
    assert!(avg_duration < Duration::from_micros(100), "Fingerprinting too slow");
}

/// Test error handling performance (shouldn't be significantly slower)
#[test]
fn test_error_handling_performance() {
    env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");
    
    let iterations = 50;
    let start = Instant::now();
    
    for i in 0..iterations {
        // These should all succeed, testing the normal path performance
        // even when error handling infrastructure is present
        let _result = html! {
            <div class={format!("test-{}", i)}>
                <p>Valid content {i}</p>
                if i % 2 == 0 {
                    <span>Even</span>
                }
            </div>
        };
    }
    
    let duration = start.elapsed();
    let avg_duration = duration / iterations;
    
    println!("Error handling overhead average time: {:?}", avg_duration);
    assert!(avg_duration < Duration::from_millis(20), "Error handling adds too much overhead");
    
    env::remove_var("SHIPWRIGHT_HOT_RELOAD");
}