//! Comprehensive attribute validation and handling module.
//!
//! This module provides robust validation for HTML attributes, including:
//! - Attribute name validation according to HTML5 standards
//! - Attribute value type validation and coercion
//! - Proper quote handling for all attribute values
//! - Boolean attribute support
//! - Integration with html5ever for standards compliance

use crate::html5_validation::{Html5Validator, ValidationError};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::fmt;
use syn::{spanned::Spanned, Block, LitStr};

/// Comprehensive attribute processor that handles all aspects of attribute validation
#[derive(Debug)]
pub(crate) struct AttributeProcessor {
    validator: Html5Validator,
}

impl Default for AttributeProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeProcessor {
    /// Create a new attribute processor with HTML5 validation
    pub(crate) fn new() -> Self {
        Self {
            validator: Html5Validator::new(),
        }
    }

    /// Validate and process an attribute name
    pub(crate) fn validate_attribute_name(&self, name: &str, span: Span) -> syn::Result<ValidatedAttributeName> {
        // First validate using HTML5 rules
        self.validator
            .validate_attribute_name(name)
            .map_err(|err| syn::Error::new(span, format!("Invalid attribute name: {}", err)))?;

        // Check for special attributes that need special handling
        let attr_type = if self.validator.is_boolean_attribute(name) {
            AttributeType::Boolean
        } else if name.starts_with("data-") {
            AttributeType::Data
        } else if name.starts_with("aria-") {
            AttributeType::Aria
        } else if name.starts_with("axm-") {
            AttributeType::AxmEvent
        } else if is_event_attribute(name) {
            AttributeType::Event
        } else {
            AttributeType::Standard
        };

        Ok(ValidatedAttributeName {
            name: name.to_string(),
            attr_type,
            span,
        })
    }

    /// Validate and process an attribute value
    pub(crate) fn validate_attribute_value(
        &self,
        attr_name: &ValidatedAttributeName,
        value: &AttributeValue,
    ) -> syn::Result<ValidatedAttributeValue> {
        match (&attr_name.attr_type, value) {
            // Boolean attributes should only have boolean-style values
            (AttributeType::Boolean, AttributeValue::None) => {
                Ok(ValidatedAttributeValue::BooleanPresent)
            }
            (AttributeType::Boolean, AttributeValue::Unit) => {
                Ok(ValidatedAttributeValue::BooleanPresent)
            }
            (AttributeType::Boolean, AttributeValue::LitStr(lit)) => {
                let val = lit.value();
                if val.is_empty() || val == attr_name.name {
                    Ok(ValidatedAttributeValue::BooleanPresent)
                } else {
                    Err(syn::Error::new(
                        lit.span(),
                        format!(
                            "Boolean attribute '{}' should be empty or match attribute name, got '{}'",
                            attr_name.name, val
                        ),
                    ))
                }
            }
            (AttributeType::Boolean, AttributeValue::Block(block)) => {
                // Allow dynamic boolean values through blocks
                Ok(ValidatedAttributeValue::DynamicBoolean(block.clone()))
            }

            // Event attributes need special handling
            (AttributeType::Event | AttributeType::AxmEvent, AttributeValue::Block(block)) => {
                Ok(ValidatedAttributeValue::EventHandler(block.clone()))
            }
            (AttributeType::Event | AttributeType::AxmEvent, _) => {
                Err(syn::Error::new(
                    value.span(),
                    format!("Event attribute '{}' requires a block expression", attr_name.name),
                ))
            }

            // Standard attributes can have various value types
            (_, AttributeValue::LitStr(lit)) => {
                Ok(ValidatedAttributeValue::QuotedString(self.ensure_proper_quotes(lit)?))
            }
            (_, AttributeValue::Block(block)) => {
                Ok(ValidatedAttributeValue::Dynamic(block.clone()))
            }
            (_, AttributeValue::Unit) => {
                Err(syn::Error::new(
                    attr_name.span,
                    format!("Attribute '{}' requires a value", attr_name.name),
                ))
            }
            (_, AttributeValue::None) => {
                if attr_name.attr_type == AttributeType::Boolean {
                    Ok(ValidatedAttributeValue::BooleanPresent)
                } else {
                    Err(syn::Error::new(
                        attr_name.span,
                        format!("Non-boolean attribute '{}' requires a value", attr_name.name),
                    ))
                }
            }
        }
    }

    /// Ensure proper quote handling for string literals
    fn ensure_proper_quotes(&self, lit: &LitStr) -> syn::Result<QuotedAttributeValue> {
        let value = lit.value();
        let quote_style = determine_quote_style(&value);
        
        Ok(QuotedAttributeValue {
            value,
            quote_style,
            span: lit.span(),
        })
    }

    /// Generate tokens for a validated attribute
    /// This method generates the appropriate tokens for the attribute based on its type
    /// Note: For static content, this is handled by the NodeToTokens implementation
    pub(crate) fn generate_attribute_tokens(
        &self,
        name: &ValidatedAttributeName,
        value: &ValidatedAttributeValue,
    ) -> TokenStream {
        let attr_name = &name.name;

        match value {
            ValidatedAttributeValue::BooleanPresent => {
                // Static boolean attributes are handled by NodeToTokens
                quote! {}
            }
            ValidatedAttributeValue::QuotedString(_) => {
                // Static string attributes are handled by NodeToTokens
                quote! {}
            }
            ValidatedAttributeValue::Dynamic(block) => {
                quote! {
                    __dynamic.push_fragment(format!("{}", #block));
                }
            }
            ValidatedAttributeValue::DynamicBoolean(block) => {
                let attr_name_str = attr_name;
                quote! {
                    if #block {
                        __dynamic.push_fragment(format!(" {}", #attr_name_str));
                    }
                }
            }
            ValidatedAttributeValue::EventHandler(block) => {
                if name.attr_type == AttributeType::AxmEvent {
                    quote! {
                        __dynamic.push_message(#block);
                    }
                } else {
                    quote! {
                        __dynamic.push_event_handler(#block);
                    }
                }
            }
        }
    }
}

/// A validated attribute name with type information
#[derive(Debug, Clone)]
pub(crate) struct ValidatedAttributeName {
    pub name: String,
    pub attr_type: AttributeType,
    pub span: Span,
}

/// Types of HTML attributes requiring different validation and handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AttributeType {
    /// Standard HTML attributes (class, id, style, etc.)
    Standard,
    /// Boolean attributes (checked, disabled, hidden, etc.)
    Boolean,
    /// Data attributes (data-*)
    Data,
    /// ARIA attributes (aria-*)
    Aria,
    /// Event attributes (onclick, onchange, etc.)
    Event,
    /// Axm event attributes (axm-click, axm-input, etc.)
    AxmEvent,
}

/// Raw attribute value as parsed from the macro
#[derive(Debug, Clone)]
pub(crate) enum AttributeValue {
    /// String literal value
    LitStr(LitStr),
    /// Block expression value
    Block(Block),
    /// Unit value () for boolean attributes
    Unit,
    /// No value provided
    None,
}

impl AttributeValue {
    fn span(&self) -> Span {
        match self {
            AttributeValue::LitStr(lit) => lit.span(),
            AttributeValue::Block(block) => block.span(),
            AttributeValue::Unit => Span::call_site(),
            AttributeValue::None => Span::call_site(),
        }
    }
}

/// A validated and processed attribute value
#[derive(Debug, Clone)]
pub(crate) enum ValidatedAttributeValue {
    /// Boolean attribute that is present
    BooleanPresent,
    /// String value with proper quotes
    QuotedString(QuotedAttributeValue),
    /// Dynamic value from a block expression
    Dynamic(Block),
    /// Dynamic boolean value from a block expression
    DynamicBoolean(Block),
    /// Event handler from a block expression
    EventHandler(Block),
}

/// A string attribute value with proper quote handling
#[derive(Debug, Clone)]
pub(crate) struct QuotedAttributeValue {
    pub value: String,
    pub quote_style: QuoteStyle,
    pub span: Span,
}

/// Quote style for attribute values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QuoteStyle {
    /// Double quotes (")
    Double,
    /// Single quotes (')
    Single,
}

impl QuoteStyle {
    fn as_char(self) -> char {
        match self {
            QuoteStyle::Double => '"',
            QuoteStyle::Single => '\'',
        }
    }
}

/// Determine the appropriate quote style for an attribute value
fn determine_quote_style(value: &str) -> QuoteStyle {
    // Use single quotes if the value contains double quotes but no single quotes
    if value.contains('"') && !value.contains('\'') {
        QuoteStyle::Single
    } else {
        // Default to double quotes for consistency
        QuoteStyle::Double
    }
}

/// Check if an attribute name is an event attribute
fn is_event_attribute(name: &str) -> bool {
    // Standard HTML event attributes start with "on"
    name.starts_with("on") && name.len() > 2
}

/// A complete validated attribute ready for code generation
#[derive(Debug, Clone)]
pub(crate) struct ValidatedAttribute {
    pub name: ValidatedAttributeName,
    pub value: ValidatedAttributeValue,
}

impl ValidatedAttribute {
    /// Create a new validated attribute
    pub(crate) fn new(
        processor: &AttributeProcessor,
        name: &str,
        name_span: Span,
        value: AttributeValue,
    ) -> syn::Result<Self> {
        let validated_name = processor.validate_attribute_name(name, name_span)?;
        let validated_value = processor.validate_attribute_value(&validated_name, &value)?;
        
        Ok(Self {
            name: validated_name,
            value: validated_value,
        })
    }

    /// Generate tokens for this attribute
    pub(crate) fn to_tokens(&self, processor: &AttributeProcessor) -> TokenStream {
        processor.generate_attribute_tokens(&self.name, &self.value)
    }
}

impl fmt::Display for ValidatedAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            ValidatedAttributeValue::BooleanPresent => write!(f, "{}", self.name.name),
            ValidatedAttributeValue::QuotedString(quoted) => {
                let quote_char = quoted.quote_style.as_char();
                write!(f, "{}={}{}{}", self.name.name, quote_char, quoted.value, quote_char)
            }
            ValidatedAttributeValue::Dynamic(_) => {
                write!(f, "{}={{dynamic}}", self.name.name)
            }
            ValidatedAttributeValue::DynamicBoolean(_) => {
                write!(f, "{}={{boolean}}", self.name.name)
            }
            ValidatedAttributeValue::EventHandler(_) => {
                write!(f, "{}={{handler}}", self.name.name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_attribute_name_validation() {
        let processor = AttributeProcessor::new();
        
        // Valid attribute names
        assert!(processor.validate_attribute_name("class", Span::call_site()).is_ok());
        assert!(processor.validate_attribute_name("data-value", Span::call_site()).is_ok());
        assert!(processor.validate_attribute_name("aria-label", Span::call_site()).is_ok());
        assert!(processor.validate_attribute_name("onclick", Span::call_site()).is_ok());
        
        // Invalid attribute names
        assert!(processor.validate_attribute_name("", Span::call_site()).is_err());
        assert!(processor.validate_attribute_name("class name", Span::call_site()).is_err());
        assert!(processor.validate_attribute_name("class=", Span::call_site()).is_err());
    }

    #[test]
    fn test_boolean_attribute_handling() {
        let processor = AttributeProcessor::new();
        let attr_name = processor.validate_attribute_name("checked", Span::call_site()).unwrap();
        
        // Boolean attribute with no value should be valid
        let result = processor.validate_attribute_value(&attr_name, &AttributeValue::None);
        assert!(result.is_ok());
        matches!(result.unwrap(), ValidatedAttributeValue::BooleanPresent);
        
        // Boolean attribute with unit value should be valid
        let result = processor.validate_attribute_value(&attr_name, &AttributeValue::Unit);
        assert!(result.is_ok());
        matches!(result.unwrap(), ValidatedAttributeValue::BooleanPresent);
    }

    #[test]
    fn test_quote_style_determination() {
        assert_eq!(determine_quote_style("simple"), QuoteStyle::Double);
        assert_eq!(determine_quote_style("has \"quotes\""), QuoteStyle::Single);
        assert_eq!(determine_quote_style("has 'apostrophe'"), QuoteStyle::Double);
        assert_eq!(determine_quote_style("has \"both\" 'types'"), QuoteStyle::Double);
    }

    #[test]
    fn test_event_attribute_detection() {
        assert!(is_event_attribute("onclick"));
        assert!(is_event_attribute("onchange"));
        assert!(is_event_attribute("onmouseover"));
        assert!(!is_event_attribute("on"));
        assert!(!is_event_attribute("once"));
        assert!(!is_event_attribute("class"));
    }

    #[test]
    fn test_attribute_type_classification() {
        let processor = AttributeProcessor::new();
        
        let boolean_attr = processor.validate_attribute_name("checked", Span::call_site()).unwrap();
        assert_eq!(boolean_attr.attr_type, AttributeType::Boolean);
        
        let data_attr = processor.validate_attribute_name("data-value", Span::call_site()).unwrap();
        assert_eq!(data_attr.attr_type, AttributeType::Data);
        
        let aria_attr = processor.validate_attribute_name("aria-label", Span::call_site()).unwrap();
        assert_eq!(aria_attr.attr_type, AttributeType::Aria);
        
        let event_attr = processor.validate_attribute_name("onclick", Span::call_site()).unwrap();
        assert_eq!(event_attr.attr_type, AttributeType::Event);
        
        let axm_attr = processor.validate_attribute_name("axm-click", Span::call_site()).unwrap();
        assert_eq!(axm_attr.attr_type, AttributeType::AxmEvent);
        
        let standard_attr = processor.validate_attribute_name("class", Span::call_site()).unwrap();
        assert_eq!(standard_attr.attr_type, AttributeType::Standard);
    }

    #[test]
    fn test_validated_attribute_creation() {
        let processor = AttributeProcessor::new();
        
        // Create a boolean attribute
        let boolean_attr = ValidatedAttribute::new(
            &processor,
            "checked",
            Span::call_site(),
            AttributeValue::None,
        ).unwrap();
        
        assert_eq!(boolean_attr.name.name, "checked");
        assert_eq!(boolean_attr.name.attr_type, AttributeType::Boolean);
        matches!(boolean_attr.value, ValidatedAttributeValue::BooleanPresent);
        
        // Create a string attribute
        let lit_str: LitStr = parse_quote!("test-value");
        let string_attr = ValidatedAttribute::new(
            &processor,
            "class",
            Span::call_site(),
            AttributeValue::LitStr(lit_str),
        ).unwrap();
        
        assert_eq!(string_attr.name.name, "class");
        assert_eq!(string_attr.name.attr_type, AttributeType::Standard);
        matches!(string_attr.value, ValidatedAttributeValue::QuotedString(_));
    }
}