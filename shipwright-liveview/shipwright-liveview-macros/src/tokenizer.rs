//! HTML tokenizer validation module using html5ever for compile-time HTML syntax validation.
//!
//! This module provides comprehensive HTML tokenization and validation capabilities
//! at compile time, ensuring that generated HTML is syntactically correct and follows
//! HTML5 standards.

use html5ever::{
    tokenizer::{
        BufferQueue, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
    },
    tendril::{StrTendril, TendrilSink},
    Attribute, QualName, LocalName,
};
use std::collections::HashMap;
use crate::html5_validation::{Html5Validator, ValidationError};

/// Compile-time HTML tokenizer that validates syntax and structure
pub(crate) struct HtmlTokenizer {
    validator: Html5Validator,
    tokenizer: Option<Tokenizer<TokenSinkState>>,
    errors: Vec<TokenizationError>,
}

/// State for tracking tokenization and validation
#[derive(Debug)]
struct TokenSinkState {
    stack: Vec<String>,
    errors: Vec<TokenizationError>,
    validator: Html5Validator,
}

/// Errors that can occur during HTML tokenization and validation
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenizationError {
    /// Basic validation error from the html5_validation module
    Validation(ValidationError),
    /// Invalid HTML structure
    InvalidStructure(String),
    /// Mismatched tags
    MismatchedTags { open: String, close: String },
    /// Unclosed tags
    UnclosedTags(Vec<String>),
    /// Void element with children
    VoidElementWithChildren(String),
    /// Invalid character in tag or attribute name
    InvalidCharacter { context: String, character: char, position: usize },
    /// Malformed HTML entity
    MalformedEntity(String),
    /// Invalid attribute value
    InvalidAttributeValue { attr_name: String, value: String, reason: String },
    /// Generic tokenization error
    TokenizationFailed(String),
}

impl std::fmt::Display for TokenizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenizationError::Validation(e) => write!(f, "Validation error: {}", e),
            TokenizationError::InvalidStructure(msg) => write!(f, "Invalid HTML structure: {}", msg),
            TokenizationError::MismatchedTags { open, close } => {
                write!(f, "Mismatched tags: opened '{}' but closed '{}'", open, close)
            }
            TokenizationError::UnclosedTags(tags) => {
                write!(f, "Unclosed tags: {}", tags.join(", "))
            }
            TokenizationError::VoidElementWithChildren(tag) => {
                write!(f, "Void element '{}' cannot have children", tag)
            }
            TokenizationError::InvalidCharacter { context, character, position } => {
                write!(f, "Invalid character '{}' at position {} in {}", character, position, context)
            }
            TokenizationError::MalformedEntity(entity) => {
                write!(f, "Malformed HTML entity: {}", entity)
            }
            TokenizationError::InvalidAttributeValue { attr_name, value, reason } => {
                write!(f, "Invalid value '{}' for attribute '{}': {}", value, attr_name, reason)
            }
            TokenizationError::TokenizationFailed(msg) => {
                write!(f, "Tokenization failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for TokenizationError {}

impl From<ValidationError> for TokenizationError {
    fn from(err: ValidationError) -> Self {
        TokenizationError::Validation(err)
    }
}

impl TokenSink for TokenSinkState {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        match token {
            Token::TagToken(tag) => {
                let tag_name = tag.name.to_string();
                
                // Validate tag name
                if let Err(e) = self.validator.validate_element_name(&tag_name) {
                    self.errors.push(TokenizationError::Validation(e));
                }

                // Validate attributes
                for attr in &tag.attrs {
                    let attr_name = attr.name.local.to_string();
                    if let Err(e) = self.validator.validate_attribute_name(&attr_name) {
                        self.errors.push(TokenizationError::Validation(e));
                    }
                    
                    // Validate attribute value
                    self.validate_attribute_value(&attr_name, &attr.value);
                }

                match tag.kind {
                    html5ever::tokenizer::TagKind::StartTag => {
                        // Check for void elements
                        if self.validator.is_void_element(&tag_name) && !tag.self_closing {
                            // This is OK for void elements - they don't need explicit self-closing
                        }
                        
                        if !tag.self_closing && !self.validator.is_void_element(&tag_name) {
                            self.stack.push(tag_name);
                        }
                    }
                    html5ever::tokenizer::TagKind::EndTag => {
                        if self.validator.is_void_element(&tag_name) {
                            self.errors.push(TokenizationError::InvalidStructure(
                                format!("Void element '{}' cannot have a closing tag", tag_name)
                            ));
                        } else if let Some(open_tag) = self.stack.pop() {
                            if open_tag != tag_name {
                                self.errors.push(TokenizationError::MismatchedTags {
                                    open: open_tag,
                                    close: tag_name,
                                });
                            }
                        } else {
                            self.errors.push(TokenizationError::InvalidStructure(
                                format!("Closing tag '{}' has no matching opening tag", tag_name)
                            ));
                        }
                    }
                }
            }
            Token::CharacterTokens(chars) => {
                // Validate entities in character data
                self.validate_entities(&chars);
            }
            Token::ParseError(err) => {
                self.errors.push(TokenizationError::TokenizationFailed(
                    format!("Parse error: {}", err)
                ));
            }
            Token::CommentToken(_) => {
                // Comments are valid, no validation needed
            }
            Token::DoctypeToken(doctype) => {
                // Basic doctype validation
                if doctype.name.is_none() || doctype.name.as_ref() != Some(&StrTendril::from("html")) {
                    self.errors.push(TokenizationError::InvalidStructure(
                        "Invalid or missing DOCTYPE declaration".to_string()
                    ));
                }
            }
            Token::NullCharacterToken => {
                self.errors.push(TokenizationError::InvalidCharacter {
                    context: "character data".to_string(),
                    character: '\0',
                    position: 0,
                });
            }
            Token::EOFToken => {
                // Check for unclosed tags at EOF
                if !self.stack.is_empty() {
                    self.errors.push(TokenizationError::UnclosedTags(self.stack.clone()));
                }
            }
        }
        
        TokenSinkResult::Continue
    }
}

impl TokenSinkState {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            errors: Vec::new(),
            validator: Html5Validator::new(),
        }
    }

    /// Validate HTML entities in character data
    fn validate_entities(&mut self, text: &str) {
        let mut chars = text.char_indices().peekable();
        
        while let Some((pos, ch)) = chars.next() {
            if ch == '&' {
                // Start of a potential entity
                let entity_start = pos;
                let mut entity = String::new();
                entity.push(ch);
                
                // Collect the entity
                let mut found_semicolon = false;
                while let Some((_, entity_ch)) = chars.peek() {
                    let entity_ch = *entity_ch;
                    chars.next(); // consume the character
                    entity.push(entity_ch);
                    
                    if entity_ch == ';' {
                        found_semicolon = true;
                        break;
                    }
                    
                    // Stop if we hit a space or other delimiter
                    if entity_ch.is_whitespace() || entity.len() > 10 {
                        break;
                    }
                }
                
                // Validate the entity
                if found_semicolon {
                    if !self.is_valid_entity(&entity) {
                        self.errors.push(TokenizationError::MalformedEntity(entity));
                    }
                } else if entity.len() > 1 && entity != "&" {
                    // Looks like an incomplete entity
                    self.errors.push(TokenizationError::MalformedEntity(entity));
                }
            }
        }
    }

    /// Check if an HTML entity is valid
    fn is_valid_entity(&self, entity: &str) -> bool {
        // Basic check for common HTML entities
        matches!(entity,
            "&amp;" | "&lt;" | "&gt;" | "&quot;" | "&apos;" |
            "&nbsp;" | "&copy;" | "&reg;" | "&trade;" |
            "&hellip;" | "&mdash;" | "&ndash;" | "&laquo;" | "&raquo;"
        ) || entity.starts_with("&#") && entity.ends_with(';') && {
            // Numeric character reference
            let inner = &entity[2..entity.len()-1];
            if inner.starts_with('x') || inner.starts_with('X') {
                // Hexadecimal
                inner[1..].chars().all(|c| c.is_ascii_hexdigit())
            } else {
                // Decimal
                inner.chars().all(|c| c.is_ascii_digit())
            }
        }
    }

    /// Validate attribute values based on attribute name and context
    fn validate_attribute_value(&mut self, attr_name: &str, value: &StrTendril) {
        let value_str = value.to_string();
        
        match attr_name {
            "id" => {
                // ID must be unique and follow specific rules
                if value_str.is_empty() {
                    self.errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value_str,
                        reason: "ID cannot be empty".to_string(),
                    });
                } else if value_str.contains(char::is_whitespace) {
                    self.errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value_str,
                        reason: "ID cannot contain whitespace".to_string(),
                    });
                }
            }
            "class" => {
                // Class names should not be empty
                if value_str.trim().is_empty() {
                    self.errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value_str,
                        reason: "Class attribute cannot be empty or only whitespace".to_string(),
                    });
                }
            }
            attr if attr.starts_with("data-") => {
                // Data attributes have specific naming rules
                let data_name = &attr[5..]; // Remove "data-" prefix
                if data_name.is_empty() {
                    self.errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value_str,
                        reason: "Data attribute name cannot be empty after 'data-'".to_string(),
                    });
                } else if data_name.contains(char::is_uppercase) {
                    self.errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value_str,
                        reason: "Data attribute names must be lowercase".to_string(),
                    });
                }
            }
            "href" | "src" | "action" => {
                // URL attributes should not be empty if present
                if value_str.trim().is_empty() {
                    self.errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value_str,
                        reason: format!("{} attribute should not be empty", attr_name),
                    });
                }
            }
            _ => {
                // For boolean attributes, the value should match the attribute name or be empty
                if self.validator.is_boolean_attribute(attr_name) && !value_str.is_empty() && value_str != attr_name {
                    self.errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value_str,
                        reason: format!("Boolean attribute should be empty or match attribute name '{}'", attr_name),
                    });
                }
            }
        }
    }
}

impl HtmlTokenizer {
    /// Create a new HTML tokenizer with validation
    pub(crate) fn new() -> Self {
        Self {
            validator: Html5Validator::new(),
            tokenizer: None,
            errors: Vec::new(),
        }
    }

    /// Tokenize and validate HTML content at compile time
    pub(crate) fn tokenize_and_validate(&mut self, html: &str) -> Result<(), Vec<TokenizationError>> {
        self.errors.clear();
        
        let mut sink = TokenSinkState::new();
        let opts = TokenizerOpts::default();
        let mut tokenizer = Tokenizer::new(sink, opts);
        
        let mut input = BufferQueue::new();
        input.push_back(StrTendril::from(html));
        
        // Process the input
        while !input.is_empty() {
            let _ = tokenizer.feed(&mut input);
        }
        
        // Finalize tokenization
        tokenizer.end();
        
        // Extract errors from the sink
        let sink = tokenizer.sink;
        self.errors.extend(sink.errors);
        
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Validate a single HTML tag and its attributes
    pub(crate) fn validate_tag(&self, tag_name: &str, attributes: &[(String, Option<String>)]) -> Result<(), Vec<TokenizationError>> {
        let mut errors = Vec::new();
        
        // Validate tag name
        if let Err(e) = self.validator.validate_element_name(tag_name) {
            errors.push(TokenizationError::Validation(e));
        }
        
        // Validate attributes
        for (attr_name, attr_value) in attributes {
            if let Err(e) = self.validator.validate_attribute_name(attr_name) {
                errors.push(TokenizationError::Validation(e));
            }
            
            // Additional attribute validation
            if let Some(value) = attr_value {
                self.validate_attribute_value_simple(attr_name, value, &mut errors);
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Simple attribute value validation (used for single tag validation)
    fn validate_attribute_value_simple(&self, attr_name: &str, value: &str, errors: &mut Vec<TokenizationError>) {
        match attr_name {
            "id" => {
                if value.is_empty() {
                    errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value.to_string(),
                        reason: "ID cannot be empty".to_string(),
                    });
                } else if value.contains(char::is_whitespace) {
                    errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value.to_string(),
                        reason: "ID cannot contain whitespace".to_string(),
                    });
                }
            }
            "class" => {
                if value.trim().is_empty() {
                    errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value.to_string(),
                        reason: "Class attribute cannot be empty or only whitespace".to_string(),
                    });
                }
            }
            attr if attr.starts_with("data-") => {
                let data_name = &attr[5..];
                if data_name.is_empty() {
                    errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value.to_string(),
                        reason: "Data attribute name cannot be empty after 'data-'".to_string(),
                    });
                } else if data_name.contains(char::is_uppercase) {
                    errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value.to_string(),
                        reason: "Data attribute names must be lowercase".to_string(),
                    });
                }
            }
            _ => {
                if self.validator.is_boolean_attribute(attr_name) && !value.is_empty() && value != attr_name {
                    errors.push(TokenizationError::InvalidAttributeValue {
                        attr_name: attr_name.to_string(),
                        value: value.to_string(),
                        reason: format!("Boolean attribute should be empty or match attribute name '{}'", attr_name),
                    });
                }
            }
        }
    }

    /// Get the current validation errors
    pub(crate) fn errors(&self) -> &[TokenizationError] {
        &self.errors
    }

    /// Check if the tokenizer has any errors
    pub(crate) fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Create a detailed error report
    pub(crate) fn error_report(&self) -> String {
        if self.errors.is_empty() {
            return "No errors found.".to_string();
        }
        
        let mut report = String::new();
        report.push_str(&format!("Found {} validation error(s):\n", self.errors.len()));
        
        for (i, error) in self.errors.iter().enumerate() {
            report.push_str(&format!("  {}. {}\n", i + 1, error));
        }
        
        report
    }
}

impl Default for HtmlTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_html() {
        let mut tokenizer = HtmlTokenizer::new();
        let html = "<div class=\"test\"><p>Hello</p></div>";
        
        assert!(tokenizer.tokenize_and_validate(html).is_ok());
    }

    #[test]
    fn test_mismatched_tags() {
        let mut tokenizer = HtmlTokenizer::new();
        let html = "<div><p>Hello</div></p>";
        
        let result = tokenizer.tokenize_and_validate(html);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, TokenizationError::MismatchedTags { .. })));
    }

    #[test]
    fn test_unclosed_tags() {
        let mut tokenizer = HtmlTokenizer::new();
        let html = "<div><p>Hello";
        
        let result = tokenizer.tokenize_and_validate(html);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, TokenizationError::UnclosedTags(_))));
    }

    #[test]
    fn test_void_element_validation() {
        let mut tokenizer = HtmlTokenizer::new();
        
        // Valid self-closing void element
        let html1 = "<br />";
        assert!(tokenizer.tokenize_and_validate(html1).is_ok());
        
        // Valid void element without self-closing
        let html2 = "<br>";
        assert!(tokenizer.tokenize_and_validate(html2).is_ok());
        
        // Invalid: void element with closing tag
        let html3 = "<br></br>";
        let result = tokenizer.tokenize_and_validate(html3);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_tag_names() {
        let tokenizer = HtmlTokenizer::new();
        
        // Test invalid tag names
        let result1 = tokenizer.validate_tag("123invalid", &[]);
        assert!(result1.is_err());
        
        let result2 = tokenizer.validate_tag("div@", &[]);
        assert!(result2.is_err());
        
        // Test valid tag names
        let result3 = tokenizer.validate_tag("div", &[]);
        assert!(result3.is_ok());
        
        let result4 = tokenizer.validate_tag("custom-element", &[]);
        assert!(result4.is_ok());
    }

    #[test]
    fn test_attribute_validation() {
        let tokenizer = HtmlTokenizer::new();
        
        // Valid attributes
        let result1 = tokenizer.validate_tag("div", &[
            ("class".to_string(), Some("test".to_string())),
            ("id".to_string(), Some("myid".to_string())),
        ]);
        assert!(result1.is_ok());
        
        // Invalid ID with whitespace
        let result2 = tokenizer.validate_tag("div", &[
            ("id".to_string(), Some("my id".to_string())),
        ]);
        assert!(result2.is_err());
        
        // Invalid empty class
        let result3 = tokenizer.validate_tag("div", &[
            ("class".to_string(), Some("   ".to_string())),
        ]);
        assert!(result3.is_err());
    }

    #[test]
    fn test_html_entities() {
        let mut tokenizer = HtmlTokenizer::new();
        
        // Valid entities
        let html1 = "<p>&amp; &lt; &gt; &quot;</p>";
        assert!(tokenizer.tokenize_and_validate(html1).is_ok());
        
        // Valid numeric entities
        let html2 = "<p>&#65; &#x41;</p>";
        assert!(tokenizer.tokenize_and_validate(html2).is_ok());
        
        // Invalid entity
        let html3 = "<p>&invalidentity;</p>";
        let result = tokenizer.tokenize_and_validate(html3);
        assert!(result.is_err());
    }

    #[test]
    fn test_boolean_attributes() {
        let tokenizer = HtmlTokenizer::new();
        
        // Valid boolean attribute
        let result1 = tokenizer.validate_tag("input", &[
            ("checked".to_string(), None),
        ]);
        assert!(result1.is_ok());
        
        // Valid boolean attribute with matching value
        let result2 = tokenizer.validate_tag("input", &[
            ("checked".to_string(), Some("checked".to_string())),
        ]);
        assert!(result2.is_ok());
        
        // Invalid boolean attribute with wrong value
        let result3 = tokenizer.validate_tag("input", &[
            ("checked".to_string(), Some("true".to_string())),
        ]);
        assert!(result3.is_err());
    }

    #[test]
    fn test_data_attributes() {
        let tokenizer = HtmlTokenizer::new();
        
        // Valid data attribute
        let result1 = tokenizer.validate_tag("div", &[
            ("data-value".to_string(), Some("test".to_string())),
        ]);
        assert!(result1.is_ok());
        
        // Invalid data attribute with uppercase
        let result2 = tokenizer.validate_tag("div", &[
            ("data-Value".to_string(), Some("test".to_string())),
        ]);
        assert!(result2.is_err());
        
        // Invalid empty data attribute name
        let result3 = tokenizer.validate_tag("div", &[
            ("data-".to_string(), Some("test".to_string())),
        ]);
        assert!(result3.is_err());
    }
}