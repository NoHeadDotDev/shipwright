//! HTML5 validation module using html5ever for robust HTML parsing and validation.

use html5ever::{namespace_url, ns, QualName, LocalName};
use std::collections::HashSet;

/// HTML5 validation utilities
#[derive(Debug)]
pub(crate) struct Html5Validator {
    /// Set of void elements that cannot have closing tags
    void_elements: HashSet<&'static str>,
    /// Set of boolean attributes
    boolean_attributes: HashSet<&'static str>,
}

impl Default for Html5Validator {
    fn default() -> Self {
        Self::new()
    }
}

impl Html5Validator {
    /// Create a new HTML5 validator with standard HTML5 rules
    pub(crate) fn new() -> Self {
        let mut void_elements = HashSet::new();
        void_elements.insert("area");
        void_elements.insert("base");
        void_elements.insert("br");
        void_elements.insert("col");
        void_elements.insert("embed");
        void_elements.insert("hr");
        void_elements.insert("img");
        void_elements.insert("input");
        void_elements.insert("link");
        void_elements.insert("meta");
        void_elements.insert("param");
        void_elements.insert("source");
        void_elements.insert("track");
        void_elements.insert("wbr");

        let mut boolean_attributes = HashSet::new();
        boolean_attributes.insert("autofocus");
        boolean_attributes.insert("autoplay");
        boolean_attributes.insert("checked");
        boolean_attributes.insert("controls");
        boolean_attributes.insert("defer");
        boolean_attributes.insert("disabled");
        boolean_attributes.insert("hidden");
        boolean_attributes.insert("loop");
        boolean_attributes.insert("multiple");
        boolean_attributes.insert("muted");
        boolean_attributes.insert("open");
        boolean_attributes.insert("readonly");
        boolean_attributes.insert("required");
        boolean_attributes.insert("reversed");
        boolean_attributes.insert("selected");

        Self {
            void_elements,
            boolean_attributes,
        }
    }

    /// Check if an element is a void element (self-closing)
    pub(crate) fn is_void_element(&self, tag_name: &str) -> bool {
        self.void_elements.contains(tag_name)
    }

    /// Check if an attribute is a boolean attribute
    pub(crate) fn is_boolean_attribute(&self, attr_name: &str) -> bool {
        self.boolean_attributes.contains(attr_name)
    }

    /// Validate an element name according to HTML5 rules
    pub(crate) fn validate_element_name(&self, name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::EmptyElementName);
        }

        // Check first character
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() {
            return Err(ValidationError::InvalidElementNameStart(first_char));
        }

        // Check remaining characters
        for (i, c) in name.chars().enumerate() {
            if i == 0 {
                continue;
            }
            if !c.is_ascii_alphanumeric() && c != '-' && c != '_' && c != '.' {
                return Err(ValidationError::InvalidElementNameChar(c, i));
            }
        }

        Ok(())
    }

    /// Validate an attribute name according to HTML5 rules
    pub(crate) fn validate_attribute_name(&self, name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::EmptyAttributeName);
        }

        // HTML5 attribute names are more permissive than element names
        for (i, c) in name.chars().enumerate() {
            if c.is_ascii_control() || c == ' ' || c == '"' || c == '\'' || c == '>' || c == '/' || c == '=' {
                return Err(ValidationError::InvalidAttributeNameChar(c, i));
            }
        }

        Ok(())
    }

    /// Create a qualified name for html5ever integration
    #[allow(dead_code)]
    pub(crate) fn create_qualified_name(&self, local_name: &str, namespace: Option<&str>) -> QualName {
        let ns = match namespace {
            Some("svg") => ns!(svg),
            Some("mathml") => ns!(mathml),
            _ => ns!(html),
        };

        QualName::new(None, ns, LocalName::from(local_name))
    }
}

/// Validation errors for HTML5 content
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ValidationError {
    /// Element name is empty
    EmptyElementName,
    /// Element name starts with invalid character
    InvalidElementNameStart(char),
    /// Element name contains invalid character at position
    InvalidElementNameChar(char, usize),
    /// Attribute name is empty
    EmptyAttributeName,
    /// Attribute name contains invalid character at position
    InvalidAttributeNameChar(char, usize),
    /// Void element cannot have children
    VoidElementWithChildren(String),
    /// Element nesting violation
    InvalidNesting {
        parent: String,
        child: String,
        reason: String,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyElementName => {
                write!(f, "Element name cannot be empty")
            }
            ValidationError::InvalidElementNameStart(c) => {
                write!(f, "Element name cannot start with '{}'", c)
            }
            ValidationError::InvalidElementNameChar(c, pos) => {
                write!(f, "Invalid character '{}' at position {} in element name", c, pos)
            }
            ValidationError::EmptyAttributeName => {
                write!(f, "Attribute name cannot be empty")
            }
            ValidationError::InvalidAttributeNameChar(c, pos) => {
                write!(f, "Invalid character '{}' at position {} in attribute name", c, pos)
            }
            ValidationError::VoidElementWithChildren(element) => {
                write!(f, "Void element '{}' cannot have children", element)
            }
            ValidationError::InvalidNesting { parent, child, reason } => {
                write!(f, "Invalid nesting: '{}' cannot contain '{}' ({})", parent, child, reason)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_void_elements() {
        let validator = Html5Validator::new();
        assert!(validator.is_void_element("br"));
        assert!(validator.is_void_element("img"));
        assert!(validator.is_void_element("input"));
        assert!(!validator.is_void_element("div"));
        assert!(!validator.is_void_element("span"));
    }

    #[test]
    fn test_boolean_attributes() {
        let validator = Html5Validator::new();
        assert!(validator.is_boolean_attribute("checked"));
        assert!(validator.is_boolean_attribute("disabled"));
        assert!(validator.is_boolean_attribute("hidden"));
        assert!(!validator.is_boolean_attribute("id"));
        assert!(!validator.is_boolean_attribute("class"));
    }

    #[test]
    fn test_element_name_validation() {
        let validator = Html5Validator::new();
        
        // Valid names
        assert!(validator.validate_element_name("div").is_ok());
        assert!(validator.validate_element_name("custom-element").is_ok());
        assert!(validator.validate_element_name("my_component").is_ok());
        
        // Invalid names
        assert!(validator.validate_element_name("").is_err());
        assert!(validator.validate_element_name("123div").is_err());
        assert!(validator.validate_element_name("div@").is_err());
    }

    #[test]
    fn test_attribute_name_validation() {
        let validator = Html5Validator::new();
        
        // Valid names
        assert!(validator.validate_attribute_name("class").is_ok());
        assert!(validator.validate_attribute_name("data-value").is_ok());
        assert!(validator.validate_attribute_name("aria-label").is_ok());
        
        // Invalid names
        assert!(validator.validate_attribute_name("").is_err());
        assert!(validator.validate_attribute_name("class name").is_err());
        assert!(validator.validate_attribute_name("class\"").is_err());
        assert!(validator.validate_attribute_name("class=").is_err());
    }
}