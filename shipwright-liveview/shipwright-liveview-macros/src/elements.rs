//! HTML element structure validation and content model enforcement.
//!
//! This module provides functionality for validating HTML element structures,
//! including void element handling and content model validation for proper nesting.

use std::collections::{HashMap, HashSet};

/// Validation errors for element structure
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ValidationError {
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

/// Content model categories for HTML5 elements
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ContentCategory {
    /// Metadata content (head elements)
    Metadata,
    /// Flow content (most body elements)
    Flow,
    /// Sectioning content (article, section, etc.)
    Sectioning,
    /// Heading content (h1-h6, hgroup)
    Heading,
    /// Phrasing content (inline elements)
    Phrasing,
    /// Embedded content (img, video, etc.)
    Embedded,
    /// Interactive content (form controls, links)
    Interactive,
    /// Script-supporting elements
    ScriptSupporting,
    /// Table content
    Table,
    /// Form-associated content
    FormAssociated,
}

/// Content models define what kinds of content an element can contain
#[derive(Debug, Clone)]
pub(crate) struct ContentModel {
    /// Categories of content this element accepts
    pub accepts: Vec<ContentCategory>,
    /// Specific elements that are allowed as children
    pub allowed_children: HashSet<String>,
    /// Specific elements that are forbidden as children
    pub forbidden_children: HashSet<String>,
    /// Whether this element is transparent (inherits parent's content model)
    pub is_transparent: bool,
}

impl Default for ContentModel {
    fn default() -> Self {
        Self {
            accepts: vec![ContentCategory::Flow, ContentCategory::Phrasing],
            allowed_children: HashSet::new(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        }
    }
}

/// Element structure validator that enforces HTML5 content models and void element rules
pub(crate) struct ElementStructureValidator {
    /// Set of void elements that cannot have closing tags
    void_elements: HashSet<&'static str>,
    /// Content models for elements
    content_models: HashMap<String, ContentModel>,
    /// Element categories mapping
    element_categories: HashMap<String, Vec<ContentCategory>>,
}

impl Default for ElementStructureValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementStructureValidator {
    /// Create a new element structure validator with HTML5 rules
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

        let mut content_models = HashMap::new();
        let mut element_categories = HashMap::new();

        // Initialize content models and categories
        Self::init_content_models(&mut content_models);
        Self::init_element_categories(&mut element_categories);

        Self {
            void_elements,
            content_models,
            element_categories,
        }
    }

    /// Check if an element is a void element (cannot have children)
    pub(crate) fn is_void_element(&self, tag_name: &str) -> bool {
        self.void_elements.contains(tag_name)
    }

    /// Validate that a void element doesn't have children
    pub(crate) fn validate_void_element(&self, tag_name: &str, has_children: bool) -> Result<(), ValidationError> {
        if self.is_void_element(tag_name) && has_children {
            return Err(ValidationError::VoidElementWithChildren(tag_name.to_string()));
        }
        Ok(())
    }

    /// Validate element nesting according to HTML5 content models
    pub(crate) fn validate_nesting(&self, parent: &str, child: &str) -> Result<(), ValidationError> {
        // Get content model for parent
        let parent_model = self.content_models.get(parent)
            .cloned()
            .unwrap_or_default();

        // Check if child is explicitly forbidden
        if parent_model.forbidden_children.contains(child) {
            return Err(ValidationError::InvalidNesting {
                parent: parent.to_string(),
                child: child.to_string(),
                reason: "explicitly forbidden".to_string(),
            });
        }

        // Check if child is explicitly allowed
        if parent_model.allowed_children.contains(child) {
            return Ok(());
        }

        // Check content categories
        if let Some(child_categories) = self.element_categories.get(child) {
            for child_category in child_categories {
                if parent_model.accepts.contains(child_category) {
                    return Ok(());
                }
            }
        }

        // Special cases and specific validation rules
        self.validate_specific_nesting_rules(parent, child)
    }

    /// Validate specific nesting rules that don't fit the general content model
    fn validate_specific_nesting_rules(&self, parent: &str, child: &str) -> Result<(), ValidationError> {
        match (parent, child) {
            // Interactive elements cannot contain other interactive elements
            (p, c) if self.is_interactive_element(p) && self.is_interactive_element(c) => {
                Err(ValidationError::InvalidNesting {
                    parent: parent.to_string(),
                    child: child.to_string(),
                    reason: "interactive elements cannot contain other interactive elements".to_string(),
                })
            }
            
            // Form elements cannot contain form elements
            ("form", "form") => {
                Err(ValidationError::InvalidNesting {
                    parent: parent.to_string(),
                    child: child.to_string(),
                    reason: "form elements cannot be nested".to_string(),
                })
            }

            // Label cannot contain other label elements
            ("label", "label") => {
                Err(ValidationError::InvalidNesting {
                    parent: parent.to_string(),
                    child: child.to_string(),
                    reason: "label elements cannot be nested".to_string(),
                })
            }

            // Button cannot contain interactive content
            ("button", c) if self.is_interactive_element(c) => {
                Err(ValidationError::InvalidNesting {
                    parent: parent.to_string(),
                    child: child.to_string(),
                    reason: "button cannot contain interactive elements".to_string(),
                })
            }

            // Paragraph cannot contain flow content (only phrasing content)
            ("p", c) if self.is_flow_content(c) && !self.is_phrasing_content(c) => {
                Err(ValidationError::InvalidNesting {
                    parent: parent.to_string(),
                    child: child.to_string(),
                    reason: "paragraph can only contain phrasing content".to_string(),
                })
            }

            // Table-related elements have strict rules
            ("table", c) if !matches!(c, "caption" | "colgroup" | "thead" | "tbody" | "tfoot" | "tr") => {
                Err(ValidationError::InvalidNesting {
                    parent: parent.to_string(),
                    child: child.to_string(),
                    reason: "table can only contain caption, colgroup, thead, tbody, tfoot, or tr".to_string(),
                })
            }

            ("tr", c) if !matches!(c, "td" | "th") => {
                Err(ValidationError::InvalidNesting {
                    parent: parent.to_string(),
                    child: child.to_string(),
                    reason: "tr can only contain td or th elements".to_string(),
                })
            }

            // List elements
            ("ul" | "ol", c) if c != "li" => {
                Err(ValidationError::InvalidNesting {
                    parent: parent.to_string(),
                    child: child.to_string(),
                    reason: "ul and ol can only contain li elements".to_string(),
                })
            }

            ("dl", c) if !matches!(c, "dt" | "dd" | "div") => {
                Err(ValidationError::InvalidNesting {
                    parent: parent.to_string(),
                    child: child.to_string(),
                    reason: "dl can only contain dt, dd, or div elements".to_string(),
                })
            }

            // Default: allow if we don't have specific rules
            _ => Ok(()),
        }
    }

    /// Check if an element is interactive
    fn is_interactive_element(&self, element: &str) -> bool {
        matches!(element, 
            "a" | "button" | "details" | "embed" | "iframe" | "keygen" | 
            "label" | "select" | "textarea" | "input" | "audio" | "video"
        )
    }

    /// Check if an element is flow content
    fn is_flow_content(&self, element: &str) -> bool {
        // Most elements are flow content, easier to list exceptions
        !matches!(element,
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | 
            "input" | "link" | "meta" | "param" | "source" | "track" | "wbr" |
            "title" | "style" | "script" | "noscript" | "template"
        )
    }

    /// Check if an element is phrasing content
    fn is_phrasing_content(&self, element: &str) -> bool {
        matches!(element,
            "a" | "abbr" | "area" | "audio" | "b" | "bdi" | "bdo" | "br" | 
            "button" | "canvas" | "cite" | "code" | "data" | "datalist" | 
            "del" | "dfn" | "em" | "embed" | "i" | "iframe" | "img" | "input" | 
            "ins" | "kbd" | "keygen" | "label" | "map" | "mark" | "math" | 
            "meter" | "noscript" | "object" | "output" | "picture" | "progress" | 
            "q" | "ruby" | "s" | "samp" | "script" | "select" | "small" | 
            "span" | "strong" | "sub" | "sup" | "svg" | "template" | "textarea" | 
            "time" | "u" | "var" | "video" | "wbr"
        )
    }

    /// Initialize content models for HTML5 elements
    fn init_content_models(content_models: &mut HashMap<String, ContentModel>) {
        // Document structure
        content_models.insert("html".to_string(), ContentModel {
            accepts: vec![ContentCategory::Metadata, ContentCategory::Flow],
            allowed_children: ["head", "body"].iter().map(|s| s.to_string()).collect(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });

        content_models.insert("head".to_string(), ContentModel {
            accepts: vec![ContentCategory::Metadata],
            allowed_children: HashSet::new(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });

        content_models.insert("body".to_string(), ContentModel {
            accepts: vec![ContentCategory::Flow],
            allowed_children: HashSet::new(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });

        // Sectioning content
        for element in ["article", "aside", "nav", "section"] {
            content_models.insert(element.to_string(), ContentModel {
                accepts: vec![ContentCategory::Flow],
                allowed_children: HashSet::new(),
                forbidden_children: HashSet::new(),
                is_transparent: false,
            });
        }

        // Heading content
        for element in ["h1", "h2", "h3", "h4", "h5", "h6"] {
            content_models.insert(element.to_string(), ContentModel {
                accepts: vec![ContentCategory::Phrasing],
                allowed_children: HashSet::new(),
                forbidden_children: HashSet::new(),
                is_transparent: false,
            });
        }

        // Phrasing content containers
        content_models.insert("p".to_string(), ContentModel {
            accepts: vec![ContentCategory::Phrasing],
            allowed_children: HashSet::new(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });

        // Form elements
        content_models.insert("form".to_string(), ContentModel {
            accepts: vec![ContentCategory::Flow],
            allowed_children: HashSet::new(),
            forbidden_children: ["form"].iter().map(|s| s.to_string()).collect(),
            is_transparent: false,
        });

        content_models.insert("button".to_string(), ContentModel {
            accepts: vec![ContentCategory::Phrasing],
            allowed_children: HashSet::new(),
            forbidden_children: ["a", "button", "input", "select", "textarea", "keygen", "label", 
                               "embed", "iframe", "object"].iter().map(|s| s.to_string()).collect(),
            is_transparent: false,
        });

        // Table elements
        content_models.insert("table".to_string(), ContentModel {
            accepts: vec![ContentCategory::Table],
            allowed_children: ["caption", "colgroup", "thead", "tbody", "tfoot", "tr"]
                .iter().map(|s| s.to_string()).collect(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });

        content_models.insert("tr".to_string(), ContentModel {
            accepts: vec![ContentCategory::Table],
            allowed_children: ["td", "th"].iter().map(|s| s.to_string()).collect(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });

        // List elements
        content_models.insert("ul".to_string(), ContentModel {
            accepts: vec![],
            allowed_children: ["li"].iter().map(|s| s.to_string()).collect(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });

        content_models.insert("ol".to_string(), ContentModel {
            accepts: vec![],
            allowed_children: ["li"].iter().map(|s| s.to_string()).collect(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });

        content_models.insert("li".to_string(), ContentModel {
            accepts: vec![ContentCategory::Flow],
            allowed_children: HashSet::new(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });

        content_models.insert("dl".to_string(), ContentModel {
            accepts: vec![],
            allowed_children: ["dt", "dd", "div"].iter().map(|s| s.to_string()).collect(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });

        // Generic containers
        content_models.insert("div".to_string(), ContentModel {
            accepts: vec![ContentCategory::Flow],
            allowed_children: HashSet::new(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });

        content_models.insert("span".to_string(), ContentModel {
            accepts: vec![ContentCategory::Phrasing],
            allowed_children: HashSet::new(),
            forbidden_children: HashSet::new(),
            is_transparent: false,
        });
    }

    /// Initialize element categories mapping
    fn init_element_categories(element_categories: &mut HashMap<String, Vec<ContentCategory>>) {
        // Metadata content
        for element in ["base", "link", "meta", "noscript", "script", "style", "title"] {
            element_categories.insert(element.to_string(), vec![ContentCategory::Metadata]);
        }

        // Flow content (most elements)
        for element in ["a", "abbr", "address", "article", "aside", "audio", "b", "bdi", "bdo", 
                       "blockquote", "br", "button", "canvas", "cite", "code", "data", "datalist",
                       "del", "details", "dfn", "div", "dl", "em", "embed", "fieldset", "figure",
                       "footer", "form", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hr", "i",
                       "iframe", "img", "input", "ins", "kbd", "keygen", "label", "main", "map",
                       "mark", "math", "meter", "nav", "noscript", "object", "ol", "output", "p",
                       "picture", "pre", "progress", "q", "ruby", "s", "samp", "script", "section",
                       "select", "small", "span", "strong", "sub", "sup", "svg", "table", "template",
                       "textarea", "time", "u", "ul", "var", "video", "wbr"] {
            element_categories.entry(element.to_string())
                .or_insert_with(Vec::new)
                .push(ContentCategory::Flow);
        }

        // Sectioning content
        for element in ["article", "aside", "nav", "section"] {
            element_categories.entry(element.to_string())
                .or_insert_with(Vec::new)
                .push(ContentCategory::Sectioning);
        }

        // Heading content
        for element in ["h1", "h2", "h3", "h4", "h5", "h6", "hgroup"] {
            element_categories.entry(element.to_string())
                .or_insert_with(Vec::new)
                .push(ContentCategory::Heading);
        }

        // Phrasing content
        for element in ["a", "abbr", "area", "audio", "b", "bdi", "bdo", "br", "button", "canvas",
                       "cite", "code", "data", "datalist", "del", "dfn", "em", "embed", "i",
                       "iframe", "img", "input", "ins", "kbd", "keygen", "label", "map", "mark",
                       "math", "meter", "noscript", "object", "output", "picture", "progress",
                       "q", "ruby", "s", "samp", "script", "select", "small", "span", "strong",
                       "sub", "sup", "svg", "template", "textarea", "time", "u", "var", "video",
                       "wbr"] {
            element_categories.entry(element.to_string())
                .or_insert_with(Vec::new)
                .push(ContentCategory::Phrasing);
        }

        // Embedded content
        for element in ["audio", "canvas", "embed", "iframe", "img", "math", "object", "picture",
                       "svg", "video"] {
            element_categories.entry(element.to_string())
                .or_insert_with(Vec::new)
                .push(ContentCategory::Embedded);
        }

        // Interactive content
        for element in ["a", "button", "details", "embed", "iframe", "keygen", "label", "select",
                       "textarea"] {
            element_categories.entry(element.to_string())
                .or_insert_with(Vec::new)
                .push(ContentCategory::Interactive);
        }

        // Form-associated content
        for element in ["button", "fieldset", "input", "keygen", "label", "meter", "object",
                       "output", "progress", "select", "textarea"] {
            element_categories.entry(element.to_string())
                .or_insert_with(Vec::new)
                .push(ContentCategory::FormAssociated);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_void_element_detection() {
        let validator = ElementStructureValidator::new();
        
        // Test void elements
        assert!(validator.is_void_element("br"));
        assert!(validator.is_void_element("img"));
        assert!(validator.is_void_element("input"));
        assert!(validator.is_void_element("hr"));
        assert!(validator.is_void_element("meta"));
        
        // Test non-void elements
        assert!(!validator.is_void_element("div"));
        assert!(!validator.is_void_element("span"));
        assert!(!validator.is_void_element("p"));
        assert!(!validator.is_void_element("button"));
    }

    #[test]
    fn test_void_element_validation() {
        let validator = ElementStructureValidator::new();
        
        // Void elements without children should pass
        assert!(validator.validate_void_element("br", false).is_ok());
        assert!(validator.validate_void_element("img", false).is_ok());
        
        // Void elements with children should fail
        assert!(validator.validate_void_element("br", true).is_err());
        assert!(validator.validate_void_element("img", true).is_err());
        
        // Non-void elements with children should pass
        assert!(validator.validate_void_element("div", true).is_ok());
        assert!(validator.validate_void_element("span", true).is_ok());
    }

    #[test]
    fn test_basic_nesting_validation() {
        let validator = ElementStructureValidator::new();
        
        // Valid nesting
        assert!(validator.validate_nesting("div", "span").is_ok());
        assert!(validator.validate_nesting("div", "p").is_ok());
        assert!(validator.validate_nesting("p", "span").is_ok());
        assert!(validator.validate_nesting("ul", "li").is_ok());
        assert!(validator.validate_nesting("table", "tr").is_ok());
        assert!(validator.validate_nesting("tr", "td").is_ok());
        
        // Invalid nesting
        assert!(validator.validate_nesting("p", "div").is_err()); // p can only contain phrasing
        assert!(validator.validate_nesting("ul", "div").is_err()); // ul can only contain li
        assert!(validator.validate_nesting("tr", "div").is_err()); // tr can only contain td/th
    }

    #[test]
    fn test_interactive_element_nesting() {
        let validator = ElementStructureValidator::new();
        
        // Interactive elements cannot contain other interactive elements
        assert!(validator.validate_nesting("button", "a").is_err());
        assert!(validator.validate_nesting("a", "button").is_err());
        assert!(validator.validate_nesting("button", "input").is_err());
        
        // But they can contain non-interactive elements
        assert!(validator.validate_nesting("button", "span").is_ok());
        assert!(validator.validate_nesting("a", "strong").is_ok());
    }

    #[test]
    fn test_form_nesting_validation() {
        let validator = ElementStructureValidator::new();
        
        // Forms cannot be nested
        assert!(validator.validate_nesting("form", "form").is_err());
        
        // Labels cannot be nested
        assert!(validator.validate_nesting("label", "label").is_err());
        
        // Forms can contain other elements
        assert!(validator.validate_nesting("form", "div").is_ok());
        assert!(validator.validate_nesting("form", "input").is_ok());
    }

    #[test]
    fn test_table_nesting_validation() {
        let validator = ElementStructureValidator::new();
        
        // Table can only contain specific elements
        assert!(validator.validate_nesting("table", "tr").is_ok());
        assert!(validator.validate_nesting("table", "thead").is_ok());
        assert!(validator.validate_nesting("table", "tbody").is_ok());
        assert!(validator.validate_nesting("table", "div").is_err());
        
        // TR can only contain td/th
        assert!(validator.validate_nesting("tr", "td").is_ok());
        assert!(validator.validate_nesting("tr", "th").is_ok());
        assert!(validator.validate_nesting("tr", "div").is_err());
    }

    #[test]
    fn test_list_nesting_validation() {
        let validator = ElementStructureValidator::new();
        
        // Lists can only contain li elements
        assert!(validator.validate_nesting("ul", "li").is_ok());
        assert!(validator.validate_nesting("ol", "li").is_ok());
        assert!(validator.validate_nesting("ul", "div").is_err());
        assert!(validator.validate_nesting("ol", "p").is_err());
        
        // Definition lists
        assert!(validator.validate_nesting("dl", "dt").is_ok());
        assert!(validator.validate_nesting("dl", "dd").is_ok());
        assert!(validator.validate_nesting("dl", "div").is_ok());
        assert!(validator.validate_nesting("dl", "p").is_err());
    }
}