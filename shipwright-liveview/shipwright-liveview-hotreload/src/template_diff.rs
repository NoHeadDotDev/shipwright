//! Template diffing engine for AST-level comparison and hot reload optimization
//! 
//! This module provides a sophisticated diffing engine that compares templates at the AST level
//! to determine whether changes can be hot-reloaded or require a full rebuild.

use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::protocol::{DynamicKind, DynamicPart, TemplateId, TemplateUpdate};

/// Represents a node in the template AST
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateNode {
    /// Static HTML element
    Element {
        tag: String,
        attributes: Vec<Attribute>,
        children: Vec<TemplateNode>,
    },
    /// Text content
    Text(String),
    /// Dynamic expression
    Expression {
        /// The expression content
        content: String,
        /// Position in the template
        index: usize,
    },
    /// Conditional block
    Conditional {
        condition: String,
        then_branch: Vec<TemplateNode>,
        else_branch: Option<Vec<TemplateNode>>,
        index: usize,
    },
    /// Loop block
    Loop {
        iterator: String,
        iterable: String,
        body: Vec<TemplateNode>,
        index: usize,
    },
    /// Component reference
    Component {
        name: String,
        props: Vec<Attribute>,
        children: Vec<TemplateNode>,
    },
}

/// Attribute in a template
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

/// Attribute value types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue {
    /// Static string value
    Static(String),
    /// Dynamic expression
    Dynamic(String),
    /// Event handler
    EventHandler(String),
}

/// Result of diffing two templates
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Whether the templates are compatible for hot reload
    pub compatible: bool,
    /// Specific changes found
    pub changes: Vec<TemplateChange>,
    /// Compatibility issues preventing hot reload
    pub compatibility_issues: Vec<CompatibilityIssue>,
    /// Delta operations for applying the changes
    pub delta_ops: Vec<DeltaOperation>,
}

/// Types of template changes
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateChange {
    /// Text content changed
    TextChanged {
        path: Vec<usize>,
        old: String,
        new: String,
    },
    /// Attribute value changed
    AttributeChanged {
        path: Vec<usize>,
        attr_name: String,
        old: AttributeValue,
        new: AttributeValue,
    },
    /// Element added
    ElementAdded {
        path: Vec<usize>,
        node: TemplateNode,
    },
    /// Element removed
    ElementRemoved {
        path: Vec<usize>,
        node: TemplateNode,
    },
    /// Element replaced
    ElementReplaced {
        path: Vec<usize>,
        old: TemplateNode,
        new: TemplateNode,
    },
    /// Dynamic part changed
    DynamicPartChanged {
        index: usize,
        old: DynamicKind,
        new: DynamicKind,
    },
}

/// Issues that prevent hot reload compatibility
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityIssue {
    /// Root element type changed
    RootElementChanged { old: String, new: String },
    /// Dynamic part count mismatch
    DynamicPartCountMismatch { old: usize, new: usize },
    /// Dynamic part type changed at index
    DynamicPartTypeMismatch { index: usize, old: DynamicKind, new: DynamicKind },
    /// Component structure fundamentally changed
    StructuralChange { description: String },
    /// State preservation not possible
    StatePreservationIssue { description: String },
}

/// Delta operations for incremental updates
#[derive(Debug, Clone)]
pub enum DeltaOperation {
    /// Update text content
    UpdateText {
        path: Vec<usize>,
        content: String,
    },
    /// Update attribute
    UpdateAttribute {
        path: Vec<usize>,
        name: String,
        value: String,
    },
    /// Insert node
    InsertNode {
        parent_path: Vec<usize>,
        index: usize,
        html: String,
    },
    /// Remove node
    RemoveNode {
        path: Vec<usize>,
    },
    /// Replace node
    ReplaceNode {
        path: Vec<usize>,
        html: String,
    },
}

/// Template diffing engine
pub struct TemplateDiffer {
    /// Cache of parsed templates
    ast_cache: HashMap<String, TemplateAst>,
}

/// Parsed template AST with metadata
#[derive(Debug, Clone)]
pub struct TemplateAst {
    /// Root nodes of the template
    pub roots: Vec<TemplateNode>,
    /// Dynamic parts in the template
    pub dynamic_parts: Vec<DynamicPart>,
    /// Original HTML for reference
    pub original_html: String,
}

impl TemplateDiffer {
    /// Create a new template differ
    pub fn new() -> Self {
        Self {
            ast_cache: HashMap::new(),
        }
    }

    /// Diff two template updates
    pub fn diff_updates(&mut self, old: &TemplateUpdate, new: &TemplateUpdate) -> Result<DiffResult> {
        // Parse templates to AST if not cached
        let old_ast = self.parse_or_get_cached(&old.id.hash(), &old.html, &old.dynamic_parts)?;
        let new_ast = self.parse_or_get_cached(&new.id.hash(), &new.html, &new.dynamic_parts)?;

        // Perform the diff
        self.diff_templates(&old_ast, &new_ast)
    }

    /// Parse template or retrieve from cache
    fn parse_or_get_cached(
        &mut self,
        hash: &str,
        html: &str,
        dynamic_parts: &[DynamicPart],
    ) -> Result<TemplateAst> {
        if let Some(cached) = self.ast_cache.get(hash) {
            return Ok(cached.clone());
        }

        let ast = self.parse_template(html, dynamic_parts)?;
        self.ast_cache.insert(hash.to_string(), ast.clone());
        Ok(ast)
    }

    /// Parse HTML template into AST
    fn parse_template(&self, html: &str, dynamic_parts: &[DynamicPart]) -> Result<TemplateAst> {
        // For now, create a simple AST representation
        // In a full implementation, this would use a proper HTML parser
        let roots = vec![self.parse_simple_html(html, dynamic_parts)?];
        
        Ok(TemplateAst {
            roots,
            dynamic_parts: dynamic_parts.to_vec(),
            original_html: html.to_string(),
        })
    }

    /// Simple HTML parsing (placeholder for full parser)
    fn parse_simple_html(&self, html: &str, dynamic_parts: &[DynamicPart]) -> Result<TemplateNode> {
        // This is a simplified parser - in production, use html5ever or similar
        // For now, treat the entire content as a text node with dynamic parts
        Ok(TemplateNode::Text(html.to_string()))
    }

    /// Diff two template ASTs
    fn diff_templates(&self, old: &TemplateAst, new: &TemplateAst) -> Result<DiffResult> {
        let mut changes = Vec::new();
        let mut compatibility_issues = Vec::new();
        let mut delta_ops = Vec::new();

        // Check dynamic parts compatibility
        if old.dynamic_parts.len() != new.dynamic_parts.len() {
            compatibility_issues.push(CompatibilityIssue::DynamicPartCountMismatch {
                old: old.dynamic_parts.len(),
                new: new.dynamic_parts.len(),
            });
        } else {
            // Check each dynamic part type
            for (i, (old_part, new_part)) in old.dynamic_parts.iter().zip(&new.dynamic_parts).enumerate() {
                if !self.dynamic_parts_compatible(&old_part.kind, &new_part.kind) {
                    compatibility_issues.push(CompatibilityIssue::DynamicPartTypeMismatch {
                        index: i,
                        old: old_part.kind.clone(),
                        new: new_part.kind.clone(),
                    });
                    changes.push(TemplateChange::DynamicPartChanged {
                        index: i,
                        old: old_part.kind.clone(),
                        new: new_part.kind.clone(),
                    });
                }
            }
        }

        // Diff the root nodes
        if old.roots.len() != new.roots.len() {
            compatibility_issues.push(CompatibilityIssue::StructuralChange {
                description: "Root node count changed".to_string(),
            });
        } else {
            for (i, (old_root, new_root)) in old.roots.iter().zip(&new.roots).enumerate() {
                self.diff_nodes(old_root, new_root, vec![i], &mut changes, &mut compatibility_issues, &mut delta_ops)?;
            }
        }

        let compatible = compatibility_issues.is_empty();

        Ok(DiffResult {
            compatible,
            changes,
            compatibility_issues,
            delta_ops,
        })
    }

    /// Check if two dynamic part kinds are compatible
    fn dynamic_parts_compatible(&self, old: &DynamicKind, new: &DynamicKind) -> bool {
        match (old, new) {
            (DynamicKind::Expression, DynamicKind::Expression) => true,
            (DynamicKind::EventHandler { event: e1 }, DynamicKind::EventHandler { event: e2 }) => e1 == e2,
            (DynamicKind::Conditional, DynamicKind::Conditional) => true,
            (DynamicKind::Loop, DynamicKind::Loop) => true,
            _ => false,
        }
    }

    /// Recursively diff two nodes
    fn diff_nodes(
        &self,
        old: &TemplateNode,
        new: &TemplateNode,
        path: Vec<usize>,
        changes: &mut Vec<TemplateChange>,
        compatibility_issues: &mut Vec<CompatibilityIssue>,
        delta_ops: &mut Vec<DeltaOperation>,
    ) -> Result<()> {
        match (old, new) {
            (TemplateNode::Text(old_text), TemplateNode::Text(new_text)) => {
                if old_text != new_text {
                    changes.push(TemplateChange::TextChanged {
                        path: path.clone(),
                        old: old_text.clone(),
                        new: new_text.clone(),
                    });
                    delta_ops.push(DeltaOperation::UpdateText {
                        path,
                        content: new_text.clone(),
                    });
                }
            }
            (TemplateNode::Element { tag: old_tag, attributes: old_attrs, children: old_children },
             TemplateNode::Element { tag: new_tag, attributes: new_attrs, children: new_children }) => {
                if old_tag != new_tag {
                    if path.len() == 1 {
                        compatibility_issues.push(CompatibilityIssue::RootElementChanged {
                            old: old_tag.clone(),
                            new: new_tag.clone(),
                        });
                    }
                    changes.push(TemplateChange::ElementReplaced {
                        path: path.clone(),
                        old: old.clone(),
                        new: new.clone(),
                    });
                    return Ok(());
                }

                // Diff attributes
                self.diff_attributes(old_attrs, new_attrs, &path, changes, delta_ops);

                // Diff children using Myers algorithm
                self.diff_children(old_children, new_children, path, changes, compatibility_issues, delta_ops)?;
            }
            _ => {
                // Different node types - not compatible
                changes.push(TemplateChange::ElementReplaced {
                    path: path.clone(),
                    old: old.clone(),
                    new: new.clone(),
                });
                compatibility_issues.push(CompatibilityIssue::StructuralChange {
                    description: format!("Node type changed at path {:?}", path),
                });
            }
        }

        Ok(())
    }

    /// Diff attributes between two elements
    fn diff_attributes(
        &self,
        old_attrs: &[Attribute],
        new_attrs: &[Attribute],
        path: &[usize],
        changes: &mut Vec<TemplateChange>,
        delta_ops: &mut Vec<DeltaOperation>,
    ) {
        let old_map: HashMap<_, _> = old_attrs.iter().map(|a| (&a.name, &a.value)).collect();
        let new_map: HashMap<_, _> = new_attrs.iter().map(|a| (&a.name, &a.value)).collect();

        // Check for changed/removed attributes
        for (name, old_value) in &old_map {
            if let Some(new_value) = new_map.get(name) {
                if old_value != new_value {
                    changes.push(TemplateChange::AttributeChanged {
                        path: path.to_vec(),
                        attr_name: name.to_string(),
                        old: (*old_value).clone(),
                        new: (*new_value).clone(),
                    });
                    if let AttributeValue::Static(value) = new_value {
                        delta_ops.push(DeltaOperation::UpdateAttribute {
                            path: path.to_vec(),
                            name: name.to_string(),
                            value: value.clone(),
                        });
                    }
                }
            }
        }
    }

    /// Diff children using a greedy algorithm (similar to Dioxus)
    fn diff_children(
        &self,
        old_children: &[TemplateNode],
        new_children: &[TemplateNode],
        mut parent_path: Vec<usize>,
        changes: &mut Vec<TemplateChange>,
        compatibility_issues: &mut Vec<CompatibilityIssue>,
        delta_ops: &mut Vec<DeltaOperation>,
    ) -> Result<()> {
        // Simple greedy matching for now
        let max_len = old_children.len().max(new_children.len());
        
        for i in 0..max_len {
            let mut child_path = parent_path.clone();
            child_path.push(i);

            match (old_children.get(i), new_children.get(i)) {
                (Some(old_child), Some(new_child)) => {
                    self.diff_nodes(old_child, new_child, child_path, changes, compatibility_issues, delta_ops)?;
                }
                (Some(old_child), None) => {
                    changes.push(TemplateChange::ElementRemoved {
                        path: child_path.clone(),
                        node: old_child.clone(),
                    });
                    delta_ops.push(DeltaOperation::RemoveNode { path: child_path });
                }
                (None, Some(new_child)) => {
                    changes.push(TemplateChange::ElementAdded {
                        path: child_path.clone(),
                        node: new_child.clone(),
                    });
                    // In real implementation, render the node to HTML
                    delta_ops.push(DeltaOperation::InsertNode {
                        parent_path: parent_path.clone(),
                        index: i,
                        html: "<!-- new node -->".to_string(),
                    });
                }
                (None, None) => unreachable!(),
            }
        }

        Ok(())
    }
}

/// Enhanced template parser that produces a proper AST
pub struct EnhancedTemplateParser;

impl EnhancedTemplateParser {
    /// Parse template HTML into an AST
    pub fn parse(html: &str) -> Result<Vec<TemplateNode>> {
        // This would integrate with html5ever or similar
        // For now, return a placeholder
        Ok(vec![TemplateNode::Text(html.to_string())])
    }
}

/// Compatibility checker for determining hot reload eligibility
pub struct CompatibilityChecker {
    rules: Vec<Box<dyn CompatibilityRule>>,
}

/// Trait for compatibility rules
pub trait CompatibilityRule: Send + Sync {
    /// Check if the change is compatible
    fn check(&self, old: &TemplateAst, new: &TemplateAst, changes: &[TemplateChange]) -> Option<CompatibilityIssue>;
}

impl CompatibilityChecker {
    /// Create a new compatibility checker with default rules
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(RootElementRule),
                Box::new(DynamicPartRule),
                Box::new(StatePreservationRule),
            ],
        }
    }

    /// Check if templates are compatible for hot reload
    pub fn check(&self, old: &TemplateAst, new: &TemplateAst, changes: &[TemplateChange]) -> Vec<CompatibilityIssue> {
        self.rules
            .iter()
            .filter_map(|rule| rule.check(old, new, changes))
            .collect()
    }
}

/// Rule: Root element must not change type
struct RootElementRule;

impl CompatibilityRule for RootElementRule {
    fn check(&self, old: &TemplateAst, new: &TemplateAst, _changes: &[TemplateChange]) -> Option<CompatibilityIssue> {
        // Check if root elements have same type
        match (old.roots.first(), new.roots.first()) {
            (Some(TemplateNode::Element { tag: old_tag, .. }), Some(TemplateNode::Element { tag: new_tag, .. })) => {
                if old_tag != new_tag {
                    Some(CompatibilityIssue::RootElementChanged {
                        old: old_tag.clone(),
                        new: new_tag.clone(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Rule: Dynamic parts must maintain compatible types
struct DynamicPartRule;

impl CompatibilityRule for DynamicPartRule {
    fn check(&self, old: &TemplateAst, new: &TemplateAst, _changes: &[TemplateChange]) -> Option<CompatibilityIssue> {
        if old.dynamic_parts.len() != new.dynamic_parts.len() {
            return Some(CompatibilityIssue::DynamicPartCountMismatch {
                old: old.dynamic_parts.len(),
                new: new.dynamic_parts.len(),
            });
        }
        None
    }
}

/// Rule: State preservation must be possible
struct StatePreservationRule;

impl CompatibilityRule for StatePreservationRule {
    fn check(&self, _old: &TemplateAst, _new: &TemplateAst, changes: &[TemplateChange]) -> Option<CompatibilityIssue> {
        // Check if any changes would break state preservation
        for change in changes {
            if let TemplateChange::ElementReplaced { .. } = change {
                return Some(CompatibilityIssue::StatePreservationIssue {
                    description: "Element replacement breaks state preservation".to_string(),
                });
            }
        }
        None
    }
}

/// Batch operation builder for efficient updates
pub struct BatchOperationBuilder {
    operations: Vec<DeltaOperation>,
}

impl BatchOperationBuilder {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Add a delta operation
    pub fn add_operation(&mut self, op: DeltaOperation) {
        self.operations.push(op);
    }

    /// Build optimized batch operations
    pub fn build(self) -> Vec<DeltaOperation> {
        // Optimize operations (e.g., merge adjacent text updates)
        self.optimize_operations(self.operations)
    }

    fn optimize_operations(&self, ops: Vec<DeltaOperation>) -> Vec<DeltaOperation> {
        // Simple optimization: deduplicate operations on same path
        let mut optimized = Vec::new();
        let mut seen_paths = HashSet::new();

        for op in ops.into_iter().rev() {
            let path_key = match &op {
                DeltaOperation::UpdateText { path, .. } |
                DeltaOperation::UpdateAttribute { path, .. } |
                DeltaOperation::RemoveNode { path } |
                DeltaOperation::ReplaceNode { path, .. } => {
                    format!("{:?}", path)
                }
                DeltaOperation::InsertNode { parent_path, index, .. } => {
                    format!("{:?}:{}", parent_path, index)
                }
            };

            if seen_paths.insert(path_key) {
                optimized.push(op);
            }
        }

        optimized.reverse();
        optimized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_text_diff() {
        let mut differ = TemplateDiffer::new();
        
        let old = TemplateUpdate {
            id: TemplateId::new("test.rs".into(), 1, 1),
            hash: "old".to_string(),
            content_hash: "old_content".to_string(),
            html: "<div>Hello</div>".to_string(),
            dynamic_parts: vec![],
        };

        let new = TemplateUpdate {
            id: TemplateId::new("test.rs".into(), 1, 1),
            hash: "new".to_string(),
            content_hash: "new_content".to_string(),
            html: "<div>World</div>".to_string(),
            dynamic_parts: vec![],
        };

        let result = differ.diff_updates(&old, &new).unwrap();
        assert!(result.compatible);
        assert!(!result.changes.is_empty());
    }

    #[test]
    fn test_dynamic_part_compatibility() {
        let differ = TemplateDiffer::new();
        
        assert!(differ.dynamic_parts_compatible(
            &DynamicKind::Expression,
            &DynamicKind::Expression
        ));
        
        assert!(!differ.dynamic_parts_compatible(
            &DynamicKind::Expression,
            &DynamicKind::Loop
        ));
        
        assert!(differ.dynamic_parts_compatible(
            &DynamicKind::EventHandler { event: "click".to_string() },
            &DynamicKind::EventHandler { event: "click".to_string() }
        ));
        
        assert!(!differ.dynamic_parts_compatible(
            &DynamicKind::EventHandler { event: "click".to_string() },
            &DynamicKind::EventHandler { event: "mouseover".to_string() }
        ));
    }

    #[test]
    fn test_batch_operation_optimization() {
        let mut builder = BatchOperationBuilder::new();
        
        builder.add_operation(DeltaOperation::UpdateText {
            path: vec![0, 1],
            content: "First".to_string(),
        });
        
        builder.add_operation(DeltaOperation::UpdateText {
            path: vec![0, 1],
            content: "Second".to_string(),
        });
        
        let ops = builder.build();
        assert_eq!(ops.len(), 1); // Should deduplicate
    }
}