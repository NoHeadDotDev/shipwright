//! DOM diffing system optimized for hot reload operations
//! 
//! This module provides an enhanced DOM diffing system that integrates with
//! shipwright-liveview's morphdom implementation while adding hot reload specific
//! optimizations and state preservation capabilities.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
// Temporarily comment out to see other errors
// use shipwright_liveview::diff::{diff_html, DiffOptions, Patch, PatchOp};

use crate::runtime::{HotReloadError, HotReloadPatch, HotReloadPatchOp};

// Temporary stubs for compilation
#[derive(Debug)]
struct DiffOptions {
    preserve_whitespace: bool,
    use_keys: bool,
    track_components: bool,
    force_update_attrs: HashSet<String>,
}

#[derive(Debug)]
struct Patch {
    ops: Vec<PatchOp>,
}

#[derive(Debug)]
enum PatchOp {
    Replace { path: Vec<usize>, new_html: String },
    UpdateText { path: Vec<usize>, new_text: String },
    SetAttribute { path: Vec<usize>, name: String, value: String },
    RemoveAttribute { path: Vec<usize>, name: String },
    InsertChild { parent_path: Vec<usize>, index: usize, html: String },
    RemoveChild { parent_path: Vec<usize>, index: usize },
    MoveChild { parent_path: Vec<usize>, from_index: usize, to_index: usize },
}

fn diff_html(_from: &str, _to: &str, _options: &DiffOptions) -> Result<Patch, String> {
    // Stub implementation
    Ok(Patch { ops: vec![] })
}

/// Enhanced DOM diffing engine for hot reload
#[derive(Debug)]
pub struct HotReloadDomDiffer {
    /// Options for controlling diff behavior
    options: HotReloadDiffOptions,
    /// Cache of previous diffs for optimization
    diff_cache: HashMap<String, CachedDiff>,
    /// Component boundaries for targeted updates
    component_boundaries: HashMap<String, ComponentBoundary>,
}

/// Options for hot reload DOM diffing
#[derive(Debug, Clone)]
pub struct HotReloadDiffOptions {
    /// Whether to preserve input focus during updates
    pub preserve_focus: bool,
    /// Whether to preserve scroll positions
    pub preserve_scroll: bool,
    /// Whether to preserve form state
    pub preserve_form_state: bool,
    /// Whether to animate transitions
    pub animate_transitions: bool,
    /// Custom attributes that should trigger full component re-render
    pub component_triggers: HashSet<String>,
    /// Selectors for elements that should never be updated
    pub preserve_selectors: HashSet<String>,
    /// Maximum depth for nested component updates
    pub max_update_depth: usize,
}

impl Default for HotReloadDiffOptions {
    fn default() -> Self {
        let mut component_triggers = HashSet::new();
        component_triggers.insert("data-live-view".to_string());
        component_triggers.insert("data-component-id".to_string());
        
        let mut preserve_selectors = HashSet::new();
        preserve_selectors.insert("input[type=\"file\"]".to_string());
        preserve_selectors.insert("video".to_string());
        preserve_selectors.insert("audio".to_string());
        
        Self {
            preserve_focus: true,
            preserve_scroll: true,
            preserve_form_state: true,
            animate_transitions: false,
            component_triggers,
            preserve_selectors,
            max_update_depth: 10,
        }
    }
}

/// Cached diff result for optimization
#[derive(Debug, Clone)]
struct CachedDiff {
    /// Hash of the source HTML
    source_hash: String,
    /// Hash of the target HTML
    target_hash: String,
    /// The computed patch
    patch: HotReloadPatch,
    /// Timestamp when cached
    cached_at: std::time::SystemTime,
}

/// Component boundary information for targeted updates
#[derive(Debug, Clone)]
pub struct ComponentBoundary {
    /// Component identifier
    pub component_id: String,
    /// CSS selector for the component root
    pub root_selector: String,
    /// Whether this component can be updated independently
    pub isolated: bool,
    /// Child component boundaries
    pub children: Vec<ComponentBoundary>,
    /// State preservation data
    pub state_data: Option<serde_json::Value>,
}

/// State that should be preserved during DOM updates
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreservedDomState {
    /// Currently focused element selector
    pub focused_element: Option<String>,
    /// Scroll positions by selector
    pub scroll_positions: HashMap<String, ScrollPosition>,
    /// Form field values
    pub form_values: HashMap<String, FormValue>,
    /// Selection ranges for text inputs
    pub text_selections: HashMap<String, TextSelection>,
}

/// Scroll position data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollPosition {
    pub x: f64,
    pub y: f64,
}

/// Form field value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FormValue {
    Text(String),
    Boolean(bool),
    Number(f64),
    Array(Vec<String>),
}

/// Text selection range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSelection {
    pub start: u32,
    pub end: u32,
    pub direction: String,
}

impl HotReloadDomDiffer {
    /// Create a new DOM differ with default options
    pub fn new() -> Self {
        Self::with_options(HotReloadDiffOptions::default())
    }
    
    /// Create a new DOM differ with custom options
    pub fn with_options(options: HotReloadDiffOptions) -> Self {
        Self {
            options,
            diff_cache: HashMap::new(),
            component_boundaries: HashMap::new(),
        }
    }
    
    /// Register a component boundary for targeted updates
    pub fn register_component_boundary(&mut self, boundary: ComponentBoundary) {
        debug!("Registering component boundary: {}", boundary.component_id);
        self.component_boundaries.insert(boundary.component_id.clone(), boundary);
    }
    
    /// Generate an optimized DOM patch for hot reload
    pub fn generate_hot_reload_patch(
        &mut self,
        from_html: &str,
        to_html: &str,
        component_id: Option<&str>,
    ) -> Result<OptimizedPatch, HotReloadError> {
        debug!("Generating hot reload patch (component: {:?})", component_id);
        
        // Check cache first
        let cache_key = self.compute_cache_key(from_html, to_html, component_id);
        if let Some(cached) = self.diff_cache.get(&cache_key) {
            // Check if cache is still valid (1 minute)
            if cached.cached_at.elapsed().unwrap_or_default().as_secs() < 60 {
                debug!("Using cached diff result");
                return Ok(OptimizedPatch {
                    patch: cached.patch.clone(),
                    component_boundaries: self.get_affected_components(&cached.patch),
                    state_preservation: PreservedDomState::default(),
                    requires_full_refresh: false,
                });
            }
        }
        
        // Generate new diff
        let patch = self.compute_diff(from_html, to_html, component_id)?;
        
        // Optimize the patch
        let optimized = self.optimize_patch(patch, component_id)?;
        
        // Cache the result
        self.cache_diff(cache_key, from_html, to_html, &optimized.patch);
        
        Ok(optimized)
    }
    
    /// Compute the actual diff using shipwright-liveview's morphdom
    fn compute_diff(
        &self,
        from_html: &str,
        to_html: &str,
        component_id: Option<&str>,
    ) -> Result<HotReloadPatch, HotReloadError> {
        let diff_options = DiffOptions {
            preserve_whitespace: false,
            use_keys: true,
            track_components: true,
            force_update_attrs: self.options.component_triggers.clone(),
        };
        
        // Use shipwright-liveview's diff engine
        let patch = diff_html(from_html, to_html, &diff_options)
            .map_err(|e| HotReloadError::DomDiffingFailed { 
                reason: format!("Morphdom diff failed: {}", e) 
            })?;
        
        // Convert to hot reload format
        self.convert_patch_to_hot_reload(&patch, component_id)
    }
    
    /// Convert shipwright-liveview patch to hot reload format
    fn convert_patch_to_hot_reload(
        &self,
        patch: &Patch,
        component_id: Option<&str>,
    ) -> Result<HotReloadPatch, HotReloadError> {
        let operations = patch.ops
            .iter()
            .filter_map(|op| self.convert_patch_operation(op, component_id).ok())
            .collect();
        
        Ok(HotReloadPatch { operations })
    }
    
    /// Convert a single patch operation with hot reload optimizations
    fn convert_patch_operation(
        &self,
        op: &PatchOp,
        component_id: Option<&str>,
    ) -> Result<HotReloadPatchOp, HotReloadError> {
        match op {
            PatchOp::Replace { path, new_html } => {
                let selector = self.path_to_selector(path, component_id);
                
                // Check if this selector should be preserved
                if self.should_preserve_element(&selector) {
                    debug!("Skipping replace for preserved element: {}", selector);
                    return Err(HotReloadError::DomDiffingFailed { 
                        reason: "Element marked for preservation".to_string() 
                    });
                }
                
                Ok(HotReloadPatchOp::Replace {
                    selector,
                    html: new_html.clone(),
                })
            }
            PatchOp::UpdateText { path, new_text } => {
                Ok(HotReloadPatchOp::UpdateText {
                    selector: self.path_to_selector(path, component_id),
                    text: new_text.clone(),
                })
            }
            PatchOp::SetAttribute { path, name, value } => {
                let selector = self.path_to_selector(path, component_id);
                
                // Special handling for component trigger attributes
                if self.options.component_triggers.contains(name) {
                    debug!("Component trigger attribute changed: {}", name);
                }
                
                Ok(HotReloadPatchOp::SetAttribute {
                    selector,
                    name: name.clone(),
                    value: value.clone(),
                })
            }
            PatchOp::RemoveAttribute { path, name } => {
                Ok(HotReloadPatchOp::RemoveAttribute {
                    selector: self.path_to_selector(path, component_id),
                    name: name.clone(),
                })
            }
            PatchOp::InsertChild { parent_path, index, html } => {
                Ok(HotReloadPatchOp::InsertChild {
                    parent_selector: self.path_to_selector(parent_path, component_id),
                    index: *index,
                    html: html.clone(),
                })
            }
            PatchOp::RemoveChild { parent_path, index } => {
                Ok(HotReloadPatchOp::RemoveChild {
                    parent_selector: self.path_to_selector(parent_path, component_id),
                    index: *index,
                })
            }
            PatchOp::MoveChild { parent_path, from_index, to_index } => {
                Ok(HotReloadPatchOp::MoveChild {
                    parent_selector: self.path_to_selector(parent_path, component_id),
                    from_index: *from_index,
                    to_index: *to_index,
                })
            }
        }
    }
    
    /// Convert a path array to a CSS selector with component scoping
    fn path_to_selector(&self, path: &[usize], component_id: Option<&str>) -> String {
        let base_selector = if path.is_empty() {
            "body".to_string()
        } else {
            path.iter()
                .map(|&index| format!(":nth-child({})", index + 1))
                .collect::<Vec<_>>()
                .join(" > ")
        };
        
        // Scope to component if provided
        if let Some(comp_id) = component_id {
            if let Some(boundary) = self.component_boundaries.get(comp_id) {
                return format!("{} {}", boundary.root_selector, base_selector);
            }
        }
        
        base_selector
    }
    
    /// Check if an element should be preserved during updates
    fn should_preserve_element(&self, selector: &str) -> bool {
        self.options.preserve_selectors
            .iter()
            .any(|preserve_sel| selector.contains(preserve_sel))
    }
    
    /// Optimize a patch for hot reload efficiency
    fn optimize_patch(
        &self,
        patch: HotReloadPatch,
        component_id: Option<&str>,
    ) -> Result<OptimizedPatch, HotReloadError> {
        let mut optimized_ops = Vec::new();
        let mut affected_components = HashSet::new();
        let mut requires_full_refresh = false;
        
        // Group operations by target elements for batching
        let mut element_ops: HashMap<String, Vec<HotReloadPatchOp>> = HashMap::new();
        
        for op in patch.operations {
            let target = self.get_operation_target(&op);
            element_ops.entry(target).or_insert_with(Vec::new).push(op);
        }
        
        // Optimize operations for each element
        for (target, ops) in element_ops {
            let optimized = self.optimize_element_operations(target, ops)?;
            optimized_ops.extend(optimized);
        }
        
        // Check if we need a full refresh
        if optimized_ops.len() > 100 {
            warn!("Large patch detected ({} operations), considering full refresh", optimized_ops.len());
            requires_full_refresh = true;
        }
        
        // Determine affected components
        for op in &optimized_ops {
            if let Some(comp_id) = self.extract_component_id_from_operation(op) {
                affected_components.insert(comp_id);
            }
        }
        
        Ok(OptimizedPatch {
            patch: HotReloadPatch { operations: optimized_ops },
            component_boundaries: affected_components.into_iter().collect(),
            state_preservation: self.generate_state_preservation_data(),
            requires_full_refresh,
        })
    }
    
    /// Get the target selector for an operation
    fn get_operation_target(&self, op: &HotReloadPatchOp) -> String {
        match op {
            HotReloadPatchOp::Replace { selector, .. } => selector.clone(),
            HotReloadPatchOp::UpdateText { selector, .. } => selector.clone(),
            HotReloadPatchOp::SetAttribute { selector, .. } => selector.clone(),
            HotReloadPatchOp::RemoveAttribute { selector, .. } => selector.clone(),
            HotReloadPatchOp::InsertChild { parent_selector, .. } => parent_selector.clone(),
            HotReloadPatchOp::RemoveChild { parent_selector, .. } => parent_selector.clone(),
            HotReloadPatchOp::MoveChild { parent_selector, .. } => parent_selector.clone(),
        }
    }
    
    /// Optimize operations for a single element
    fn optimize_element_operations(
        &self,
        _target: String,
        operations: Vec<HotReloadPatchOp>,
    ) -> Result<Vec<HotReloadPatchOp>, HotReloadError> {
        // For now, just return the operations as-is
        // Future optimizations could include:
        // - Merging multiple attribute changes
        // - Batching child insertions/removals
        // - Detecting patterns that suggest full replacement
        Ok(operations)
    }
    
    /// Extract component ID from an operation if possible
    fn extract_component_id_from_operation(&self, op: &HotReloadPatchOp) -> Option<String> {
        let selector = self.get_operation_target(op);
        
        // Look for component boundaries that match this selector
        for (comp_id, boundary) in &self.component_boundaries {
            if selector.starts_with(&boundary.root_selector) {
                return Some(comp_id.clone());
            }
        }
        
        None
    }
    
    /// Generate state preservation data
    fn generate_state_preservation_data(&self) -> PreservedDomState {
        // This would typically extract current DOM state from the client
        // For now, return empty state
        PreservedDomState {
            focused_element: None,
            scroll_positions: HashMap::new(),
            form_values: HashMap::new(),
            text_selections: HashMap::new(),
        }
    }
    
    /// Get components affected by a patch
    fn get_affected_components(&self, patch: &HotReloadPatch) -> Vec<String> {
        let mut affected = HashSet::new();
        
        for op in &patch.operations {
            if let Some(comp_id) = self.extract_component_id_from_operation(op) {
                affected.insert(comp_id);
            }
        }
        
        affected.into_iter().collect()
    }
    
    /// Compute cache key for a diff
    fn compute_cache_key(&self, from_html: &str, to_html: &str, component_id: Option<&str>) -> String {
        use blake3::Hasher;
        
        let mut hasher = Hasher::new();
        hasher.update(from_html.as_bytes());
        hasher.update(to_html.as_bytes());
        if let Some(comp_id) = component_id {
            hasher.update(comp_id.as_bytes());
        }
        
        hasher.finalize().to_hex().to_string()
    }
    
    /// Cache a diff result
    fn cache_diff(&mut self, key: String, from_html: &str, to_html: &str, patch: &HotReloadPatch) {
        use blake3::Hasher;
        
        let mut hasher = Hasher::new();
        let source_hash = {
            hasher.update(from_html.as_bytes());
            hasher.finalize().to_hex().to_string()
        };
        
        hasher = Hasher::new();
        let target_hash = {
            hasher.update(to_html.as_bytes());
            hasher.finalize().to_hex().to_string()
        };
        
        let cached = CachedDiff {
            source_hash,
            target_hash,
            patch: patch.clone(),
            cached_at: std::time::SystemTime::now(),
        };
        
        self.diff_cache.insert(key, cached);
        
        // Clean old cache entries (keep last 100)
        if self.diff_cache.len() > 100 {
            let mut entries: Vec<_> = self.diff_cache.iter().map(|(k, v)| (k.clone(), v.cached_at)).collect();
            entries.sort_by_key(|(_, cached_at)| *cached_at);
            
            // Remove oldest 20 entries
            for (key, _) in entries.iter().take(20) {
                self.diff_cache.remove(key);
            }
        }
    }
    
    /// Clear all cached diffs
    pub fn clear_cache(&mut self) {
        debug!("Clearing DOM diff cache");
        self.diff_cache.clear();
    }
}

/// Optimized patch result with metadata
#[derive(Debug, Clone)]
pub struct OptimizedPatch {
    /// The optimized patch operations
    pub patch: HotReloadPatch,
    /// Components affected by this patch
    pub component_boundaries: Vec<String>,
    /// State that should be preserved
    pub state_preservation: PreservedDomState,
    /// Whether a full page refresh is recommended
    pub requires_full_refresh: bool,
}

impl Default for HotReloadDomDiffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilities for DOM state preservation
pub mod state_preservation {
    use super::*;
    
    /// Generate JavaScript code to capture current DOM state
    pub fn generate_state_capture_script() -> String {
        r#"
        (function() {
            const state = {
                focused_element: null,
                scroll_positions: {},
                form_values: {},
                text_selections: {}
            };
            
            // Capture focused element
            if (document.activeElement && document.activeElement !== document.body) {
                state.focused_element = getElementSelector(document.activeElement);
            }
            
            // Capture scroll positions
            document.querySelectorAll('*').forEach(el => {
                if (el.scrollTop > 0 || el.scrollLeft > 0) {
                    const selector = getElementSelector(el);
                    state.scroll_positions[selector] = {
                        x: el.scrollLeft,
                        y: el.scrollTop
                    };
                }
            });
            
            // Capture form values
            document.querySelectorAll('input, textarea, select').forEach(el => {
                const selector = getElementSelector(el);
                if (el.type === 'checkbox' || el.type === 'radio') {
                    state.form_values[selector] = el.checked;
                } else if (el.type === 'file') {
                    // Don't capture file inputs
                } else if (el.tagName === 'SELECT' && el.multiple) {
                    state.form_values[selector] = Array.from(el.selectedOptions).map(o => o.value);
                } else {
                    state.form_values[selector] = el.value;
                }
                
                // Capture text selections
                if ((el.type === 'text' || el.type === 'textarea') && 
                    el.selectionStart !== el.selectionEnd) {
                    state.text_selections[selector] = {
                        start: el.selectionStart,
                        end: el.selectionEnd,
                        direction: el.selectionDirection || 'forward'
                    };
                }
            });
            
            function getElementSelector(el) {
                if (el.id) return '#' + el.id;
                if (el.className) return '.' + el.className.split(' ').join('.');
                
                let path = [];
                while (el.parentNode) {
                    let siblings = Array.from(el.parentNode.children);
                    let index = siblings.indexOf(el) + 1;
                    path.unshift(`${el.tagName.toLowerCase()}:nth-child(${index})`);
                    el = el.parentNode;
                }
                return path.join(' > ');
            }
            
            return state;
        })();
        "#.to_string()
    }
    
    /// Generate JavaScript code to restore DOM state
    pub fn generate_state_restore_script(state: &PreservedDomState) -> String {
        let state_json = serde_json::to_string(state).unwrap_or_default();
        
        format!(r#"
        (function() {{
            const state = {};
            
            // Restore focus
            if (state.focused_element) {{
                const el = document.querySelector(state.focused_element);
                if (el && el.focus) {{
                    el.focus();
                }}
            }}
            
            // Restore scroll positions
            Object.entries(state.scroll_positions).forEach(([selector, pos]) => {{
                const el = document.querySelector(selector);
                if (el) {{
                    el.scrollLeft = pos.x;
                    el.scrollTop = pos.y;
                }}
            }});
            
            // Restore form values
            Object.entries(state.form_values).forEach(([selector, value]) => {{
                const el = document.querySelector(selector);
                if (el) {{
                    if (el.type === 'checkbox' || el.type === 'radio') {{
                        el.checked = value;
                    }} else if (el.tagName === 'SELECT' && el.multiple) {{
                        Array.from(el.options).forEach(option => {{
                            option.selected = value.includes(option.value);
                        }});
                    }} else {{
                        el.value = value;
                    }}
                }}
            }});
            
            // Restore text selections
            Object.entries(state.text_selections).forEach(([selector, selection]) => {{
                const el = document.querySelector(selector);
                if (el && el.setSelectionRange) {{
                    el.setSelectionRange(selection.start, selection.end, selection.direction);
                }}
            }});
        }})();
        "#, state_json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dom_differ_creation() {
        let differ = HotReloadDomDiffer::new();
        assert_eq!(differ.diff_cache.len(), 0);
        assert_eq!(differ.component_boundaries.len(), 0);
    }
    
    #[test]
    fn test_component_boundary_registration() {
        let mut differ = HotReloadDomDiffer::new();
        
        let boundary = ComponentBoundary {
            component_id: "test-component".to_string(),
            root_selector: "#test-component".to_string(),
            isolated: true,
            children: vec![],
            state_data: None,
        };
        
        differ.register_component_boundary(boundary);
        assert_eq!(differ.component_boundaries.len(), 1);
        assert!(differ.component_boundaries.contains_key("test-component"));
    }
    
    #[test]
    fn test_path_to_selector() {
        let differ = HotReloadDomDiffer::new();
        
        assert_eq!(differ.path_to_selector(&[], None), "body");
        assert_eq!(differ.path_to_selector(&[0], None), ":nth-child(1)");
        assert_eq!(differ.path_to_selector(&[0, 1, 2], None), ":nth-child(1) > :nth-child(2) > :nth-child(3)");
    }
    
    #[test]
    fn test_cache_key_generation() {
        let differ = HotReloadDomDiffer::new();
        
        let key1 = differ.compute_cache_key("<div>a</div>", "<div>b</div>", None);
        let key2 = differ.compute_cache_key("<div>a</div>", "<div>b</div>", None);
        let key3 = differ.compute_cache_key("<div>a</div>", "<div>c</div>", None);
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}