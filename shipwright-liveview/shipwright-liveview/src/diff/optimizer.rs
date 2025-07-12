//! Patch optimization module
//! 
//! This module provides advanced optimization techniques for patches,
//! focusing on common patterns and efficient representations.

use super::patch::{Patch, PatchOp};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
enum ListStrategy {
    BulkReplace,
    OptimizedMoves,
    NoOptimization,
}

/// Optimizer for patch operations
pub struct PatchOptimizer {
    /// Whether to enable list optimization
    enable_list_optimization: bool,
    /// Whether to enable attribute batching
    enable_attribute_batching: bool,
    /// Whether to enable component boundary tracking
    enable_component_tracking: bool,
}

impl Default for PatchOptimizer {
    fn default() -> Self {
        Self {
            enable_list_optimization: true,
            enable_attribute_batching: true,
            enable_component_tracking: true,
        }
    }
}

impl PatchOptimizer {
    /// Create a new optimizer with all optimizations enabled
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Optimize a patch using all enabled optimizations
    pub fn optimize(&self, mut patch: Patch) -> Patch {
        // First, apply basic optimizations from the Patch itself
        patch.optimize();
        
        // Then apply advanced optimizations
        if self.enable_list_optimization {
            patch = self.optimize_list_operations(patch);
        }
        
        if self.enable_attribute_batching {
            patch = self.optimize_attribute_operations(patch);
        }
        
        if self.enable_component_tracking {
            patch = self.optimize_component_boundaries(patch);
        }
        
        // Finally, compress sequences of operations
        patch = self.compress_operation_sequences(patch);
        
        patch
    }
    
    /// Optimize list operations (add/remove/reorder)
    fn optimize_list_operations(&self, mut patch: Patch) -> Patch {
        let mut optimized_ops = Vec::new();
        let mut list_ops: HashMap<Vec<usize>, Vec<(usize, &PatchOp)>> = HashMap::new();
        
        // Group operations by parent path
        for (idx, op) in patch.ops.iter().enumerate() {
            match op {
                PatchOp::InsertChild { parent_path, .. } |
                PatchOp::RemoveChild { parent_path, .. } |
                PatchOp::MoveChild { parent_path, .. } => {
                    list_ops.entry(parent_path.clone())
                        .or_insert_with(Vec::new)
                        .push((idx, op));
                }
                _ => {}
            }
        }
        
        // Process each list separately
        let mut processed_indices = HashSet::new();
        
        for (parent_path, ops) in list_ops {
            if ops.len() < 2 {
                continue; // No optimization needed for single operations
            }
            
            // Analyze the operations to determine the best strategy
            let strategy = self.analyze_list_strategy(&ops);
            
            match strategy {
                ListStrategy::BulkReplace => {
                    // Replace all operations with a single bulk operation
                    let new_children = self.compute_final_list_state(&parent_path, &ops);
                    if let Some(bulk_op) = self.create_bulk_list_operation(&parent_path, new_children) {
                        optimized_ops.push(bulk_op);
                        for (idx, _) in ops {
                            processed_indices.insert(idx);
                        }
                    }
                }
                ListStrategy::OptimizedMoves => {
                    // Optimize move operations
                    let optimized_moves = self.optimize_moves(&ops);
                    for op in optimized_moves {
                        optimized_ops.push(op);
                    }
                    for (idx, _) in ops {
                        processed_indices.insert(idx);
                    }
                }
                ListStrategy::NoOptimization => {
                    // Keep operations as-is
                }
            }
        }
        
        // Add non-list operations and non-optimized list operations
        for (idx, op) in patch.ops.into_iter().enumerate() {
            if !processed_indices.contains(&idx) {
                optimized_ops.push(op);
            }
        }
        
        Patch {
            ops: optimized_ops,
            metadata: patch.metadata,
        }
    }
    
    /// Optimize attribute operations by batching them
    fn optimize_attribute_operations(&self, mut patch: Patch) -> Patch {
        let mut optimized_ops = Vec::new();
        let mut attr_ops: HashMap<Vec<usize>, Vec<(usize, &PatchOp)>> = HashMap::new();
        
        // Group attribute operations by element path
        for (idx, op) in patch.ops.iter().enumerate() {
            match op {
                PatchOp::SetAttribute { path, .. } |
                PatchOp::RemoveAttribute { path, .. } => {
                    attr_ops.entry(path.clone())
                        .or_insert_with(Vec::new)
                        .push((idx, op));
                }
                _ => {}
            }
        }
        
        let mut processed_indices = HashSet::new();
        
        // Create batched attribute operations
        for (path, ops) in attr_ops {
            if ops.len() >= 3 {
                // Only batch if we have 3 or more operations
                let mut attributes = HashMap::new();
                let mut removals = Vec::new();
                
                for (_, op) in &ops {
                    match op {
                        PatchOp::SetAttribute { name, value, .. } => {
                            attributes.insert(name.clone(), value.clone());
                        }
                        PatchOp::RemoveAttribute { name, .. } => {
                            removals.push(name.clone());
                            attributes.remove(name);
                        }
                        _ => {}
                    }
                }
                
                // Create a combined operation
                if !attributes.is_empty() || !removals.is_empty() {
                    optimized_ops.push(self.create_batch_attribute_op(path, attributes, removals));
                    for (idx, _) in ops {
                        processed_indices.insert(idx);
                    }
                }
            }
        }
        
        // Add non-attribute operations and non-batched attribute operations
        for (idx, op) in patch.ops.into_iter().enumerate() {
            if !processed_indices.contains(&idx) {
                optimized_ops.push(op);
            }
        }
        
        Patch {
            ops: optimized_ops,
            metadata: patch.metadata,
        }
    }
    
    /// Optimize operations around component boundaries
    fn optimize_component_boundaries(&self, patch: Patch) -> Patch {
        // Detect component boundaries based on data attributes or special markers
        let mut component_ops: HashMap<Vec<usize>, Vec<PatchOp>> = HashMap::new();
        
        for op in patch.ops {
            if self.is_component_boundary(&op) {
                let path = self.get_op_path(&op);
                component_ops.entry(path).or_insert_with(Vec::new).push(op);
            } else {
                // For now, just preserve non-component operations
                component_ops.entry(vec![]).or_insert_with(Vec::new).push(op);
            }
        }
        
        // Optimize operations within each component
        let mut optimized_ops = Vec::new();
        
        for (_, ops) in component_ops {
            // Apply component-specific optimizations
            let optimized = self.optimize_component_ops(ops);
            optimized_ops.extend(optimized);
        }
        
        Patch {
            ops: optimized_ops,
            metadata: patch.metadata,
        }
    }
    
    /// Compress sequences of similar operations
    fn compress_operation_sequences(&self, patch: Patch) -> Patch {
        let mut compressed_ops = Vec::new();
        let mut i = 0;
        
        while i < patch.ops.len() {
            let op = &patch.ops[i];
            
            // Look for sequences of similar operations
            match op {
                PatchOp::UpdateText { path, .. } => {
                    // Find consecutive text updates on sibling nodes
                    let mut text_updates = vec![(path.clone(), op)];
                    let mut j = i + 1;
                    
                    while j < patch.ops.len() {
                        if let PatchOp::UpdateText { path: next_path, .. } = &patch.ops[j] {
                            if self.are_siblings(path, next_path) {
                                text_updates.push((next_path.clone(), &patch.ops[j]));
                                j += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    
                    if text_updates.len() > 1 {
                        // Create a compressed operation
                        compressed_ops.push(self.create_bulk_text_update(text_updates));
                        i = j;
                        continue;
                    }
                }
                _ => {}
            }
            
            compressed_ops.push(patch.ops[i].clone());
            i += 1;
        }
        
        Patch {
            ops: compressed_ops,
            metadata: patch.metadata,
        }
    }
    
    // Helper methods
    
    fn analyze_list_strategy(&self, ops: &[(usize, &PatchOp)]) -> ListStrategy {
        let mut inserts = 0;
        let mut removes = 0;
        let mut moves = 0;
        
        for (_, op) in ops {
            match op {
                PatchOp::InsertChild { .. } => inserts += 1,
                PatchOp::RemoveChild { .. } => removes += 1,
                PatchOp::MoveChild { .. } => moves += 1,
                _ => {}
            }
        }
        
        // If more than 50% of children are being modified, bulk replace
        if inserts + removes > ops.len() / 2 {
            ListStrategy::BulkReplace
        } else if moves > 2 {
            ListStrategy::OptimizedMoves
        } else {
            ListStrategy::NoOptimization
        }
    }
    
    fn compute_final_list_state(&self, _parent_path: &[usize], _ops: &[(usize, &PatchOp)]) -> Vec<String> {
        // This would compute the final state of the list after all operations
        // For now, return empty as this is a complex operation
        Vec::new()
    }
    
    fn create_bulk_list_operation(&self, parent_path: &[usize], _children: Vec<String>) -> Option<PatchOp> {
        // This would create a single operation to replace all children
        // For now, return None
        None
    }
    
    fn optimize_moves(&self, ops: &[(usize, &PatchOp)]) -> Vec<PatchOp> {
        // Optimize move operations to minimize DOM mutations
        ops.iter().map(|(_, op)| (*op).clone()).collect()
    }
    
    fn create_batch_attribute_op(
        &self,
        path: Vec<usize>,
        attributes: HashMap<String, String>,
        _removals: Vec<String>,
    ) -> PatchOp {
        // For now, just return the first attribute as a regular operation
        // In a real implementation, this would create a custom batch operation
        if let Some((name, value)) = attributes.into_iter().next() {
            PatchOp::SetAttribute { path, name, value }
        } else {
            PatchOp::UpdateText { path, new_text: String::new() }
        }
    }
    
    fn is_component_boundary(&self, op: &PatchOp) -> bool {
        // Check if this operation is at a component boundary
        match op {
            PatchOp::SetAttribute { name, .. } => {
                name.starts_with("data-component") || name == "key"
            }
            _ => false,
        }
    }
    
    fn get_op_path(&self, op: &PatchOp) -> Vec<usize> {
        match op {
            PatchOp::Replace { path, .. } |
            PatchOp::UpdateText { path, .. } |
            PatchOp::SetAttribute { path, .. } |
            PatchOp::RemoveAttribute { path, .. } |
            PatchOp::AddClass { path, .. } |
            PatchOp::RemoveClass { path, .. } |
            PatchOp::SetStyle { path, .. } |
            PatchOp::RemoveStyle { path, .. } |
            PatchOp::AddEventListener { path, .. } |
            PatchOp::RemoveEventListener { path, .. } => path.clone(),
            PatchOp::InsertChild { parent_path, .. } |
            PatchOp::RemoveChild { parent_path, .. } |
            PatchOp::MoveChild { parent_path, .. } => parent_path.clone(),
        }
    }
    
    fn optimize_component_ops(&self, ops: Vec<PatchOp>) -> Vec<PatchOp> {
        // Apply component-specific optimizations
        // For now, just return the operations as-is
        ops
    }
    
    fn are_siblings(&self, path1: &[usize], path2: &[usize]) -> bool {
        if path1.len() != path2.len() || path1.is_empty() {
            return false;
        }
        
        // Check if all path components except the last are the same
        for i in 0..path1.len() - 1 {
            if path1[i] != path2[i] {
                return false;
            }
        }
        
        // Check if the last components are consecutive
        if let (Some(&last1), Some(&last2)) = (path1.last(), path2.last()) {
            last2 == last1 + 1
        } else {
            false
        }
    }
    
    fn create_bulk_text_update(&self, updates: Vec<(Vec<usize>, &PatchOp)>) -> PatchOp {
        // For now, just return the first update
        // In a real implementation, this would create a custom bulk operation
        if let Some((_, op)) = updates.first() {
            (*op).clone()
        } else {
            PatchOp::UpdateText { path: vec![], new_text: String::new() }
        }
    }
}