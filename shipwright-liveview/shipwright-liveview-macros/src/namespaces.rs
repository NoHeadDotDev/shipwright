//! Namespace support for SVG and MathML elements in HTML documents.
//!
//! This module provides utilities for detecting and handling different XML namespaces
//! within the html! macro, particularly for SVG graphics and MathML mathematical content.

use html5ever::{namespace_url, ns, LocalName, QualName};
use std::collections::HashSet;

/// Supported XML namespaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Namespace {
    /// HTML namespace (default)
    Html,
    /// SVG namespace for scalable vector graphics
    Svg,
    /// MathML namespace for mathematical markup
    MathML,
}

impl Namespace {
    /// Get the namespace URI string
    pub(crate) fn uri(&self) -> &'static str {
        match self {
            Namespace::Html => "http://www.w3.org/1999/xhtml",
            Namespace::Svg => "http://www.w3.org/2000/svg",
            Namespace::MathML => "http://www.w3.org/1998/Math/MathML",
        }
    }

    /// Get the html5ever namespace constant
    pub(crate) fn html5ever_ns(&self) -> html5ever::Namespace {
        match self {
            Namespace::Html => ns!(html),
            Namespace::Svg => ns!(svg),
            Namespace::MathML => ns!(mathml),
        }
    }

    /// Get the prefix commonly used for this namespace
    pub(crate) fn common_prefix(&self) -> Option<&'static str> {
        match self {
            Namespace::Html => None,
            Namespace::Svg => Some("svg"),
            Namespace::MathML => Some("math"),
        }
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Namespace::Html => write!(f, "HTML"),
            Namespace::Svg => write!(f, "SVG"),
            Namespace::MathML => write!(f, "MathML"),
        }
    }
}

/// Namespace detector for determining element namespaces
pub(crate) struct NamespaceDetector {
    /// SVG root elements that switch to SVG namespace
    svg_root_elements: HashSet<&'static str>,
    /// SVG elements that can appear in HTML context
    svg_elements: HashSet<&'static str>,
    /// MathML root elements that switch to MathML namespace
    mathml_root_elements: HashSet<&'static str>,
    /// MathML elements
    mathml_elements: HashSet<&'static str>,
}

impl Default for NamespaceDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl NamespaceDetector {
    /// Create a new namespace detector with predefined element sets
    pub(crate) fn new() -> Self {
        let mut svg_root_elements = HashSet::new();
        svg_root_elements.insert("svg");

        let mut svg_elements = HashSet::new();
        // SVG structure elements
        svg_elements.insert("svg");
        svg_elements.insert("g");
        svg_elements.insert("defs");
        svg_elements.insert("symbol");
        svg_elements.insert("use");
        svg_elements.insert("switch");
        // SVG shape elements
        svg_elements.insert("rect");
        svg_elements.insert("circle");
        svg_elements.insert("ellipse");
        svg_elements.insert("line");
        svg_elements.insert("polyline");
        svg_elements.insert("polygon");
        svg_elements.insert("path");
        // SVG text elements
        svg_elements.insert("text");
        svg_elements.insert("tspan");
        svg_elements.insert("textPath");
        // SVG painting elements
        svg_elements.insert("marker");
        svg_elements.insert("pattern");
        svg_elements.insert("clipPath");
        svg_elements.insert("mask");
        svg_elements.insert("linearGradient");
        svg_elements.insert("radialGradient");
        svg_elements.insert("stop");
        // SVG filter elements
        svg_elements.insert("filter");
        svg_elements.insert("feGaussianBlur");
        svg_elements.insert("feOffset");
        svg_elements.insert("feFlood");
        svg_elements.insert("feComposite");
        svg_elements.insert("feColorMatrix");
        // SVG animation elements
        svg_elements.insert("animate");
        svg_elements.insert("animateTransform");
        svg_elements.insert("animateMotion");
        svg_elements.insert("set");
        // SVG interaction elements
        svg_elements.insert("a");
        svg_elements.insert("view");
        // SVG image elements
        svg_elements.insert("image");
        svg_elements.insert("foreignObject");

        let mut mathml_root_elements = HashSet::new();
        mathml_root_elements.insert("math");

        let mut mathml_elements = HashSet::new();
        // MathML root
        mathml_elements.insert("math");
        // MathML layout elements
        mathml_elements.insert("mrow");
        mathml_elements.insert("mfrac");
        mathml_elements.insert("msqrt");
        mathml_elements.insert("mroot");
        mathml_elements.insert("mstyle");
        mathml_elements.insert("merror");
        mathml_elements.insert("mpadded");
        mathml_elements.insert("mphantom");
        mathml_elements.insert("mfenced");
        mathml_elements.insert("menclose");
        // MathML content elements
        mathml_elements.insert("mi");
        mathml_elements.insert("mn");
        mathml_elements.insert("mo");
        mathml_elements.insert("mtext");
        mathml_elements.insert("mspace");
        mathml_elements.insert("ms");
        // MathML script elements
        mathml_elements.insert("msup");
        mathml_elements.insert("msub");
        mathml_elements.insert("msubsup");
        mathml_elements.insert("munder");
        mathml_elements.insert("mover");
        mathml_elements.insert("munderover");
        mathml_elements.insert("mmultiscripts");
        // MathML table elements
        mathml_elements.insert("mtable");
        mathml_elements.insert("mtr");
        mathml_elements.insert("mtd");
        mathml_elements.insert("maligngroup");
        mathml_elements.insert("malignmark");
        // MathML action elements
        mathml_elements.insert("maction");
        // MathML semantics elements
        mathml_elements.insert("semantics");
        mathml_elements.insert("annotation");
        mathml_elements.insert("annotation-xml");

        Self {
            svg_root_elements,
            svg_elements,
            mathml_root_elements,
            mathml_elements,
        }
    }

    /// Detect the namespace for a given element name
    pub(crate) fn detect_namespace(&self, element_name: &str, parent_namespace: Option<Namespace>) -> Namespace {
        // Check for namespace root elements first
        if self.svg_root_elements.contains(element_name) {
            return Namespace::Svg;
        }
        
        if self.mathml_root_elements.contains(element_name) {
            return Namespace::MathML;
        }

        // If we have a parent namespace, check if the element belongs to that namespace
        if let Some(parent_ns) = parent_namespace {
            match parent_ns {
                Namespace::Svg => {
                    if self.svg_elements.contains(element_name) {
                        return Namespace::Svg;
                    }
                    // SVG can contain HTML elements via foreignObject
                    if element_name == "foreignObject" {
                        return Namespace::Svg;
                    }
                }
                Namespace::MathML => {
                    if self.mathml_elements.contains(element_name) {
                        return Namespace::MathML;
                    }
                }
                Namespace::Html => {
                    // HTML is the default namespace
                }
            }
        }

        // Default to HTML namespace
        Namespace::Html
    }

    /// Check if an element is a namespace root element
    pub(crate) fn is_namespace_root(&self, element_name: &str) -> bool {
        self.svg_root_elements.contains(element_name) || 
        self.mathml_root_elements.contains(element_name)
    }

    /// Check if an element belongs to SVG namespace
    pub(crate) fn is_svg_element(&self, element_name: &str) -> bool {
        self.svg_elements.contains(element_name)
    }

    /// Check if an element belongs to MathML namespace
    pub(crate) fn is_mathml_element(&self, element_name: &str) -> bool {
        self.mathml_elements.contains(element_name)
    }

    /// Create a qualified name for html5ever integration
    pub(crate) fn create_qualified_name(&self, element_name: &str, namespace: Namespace) -> QualName {
        QualName::new(
            None, // prefix
            namespace.html5ever_ns(),
            LocalName::from(element_name)
        )
    }

    /// Get namespace-specific attributes that may be required
    pub(crate) fn get_namespace_attributes(&self, namespace: Namespace) -> Vec<(&'static str, &'static str)> {
        match namespace {
            Namespace::Html => vec![],
            Namespace::Svg => vec![
                ("xmlns", "http://www.w3.org/2000/svg"),
            ],
            Namespace::MathML => vec![
                ("xmlns", "http://www.w3.org/1998/Math/MathML"),
            ],
        }
    }

    /// Check if a namespace transition requires special handling
    pub(crate) fn requires_namespace_declaration(&self, from: Option<Namespace>, to: Namespace) -> bool {
        match (from, to) {
            (None, Namespace::Html) => false,
            (Some(Namespace::Html), Namespace::Html) => false,
            (None, Namespace::Svg) => true,
            (None, Namespace::MathML) => true,
            (Some(from_ns), to_ns) if from_ns == to_ns => false,
            _ => true,
        }
    }
}

/// Entity support for different namespaces
pub(crate) struct EntityHandler {
    /// HTML entities
    html_entities: HashSet<&'static str>,
    /// SVG-specific entities
    svg_entities: HashSet<&'static str>,
    /// MathML-specific entities
    mathml_entities: HashSet<&'static str>,
}

impl Default for EntityHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityHandler {
    /// Create a new entity handler with predefined entity sets
    pub(crate) fn new() -> Self {
        let mut html_entities = HashSet::new();
        // Common HTML entities
        html_entities.insert("amp");
        html_entities.insert("lt");
        html_entities.insert("gt");
        html_entities.insert("quot");
        html_entities.insert("apos");
        html_entities.insert("nbsp");
        html_entities.insert("copy");
        html_entities.insert("reg");
        html_entities.insert("trade");

        let mut svg_entities = HashSet::new();
        // SVG doesn't typically use many custom entities
        // Most SVG content uses direct Unicode or standard HTML entities

        let mut mathml_entities = HashSet::new();
        // Common MathML entities
        mathml_entities.insert("alpha");
        mathml_entities.insert("beta");
        mathml_entities.insert("gamma");
        mathml_entities.insert("delta");
        mathml_entities.insert("epsilon");
        mathml_entities.insert("pi");
        mathml_entities.insert("sigma");
        mathml_entities.insert("theta");
        mathml_entities.insert("omega");
        mathml_entities.insert("infin");
        mathml_entities.insert("sum");
        mathml_entities.insert("prod");
        mathml_entities.insert("int");
        mathml_entities.insert("part");
        mathml_entities.insert("nabla");
        mathml_entities.insert("radic");
        mathml_entities.insert("prop");
        mathml_entities.insert("empty");
        mathml_entities.insert("isin");
        mathml_entities.insert("notin");
        mathml_entities.insert("ni");
        mathml_entities.insert("cap");
        mathml_entities.insert("cup");
        mathml_entities.insert("sub");
        mathml_entities.insert("sup");
        mathml_entities.insert("sube");
        mathml_entities.insert("supe");

        Self {
            html_entities,
            svg_entities,
            mathml_entities,
        }
    }

    /// Check if an entity is valid for the given namespace
    pub(crate) fn is_valid_entity(&self, entity_name: &str, namespace: Namespace) -> bool {
        match namespace {
            Namespace::Html => self.html_entities.contains(entity_name),
            Namespace::Svg => {
                // SVG can use HTML entities plus its own
                self.html_entities.contains(entity_name) || 
                self.svg_entities.contains(entity_name)
            },
            Namespace::MathML => {
                // MathML can use HTML entities plus its own
                self.html_entities.contains(entity_name) || 
                self.mathml_entities.contains(entity_name)
            },
        }
    }

    /// Get all entities available for a namespace
    pub(crate) fn get_available_entities(&self, namespace: Namespace) -> Vec<&'static str> {
        let mut entities = Vec::new();
        
        match namespace {
            Namespace::Html => {
                entities.extend(self.html_entities.iter().copied());
            },
            Namespace::Svg => {
                entities.extend(self.html_entities.iter().copied());
                entities.extend(self.svg_entities.iter().copied());
            },
            Namespace::MathML => {
                entities.extend(self.html_entities.iter().copied());
                entities.extend(self.mathml_entities.iter().copied());
            },
        }
        
        entities.sort();
        entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_detection() {
        let detector = NamespaceDetector::new();
        
        // Test SVG detection
        assert_eq!(detector.detect_namespace("svg", None), Namespace::Svg);
        assert_eq!(detector.detect_namespace("rect", Some(Namespace::Svg)), Namespace::Svg);
        assert_eq!(detector.detect_namespace("circle", Some(Namespace::Svg)), Namespace::Svg);
        
        // Test MathML detection
        assert_eq!(detector.detect_namespace("math", None), Namespace::MathML);
        assert_eq!(detector.detect_namespace("mrow", Some(Namespace::MathML)), Namespace::MathML);
        assert_eq!(detector.detect_namespace("mi", Some(Namespace::MathML)), Namespace::MathML);
        
        // Test HTML detection (default)
        assert_eq!(detector.detect_namespace("div", None), Namespace::Html);
        assert_eq!(detector.detect_namespace("p", Some(Namespace::Html)), Namespace::Html);
        assert_eq!(detector.detect_namespace("unknown", None), Namespace::Html);
    }

    #[test]
    fn test_namespace_roots() {
        let detector = NamespaceDetector::new();
        
        assert!(detector.is_namespace_root("svg"));
        assert!(detector.is_namespace_root("math"));
        assert!(!detector.is_namespace_root("div"));
        assert!(!detector.is_namespace_root("rect"));
    }

    #[test]
    fn test_svg_elements() {
        let detector = NamespaceDetector::new();
        
        assert!(detector.is_svg_element("svg"));
        assert!(detector.is_svg_element("rect"));
        assert!(detector.is_svg_element("circle"));
        assert!(detector.is_svg_element("path"));
        assert!(detector.is_svg_element("text"));
        assert!(!detector.is_svg_element("div"));
        assert!(!detector.is_svg_element("math"));
    }

    #[test]
    fn test_mathml_elements() {
        let detector = NamespaceDetector::new();
        
        assert!(detector.is_mathml_element("math"));
        assert!(detector.is_mathml_element("mrow"));
        assert!(detector.is_mathml_element("mi"));
        assert!(detector.is_mathml_element("mn"));
        assert!(detector.is_mathml_element("mo"));
        assert!(!detector.is_mathml_element("div"));
        assert!(!detector.is_mathml_element("svg"));
    }

    #[test]
    fn test_qualified_name_creation() {
        let detector = NamespaceDetector::new();
        
        let html_qname = detector.create_qualified_name("div", Namespace::Html);
        assert_eq!(html_qname.ns, ns!(html));
        assert_eq!(html_qname.local, LocalName::from("div"));
        
        let svg_qname = detector.create_qualified_name("rect", Namespace::Svg);
        assert_eq!(svg_qname.ns, ns!(svg));
        assert_eq!(svg_qname.local, LocalName::from("rect"));
        
        let mathml_qname = detector.create_qualified_name("mi", Namespace::MathML);
        assert_eq!(mathml_qname.ns, ns!(mathml));
        assert_eq!(mathml_qname.local, LocalName::from("mi"));
    }

    #[test]
    fn test_namespace_attributes() {
        let detector = NamespaceDetector::new();
        
        let html_attrs = detector.get_namespace_attributes(Namespace::Html);
        assert!(html_attrs.is_empty());
        
        let svg_attrs = detector.get_namespace_attributes(Namespace::Svg);
        assert_eq!(svg_attrs, vec![("xmlns", "http://www.w3.org/2000/svg")]);
        
        let mathml_attrs = detector.get_namespace_attributes(Namespace::MathML);
        assert_eq!(mathml_attrs, vec![("xmlns", "http://www.w3.org/1998/Math/MathML")]);
    }

    #[test]
    fn test_namespace_declaration_requirements() {
        let detector = NamespaceDetector::new();
        
        // HTML doesn't require declaration
        assert!(!detector.requires_namespace_declaration(None, Namespace::Html));
        assert!(!detector.requires_namespace_declaration(Some(Namespace::Html), Namespace::Html));
        
        // SVG and MathML require declarations when switching from HTML or None
        assert!(detector.requires_namespace_declaration(None, Namespace::Svg));
        assert!(detector.requires_namespace_declaration(Some(Namespace::Html), Namespace::Svg));
        assert!(!detector.requires_namespace_declaration(Some(Namespace::Svg), Namespace::Svg));
        
        assert!(detector.requires_namespace_declaration(None, Namespace::MathML));
        assert!(detector.requires_namespace_declaration(Some(Namespace::Html), Namespace::MathML));
        assert!(!detector.requires_namespace_declaration(Some(Namespace::MathML), Namespace::MathML));
    }

    #[test]
    fn test_entity_validation() {
        let handler = EntityHandler::new();
        
        // HTML entities work in all namespaces
        assert!(handler.is_valid_entity("amp", Namespace::Html));
        assert!(handler.is_valid_entity("amp", Namespace::Svg));
        assert!(handler.is_valid_entity("amp", Namespace::MathML));
        
        // MathML entities only work in MathML
        assert!(!handler.is_valid_entity("alpha", Namespace::Html));
        assert!(!handler.is_valid_entity("alpha", Namespace::Svg));
        assert!(handler.is_valid_entity("alpha", Namespace::MathML));
        
        // Unknown entities don't work anywhere
        assert!(!handler.is_valid_entity("unknown", Namespace::Html));
        assert!(!handler.is_valid_entity("unknown", Namespace::Svg));
        assert!(!handler.is_valid_entity("unknown", Namespace::MathML));
    }

    #[test]
    fn test_namespace_uri() {
        assert_eq!(Namespace::Html.uri(), "http://www.w3.org/1999/xhtml");
        assert_eq!(Namespace::Svg.uri(), "http://www.w3.org/2000/svg");
        assert_eq!(Namespace::MathML.uri(), "http://www.w3.org/1998/Math/MathML");
    }

    #[test]
    fn test_namespace_display() {
        assert_eq!(format!("{}", Namespace::Html), "HTML");
        assert_eq!(format!("{}", Namespace::Svg), "SVG");
        assert_eq!(format!("{}", Namespace::MathML), "MathML");
    }
}