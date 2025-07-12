//! Smart change analysis for detecting template vs code changes
//!
//! This module provides intelligent analysis of file changes to determine whether
//! a change is template-only (enabling fast hot reload) or requires a full rebuild.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use syn::{parse_file, spanned::Spanned, visit::Visit, ExprMacro, Stmt, Item, UseTree, Macro};
use tracing::{debug, trace, warn};

use crate::protocol::{
    ChangeAnalysis, ChangeComplexity, ChangeMetadata, ChangeType, ContentType, LineRange,
};

/// Smart analyzer for detecting types of changes in Rust files
pub struct ChangeAnalyzer {
    /// Previous file content for comparison
    previous_content: Option<String>,
    /// Previous syntax tree
    previous_ast: Option<syn::File>,
    /// Cache of macro locations from previous parse
    previous_macros: Vec<MacroLocation>,
}

/// Location and metadata of a macro in the source code
#[derive(Debug, Clone)]
struct MacroLocation {
    /// Macro name (html, view, etc.)
    name: String,
    /// Start line (1-based)
    start_line: u32,
    /// End line (1-based)
    end_line: u32,
    /// Start column (0-based)
    start_column: u32,
    /// End column (0-based)
    end_column: u32,
    /// Raw token content
    content: String,
}

/// Visitor for collecting macro locations and other syntax elements
struct MacroCollector {
    macros: Vec<MacroLocation>,
    imports: Vec<LineRange>,
    functions: Vec<LineRange>,
    structs: Vec<LineRange>,
    current_source: String,
}

impl ChangeAnalyzer {
    /// Create a new change analyzer
    pub fn new() -> Self {
        Self {
            previous_content: None,
            previous_ast: None,
            previous_macros: Vec::new(),
        }
    }

    /// Analyze changes in a file
    pub fn analyze_change(
        &mut self,
        file_path: &Path,
        new_content: &str,
    ) -> Result<ChangeAnalysis> {
        let start_time = std::time::Instant::now();
        debug!("Starting change analysis for {}", file_path.display());

        // Check if this is a supported file type
        if !self.is_supported_file(file_path) {
            return Ok(ChangeAnalysis {
                change_type: ChangeType::Unknown,
                template_count: 0,
                templates_added_or_removed: false,
                affected_ranges: vec![],
                confidence: 1.0,
                metadata: None,
            });
        }

        // Check for asset files
        if self.is_asset_file(file_path) {
            return Ok(ChangeAnalysis {
                change_type: ChangeType::Assets,
                template_count: 0,
                templates_added_or_removed: false,
                affected_ranges: vec![],
                confidence: 1.0,
                metadata: Some(ChangeMetadata {
                    macro_types: vec![],
                    is_new_file: self.previous_content.is_none(),
                    complexity: ChangeComplexity::Simple,
                }),
            });
        }

        // Parse the new content
        let new_ast = parse_file(new_content).context("Failed to parse new file content")?;
        let new_macros = self.extract_macros(&new_content, &new_ast)?;

        // Determine the type of change
        let analysis = if let (Some(prev_content), Some(prev_ast)) = 
            (&self.previous_content, &self.previous_ast) {
            
            self.analyze_incremental_change(
                prev_content,
                prev_ast,
                &self.previous_macros,
                new_content,
                &new_ast,
                &new_macros,
            )?
        } else {
            // First time seeing this file
            self.analyze_new_file(new_content, &new_ast, &new_macros)?
        };

        // Cache current state for next comparison
        self.previous_content = Some(new_content.to_string());
        self.previous_ast = Some(new_ast);
        self.previous_macros = new_macros;

        let analysis_time = start_time.elapsed();
        
        // Comprehensive logging with performance metrics
        match analysis.change_type {
            ChangeType::TemplateOnly => {
                info!(
                    "⚡ Template-only change in {} ({} templates, {:.1}% confidence) - analyzed in {:.2}ms",
                    file_path.display(),
                    analysis.template_count,
                    analysis.confidence * 100.0,
                    analysis_time.as_secs_f64() * 1000.0
                );
                
                // Log detailed template info
                if let Some(ref metadata) = analysis.metadata {
                    debug!(
                        "Template macros found: [{}], complexity: {:?}",
                        metadata.macro_types.join(", "),
                        metadata.complexity
                    );
                }
                
                // Log affected ranges for debugging
                for (i, range) in analysis.affected_ranges.iter().enumerate() {
                    trace!(
                        "Template range {}: lines {}-{} ({:?})",
                        i + 1,
                        range.start,
                        range.end,
                        range.content_type
                    );
                }
            }
            ChangeType::RustCode => {
                warn!(
                    "🔄 Rust code change in {} ({} ranges affected, {:.1}% confidence) - analyzed in {:.2}ms",
                    file_path.display(),
                    analysis.affected_ranges.len(),
                    analysis.confidence * 100.0,
                    analysis_time.as_secs_f64() * 1000.0
                );
                
                // Log what type of code changed
                for range in &analysis.affected_ranges {
                    debug!(
                        "Code change: lines {}-{} ({:?})",
                        range.start,
                        range.end,
                        range.content_type
                    );
                }
            }
            ChangeType::Mixed => {
                warn!(
                    "🔀 Mixed change in {} ({} templates + {} code ranges, {:.1}% confidence) - analyzed in {:.2}ms",
                    file_path.display(),
                    analysis.template_count,
                    analysis.affected_ranges.iter().filter(|r| r.content_type == ContentType::RustCode).count(),
                    analysis.confidence * 100.0,
                    analysis_time.as_secs_f64() * 1000.0
                );
            }
            ChangeType::Assets => {
                info!(
                    "🎨 Asset change in {} - analyzed in {:.2}ms",
                    file_path.display(),
                    analysis_time.as_secs_f64() * 1000.0
                );
            }
            ChangeType::Unknown => {
                debug!(
                    "❓ Unknown change type in {} - analyzed in {:.2}ms",
                    file_path.display(),
                    analysis_time.as_secs_f64() * 1000.0
                );
            }
        }
        
        // Log templates added/removed information
        if analysis.templates_added_or_removed {
            info!("📝 Templates were added or removed in {}", file_path.display());
        }
        
        // Performance warning for slow analysis
        if analysis_time.as_millis() > 50 {
            warn!(
                "⚠️  Slow change analysis for {} took {:.2}ms (file size: {} bytes)",
                file_path.display(),
                analysis_time.as_secs_f64() * 1000.0,
                new_content.len()
            );
        }

        Ok(analysis)
    }

    /// Extract macro locations from the AST
    fn extract_macros(&self, content: &str, ast: &syn::File) -> Result<Vec<MacroLocation>> {
        let mut collector = MacroCollector {
            macros: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            structs: Vec::new(),
            current_source: content.to_string(),
        };

        collector.visit_file(ast);
        Ok(collector.macros)
    }

    /// Analyze changes incrementally by comparing with previous state
    fn analyze_incremental_change(
        &self,
        prev_content: &str,
        _prev_ast: &syn::File,
        prev_macros: &[MacroLocation],
        new_content: &str,
        _new_ast: &syn::File,
        new_macros: &[MacroLocation],
    ) -> Result<ChangeAnalysis> {
        trace!(
            "Performing incremental change analysis: {} -> {} macros",
            prev_macros.len(),
            new_macros.len()
        );

        // Quick check: if macro count changed, templates were added/removed
        let templates_added_or_removed = prev_macros.len() != new_macros.len();
        
        // Find which lines changed
        let changed_lines = self.find_changed_lines(prev_content, new_content);
        debug!("Found {} changed lines", changed_lines.len());
        
        // Log a sample of changed lines for debugging
        if !changed_lines.is_empty() && log::log_enabled!(log::Level::Trace) {
            let sample_size = changed_lines.len().min(5);
            let sample: Vec<String> = changed_lines
                .iter()
                .take(sample_size)
                .map(|&line| format!("L{}", line))
                .collect();
            trace!("Changed lines (sample): [{}]", sample.join(", "));
            if changed_lines.len() > sample_size {
                trace!("... and {} more lines", changed_lines.len() - sample_size);
            }
        }
        
        // Determine which macros are affected
        let affected_macros = self.find_affected_macros(new_macros, &changed_lines);
        debug!("Found {} affected macros", affected_macros.len());
        
        // Check for non-template changes
        let non_template_changes = self.find_non_template_changes(
            &changed_lines,
            new_macros,
            new_content,
        );
        debug!("Found {} non-template change ranges", non_template_changes.len());

        // Determine change type
        let change_type = if non_template_changes.is_empty() && !affected_macros.is_empty() {
            ChangeType::TemplateOnly
        } else if affected_macros.is_empty() && !non_template_changes.is_empty() {
            ChangeType::RustCode
        } else if !affected_macros.is_empty() && !non_template_changes.is_empty() {
            ChangeType::Mixed
        } else {
            // No significant changes detected
            ChangeType::Unknown
        };

        // Build affected ranges
        let mut affected_ranges = Vec::new();
        
        // Add template ranges
        for macro_loc in &affected_macros {
            affected_ranges.push(LineRange {
                start: macro_loc.start_line,
                end: macro_loc.end_line,
                content_type: ContentType::Template,
            });
        }
        
        // Add non-template ranges
        for range in non_template_changes {
            affected_ranges.push(range);
        }

        // Calculate confidence based on analysis quality
        let confidence = self.calculate_confidence(&change_type, &affected_macros, &changed_lines);

        // Extract macro types
        let macro_types: Vec<String> = new_macros
            .iter()
            .map(|m| m.name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Determine complexity
        let complexity = if changed_lines.len() <= 5 && affected_macros.len() <= 1 {
            ChangeComplexity::Simple
        } else if changed_lines.len() <= 20 && affected_macros.len() <= 3 {
            ChangeComplexity::Moderate
        } else {
            ChangeComplexity::Complex
        };

        Ok(ChangeAnalysis {
            change_type,
            template_count: affected_macros.len(),
            templates_added_or_removed,
            affected_ranges,
            confidence,
            metadata: Some(ChangeMetadata {
                macro_types,
                is_new_file: false,
                complexity,
            }),
        })
    }

    /// Analyze a new file (first time seeing it)
    fn analyze_new_file(
        &self,
        content: &str,
        _ast: &syn::File,
        macros: &[MacroLocation],
    ) -> Result<ChangeAnalysis> {
        trace!("Analyzing new file with {} macros", macros.len());

        let change_type = if macros.is_empty() {
            ChangeType::RustCode
        } else {
            ChangeType::TemplateOnly
        };

        let macro_types: Vec<String> = macros
            .iter()
            .map(|m| m.name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let complexity = if content.lines().count() <= 50 {
            ChangeComplexity::Simple
        } else if content.lines().count() <= 200 {
            ChangeComplexity::Moderate
        } else {
            ChangeComplexity::Complex
        };

        Ok(ChangeAnalysis {
            change_type,
            template_count: macros.len(),
            templates_added_or_removed: !macros.is_empty(),
            affected_ranges: vec![],
            confidence: 0.8, // Lower confidence for new files
            metadata: Some(ChangeMetadata {
                macro_types,
                is_new_file: true,
                complexity,
            }),
        })
    }

    /// Find lines that changed between two versions (optimized for large files)
    fn find_changed_lines(&self, old_content: &str, new_content: &str) -> Vec<u32> {
        // Quick hash comparison for identical content
        if old_content == new_content {
            return Vec::new();
        }
        
        // For very large files, use a more efficient diff approach
        const LARGE_FILE_THRESHOLD: usize = 50_000; // 50KB
        if old_content.len() > LARGE_FILE_THRESHOLD || new_content.len() > LARGE_FILE_THRESHOLD {
            return self.find_changed_lines_efficient(old_content, new_content);
        }
        
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();
        
        let mut changed_lines = Vec::new();
        let max_lines = old_lines.len().max(new_lines.len());
        
        // Early termination for files that only differ at the end
        let min_lines = old_lines.len().min(new_lines.len());
        
        for i in 0..min_lines {
            if old_lines[i] != new_lines[i] {
                changed_lines.push((i + 1) as u32); // 1-based line numbers
            }
        }
        
        // Add any remaining lines as changed
        for i in min_lines..max_lines {
            changed_lines.push((i + 1) as u32);
        }
        
        trace!("Found {} changed lines", changed_lines.len());
        changed_lines
    }

    /// More efficient line change detection for large files
    fn find_changed_lines_efficient(&self, old_content: &str, new_content: &str) -> Vec<u32> {
        use std::collections::HashMap;
        
        // Hash-based approach for large files
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();
        
        // Create hash map of old lines for O(1) lookup
        let old_line_hashes: HashMap<u32, u64> = old_lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                use std::hash::{Hash, Hasher};
                line.hash(&mut hasher);
                (i as u32, hasher.finish())
            })
            .collect();
        
        let mut changed_lines = Vec::new();
        
        // Check each new line against corresponding old line
        for (i, &new_line) in new_lines.iter().enumerate() {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            use std::hash::{Hash, Hasher};
            new_line.hash(&mut hasher);
            let new_hash = hasher.finish();
            
            let line_changed = old_line_hashes
                .get(&(i as u32))
                .map(|&old_hash| old_hash != new_hash)
                .unwrap_or(true); // Line doesn't exist in old file
            
            if line_changed {
                changed_lines.push((i + 1) as u32);
            }
        }
        
        // Mark any lines that were removed (exist in old but not new)
        if old_lines.len() > new_lines.len() {
            for i in new_lines.len()..old_lines.len() {
                changed_lines.push((i + 1) as u32);
            }
        }
        
        trace!("Found {} changed lines (efficient method)", changed_lines.len());
        changed_lines
    }

    /// Find macros that are affected by the changed lines
    fn find_affected_macros(
        &self,
        macros: &[MacroLocation],
        changed_lines: &[u32],
    ) -> Vec<MacroLocation> {
        let mut affected = Vec::new();
        
        for macro_loc in macros {
            if changed_lines.iter().any(|&line| {
                line >= macro_loc.start_line && line <= macro_loc.end_line
            }) {
                affected.push(macro_loc.clone());
            }
        }
        
        trace!("Found {} affected macros", affected.len());
        affected
    }

    /// Find changes that are outside of template macros
    fn find_non_template_changes(
        &self,
        changed_lines: &[u32],
        macros: &[MacroLocation],
        content: &str,
    ) -> Vec<LineRange> {
        let mut non_template_ranges = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for &line_num in changed_lines {
            let line_idx = (line_num - 1) as usize;
            
            // Check if this line is within any macro
            let in_macro = macros.iter().any(|m| {
                line_num >= m.start_line && line_num <= m.end_line
            });
            
            if !in_macro {
                let line_content = lines.get(line_idx).unwrap_or(&"").trim();
                
                // Classify the type of change
                let content_type = if line_content.starts_with("use ") || line_content.starts_with("mod ") {
                    ContentType::Imports
                } else if line_content.starts_with("//") || line_content.starts_with("/*") {
                    ContentType::Comments
                } else if !line_content.is_empty() {
                    ContentType::RustCode
                } else {
                    continue; // Skip empty lines
                };
                
                non_template_ranges.push(LineRange {
                    start: line_num,
                    end: line_num,
                    content_type,
                });
            }
        }
        
        // Merge adjacent ranges of the same type
        self.merge_adjacent_ranges(non_template_ranges)
    }

    /// Merge adjacent line ranges of the same content type
    fn merge_adjacent_ranges(&self, mut ranges: Vec<LineRange>) -> Vec<LineRange> {
        if ranges.is_empty() {
            return ranges;
        }
        
        ranges.sort_by_key(|r| r.start);
        let mut merged = Vec::new();
        let mut current = ranges[0].clone();
        
        for range in ranges.into_iter().skip(1) {
            if range.start <= current.end + 1 && range.content_type == current.content_type {
                // Merge with current range
                current.end = range.end.max(current.end);
            } else {
                // Start a new range
                merged.push(current);
                current = range;
            }
        }
        
        merged.push(current);
        merged
    }

    /// Calculate confidence level for the analysis
    fn calculate_confidence(
        &self,
        change_type: &ChangeType,
        affected_macros: &[MacroLocation],
        changed_lines: &[u32],
    ) -> f32 {
        let mut confidence = 1.0;
        
        // Reduce confidence for complex scenarios
        if matches!(change_type, ChangeType::Mixed) {
            confidence *= 0.8;
        }
        
        // Reduce confidence if many lines changed
        if changed_lines.len() > 20 {
            confidence *= 0.7;
        }
        
        // Reduce confidence if many macros affected
        if affected_macros.len() > 5 {
            confidence *= 0.8;
        }
        
        confidence.max(0.1) // Minimum confidence
    }

    /// Check if a file is supported for analysis
    fn is_supported_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(ext.to_string_lossy().as_ref(), "rs")
        } else {
            false
        }
    }

    /// Check if a file is an asset file
    fn is_asset_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_string_lossy().as_ref(),
                "css" | "js" | "ts" | "scss" | "sass" | "less" | 
                "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" |
                "woff" | "woff2" | "ttf" | "eot"
            )
        } else {
            false
        }
    }
}

impl Default for ChangeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroCollector {
    /// Extract line and column from a span
    fn span_to_location(&self, span: proc_macro2::Span) -> (u32, u32, u32, u32) {
        let start = span.start();
        let end = span.end();
        (
            start.line as u32,
            start.column as u32,
            end.line as u32,
            end.column as u32,
        )
    }

    /// Check if a macro is a template macro
    fn is_template_macro(&self, mac: &Macro) -> bool {
        let path = &mac.path;
        if path.segments.len() == 1 {
            let ident = &path.segments[0].ident;
            ident == "html" || ident == "view"
        } else if path.segments.len() == 2 {
            let first = &path.segments[0].ident;
            let second = &path.segments[1].ident;
            (first == "axum_live_view" && second == "html")
                || (first == "shipwright_liveview" && second == "view")
        } else {
            false
        }
    }

    /// Extract macro content from the source
    fn extract_macro_content(
        &self,
        start_line: u32,
        start_col: u32,
        end_line: u32,
        end_col: u32,
    ) -> String {
        let lines: Vec<&str> = self.current_source.lines().collect();
        
        if start_line == end_line {
            // Single line macro
            if let Some(line) = lines.get((start_line - 1) as usize) {
                let start_idx = start_col as usize;
                let end_idx = (end_col as usize).min(line.len());
                if start_idx < line.len() {
                    return line[start_idx..end_idx].to_string();
                }
            }
        } else {
            // Multi-line macro
            let mut content = String::new();
            for line_idx in (start_line - 1)..end_line {
                if let Some(line) = lines.get(line_idx as usize) {
                    if line_idx == start_line - 1 {
                        // First line - start from start_col
                        content.push_str(&line[start_col as usize..]);
                    } else if line_idx == end_line - 1 {
                        // Last line - end at end_col
                        let end_idx = (end_col as usize).min(line.len());
                        content.push_str(&line[..end_idx]);
                    } else {
                        // Middle line - include entire line
                        content.push_str(line);
                    }
                    content.push('\n');
                }
            }
            return content;
        }
        
        String::new()
    }
}

impl<'ast> Visit<'ast> for MacroCollector {
    fn visit_expr_macro(&mut self, node: &'ast ExprMacro) {
        if self.is_template_macro(&node.mac) {
            let (start_line, start_col, end_line, end_col) = self.span_to_location(node.span());
            let macro_name = node.mac.path.segments.last()
                .map(|s| s.ident.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            
            let content = self.extract_macro_content(start_line, start_col, end_line, end_col);
            
            self.macros.push(MacroLocation {
                name: macro_name,
                start_line,
                end_line,
                start_column: start_col,
                end_column: end_col,
                content,
            });
        }
        
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_stmt(&mut self, node: &'ast Stmt) {
        if let Stmt::Macro(stmt_macro) = node {
            if self.is_template_macro(&stmt_macro.mac) {
                let (start_line, start_col, end_line, end_col) = self.span_to_location(stmt_macro.span());
                let macro_name = stmt_macro.mac.path.segments.last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                
                let content = self.extract_macro_content(start_line, start_col, end_line, end_col);
                
                self.macros.push(MacroLocation {
                    name: macro_name,
                    start_line,
                    end_line,
                    start_column: start_col,
                    end_column: end_col,
                    content,
                });
            }
        }
        
        syn::visit::visit_stmt(self, node);
    }

    fn visit_item(&mut self, node: &'ast Item) {
        let (start_line, end_line, content_type) = match node {
            Item::Fn(item_fn) => {
                let (start_line, _, end_line, _) = self.span_to_location(item_fn.span());
                (start_line, end_line, ContentType::RustCode)
            }
            Item::Struct(item_struct) => {
                let (start_line, _, end_line, _) = self.span_to_location(item_struct.span());
                (start_line, end_line, ContentType::RustCode)
            }
            Item::Use(item_use) => {
                let (start_line, _, end_line, _) = self.span_to_location(item_use.span());
                (start_line, end_line, ContentType::Imports)
            }
            _ => {
                let (start_line, _, end_line, _) = self.span_to_location(node.span());
                (start_line, end_line, ContentType::RustCode)
            }
        };

        let range = LineRange {
            start: start_line,
            end: end_line,
            content_type,
        };

        match range.content_type {
            ContentType::RustCode => {
                if matches!(node, Item::Fn(_)) {
                    self.functions.push(range);
                } else if matches!(node, Item::Struct(_)) {
                    self.structs.push(range);
                }
            }
            ContentType::Imports => {
                self.imports.push(range);
            }
            _ => {}
        }

        syn::visit::visit_item(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_template_only_change() {
        let mut analyzer = ChangeAnalyzer::new();
        let path = PathBuf::from("test.rs");

        let initial_content = r#"
fn render() {
    html! {
        <div>Hello World</div>
    }
}
"#;

        let modified_content = r#"
fn render() {
    html! {
        <div>Hello Rust</div>
    }
}
"#;

        // First analysis (new file)
        let analysis = analyzer.analyze_change(&path, initial_content).unwrap();
        assert_eq!(analysis.change_type, ChangeType::TemplateOnly);
        assert!(analysis.metadata.as_ref().unwrap().is_new_file);

        // Second analysis (template change)
        let analysis = analyzer.analyze_change(&path, modified_content).unwrap();
        assert_eq!(analysis.change_type, ChangeType::TemplateOnly);
        assert_eq!(analysis.template_count, 1);
        assert!(!analysis.templates_added_or_removed);
    }

    #[test]
    fn test_rust_code_change() {
        let mut analyzer = ChangeAnalyzer::new();
        let path = PathBuf::from("test.rs");

        let initial_content = r#"
fn calculate(x: i32) -> i32 {
    x * 2
}

fn render() {
    html! {
        <div>Hello World</div>
    }
}
"#;

        let modified_content = r#"
fn calculate(x: i32) -> i32 {
    x * 3  // Changed calculation
}

fn render() {
    html! {
        <div>Hello World</div>
    }
}
"#;

        // First analysis
        analyzer.analyze_change(&path, initial_content).unwrap();

        // Second analysis (code change)
        let analysis = analyzer.analyze_change(&path, modified_content).unwrap();
        assert_eq!(analysis.change_type, ChangeType::RustCode);
        assert_eq!(analysis.template_count, 0);
    }

    #[test]
    fn test_mixed_change() {
        let mut analyzer = ChangeAnalyzer::new();
        let path = PathBuf::from("test.rs");

        let initial_content = r#"
fn calculate(x: i32) -> i32 {
    x * 2
}

fn render() {
    html! {
        <div>Hello World</div>
    }
}
"#;

        let modified_content = r#"
fn calculate(x: i32) -> i32 {
    x * 3  // Changed calculation
}

fn render() {
    html! {
        <div>Hello Rust</div>
    }
}
"#;

        // First analysis
        analyzer.analyze_change(&path, initial_content).unwrap();

        // Second analysis (mixed change)
        let analysis = analyzer.analyze_change(&path, modified_content).unwrap();
        assert_eq!(analysis.change_type, ChangeType::Mixed);
        assert_eq!(analysis.template_count, 1);
    }

    #[test]
    fn test_asset_file() {
        let mut analyzer = ChangeAnalyzer::new();
        let path = PathBuf::from("styles.css");

        let content = "body { color: red; }";
        let analysis = analyzer.analyze_change(&path, content).unwrap();
        
        assert_eq!(analysis.change_type, ChangeType::Assets);
        assert!(analysis.metadata.as_ref().unwrap().is_new_file);
    }
}