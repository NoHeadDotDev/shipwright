// Standalone protocol tests that don't depend on other modules
#[test]
fn test_basic_serialization() {
    use std::path::PathBuf;
    
    // Manual struct definitions for testing
    #[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct TemplateId {
        pub file: PathBuf,
        pub line: u32,
        pub column: u32,
    }
    
    impl TemplateId {
        pub fn new(file: PathBuf, line: u32, column: u32) -> Self {
            Self { file, line, column }
        }
        
        pub fn hash(&self) -> String {
            use blake3::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(self.file.to_string_lossy().as_bytes());
            hasher.update(&self.line.to_le_bytes());
            hasher.update(&self.column.to_le_bytes());
            hasher.finalize().to_hex().to_string()
        }
    }
    
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct DynamicPart {
        pub index: usize,
        pub kind: DynamicKind,
    }
    
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum DynamicKind {
        Expression,
        EventHandler { event: String },
        Conditional,
        Loop,
    }
    
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct TemplateUpdate {
        pub id: TemplateId,
        pub hash: String,
        pub content_hash: String,
        pub html: String,
        pub dynamic_parts: Vec<DynamicPart>,
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum SerializationFormat {
        Json,
        Cbor,
        MessagePack,
    }
    
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum HotReloadMessage {
        TemplateUpdated(TemplateUpdate),
    }
    
    impl HotReloadMessage {
        pub fn serialize(&self, format: SerializationFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            match format {
                SerializationFormat::Json => {
                    Ok(serde_json::to_vec(self)?)
                }
                SerializationFormat::Cbor => {
                    let mut buffer = Vec::new();
                    ciborium::ser::into_writer(self, &mut buffer)?;
                    Ok(buffer)
                }
                SerializationFormat::MessagePack => {
                    Ok(rmp_serde::to_vec(self)?)
                }
            }
        }
        
        pub fn deserialize(data: &[u8], format: SerializationFormat) -> Result<Self, Box<dyn std::error::Error>> {
            match format {
                SerializationFormat::Json => {
                    Ok(serde_json::from_slice(data)?)
                }
                SerializationFormat::Cbor => {
                    Ok(ciborium::de::from_reader(data)?)
                }
                SerializationFormat::MessagePack => {
                    Ok(rmp_serde::from_slice(data)?)
                }
            }
        }
    }
    
    // Test the implementation
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
    
    // Verify binary formats are more compact than JSON for this message
    println!("JSON size: {} bytes", json_data.len());
    println!("CBOR size: {} bytes", cbor_data.len());
    println!("MessagePack size: {} bytes", msgpack_data.len());
    
    // Binary formats should be smaller or equal
    assert!(cbor_data.len() <= json_data.len());
    assert!(msgpack_data.len() <= json_data.len());
}

#[test]
fn test_compression_algorithms() {
    use std::io::{Read, Write};
    
    let test_data = "Hello World! ".repeat(100); // Repeatable data that compresses well
    let data = test_data.as_bytes();
    
    // Test Gzip compression
    {
        use flate2::write::GzEncoder;
        use flate2::read::GzDecoder;
        use flate2::Compression;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        let compressed = encoder.finish().unwrap();
        
        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        
        assert_eq!(data, decompressed.as_slice());
        assert!(compressed.len() < data.len()); // Should be compressed
        
        println!("Original size: {} bytes", data.len());
        println!("Gzip compressed size: {} bytes", compressed.len());
        println!("Compression ratio: {:.2}%", (compressed.len() as f64 / data.len() as f64) * 100.0);
    }
    
    // Test Brotli compression
    {
        use brotli::enc::BrotliEncoderParams;
        
        let params = BrotliEncoderParams::default();
        let mut compressed = Vec::new();
        let mut writer = brotli::CompressorWriter::with_params(&mut compressed, 4096, &params);
        writer.write_all(data).unwrap();
        writer.flush().unwrap();
        drop(writer);
        
        let mut decompressed = Vec::new();
        let mut reader = brotli::Decompressor::new(compressed.as_slice(), 4096);
        reader.read_to_end(&mut decompressed).unwrap();
        
        assert_eq!(data, decompressed.as_slice());
        assert!(compressed.len() < data.len()); // Should be compressed
        
        println!("Brotli compressed size: {} bytes", compressed.len());
        println!("Compression ratio: {:.2}%", (compressed.len() as f64 / data.len() as f64) * 100.0);
    }
}