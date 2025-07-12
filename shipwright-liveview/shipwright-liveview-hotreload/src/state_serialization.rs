//! Component state serialization and restoration system
//! 
//! This module provides comprehensive state management for LiveView components
//! during hot reload operations, enabling seamless updates while preserving
//! application state.

use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::protocol::TemplateId;
use crate::runtime::HotReloadError;

/// State serialization and restoration manager
#[derive(Debug)]
pub struct StateManager {
    /// Serialized component states by instance ID
    component_states: Arc<RwLock<HashMap<String, SerializedComponentState>>>,
    /// State serializers by type
    serializers: Arc<RwLock<HashMap<TypeId, Box<dyn StateSerializer + Send + Sync>>>>,
    /// Configuration
    config: StateManagerConfig,
}

/// Configuration for state management
#[derive(Debug, Clone)]
pub struct StateManagerConfig {
    /// Maximum number of states to keep in memory
    pub max_states: usize,
    /// Whether to compress serialized state
    pub compress_state: bool,
    /// Whether to validate state integrity
    pub validate_integrity: bool,
    /// Timeout for state operations
    pub operation_timeout_ms: u64,
    /// Whether to enable debug logging
    pub debug_mode: bool,
}

impl Default for StateManagerConfig {
    fn default() -> Self {
        Self {
            max_states: 1000,
            compress_state: true,
            validate_integrity: true,
            operation_timeout_ms: 5000,
            debug_mode: cfg!(debug_assertions),
        }
    }
}

/// Serialized component state container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedComponentState {
    /// Component instance ID
    pub instance_id: String,
    /// Template ID associated with this state
    pub template_id: TemplateId,
    /// Component type name
    pub component_type: String,
    /// Serialized state data
    pub state_data: Vec<u8>,
    /// State metadata
    pub metadata: StateMetadata,
    /// When this state was captured
    pub captured_at: std::time::SystemTime,
    /// State format version for compatibility
    pub format_version: u32,
}

/// Metadata about serialized state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    /// Size of original state in bytes
    pub original_size: usize,
    /// Size after compression (if enabled)
    pub compressed_size: Option<usize>,
    /// Checksum for integrity validation
    pub checksum: Option<String>,
    /// Custom metadata from the component
    pub custom: HashMap<String, serde_json::Value>,
}

/// Trait for component state serialization
pub trait StateSerializer {
    /// Serialize component state to bytes
    fn serialize(&self, state: &dyn Any) -> Result<Vec<u8>, StateSerializationError>;
    
    /// Deserialize component state from bytes
    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn Any + Send>, StateSerializationError>;
    
    /// Get the type ID this serializer handles
    fn type_id(&self) -> TypeId;
    
    /// Get format version for compatibility
    fn format_version(&self) -> u32;
    
    /// Validate state integrity (optional)
    fn validate(&self, state: &dyn Any) -> bool {
        true // Default implementation
    }
}

/// Errors that can occur during state serialization
#[derive(Debug, Clone, thiserror::Error)]
pub enum StateSerializationError {
    #[error("Serialization failed: {reason}")]
    SerializationFailed { reason: String },
    
    #[error("Deserialization failed: {reason}")]
    DeserializationFailed { reason: String },
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Compression failed: {reason}")]
    CompressionFailed { reason: String },
    
    #[error("Integrity validation failed")]
    IntegrityValidationFailed,
    
    #[error("Format version incompatible: {version}")]
    IncompatibleVersion { version: u32 },
    
    #[error("Timeout during operation")]
    Timeout,
}

/// Built-in serializer for JSON-serializable types
#[derive(Debug)]
pub struct JsonStateSerializer<T> {
    type_id: TypeId,
    phantom: std::marker::PhantomData<T>,
}

impl<T> JsonStateSerializer<T>
where
    T: Serialize + for<'de> Deserialize<'de> + 'static,
{
    pub fn new() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> StateSerializer for JsonStateSerializer<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    fn serialize(&self, state: &dyn Any) -> Result<Vec<u8>, StateSerializationError> {
        let typed_state = state.downcast_ref::<T>()
            .ok_or_else(|| StateSerializationError::TypeMismatch {
                expected: std::any::type_name::<T>().to_string(),
                actual: "unknown".to_string(),
            })?;
        
        serde_json::to_vec(typed_state)
            .map_err(|e| StateSerializationError::SerializationFailed {
                reason: e.to_string(),
            })
    }
    
    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn Any + Send>, StateSerializationError> {
        let state: T = serde_json::from_slice(data)
            .map_err(|e| StateSerializationError::DeserializationFailed {
                reason: e.to_string(),
            })?;
        
        Ok(Box::new(state))
    }
    
    fn type_id(&self) -> TypeId {
        self.type_id
    }
    
    fn format_version(&self) -> u32 {
        1
    }
}

/// Built-in serializer for binary-serializable types (MessagePack)
#[derive(Debug)]
pub struct BinaryStateSerializer<T> {
    type_id: TypeId,
    phantom: std::marker::PhantomData<T>,
}

impl<T> BinaryStateSerializer<T>
where
    T: Serialize + for<'de> Deserialize<'de> + 'static,
{
    pub fn new() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> StateSerializer for BinaryStateSerializer<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    fn serialize(&self, state: &dyn Any) -> Result<Vec<u8>, StateSerializationError> {
        let typed_state = state.downcast_ref::<T>()
            .ok_or_else(|| StateSerializationError::TypeMismatch {
                expected: std::any::type_name::<T>().to_string(),
                actual: "unknown".to_string(),
            })?;
        
        rmp_serde::to_vec(typed_state)
            .map_err(|e| StateSerializationError::SerializationFailed {
                reason: e.to_string(),
            })
    }
    
    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn Any + Send>, StateSerializationError> {
        let state: T = rmp_serde::from_slice(data)
            .map_err(|e| StateSerializationError::DeserializationFailed {
                reason: e.to_string(),
            })?;
        
        Ok(Box::new(state))
    }
    
    fn type_id(&self) -> TypeId {
        self.type_id
    }
    
    fn format_version(&self) -> u32 {
        1
    }
}

impl StateManager {
    /// Create a new state manager
    pub fn new(config: StateManagerConfig) -> Self {
        Self {
            component_states: Arc::new(RwLock::new(HashMap::new())),
            serializers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Register a state serializer for a specific type
    pub async fn register_serializer<T>(&self, serializer: Box<dyn StateSerializer + Send + Sync>)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        debug!("Registering state serializer for type: {:?}", std::any::type_name::<T>());
        
        let mut serializers = self.serializers.write().await;
        serializers.insert(type_id, serializer);
    }
    
    /// Serialize and store component state
    pub async fn preserve_state<T>(
        &self,
        instance_id: String,
        template_id: TemplateId,
        component_type: String,
        state: &T,
    ) -> Result<(), StateSerializationError>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        
        // Find appropriate serializer
        let serializers = self.serializers.read().await;
        let serializer = serializers.get(&type_id)
            .ok_or_else(|| StateSerializationError::SerializationFailed {
                reason: format!("No serializer registered for type: {}", std::any::type_name::<T>()),
            })?;
        
        // Serialize the state
        let state_data = serializer.serialize(state as &dyn Any)?;
        let original_size = state_data.len();
        
        // Compress if enabled
        let (final_data, compressed_size) = if self.config.compress_state {
            let compressed = self.compress_data(&state_data)?;
            let compressed_size = compressed.len();
            (compressed, Some(compressed_size))
        } else {
            (state_data, None)
        };
        
        // Calculate checksum if integrity validation is enabled
        let checksum = if self.config.validate_integrity {
            Some(self.calculate_checksum(&final_data))
        } else {
            None
        };
        
        // Create serialized state
        let serialized_state = SerializedComponentState {
            instance_id: instance_id.clone(),
            template_id,
            component_type,
            state_data: final_data,
            metadata: StateMetadata {
                original_size,
                compressed_size,
                checksum,
                custom: HashMap::new(),
            },
            captured_at: std::time::SystemTime::now(),
            format_version: serializer.format_version(),
        };
        
        drop(serializers);
        
        // Store the state
        {
            let mut states = self.component_states.write().await;
            states.insert(instance_id.clone(), serialized_state);
            
            // Clean up old states if necessary
            self.cleanup_old_states(&mut states).await;
        }
        
        info!("Preserved state for component: {} ({})", instance_id, component_type);
        
        if self.config.debug_mode {
            debug!("State details - Original: {} bytes, Final: {} bytes", 
                   original_size, compressed_size.unwrap_or(original_size));
        }
        
        Ok(())
    }
    
    /// Restore component state
    pub async fn restore_state<T>(
        &self,
        instance_id: &str,
    ) -> Result<Option<T>, StateSerializationError>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        
        // Retrieve serialized state
        let serialized_state = {
            let states = self.component_states.read().await;
            states.get(instance_id).cloned()
        };
        
        let serialized_state = match serialized_state {
            Some(state) => state,
            None => {
                debug!("No preserved state found for component: {}", instance_id);
                return Ok(None);
            }
        };
        
        // Find appropriate serializer
        let serializers = self.serializers.read().await;
        let serializer = serializers.get(&type_id)
            .ok_or_else(|| StateSerializationError::DeserializationFailed {
                reason: format!("No serializer registered for type: {}", std::any::type_name::<T>()),
            })?;
        
        // Check format version compatibility
        if serializer.format_version() != serialized_state.format_version {
            return Err(StateSerializationError::IncompatibleVersion {
                version: serialized_state.format_version,
            });
        }
        
        // Validate integrity if enabled
        if self.config.validate_integrity {
            if let Some(ref expected_checksum) = serialized_state.metadata.checksum {
                let actual_checksum = self.calculate_checksum(&serialized_state.state_data);
                if &actual_checksum != expected_checksum {
                    return Err(StateSerializationError::IntegrityValidationFailed);
                }
            }
        }
        
        // Decompress if necessary
        let state_data = if serialized_state.metadata.compressed_size.is_some() {
            self.decompress_data(&serialized_state.state_data)?
        } else {
            serialized_state.state_data
        };
        
        // Deserialize the state
        let restored_state = serializer.deserialize(&state_data)?;
        
        // Downcast to the requested type
        let typed_state = restored_state.downcast::<T>()
            .map_err(|_| StateSerializationError::TypeMismatch {
                expected: std::any::type_name::<T>().to_string(),
                actual: "unknown".to_string(),
            })?;
        
        info!("Restored state for component: {}", instance_id);
        
        Ok(Some(*typed_state))
    }
    
    /// Remove preserved state for a component
    pub async fn remove_state(&self, instance_id: &str) -> bool {
        let mut states = self.component_states.write().await;
        states.remove(instance_id).is_some()
    }
    
    /// Get all preserved state instances
    pub async fn get_all_instances(&self) -> Vec<String> {
        let states = self.component_states.read().await;
        states.keys().cloned().collect()
    }
    
    /// Get metadata about preserved state
    pub async fn get_state_metadata(&self, instance_id: &str) -> Option<StateMetadata> {
        let states = self.component_states.read().await;
        states.get(instance_id).map(|state| state.metadata.clone())
    }
    
    /// Clear all preserved states
    pub async fn clear_all_states(&self) {
        let mut states = self.component_states.write().await;
        let count = states.len();
        states.clear();
        info!("Cleared {} preserved component states", count);
    }
    
    /// Get statistics about state management
    pub async fn get_statistics(&self) -> StateStatistics {
        let states = self.component_states.read().await;
        let serializers = self.serializers.read().await;
        
        let mut total_size = 0;
        let mut compressed_savings = 0;
        let mut by_type: HashMap<String, usize> = HashMap::new();
        
        for state in states.values() {
            total_size += state.state_data.len();
            
            if let Some(compressed_size) = state.metadata.compressed_size {
                compressed_savings += state.metadata.original_size - compressed_size;
            }
            
            *by_type.entry(state.component_type.clone()).or_insert(0) += 1;
        }
        
        StateStatistics {
            total_states: states.len(),
            total_size_bytes: total_size,
            compressed_savings_bytes: compressed_savings,
            registered_serializers: serializers.len(),
            states_by_type: by_type,
        }
    }
    
    /// Compress data using gzip
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, StateSerializationError> {
        use flate2::{Compression, write::GzEncoder};
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)
            .map_err(|e| StateSerializationError::CompressionFailed {
                reason: e.to_string(),
            })?;
        
        encoder.finish()
            .map_err(|e| StateSerializationError::CompressionFailed {
                reason: e.to_string(),
            })
    }
    
    /// Decompress data using gzip
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, StateSerializationError> {
        use flate2::read::GzDecoder;
        use std::io::Read;
        
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| StateSerializationError::CompressionFailed {
                reason: e.to_string(),
            })?;
        
        Ok(decompressed)
    }
    
    /// Calculate checksum for integrity validation
    fn calculate_checksum(&self, data: &[u8]) -> String {
        use blake3::Hasher;
        
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize().to_hex().to_string()
    }
    
    /// Clean up old states to maintain memory limits
    async fn cleanup_old_states(&self, states: &mut HashMap<String, SerializedComponentState>) {
        if states.len() <= self.config.max_states {
            return;
        }
        
        // Sort by capture time and remove oldest
        let mut state_entries: Vec<_> = states.iter().collect();
        state_entries.sort_by_key(|(_, state)| state.captured_at);
        
        let to_remove = states.len() - self.config.max_states;
        for (instance_id, _) in state_entries.iter().take(to_remove) {
            states.remove(*instance_id);
        }
        
        debug!("Cleaned up {} old component states", to_remove);
    }
}

/// Statistics about state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateStatistics {
    /// Total number of preserved states
    pub total_states: usize,
    /// Total size of all preserved states in bytes
    pub total_size_bytes: usize,
    /// Bytes saved through compression
    pub compressed_savings_bytes: usize,
    /// Number of registered serializers
    pub registered_serializers: usize,
    /// States by component type
    pub states_by_type: HashMap<String, usize>,
}

/// Utility macros for easy state management
#[macro_export]
macro_rules! preserve_component_state {
    ($state_manager:expr, $instance_id:expr, $template_id:expr, $component_type:expr, $state:expr) => {{
        $state_manager.preserve_state(
            $instance_id.to_string(),
            $template_id,
            $component_type.to_string(),
            $state
        ).await
    }};
}

#[macro_export]
macro_rules! restore_component_state {
    ($state_manager:expr, $instance_id:expr, $state_type:ty) => {{
        $state_manager.restore_state::<$state_type>($instance_id).await
    }};
}

/// Built-in serializers for common LiveView types
pub mod builtin_serializers {
    use super::*;
    
    /// Register built-in serializers for common types
    pub async fn register_builtin_serializers(state_manager: &StateManager) {
        // Register JSON serializers for common types
        state_manager.register_serializer::<String>(
            Box::new(JsonStateSerializer::<String>::new())
        ).await;
        
        state_manager.register_serializer::<i32>(
            Box::new(JsonStateSerializer::<i32>::new())
        ).await;
        
        state_manager.register_serializer::<i64>(
            Box::new(JsonStateSerializer::<i64>::new())
        ).await;
        
        state_manager.register_serializer::<f64>(
            Box::new(JsonStateSerializer::<f64>::new())
        ).await;
        
        state_manager.register_serializer::<bool>(
            Box::new(JsonStateSerializer::<bool>::new())
        ).await;
        
        state_manager.register_serializer::<Vec<String>>(
            Box::new(JsonStateSerializer::<Vec<String>>::new())
        ).await;
        
        state_manager.register_serializer::<HashMap<String, String>>(
            Box::new(JsonStateSerializer::<HashMap<String, String>>::new())
        ).await;
        
        // Register binary serializers for more complex types
        state_manager.register_serializer::<serde_json::Value>(
            Box::new(BinaryStateSerializer::<serde_json::Value>::new())
        ).await;
        
        info!("Registered built-in state serializers");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestComponentState {
        count: i32,
        message: String,
        values: Vec<String>,
    }
    
    #[tokio::test]
    async fn test_state_manager_creation() {
        let config = StateManagerConfig::default();
        let manager = StateManager::new(config);
        
        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_states, 0);
    }
    
    #[tokio::test]
    async fn test_state_serialization_and_restoration() {
        let config = StateManagerConfig::default();
        let manager = StateManager::new(config);
        
        // Register serializer
        manager.register_serializer::<TestComponentState>(
            Box::new(JsonStateSerializer::<TestComponentState>::new())
        ).await;
        
        // Create test state
        let original_state = TestComponentState {
            count: 42,
            message: "Hello, world!".to_string(),
            values: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        
        // Preserve state
        let result = manager.preserve_state(
            "test-instance".to_string(),
            template_id,
            "TestComponent".to_string(),
            &original_state,
        ).await;
        assert!(result.is_ok());
        
        // Restore state
        let restored_state: Option<TestComponentState> = manager.restore_state("test-instance").await.unwrap();
        assert!(restored_state.is_some());
        
        let restored_state = restored_state.unwrap();
        assert_eq!(restored_state, original_state);
        
        // Check statistics
        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_states, 1);
        assert_eq!(stats.states_by_type.get("TestComponent"), Some(&1));
    }
    
    #[tokio::test]
    async fn test_state_removal() {
        let config = StateManagerConfig::default();
        let manager = StateManager::new(config);
        
        manager.register_serializer::<String>(
            Box::new(JsonStateSerializer::<String>::new())
        ).await;
        
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        let test_state = "test state".to_string();
        
        // Preserve state
        manager.preserve_state(
            "test-instance".to_string(),
            template_id,
            "TestComponent".to_string(),
            &test_state,
        ).await.unwrap();
        
        // Verify it exists
        let restored: Option<String> = manager.restore_state("test-instance").await.unwrap();
        assert!(restored.is_some());
        
        // Remove state
        let removed = manager.remove_state("test-instance").await;
        assert!(removed);
        
        // Verify it's gone
        let restored: Option<String> = manager.restore_state("test-instance").await.unwrap();
        assert!(restored.is_none());
    }
    
    #[tokio::test]
    async fn test_compression() {
        let mut config = StateManagerConfig::default();
        config.compress_state = true;
        let manager = StateManager::new(config);
        
        manager.register_serializer::<Vec<String>>(
            Box::new(JsonStateSerializer::<Vec<String>>::new())
        ).await;
        
        // Create large state that should compress well
        let large_state: Vec<String> = (0..1000)
            .map(|i| format!("This is a repeated string number {}", i))
            .collect();
        
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        
        manager.preserve_state(
            "test-large".to_string(),
            template_id,
            "LargeComponent".to_string(),
            &large_state,
        ).await.unwrap();
        
        let metadata = manager.get_state_metadata("test-large").await.unwrap();
        assert!(metadata.compressed_size.is_some());
        assert!(metadata.compressed_size.unwrap() < metadata.original_size);
        
        // Verify we can restore it correctly
        let restored: Option<Vec<String>> = manager.restore_state("test-large").await.unwrap();
        assert!(restored.is_some());
        assert_eq!(restored.unwrap(), large_state);
    }
}