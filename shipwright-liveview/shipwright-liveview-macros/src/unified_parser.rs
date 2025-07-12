//! Unified template parser that extends the existing AST with hot reload capabilities.
//!
//! This parser builds upon the existing HTML parsing infrastructure while adding
//! sophisticated hot reload features including location tracking, fingerprinting,
//! and dynamic part extraction.

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Error as SynError,
};
use std::collections::HashMap;

use crate::{
    Tree, HtmlNode, TagNode, Attr, parse_many,
    enhanced_ast::{
        EnhancedTree, EnhancedHtmlNode, TemplateLocation, NodeId, TemplateMeta,
        DynamicPart, DynamicPartKind, HotReloadMeta, TemplateDependencies,
    },
    fingerprinting::{
        FingerprintEngine, TemplateStructure, FingerprintConfig,
        TemplateFingerprint, FingerprintComparison,
    },
    location_tracking::{
        LocationTracker, TrackedLocation, TemplateId, LocationRegistry,
        TemplateLocationInfo, TemplateMetadata, TemplateSizeInfo,
    },
    errors::{HtmlError, HtmlResult, HtmlContext, ErrorReporter},
};

/// Unified parser for templates with hot reload capabilities
pub(crate) struct UnifiedTemplateParser {
    /// Location tracker for template identification
    location_tracker: LocationTracker,
    /// Fingerprint engine for change detection
    fingerprint_engine: FingerprintEngine,
    /// Location registry for hot reload coordination
    location_registry: LocationRegistry,
    /// Error reporter for enhanced error messages
    error_reporter: ErrorReporter,
    /// Parser configuration
    config: UnifiedParserConfig,
    /// Current parsing context
    context: ParsingContext,
}

/// Configuration for the unified parser
#[derive(Debug, Clone)]
pub(crate) struct UnifiedParserConfig {
    /// Enable hot reload features
    pub enable_hot_reload: bool,
    /// Enable location tracking
    pub enable_location_tracking: bool,
    /// Enable fingerprinting
    pub enable_fingerprinting: bool,
    /// Enable enhanced error reporting
    pub enable_enhanced_errors: bool,
    /// Fingerprint configuration
    pub fingerprint_config: FingerprintConfig,
    /// Maximum template size for hot reload
    pub max_hot_reload_size: usize,
    /// Enable dependency tracking
    pub track_dependencies: bool,
}

/// Current parsing context
#[derive(Debug, Clone, Default)]
struct ParsingContext {
    /// Current template ID being parsed
    current_template_id: Option<String>,
    /// Current node path within template
    current_node_path: String,
    /// Current nesting depth
    nesting_depth: usize,
    /// Whether we're in a control flow construct
    in_control_flow: bool,
    /// Accumulated dynamic parts
    dynamic_parts: Vec<DynamicPart>,
    /// Template dependencies
    dependencies: TemplateDependencies,
}

/// Enhanced parsing result with hot reload metadata
#[derive(Debug, Clone)]
pub(crate) struct EnhancedParseResult {
    /// The enhanced tree with location tracking
    pub tree: EnhancedTree,
    /// Hot reload metadata
    pub hot_reload_meta: HotReloadMeta,
    /// Template fingerprint
    pub fingerprint: TemplateFingerprint,
    /// Parsing statistics
    pub stats: ParsingStats,
}

/// Statistics from the parsing process
#[derive(Debug, Clone, Default)]
pub(crate) struct ParsingStats {
    /// Total number of HTML elements parsed
    pub element_count: usize,
    /// Number of dynamic parts found
    pub dynamic_part_count: usize,
    /// Maximum nesting depth
    pub max_nesting_depth: usize,
    /// Total character count
    pub character_count: usize,
    /// Parsing duration (if measured)
    pub parse_duration: Option<std::time::Duration>,
}

impl Default for UnifiedParserConfig {
    fn default() -> Self {
        Self {
            enable_hot_reload: true,
            enable_location_tracking: true,
            enable_fingerprinting: true,
            enable_enhanced_errors: true,
            fingerprint_config: FingerprintConfig::default(),
            max_hot_reload_size: 10_000, // 10KB limit
            track_dependencies: true,
        }
    }
}

impl UnifiedTemplateParser {
    /// Create a new unified parser with default configuration
    pub(crate) fn new() -> Self {
        Self::with_config(UnifiedParserConfig::default())
    }

    /// Create a unified parser with custom configuration
    pub(crate) fn with_config(config: UnifiedParserConfig) -> Self {
        Self {
            location_tracker: LocationTracker::new(),
            fingerprint_engine: FingerprintEngine::with_config(config.fingerprint_config.clone()),
            location_registry: LocationRegistry::new(),
            error_reporter: ErrorReporter::new(),
            config,
            context: ParsingContext::default(),
        }
    }

    /// Parse a template with enhanced features
    pub(crate) fn parse_enhanced(&mut self, input: ParseStream) -> HtmlResult<EnhancedParseResult> {
        let start_time = std::time::Instant::now();
        let input_span = input.span();

        // Set up parsing context
        self.setup_parsing_context(input_span)?;

        // Parse the base tree first
        let base_tree = self.parse_base_tree(input)?;

        // Enhance the tree with hot reload features
        let enhanced_tree = self.enhance_tree(base_tree, input_span)?;

        // Generate hot reload metadata
        let hot_reload_meta = enhanced_tree.hot_reload_meta();

        // Generate fingerprint
        let fingerprint = self.generate_fingerprint(&enhanced_tree)?;

        // Collect parsing statistics
        let stats = self.collect_stats(&enhanced_tree, start_time);

        // Register the template for hot reload
        if self.config.enable_hot_reload {
            self.register_template(&enhanced_tree)?;
        }

        Ok(EnhancedParseResult {
            tree: enhanced_tree,
            hot_reload_meta,
            fingerprint,
            stats,
        })
    }

    /// Parse backward-compatible (returns regular Tree)
    pub(crate) fn parse_compatible(&mut self, input: ParseStream) -> syn::Result<Tree> {
        // For backward compatibility, just parse the base tree
        self.parse_base_tree(input).map_err(|e| e.into_syn_error())
    }

    /// Set up the parsing context
    fn setup_parsing_context(&mut self, span: Span) -> HtmlResult<()> {
        // Track the template location
        let location = if self.config.enable_location_tracking {
            self.location_tracker.track_location(span)
        } else {
            TrackedLocation {
                file_path: std::path::PathBuf::from("unknown"),
                relative_path: std::path::PathBuf::from("unknown"),
                line: 0,
                column: 0,
                location_id: "unknown".to_string(),
                template_id: "unknown".to_string(),
                template_path: String::new(),
                span,
            }
        };

        // Set up context
        self.context = ParsingContext {
            current_template_id: Some(location.template_id.clone()),
            current_node_path: String::new(),
            nesting_depth: 0,
            in_control_flow: false,
            dynamic_parts: Vec::new(),
            dependencies: TemplateDependencies::default(),
        };

        Ok(())
    }

    /// Parse the base tree using existing parser
    fn parse_base_tree(&mut self, input: ParseStream) -> HtmlResult<Tree> {
        let nodes = parse_many::<HtmlNode>(input)
            .map_err(|e| HtmlError::new(input.span(), e.to_string()))?;

        Ok(Tree { nodes })
    }

    /// Enhance a regular tree with hot reload features
    fn enhance_tree(&mut self, tree: Tree, span: Span) -> HtmlResult<EnhancedTree> {
        let location = TemplateLocation::from_span(span);
        let mut enhanced_tree = EnhancedTree::from_tree(tree, location);

        // Extract additional dynamic parts if needed
        if self.config.enable_hot_reload {
            self.extract_advanced_dynamic_parts(&mut enhanced_tree)?;
        }

        // Update dependencies
        enhanced_tree.template_meta.dependencies = self.context.dependencies.clone();

        Ok(enhanced_tree)
    }

    /// Extract advanced dynamic parts for hot reload
    fn extract_advanced_dynamic_parts(&mut self, tree: &mut EnhancedTree) -> HtmlResult<()> {
        for node in &mut tree.nodes {
            self.extract_node_dynamic_parts(node)?;
        }
        Ok(())
    }

    /// Extract dynamic parts from a single node
    fn extract_node_dynamic_parts(&mut self, node: &mut EnhancedHtmlNode) -> HtmlResult<()> {
        // Update context
        self.context.current_node_path = node.node_id.path.clone();
        
        let node_clone = node.node.clone();
        match &node_clone {
            HtmlNode::TagNode(tag) => {
                self.error_reporter.enter_element(tag.open.to_string());
                self.extract_tag_dynamic_parts(node, tag)?;
                self.error_reporter.exit_element();
            }
            HtmlNode::Block(block) => {
                let dependencies = self.extract_block_dependencies(block);
                self.context.dependencies.variables.extend(dependencies);
                
                // Create enhanced dynamic part
                let dynamic_part = DynamicPart {
                    kind: DynamicPartKind::Block,
                    location: node.location.clone(),
                    code: block.to_token_stream(),
                    dependencies: self.extract_block_dependencies(block),
                };
                
                node.dynamic_parts.push(dynamic_part);
            }
            HtmlNode::If(if_node) => {
                self.error_reporter.enter_control_flow();
                self.context.in_control_flow = true;
                
                let dependencies = self.extract_expr_dependencies(&if_node.cond);
                self.context.dependencies.variables.extend(dependencies.clone());
                
                let dynamic_part = DynamicPart {
                    kind: DynamicPartKind::ControlFlow,
                    location: node.location.clone(),
                    code: quote! { #if_node },
                    dependencies,
                };
                
                node.dynamic_parts.push(dynamic_part);
                
                self.context.in_control_flow = false;
                self.error_reporter.exit_control_flow();
            }
            HtmlNode::For(for_node) => {
                self.error_reporter.enter_control_flow();
                self.context.in_control_flow = true;
                
                let dependencies = self.extract_expr_dependencies(&for_node.expr);
                self.context.dependencies.variables.extend(dependencies.clone());
                
                let dynamic_part = DynamicPart {
                    kind: DynamicPartKind::ControlFlow,
                    location: node.location.clone(),
                    code: quote! { #for_node },
                    dependencies,
                };
                
                node.dynamic_parts.push(dynamic_part);
                
                self.context.in_control_flow = false;
                self.error_reporter.exit_control_flow();
            }
            HtmlNode::Match(match_node) => {
                self.error_reporter.enter_control_flow();
                self.context.in_control_flow = true;
                
                let dependencies = self.extract_expr_dependencies(&match_node.expr);
                self.context.dependencies.variables.extend(dependencies.clone());
                
                let dynamic_part = DynamicPart {
                    kind: DynamicPartKind::ControlFlow,
                    location: node.location.clone(),
                    code: quote! { #match_node },
                    dependencies,
                };
                
                node.dynamic_parts.push(dynamic_part);
                
                self.context.in_control_flow = false;
                self.error_reporter.exit_control_flow();
            }
            _ => {
                // Static nodes don't need additional processing
            }
        }

        Ok(())
    }

    /// Extract dynamic parts from tag attributes
    fn extract_tag_dynamic_parts(
        &mut self,
        node: &mut EnhancedHtmlNode,
        tag: &TagNode,
    ) -> HtmlResult<()> {
        for attr in &tag.attrs {
            self.extract_attr_dynamic_parts(node, attr)?;
        }
        Ok(())
    }

    /// Extract dynamic parts from a single attribute
    fn extract_attr_dynamic_parts(
        &mut self,
        node: &mut EnhancedHtmlNode,
        attr: &Attr,
    ) -> HtmlResult<()> {
        match attr {
            Attr::Normal { ident, value } => {
                let attr_name = self.attr_ident_to_string(ident);
                self.error_reporter.enter_attribute(attr_name.clone());
                
                match value {
                    crate::NormalAttrValue::Block(block) => {
                        let dependencies = self.extract_block_dependencies(block);
                        self.context.dependencies.variables.extend(dependencies.clone());
                        
                        let dynamic_part = DynamicPart {
                            kind: DynamicPartKind::AttributeValue { attr_name },
                            location: node.location.clone(),
                            code: block.to_token_stream(),
                            dependencies,
                        };
                        
                        node.dynamic_parts.push(dynamic_part);
                    }
                    crate::NormalAttrValue::If(if_node) => {
                        let dependencies = self.extract_expr_dependencies(&if_node.cond);
                        self.context.dependencies.variables.extend(dependencies.clone());
                        
                        let dynamic_part = DynamicPart {
                            kind: DynamicPartKind::AttributePresence { attr_name },
                            location: node.location.clone(),
                            code: quote! { #if_node },
                            dependencies,
                        };
                        
                        node.dynamic_parts.push(dynamic_part);
                    }
                    _ => {
                        // Static attribute values
                    }
                }
                
                self.error_reporter.exit_attribute();
            }
            Attr::Axm { ident, value } => {
                let event_name = self.attr_ident_to_string(ident);
                self.error_reporter.enter_attribute(event_name.clone());
                
                match value {
                    crate::AxmAttrValue::Block(block) => {
                        let dependencies = self.extract_block_dependencies(block);
                        self.context.dependencies.variables.extend(dependencies.clone());
                        
                        let dynamic_part = DynamicPart {
                            kind: DynamicPartKind::EventHandler { event_name },
                            location: node.location.clone(),
                            code: block.to_token_stream(),
                            dependencies,
                        };
                        
                        node.dynamic_parts.push(dynamic_part);
                    }
                    crate::AxmAttrValue::If(if_node) => {
                        let dependencies = self.extract_expr_dependencies(&if_node.cond);
                        self.context.dependencies.variables.extend(dependencies.clone());
                        
                        let dynamic_part = DynamicPart {
                            kind: DynamicPartKind::EventHandler { event_name },
                            location: node.location.clone(),
                            code: quote! { #if_node },
                            dependencies,
                        };
                        
                        node.dynamic_parts.push(dynamic_part);
                    }
                }
                
                self.error_reporter.exit_attribute();
            }
        }
        Ok(())
    }

    /// Convert attribute identifier to string
    fn attr_ident_to_string(&self, ident: &crate::AttrIdent) -> String {
        match ident {
            crate::AttrIdent::Lit(name) => name.clone(),
            crate::AttrIdent::Axm(name) => name.clone(),
        }
    }

    /// Extract dependencies from a Rust expression (enhanced)
    fn extract_expr_dependencies(&self, expr: &syn::Expr) -> Vec<String> {
        if !self.config.track_dependencies {
            return Vec::new();
        }

        let mut dependencies = Vec::new();
        
        // This is a simplified implementation
        // In a full implementation, you'd use syn::visit to traverse the entire AST
        let expr_str = quote! { #expr }.to_string();
        
        // Look for common patterns
        self.extract_dependencies_from_string(&expr_str, &mut dependencies);
        
        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    /// Extract dependencies from a Rust block (enhanced)
    fn extract_block_dependencies(&self, block: &syn::Block) -> Vec<String> {
        if !self.config.track_dependencies {
            return Vec::new();
        }

        let mut dependencies = Vec::new();
        
        let block_str = quote! { #block }.to_string();
        self.extract_dependencies_from_string(&block_str, &mut dependencies);
        
        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    /// Extract dependencies from a string representation (simplified)
    fn extract_dependencies_from_string(&self, code: &str, dependencies: &mut Vec<String>) {
        // This is a very simplified dependency extraction
        // A real implementation would parse the AST properly
        
        if code.contains("self.") {
            dependencies.push("self".to_string());
        }
        
        // Look for variable patterns
        let words: Vec<&str> = code.split_whitespace().collect();
        for window in words.windows(2) {
            if window[0] == "let" {
                if let Some(var_name) = window[1].strip_suffix('=') {
                    dependencies.push(var_name.trim().to_string());
                } else {
                    dependencies.push(window[1].to_string());
                }
            }
        }
        
        // Look for function calls (simplified)
        for word in words {
            if word.ends_with("()") || word.ends_with("(") {
                let func_name = word.trim_end_matches('(').trim_end_matches(')');
                if !func_name.is_empty() && func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    // Note: In a proper implementation, function dependencies would be collected
                    // in a mutable context or returned from this function
                    dependencies.push(format!("fn:{}", func_name));
                }
            }
        }
    }

    /// Generate fingerprint for the enhanced tree
    fn generate_fingerprint(&mut self, tree: &EnhancedTree) -> HtmlResult<TemplateFingerprint> {
        if !self.config.enable_fingerprinting {
            // Return a dummy fingerprint
            return Ok(TemplateFingerprint {
                combined_hash: 0,
                static_hash: 0,
                structure_hash: 0,
                dynamic_hash: 0,
                styling_hash: 0,
                interaction_hash: 0,
                part_hashes: HashMap::new(),
                hierarchy: crate::fingerprinting::HierarchyFingerprint {
                    level_hash: 0,
                    children: vec![],
                    element_type: crate::fingerprinting::ElementType::Root,
                },
            });
        }

        let template_id = &tree.template_meta.template_id;
        let static_content = &tree.template_meta.static_structure;
        let structure = TemplateStructure::from_html_nodes(&tree.nodes);
        let dynamic_parts = tree.dynamic_parts();

        let fingerprint = self.fingerprint_engine.fingerprint_template(
            template_id,
            static_content,
            &structure,
            &dynamic_parts.into_iter().cloned().collect::<Vec<_>>(),
        );

        Ok(fingerprint)
    }

    /// Collect parsing statistics
    fn collect_stats(&self, tree: &EnhancedTree, start_time: std::time::Instant) -> ParsingStats {
        let parse_duration = start_time.elapsed();
        
        let mut element_count = 0;
        let mut max_depth = 0;
        let mut character_count = 0;

        for node in &tree.nodes {
            self.count_node_stats(&node.node, &mut element_count, &mut max_depth, 0, &mut character_count);
        }

        ParsingStats {
            element_count,
            dynamic_part_count: tree.dynamic_parts().len(),
            max_nesting_depth: max_depth,
            character_count,
            parse_duration: Some(parse_duration),
        }
    }

    /// Count statistics for a single node recursively
    fn count_node_stats(
        &self,
        node: &HtmlNode,
        element_count: &mut usize,
        max_depth: &mut usize,
        current_depth: usize,
        character_count: &mut usize,
    ) {
        *max_depth = (*max_depth).max(current_depth);

        match node {
            HtmlNode::TagNode(tag) => {
                *element_count += 1;
                *character_count += tag.open.to_string().len();
                
                if let Some(tag_close) = &tag.close {
                    for child in &tag_close.inner {
                        self.count_node_stats(child, element_count, max_depth, current_depth + 1, character_count);
                    }
                    *character_count += tag_close.close.0.to_string().len();
                }
            }
            HtmlNode::LitStr(lit) => {
                *character_count += lit.value().len();
            }
            HtmlNode::If(if_node) => {
                // Count the if tree
                for child_node in &if_node.then_tree.nodes {
                    self.count_node_stats(child_node, element_count, max_depth, current_depth, character_count);
                }
                
                if let Some(else_branch) = &if_node.else_tree {
                    match else_branch {
                        crate::ElseBranch::If(else_if) => {
                            for child_node in &else_if.then_tree.nodes {
                                self.count_node_stats(child_node, element_count, max_depth, current_depth, character_count);
                            }
                        }
                        crate::ElseBranch::Else(else_tree) => {
                            for child_node in &else_tree.nodes {
                                self.count_node_stats(child_node, element_count, max_depth, current_depth, character_count);
                            }
                        }
                    }
                }
            }
            HtmlNode::For(for_node) => {
                for child_node in &for_node.tree.nodes {
                    self.count_node_stats(child_node, element_count, max_depth, current_depth, character_count);
                }
            }
            HtmlNode::Match(match_node) => {
                for arm in &match_node.arms {
                    for child_node in &arm.tree.nodes {
                        self.count_node_stats(child_node, element_count, max_depth, current_depth, character_count);
                    }
                }
            }
            _ => {
                // Other node types don't contribute to element count
            }
        }
    }

    /// Register template for hot reload tracking
    fn register_template(&mut self, tree: &EnhancedTree) -> HtmlResult<()> {
        let template_id = tree.template_meta.template_id.clone();
        
        let mut metadata = TemplateMetadata::default();
        metadata.size_info = TemplateSizeInfo {
            element_count: tree.nodes.len(),
            dynamic_count: tree.dynamic_parts().len(),
            max_depth: 0, // Would be calculated properly
            character_count: tree.template_meta.static_structure.len(),
        };
        
        // Convert TemplateLocation to TrackedLocation
        let primary_location = tree.nodes.first()
            .map(|n| {
                let loc = &n.location;
                TrackedLocation {
                    file_path: std::path::PathBuf::from(&loc.file),
                    relative_path: std::path::PathBuf::from(&loc.file),
                    line: loc.line,
                    column: loc.column,
                    location_id: format!("{}:{}:{}", loc.file, loc.line, loc.column),
                    template_id: template_id.clone(),
                    template_path: loc.template_path.clone(),
                    span: loc.span,
                }
            })
            .ok_or_else(|| HtmlError::new(Span::call_site(), "Empty template"))?;
        
        let all_locations: Vec<TrackedLocation> = tree.nodes.iter().map(|n| {
            let loc = &n.location;
            TrackedLocation {
                file_path: std::path::PathBuf::from(&loc.file),
                relative_path: std::path::PathBuf::from(&loc.file),
                line: loc.line,
                column: loc.column,
                location_id: format!("{}:{}:{}", loc.file, loc.line, loc.column),
                template_id: template_id.clone(),
                template_path: loc.template_path.clone(),
                span: loc.span,
            }
        }).collect();
        
        let info = TemplateLocationInfo {
            primary_location,
            all_locations,
            metadata,
            last_modified: Some(std::time::SystemTime::now()),
        };

        self.location_registry.register_template(template_id, info);
        Ok(())
    }

    /// Compare two templates for hot reload compatibility
    pub(crate) fn compare_for_hot_reload(
        &self,
        old_fingerprint: &TemplateFingerprint,
        new_fingerprint: &TemplateFingerprint,
    ) -> FingerprintComparison {
        self.fingerprint_engine.compare_fingerprints(old_fingerprint, new_fingerprint)
    }

    /// Get location registry for external access
    pub(crate) fn location_registry(&self) -> &LocationRegistry {
        &self.location_registry
    }

    /// Get parsing statistics from the location registry
    pub(crate) fn registry_stats(&self) -> crate::location_tracking::RegistryStats {
        self.location_registry.stats()
    }
}

impl Default for UnifiedTemplateParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function for parsing with hot reload features
pub(crate) fn parse_enhanced_template(input: ParseStream) -> HtmlResult<EnhancedParseResult> {
    let mut parser = UnifiedTemplateParser::new();
    parser.parse_enhanced(input)
}

/// Convenience function for backward-compatible parsing
pub(crate) fn parse_compatible_template(input: ParseStream) -> syn::Result<Tree> {
    let mut parser = UnifiedTemplateParser::new();
    parser.parse_compatible(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_unified_parser_creation() {
        let parser = UnifiedTemplateParser::new();
        assert!(parser.config.enable_hot_reload);
        assert!(parser.config.enable_location_tracking);
        assert!(parser.config.enable_fingerprinting);
    }

    #[test]
    fn test_parser_config() {
        let config = UnifiedParserConfig {
            enable_hot_reload: false,
            enable_location_tracking: true,
            ..Default::default()
        };

        let parser = UnifiedTemplateParser::with_config(config);
        assert!(!parser.config.enable_hot_reload);
        assert!(parser.config.enable_location_tracking);
    }

    #[test]
    fn test_parsing_stats() {
        let stats = ParsingStats {
            element_count: 5,
            dynamic_part_count: 2,
            max_nesting_depth: 3,
            character_count: 100,
            parse_duration: Some(std::time::Duration::from_millis(10)),
        };

        assert_eq!(stats.element_count, 5);
        assert_eq!(stats.dynamic_part_count, 2);
        assert_eq!(stats.max_nesting_depth, 3);
        assert!(stats.parse_duration.is_some());
    }

    #[test]
    fn test_dependency_extraction() {
        let parser = UnifiedTemplateParser::new();
        
        let code = "let x = self.value + some_func()";
        let mut deps = Vec::new();
        parser.extract_dependencies_from_string(code, &mut deps);
        
        assert!(deps.contains(&"self".to_string()));
        assert!(deps.contains(&"x".to_string()));
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that the parser can still produce regular Tree objects
        let input = quote! { <div>Hello</div> };
        let mut parser = UnifiedTemplateParser::new();
        
        // This should work without errors
        let parse_result = syn::parse2::<Tree>(input);
        assert!(parse_result.is_ok());
    }
}

// ToTokens implementations are already provided in enhanced_ast.rs