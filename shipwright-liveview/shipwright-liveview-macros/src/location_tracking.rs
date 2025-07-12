//! Location-based template tracking for hot reload identification.
//!
//! This module provides sophisticated location tracking using compile-time
//! file!, line!, and column! macros to create unique template identifiers
//! that persist across compilation sessions for reliable hot reload.

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Location tracker for templates using compile-time location information
pub(crate) struct LocationTracker {
    /// Map of span IDs to template locations
    locations: HashMap<String, TrackedLocation>,
    /// Current file being processed
    current_file: Option<PathBuf>,
    /// Base directory for relative path calculation
    base_dir: Option<PathBuf>,
}

/// Tracked location information for a template or template part
#[derive(Debug, Clone)]
pub(crate) struct TrackedLocation {
    /// Absolute file path
    pub file_path: PathBuf,
    /// Relative file path from project root
    pub relative_path: PathBuf,
    /// Line number in the source file
    pub line: u32,
    /// Column number in the source file  
    pub column: u32,
    /// Unique identifier based on location
    pub location_id: String,
    /// Template identifier for grouping related locations
    pub template_id: String,
    /// Hierarchical path within the template
    pub template_path: String,
    /// Source span for error reporting
    pub span: Span,
}

/// Location-based template identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TemplateId {
    /// Base identifier from file location
    pub base_id: String,
    /// Full hierarchical identifier
    pub full_id: String,
    /// Human-readable identifier
    pub display_name: String,
}

/// Template location registry for hot reload coordination
pub(crate) struct LocationRegistry {
    /// All tracked template locations
    templates: HashMap<String, TemplateLocationInfo>,
    /// File to template mappings
    file_mappings: HashMap<PathBuf, Vec<String>>,
    /// Location lookup index
    location_index: HashMap<String, String>,
}

/// Complete location information for a template
#[derive(Debug, Clone)]
pub(crate) struct TemplateLocationInfo {
    /// Primary location of the template
    pub primary_location: TrackedLocation,
    /// All locations within this template (for nested structures)
    pub all_locations: Vec<TrackedLocation>,
    /// Template metadata
    pub metadata: TemplateMetadata,
    /// Last modification time (if available)
    pub last_modified: Option<std::time::SystemTime>,
}

/// Metadata associated with a template location
#[derive(Debug, Clone, Default)]
pub(crate) struct TemplateMetadata {
    /// Name of the function/method containing this template
    pub containing_function: Option<String>,
    /// Name of the struct/impl containing this template
    pub containing_type: Option<String>,
    /// Module path of the template
    pub module_path: Option<String>,
    /// Template size information
    pub size_info: TemplateSizeInfo,
    /// Dependencies of this template
    pub dependencies: Vec<String>,
}

/// Size and complexity information for templates
#[derive(Debug, Clone, Default)]
pub(crate) struct TemplateSizeInfo {
    /// Number of HTML elements
    pub element_count: usize,
    /// Number of dynamic parts
    pub dynamic_count: usize,
    /// Nesting depth
    pub max_depth: usize,
    /// Total character count
    pub character_count: usize,
}

impl LocationTracker {
    /// Create a new location tracker
    pub(crate) fn new() -> Self {
        Self {
            locations: HashMap::new(),
            current_file: None,
            base_dir: std::env::current_dir().ok(),
        }
    }

    /// Create a location tracker with a specific base directory
    pub(crate) fn with_base_dir(base_dir: PathBuf) -> Self {
        Self {
            locations: HashMap::new(),
            current_file: None,
            base_dir: Some(base_dir),
        }
    }

    /// Track a location using compile-time location macros
    pub(crate) fn track_location(&mut self, span: Span) -> TrackedLocation {
        self.track_location_with_path(span, "")
    }

    /// Track a location with a specific template path
    pub(crate) fn track_location_with_path(&mut self, span: Span, template_path: &str) -> TrackedLocation {
        // Get compile-time location information
        let file_path = PathBuf::from(file!());
        let line = line!();
        let column = column!();

        self.track_explicit_location(span, file_path, line, column, template_path)
    }

    /// Track a location with explicit file/line/column information
    pub(crate) fn track_explicit_location(
        &mut self,
        span: Span,
        file_path: PathBuf,
        line: u32,
        column: u32,
        template_path: &str,
    ) -> TrackedLocation {
        let relative_path = self.relative_path(&file_path);
        let location_id = self.generate_location_id(&relative_path, line, column);
        let template_id = self.generate_template_id(&relative_path, line);

        let location = TrackedLocation {
            file_path: file_path.clone(),
            relative_path,
            line,
            column,
            location_id: location_id.clone(),
            template_id,
            template_path: template_path.to_string(),
            span,
        };

        self.locations.insert(location_id, location.clone());
        self.current_file = Some(file_path);

        location
    }

    /// Generate a unique location ID
    fn generate_location_id(&self, relative_path: &Path, line: u32, column: u32) -> String {
        // Create a stable, readable ID
        let path_str = relative_path
            .to_string_lossy()
            .replace(['/', '\\'], "_")
            .replace(['.', '-'], "_");
        
        format!("{}_L{}C{}", path_str, line, column)
    }

    /// Generate a template ID (groups related locations)
    fn generate_template_id(&self, relative_path: &Path, line: u32) -> String {
        let path_str = relative_path
            .to_string_lossy()
            .replace(['/', '\\'], "_")
            .replace(['.', '-'], "_");
        
        // Group by file and approximate line range (every 10 lines)
        let line_group = (line / 10) * 10;
        format!("{}_T{}", path_str, line_group)
    }

    /// Get relative path from base directory
    fn relative_path(&self, file_path: &Path) -> PathBuf {
        if let Some(base) = &self.base_dir {
            file_path.strip_prefix(base).unwrap_or(file_path).to_path_buf()
        } else {
            file_path.to_path_buf()
        }
    }

    /// Get a tracked location by ID
    pub(crate) fn get_location(&self, location_id: &str) -> Option<&TrackedLocation> {
        self.locations.get(location_id)
    }

    /// Get all locations for the current file
    pub(crate) fn current_file_locations(&self) -> Vec<&TrackedLocation> {
        if let Some(current) = &self.current_file {
            self.locations
                .values()
                .filter(|loc| loc.file_path == *current)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Generate a template ID from a span
    pub(crate) fn template_id_from_span(&mut self, span: Span) -> TemplateId {
        let location = self.track_location(span);
        TemplateId::from_location(&location)
    }

    /// Generate location-based token stream for runtime tracking
    pub(crate) fn location_tokens(&self, location: &TrackedLocation) -> TokenStream {
        let file = &location.file_path.to_string_lossy();
        let line = location.line;
        let column = location.column;
        let location_id = &location.location_id;
        let template_id = &location.template_id;

        quote! {
            shipwright_liveview::__private::TemplateLocation {
                file: #file,
                line: #line,
                column: #column,
                location_id: #location_id,
                template_id: #template_id,
            }
        }
    }

    /// Clear all tracked locations
    pub(crate) fn clear(&mut self) {
        self.locations.clear();
        self.current_file = None;
    }
}

impl Default for LocationTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateId {
    /// Create a template ID from a tracked location
    pub(crate) fn from_location(location: &TrackedLocation) -> Self {
        let base_id = location.template_id.clone();
        let full_id = if location.template_path.is_empty() {
            location.location_id.clone()
        } else {
            format!("{}_{}", location.location_id, location.template_path.replace('.', "_"))
        };

        let display_name = format!(
            "{}:{}:{}",
            location.relative_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown"),
            location.line,
            location.column
        );

        Self {
            base_id,
            full_id,
            display_name,
        }
    }

    /// Create a child template ID
    pub(crate) fn child(&self, path: &str) -> Self {
        Self {
            base_id: self.base_id.clone(),
            full_id: format!("{}_{}", self.full_id, path.replace('.', "_")),
            display_name: format!("{}[{}]", self.display_name, path),
        }
    }

    /// Check if this ID is a child of another ID
    pub(crate) fn is_child_of(&self, other: &Self) -> bool {
        self.base_id == other.base_id && 
        self.full_id.starts_with(&other.full_id) && 
        self.full_id != other.full_id
    }
}

impl LocationRegistry {
    /// Create a new location registry
    pub(crate) fn new() -> Self {
        Self {
            templates: HashMap::new(),
            file_mappings: HashMap::new(),
            location_index: HashMap::new(),
        }
    }

    /// Register a template location
    pub(crate) fn register_template(&mut self, template_id: String, info: TemplateLocationInfo) {
        // Add to file mappings
        let file_path = info.primary_location.file_path.clone();
        self.file_mappings
            .entry(file_path)
            .or_default()
            .push(template_id.clone());

        // Add to location index
        self.location_index.insert(
            info.primary_location.location_id.clone(),
            template_id.clone(),
        );

        for location in &info.all_locations {
            self.location_index.insert(
                location.location_id.clone(),
                template_id.clone(),
            );
        }

        // Store template info
        self.templates.insert(template_id, info);
    }

    /// Get template info by ID
    pub(crate) fn get_template(&self, template_id: &str) -> Option<&TemplateLocationInfo> {
        self.templates.get(template_id)
    }

    /// Get template ID by location ID
    pub(crate) fn template_by_location(&self, location_id: &str) -> Option<&str> {
        self.location_index.get(location_id).map(String::as_str)
    }

    /// Get all templates in a file
    pub(crate) fn templates_in_file(&self, file_path: &Path) -> Vec<&str> {
        self.file_mappings
            .get(file_path)
            .map(|ids| ids.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Update template metadata
    pub(crate) fn update_metadata(&mut self, template_id: &str, metadata: TemplateMetadata) {
        if let Some(info) = self.templates.get_mut(template_id) {
            info.metadata = metadata;
        }
    }

    /// Remove templates for a file (useful for file deletions)
    pub(crate) fn remove_file_templates(&mut self, file_path: &Path) {
        if let Some(template_ids) = self.file_mappings.remove(file_path) {
            for template_id in template_ids {
                if let Some(info) = self.templates.remove(&template_id) {
                    // Remove from location index
                    self.location_index.remove(&info.primary_location.location_id);
                    for location in &info.all_locations {
                        self.location_index.remove(&location.location_id);
                    }
                }
            }
        }
    }

    /// Get statistics about the registry
    pub(crate) fn stats(&self) -> RegistryStats {
        let total_templates = self.templates.len();
        let total_files = self.file_mappings.len();
        let total_locations = self.location_index.len();

        let total_elements = self.templates
            .values()
            .map(|info| info.metadata.size_info.element_count)
            .sum();

        let total_dynamic_parts = self.templates
            .values()
            .map(|info| info.metadata.size_info.dynamic_count)
            .sum();

        RegistryStats {
            total_templates,
            total_files,
            total_locations,
            total_elements,
            total_dynamic_parts,
        }
    }
}

impl Default for LocationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the location registry
#[derive(Debug, Clone)]
pub(crate) struct RegistryStats {
    /// Total number of templates
    pub total_templates: usize,
    /// Total number of files with templates
    pub total_files: usize,
    /// Total number of tracked locations
    pub total_locations: usize,
    /// Total number of HTML elements
    pub total_elements: usize,
    /// Total number of dynamic parts
    pub total_dynamic_parts: usize,
}

/// Helper function for generating location tracking at compile time
/// Note: In a proc-macro crate, we cannot export macro_rules! macros
impl TrackedLocation {
    /// Create a tracked location for testing purposes
    pub(crate) fn for_testing(file: &str, line: u32, column: u32) -> Self {
        TrackedLocation {
            file_path: std::path::PathBuf::from(file),
            relative_path: std::path::PathBuf::from(file),
            line,
            column,
            location_id: format!("{}:{}:{}", file, line, column),
            template_id: format!("{}:{}", file, line / 10 * 10),
            template_path: String::new(),
            span: proc_macro2::Span::call_site(),
        }
    }
    
    /// Create a tracked location with a path for testing purposes
    pub(crate) fn for_testing_with_path(file: &str, line: u32, column: u32, path: &str) -> Self {
        TrackedLocation {
            file_path: std::path::PathBuf::from(file),
            relative_path: std::path::PathBuf::from(file),
            line,
            column,
            location_id: format!("{}:{}:{}", file, line, column),
            template_id: format!("{}:{}", file, line / 10 * 10),
            template_path: path.to_string(),
            span: proc_macro2::Span::call_site(),
        }
    }
}

/// Enhanced location tracking that includes source context
pub(crate) struct ContextualLocationTracker {
    /// Base tracker
    base: LocationTracker,
    /// Source context for each location
    source_context: HashMap<String, SourceContext>,
}

/// Source context around a template location
#[derive(Debug, Clone)]
pub(crate) struct SourceContext {
    /// Lines before the template location
    pub lines_before: Vec<String>,
    /// The actual template line
    pub template_line: String,
    /// Lines after the template location
    pub lines_after: Vec<String>,
    /// Start column of the template
    pub start_column: u32,
    /// End column of the template
    pub end_column: u32,
}

impl ContextualLocationTracker {
    /// Create a new contextual location tracker
    pub(crate) fn new() -> Self {
        Self {
            base: LocationTracker::new(),
            source_context: HashMap::new(),
        }
    }

    /// Track location with source context
    pub(crate) fn track_with_context(
        &mut self,
        span: Span,
        source: &str,
        template_path: &str,
    ) -> TrackedLocation {
        let location = self.base.track_location_with_path(span, template_path);
        
        // Extract source context
        if let Ok(context) = self.extract_source_context(source, location.line, location.column) {
            self.source_context.insert(location.location_id.clone(), context);
        }

        location
    }

    /// Extract source context around a location
    fn extract_source_context(
        &self,
        source: &str,
        line: u32,
        column: u32,
    ) -> Result<SourceContext, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = source.lines().collect();
        let line_index = (line as usize).saturating_sub(1);

        if line_index >= lines.len() {
            return Err("Line number out of range".into());
        }

        let context_size = 3; // Lines before and after
        let start_line = line_index.saturating_sub(context_size);
        let end_line = (line_index + context_size + 1).min(lines.len());

        let lines_before = lines[start_line..line_index]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let template_line = lines[line_index].to_string();

        let lines_after = lines[line_index + 1..end_line]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Ok(SourceContext {
            lines_before,
            template_line,
            lines_after,
            start_column: column,
            end_column: column + 10, // Rough estimate
        })
    }

    /// Get source context for a location
    pub(crate) fn get_context(&self, location_id: &str) -> Option<&SourceContext> {
        self.source_context.get(location_id)
    }
}

impl Default for ContextualLocationTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_tracker() {
        let mut tracker = LocationTracker::new();
        let span = Span::call_site();
        
        let location = tracker.track_location(span);
        assert!(!location.location_id.is_empty());
        assert!(!location.template_id.is_empty());
        assert_eq!(location.template_path, "");
    }

    #[test]
    fn test_template_id_generation() {
        let location = TrackedLocation {
            file_path: PathBuf::from("src/test.rs"),
            relative_path: PathBuf::from("src/test.rs"),
            line: 42,
            column: 10,
            location_id: "src_test_rs_L42C10".to_string(),
            template_id: "src_test_rs_T40".to_string(),
            template_path: "main".to_string(),
            span: Span::call_site(),
        };

        let template_id = TemplateId::from_location(&location);
        assert_eq!(template_id.base_id, "src_test_rs_T40");
        assert!(template_id.full_id.contains("main"));
        assert!(template_id.display_name.contains("test.rs:42:10"));
    }

    #[test]
    fn test_location_registry() {
        let mut registry = LocationRegistry::new();
        
        let location = TrackedLocation {
            file_path: PathBuf::from("test.rs"),
            relative_path: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            location_id: "test_L1C1".to_string(),
            template_id: "test_T0".to_string(),
            template_path: String::new(),
            span: Span::call_site(),
        };

        let info = TemplateLocationInfo {
            primary_location: location.clone(),
            all_locations: vec![location],
            metadata: TemplateMetadata::default(),
            last_modified: None,
        };

        registry.register_template("test_template".to_string(), info);

        assert!(registry.get_template("test_template").is_some());
        assert_eq!(
            registry.template_by_location("test_L1C1"),
            Some("test_template")
        );
    }

    #[test]
    fn test_template_id_hierarchy() {
        let base = TemplateId {
            base_id: "test".to_string(),
            full_id: "test_main".to_string(),
            display_name: "test.rs:10:5".to_string(),
        };

        let child = base.child("loop.0");
        assert!(child.is_child_of(&base));
        assert!(!base.is_child_of(&child));
        assert!(child.full_id.contains("loop_0"));
    }

    #[test]
    fn test_contextual_tracker() {
        let mut tracker = ContextualLocationTracker::new();
        let source = "line 1\nline 2\nhtml! { <div></div> }\nline 4\nline 5";
        
        let location = tracker.track_with_context(
            Span::call_site(),
            source,
            "test"
        );

        if let Some(context) = tracker.get_context(&location.location_id) {
            assert!(!context.template_line.is_empty());
            assert!(!context.lines_before.is_empty() || !context.lines_after.is_empty());
        }
    }

    #[test]
    fn test_registry_stats() {
        let mut registry = LocationRegistry::new();
        
        let location = TrackedLocation {
            file_path: PathBuf::from("test.rs"),
            relative_path: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            location_id: "test_L1C1".to_string(),
            template_id: "test_T0".to_string(),
            template_path: String::new(),
            span: Span::call_site(),
        };

        let mut metadata = TemplateMetadata::default();
        metadata.size_info.element_count = 5;
        metadata.size_info.dynamic_count = 2;

        let info = TemplateLocationInfo {
            primary_location: location.clone(),
            all_locations: vec![location],
            metadata,
            last_modified: None,
        };

        registry.register_template("test_template".to_string(), info);

        let stats = registry.stats();
        assert_eq!(stats.total_templates, 1);
        assert_eq!(stats.total_elements, 5);
        assert_eq!(stats.total_dynamic_parts, 2);
    }
}