use shipwright_liveview_hotreload::protocol::*;
use std::path::PathBuf;

#[test]
fn test_serialization_formats() {
    let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
    let message = HotReloadMessage::TemplateUpdated(TemplateUpdate {
        id: template_id.clone(),
        hash: template_id.hash(),
        content_hash: "test_hash".to_string(),
        html: "<div>Hello World</div>".to_string(),
        dynamic_parts: vec![],
    });

    // Test JSON serialization
    let json_data = message.serialize(SerializationFormat::Json).unwrap();
    let deserialized = HotReloadMessage::deserialize(&json_data, SerializationFormat::Json).unwrap();
    matches!(deserialized, HotReloadMessage::TemplateUpdated(_));

    // Test CBOR serialization
    let cbor_data = message.serialize(SerializationFormat::Cbor).unwrap();
    let deserialized = HotReloadMessage::deserialize(&cbor_data, SerializationFormat::Cbor).unwrap();
    matches!(deserialized, HotReloadMessage::TemplateUpdated(_));

    // Test MessagePack serialization
    let msgpack_data = message.serialize(SerializationFormat::MessagePack).unwrap();
    let deserialized = HotReloadMessage::deserialize(&msgpack_data, SerializationFormat::MessagePack).unwrap();
    matches!(deserialized, HotReloadMessage::TemplateUpdated(_));
}

#[test]
fn test_compression() {
    let data = b"Hello World! This is a test string that should compress well.";
    
    // Test Gzip compression
    let compressed = HotReloadMessage::compress(data, CompressionAlgorithm::Gzip).unwrap();
    let decompressed = HotReloadMessage::decompress(&compressed, CompressionAlgorithm::Gzip).unwrap();
    assert_eq!(data, decompressed.as_slice());

    // Test Brotli compression
    let compressed = HotReloadMessage::compress(data, CompressionAlgorithm::Brotli).unwrap();
    let decompressed = HotReloadMessage::decompress(&compressed, CompressionAlgorithm::Brotli).unwrap();
    assert_eq!(data, decompressed.as_slice());
}

#[test]
fn test_delta_computation() {
    let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
    
    let old_template = TemplateUpdate {
        id: template_id.clone(),
        hash: template_id.hash(),
        content_hash: "old_hash".to_string(),
        html: "<div>Hello World</div>".to_string(),
        dynamic_parts: vec![],
    };

    let new_template = TemplateUpdate {
        id: template_id.clone(),
        hash: template_id.hash(),
        content_hash: "new_hash".to_string(),
        html: "<div>Hello Universe</div>".to_string(),
        dynamic_parts: vec![],
    };

    let delta = TemplateDelta::compute_delta(&old_template, &new_template).unwrap();
    assert_eq!(delta.previous_content_hash, "old_hash");
    assert_eq!(delta.new_content_hash, "new_hash");
    assert!(!delta.operations.is_empty());

    // Test applying delta
    let mut template = old_template.clone();
    delta.apply_to_template(&mut template).unwrap();
    assert_eq!(template.html, "<div>Hello Universe</div>");
    assert_eq!(template.content_hash, "new_hash");
}

#[test]
fn test_batch_operations() {
    let mut batch = BatchOperation::new();
    assert!(batch.is_empty());

    let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
    let template_update = TemplateUpdate {
        id: template_id.clone(),
        hash: template_id.hash(),
        content_hash: "test_hash".to_string(),
        html: "<div>Test</div>".to_string(),
        dynamic_parts: vec![],
    };

    batch.add_full_update(template_update);
    assert_eq!(batch.len(), 1);
    assert!(!batch.is_empty());
}

#[test]
fn test_capability_negotiation() {
    let client_caps = ProtocolCapabilities::enhanced();
    let server_caps = ProtocolCapabilities::default();

    let negotiated = HotReloadMessage::negotiate_capabilities(&client_caps, &server_caps);
    
    // Should fall back to JSON since server only supports JSON
    assert_eq!(negotiated.preferred_serialization, SerializationFormat::Json);
    assert_eq!(negotiated.preferred_compression, CompressionAlgorithm::None);
    assert!(!negotiated.supports_delta_updates); // Server doesn't support it
    assert!(!negotiated.supports_batch_operations); // Server doesn't support it
}

#[test]
fn test_enhanced_capabilities() {
    let caps = ProtocolCapabilities::enhanced();
    
    assert!(caps.serialization_formats.contains(&SerializationFormat::Json));
    assert!(caps.serialization_formats.contains(&SerializationFormat::Cbor));
    assert!(caps.serialization_formats.contains(&SerializationFormat::MessagePack));
    
    assert!(caps.compression_algorithms.contains(&CompressionAlgorithm::None));
    assert!(caps.compression_algorithms.contains(&CompressionAlgorithm::Gzip));
    assert!(caps.compression_algorithms.contains(&CompressionAlgorithm::Brotli));
    
    assert!(caps.supports_delta_updates);
    assert!(caps.supports_batch_operations);
    assert_eq!(caps.preferred_serialization, SerializationFormat::Cbor);
    assert_eq!(caps.preferred_compression, CompressionAlgorithm::Gzip);
}

#[test]
fn test_message_size_validation() {
    let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
    let large_html = "x".repeat(1000); // 1KB of content
    
    let message = HotReloadMessage::TemplateUpdated(TemplateUpdate {
        id: template_id.clone(),
        hash: template_id.hash(),
        content_hash: "test_hash".to_string(),
        html: large_html,
        dynamic_parts: vec![],
    });

    // Should pass validation with reasonable size limit
    assert!(message.validate_size(10_000, SerializationFormat::Json).is_ok());
    
    // Should fail validation with very small size limit
    assert!(message.validate_size(100, SerializationFormat::Json).is_err());
}

#[test]
fn test_comprehensive_protocol_flow() {
    // Test a complete protocol flow with negotiation, delta updates, and batch operations
    let client_caps = ProtocolCapabilities::enhanced();
    let server_caps = ProtocolCapabilities::enhanced();
    
    // Negotiate capabilities
    let negotiated = HotReloadMessage::negotiate_capabilities(&client_caps, &server_caps);
    assert_eq!(negotiated.preferred_serialization, SerializationFormat::Cbor);
    assert_eq!(negotiated.preferred_compression, CompressionAlgorithm::Gzip);
    
    // Create initial template
    let template_id = TemplateId::new(PathBuf::from("app.rs"), 15, 10);
    let initial_template = TemplateUpdate {
        id: template_id.clone(),
        hash: template_id.hash(),
        content_hash: TemplateUpdate::compute_content_hash("<div>Initial</div>", &[]),
        html: "<div>Initial</div>".to_string(),
        dynamic_parts: vec![],
    };
    
    // Create updated template
    let updated_template = TemplateUpdate {
        id: template_id.clone(),
        hash: template_id.hash(),
        content_hash: TemplateUpdate::compute_content_hash("<div>Updated</div>", &[]),
        html: "<div>Updated</div>".to_string(),
        dynamic_parts: vec![],
    };
    
    // Compute delta
    let delta = TemplateDelta::compute_delta(&initial_template, &updated_template).unwrap();
    
    // Create batch operation
    let mut batch = BatchOperation::new();
    batch.add_delta_update(delta);
    
    // Serialize with compression
    let batch_message = HotReloadMessage::BatchOperation(batch);
    let serialized = batch_message.serialize_compressed(
        negotiated.preferred_serialization,
        negotiated.preferred_compression,
    ).unwrap();
    
    // Deserialize
    let deserialized = HotReloadMessage::deserialize_compressed(
        &serialized,
        negotiated.preferred_serialization,
        negotiated.preferred_compression,
    ).unwrap();
    
    // Verify it round-tripped correctly
    matches!(deserialized, HotReloadMessage::BatchOperation(_));
}