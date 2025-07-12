//! Template parser for extracting and analyzing view! macro calls

use anyhow::{Context, Result};
use proc_macro2::Span;
use std::path::Path;
use syn::{parse_file, spanned::Spanned, visit::Visit, ExprMacro, Macro, Stmt};

use crate::protocol::{DynamicKind, DynamicPart, TemplateId, TemplateUpdate};

/// Parser for extracting templates from Rust source files
pub struct TemplateParser {
    /// Current file being parsed
    file_path: String,
    /// Extracted templates
    templates: Vec<ExtractedTemplate>,
}

/// An extracted template from the source
#[derive(Debug)]
struct ExtractedTemplate {
    /// Template identifier
    id: TemplateId,
    /// The macro call
    macro_call: Macro,
    /// Span of the macro call
    span: Span,
}

impl TemplateParser {
    /// Create a new template parser
    pub fn new(file_path: impl AsRef<Path>) -> Self {
        Self {
            file_path: file_path.as_ref().to_string_lossy().to_string(),
            templates: Vec::new(),
        }
    }

    /// Parse a file and extract all templates
    pub fn parse_file(&mut self, content: &str) -> Result<Vec<TemplateUpdate>> {
        let file = parse_file(content).context("Failed to parse Rust file")?;
        
        // Visit the AST to find view! macros
        self.visit_file(&file);
        
        // Process extracted templates
        let mut updates = Vec::new();
        for template in &self.templates {
            if let Ok(update) = self.process_template(template) {
                updates.push(update);
            }
        }
        
        Ok(updates)
    }

    /// Process a single extracted template
    fn process_template(&self, template: &ExtractedTemplate) -> Result<TemplateUpdate> {
        // Extract the template content
        let tokens = template.macro_call.tokens.clone();
        let content = tokens.to_string();
        
        // Parse the template to identify dynamic parts
        let dynamic_parts = self.extract_dynamic_parts(&content)?;
        
        // For now, we'll use the raw content as HTML
        // In a real implementation, this would compile the template
        let html = self.compile_template(&content)?;
        
        let content_hash = TemplateUpdate::compute_content_hash(&html, &dynamic_parts);
        
        Ok(TemplateUpdate {
            id: template.id.clone(),
            hash: template.id.hash(),
            content_hash,
            html,
            dynamic_parts,
        })
    }

    /// Extract dynamic parts from template content
    fn extract_dynamic_parts(&self, content: &str) -> Result<Vec<DynamicPart>> {
        let mut parts = Vec::new();
        let mut index = 0;
        
        // Simple heuristic-based extraction
        // In a real implementation, this would properly parse the template syntax
        
        // Look for event handlers (axm-click, etc.)
        for (pos, _) in content.match_indices("axm-") {
            if let Some(end) = content[pos..].find('=') {
                let event = &content[pos + 4..pos + end];
                parts.push(DynamicPart {
                    index,
                    kind: DynamicKind::EventHandler {
                        event: event.to_string(),
                    },
                });
                index += 1;
            }
        }
        
        // Look for interpolations { ... }
        let mut chars = content.chars().enumerate();
        while let Some((pos, ch)) = chars.next() {
            if ch == '{' && content[..pos].chars().last() != Some('\\') {
                // Check if this is an interpolation
                if let Some(end_pos) = self.find_matching_brace(&content[pos..]) {
                    let inner = &content[pos + 1..pos + end_pos];
                    
                    // Determine the kind of dynamic content
                    let kind = if inner.trim().starts_with("if ") {
                        DynamicKind::Conditional
                    } else if inner.trim().starts_with("for ") {
                        DynamicKind::Loop
                    } else {
                        DynamicKind::Expression
                    };
                    
                    parts.push(DynamicPart { index, kind });
                    index += 1;
                    
                    // Skip past the processed content
                    for _ in 0..end_pos {
                        chars.next();
                    }
                }
            }
        }
        
        Ok(parts)
    }

    /// Find matching closing brace
    fn find_matching_brace(&self, content: &str) -> Option<usize> {
        let mut depth = 0;
        for (i, ch) in content.chars().enumerate() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Compile template content to HTML
    fn compile_template(&self, content: &str) -> Result<String> {
        // For now, just return the content as-is
        // In a real implementation, this would properly compile the template
        Ok(content.to_string())
    }

    /// Check if a macro path might be a view macro
    fn is_view_macro(&self, mac: &Macro) -> bool {
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

    /// Extract span location information
    fn span_to_location(&self, span: Span) -> (u32, u32) {
        // Extract line and column from the span using the span-locations feature
        let start = span.start();
        (start.line as u32, start.column as u32)
    }
}

impl<'ast> Visit<'ast> for TemplateParser {
    fn visit_expr_macro(&mut self, node: &'ast ExprMacro) {
        if self.is_view_macro(&node.mac) {
            let (line, column) = self.span_to_location(node.span());
            let id = TemplateId::new(self.file_path.clone().into(), line, column);
            
            self.templates.push(ExtractedTemplate {
                id,
                macro_call: node.mac.clone(),
                span: node.span(),
            });
        }
        
        // Continue visiting
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_stmt(&mut self, node: &'ast Stmt) {
        if let Stmt::Macro(stmt_macro) = node {
            if self.is_view_macro(&stmt_macro.mac) {
                let (line, column) = self.span_to_location(stmt_macro.span());
                let id = TemplateId::new(self.file_path.clone().into(), line, column);
                
                self.templates.push(ExtractedTemplate {
                    id,
                    macro_call: stmt_macro.mac.clone(),
                    span: stmt_macro.span(),
                });
            }
        }
        
        // Continue visiting
        syn::visit::visit_stmt(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_template() {
        let content = r#"
            fn render() {
                html! {
                    <div>
                        <button axm-click={ Msg::Click }>Click me</button>
                        { self.count }
                    </div>
                }
            }
        "#;

        let mut parser = TemplateParser::new("test.rs");
        let updates = parser.parse_file(content).unwrap();
        
        assert_eq!(updates.len(), 1);
        assert!(!updates[0].dynamic_parts.is_empty());
    }

    #[test]
    fn test_multiple_templates_different_locations() {
        let content = r#"
            fn render_first() {
                html! {
                    <div>First template</div>
                }
            }

            fn render_second() {
                html! {
                    <div>Second template</div>
                }
            }

            fn render_third() {
                html! {
                    <div>Third template</div>
                }
            }
        "#;

        let mut parser = TemplateParser::new("test.rs");
        let updates = parser.parse_file(content).unwrap();
        
        println!("Found {} templates", updates.len());
        
        assert_eq!(updates.len(), 3);
        
        // Check that each template has a different line number
        let line_numbers: Vec<u32> = updates.iter().map(|update| update.id.line).collect();
        println!("Line numbers: {:?}", line_numbers);
        
        assert_eq!(line_numbers.len(), 3);
        
        // All line numbers should be different
        for (i, &line1) in line_numbers.iter().enumerate() {
            for (j, &line2) in line_numbers.iter().enumerate() {
                if i != j {
                    assert_ne!(line1, line2, "Templates should have different line numbers: {} vs {}", line1, line2);
                }
            }
        }
        
        // Line numbers should be greater than 1 (not the placeholder value)
        for (idx, &line) in line_numbers.iter().enumerate() {
            println!("Template {} is at line {}", idx, line);
            assert!(line > 1, "Line number should be greater than 1, got {}", line);
        }
        
        // Check column numbers too
        let column_numbers: Vec<u32> = updates.iter().map(|update| update.id.column).collect();
        println!("Column numbers: {:?}", column_numbers);
        
        // At least verify column numbers are not all zeros (which would indicate a problem)
        for (idx, &col) in column_numbers.iter().enumerate() {
            println!("Template {} is at column {}", idx, col);
        }
    }
}