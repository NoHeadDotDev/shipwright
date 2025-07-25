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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Asset file updated (CSS, JS, etc.)
    AssetUpdated {
        /// Type of asset
        asset_type: String,
        /// Path to the asset
        path: String,
    },
    /// Full page reload requested
    FullReload {
        /// Reason for full reload
        reason: String,
    },
    /// Server status update
    ServerStatus {
        /// Server status
        status: ServerStatusType,
        /// Additional message
        message: Option<String>,
        /// Statistics
        stats: Option<ServerStats>,
    },
    /// File change detected (for debugging)
    FileChangeDetected {
        /// Path that changed
        path: String,
        /// Type of change
        change_type: String,
        /// Number of templates affected
        templates_affected: usize,
    },
    /// Error occurred
    Error {
        /// Error message
        message: String,
        /// Error code
        code: Option<String>,
        /// Suggested actions
        suggestions: Option<Vec<String>>,
    },
    /// Heartbeat to keep connection alive
    Ping,
    /// Response to ping
    Pong,
}

/// Server status types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatusType {
    /// Server starting up
    Starting,
    /// Server running normally
    Running,
    /// Server experiencing issues
    Warning,
    /// Server shutting down
    Shutdown,
}

/// Server statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    /// Number of connected clients
    pub connected_clients: usize,
    /// Number of cached templates
    pub cached_templates: usize,
    /// Number of files being watched
    pub watched_files: usize,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Number of updates sent
    pub updates_sent: u64,
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

/// Enhanced protocol types for change analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeAnalysis {
    /// Type of change detected
    pub change_type: ChangeType,
    /// Number of templates in the file
    pub template_count: usize,
    /// Whether templates were added or removed
    pub templates_added_or_removed: bool,
    /// Affected line ranges
    pub affected_ranges: Vec<LineRange>,
    /// Confidence in the analysis (0.0 to 1.0)
    pub confidence: f32,
    /// Additional metadata
    pub metadata: Option<ChangeMetadata>,
}

/// Type of change detected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// Only template content changed
    TemplateOnly,
    /// Code structure changed
    CodeStructure,
    /// Mixed changes
    Mixed,
    /// Asset files changed
    Assets,
    /// Unknown change type
    Unknown,
}

/// Line range in a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineRange {
    pub start: u32,
    pub end: u32,
}

/// Additional metadata about changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeMetadata {
    /// Types of macros found
    pub macro_types: Vec<String>,
    /// Whether this is a new file
    pub is_new_file: bool,
    /// Complexity of the change
    pub complexity: ChangeComplexity,
}

/// Complexity of a change
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeComplexity {
    /// Simple change (e.g., text only)
    Simple,
    /// Moderate change (e.g., attributes)
    Moderate,
    /// Complex change (e.g., structure)
    Complex,
}

/// Content type classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    /// Rust source code
    Rust,
    /// CSS/SCSS files
    Style,
    /// JavaScript/TypeScript
    Script,
    /// HTML templates
    Html,
    /// Other asset types
    Asset,
}