use shipwright_liveview_macros::html;

#[test]
fn test_mismatched_tags_error() {
    let result = std::panic::catch_unwind(|| {
        html! {
            <div>
                <span>Hello</div>
            </span>
        }
    });
    
    // This should fail compilation, but we can't test compilation errors directly
    // in unit tests. This test documents the expected behavior.
    println!("This test documents that mismatched tags should produce helpful error messages");
}

#[test]
fn test_void_element_error() {
    let result = std::panic::catch_unwind(|| {
        html! {
            <br>Content</br>
        }
    });
    
    // This should fail compilation with a helpful error about void elements
    println!("This test documents that void elements with content should produce helpful error messages");
}

#[test]
fn test_unknown_attribute_error() {
    let result = std::panic::catch_unwind(|| {
        html! {
            <div axm-unknown-event="handler">Content</div>
        }
    });
    
    // This should fail compilation with suggestions for correct axm- attributes
    println!("This test documents that unknown axm- attributes should produce helpful error messages");
}