//! Template fingerprinting for change detection and hot reload optimization.
//!
//! This module provides sophisticated content-based hashing for templates,
//! enabling efficient hot reload by detecting what parts of a template
//! have actually changed.

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher, DefaultHasher};
use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::enhanced_ast::{DynamicPart, DynamicPartKind, TemplateLocation};

/// Comprehensive template fingerprint with hierarchical change detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemplateFingerprint {
    /// Overall template hash combining all aspects
    pub combined_hash: u64,
    /// Hash of static HTML structure only
    pub static_hash: u64,
    /// Hash of element hierarchy and nesting
    pub structure_hash: u64,
    /// Hash of dynamic parts and their types
    pub dynamic_hash: u64,
    /// Hash of style/class information
    pub styling_hash: u64,
    /// Hash of event handlers and interactions
    pub interaction_hash: u64,
    /// Individual hashes for each dynamic part
    pub part_hashes: HashMap<String, u64>,
    /// Hierarchical structure fingerprint
    pub hierarchy: HierarchyFingerprint,
}

/// Hierarchical fingerprint for nested template structures
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HierarchyFingerprint {
    /// Hash of the current level
    pub level_hash: u64,
    /// Fingerprints of child elements
    pub children: Vec<HierarchyFingerprint>,
    /// Element type at this level
    pub element_type: ElementType,
}

/// Element types for structural fingerprinting
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ElementType {
    /// HTML element with tag name
    HtmlElement(String),
    /// Dynamic text content
    TextContent,
    /// Control flow construct
    ControlFlow(String), // "if", "for", "match"
    /// Embedded Rust block
    Block,
    /// Document type declaration
    Doctype,
    /// Root container
    Root,
}

/// Template fingerprinting engine
pub(crate) struct FingerprintEngine {
    /// Configuration for fingerprint generation
    config: FingerprintConfig,
    /// Cache of previously computed fingerprints
    cache: HashMap<String, TemplateFingerprint>,
}

/// Configuration for fingerprint generation
#[derive(Debug, Clone)]
pub(crate) struct FingerprintConfig {
    /// Include CSS classes in styling hash
    pub include_classes: bool,
    /// Include inline styles in styling hash
    pub include_inline_styles: bool,
    /// Include data attributes in structure hash
    pub include_data_attributes: bool,
    /// Sensitivity for detecting dynamic part changes
    pub dynamic_sensitivity: DynamicSensitivity,
    /// Whether to generate hierarchical fingerprints
    pub hierarchical: bool,
}

/// Sensitivity levels for dynamic part change detection
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DynamicSensitivity {
    /// Only detect changes in dynamic part structure
    Structure,
    /// Detect changes in dependencies as well
    Dependencies,
    /// Detect any changes in the code content
    Content,
}

/// Fingerprint comparison result
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FingerprintComparison {
    /// Overall match status
    pub matches: bool,
    /// What aspects have changed
    pub changes: ChangeSet,
    /// Specific parts that changed
    pub changed_parts: Vec<String>,
    /// Hot reload compatibility
    pub hot_reload_compatible: bool,
}

/// Set of changes detected between templates
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct ChangeSet {
    /// Static HTML content changed
    pub static_content: bool,
    /// Element structure/hierarchy changed
    pub structure: bool,
    /// Dynamic parts changed
    pub dynamic_parts: bool,
    /// Styling/CSS changed
    pub styling: bool,
    /// Event handlers/interactions changed
    pub interactions: bool,
    /// New elements added
    pub additions: bool,
    /// Elements removed
    pub removals: bool,
}

impl Default for FingerprintConfig {
    fn default() -> Self {
        Self {
            include_classes: true,
            include_inline_styles: true,
            include_data_attributes: false,
            dynamic_sensitivity: DynamicSensitivity::Dependencies,
            hierarchical: true,
        }
    }
}

impl FingerprintEngine {
    /// Create a new fingerprint engine with default configuration
    pub(crate) fn new() -> Self {
        Self {
            config: FingerprintConfig::default(),
            cache: HashMap::new(),
        }
    }

    /// Create a fingerprint engine with custom configuration
    pub(crate) fn with_config(config: FingerprintConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
        }
    }

    /// Generate a comprehensive fingerprint for a template
    pub(crate) fn fingerprint_template(
        &mut self,
        template_id: &str,
        static_content: &str,
        structure: &TemplateStructure,
        dynamic_parts: &[DynamicPart],
    ) -> TemplateFingerprint {
        // Check cache first
        if let Some(cached) = self.cache.get(template_id) {
            return cached.clone();
        }

        let fingerprint = self.compute_fingerprint(static_content, structure, dynamic_parts);
        
        // Cache the result
        self.cache.insert(template_id.to_string(), fingerprint.clone());
        
        fingerprint
    }

    /// Compute a fingerprint from template components
    fn compute_fingerprint(
        &self,
        static_content: &str,
        structure: &TemplateStructure,
        dynamic_parts: &[DynamicPart],
    ) -> TemplateFingerprint {
        let static_hash = self.hash_static_content(static_content);
        let structure_hash = self.hash_structure(structure);
        let dynamic_hash = self.hash_dynamic_parts(dynamic_parts);
        let styling_hash = self.hash_styling(structure);
        let interaction_hash = self.hash_interactions(dynamic_parts);
        
        let part_hashes = self.hash_individual_parts(dynamic_parts);
        let hierarchy = self.build_hierarchy_fingerprint(structure);

        let combined_hash = self.combine_hashes(&[
            static_hash,
            structure_hash,
            dynamic_hash,
            styling_hash,
            interaction_hash,
        ]);

        TemplateFingerprint {
            combined_hash,
            static_hash,
            structure_hash,
            dynamic_hash,
            styling_hash,
            interaction_hash,
            part_hashes,
            hierarchy,
        }
    }

    /// Hash static HTML content
    fn hash_static_content(&self, content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Hash template structure
    fn hash_structure(&self, structure: &TemplateStructure) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash_structure_recursive(&structure.root, &mut hasher);
        hasher.finish()
    }

    /// Recursively hash structure elements
    fn hash_structure_recursive(&self, element: &StructureElement, hasher: &mut DefaultHasher) {
        element.element_type.hash(hasher);
        element.attributes.len().hash(hasher);
        
        for (name, value) in &element.attributes {
            name.hash(hasher);
            
            // Include data attributes based on configuration
            if name.starts_with("data-") && !self.config.include_data_attributes {
                continue;
            }
            
            if let Some(val) = value {
                val.hash(hasher);
            }
        }

        element.children.len().hash(hasher);
        for child in &element.children {
            self.hash_structure_recursive(child, hasher);
        }
    }

    /// Hash dynamic parts
    fn hash_dynamic_parts(&self, parts: &[DynamicPart]) -> u64 {
        let mut hasher = DefaultHasher::new();
        
        for part in parts {
            part.kind.hash(&mut hasher);
            part.location.file.hash(&mut hasher);
            part.location.line.hash(&mut hasher);
            part.location.column.hash(&mut hasher);
            
            match self.config.dynamic_sensitivity {
                DynamicSensitivity::Structure => {
                    // Only hash the kind and location
                }
                DynamicSensitivity::Dependencies => {
                    // Include dependencies
                    for dep in &part.dependencies {
                        dep.hash(&mut hasher);
                    }
                }
                DynamicSensitivity::Content => {
                    // Include the actual code content
                    part.code.to_string().hash(&mut hasher);
                }
            }
        }
        
        hasher.finish()
    }

    /// Hash styling-related content
    fn hash_styling(&self, structure: &TemplateStructure) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash_styling_recursive(&structure.root, &mut hasher);
        hasher.finish()
    }

    /// Recursively hash styling elements
    fn hash_styling_recursive(&self, element: &StructureElement, hasher: &mut DefaultHasher) {
        if self.config.include_classes {
            if let Some(class_value) = element.attributes.get("class") {
                if let Some(classes) = class_value {
                    classes.hash(hasher);
                }
            }
        }

        if self.config.include_inline_styles {
            if let Some(style_value) = element.attributes.get("style") {
                if let Some(style) = style_value {
                    style.hash(hasher);
                }
            }
        }

        for child in &element.children {
            self.hash_styling_recursive(child, hasher);
        }
    }

    /// Hash interaction-related content (event handlers)
    fn hash_interactions(&self, parts: &[DynamicPart]) -> u64 {
        let mut hasher = DefaultHasher::new();
        
        for part in parts {
            if let DynamicPartKind::EventHandler { event_name } = &part.kind {
                event_name.hash(&mut hasher);
                part.code.to_string().hash(&mut hasher);
            }
        }
        
        hasher.finish()
    }

    /// Hash individual dynamic parts separately
    fn hash_individual_parts(&self, parts: &[DynamicPart]) -> HashMap<String, u64> {
        let mut part_hashes = HashMap::new();
        
        for (index, part) in parts.iter().enumerate() {
            let mut hasher = DefaultHasher::new();
            part.kind.hash(&mut hasher);
            part.code.to_string().hash(&mut hasher);
            
            let part_id = format!("{}_{}", index, part.location.location_id());
            part_hashes.insert(part_id, hasher.finish());
        }
        
        part_hashes
    }

    /// Build hierarchical fingerprint
    fn build_hierarchy_fingerprint(&self, structure: &TemplateStructure) -> HierarchyFingerprint {
        if !self.config.hierarchical {
            return HierarchyFingerprint {
                level_hash: 0,
                children: vec![],
                element_type: ElementType::Root,
            };
        }

        self.build_hierarchy_recursive(&structure.root)
    }

    /// Recursively build hierarchy fingerprint
    fn build_hierarchy_recursive(&self, element: &StructureElement) -> HierarchyFingerprint {
        let mut hasher = DefaultHasher::new();
        element.element_type.hash(&mut hasher);
        element.attributes.len().hash(&mut hasher);
        
        let children: Vec<HierarchyFingerprint> = element
            .children
            .iter()
            .map(|child| self.build_hierarchy_recursive(child))
            .collect();

        // Include children hashes in level hash
        for child in &children {
            child.level_hash.hash(&mut hasher);
        }

        HierarchyFingerprint {
            level_hash: hasher.finish(),
            children,
            element_type: element.element_type.clone(),
        }
    }

    /// Combine multiple hashes into a single hash
    fn combine_hashes(&self, hashes: &[u64]) -> u64 {
        let mut hasher = DefaultHasher::new();
        for hash in hashes {
            hash.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Compare two fingerprints and determine what changed
    pub(crate) fn compare_fingerprints(
        &self,
        old: &TemplateFingerprint,
        new: &TemplateFingerprint,
    ) -> FingerprintComparison {
        let matches = old.combined_hash == new.combined_hash;
        
        if matches {
            return FingerprintComparison {
                matches: true,
                changes: ChangeSet::default(),
                changed_parts: vec![],
                hot_reload_compatible: true,
            };
        }

        let mut changes = ChangeSet::default();
        let mut changed_parts = Vec::new();

        // Check what aspects changed
        if old.static_hash != new.static_hash {
            changes.static_content = true;
        }

        if old.structure_hash != new.structure_hash {
            changes.structure = true;
        }

        if old.dynamic_hash != new.dynamic_hash {
            changes.dynamic_parts = true;
        }

        if old.styling_hash != new.styling_hash {
            changes.styling = true;
        }

        if old.interaction_hash != new.interaction_hash {
            changes.interactions = true;
        }

        // Find specific changed parts
        for (part_id, old_hash) in &old.part_hashes {
            if let Some(new_hash) = new.part_hashes.get(part_id) {
                if old_hash != new_hash {
                    changed_parts.push(part_id.clone());
                }
            } else {
                changes.removals = true;
                changed_parts.push(part_id.clone());
            }
        }

        for part_id in new.part_hashes.keys() {
            if !old.part_hashes.contains_key(part_id) {
                changes.additions = true;
                changed_parts.push(part_id.clone());
            }
        }

        // Determine hot reload compatibility
        let hot_reload_compatible = !changes.structure && !changes.additions && !changes.removals;

        FingerprintComparison {
            matches,
            changes,
            changed_parts,
            hot_reload_compatible,
        }
    }

    /// Clear the fingerprint cache
    pub(crate) fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub(crate) fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.capacity())
    }
}

impl Default for FingerprintEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Template structure representation for fingerprinting
#[derive(Debug, Clone)]
pub(crate) struct TemplateStructure {
    /// Root element of the template
    pub root: StructureElement,
}

/// Individual structure element
#[derive(Debug, Clone)]
pub(crate) struct StructureElement {
    /// Type of element
    pub element_type: ElementType,
    /// Element attributes
    pub attributes: HashMap<String, Option<String>>,
    /// Child elements
    pub children: Vec<StructureElement>,
}

impl Hash for DynamicPartKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DynamicPartKind::TextContent => 0.hash(state),
            DynamicPartKind::AttributeValue { attr_name } => {
                1.hash(state);
                attr_name.hash(state);
            }
            DynamicPartKind::AttributePresence { attr_name } => {
                2.hash(state);
                attr_name.hash(state);
            }
            DynamicPartKind::EventHandler { event_name } => {
                3.hash(state);
                event_name.hash(state);
            }
            DynamicPartKind::ControlFlow => 4.hash(state),
            DynamicPartKind::Block => 5.hash(state),
        }
    }
}

impl TemplateStructure {
    /// Create a template structure from enhanced AST
    pub(crate) fn from_html_nodes(nodes: &[crate::enhanced_ast::EnhancedHtmlNode]) -> Self {
        let mut root_children = Vec::new();
        
        for node in nodes {
            root_children.push(Self::structure_from_node(&node.node));
        }

        Self {
            root: StructureElement {
                element_type: ElementType::Root,
                attributes: HashMap::new(),
                children: root_children,
            },
        }
    }

    /// Convert an HTML node to a structure element
    fn structure_from_node(node: &crate::HtmlNode) -> StructureElement {
        match node {
            crate::HtmlNode::TagNode(tag) => {
                let mut attributes = HashMap::new();
                
                for attr in &tag.attrs {
                    let (name, value) = match attr {
                        crate::Attr::Normal { ident, value } => {
                            let name = match ident {
                                crate::AttrIdent::Lit(n) => n.clone(),
                                crate::AttrIdent::Axm(n) => n.clone(),
                            };
                            
                            let val = match value {
                                crate::NormalAttrValue::LitStr(lit) => Some(lit.value()),
                                crate::NormalAttrValue::Unit(_) => None,
                                crate::NormalAttrValue::None => None,
                                _ => Some("<dynamic>".to_string()),
                            };
                            
                            (name, val)
                        }
                        crate::Attr::Axm { ident, .. } => {
                            let name = match ident {
                                crate::AttrIdent::Axm(n) => n.clone(),
                                crate::AttrIdent::Lit(n) => n.clone(),
                            };
                            (name, Some("<handler>".to_string()))
                        }
                    };
                    
                    attributes.insert(name, value);
                }

                let children = if let Some(tag_close) = &tag.close {
                    tag_close.inner
                        .iter()
                        .map(Self::structure_from_node)
                        .collect()
                } else {
                    Vec::new()
                };

                StructureElement {
                    element_type: ElementType::HtmlElement(tag.open.to_string()),
                    attributes,
                    children,
                }
            }
            crate::HtmlNode::LitStr(_) => StructureElement {
                element_type: ElementType::TextContent,
                attributes: HashMap::new(),
                children: Vec::new(),
            },
            crate::HtmlNode::Block(_) => StructureElement {
                element_type: ElementType::Block,
                attributes: HashMap::new(),
                children: Vec::new(),
            },
            crate::HtmlNode::If(_) => StructureElement {
                element_type: ElementType::ControlFlow("if".to_string()),
                attributes: HashMap::new(),
                children: Vec::new(),
            },
            crate::HtmlNode::For(_) => StructureElement {
                element_type: ElementType::ControlFlow("for".to_string()),
                attributes: HashMap::new(),
                children: Vec::new(),
            },
            crate::HtmlNode::Match(_) => StructureElement {
                element_type: ElementType::ControlFlow("match".to_string()),
                attributes: HashMap::new(),
                children: Vec::new(),
            },
            crate::HtmlNode::Doctype(_) => StructureElement {
                element_type: ElementType::Doctype,
                attributes: HashMap::new(),
                children: Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_engine() {
        let mut engine = FingerprintEngine::new();
        
        let structure = TemplateStructure {
            root: StructureElement {
                element_type: ElementType::Root,
                attributes: HashMap::new(),
                children: vec![
                    StructureElement {
                        element_type: ElementType::HtmlElement("div".to_string()),
                        attributes: HashMap::new(),
                        children: Vec::new(),
                    }
                ],
            },
        };

        let fp1 = engine.fingerprint_template("test1", "content", &structure, &[]);
        let fp2 = engine.fingerprint_template("test1", "content", &structure, &[]);
        
        // Should be identical due to caching
        assert_eq!(fp1.combined_hash, fp2.combined_hash);
    }

    #[test]
    fn test_fingerprint_comparison() {
        let engine = FingerprintEngine::new();
        
        let fp1 = TemplateFingerprint {
            combined_hash: 123,
            static_hash: 456,
            structure_hash: 789,
            dynamic_hash: 0,
            styling_hash: 0,
            interaction_hash: 0,
            part_hashes: HashMap::new(),
            hierarchy: HierarchyFingerprint {
                level_hash: 0,
                children: vec![],
                element_type: ElementType::Root,
            },
        };

        let fp2 = TemplateFingerprint {
            combined_hash: 124,
            static_hash: 457,
            structure_hash: 789,
            dynamic_hash: 0,
            styling_hash: 0,
            interaction_hash: 0,
            part_hashes: HashMap::new(),
            hierarchy: HierarchyFingerprint {
                level_hash: 0,
                children: vec![],
                element_type: ElementType::Root,
            },
        };

        let comparison = engine.compare_fingerprints(&fp1, &fp2);
        assert!(!comparison.matches);
        assert!(comparison.changes.static_content);
        assert!(!comparison.changes.structure);
        assert!(comparison.hot_reload_compatible);
    }

    #[test]
    fn test_template_structure() {
        let element = StructureElement {
            element_type: ElementType::HtmlElement("div".to_string()),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("class".to_string(), Some("test".to_string()));
                attrs
            },
            children: Vec::new(),
        };

        assert_eq!(element.element_type, ElementType::HtmlElement("div".to_string()));
        assert_eq!(element.attributes.get("class"), Some(&Some("test".to_string())));
    }

    #[test]
    fn test_dynamic_sensitivity() {
        let config = FingerprintConfig {
            dynamic_sensitivity: DynamicSensitivity::Content,
            ..Default::default()
        };

        assert_eq!(config.dynamic_sensitivity, DynamicSensitivity::Content);
        assert!(config.include_classes);
        assert!(config.hierarchical);
    }
}