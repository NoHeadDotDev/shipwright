
//! Reusable LiveView components
//!
//! This module contains reusable UI components that can be composed
//! to build complex user interfaces. Components are self-contained
//! and can manage their own state.

pub mod button;
pub mod modal;
pub mod progress_bar;
pub mod form;
pub mod notification;
pub mod tabs;

// Re-export commonly used components
pub use button::*;
pub use modal::*;
pub use progress_bar::*;
pub use form::*;
pub use notification::*;
pub use tabs::*;

use shipwright_liveview::Html;
use shipwright_liveview_macros::html;

/// Common component props trait
pub trait ComponentProps {
    /// CSS classes to apply to the component
    fn class(&self) -> Option<&str> {
        None
    }
    
    /// Additional CSS styles
    fn style(&self) -> Option<&str> {
        None
    }
    
    /// Component ID
    fn id(&self) -> Option<&str> {
        None
    }
}

/// Helper function to build CSS class string
pub fn build_class(base: &str, additional: Option<&str>) -> String {
    match additional {
        Some(extra) => format!("{} {}", base, extra),
        None => base.to_string(),
    }
}

/// Helper function to render component wrapper with common attributes
pub fn component_wrapper<T>(
    tag: &str,
    class: &str,
    props: &dyn ComponentProps,
    content: Html<T>,
) -> Html<T> {
    let class_str = build_class(class, props.class());
    
    html! {
        <div 
            class={ class_str }
            style={ props.style().unwrap_or("") }
            id={ props.id().unwrap_or("") }
        >
            { content }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProps {
        class: Option<String>,
    }

    impl ComponentProps for TestProps {
        fn class(&self) -> Option<&str> {
            self.class.as_deref()
        }
    }

    #[test]
    fn test_build_class() {
        assert_eq!(build_class("base", None), "base");
        assert_eq!(build_class("base", Some("extra")), "base extra");
    }

    #[test]
    fn test_component_props() {
        let props = TestProps {
            class: Some("custom-class".to_string()),
        };
        assert_eq!(props.class(), Some("custom-class"));
    }
}