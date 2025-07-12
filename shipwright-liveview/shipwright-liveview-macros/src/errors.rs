//! Enhanced error handling for shipwright-liveview HTML macros.
//!
//! This module provides HTML-specific error messages with context and suggestions,
//! improving upon the basic syn::Error usage throughout the codebase.

use crate::html5_validation::ValidationError;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::fmt;
use syn::spanned::Spanned;

/// Enhanced error type for HTML macro parsing with HTML-specific context and suggestions.
#[derive(Debug, Clone)]
pub(crate) struct HtmlError {
    /// The span where the error occurred
    pub(crate) span: Span,
    /// The main error message
    pub(crate) message: String,
    /// HTML-specific context for the error
    pub(crate) context: Option<HtmlContext>,
    /// Helpful suggestions for fixing the error
    pub(crate) suggestions: Vec<String>,
    /// The underlying error kind
    pub(crate) kind: HtmlErrorKind,
}

/// HTML-specific context information for errors
#[derive(Debug, Clone)]
pub(crate) struct HtmlContext {
    /// The HTML element being parsed when the error occurred
    pub element: Option<String>,
    /// The attribute being parsed when the error occurred
    pub attribute: Option<String>,
    /// The nesting depth when the error occurred
    pub nesting_depth: Option<usize>,
    /// Whether we're inside a control flow construct (if, for, match)
    pub in_control_flow: bool,
}

/// Kinds of HTML parsing errors
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum HtmlErrorKind {
    /// Syntax error in HTML structure
    Syntax,
    /// Invalid HTML element or attribute
    InvalidHtml,
    /// Mismatched opening/closing tags
    MismatchedTags,
    /// Invalid nesting of elements
    InvalidNesting,
    /// Missing required attributes
    MissingAttribute,
    /// Invalid attribute value
    InvalidAttributeValue,
    /// Control flow syntax error
    ControlFlow,
    /// HTML5 validation error
    Html5Validation(ValidationError),
    /// Unexpected token in HTML context
    UnexpectedToken,
}

impl HtmlError {
    /// Create a new HTML error with the given span and message
    pub(crate) fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            context: None,
            suggestions: Vec::new(),
            kind: HtmlErrorKind::Syntax,
        }
    }

    /// Create an error for mismatched HTML tags
    pub(crate) fn mismatched_tags(
        open_span: Span,
        close_span: Span,
        open_tag: &str,
        close_tag: &str,
    ) -> Self {
        let span = open_span
            .join(close_span)
            .unwrap_or(close_span);
        
        let mut error = Self {
            span,
            message: format!("Mismatched HTML tags: opening tag '{}' does not match closing tag '{}'", open_tag, close_tag),
            context: Some(HtmlContext {
                element: Some(open_tag.to_string()),
                attribute: None,
                nesting_depth: None,
                in_control_flow: false,
            }),
            suggestions: vec![
                format!("Change the closing tag to '</{}>>'", open_tag),
                format!("Change the opening tag to '<{}>'", close_tag),
                "Ensure all HTML tags are properly matched".to_string(),
            ],
            kind: HtmlErrorKind::MismatchedTags,
        };

        // Add specific suggestions for common mistakes
        if open_tag.to_lowercase() == close_tag.to_lowercase() {
            error.suggestions.insert(0, "Check the case sensitivity of your tag names".to_string());
        }
        
        error
    }

    /// Create an error for void elements with children
    pub(crate) fn void_element_with_children(span: Span, element: &str) -> Self {
        Self {
            span,
            message: format!("Void element '{}' cannot have children or a closing tag", element),
            context: Some(HtmlContext {
                element: Some(element.to_string()),
                attribute: None,
                nesting_depth: None,
                in_control_flow: false,
            }),
            suggestions: vec![
                format!("Use '<{} />' for self-closing syntax", element),
                "Remove any children or closing tag".to_string(),
                format!("Void elements like '{}' cannot contain content", element),
            ],
            kind: HtmlErrorKind::InvalidHtml,
        }
    }

    /// Create an error for unexpected tokens in HTML context
    pub(crate) fn unexpected_token(span: Span, context: Option<HtmlContext>) -> Self {
        let mut suggestions = vec![
            "Expected an HTML element, text, or control flow construct".to_string(),
            "Check that all braces and parentheses are properly balanced".to_string(),
        ];

        let message = if let Some(ref ctx) = context {
            if ctx.in_control_flow {
                suggestions.push("Ensure control flow syntax (if, for, match) is correct".to_string());
                "Unexpected token in control flow construct".to_string()
            } else if let Some(ref element) = ctx.element {
                suggestions.push(format!("Check the syntax for element '{}'", element));
                format!("Unexpected token while parsing element '{}'", element)
            } else {
                "Unexpected token in HTML macro".to_string()
            }
        } else {
            "Unexpected token in HTML macro".to_string()
        };

        Self {
            span,
            message,
            context,
            suggestions,
            kind: HtmlErrorKind::UnexpectedToken,
        }
    }

    /// Create an error for invalid HTML5 validation
    pub(crate) fn html5_validation(span: Span, validation_error: ValidationError, context: Option<HtmlContext>) -> Self {
        let mut suggestions = Vec::new();

        match &validation_error {
            ValidationError::EmptyElementName => {
                suggestions.push("Provide a valid HTML element name".to_string());
                suggestions.push("Element names must start with a letter".to_string());
            }
            ValidationError::InvalidElementNameStart(c) => {
                suggestions.push(format!("Element names cannot start with '{}' - use a letter instead", c));
                suggestions.push("Valid examples: 'div', 'span', 'custom-element'".to_string());
            }
            ValidationError::InvalidElementNameChar(c, _) => {
                suggestions.push(format!("Remove or replace the invalid character '{}'", c));
                suggestions.push("Element names can only contain letters, numbers, hyphens, underscores, and dots".to_string());
            }
            ValidationError::EmptyAttributeName => {
                suggestions.push("Provide a valid attribute name".to_string());
                suggestions.push("Attribute names cannot be empty".to_string());
            }
            ValidationError::InvalidAttributeNameChar(c, _) => {
                suggestions.push(format!("Remove or replace the invalid character '{}'", c));
                suggestions.push("Attribute names cannot contain spaces, quotes, or control characters".to_string());
            }
            ValidationError::VoidElementWithChildren(element) => {
                suggestions.push(format!("Use '<{} />' for the void element", element));
                suggestions.push("Remove any children or closing tag".to_string());
            }
            ValidationError::InvalidNesting { parent, child, reason } => {
                suggestions.push(format!("Move '{}' outside of '{}'", child, parent));
                suggestions.push(format!("Reason: {}", reason));
            }
        }

        Self {
            span,
            message: validation_error.to_string(),
            context,
            suggestions,
            kind: HtmlErrorKind::Html5Validation(validation_error),
        }
    }

    /// Create an error for unknown attributes
    pub(crate) fn unknown_attribute(span: Span, attr_name: &str, element: Option<&str>) -> Self {
        let mut suggestions = vec![
            "Check the spelling of the attribute name".to_string(),
            "Refer to HTML5 specification for valid attributes".to_string(),
        ];

        let message = if let Some(element) = element {
            suggestions.push(format!("Check which attributes are valid for the '{}' element", element));
            format!("Unknown attribute '{}' for element '{}'", attr_name, element)
        } else {
            format!("Unknown attribute '{}'", attr_name)
        };

        // Add specific suggestions for common typos
        if attr_name.starts_with("axm-") {
            suggestions.insert(0, "Check that this is a valid LiveView attribute".to_string());
            suggestions.insert(1, "Refer to shipwright-liveview documentation for valid axm- attributes".to_string());
        }

        Self {
            span,
            message,
            context: Some(HtmlContext {
                element: element.map(String::from),
                attribute: Some(attr_name.to_string()),
                nesting_depth: None,
                in_control_flow: false,
            }),
            suggestions,
            kind: HtmlErrorKind::InvalidHtml,
        }
    }

    /// Create an error for invalid attribute values
    pub(crate) fn invalid_attribute_value(
        span: Span,
        attr_name: &str,
        value: &str,
        expected: &str,
    ) -> Self {
        Self {
            span,
            message: format!("Invalid value '{}' for attribute '{}'", value, attr_name),
            context: Some(HtmlContext {
                element: None,
                attribute: Some(attr_name.to_string()),
                nesting_depth: None,
                in_control_flow: false,
            }),
            suggestions: vec![
                format!("Expected: {}", expected),
                "Check the HTML5 specification for valid attribute values".to_string(),
                "Ensure the value is properly quoted if it contains spaces".to_string(),
            ],
            kind: HtmlErrorKind::InvalidAttributeValue,
        }
    }

    /// Create an error for control flow syntax issues
    pub(crate) fn control_flow_syntax(span: Span, construct: &str, issue: &str) -> Self {
        let suggestions = match construct {
            "if" => vec![
                "Check that the if condition is valid Rust syntax".to_string(),
                "Ensure braces are properly balanced: if condition { ... }".to_string(),
                "Optional else clause: if condition { ... } else { ... }".to_string(),
            ],
            "for" => vec![
                "Check the for loop syntax: for item in iterator { ... }".to_string(),
                "Ensure the iterator expression is valid".to_string(),
                "Pattern must be a valid Rust pattern".to_string(),
            ],
            "match" => vec![
                "Check match expression syntax: match value { pattern => { ... }, }".to_string(),
                "Ensure all arms end with commas".to_string(),
                "Pattern must be followed by '=>'".to_string(),
            ],
            _ => vec![
                "Check the control flow syntax".to_string(),
                "Refer to Rust documentation for control flow constructs".to_string(),
            ],
        };

        Self {
            span,
            message: format!("Invalid {} syntax: {}", construct, issue),
            context: Some(HtmlContext {
                element: None,
                attribute: None,
                nesting_depth: None,
                in_control_flow: true,
            }),
            suggestions,
            kind: HtmlErrorKind::ControlFlow,
        }
    }

    /// Add context to an existing error
    pub(crate) fn with_context(mut self, context: HtmlContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Add a suggestion to the error
    pub(crate) fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// Add multiple suggestions to the error
    pub(crate) fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    /// Convert this error into a syn::Error for compatibility
    pub(crate) fn into_syn_error(self) -> syn::Error {
        let mut message = self.message.clone();

        // Add context information
        if let Some(context) = &self.context {
            if let Some(element) = &context.element {
                message.push_str(&format!(" (in element '{}')", element));
            }
            if let Some(attribute) = &context.attribute {
                message.push_str(&format!(" (in attribute '{}')", attribute));
            }
        }

        // Add suggestions
        if !self.suggestions.is_empty() {
            message.push_str("\n\nSuggestions:");
            for suggestion in &self.suggestions {
                message.push_str(&format!("\n  - {}", suggestion));
            }
        }

        syn::Error::new(self.span, message)
    }

    /// Convert this error into a compile error token stream
    pub(crate) fn into_compile_error(self) -> TokenStream {
        self.into_syn_error().into_compile_error()
    }
}

impl fmt::Display for HtmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;

        if let Some(context) = &self.context {
            if let Some(element) = &context.element {
                write!(f, " (in element '{}')", element)?;
            }
            if let Some(attribute) = &context.attribute {
                write!(f, " (in attribute '{}')", attribute)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for HtmlError {}

/// Convenience trait for creating HTML errors from spans
pub(crate) trait HtmlErrorExt {
    /// Create an HTML error with this span
    fn html_error(self, message: impl Into<String>) -> HtmlError;
    
    /// Create a mismatched tags error
    fn mismatched_tags_error(self, close_span: Span, open_tag: &str, close_tag: &str) -> HtmlError;
    
    /// Create an unexpected token error
    fn unexpected_token_error(self, context: Option<HtmlContext>) -> HtmlError;
}

impl HtmlErrorExt for Span {
    fn html_error(self, message: impl Into<String>) -> HtmlError {
        HtmlError::new(self, message)
    }
    
    fn mismatched_tags_error(self, close_span: Span, open_tag: &str, close_tag: &str) -> HtmlError {
        HtmlError::mismatched_tags(self, close_span, open_tag, close_tag)
    }
    
    fn unexpected_token_error(self, context: Option<HtmlContext>) -> HtmlError {
        HtmlError::unexpected_token(self, context)
    }
}

/// Result type for HTML parsing operations
pub(crate) type HtmlResult<T> = Result<T, HtmlError>;

/// Trait for converting other errors into HtmlError
pub(crate) trait IntoHtmlError {
    fn into_html_error(self, span: Span) -> HtmlError;
}

impl IntoHtmlError for ValidationError {
    fn into_html_error(self, span: Span) -> HtmlError {
        HtmlError::html5_validation(span, self, None)
    }
}

impl IntoHtmlError for syn::Error {
    fn into_html_error(self, _span: Span) -> HtmlError {
        // Extract span from syn::Error if possible, otherwise use provided span
        let span = self.span();
        HtmlError::new(span, self.to_string())
    }
}

/// Helper for creating better error messages with span information
pub(crate) struct ErrorReporter {
    /// Current HTML context
    context: HtmlContext,
}

impl ErrorReporter {
    /// Create a new error reporter
    pub(crate) fn new() -> Self {
        Self {
            context: HtmlContext {
                element: None,
                attribute: None,
                nesting_depth: Some(0),
                in_control_flow: false,
            },
        }
    }

    /// Enter an HTML element context
    pub(crate) fn enter_element(&mut self, element: String) {
        self.context.element = Some(element);
        if let Some(depth) = &mut self.context.nesting_depth {
            *depth += 1;
        }
    }

    /// Exit the current element context
    pub(crate) fn exit_element(&mut self) {
        self.context.element = None;
        if let Some(depth) = &mut self.context.nesting_depth {
            *depth = depth.saturating_sub(1);
        }
    }

    /// Enter an attribute context
    pub(crate) fn enter_attribute(&mut self, attribute: String) {
        self.context.attribute = Some(attribute);
    }

    /// Exit the current attribute context
    pub(crate) fn exit_attribute(&mut self) {
        self.context.attribute = None;
    }

    /// Enter a control flow context
    pub(crate) fn enter_control_flow(&mut self) {
        self.context.in_control_flow = true;
    }

    /// Exit the control flow context
    pub(crate) fn exit_control_flow(&mut self) {
        self.context.in_control_flow = false;
    }

    /// Get the current context
    pub(crate) fn context(&self) -> &HtmlContext {
        &self.context
    }

    /// Create an error with the current context
    pub(crate) fn error(&self, span: Span, message: impl Into<String>) -> HtmlError {
        HtmlError::new(span, message).with_context(self.context.clone())
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    #[test]
    fn test_html_error_creation() {
        let span = Span::call_site();
        let error = HtmlError::new(span, "Test error");
        assert_eq!(error.message, "Test error");
        assert_eq!(error.kind, HtmlErrorKind::Syntax);
    }

    #[test]
    fn test_mismatched_tags_error() {
        let span = Span::call_site();
        let error = HtmlError::mismatched_tags(span, span, "div", "span");
        assert!(error.message.contains("Mismatched HTML tags"));
        assert_eq!(error.kind, HtmlErrorKind::MismatchedTags);
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_void_element_error() {
        let span = Span::call_site();
        let error = HtmlError::void_element_with_children(span, "br");
        assert!(error.message.contains("Void element"));
        assert_eq!(error.kind, HtmlErrorKind::InvalidHtml);
    }

    #[test]
    fn test_error_reporter() {
        let mut reporter = ErrorReporter::new();
        reporter.enter_element("div".to_string());
        reporter.enter_attribute("class".to_string());
        
        assert_eq!(reporter.context().element, Some("div".to_string()));
        assert_eq!(reporter.context().attribute, Some("class".to_string()));
        assert_eq!(reporter.context().nesting_depth, Some(1));
        
        reporter.exit_attribute();
        reporter.exit_element();
        
        assert_eq!(reporter.context().element, None);
        assert_eq!(reporter.context().attribute, None);
        assert_eq!(reporter.context().nesting_depth, Some(0));
    }

    #[test]
    fn test_html5_validation_error() {
        let span = Span::call_site();
        let validation_err = ValidationError::EmptyElementName;
        let error = HtmlError::html5_validation(span, validation_err, None);
        
        assert_eq!(error.kind, HtmlErrorKind::Html5Validation(ValidationError::EmptyElementName));
        assert!(!error.suggestions.is_empty());
    }
}