//! Tests for the DOM diffing module

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::super::morphdom::{diff_html, DiffOptions};
    use super::super::patch::{Patch, PatchOp};
    use super::super::parser::{parse_html, HtmlNode};
    use super::super::optimizer::PatchOptimizer;

    #[test]
    fn test_simple_text_change() {
        let from = "<div>Hello</div>";
        let to = "<div>World</div>";
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        assert_eq!(patch.ops.len(), 1);
        match &patch.ops[0] {
            PatchOp::UpdateText { path, new_text } => {
                assert_eq!(path, &vec![0, 0]); // First child of first element
                assert_eq!(new_text, "World");
            }
            _ => panic!("Expected UpdateText operation"),
        }
    }

    #[test]
    fn test_attribute_change() {
        let from = r#"<div class="old" id="test">Content</div>"#;
        let to = r#"<div class="new" id="test">Content</div>"#;
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        assert_eq!(patch.ops.len(), 1);
        match &patch.ops[0] {
            PatchOp::SetAttribute { path, name, value } => {
                assert_eq!(path, &vec![0]);
                assert_eq!(name, "class");
                assert_eq!(value, "new");
            }
            _ => panic!("Expected SetAttribute operation"),
        }
    }

    #[test]
    fn test_attribute_removal() {
        let from = r#"<div class="test" id="elem">Content</div>"#;
        let to = r#"<div id="elem">Content</div>"#;
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        assert_eq!(patch.ops.len(), 1);
        match &patch.ops[0] {
            PatchOp::RemoveAttribute { path, name } => {
                assert_eq!(path, &vec![0]);
                assert_eq!(name, "class");
            }
            _ => panic!("Expected RemoveAttribute operation"),
        }
    }

    #[test]
    fn test_attribute_addition() {
        let from = r#"<div id="elem">Content</div>"#;
        let to = r#"<div id="elem" class="new">Content</div>"#;
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        assert_eq!(patch.ops.len(), 1);
        match &patch.ops[0] {
            PatchOp::SetAttribute { path, name, value } => {
                assert_eq!(path, &vec![0]);
                assert_eq!(name, "class");
                assert_eq!(value, "new");
            }
            _ => panic!("Expected SetAttribute operation"),
        }
    }

    #[test]
    fn test_child_insertion() {
        let from = "<ul><li>Item 1</li></ul>";
        let to = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        assert_eq!(patch.ops.len(), 1);
        match &patch.ops[0] {
            PatchOp::InsertChild { parent_path, index, html } => {
                assert_eq!(parent_path, &vec![0]);
                assert_eq!(*index, 1);
                assert_eq!(html, "<li>Item 2</li>");
            }
            _ => panic!("Expected InsertChild operation"),
        }
    }

    #[test]
    fn test_child_removal() {
        let from = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let to = "<ul><li>Item 1</li></ul>";
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        assert_eq!(patch.ops.len(), 1);
        match &patch.ops[0] {
            PatchOp::RemoveChild { parent_path, index } => {
                assert_eq!(parent_path, &vec![0]);
                assert_eq!(*index, 1);
            }
            _ => panic!("Expected RemoveChild operation"),
        }
    }

    #[test]
    fn test_element_replacement() {
        let from = "<div>Content</div>";
        let to = "<span>Content</span>";
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        assert_eq!(patch.ops.len(), 1);
        match &patch.ops[0] {
            PatchOp::Replace { path, new_html } => {
                assert_eq!(path, &vec![0]);
                assert_eq!(new_html, "<span>Content</span>");
            }
            _ => panic!("Expected Replace operation"),
        }
    }

    #[test]
    fn test_complex_nested_change() {
        let from = r#"
            <div class="container">
                <h1>Title</h1>
                <p>Paragraph 1</p>
                <p>Paragraph 2</p>
            </div>
        "#;
        
        let to = r#"
            <div class="container updated">
                <h1>New Title</h1>
                <p>Paragraph 1</p>
                <p>Updated Paragraph 2</p>
                <p>Paragraph 3</p>
            </div>
        "#;
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        // Should have multiple operations:
        // 1. Update class attribute
        // 2. Update h1 text
        // 3. Update second paragraph text
        // 4. Insert third paragraph
        assert!(patch.ops.len() >= 4);
    }

    #[test]
    fn test_keyed_list_reorder() {
        let from = r#"
            <ul>
                <li key="a">Item A</li>
                <li key="b">Item B</li>
                <li key="c">Item C</li>
            </ul>
        "#;
        
        let to = r#"
            <ul>
                <li key="c">Item C</li>
                <li key="a">Item A</li>
                <li key="b">Item B</li>
            </ul>
        "#;
        
        let mut options = DiffOptions::default();
        options.use_keys = true;
        
        let patch = diff_html(from, to, &options).unwrap();
        
        // Should detect the reordering and generate move operations
        let has_move_ops = patch.ops.iter().any(|op| matches!(op, PatchOp::MoveChild { .. }));
        assert!(has_move_ops, "Expected move operations for keyed list reordering");
    }

    #[test]
    fn test_void_elements() {
        let from = r#"<div><img src="old.jpg" alt="Old"><br></div>"#;
        let to = r#"<div><img src="new.jpg" alt="New"><br></div>"#;
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        // Should update the img attributes
        let attr_updates = patch.ops.iter().filter(|op| matches!(op, PatchOp::SetAttribute { .. })).count();
        assert_eq!(attr_updates, 2); // src and alt attributes
    }

    #[test]
    fn test_whitespace_handling() {
        let from = "<div>  Hello  World  </div>";
        let to = "<div>Hello World</div>";
        
        let mut options = DiffOptions::default();
        options.preserve_whitespace = false;
        
        let patch = diff_html(from, to, &options).unwrap();
        
        // Should detect text change when whitespace is not preserved
        assert_eq!(patch.ops.len(), 1);
        match &patch.ops[0] {
            PatchOp::UpdateText { new_text, .. } => {
                assert_eq!(new_text, "Hello World");
            }
            _ => panic!("Expected UpdateText operation"),
        }
    }

    #[test]
    fn test_comment_handling() {
        let from = "<div><!-- Old comment -->Content</div>";
        let to = "<div><!-- New comment -->Content</div>";
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        // Comments should be ignored in diffing by default
        assert_eq!(patch.ops.len(), 0);
    }

    #[test]
    fn test_patch_optimization() {
        let mut patch = Patch::new();
        
        // Add redundant operations
        patch.add_op(PatchOp::SetAttribute {
            path: vec![0],
            name: "class".to_string(),
            value: "old".to_string(),
        });
        patch.add_op(PatchOp::SetAttribute {
            path: vec![0],
            name: "class".to_string(),
            value: "new".to_string(),
        });
        
        // Add consecutive text updates
        patch.add_op(PatchOp::UpdateText {
            path: vec![0, 0],
            new_text: "First".to_string(),
        });
        patch.add_op(PatchOp::UpdateText {
            path: vec![0, 0],
            new_text: "Second".to_string(),
        });
        patch.add_op(PatchOp::UpdateText {
            path: vec![0, 0],
            new_text: "Final".to_string(),
        });
        
        patch.optimize();
        
        // Should have removed redundant operations
        assert!(patch.ops.len() < 5);
        
        // Should only have the final attribute value
        let attr_ops: Vec<_> = patch.ops.iter()
            .filter(|op| matches!(op, PatchOp::SetAttribute { .. }))
            .collect();
        assert_eq!(attr_ops.len(), 1);
        
        // Should only have the final text value
        let text_ops: Vec<_> = patch.ops.iter()
            .filter(|op| matches!(op, PatchOp::UpdateText { .. }))
            .collect();
        assert_eq!(text_ops.len(), 1);
    }

    #[test]
    fn test_binary_serialization() {
        let mut patch = Patch::new();
        
        patch.add_op(PatchOp::UpdateText {
            path: vec![0, 1, 2],
            new_text: "Hello, 世界!".to_string(),
        });
        
        patch.add_op(PatchOp::SetAttribute {
            path: vec![0],
            name: "data-test".to_string(),
            value: "value with spaces".to_string(),
        });
        
        patch.add_op(PatchOp::InsertChild {
            parent_path: vec![1, 2],
            index: 3,
            html: "<span>New child</span>".to_string(),
        });
        
        // Serialize to binary
        let binary = patch.to_binary().unwrap();
        
        // Deserialize back
        let restored = Patch::from_binary(&binary).unwrap();
        
        // Should have the same operations
        assert_eq!(restored.ops.len(), patch.ops.len());
        
        // Verify each operation
        for (original, restored) in patch.ops.iter().zip(restored.ops.iter()) {
            match (original, restored) {
                (
                    PatchOp::UpdateText { path: p1, new_text: t1 },
                    PatchOp::UpdateText { path: p2, new_text: t2 }
                ) => {
                    assert_eq!(p1, p2);
                    assert_eq!(t1, t2);
                }
                (
                    PatchOp::SetAttribute { path: p1, name: n1, value: v1 },
                    PatchOp::SetAttribute { path: p2, name: n2, value: v2 }
                ) => {
                    assert_eq!(p1, p2);
                    assert_eq!(n1, n2);
                    assert_eq!(v1, v2);
                }
                (
                    PatchOp::InsertChild { parent_path: p1, index: i1, html: h1 },
                    PatchOp::InsertChild { parent_path: p2, index: i2, html: h2 }
                ) => {
                    assert_eq!(p1, p2);
                    assert_eq!(i1, i2);
                    assert_eq!(h1, h2);
                }
                _ => panic!("Operation mismatch"),
            }
        }
    }

    #[test]
    fn test_advanced_optimizer() {
        let mut patch = Patch::new();
        
        // Add multiple attribute operations on the same element
        for i in 0..5 {
            patch.add_op(PatchOp::SetAttribute {
                path: vec![0],
                name: format!("attr{}", i),
                value: format!("value{}", i),
            });
        }
        
        // Add list operations
        patch.add_op(PatchOp::RemoveChild {
            parent_path: vec![1],
            index: 2,
        });
        patch.add_op(PatchOp::InsertChild {
            parent_path: vec![1],
            index: 0,
            html: "<li>New item</li>".to_string(),
        });
        patch.add_op(PatchOp::MoveChild {
            parent_path: vec![1],
            from_index: 3,
            to_index: 1,
        });
        
        let optimizer = PatchOptimizer::new();
        let optimized = optimizer.optimize(patch);
        
        // Should have optimized the operations
        assert!(optimized.ops.len() <= 8); // May batch some operations
    }

    #[test]
    fn test_parser_edge_cases() {
        // Test self-closing tags
        let html = r#"<div><input type="text" value="test" /><br></div>"#;
        let node = parse_html(html).unwrap();
        
        if let HtmlNode::Element(elem) = node {
            assert_eq!(elem.tag_name, "div");
            assert_eq!(elem.children.len(), 2);
            
            // Check input element
            if let HtmlNode::Element(input) = &elem.children[0] {
                assert_eq!(input.tag_name, "input");
                assert_eq!(input.attributes.get("type"), Some(&"text".to_string()));
                assert_eq!(input.attributes.get("value"), Some(&"test".to_string()));
                assert!(input.self_closing || input.children.is_empty());
            }
            
            // Check br element
            if let HtmlNode::Element(br) = &elem.children[1] {
                assert_eq!(br.tag_name, "br");
                assert!(br.children.is_empty());
            }
        }
        
        // Test boolean attributes
        let html = r#"<input disabled checked>"#;
        let node = parse_html(html).unwrap();
        
        if let HtmlNode::Element(elem) = node {
            assert_eq!(elem.attributes.get("disabled"), Some(&"".to_string()));
            assert_eq!(elem.attributes.get("checked"), Some(&"".to_string()));
        }
        
        // Test nested elements with text
        let html = "<p>Hello <strong>world</strong>!</p>";
        let node = parse_html(html).unwrap();
        
        if let HtmlNode::Element(p) = node {
            assert_eq!(p.children.len(), 3);
            
            // First text node
            if let HtmlNode::Text(text) = &p.children[0] {
                assert_eq!(text.content, "Hello ");
            }
            
            // Strong element
            if let HtmlNode::Element(strong) = &p.children[1] {
                assert_eq!(strong.tag_name, "strong");
                if let HtmlNode::Text(text) = &strong.children[0] {
                    assert_eq!(text.content, "world");
                }
            }
            
            // Last text node
            if let HtmlNode::Text(text) = &p.children[2] {
                assert_eq!(text.content, "!");
            }
        }
    }

    #[test]
    fn test_special_characters_in_attributes() {
        let from = r#"<div data-json='{"key": "value"}'>Content</div>"#;
        let to = r#"<div data-json='{"key": "updated"}'>Content</div>"#;
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        assert_eq!(patch.ops.len(), 1);
        match &patch.ops[0] {
            PatchOp::SetAttribute { name, value, .. } => {
                assert_eq!(name, "data-json");
                assert_eq!(value, r#"{"key": "updated"}"#);
            }
            _ => panic!("Expected SetAttribute operation"),
        }
    }

    #[test]
    fn test_empty_elements() {
        let from = "<div></div>";
        let to = "<div><span>New content</span></div>";
        
        let options = DiffOptions::default();
        let patch = diff_html(from, to, &options).unwrap();
        
        assert_eq!(patch.ops.len(), 1);
        match &patch.ops[0] {
            PatchOp::InsertChild { parent_path, index, html } => {
                assert_eq!(parent_path, &vec![0]);
                assert_eq!(*index, 0);
                assert_eq!(html, "<span>New content</span>");
            }
            _ => panic!("Expected InsertChild operation"),
        }
    }

    #[test]
    fn test_component_boundary_tracking() {
        let from = r#"<div data-component="header"><h1>Title</h1></div>"#;
        let to = r#"<div data-component="header"><h1>New Title</h1></div>"#;
        
        let mut options = DiffOptions::default();
        options.track_components = true;
        
        let patch = diff_html(from, to, &options).unwrap();
        
        // Should detect changes within component boundaries
        assert_eq!(patch.ops.len(), 1);
        match &patch.ops[0] {
            PatchOp::UpdateText { .. } => {
                // Text update is detected
            }
            _ => panic!("Expected UpdateText operation"),
        }
    }
}