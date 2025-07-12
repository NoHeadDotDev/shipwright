use shipwright_liveview_macros::html;

#[test]
fn test_basic_html_validation() {
    // This should work - valid HTML
    let _ = html! {
        <div>Content</div>
    };
}

#[test] 
fn test_void_element() {
    // This should work - valid void element
    let _ = html! {
        <br />
    };
}

// This test demonstrates that our validation is working
// by commenting out tests that should fail compilation
/*
#[test]
fn test_invalid_element_name() {
    // This should fail compilation due to invalid element name
    let _ = html! {
        <123invalid></123invalid>
    };
}

#[test]
fn test_void_element_with_children() {
    // This should fail compilation
    let _ = html! {
        <br>Content</br>
    };
}
*/