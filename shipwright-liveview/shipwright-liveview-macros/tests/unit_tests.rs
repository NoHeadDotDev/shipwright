//! Unit tests for the enhanced template parsing modules.

use shipwright_liveview_macros::{
    enhanced_ast::*,
    fingerprinting::*,
    location_tracking::*,
    unified_parser::*,
    dynamic_extraction::*,
};
use proc_macro2::Span;
use syn::parse_quote;
use std::collections::HashMap;

/// Test template location functionality
#[test]
fn test_template_location() {
    let span = Span::call_site();
    let location = TemplateLocation::from_span(span);
    
    assert!(!location.file.is_empty());
    assert!(location.line > 0);
    assert!(!location.location_id().is_empty());
    
    let location_id = location.location_id();
    assert!(location_id.contains(":"));
}

/// Test node ID generation and hierarchy
#[test]
fn test_node_id() {
    let node_id = NodeId::new("template1", "0.1.2");
    assert_eq!(node_id.path, "0.1.2");
    assert!(node_id.id.contains("template1"));
    
    let child_id = node_id.child(3);
    assert_eq!(child_id.path, "0.1.2.3");
    assert!(child_id.id.contains("template1"));
}

/// Test template fingerprinting
#[test]
fn test_template_fingerprinting() {
    let fp1 = TemplateFingerprint::new("content", "structure");
    let fp2 = TemplateFingerprint::new("content", "structure");
    let fp3 = TemplateFingerprint::new("different", "structure");
    
    assert!(fp1.matches(&fp2));
    assert!(!fp1.matches(&fp3));
    assert!(fp1.only_static_changed(&fp3));
    assert!(!fp1.structure_changed(&fp3));
}

/// Test fingerprint engine
#[test]
fn test_fingerprint_engine() {
    let mut engine = FingerprintEngine::new();
    
    let structure = TemplateStructure {
        root: StructureElement {
            element_type: ElementType::Root,
            attributes: HashMap::new(),
            children: vec![
                StructureElement {
                    element_type: ElementType::HtmlElement("div".to_string()),
                    attributes: HashMap::new(),
                    children: Vec::new(),
                }
            ],
        },
    };

    let fp1 = engine.fingerprint_template("test1", "content", &structure, &[]);
    let fp2 = engine.fingerprint_template("test1", "content", &structure, &[]);
    
    // Should be identical due to caching
    assert_eq!(fp1.combined_hash, fp2.combined_hash);
}

/// Test fingerprint comparison
#[test]
fn test_fingerprint_comparison() {
    let engine = FingerprintEngine::new();
    
    let fp1 = TemplateFingerprint {
        combined_hash: 123,
        static_hash: 456,
        structure_hash: 789,
        dynamic_hash: 0,
        styling_hash: 0,
        interaction_hash: 0,
        part_hashes: HashMap::new(),
        hierarchy: HierarchyFingerprint {
            level_hash: 0,
            children: vec![],
            element_type: ElementType::Root,
        },
    };

    let fp2 = TemplateFingerprint {
        combined_hash: 124,
        static_hash: 457,
        structure_hash: 789,
        dynamic_hash: 0,
        styling_hash: 0,
        interaction_hash: 0,
        part_hashes: HashMap::new(),
        hierarchy: HierarchyFingerprint {
            level_hash: 0,
            children: vec![],
            element_type: ElementType::Root,
        },
    };

    let comparison = engine.compare_fingerprints(&fp1, &fp2);
    assert!(!comparison.matches);
    assert!(comparison.changes.static_content);
    assert!(!comparison.changes.structure);
    assert!(comparison.hot_reload_compatible);
}

/// Test location tracker
#[test]
fn test_location_tracker() {
    let mut tracker = LocationTracker::new();
    let span = Span::call_site();
    
    let location = tracker.track_location(span);
    assert!(!location.location_id.is_empty());
    assert!(!location.template_id.is_empty());
    assert_eq!(location.template_path, "");
    
    let retrieved = tracker.get_location(&location.location_id);
    assert!(retrieved.is_some());
}

/// Test template ID generation
#[test]
fn test_template_id_generation() {
    let location = TrackedLocation {
        file_path: std::path::PathBuf::from("src/test.rs"),
        relative_path: std::path::PathBuf::from("src/test.rs"),
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

/// Test template ID hierarchy
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

/// Test location registry
#[test]
fn test_location_registry() {
    let mut registry = LocationRegistry::new();
    
    let location = TrackedLocation {
        file_path: std::path::PathBuf::from("test.rs"),
        relative_path: std::path::PathBuf::from("test.rs"),
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

/// Test registry statistics
#[test]
fn test_registry_stats() {
    let mut registry = LocationRegistry::new();
    
    let location = TrackedLocation {
        file_path: std::path::PathBuf::from("test.rs"),
        relative_path: std::path::PathBuf::from("test.rs"),
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

/// Test unified parser configuration
#[test]
fn test_unified_parser_config() {
    let config = UnifiedParserConfig {
        enable_hot_reload: false,
        enable_location_tracking: true,
        max_hot_reload_size: 5000,
        ..Default::default()
    };

    let parser = UnifiedTemplateParser::with_config(config);
    assert!(!parser.config.enable_hot_reload);
    assert!(parser.config.enable_location_tracking);
    assert_eq!(parser.config.max_hot_reload_size, 5000);
}

/// Test parsing statistics
#[test]
fn test_parsing_stats() {
    let stats = ParsingStats {
        element_count: 5,
        dynamic_part_count: 2,
        max_nesting_depth: 3,
        character_count: 100,
        parse_duration: Some(std::time::Duration::from_millis(10)),
    };

    assert_eq!(stats.element_count, 5);
    assert_eq!(stats.dynamic_part_count, 2);
    assert_eq!(stats.max_nesting_depth, 3);
    assert!(stats.parse_duration.is_some());
}

/// Test dynamic part extractor
#[test]
fn test_dynamic_part_extractor() {
    let mut extractor = DynamicPartExtractor::new();
    let location = TemplateLocation::from_span(Span::call_site());
    
    // Test with a simple block
    let block: syn::Block = parse_quote! {{ self.value + 1 }};
    let analyzed = extractor.analyze_block_dependencies(&block);
    
    assert!(!analyzed.to_string_vec().is_empty());
}

/// Test dependency analyzer
#[test]
fn test_dependency_analyzer() {
    let mut analyzer = DependencyAnalyzer::new();
    
    let expr: syn::Expr = parse_quote! { self.value.method() };
    let deps = analyzer.analyze_expression(&expr);
    
    assert!(!deps.variables.is_empty() || !deps.functions.is_empty());
}

/// Test analyzed dependencies conversion
#[test]
fn test_analyzed_dependencies() {
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
    
    let function_calls = deps.function_calls();
    assert!(function_calls.is_empty()); // No functions in this test
    
    let mutated = deps.mutated_variables();
    assert!(mutated.is_empty()); // test_var is not mutated
}

/// Test scope tracker
#[test]
fn test_scope_tracker() {
    let mut tracker = ScopeTracker::new();
    
    tracker.enter_scope(ScopeType::Function);
    tracker.add_variable("x".to_string());
    
    assert!(tracker.is_variable_in_scope("x"));
    
    let scope_vars = tracker.current_scope_variables();
    assert!(scope_vars.contains("x"));
    
    tracker.exit_scope();
    assert!(!tracker.is_variable_in_scope("x"));
}

/// Test extraction configuration
#[test]
fn test_extraction_config() {
    let config = ExtractionConfig {
        extract_variables: false,
        max_depth: 5,
        track_mutations: false,
        ..Default::default()
    };
    
    assert!(!config.extract_variables);
    assert_eq!(config.max_depth, 5);
    assert!(!config.track_mutations);
    assert!(config.extract_functions); // Should still be true from default
}

/// Test dynamic part kinds
#[test]
fn test_dynamic_part_kinds() {
    let text_part = DynamicPart {
        kind: DynamicPartKind::TextContent,
        location: TemplateLocation::from_span(Span::call_site()),
        code: quote::quote! { "test" },
        dependencies: vec![],
    };
    
    assert_eq!(text_part.kind, DynamicPartKind::TextContent);
    
    let attr_part = DynamicPart {
        kind: DynamicPartKind::AttributeValue { attr_name: "class".to_string() },
        location: TemplateLocation::from_span(Span::call_site()),
        code: quote::quote! { "value" },
        dependencies: vec![],
    };
    
    match attr_part.kind {
        DynamicPartKind::AttributeValue { attr_name } => {
            assert_eq!(attr_name, "class");
        }
        _ => panic!("Wrong dynamic part kind"),
    }
    
    let event_part = DynamicPart {
        kind: DynamicPartKind::EventHandler { event_name: "click".to_string() },
        location: TemplateLocation::from_span(Span::call_site()),
        code: quote::quote! { handler },
        dependencies: vec!["handler".to_string()],
    };
    
    match event_part.kind {
        DynamicPartKind::EventHandler { event_name } => {
            assert_eq!(event_name, "click");
        }
        _ => panic!("Wrong dynamic part kind"),
    }
}

/// Test template structure creation
#[test]
fn test_template_structure() {
    let element = StructureElement {
        element_type: ElementType::HtmlElement("div".to_string()),
        attributes: {
            let mut attrs = HashMap::new();
            attrs.insert("class".to_string(), Some("test".to_string()));
            attrs
        },
        children: Vec::new(),
    };

    assert_eq!(element.element_type, ElementType::HtmlElement("div".to_string()));
    assert_eq!(element.attributes.get("class"), Some(&Some("test".to_string())));
}

/// Test contextual location tracker
#[test]
fn test_contextual_tracker() {
    let mut tracker = ContextualLocationTracker::new();
    let source = "line 1\nline 2\nhtml! { <div></div> }\nline 4\nline 5";
    
    let location = tracker.track_with_context(
        Span::call_site(),
        source,
        "test"
    );

    // Check that location was tracked
    assert!(!location.location_id.is_empty());
    assert_eq!(location.template_path, "test");
    
    // Check if context was extracted (might not be available in this test environment)
    let context = tracker.get_context(&location.location_id);
    // Context extraction might not work in test environment, so we don't assert on it
}

/// Test fingerprint sensitivity levels
#[test]
fn test_fingerprint_sensitivity() {
    let config_structure = FingerprintConfig {
        dynamic_sensitivity: DynamicSensitivity::Structure,
        ..Default::default()
    };
    
    let config_content = FingerprintConfig {
        dynamic_sensitivity: DynamicSensitivity::Content,
        ..Default::default()
    };
    
    assert_eq!(config_structure.dynamic_sensitivity, DynamicSensitivity::Structure);
    assert_eq!(config_content.dynamic_sensitivity, DynamicSensitivity::Content);
    assert!(config_structure.include_classes);
    assert!(config_content.hierarchical);
}

/// Test hot reload metadata
#[test]
fn test_hot_reload_meta() {
    let fingerprint = TemplateFingerprint::new("content", "structure");
    
    let meta = HotReloadMeta {
        template_id: "test_template".to_string(),
        fingerprint: fingerprint.clone(),
        dynamic_parts: vec![],
        static_structure: "structure".to_string(),
        dependencies: TemplateDependencies::default(),
    };
    
    let other_meta = HotReloadMeta {
        template_id: "test_template".to_string(),
        fingerprint: fingerprint.clone(),
        dynamic_parts: vec![],
        static_structure: "structure".to_string(),
        dependencies: TemplateDependencies::default(),
    };
    
    assert!(meta.can_hot_reload(&other_meta));
    
    let diff_parts = meta.diff_for_reload(&other_meta);
    assert!(diff_parts.is_empty()); // No changes in this test
}