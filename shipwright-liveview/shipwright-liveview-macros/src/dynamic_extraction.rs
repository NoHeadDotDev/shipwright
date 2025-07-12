//! Sophisticated dynamic part extraction for efficient hot reload updates.
//!
//! This module provides advanced analysis of template dynamic parts to enable
//! granular hot reload updates by identifying exactly what needs to change
//! when template code is modified.

use proc_macro2::{TokenStream, TokenTree, Delimiter, Span};
use quote::{quote, ToTokens};
use syn::{
    visit::{self, Visit},
    Expr, Block, Ident, Path, Type, Pat, Stmt, Item,
    ExprCall, ExprMethodCall, ExprField, ExprPath,
};
use std::collections::{HashMap, HashSet, BTreeMap};

use crate::{
    enhanced_ast::{DynamicPart, DynamicPartKind, TemplateLocation},
    HtmlNode, TagNode, Attr, If, For, Match,
};

/// Advanced dynamic part extractor with dependency analysis
pub(crate) struct DynamicPartExtractor {
    /// Configuration for extraction
    config: ExtractionConfig,
    /// Current extraction context
    context: ExtractionContext,
    /// Dependency analyzer
    dependency_analyzer: DependencyAnalyzer,
    /// Variable scope tracker
    scope_tracker: ScopeTracker,
}

/// Configuration for dynamic part extraction
#[derive(Debug, Clone)]
pub(crate) struct ExtractionConfig {
    /// Extract variable dependencies
    pub extract_variables: bool,
    /// Extract function call dependencies
    pub extract_functions: bool,
    /// Extract type dependencies
    pub extract_types: bool,
    /// Extract macro dependencies
    pub extract_macros: bool,
    /// Track variable mutations
    pub track_mutations: bool,
    /// Analyze closures and async blocks
    pub analyze_closures: bool,
    /// Maximum analysis depth
    pub max_depth: usize,
}

/// Context for the current extraction operation
#[derive(Debug, Clone, Default)]
struct ExtractionContext {
    /// Current depth in the AST
    depth: usize,
    /// Variables in current scope
    scope_variables: HashSet<String>,
    /// Current function/method context
    current_function: Option<String>,
    /// Whether we're inside a closure
    in_closure: bool,
    /// Whether we're inside an async context
    in_async: bool,
}

/// Advanced dependency analyzer using syn AST visitor
pub(crate) struct DependencyAnalyzer {
    /// Found dependencies
    dependencies: AnalyzedDependencies,
    /// Current analysis context
    context: AnalysisContext,
}

/// Comprehensive dependency information
#[derive(Debug, Clone, Default)]
pub(crate) struct AnalyzedDependencies {
    /// Variable references
    pub variables: BTreeMap<String, VariableUsage>,
    /// Function calls
    pub functions: BTreeMap<String, FunctionUsage>,
    /// Type references
    pub types: BTreeMap<String, TypeUsage>,
    /// Macro invocations
    pub macros: BTreeMap<String, MacroUsage>,
    /// External crate dependencies
    pub external_crates: HashSet<String>,
    /// Closure captures
    pub closures: Vec<ClosureAnalysis>,
}

/// Variable usage analysis
#[derive(Debug, Clone)]
pub(crate) struct VariableUsage {
    /// Variable name
    pub name: String,
    /// How the variable is used
    pub usage_type: VariableUsageType,
    /// Locations where used
    pub locations: Vec<Span>,
    /// Whether the variable is mutated
    pub is_mutated: bool,
    /// Whether it's captured in a closure
    pub captured_in_closure: bool,
}

/// Types of variable usage
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum VariableUsageType {
    /// Read-only access
    Read,
    /// Mutable access
    Write,
    /// Both read and write
    ReadWrite,
    /// Passed as argument
    Argument,
    /// Used in pattern matching
    Pattern,
}

/// Function call analysis
#[derive(Debug, Clone)]
pub(crate) struct FunctionUsage {
    /// Function name or path
    pub name: String,
    /// Full path if available
    pub full_path: Option<String>,
    /// Call type (function, method, etc.)
    pub call_type: FunctionCallType,
    /// Arguments passed
    pub argument_count: usize,
    /// Whether it's an async call
    pub is_async: bool,
    /// Locations where called
    pub locations: Vec<Span>,
}

/// Types of function calls
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FunctionCallType {
    /// Free function call
    Function,
    /// Method call on an object
    Method,
    /// Associated function call
    Associated,
    /// Macro invocation
    Macro,
    /// Closure call
    Closure,
}

/// Type usage analysis
#[derive(Debug, Clone)]
pub(crate) struct TypeUsage {
    /// Type name
    pub name: String,
    /// Full type path
    pub full_path: Option<String>,
    /// How the type is used
    pub usage_type: TypeUsageType,
    /// Generic parameters
    pub generic_params: Vec<String>,
    /// Locations where used
    pub locations: Vec<Span>,
}

/// Types of type usage
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TypeUsageType {
    /// Type annotation
    Annotation,
    /// Constructor call
    Constructor,
    /// Pattern matching
    Pattern,
    /// Generic parameter
    Generic,
    /// Trait bound
    TraitBound,
}

/// Macro usage analysis
#[derive(Debug, Clone)]
pub(crate) struct MacroUsage {
    /// Macro name
    pub name: String,
    /// Macro path
    pub path: Option<String>,
    /// Locations where invoked
    pub locations: Vec<Span>,
    /// Whether it's a procedural macro
    pub is_proc_macro: bool,
}

/// Closure analysis
#[derive(Debug, Clone)]
pub(crate) struct ClosureAnalysis {
    /// Variables captured by the closure
    pub captures: Vec<String>,
    /// Capture mode (by reference, by value, etc.)
    pub capture_mode: CaptureMode,
    /// Whether the closure is async
    pub is_async: bool,
    /// Location of the closure
    pub location: Span,
}

/// Closure capture modes
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CaptureMode {
    /// Capture by reference
    Ref,
    /// Capture by mutable reference
    RefMut,
    /// Capture by value
    Value,
    /// Mixed capture modes
    Mixed,
}

/// Current analysis context
#[derive(Debug, Clone, Default)]
struct AnalysisContext {
    /// Current depth in analysis
    depth: usize,
    /// Variables in scope
    scope_vars: HashSet<String>,
    /// Whether in async context
    in_async: bool,
    /// Whether in closure
    in_closure: bool,
}

/// Scope tracking for accurate dependency analysis
pub(crate) struct ScopeTracker {
    /// Stack of scopes
    scopes: Vec<Scope>,
    /// Global scope variables
    global_scope: HashSet<String>,
}

/// Individual scope information
#[derive(Debug, Clone, Default)]
struct Scope {
    /// Variables defined in this scope
    variables: HashSet<String>,
    /// Functions defined in this scope
    functions: HashSet<String>,
    /// Types defined in this scope
    types: HashSet<String>,
    /// Scope type
    scope_type: ScopeType,
}

/// Types of scopes
#[derive(Debug, Clone, PartialEq, Default)]
enum ScopeType {
    #[default]
    Block,
    Function,
    Closure,
    Module,
    Impl,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            extract_variables: true,
            extract_functions: true,
            extract_types: true,
            extract_macros: true,
            track_mutations: true,
            analyze_closures: true,
            max_depth: 10,
        }
    }
}

impl DynamicPartExtractor {
    /// Create a new dynamic part extractor
    pub(crate) fn new() -> Self {
        Self::with_config(ExtractionConfig::default())
    }

    /// Create extractor with custom configuration
    pub(crate) fn with_config(config: ExtractionConfig) -> Self {
        Self {
            config,
            context: ExtractionContext::default(),
            dependency_analyzer: DependencyAnalyzer::new(),
            scope_tracker: ScopeTracker::new(),
        }
    }

    /// Extract dynamic parts from an HTML node
    pub(crate) fn extract_from_node(
        &mut self,
        node: &HtmlNode,
        location: TemplateLocation,
    ) -> Vec<DynamicPart> {
        let mut parts = Vec::new();
        self.extract_node_recursive(node, location, &mut parts);
        parts
    }

    /// Extract dynamic parts from multiple nodes
    pub(crate) fn extract_from_nodes(
        &mut self,
        nodes: &[HtmlNode],
        base_location: TemplateLocation,
    ) -> Vec<DynamicPart> {
        let mut all_parts = Vec::new();
        
        for (index, node) in nodes.iter().enumerate() {
            let mut location = base_location.clone();
            location.template_path = if location.template_path.is_empty() {
                index.to_string()
            } else {
                format!("{}.{}", location.template_path, index)
            };
            
            let parts = self.extract_from_node(node, location);
            all_parts.extend(parts);
        }
        
        all_parts
    }

    /// Recursively extract dynamic parts from a node
    fn extract_node_recursive(
        &mut self,
        node: &HtmlNode,
        location: TemplateLocation,
        parts: &mut Vec<DynamicPart>,
    ) {
        if self.context.depth >= self.config.max_depth {
            return;
        }

        self.context.depth += 1;

        match node {
            HtmlNode::TagNode(tag) => {
                self.extract_from_tag(tag, location, parts);
            }
            HtmlNode::Block(block) => {
                let analyzed_deps = self.analyze_block_dependencies(block);
                let part = DynamicPart {
                    kind: DynamicPartKind::Block,
                    location,
                    code: block.to_token_stream(),
                    dependencies: analyzed_deps.to_string_vec(),
                };
                parts.push(part);
            }
            HtmlNode::If(if_node) => {
                self.extract_from_if(if_node, location, parts);
            }
            HtmlNode::For(for_node) => {
                self.extract_from_for(for_node, location, parts);
            }
            HtmlNode::Match(match_node) => {
                self.extract_from_match(match_node, location, parts);
            }
            HtmlNode::LitStr(_) | HtmlNode::Doctype(_) => {
                // Static content, no dynamic parts
            }
        }

        self.context.depth -= 1;
    }

    /// Extract dynamic parts from a tag node
    fn extract_from_tag(
        &mut self,
        tag: &TagNode,
        location: TemplateLocation,
        parts: &mut Vec<DynamicPart>,
    ) {
        // Extract from attributes
        for (attr_index, attr) in tag.attrs.iter().enumerate() {
            let mut attr_location = location.clone();
            attr_location.template_path = format!("{}.attr.{}", location.template_path, attr_index);
            
            self.extract_from_attribute(attr, attr_location, parts);
        }

        // Extract from children
        if let Some(tag_close) = &tag.close {
            for (child_index, child) in tag_close.inner.iter().enumerate() {
                let mut child_location = location.clone();
                child_location.template_path = format!("{}.child.{}", location.template_path, child_index);
                
                self.extract_node_recursive(child, child_location, parts);
            }
        }
    }

    /// Extract dynamic parts from an attribute
    fn extract_from_attribute(
        &mut self,
        attr: &Attr,
        location: TemplateLocation,
        parts: &mut Vec<DynamicPart>,
    ) {
        match attr {
            Attr::Normal { ident, value } => {
                let attr_name = self.attr_ident_to_string(ident);
                
                match value {
                    crate::NormalAttrValue::Block(block) => {
                        let analyzed_deps = self.analyze_block_dependencies(block);
                        let part = DynamicPart {
                            kind: DynamicPartKind::AttributeValue { attr_name },
                            location,
                            code: block.to_token_stream(),
                            dependencies: analyzed_deps.to_string_vec(),
                        };
                        parts.push(part);
                    }
                    crate::NormalAttrValue::If(if_node) => {
                        let analyzed_deps = self.analyze_expr_dependencies(&if_node.cond);
                        let part = DynamicPart {
                            kind: DynamicPartKind::AttributePresence { attr_name },
                            location,
                            code: quote! { #if_node },
                            dependencies: analyzed_deps.to_string_vec(),
                        };
                        parts.push(part);
                    }
                    _ => {
                        // Static attribute values
                    }
                }
            }
            Attr::Axm { ident, value } => {
                let event_name = self.attr_ident_to_string(ident);
                
                match value {
                    crate::AxmAttrValue::Block(block) => {
                        let analyzed_deps = self.analyze_block_dependencies(block);
                        let part = DynamicPart {
                            kind: DynamicPartKind::EventHandler { event_name },
                            location,
                            code: block.to_token_stream(),
                            dependencies: analyzed_deps.to_string_vec(),
                        };
                        parts.push(part);
                    }
                    crate::AxmAttrValue::If(if_node) => {
                        let analyzed_deps = self.analyze_expr_dependencies(&if_node.cond);
                        let part = DynamicPart {
                            kind: DynamicPartKind::EventHandler { event_name },
                            location,
                            code: quote! { #if_node },
                            dependencies: analyzed_deps.to_string_vec(),
                        };
                        parts.push(part);
                    }
                }
            }
        }
    }

    /// Extract dynamic parts from if construct
    fn extract_from_if(
        &mut self,
        if_node: &If<crate::Tree>,
        location: TemplateLocation,
        parts: &mut Vec<DynamicPart>,
    ) {
        let analyzed_deps = self.analyze_expr_dependencies(&if_node.cond);
        let part = DynamicPart {
            kind: DynamicPartKind::ControlFlow,
            location: location.clone(),
            code: quote! { #if_node },
            dependencies: analyzed_deps.to_string_vec(),
        };
        parts.push(part);

        // Extract from then branch
        let mut then_location = location.clone();
        then_location.template_path = format!("{}.then", location.template_path);
        for (index, node) in if_node.then_tree.nodes.iter().enumerate() {
            let mut node_location = then_location.clone();
            node_location.template_path = format!("{}.{}", then_location.template_path, index);
            self.extract_node_recursive(node, node_location, parts);
        }

        // Extract from else branch
        if let Some(else_branch) = &if_node.else_tree {
            let mut else_location = location.clone();
            else_location.template_path = format!("{}.else", location.template_path);
            
            match else_branch {
                crate::ElseBranch::If(else_if) => {
                    self.extract_from_if(else_if, else_location, parts);
                }
                crate::ElseBranch::Else(else_tree) => {
                    for (index, node) in else_tree.nodes.iter().enumerate() {
                        let mut node_location = else_location.clone();
                        node_location.template_path = format!("{}.{}", else_location.template_path, index);
                        self.extract_node_recursive(node, node_location, parts);
                    }
                }
            }
        }
    }

    /// Extract dynamic parts from for loop
    fn extract_from_for(
        &mut self,
        for_node: &For,
        location: TemplateLocation,
        parts: &mut Vec<DynamicPart>,
    ) {
        let analyzed_deps = self.analyze_expr_dependencies(&for_node.expr);
        let part = DynamicPart {
            kind: DynamicPartKind::ControlFlow,
            location: location.clone(),
            code: quote! { #for_node },
            dependencies: analyzed_deps.to_string_vec(),
        };
        parts.push(part);

        // Extract from loop body
        for (index, node) in for_node.tree.nodes.iter().enumerate() {
            let mut node_location = location.clone();
            node_location.template_path = format!("{}.body.{}", location.template_path, index);
            self.extract_node_recursive(node, node_location, parts);
        }
    }

    /// Extract dynamic parts from match expression
    fn extract_from_match(
        &mut self,
        match_node: &Match,
        location: TemplateLocation,
        parts: &mut Vec<DynamicPart>,
    ) {
        let analyzed_deps = self.analyze_expr_dependencies(&match_node.expr);
        let part = DynamicPart {
            kind: DynamicPartKind::ControlFlow,
            location: location.clone(),
            code: quote! { #match_node },
            dependencies: analyzed_deps.to_string_vec(),
        };
        parts.push(part);

        // Extract from match arms
        for (arm_index, arm) in match_node.arms.iter().enumerate() {
            for (node_index, node) in arm.tree.nodes.iter().enumerate() {
                let mut node_location = location.clone();
                node_location.template_path = format!(
                    "{}.arm.{}.{}",
                    location.template_path,
                    arm_index,
                    node_index
                );
                self.extract_node_recursive(node, node_location, parts);
            }
        }
    }

    /// Convert attribute identifier to string
    fn attr_ident_to_string(&self, ident: &crate::AttrIdent) -> String {
        match ident {
            crate::AttrIdent::Lit(name) => name.clone(),
            crate::AttrIdent::Axm(name) => name.clone(),
        }
    }

    /// Analyze dependencies in a Rust expression
    fn analyze_expr_dependencies(&mut self, expr: &Expr) -> AnalyzedDependencies {
        self.dependency_analyzer.analyze_expression(expr)
    }

    /// Analyze dependencies in a Rust block
    fn analyze_block_dependencies(&mut self, block: &Block) -> AnalyzedDependencies {
        self.dependency_analyzer.analyze_block(block)
    }
}

impl Default for DynamicPartExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyAnalyzer {
    /// Create a new dependency analyzer
    pub(crate) fn new() -> Self {
        Self {
            dependencies: AnalyzedDependencies::default(),
            context: AnalysisContext::default(),
        }
    }

    /// Analyze dependencies in an expression
    pub(crate) fn analyze_expression(&mut self, expr: &Expr) -> AnalyzedDependencies {
        self.dependencies = AnalyzedDependencies::default();
        self.visit_expr(expr);
        self.dependencies.clone()
    }

    /// Analyze dependencies in a block
    pub(crate) fn analyze_block(&mut self, block: &Block) -> AnalyzedDependencies {
        self.dependencies = AnalyzedDependencies::default();
        self.visit_block(block);
        self.dependencies.clone()
    }
}

impl<'ast> Visit<'ast> for DependencyAnalyzer {
    fn visit_expr_path(&mut self, expr: &'ast ExprPath) {
        if let Some(ident) = expr.path.get_ident() {
            let var_name = ident.to_string();
            
            if !self.context.scope_vars.contains(&var_name) {
                self.dependencies.variables.insert(
                    var_name.clone(),
                    VariableUsage {
                        name: var_name,
                        usage_type: VariableUsageType::Read,
                        locations: vec![ident.span()],
                        is_mutated: false,
                        captured_in_closure: self.context.in_closure,
                    },
                );
            }
        }
        
        visit::visit_expr_path(self, expr);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(path_expr) = &*call.func {
            if let Some(ident) = path_expr.path.get_ident() {
                let func_name = ident.to_string();
                
                self.dependencies.functions.insert(
                    func_name.clone(),
                    FunctionUsage {
                        name: func_name,
                        full_path: None,
                        call_type: FunctionCallType::Function,
                        argument_count: call.args.len(),
                        is_async: self.context.in_async,
                        locations: vec![ident.span()],
                    },
                );
            }
        }
        
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, method: &'ast ExprMethodCall) {
        let method_name = method.method.to_string();
        
        self.dependencies.functions.insert(
            method_name.clone(),
            FunctionUsage {
                name: method_name,
                full_path: None,
                call_type: FunctionCallType::Method,
                argument_count: method.args.len(),
                is_async: self.context.in_async,
                locations: vec![method.method.span()],
            },
        );
        
        visit::visit_expr_method_call(self, method);
    }

    fn visit_expr_field(&mut self, field: &'ast ExprField) {
        if let syn::Member::Named(ident) = &field.member {
            let field_name = ident.to_string();
            
            // Track field access as variable usage
            self.dependencies.variables.insert(
                field_name.clone(),
                VariableUsage {
                    name: field_name,
                    usage_type: VariableUsageType::Read,
                    locations: vec![ident.span()],
                    is_mutated: false,
                    captured_in_closure: self.context.in_closure,
                },
            );
        }
        
        visit::visit_expr_field(self, field);
    }
}

impl AnalyzedDependencies {
    /// Convert to a simple string vector for backward compatibility
    pub(crate) fn to_string_vec(&self) -> Vec<String> {
        let mut deps = Vec::new();
        
        deps.extend(self.variables.keys().cloned());
        deps.extend(self.functions.keys().cloned());
        deps.extend(self.types.keys().cloned());
        deps.extend(self.macros.keys().cloned());
        
        deps.sort();
        deps.dedup();
        deps
    }

    /// Get all variable names that are mutated
    pub(crate) fn mutated_variables(&self) -> Vec<&str> {
        self.variables
            .values()
            .filter(|usage| usage.is_mutated)
            .map(|usage| usage.name.as_str())
            .collect()
    }

    /// Get all function calls
    pub(crate) fn function_calls(&self) -> Vec<&str> {
        self.functions.keys().map(String::as_str).collect()
    }

    /// Get external crate dependencies
    pub(crate) fn external_dependencies(&self) -> Vec<&str> {
        self.external_crates.iter().map(String::as_str).collect()
    }
}

impl ScopeTracker {
    /// Create a new scope tracker
    pub(crate) fn new() -> Self {
        Self {
            scopes: Vec::new(),
            global_scope: HashSet::new(),
        }
    }

    /// Enter a new scope
    pub(crate) fn enter_scope(&mut self, scope_type: ScopeType) {
        self.scopes.push(Scope {
            scope_type,
            ..Default::default()
        });
    }

    /// Exit the current scope
    pub(crate) fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Add a variable to the current scope
    pub(crate) fn add_variable(&mut self, name: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.variables.insert(name);
        } else {
            self.global_scope.insert(name);
        }
    }

    /// Check if a variable is in scope
    pub(crate) fn is_variable_in_scope(&self, name: &str) -> bool {
        if self.global_scope.contains(name) {
            return true;
        }
        
        self.scopes
            .iter()
            .any(|scope| scope.variables.contains(name))
    }

    /// Get all variables in current scope
    pub(crate) fn current_scope_variables(&self) -> HashSet<String> {
        let mut vars = self.global_scope.clone();
        
        for scope in &self.scopes {
            vars.extend(scope.variables.iter().cloned());
        }
        
        vars
    }
}

impl Default for ScopeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_dynamic_part_extractor() {
        let mut extractor = DynamicPartExtractor::new();
        let location = TemplateLocation::from_span(Span::call_site());
        
        // Test with a simple block
        let block: Block = parse_quote! {{ self.value + 1 }};
        let analyzed = extractor.analyze_block_dependencies(&block);
        
        assert!(!analyzed.variables.is_empty());
        assert!(analyzed.to_string_vec().contains(&"self".to_string()));
    }

    #[test]
    fn test_dependency_analyzer() {
        let mut analyzer = DependencyAnalyzer::new();
        
        let expr: Expr = parse_quote! { self.value.method() };
        let deps = analyzer.analyze_expression(&expr);
        
        assert!(!deps.variables.is_empty());
        assert!(!deps.functions.is_empty());
    }

    #[test]
    fn test_scope_tracker() {
        let mut tracker = ScopeTracker::new();
        
        tracker.enter_scope(ScopeType::Function);
        tracker.add_variable("x".to_string());
        
        assert!(tracker.is_variable_in_scope("x"));
        
        tracker.exit_scope();
        assert!(!tracker.is_variable_in_scope("x"));
    }

    #[test]
    fn test_analyzed_dependencies_conversion() {
        let mut deps = AnalyzedDependencies::default();
        deps.variables.insert(
            "test_var".to_string(),
            VariableUsage {
                name: "test_var".to_string(),
                usage_type: VariableUsageType::Read,
                locations: vec![],
                is_mutated: false,
                captured_in_closure: false,
            },
        );
        
        let string_deps = deps.to_string_vec();
        assert!(string_deps.contains(&"test_var".to_string()));
    }

    #[test]
    fn test_extraction_config() {
        let config = ExtractionConfig {
            extract_variables: false,
            max_depth: 5,
            ..Default::default()
        };
        
        assert!(!config.extract_variables);
        assert_eq!(config.max_depth, 5);
        assert!(config.extract_functions);
    }
}