//! Enhanced AST types for hot reload-capable template parsing.
//!
//! This module extends the existing AST structure with location tracking,
//! template fingerprinting, and dynamic part extraction capabilities
//! for sophisticated hot reload functionality.

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Block, Ident, LitStr,
};
use std::collections::HashMap;
use std::fmt::Write;

use crate::{
    HtmlNode, Tree, TagNode, Attr, If, For, Match, Close, TagClose,
    Doctype, AttrIdent, NormalAttrValue, AxmAttrValue, ElseBranch,
    NodeToTokens, FixedParts,
};

/// Enhanced AST node that includes location tracking and hot reload metadata
#[derive(Debug, Clone)]
pub(crate) struct EnhancedHtmlNode {
    /// The original HTML node
    pub node: HtmlNode,
    /// Location information for hot reload tracking
    pub location: TemplateLocation,
    /// Unique identifier for this node within the template
    pub node_id: NodeId,
    /// Template fingerprint for change detection
    pub fingerprint: Option<TemplateFingerprint>,
    /// Dynamic parts within this node
    pub dynamic_parts: Vec<DynamicPart>,
}

/// Enhanced tree structure with hot reload capabilities
#[derive(Debug, Clone)]
pub(crate) struct EnhancedTree {
    /// Enhanced HTML nodes
    pub nodes: Vec<EnhancedHtmlNode>,
    /// Template metadata
    pub template_meta: TemplateMeta,
}

/// Location information for template tracking
#[derive(Debug, Clone)]
pub(crate) struct TemplateLocation {
    /// Source file path
    pub file: String,
    /// Line number in source
    pub line: u32,
    /// Column number in source
    pub column: u32,
    /// Template path within the template (e.g., "0.1.2" for nested elements)
    pub template_path: String,
    /// Span information from proc_macro2
    pub span: Span,
}

/// Unique identifier for nodes within a template
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NodeId {
    /// Template-unique identifier
    pub id: String,
    /// Hierarchical path within the template (e.g., "0.1.2" for nested elements)
    pub path: String,
}

/// Template fingerprint for change detection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TemplateFingerprint {
    /// Content hash of the static parts
    pub static_hash: u64,
    /// Hash of the structure (elements, attributes, nesting)
    pub structure_hash: u64,
    /// Combined hash for quick comparison
    pub combined_hash: u64,
}

/// Dynamic parts that can change during hot reload
#[derive(Debug, Clone)]
pub(crate) struct DynamicPart {
    /// Type of dynamic content
    pub kind: DynamicPartKind,
    /// Location within the template
    pub location: TemplateLocation,
    /// Rust code block for this dynamic part
    pub code: TokenStream,
    /// Dependencies (variables, functions) used in this part
    pub dependencies: Vec<String>,
}

/// Types of dynamic content
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DynamicPartKind {
    /// Dynamic text content
    TextContent,
    /// Dynamic attribute value
    AttributeValue { attr_name: String },
    /// Dynamic attribute existence (conditional attributes)
    AttributePresence { attr_name: String },
    /// Event handler
    EventHandler { event_name: String },
    /// Control flow (if, for, match)
    ControlFlow,
    /// Embedded Rust block
    Block,
}

/// Template metadata for hot reload tracking
#[derive(Debug, Clone)]
pub(crate) struct TemplateMeta {
    /// Unique template identifier based on location
    pub template_id: String,
    /// Template fingerprint
    pub fingerprint: TemplateFingerprint,
    /// All dynamic parts in the template
    pub dynamic_parts: Vec<DynamicPart>,
    /// Static HTML structure
    pub static_structure: String,
    /// Template dependencies
    pub dependencies: TemplateDependencies,
}

/// Dependencies that affect template rendering
#[derive(Debug, Clone, Default)]
pub(crate) struct TemplateDependencies {
    /// Variables referenced in the template
    pub variables: Vec<String>,
    /// Functions called in the template
    pub functions: Vec<String>,
    /// Modules imported in the template context
    pub modules: Vec<String>,
    /// External macros used
    pub macros: Vec<String>,
}

impl TemplateLocation {
    /// Create a new template location from current macro context
    pub(crate) fn from_span(span: Span) -> Self {
        Self {
            file: file!().to_string(),
            line: line!(),
            column: column!(),
            template_path: String::new(),
            span,
        }
    }

    /// Create a location with explicit file/line/column
    pub(crate) fn new(file: String, line: u32, column: u32, span: Span) -> Self {
        Self {
            file,
            line,
            column,
            template_path: String::new(),
            span,
        }
    }

    /// Generate a unique location-based ID
    pub(crate) fn location_id(&self) -> String {
        format!("{}:{}:{}", self.file, self.line, self.column)
    }
}

impl NodeId {
    /// Create a new node ID
    pub(crate) fn new(template_id: &str, path: &str) -> Self {
        Self {
            id: format!("{}_{}", template_id, path.replace('.', "_")),
            path: path.to_string(),
        }
    }

    /// Create a child node ID
    pub(crate) fn child(&self, index: usize) -> Self {
        Self {
            id: format!("{}_{}", self.id, index),
            path: if self.path.is_empty() {
                index.to_string()
            } else {
                format!("{}.{}", self.path, index)
            },
        }
    }
}

impl TemplateFingerprint {
    /// Create a new fingerprint from content
    pub(crate) fn new(static_content: &str, structure: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let static_hash = {
            let mut hasher = DefaultHasher::new();
            static_content.hash(&mut hasher);
            hasher.finish()
        };

        let structure_hash = {
            let mut hasher = DefaultHasher::new();
            structure.hash(&mut hasher);
            hasher.finish()
        };

        let combined_hash = {
            let mut hasher = DefaultHasher::new();
            static_hash.hash(&mut hasher);
            structure_hash.hash(&mut hasher);
            hasher.finish()
        };

        Self {
            static_hash,
            structure_hash,
            combined_hash,
        }
    }

    /// Check if this fingerprint matches another
    pub(crate) fn matches(&self, other: &Self) -> bool {
        self.combined_hash == other.combined_hash
    }

    /// Check if only static content changed
    pub(crate) fn only_static_changed(&self, other: &Self) -> bool {
        self.structure_hash == other.structure_hash && self.static_hash != other.static_hash
    }

    /// Check if structure changed (requiring full rebuild)
    pub(crate) fn structure_changed(&self, other: &Self) -> bool {
        self.structure_hash != other.structure_hash
    }
}

impl EnhancedHtmlNode {
    /// Create an enhanced node from a regular HTML node
    pub(crate) fn from_html_node(
        node: HtmlNode,
        location: TemplateLocation,
        template_id: &str,
        node_path: &str,
    ) -> Self {
        let node_id = NodeId::new(template_id, node_path);
        let mut enhanced_node = Self {
            node,
            location,
            node_id,
            fingerprint: None,
            dynamic_parts: Vec::new(),
        };

        // Extract dynamic parts from the node
        enhanced_node.extract_dynamic_parts();
        
        // Generate fingerprint
        enhanced_node.generate_fingerprint();

        enhanced_node
    }

    /// Extract dynamic parts from the HTML node
    fn extract_dynamic_parts(&mut self) {
        match self.node.clone() {
            HtmlNode::TagNode(tag) => {
                self.extract_tag_dynamic_parts(&tag);
            }
            HtmlNode::Block(block) => {
                self.dynamic_parts.push(DynamicPart {
                    kind: DynamicPartKind::Block,
                    location: self.location.clone(),
                    code: block.to_token_stream(),
                    dependencies: Self::extract_dependencies_from_block(&block),
                });
            }
            HtmlNode::If(if_node) => {
                self.dynamic_parts.push(DynamicPart {
                    kind: DynamicPartKind::ControlFlow,
                    location: self.location.clone(),
                    code: quote! { #if_node },
                    dependencies: Self::extract_dependencies_from_expr(&if_node.cond),
                });
            }
            HtmlNode::For(for_node) => {
                self.dynamic_parts.push(DynamicPart {
                    kind: DynamicPartKind::ControlFlow,
                    location: self.location.clone(),
                    code: quote! { #for_node },
                    dependencies: Self::extract_dependencies_from_expr(&for_node.expr),
                });
            }
            HtmlNode::Match(match_node) => {
                self.dynamic_parts.push(DynamicPart {
                    kind: DynamicPartKind::ControlFlow,
                    location: self.location.clone(),
                    code: quote! { #match_node },
                    dependencies: Self::extract_dependencies_from_expr(&match_node.expr),
                });
            }
            _ => {
                // Static nodes don't have dynamic parts
            }
        }
    }

    /// Extract dynamic parts from a tag node
    fn extract_tag_dynamic_parts(&mut self, tag: &TagNode) {
        for attr in &tag.attrs {
            match attr {
                Attr::Normal { ident, value } => {
                    match value {
                        NormalAttrValue::Block(block) => {
                            let attr_name = match ident {
                                AttrIdent::Lit(name) => name.clone(),
                                AttrIdent::Axm(name) => name.clone(),
                            };
                            
                            self.dynamic_parts.push(DynamicPart {
                                kind: DynamicPartKind::AttributeValue { attr_name },
                                location: self.location.clone(),
                                code: block.to_token_stream(),
                                dependencies: Self::extract_dependencies_from_block(block),
                            });
                        }
                        NormalAttrValue::If(if_node) => {
                            let attr_name = match ident {
                                AttrIdent::Lit(name) => name.clone(),
                                AttrIdent::Axm(name) => name.clone(),
                            };
                            
                            self.dynamic_parts.push(DynamicPart {
                                kind: DynamicPartKind::AttributePresence { attr_name },
                                location: self.location.clone(),
                                code: quote! { #if_node },
                                dependencies: Self::extract_dependencies_from_expr(&if_node.cond),
                            });
                        }
                        _ => {
                            // Static attribute values
                        }
                    }
                }
                Attr::Axm { ident, value } => {
                    let event_name = match ident {
                        AttrIdent::Axm(name) => name.clone(),
                        AttrIdent::Lit(name) => name.clone(),
                    };
                    
                    match value {
                        AxmAttrValue::Block(block) => {
                            self.dynamic_parts.push(DynamicPart {
                                kind: DynamicPartKind::EventHandler { event_name },
                                location: self.location.clone(),
                                code: block.to_token_stream(),
                                dependencies: Self::extract_dependencies_from_block(block),
                            });
                        }
                        AxmAttrValue::If(if_node) => {
                            self.dynamic_parts.push(DynamicPart {
                                kind: DynamicPartKind::EventHandler { event_name },
                                location: self.location.clone(),
                                code: quote! { #if_node },
                                dependencies: Self::extract_dependencies_from_expr(&if_node.cond),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Extract dependencies from a Rust expression (simplified)
    fn extract_dependencies_from_expr(expr: &syn::Expr) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // This is a simplified dependency extraction
        // In a full implementation, you'd use syn::visit to traverse the AST
        let expr_str = quote! { #expr }.to_string();
        
        // Basic pattern matching for common dependency patterns
        // This would be much more sophisticated in a real implementation
        if expr_str.contains("self.") {
            dependencies.push("self".to_string());
        }
        
        dependencies
    }

    /// Extract dependencies from a Rust block (simplified)
    fn extract_dependencies_from_block(block: &Block) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Simplified dependency extraction
        let block_str = quote! { #block }.to_string();
        
        if block_str.contains("self.") {
            dependencies.push("self".to_string());
        }
        
        dependencies
    }

    /// Generate a fingerprint for this node
    fn generate_fingerprint(&mut self) {
        let static_content = self.extract_static_content();
        let structure = self.extract_structure();
        self.fingerprint = Some(TemplateFingerprint::new(&static_content, &structure));
    }

    /// Extract static content from the node
    fn extract_static_content(&self) -> String {
        match &self.node {
            HtmlNode::LitStr(lit) => lit.value(),
            HtmlNode::TagNode(tag) => {
                format!("<{}>", tag.open)
            }
            HtmlNode::Doctype(_) => "<!DOCTYPE html>".to_string(),
            _ => String::new(),
        }
    }

    /// Extract structural information from the node
    fn extract_structure(&self) -> String {
        match &self.node {
            HtmlNode::TagNode(tag) => {
                let mut structure = String::new();
                let _ = write!(structure, "tag:{}", tag.open);
                
                for attr in &tag.attrs {
                    match attr {
                        Attr::Normal { ident, .. } => {
                            match ident {
                                AttrIdent::Lit(name) => {
                                    let _ = write!(structure, " attr:{}", name);
                                }
                                AttrIdent::Axm(name) => {
                                    let _ = write!(structure, " axm:{}", name);
                                }
                            }
                        }
                        Attr::Axm { ident, .. } => {
                            match ident {
                                AttrIdent::Axm(name) => {
                                    let _ = write!(structure, " axm:{}", name);
                                }
                                AttrIdent::Lit(name) => {
                                    let _ = write!(structure, " attr:{}", name);
                                }
                            }
                        }
                    }
                }
                
                structure
            }
            HtmlNode::If(_) => "if".to_string(),
            HtmlNode::For(_) => "for".to_string(),
            HtmlNode::Match(_) => "match".to_string(),
            HtmlNode::Block(_) => "block".to_string(),
            HtmlNode::LitStr(_) => "text".to_string(),
            HtmlNode::Doctype(_) => "doctype".to_string(),
        }
    }
}

impl EnhancedTree {
    /// Create an enhanced tree from a regular tree
    pub(crate) fn from_tree(tree: Tree, location: TemplateLocation) -> Self {
        let template_id = location.location_id();
        let mut enhanced_nodes = Vec::new();
        
        for (index, node) in tree.nodes.into_iter().enumerate() {
            let node_path = index.to_string();
            let enhanced_node = EnhancedHtmlNode::from_html_node(
                node,
                location.clone(),
                &template_id,
                &node_path,
            );
            enhanced_nodes.push(enhanced_node);
        }

        let template_meta = TemplateMeta::from_enhanced_nodes(&enhanced_nodes, &template_id);

        Self {
            nodes: enhanced_nodes,
            template_meta,
        }
    }

    /// Get all dynamic parts from the tree
    pub(crate) fn dynamic_parts(&self) -> Vec<&DynamicPart> {
        self.nodes
            .iter()
            .flat_map(|node| &node.dynamic_parts)
            .collect()
    }

    /// Check if the tree structure has changed compared to another tree
    pub(crate) fn structure_changed(&self, other: &Self) -> bool {
        !self.template_meta.fingerprint.matches(&other.template_meta.fingerprint)
    }

    /// Generate hot reload metadata for this template
    pub(crate) fn hot_reload_meta(&self) -> HotReloadMeta {
        HotReloadMeta {
            template_id: self.template_meta.template_id.clone(),
            fingerprint: self.template_meta.fingerprint.clone(),
            dynamic_parts: self.dynamic_parts().into_iter().cloned().collect(),
            static_structure: self.template_meta.static_structure.clone(),
            dependencies: self.template_meta.dependencies.clone(),
        }
    }
}

impl TemplateMeta {
    /// Create template metadata from enhanced nodes
    fn from_enhanced_nodes(nodes: &[EnhancedHtmlNode], template_id: &str) -> Self {
        let mut all_dynamic_parts = Vec::new();
        let mut static_content = String::new();
        let mut structure = String::new();
        let mut dependencies = TemplateDependencies::default();

        for node in nodes {
            all_dynamic_parts.extend(node.dynamic_parts.iter().cloned());
            static_content.push_str(&node.extract_static_content());
            structure.push_str(&node.extract_structure());
            
            // Collect dependencies
            for part in &node.dynamic_parts {
                dependencies.variables.extend(part.dependencies.iter().cloned());
            }
        }

        // Remove duplicates from dependencies
        dependencies.variables.sort();
        dependencies.variables.dedup();

        let fingerprint = TemplateFingerprint::new(&static_content, &structure);

        Self {
            template_id: template_id.to_string(),
            fingerprint,
            dynamic_parts: all_dynamic_parts,
            static_structure: structure,
            dependencies,
        }
    }
}

/// Hot reload metadata for runtime template updates
#[derive(Debug, Clone)]
pub(crate) struct HotReloadMeta {
    /// Template identifier
    pub template_id: String,
    /// Template fingerprint
    pub fingerprint: TemplateFingerprint,
    /// Dynamic parts that can be updated
    pub dynamic_parts: Vec<DynamicPart>,
    /// Static HTML structure
    pub static_structure: String,
    /// Template dependencies
    pub dependencies: TemplateDependencies,
}

impl HotReloadMeta {
    /// Check if this template can be hot reloaded compared to another version
    pub(crate) fn can_hot_reload(&self, other: &Self) -> bool {
        !self.fingerprint.structure_changed(&other.fingerprint)
    }

    /// Get the parts that need updating for hot reload
    pub(crate) fn diff_for_reload(&self, other: &Self) -> Vec<&DynamicPart> {
        // In a real implementation, this would compare dynamic parts
        // and return only those that have changed
        self.dynamic_parts.iter().collect()
    }
}

/// Parser for enhanced templates with hot reload capabilities
pub(crate) struct EnhancedTemplateParser {
    /// Current parsing location
    location: Option<TemplateLocation>,
}

impl EnhancedTemplateParser {
    /// Create a new enhanced template parser
    pub(crate) fn new() -> Self {
        Self {
            location: None,
        }
    }

    /// Parse an enhanced template from token stream
    pub(crate) fn parse(&mut self, input: ParseStream) -> syn::Result<EnhancedTree> {
        // Get the current location for tracking
        let location = TemplateLocation::from_span(input.span());
        self.location = Some(location.clone());

        // Parse the regular tree first
        let tree = Tree::parse(input)?;

        // Convert to enhanced tree
        Ok(EnhancedTree::from_tree(tree, location))
    }

    /// Parse with explicit location information
    pub(crate) fn parse_with_location(
        &mut self,
        input: ParseStream,
        file: String,
        line: u32,
        column: u32,
    ) -> syn::Result<EnhancedTree> {
        let location = TemplateLocation::new(file, line, column, input.span());
        self.location = Some(location.clone());

        let tree = Tree::parse(input)?;
        Ok(EnhancedTree::from_tree(tree, location))
    }
}

impl Default for EnhancedTemplateParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    #[test]
    fn test_template_location() {
        let span = Span::call_site();
        let location = TemplateLocation::from_span(span);
        
        assert!(!location.file.is_empty());
        assert!(location.line > 0);
        
        let id = location.location_id();
        assert!(id.contains(':'));
    }

    #[test]
    fn test_node_id() {
        let node_id = NodeId::new("template1", "0.1.2");
        assert_eq!(node_id.path, "0.1.2");
        assert!(node_id.id.contains("template1"));
        
        let child_id = node_id.child(3);
        assert_eq!(child_id.path, "0.1.2.3");
    }

    #[test]
    fn test_template_fingerprint() {
        let fp1 = TemplateFingerprint::new("content", "structure");
        let fp2 = TemplateFingerprint::new("content", "structure");
        let fp3 = TemplateFingerprint::new("different", "structure");
        
        assert!(fp1.matches(&fp2));
        assert!(!fp1.matches(&fp3));
        assert!(fp1.only_static_changed(&fp3));
        assert!(!fp1.structure_changed(&fp3));
    }

    #[test]
    fn test_dynamic_part_kinds() {
        let text_part = DynamicPart {
            kind: DynamicPartKind::TextContent,
            location: TemplateLocation::from_span(Span::call_site()),
            code: quote! { "test" },
            dependencies: vec![],
        };
        
        assert_eq!(text_part.kind, DynamicPartKind::TextContent);
        
        let attr_part = DynamicPart {
            kind: DynamicPartKind::AttributeValue { attr_name: "class".to_string() },
            location: TemplateLocation::from_span(Span::call_site()),
            code: quote! { "value" },
            dependencies: vec![],
        };
        
        match attr_part.kind {
            DynamicPartKind::AttributeValue { attr_name } => {
                assert_eq!(attr_name, "class");
            }
            _ => panic!("Wrong dynamic part kind"),
        }
    }
}

// Implement NodeToTokens for enhanced types to maintain compatibility
impl NodeToTokens for EnhancedTree {
    fn node_to_tokens(&self, fixed: &mut FixedParts, out: &mut TokenStream) {
        for node in &self.nodes {
            node.node_to_tokens(fixed, out);
        }
    }
}

impl NodeToTokens for EnhancedHtmlNode {
    fn node_to_tokens(&self, fixed: &mut FixedParts, out: &mut TokenStream) {
        // Delegate to the underlying HTML node
        self.node.node_to_tokens(fixed, out);
    }
}

// Implement ToTokens for types that need it for quote! macro
impl ToTokens for If<Tree> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Convert to a basic representation for quoting
        let cond = &self.cond;
        tokens.extend(quote! {
            if #cond { /* template content */ }
        });
    }
}

impl<T> ToTokens for If<Box<T>> where T: ToTokens {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cond = &self.cond;
        tokens.extend(quote! {
            if #cond { /* attribute content */ }
        });
    }
}

impl ToTokens for For {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pat = &self.pat;
        let expr = &self.expr;
        tokens.extend(quote! {
            for #pat in #expr { /* template content */ }
        });
    }
}

impl ToTokens for Match {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        tokens.extend(quote! {
            match #expr { /* arms */ }
        });
    }
}

// Add specific implementations for attribute value types
impl ToTokens for NormalAttrValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! { /* normal attr value */ });
    }
}

impl ToTokens for AxmAttrValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! { /* axm attr value */ });
    }
}