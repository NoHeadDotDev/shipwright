use shipwright_liveview_hotreload::{
    protocol::{HotReloadMessage, TemplateId, TemplateUpdate},
    template_cache::TemplateCache,
    HotReloadServer,
};
use std::path::PathBuf;
use std::net::SocketAddr;

#[test]
fn test_hot_reload_message_serialization() {
    let template_id = TemplateId::new(
        PathBuf::from("test.rs"),
        10,
        5,
    );
    
    let template_update = TemplateUpdate {
        id: template_id,
        hash: "test_hash".to_string(),
        content_hash: "content_hash".to_string(),
        html: "<div>Test</div>".to_string(),
        dynamic_parts: vec![],
    };
    
    let message = HotReloadMessage::TemplateUpdated(template_update);
    let json = message.to_json().expect("Failed to serialize message");
    
    assert!(json.contains("template_updated"));
    assert!(json.contains("test_hash"));
    assert!(json.contains("<div>Test</div>"));
    
    // Test deserialization
    let deserialized = HotReloadMessage::from_json(&json).expect("Failed to deserialize message");
    match deserialized {
        HotReloadMessage::TemplateUpdated(update) => {
            assert_eq!(update.hash, "test_hash");
            assert_eq!(update.html, "<div>Test</div>");
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_server_creation() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let watch_paths = vec![PathBuf::from("src")];
    
    let server = HotReloadServer::new(addr, watch_paths);
    // Server created successfully - this verifies the basic instantiation works
    assert_eq!(server.addr, addr);
}

#[test]
fn test_template_cache() {
    let cache = TemplateCache::new();
    
    let template_id = TemplateId::new(
        PathBuf::from("test.rs"),
        1,
        1,
    );
    
    let template_update = TemplateUpdate {
        id: template_id.clone(),
        hash: "test_hash".to_string(),
        content_hash: "content_hash".to_string(),
        html: "<div>Test</div>".to_string(),
        dynamic_parts: vec![],
    };
    
    // Test insertion
    let (is_new, content_changed) = cache.insert(template_update.clone());
    assert!(is_new);
    assert!(content_changed);
    
    // Test retrieval
    let retrieved = cache.get(&template_id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().hash, "test_hash");
    
    // Test duplicate insertion
    let (is_new, content_changed) = cache.insert(template_update.clone());
    assert!(!is_new);
    assert!(!content_changed);
    
    // Test content change detection
    let mut updated_template = template_update.clone();
    updated_template.content_hash = "new_content_hash".to_string();
    updated_template.html = "<div>Updated</div>".to_string();
    
    let (is_new, content_changed) = cache.insert(updated_template);
    assert!(!is_new);
    assert!(content_changed);
}

#[test]
fn test_enhanced_error_messages() {
    let error_msg = HotReloadMessage::Error {
        message: "Test error".to_string(),
        code: Some("TEST_ERROR".to_string()),
        suggestions: Some(vec!["Try again".to_string()]),
    };
    
    let json = error_msg.to_json().expect("Failed to serialize error");
    assert!(json.contains("Test error"));
    assert!(json.contains("TEST_ERROR"));
    assert!(json.contains("Try again"));
}