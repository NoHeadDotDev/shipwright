//! Morphdom algorithm implementation for DOM diffing
//! 
//! This module provides the core diffing algorithm based on morphdom,
//! adapted to work with HTML strings and generate patch operations.

use super::parser::{HtmlNode, parse_html};
use super::patch::{Patch, PatchOp};
use std::collections::{HashMap, HashSet};

/// Options for controlling the diff behavior
#[derive(Debug, Clone, Default)]
pub struct DiffOptions {
    /// Whether to preserve whitespace-only text nodes
    pub preserve_whitespace: bool,
    /// Whether to use node keys for efficient list updates
    pub use_keys: bool,
    /// Whether to track component boundaries
    pub track_components: bool,
    /// Custom attributes to always update
    pub force_update_attrs: HashSet<String>,
}

/// Diff two HTML strings and generate a patch
pub fn diff_html(from: &str, to: &str, options: &DiffOptions) -> Result<Patch, DiffError> {
    let from_tree = parse_html(from)?;
    let to_tree = parse_html(to)?;
    
    let mut patch = Patch::new();
    let mut context = DiffContext::new(options);
    
    diff_nodes(&from_tree, &to_tree, &mut patch, &mut context)?;
    
    Ok(patch)
}

#[derive(Debug)]
pub enum DiffError {
    ParseError(String),
}

impl std::fmt::Display for DiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for DiffError {}

struct DiffContext<'a> {
    options: &'a DiffOptions,
    key_map: HashMap<String, usize>,
    node_counter: usize,
}

impl<'a> DiffContext<'a> {
    fn new(options: &'a DiffOptions) -> Self {
        Self {
            options,
            key_map: HashMap::new(),
            node_counter: 0,
        }
    }
    
    fn next_node_id(&mut self) -> usize {
        let id = self.node_counter;
        self.node_counter += 1;
        id
    }
}

/// Diff two nodes recursively
fn diff_nodes(
    from: &HtmlNode,
    to: &HtmlNode,
    patch: &mut Patch,
    context: &mut DiffContext,
) -> Result<(), DiffError> {
    match (from, to) {
        (HtmlNode::Element(from_elem), HtmlNode::Element(to_elem)) => {
            if from_elem.tag_name != to_elem.tag_name {
                // Different tags - replace the entire node
                patch.add_op(PatchOp::Replace {
                    path: from_elem.path.clone(),
                    new_html: to.to_html(),
                });
            } else {
                // Same tag - diff attributes and children
                diff_attributes(from_elem, to_elem, patch)?;
                diff_children(&from_elem.children, &to_elem.children, &from_elem.path, patch, context)?;
            }
        }
        (HtmlNode::Text(from_text), HtmlNode::Text(to_text)) => {
            if from_text.content != to_text.content {
                patch.add_op(PatchOp::UpdateText {
                    path: from_text.path.clone(),
                    new_text: to_text.content.clone(),
                });
            }
        }
        (HtmlNode::Comment(_), HtmlNode::Comment(_)) => {
            // Comments are ignored in diffing
        }
        _ => {
            // Node types don't match - replace
            patch.add_op(PatchOp::Replace {
                path: from.get_path().to_vec(),
                new_html: to.to_html(),
            });
        }
    }
    
    Ok(())
}

/// Diff attributes between two elements
fn diff_attributes(
    from: &super::parser::Element,
    to: &super::parser::Element,
    patch: &mut Patch,
) -> Result<(), DiffError> {
    // Check for removed attributes
    for (attr_name, _) in &from.attributes {
        if !to.attributes.contains_key(attr_name) {
            patch.add_op(PatchOp::RemoveAttribute {
                path: from.path.clone(),
                name: attr_name.clone(),
            });
        }
    }
    
    // Check for added or changed attributes
    for (attr_name, to_value) in &to.attributes {
        match from.attributes.get(attr_name) {
            Some(from_value) if from_value != to_value => {
                patch.add_op(PatchOp::SetAttribute {
                    path: from.path.clone(),
                    name: attr_name.clone(),
                    value: to_value.clone(),
                });
            }
            None => {
                patch.add_op(PatchOp::SetAttribute {
                    path: from.path.clone(),
                    name: attr_name.clone(),
                    value: to_value.clone(),
                });
            }
            _ => {} // Attribute unchanged
        }
    }
    
    Ok(())
}

/// Diff children using an optimized algorithm for list updates
fn diff_children(
    from_children: &[HtmlNode],
    to_children: &[HtmlNode],
    parent_path: &[usize],
    patch: &mut Patch,
    context: &mut DiffContext,
) -> Result<(), DiffError> {
    if from_children.is_empty() && to_children.is_empty() {
        return Ok(());
    }
    
    // Handle simple cases first
    if from_children.is_empty() {
        // Just insert all new children
        for (i, child) in to_children.iter().enumerate() {
            patch.add_op(PatchOp::InsertChild {
                parent_path: parent_path.to_vec(),
                index: i,
                html: child.to_html(),
            });
        }
        return Ok(());
    }
    
    if to_children.is_empty() {
        // Remove all children
        for i in (0..from_children.len()).rev() {
            patch.add_op(PatchOp::RemoveChild {
                parent_path: parent_path.to_vec(),
                index: i,
            });
        }
        return Ok(());
    }
    
    // Use key-based diffing if enabled and keys are present
    if context.options.use_keys && has_keys(from_children) && has_keys(to_children) {
        diff_keyed_children(from_children, to_children, parent_path, patch, context)?;
    } else {
        diff_unkeyed_children(from_children, to_children, parent_path, patch, context)?;
    }
    
    Ok(())
}

/// Check if children have keys
fn has_keys(children: &[HtmlNode]) -> bool {
    children.iter().any(|child| {
        if let HtmlNode::Element(elem) = child {
            elem.attributes.contains_key("key") || elem.attributes.contains_key("data-key")
        } else {
            false
        }
    })
}

/// Diff children without keys (simple index-based)
fn diff_unkeyed_children(
    from_children: &[HtmlNode],
    to_children: &[HtmlNode],
    parent_path: &[usize],
    patch: &mut Patch,
    context: &mut DiffContext,
) -> Result<(), DiffError> {
    let from_len = from_children.len();
    let to_len = to_children.len();
    let min_len = from_len.min(to_len);
    
    // Diff common children
    for i in 0..min_len {
        diff_nodes(&from_children[i], &to_children[i], patch, context)?;
    }
    
    // Handle extra children
    if from_len > to_len {
        // Remove extra children from the end
        for i in (to_len..from_len).rev() {
            patch.add_op(PatchOp::RemoveChild {
                parent_path: parent_path.to_vec(),
                index: i,
            });
        }
    } else if to_len > from_len {
        // Add extra children to the end
        for i in from_len..to_len {
            patch.add_op(PatchOp::InsertChild {
                parent_path: parent_path.to_vec(),
                index: i,
                html: to_children[i].to_html(),
            });
        }
    }
    
    Ok(())
}

/// Diff children with keys (optimized for list reordering)
fn diff_keyed_children(
    from_children: &[HtmlNode],
    to_children: &[HtmlNode],
    parent_path: &[usize],
    patch: &mut Patch,
    context: &mut DiffContext,
) -> Result<(), DiffError> {
    // Build key maps
    let mut from_keys: HashMap<String, usize> = HashMap::new();
    let mut to_keys: HashMap<String, usize> = HashMap::new();
    
    for (i, child) in from_children.iter().enumerate() {
        if let Some(key) = get_key(child) {
            from_keys.insert(key, i);
        }
    }
    
    for (i, child) in to_children.iter().enumerate() {
        if let Some(key) = get_key(child) {
            to_keys.insert(key, i);
        }
    }
    
    // Track which nodes have been processed
    let mut processed = vec![false; from_children.len()];
    let mut moves: Vec<(usize, usize)> = Vec::new();
    
    // First pass: match by keys and collect moves
    for (to_idx, to_child) in to_children.iter().enumerate() {
        if let Some(key) = get_key(to_child) {
            if let Some(&from_idx) = from_keys.get(&key) {
                processed[from_idx] = true;
                if from_idx != to_idx {
                    moves.push((from_idx, to_idx));
                }
                // Diff the matched nodes
                diff_nodes(&from_children[from_idx], to_child, patch, context)?;
            } else {
                // New keyed node
                patch.add_op(PatchOp::InsertChild {
                    parent_path: parent_path.to_vec(),
                    index: to_idx,
                    html: to_child.to_html(),
                });
            }
        }
    }
    
    // Remove nodes that weren't matched
    for (i, &was_processed) in processed.iter().enumerate().rev() {
        if !was_processed {
            patch.add_op(PatchOp::RemoveChild {
                parent_path: parent_path.to_vec(),
                index: i,
            });
        }
    }
    
    // Apply moves efficiently
    for (from_idx, to_idx) in moves {
        patch.add_op(PatchOp::MoveChild {
            parent_path: parent_path.to_vec(),
            from_index: from_idx,
            to_index: to_idx,
        });
    }
    
    Ok(())
}

/// Extract key from a node
fn get_key(node: &HtmlNode) -> Option<String> {
    if let HtmlNode::Element(elem) = node {
        elem.attributes.get("key")
            .or_else(|| elem.attributes.get("data-key"))
            .cloned()
    } else {
        None
    }
}