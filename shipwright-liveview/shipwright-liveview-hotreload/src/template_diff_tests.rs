//! Comprehensive tests for the template diffing engine
//!
//! This module provides extensive testing of the template diffing capabilities,
//! including edge cases and performance scenarios.

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::{
        template_diff::*,
        protocol::{TemplateUpdate, TemplateId, DynamicPart, DynamicKind},
        diff_integration::{HotReloadDecisionMaker, DiffAwareTemplateCache},
    };

    fn create_template_update(
        id: &str,
        html: &str,
        dynamic_parts: Vec<DynamicPart>,
    ) -> TemplateUpdate {
        TemplateUpdate {
            id: TemplateId::new(PathBuf::from("test.rs"), 1, 1),
            hash: id.to_string(),
            content_hash: format!("{}_content", id),
            html: html.to_string(),
            dynamic_parts,
        }
    }

    #[test]
    fn test_simple_text_change() {
        let mut differ = TemplateDiffer::new();
        
        let old = create_template_update("t1", "<div>Hello World</div>", vec![]);
        let new = create_template_update("t1", "<div>Hello Universe</div>", vec![]);
        
        let result = differ.diff_updates(&old, &new).unwrap();
        
        assert!(result.compatible);
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.compatibility_issues.len(), 0);
        
        match &result.changes[0] {
            TemplateChange::TextChanged { old, new, .. } => {
                assert!(old.contains("World"));
                assert!(new.contains("Universe"));
            }
            _ => panic!("Expected text change"),
        }
    }

    #[test]
    fn test_dynamic_parts_compatibility() {
        let differ = TemplateDiffer::new();
        
        // Compatible dynamic parts
        assert!(differ.dynamic_parts_compatible(
            &DynamicKind::Expression,
            &DynamicKind::Expression
        ));
        
        assert!(differ.dynamic_parts_compatible(
            &DynamicKind::Conditional,
            &DynamicKind::Conditional
        ));
        
        assert!(differ.dynamic_parts_compatible(
            &DynamicKind::Loop,
            &DynamicKind::Loop
        ));
        
        assert!(differ.dynamic_parts_compatible(
            &DynamicKind::EventHandler { event: "click".to_string() },
            &DynamicKind::EventHandler { event: "click".to_string() }
        ));
        
        // Incompatible dynamic parts
        assert!(!differ.dynamic_parts_compatible(
            &DynamicKind::Expression,
            &DynamicKind::Loop
        ));
        
        assert!(!differ.dynamic_parts_compatible(
            &DynamicKind::EventHandler { event: "click".to_string() },
            &DynamicKind::EventHandler { event: "mouseover".to_string() }
        ));
        
        assert!(!differ.dynamic_parts_compatible(
            &DynamicKind::Conditional,
            &DynamicKind::Expression
        ));
    }

    #[test]
    fn test_dynamic_part_count_mismatch() {
        let mut differ = TemplateDiffer::new();
        
        let old = create_template_update(
            "t1",
            "<div>{count}</div>",
            vec![DynamicPart { index: 0, kind: DynamicKind::Expression }],
        );
        
        let new = create_template_update(
            "t1",
            "<div>{count} - {total}</div>",
            vec![
                DynamicPart { index: 0, kind: DynamicKind::Expression },
                DynamicPart { index: 1, kind: DynamicKind::Expression },
            ],
        );
        
        let result = differ.diff_updates(&old, &new).unwrap();
        
        assert!(!result.compatible);
        assert!(!result.compatibility_issues.is_empty());
        
        match &result.compatibility_issues[0] {
            CompatibilityIssue::DynamicPartCountMismatch { old, new } => {
                assert_eq!(*old, 1);
                assert_eq!(*new, 2);
            }
            _ => panic!("Expected dynamic part count mismatch"),
        }
    }

    #[test]
    fn test_dynamic_part_type_mismatch() {
        let mut differ = TemplateDiffer::new();
        
        let old = create_template_update(
            "t1",
            "<div>{count}</div>",
            vec![DynamicPart { index: 0, kind: DynamicKind::Expression }],
        );
        
        let new = create_template_update(
            "t1",
            "<div>{for item in items}</div>",
            vec![DynamicPart { index: 0, kind: DynamicKind::Loop }],
        );
        
        let result = differ.diff_updates(&old, &new).unwrap();
        
        assert!(!result.compatible);
        assert!(!result.compatibility_issues.is_empty());
        
        match &result.compatibility_issues[0] {
            CompatibilityIssue::DynamicPartTypeMismatch { index, old, new } => {
                assert_eq!(*index, 0);
                assert_eq!(*old, DynamicKind::Expression);
                assert_eq!(*new, DynamicKind::Loop);
            }
            _ => panic!("Expected dynamic part type mismatch"),
        }
    }

    #[test]
    fn test_template_node_equality() {
        let node1 = TemplateNode::Text("Hello".to_string());
        let node2 = TemplateNode::Text("Hello".to_string());
        let node3 = TemplateNode::Text("World".to_string());
        
        assert_eq!(node1, node2);
        assert_ne!(node1, node3);
        
        let element1 = TemplateNode::Element {
            tag: "div".to_string(),
            attributes: vec![],
            children: vec![],
        };
        
        let element2 = TemplateNode::Element {
            tag: "div".to_string(),
            attributes: vec![],
            children: vec![],
        };
        
        let element3 = TemplateNode::Element {
            tag: "span".to_string(),
            attributes: vec![],
            children: vec![],
        };
        
        assert_eq!(element1, element2);
        assert_ne!(element1, element3);
    }

    #[test]
    fn test_attribute_value_types() {
        let static_attr = AttributeValue::Static("test".to_string());
        let dynamic_attr = AttributeValue::Dynamic("count".to_string());
        let event_attr = AttributeValue::EventHandler("onClick".to_string());
        
        assert_ne!(static_attr, dynamic_attr);
        assert_ne!(static_attr, event_attr);
        assert_ne!(dynamic_attr, event_attr);
        
        let static_attr2 = AttributeValue::Static("test".to_string());
        assert_eq!(static_attr, static_attr2);
    }

    #[test]
    fn test_compatibility_rules() {
        let checker = CompatibilityChecker::new();
        
        // Test root element rule
        let old_ast = TemplateAst {
            roots: vec![TemplateNode::Element {
                tag: "div".to_string(),
                attributes: vec![],
                children: vec![],
            }],
            dynamic_parts: vec![],
            original_html: "<div></div>".to_string(),
        };
        
        let new_ast = TemplateAst {
            roots: vec![TemplateNode::Element {
                tag: "span".to_string(),
                attributes: vec![],
                children: vec![],
            }],
            dynamic_parts: vec![],
            original_html: "<span></span>".to_string(),
        };
        
        let issues = checker.check(&old_ast, &new_ast, &[]);
        assert!(!issues.is_empty());
        
        match &issues[0] {
            CompatibilityIssue::RootElementChanged { old, new } => {
                assert_eq!(old, "div");
                assert_eq!(new, "span");
            }
            _ => panic!("Expected root element changed issue"),
        }
    }

    #[test]
    fn test_batch_operation_builder() {
        let mut builder = BatchOperationBuilder::new();
        
        // Add operations that should be optimized
        builder.add_operation(DeltaOperation::UpdateText {
            path: vec![0, 1],
            content: "First".to_string(),
        });
        
        builder.add_operation(DeltaOperation::UpdateText {
            path: vec![0, 1],
            content: "Second".to_string(),
        });
        
        builder.add_operation(DeltaOperation::UpdateText {
            path: vec![0, 2],
            content: "Third".to_string(),
        });
        
        let ops = builder.build();
        
        // Should deduplicate operations on the same path
        assert_eq!(ops.len(), 2);
        
        // Should keep the last operation for each path
        match &ops[0] {
            DeltaOperation::UpdateText { content, .. } => {
                assert_eq!(content, "Second");
            }
            _ => panic!("Expected text update"),
        }
        
        match &ops[1] {
            DeltaOperation::UpdateText { content, .. } => {
                assert_eq!(content, "Third");
            }
            _ => panic!("Expected text update"),
        }
    }

    #[test]
    fn test_hot_reload_decision_maker() {
        let mut decision_maker = HotReloadDecisionMaker::new();
        
        // Test new template
        let update1 = create_template_update("t1", "<div>Hello</div>", vec![]);
        let analysis = decision_maker.analyze_updates(vec![update1.clone()]).unwrap();
        
        assert_eq!(analysis.hot_reloadable.len(), 1);
        assert_eq!(analysis.require_rebuild.len(), 0);
        
        // Test compatible change
        let update2 = create_template_update("t1", "<div>World</div>", vec![]);
        let analysis = decision_maker.analyze_updates(vec![update2]).unwrap();
        
        assert_eq!(analysis.hot_reloadable.len(), 1);
        assert_eq!(analysis.require_rebuild.len(), 0);
        
        // Test incompatible change
        let update3 = create_template_update(
            "t1",
            "<div>{count}</div>",
            vec![DynamicPart { index: 0, kind: DynamicKind::Expression }],
        );
        let analysis = decision_maker.analyze_updates(vec![update3]).unwrap();
        
        assert_eq!(analysis.hot_reloadable.len(), 0);
        assert_eq!(analysis.require_rebuild.len(), 1);
    }

    #[test]
    fn test_diff_aware_cache() {
        let mut cache = DiffAwareTemplateCache::new(100);
        
        let update1 = create_template_update("t1", "<div>Hello</div>", vec![]);
        let update2 = create_template_update("t2", "<span>World</span>", vec![]);
        
        let analysis = cache.insert_with_analysis(vec![update1.clone(), update2.clone()]).unwrap();
        
        assert_eq!(analysis.hot_reloadable.len(), 2);
        assert_eq!(analysis.require_rebuild.len(), 0);
        
        // Verify templates are in cache
        assert!(cache.get(&update1.id).is_some());
        assert!(cache.get(&update2.id).is_some());
        
        // Test cache stats
        let (cache_stats, decision_stats) = cache.stats();
        assert_eq!(cache_stats.size, 2);
        assert_eq!(decision_stats.cached_templates, 2);
    }

    #[test]
    fn test_enhanced_template_parser() {
        // Test the enhanced parser placeholder
        let html = "<div><span>Hello {name}</span></div>";
        let nodes = EnhancedTemplateParser::parse(html).unwrap();
        
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            TemplateNode::Text(content) => {
                assert_eq!(content, html);
            }
            _ => panic!("Expected text node"),
        }
    }

    #[test]
    fn test_complex_template_diff() {
        let mut differ = TemplateDiffer::new();
        
        // Complex template with multiple dynamic parts
        let old = create_template_update(
            "complex",
            "<div class='container'><h1>{title}</h1><ul>{for item in items}<li>{item.name}</li>{/for}</ul></div>",
            vec![
                DynamicPart { index: 0, kind: DynamicKind::Expression },
                DynamicPart { index: 1, kind: DynamicKind::Loop },
                DynamicPart { index: 2, kind: DynamicKind::Expression },
            ],
        );
        
        // Compatible change - just text modification
        let new = create_template_update(
            "complex",
            "<div class='container'><h1>{title}</h1><ul>{for item in items}<li>{item.label}</li>{/for}</ul></div>",
            vec![
                DynamicPart { index: 0, kind: DynamicKind::Expression },
                DynamicPart { index: 1, kind: DynamicKind::Loop },
                DynamicPart { index: 2, kind: DynamicKind::Expression },
            ],
        );
        
        let result = differ.diff_updates(&old, &new).unwrap();
        
        assert!(result.compatible);
        assert!(!result.changes.is_empty());
        assert!(result.compatibility_issues.is_empty());
    }

    #[test]
    fn test_performance_large_template() {
        let mut differ = TemplateDiffer::new();
        
        // Generate a large template with many dynamic parts
        let mut html = "<div>".to_string();
        let mut dynamic_parts = Vec::new();
        
        for i in 0..100 {
            html.push_str(&format!("<p>Item {}: {{item_{}}}</p>", i, i));
            dynamic_parts.push(DynamicPart {
                index: i,
                kind: DynamicKind::Expression,
            });
        }
        html.push_str("</div>");
        
        let old = create_template_update("large", &html, dynamic_parts.clone());
        
        // Make a small change
        let mut new_html = html.replace("Item 50", "Item Fifty");
        let new = create_template_update("large", &new_html, dynamic_parts);
        
        let start = std::time::Instant::now();
        let result = differ.diff_updates(&old, &new).unwrap();
        let elapsed = start.elapsed();
        
        // Should complete quickly (< 100ms for this size)
        assert!(elapsed.as_millis() < 100);
        assert!(result.compatible);
        assert!(!result.changes.is_empty());
    }
}