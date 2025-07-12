//! Hot reload protocol definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique identifier for a template
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateId {
    /// File path where the template is defined
    pub file: PathBuf,
    /// Line number in the file
    pub line: u32,
    /// Column number in the file
    pub column: u32,
}

impl TemplateId {
    pub fn new(file: PathBuf, line: u32, column: u32) -> Self {
        Self { file, line, column }
    }

    /// Generate a hash for the template ID
    pub fn hash(&self) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(self.file.to_string_lossy().as_bytes());
        hasher.update(&self.line.to_le_bytes());
        hasher.update(&self.column.to_le_bytes());
        hasher.finalize().to_hex().to_string()
    }
}

/// Template update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateUpdate {
    /// Template identifier
    pub id: TemplateId,
    /// Hash of the template ID for quick lookup
    pub hash: String,
    /// Hash of the template content for change detection
    pub content_hash: String,
    /// The compiled HTML template
    pub html: String,
    /// Dynamic parts of the template
    pub dynamic_parts: Vec<DynamicPart>,
}

impl TemplateUpdate {
    /// Compute content hash based on HTML and dynamic parts
    pub fn compute_content_hash(html: &str, dynamic_parts: &[DynamicPart]) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(html.as_bytes());
        
        // Include dynamic parts in the hash for complete content comparison
        for part in dynamic_parts {
            hasher.update(&part.index.to_le_bytes());
            match &part.kind {
                DynamicKind::Expression => { hasher.update(b"expression"); }
                DynamicKind::EventHandler { event } => {
                    hasher.update(b"event:");
                    hasher.update(event.as_bytes());
                }
                DynamicKind::Conditional => { hasher.update(b"conditional"); }
                DynamicKind::Loop => { hasher.update(b"loop"); }
            }
        }
        
        hasher.finalize().to_hex().to_string()
    }
}

/// Dynamic part of a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPart {
    /// Index in the template where this dynamic part appears
    pub index: usize,
    /// Type of dynamic content
    pub kind: DynamicKind,
}

/// Type of dynamic content in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DynamicKind {
    /// Simple expression interpolation
    Expression,
    /// Event handler
    EventHandler { event: String },
    /// Conditional rendering
    Conditional,
    /// Loop rendering
    Loop,
}

/// Messages sent over the WebSocket connection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HotReloadMessage {
    /// Initial connection established
    Connected {
        /// Version of the hot reload protocol
        version: String,
    },
    /// Template has been updated
    TemplateUpdated(TemplateUpdate),
    /// Multiple templates updated at once
    BatchUpdate {
        /// List of template updates
        updates: Vec<TemplateUpdate>,
    },
    /// Request from client to reload a specific template
    ReloadRequest {
        /// Template ID to reload
        template_id: TemplateId,
    },
    /// Error occurred
    Error {
        /// Error message
        message: String,
    },
    /// Heartbeat to keep connection alive
    Ping,
    /// Response to ping
    Pong,
}

impl HotReloadMessage {
    /// Serialize message to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize message from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}