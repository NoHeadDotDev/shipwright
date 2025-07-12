//! Patch representation for DOM updates
//! 
//! This module defines the patch operations and provides efficient
//! serialization support for minimal wire transfer.

use serde::{Serialize, Deserialize};
use std::io::{Write, Read};

/// A patch containing a sequence of operations to transform one DOM tree to another
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Patch {
    /// The list of operations in the patch
    pub ops: Vec<PatchOp>,
    /// Optional metadata about the patch
    pub metadata: Option<PatchMetadata>,
}

/// Metadata about a patch for optimization purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchMetadata {
    /// Number of nodes affected
    pub affected_nodes: usize,
    /// Whether this patch includes component boundary changes
    pub has_component_changes: bool,
    /// Approximate size in bytes when serialized
    pub size_estimate: usize,
}

/// Individual patch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOp {
    /// Replace an entire node
    Replace {
        path: Vec<usize>,
        new_html: String,
    },
    /// Update text content
    UpdateText {
        path: Vec<usize>,
        new_text: String,
    },
    /// Set an attribute
    SetAttribute {
        path: Vec<usize>,
        name: String,
        value: String,
    },
    /// Remove an attribute
    RemoveAttribute {
        path: Vec<usize>,
        name: String,
    },
    /// Insert a child at a specific index
    InsertChild {
        parent_path: Vec<usize>,
        index: usize,
        html: String,
    },
    /// Remove a child at a specific index
    RemoveChild {
        parent_path: Vec<usize>,
        index: usize,
    },
    /// Move a child from one index to another
    MoveChild {
        parent_path: Vec<usize>,
        from_index: usize,
        to_index: usize,
    },
    /// Add a CSS class
    AddClass {
        path: Vec<usize>,
        class_name: String,
    },
    /// Remove a CSS class
    RemoveClass {
        path: Vec<usize>,
        class_name: String,
    },
    /// Set inline style
    SetStyle {
        path: Vec<usize>,
        property: String,
        value: String,
    },
    /// Remove inline style
    RemoveStyle {
        path: Vec<usize>,
        property: String,
    },
    /// Add event listener
    AddEventListener {
        path: Vec<usize>,
        event_type: String,
        handler_id: String,
    },
    /// Remove event listener
    RemoveEventListener {
        path: Vec<usize>,
        event_type: String,
        handler_id: String,
    },
}

impl Patch {
    /// Create a new empty patch
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            metadata: None,
        }
    }
    
    /// Add an operation to the patch
    pub fn add_op(&mut self, op: PatchOp) {
        self.ops.push(op);
    }
    
    /// Get the number of operations
    pub fn len(&self) -> usize {
        self.ops.len()
    }
    
    /// Check if the patch is empty
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
    
    /// Optimize the patch by combining and reordering operations
    pub fn optimize(&mut self) {
        // Remove redundant operations
        self.remove_redundant_ops();
        
        // Combine consecutive text updates
        self.combine_text_updates();
        
        // Reorder operations for better performance
        self.reorder_ops();
    }
    
    /// Remove operations that cancel each other out
    fn remove_redundant_ops(&mut self) {
        let mut new_ops = Vec::new();
        let mut skip_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();
        
        for (i, op) in self.ops.iter().enumerate() {
            if skip_indices.contains(&i) {
                continue;
            }
            
            let mut should_add = true;
            
            // Look for canceling operations
            match op {
                PatchOp::SetAttribute { path, name, .. } => {
                    // Check if there's a later SetAttribute for the same path and name
                    for (j, later_op) in self.ops.iter().enumerate().skip(i + 1) {
                        if let PatchOp::SetAttribute { path: later_path, name: later_name, .. } = later_op {
                            if path == later_path && name == later_name {
                                should_add = false;
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
            
            if should_add {
                new_ops.push(op.clone());
            }
        }
        
        self.ops = new_ops;
    }
    
    /// Combine consecutive text updates on the same node
    fn combine_text_updates(&mut self) {
        let mut new_ops = Vec::new();
        let mut i = 0;
        
        while i < self.ops.len() {
            match &self.ops[i] {
                PatchOp::UpdateText { path, .. } => {
                    let mut last_text = None;
                    let mut j = i;
                    
                    // Find all consecutive text updates for the same path
                    while j < self.ops.len() {
                        if let PatchOp::UpdateText { path: other_path, new_text } = &self.ops[j] {
                            if path == other_path {
                                last_text = Some(new_text.clone());
                                j += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    
                    // Only keep the last text update
                    if let Some(text) = last_text {
                        new_ops.push(PatchOp::UpdateText {
                            path: path.clone(),
                            new_text: text,
                        });
                    }
                    
                    i = j;
                }
                _ => {
                    new_ops.push(self.ops[i].clone());
                    i += 1;
                }
            }
        }
        
        self.ops = new_ops;
    }
    
    /// Reorder operations for better performance
    fn reorder_ops(&mut self) {
        // Sort operations by:
        // 1. Remove operations first (to avoid unnecessary work)
        // 2. Then by path depth (parent operations before children)
        // 3. Then by operation type priority
        self.ops.sort_by(|a, b| {
            let a_priority = op_priority(a);
            let b_priority = op_priority(b);
            
            if a_priority != b_priority {
                return a_priority.cmp(&b_priority);
            }
            
            // Sort by path depth
            let a_depth = op_path_depth(a);
            let b_depth = op_path_depth(b);
            
            a_depth.cmp(&b_depth)
        });
    }
    
    /// Serialize the patch to a compact binary format
    pub fn to_binary(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut buffer = Vec::new();
        
        // Write version byte
        buffer.write_all(&[1u8])?;
        
        // Write number of operations
        buffer.write_all(&(self.ops.len() as u32).to_le_bytes())?;
        
        // Write each operation
        for op in &self.ops {
            write_op(&mut buffer, op)?;
        }
        
        Ok(buffer)
    }
    
    /// Deserialize from binary format
    pub fn from_binary(data: &[u8]) -> Result<Self, std::io::Error> {
        let mut cursor = std::io::Cursor::new(data);
        
        // Read version
        let mut version = [0u8; 1];
        cursor.read_exact(&mut version)?;
        
        if version[0] != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unsupported patch version",
            ));
        }
        
        // Read number of operations
        let mut op_count_bytes = [0u8; 4];
        cursor.read_exact(&mut op_count_bytes)?;
        let op_count = u32::from_le_bytes(op_count_bytes) as usize;
        
        // Read operations
        let mut ops = Vec::with_capacity(op_count);
        for _ in 0..op_count {
            ops.push(read_op(&mut cursor)?);
        }
        
        Ok(Self {
            ops,
            metadata: None,
        })
    }
}

/// Get operation priority for sorting
fn op_priority(op: &PatchOp) -> u8 {
    match op {
        PatchOp::RemoveChild { .. } => 0,
        PatchOp::RemoveAttribute { .. } => 1,
        PatchOp::RemoveClass { .. } => 2,
        PatchOp::RemoveStyle { .. } => 3,
        PatchOp::RemoveEventListener { .. } => 4,
        PatchOp::MoveChild { .. } => 5,
        PatchOp::Replace { .. } => 6,
        PatchOp::InsertChild { .. } => 7,
        PatchOp::UpdateText { .. } => 8,
        PatchOp::SetAttribute { .. } => 9,
        PatchOp::AddClass { .. } => 10,
        PatchOp::SetStyle { .. } => 11,
        PatchOp::AddEventListener { .. } => 12,
    }
}

/// Get the depth of an operation's path
fn op_path_depth(op: &PatchOp) -> usize {
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
        PatchOp::RemoveEventListener { path, .. } => path.len(),
        PatchOp::InsertChild { parent_path, .. } |
        PatchOp::RemoveChild { parent_path, .. } |
        PatchOp::MoveChild { parent_path, .. } => parent_path.len(),
    }
}

/// Write an operation to a binary buffer
fn write_op(buffer: &mut Vec<u8>, op: &PatchOp) -> Result<(), std::io::Error> {
    // Write operation type
    let op_type = match op {
        PatchOp::Replace { .. } => 1,
        PatchOp::UpdateText { .. } => 2,
        PatchOp::SetAttribute { .. } => 3,
        PatchOp::RemoveAttribute { .. } => 4,
        PatchOp::InsertChild { .. } => 5,
        PatchOp::RemoveChild { .. } => 6,
        PatchOp::MoveChild { .. } => 7,
        PatchOp::AddClass { .. } => 8,
        PatchOp::RemoveClass { .. } => 9,
        PatchOp::SetStyle { .. } => 10,
        PatchOp::RemoveStyle { .. } => 11,
        PatchOp::AddEventListener { .. } => 12,
        PatchOp::RemoveEventListener { .. } => 13,
    };
    buffer.write_all(&[op_type])?;
    
    // Write operation-specific data
    match op {
        PatchOp::Replace { path, new_html } => {
            write_path(buffer, path)?;
            write_string(buffer, new_html)?;
        }
        PatchOp::UpdateText { path, new_text } => {
            write_path(buffer, path)?;
            write_string(buffer, new_text)?;
        }
        PatchOp::SetAttribute { path, name, value } => {
            write_path(buffer, path)?;
            write_string(buffer, name)?;
            write_string(buffer, value)?;
        }
        PatchOp::RemoveAttribute { path, name } => {
            write_path(buffer, path)?;
            write_string(buffer, name)?;
        }
        PatchOp::InsertChild { parent_path, index, html } => {
            write_path(buffer, parent_path)?;
            buffer.write_all(&(*index as u32).to_le_bytes())?;
            write_string(buffer, html)?;
        }
        PatchOp::RemoveChild { parent_path, index } => {
            write_path(buffer, parent_path)?;
            buffer.write_all(&(*index as u32).to_le_bytes())?;
        }
        PatchOp::MoveChild { parent_path, from_index, to_index } => {
            write_path(buffer, parent_path)?;
            buffer.write_all(&(*from_index as u32).to_le_bytes())?;
            buffer.write_all(&(*to_index as u32).to_le_bytes())?;
        }
        PatchOp::AddClass { path, class_name } => {
            write_path(buffer, path)?;
            write_string(buffer, class_name)?;
        }
        PatchOp::RemoveClass { path, class_name } => {
            write_path(buffer, path)?;
            write_string(buffer, class_name)?;
        }
        PatchOp::SetStyle { path, property, value } => {
            write_path(buffer, path)?;
            write_string(buffer, property)?;
            write_string(buffer, value)?;
        }
        PatchOp::RemoveStyle { path, property } => {
            write_path(buffer, path)?;
            write_string(buffer, property)?;
        }
        PatchOp::AddEventListener { path, event_type, handler_id } => {
            write_path(buffer, path)?;
            write_string(buffer, event_type)?;
            write_string(buffer, handler_id)?;
        }
        PatchOp::RemoveEventListener { path, event_type, handler_id } => {
            write_path(buffer, path)?;
            write_string(buffer, event_type)?;
            write_string(buffer, handler_id)?;
        }
    }
    
    Ok(())
}

/// Read an operation from a binary buffer
fn read_op(cursor: &mut std::io::Cursor<&[u8]>) -> Result<PatchOp, std::io::Error> {
    let mut op_type = [0u8; 1];
    cursor.read_exact(&mut op_type)?;
    
    match op_type[0] {
        1 => {
            let path = read_path(cursor)?;
            let new_html = read_string(cursor)?;
            Ok(PatchOp::Replace { path, new_html })
        }
        2 => {
            let path = read_path(cursor)?;
            let new_text = read_string(cursor)?;
            Ok(PatchOp::UpdateText { path, new_text })
        }
        3 => {
            let path = read_path(cursor)?;
            let name = read_string(cursor)?;
            let value = read_string(cursor)?;
            Ok(PatchOp::SetAttribute { path, name, value })
        }
        4 => {
            let path = read_path(cursor)?;
            let name = read_string(cursor)?;
            Ok(PatchOp::RemoveAttribute { path, name })
        }
        5 => {
            let parent_path = read_path(cursor)?;
            let mut index_bytes = [0u8; 4];
            cursor.read_exact(&mut index_bytes)?;
            let index = u32::from_le_bytes(index_bytes) as usize;
            let html = read_string(cursor)?;
            Ok(PatchOp::InsertChild { parent_path, index, html })
        }
        6 => {
            let parent_path = read_path(cursor)?;
            let mut index_bytes = [0u8; 4];
            cursor.read_exact(&mut index_bytes)?;
            let index = u32::from_le_bytes(index_bytes) as usize;
            Ok(PatchOp::RemoveChild { parent_path, index })
        }
        7 => {
            let parent_path = read_path(cursor)?;
            let mut from_bytes = [0u8; 4];
            cursor.read_exact(&mut from_bytes)?;
            let from_index = u32::from_le_bytes(from_bytes) as usize;
            let mut to_bytes = [0u8; 4];
            cursor.read_exact(&mut to_bytes)?;
            let to_index = u32::from_le_bytes(to_bytes) as usize;
            Ok(PatchOp::MoveChild { parent_path, from_index, to_index })
        }
        8 => {
            let path = read_path(cursor)?;
            let class_name = read_string(cursor)?;
            Ok(PatchOp::AddClass { path, class_name })
        }
        9 => {
            let path = read_path(cursor)?;
            let class_name = read_string(cursor)?;
            Ok(PatchOp::RemoveClass { path, class_name })
        }
        10 => {
            let path = read_path(cursor)?;
            let property = read_string(cursor)?;
            let value = read_string(cursor)?;
            Ok(PatchOp::SetStyle { path, property, value })
        }
        11 => {
            let path = read_path(cursor)?;
            let property = read_string(cursor)?;
            Ok(PatchOp::RemoveStyle { path, property })
        }
        12 => {
            let path = read_path(cursor)?;
            let event_type = read_string(cursor)?;
            let handler_id = read_string(cursor)?;
            Ok(PatchOp::AddEventListener { path, event_type, handler_id })
        }
        13 => {
            let path = read_path(cursor)?;
            let event_type = read_string(cursor)?;
            let handler_id = read_string(cursor)?;
            Ok(PatchOp::RemoveEventListener { path, event_type, handler_id })
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unknown operation type",
        )),
    }
}

/// Write a path to the buffer
fn write_path(buffer: &mut Vec<u8>, path: &[usize]) -> Result<(), std::io::Error> {
    buffer.write_all(&(path.len() as u16).to_le_bytes())?;
    for &index in path {
        buffer.write_all(&(index as u32).to_le_bytes())?;
    }
    Ok(())
}

/// Read a path from the buffer
fn read_path(cursor: &mut std::io::Cursor<&[u8]>) -> Result<Vec<usize>, std::io::Error> {
    let mut len_bytes = [0u8; 2];
    cursor.read_exact(&mut len_bytes)?;
    let len = u16::from_le_bytes(len_bytes) as usize;
    
    let mut path = Vec::with_capacity(len);
    for _ in 0..len {
        let mut index_bytes = [0u8; 4];
        cursor.read_exact(&mut index_bytes)?;
        path.push(u32::from_le_bytes(index_bytes) as usize);
    }
    
    Ok(path)
}

/// Write a string to the buffer
fn write_string(buffer: &mut Vec<u8>, s: &str) -> Result<(), std::io::Error> {
    let bytes = s.as_bytes();
    buffer.write_all(&(bytes.len() as u32).to_le_bytes())?;
    buffer.write_all(bytes)?;
    Ok(())
}

/// Read a string from the buffer
fn read_string(cursor: &mut std::io::Cursor<&[u8]>) -> Result<String, std::io::Error> {
    let mut len_bytes = [0u8; 4];
    cursor.read_exact(&mut len_bytes)?;
    let len = u32::from_le_bytes(len_bytes) as usize;
    
    let mut bytes = vec![0u8; len];
    cursor.read_exact(&mut bytes)?;
    
    String::from_utf8(bytes).map_err(|_| std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid UTF-8 in string",
    ))
}