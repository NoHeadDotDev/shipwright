//! Integration tests for hot reload system

use shipwright_liveview_hotreload::{
    parser::TemplateParser,
    protocol::{HotReloadMessage, TemplateId},
    template_cache::TemplateCache,
    HotReloadServer,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

#[tokio::test]
async fn test_template_parser() {
    let content = r#"
        use axum_live_view::html;
        
        fn render() {
            html! {
                <div>
                    <h1>Test</h1>
                    <button axm-click={ Msg::Click }>Click me</button>
                    { self.count }
                </div>
            }
        }
    "#;

    let mut parser = TemplateParser::new("test.rs");
    let updates = parser.parse_file(content).unwrap();
    
    assert_eq!(updates.len(), 1);
    let update = &updates[0];
    assert_eq!(update.id.file, PathBuf::from("test.rs"));
    assert!(!update.dynamic_parts.is_empty());
}

#[tokio::test]
async fn test_template_cache() {
    let cache = TemplateCache::new();
    let id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
    
    let update = shipwright_liveview_hotreload::protocol::TemplateUpdate {
        id: id.clone(),
        hash: id.hash(),
        html: "<div>Test</div>".to_string(),
        dynamic_parts: vec![],
    };
    
    // Test insert and retrieve
    let version = cache.insert(update.clone());
    assert_eq!(version, 1);
    
    let retrieved = cache.get(&id).unwrap();
    assert_eq!(retrieved.html, "<div>Test</div>");
    
    // Test stats
    let stats = cache.stats();
    assert_eq!(stats.total_entries, 1);
}

#[tokio::test]
async fn test_hot_reload_server_startup() {
    let temp_dir = TempDir::new().unwrap();
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    
    let server = HotReloadServer::new(addr, vec![temp_dir.path().to_path_buf()]);
    
    // Start server in background
    let handle = tokio::spawn(async move {
        // The server will bind to a random port and immediately shut down
        // This is just to test that it can start without errors
        tokio::select! {
            _ = server.start() => {},
            _ = sleep(Duration::from_millis(100)) => {},
        }
    });
    
    // Give it time to start
    sleep(Duration::from_millis(50)).await;
    
    // Clean up
    handle.abort();
}

#[tokio::test]
async fn test_protocol_serialization() {
    let msg = HotReloadMessage::Connected {
        version: "1.0.0".to_string(),
    };
    
    let json = msg.to_json().unwrap();
    let deserialized = HotReloadMessage::from_json(&json).unwrap();
    
    match deserialized {
        HotReloadMessage::Connected { version } => {
            assert_eq!(version, "1.0.0");
        }
        _ => panic!("Wrong message type"),
    }
}

#[tokio::test]
async fn test_template_id_hashing() {
    let id1 = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
    let id2 = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
    let id3 = TemplateId::new(PathBuf::from("other.rs"), 10, 5);
    
    assert_eq!(id1.hash(), id2.hash());
    assert_ne!(id1.hash(), id3.hash());
}