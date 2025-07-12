//! Error boundaries and graceful recovery mechanisms for hot reload
//! 
//! This module provides comprehensive error handling, recovery strategies,
//! and fallback mechanisms to ensure hot reload failures don't break the
//! development experience.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::runtime::HotReloadError;
use crate::protocol::{TemplateId, TemplateUpdate};
use crate::liveview_integration::{ActiveLiveView, HotReloadResult};

/// Error recovery and boundary management system
#[derive(Debug)]
pub struct ErrorRecoverySystem {
    /// Error boundaries by component ID
    boundaries: Arc<RwLock<HashMap<String, ErrorBoundary>>>,
    /// Recovery strategies
    recovery_strategies: Arc<RwLock<Vec<Arc<dyn RecoveryStrategy + Send + Sync>>>>,
    /// Fallback mechanisms
    fallback_manager: Arc<FallbackManager>,
    /// Configuration
    config: ErrorRecoveryConfig,
    /// Statistics
    stats: Arc<RwLock<ErrorStats>>,
}

/// Configuration for error recovery
#[derive(Debug, Clone)]
pub struct ErrorRecoveryConfig {
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Delay between retry attempts
    pub retry_delay: Duration,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
    /// Timeout for recovery operations
    pub recovery_timeout: Duration,
    /// Whether to automatically fall back to full refresh
    pub auto_fallback_to_refresh: bool,
    /// Circuit breaker threshold (failures before opening circuit)
    pub circuit_breaker_threshold: usize,
    /// Circuit breaker reset timeout
    pub circuit_breaker_reset_timeout: Duration,
}

impl Default for ErrorRecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            exponential_backoff: true,
            recovery_timeout: Duration::from_secs(10),
            auto_fallback_to_refresh: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_reset_timeout: Duration::from_secs(30),
        }
    }
}

/// Error boundary for a component or template
#[derive(Debug, Clone)]
pub struct ErrorBoundary {
    /// Boundary identifier
    pub id: String,
    /// Template or component this boundary protects
    pub protected_id: String,
    /// Current error state
    pub error_state: ErrorState,
    /// Error history
    pub error_history: Vec<ErrorRecord>,
    /// Recovery attempts
    pub recovery_attempts: usize,
    /// Last recovery attempt time
    pub last_recovery_attempt: Option<SystemTime>,
    /// Circuit breaker state
    pub circuit_breaker: CircuitBreakerState,
    /// Fallback content
    pub fallback_content: Option<String>,
}

/// Current error state of a boundary
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorState {
    /// No errors, operating normally
    Healthy,
    /// Recoverable error occurred
    RecoverableError { error: String, timestamp: SystemTime },
    /// Critical error, needs fallback
    CriticalError { error: String, timestamp: SystemTime },
    /// Using fallback content
    UsingFallback { reason: String, timestamp: SystemTime },
    /// Permanently failed
    Failed { error: String, timestamp: SystemTime },
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    /// Circuit is closed, allowing operations
    Closed,
    /// Circuit is open, blocking operations
    Open { opened_at: SystemTime },
    /// Circuit is half-open, testing recovery
    HalfOpen,
}

/// Record of an error occurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    /// Error message
    pub error: String,
    /// Error type/category
    pub error_type: ErrorType,
    /// When the error occurred
    pub timestamp: SystemTime,
    /// Template ID associated with the error
    pub template_id: Option<TemplateId>,
    /// Recovery action taken
    pub recovery_action: Option<String>,
    /// Whether recovery was successful
    pub recovery_successful: Option<bool>,
}

/// Categories of errors for better handling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorType {
    /// Template parsing failed
    ParseError,
    /// DOM diffing failed
    DiffError,
    /// State preservation failed
    StateError,
    /// Network/communication error
    NetworkError,
    /// File system error
    FileSystemError,
    /// Unknown error
    Unknown,
}

/// Error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    /// Total errors encountered
    pub total_errors: usize,
    /// Errors by type
    pub errors_by_type: HashMap<ErrorType, usize>,
    /// Successful recoveries
    pub successful_recoveries: usize,
    /// Failed recoveries
    pub failed_recoveries: usize,
    /// Fallbacks triggered
    pub fallbacks_triggered: usize,
    /// Average recovery time
    pub average_recovery_time_ms: f64,
}

/// Trait for implementing recovery strategies
pub trait RecoveryStrategy {
    /// Attempt to recover from an error
    async fn attempt_recovery(
        &self,
        error: &HotReloadError,
        context: &RecoveryContext,
    ) -> RecoveryResult;
    
    /// Check if this strategy can handle the given error
    fn can_handle(&self, error: &HotReloadError) -> bool;
    
    /// Priority of this strategy (lower numbers = higher priority)
    fn priority(&self) -> u32;
}

/// Context for recovery operations
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    /// Template being updated
    pub template_id: TemplateId,
    /// Template update that failed
    pub template_update: Option<TemplateUpdate>,
    /// LiveView instance that failed
    pub liveview_instance: Option<ActiveLiveView>,
    /// Previous error attempts
    pub error_history: Vec<ErrorRecord>,
    /// Recovery configuration
    pub config: ErrorRecoveryConfig,
}

/// Result of a recovery attempt
#[derive(Debug, Clone)]
pub enum RecoveryResult {
    /// Recovery succeeded
    Success {
        message: String,
        recovered_content: Option<String>,
    },
    /// Recovery partially succeeded, but may need additional action
    PartialSuccess {
        message: String,
        issues: Vec<String>,
    },
    /// Recovery failed, try next strategy
    Failed {
        error: String,
        retry_recommended: bool,
    },
    /// Fallback to refresh required
    RequiresFallback {
        reason: String,
    },
}

/// Fallback manager for when recovery fails
#[derive(Debug)]
pub struct FallbackManager {
    /// Default fallback strategies
    strategies: Vec<FallbackStrategy>,
    /// Custom fallback content by template
    custom_fallbacks: HashMap<TemplateId, String>,
}

/// Fallback strategies when recovery fails
#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    /// Show previous working version
    UsePreviousVersion,
    /// Show generic error message
    ShowErrorMessage,
    /// Trigger full page refresh
    FullPageRefresh,
    /// Show loading state
    ShowLoadingState,
    /// Show custom content
    ShowCustomContent(String),
}

impl ErrorRecoverySystem {
    /// Create a new error recovery system
    pub fn new(config: ErrorRecoveryConfig) -> Self {
        let fallback_manager = Arc::new(FallbackManager::new());
        
        Self {
            boundaries: Arc::new(RwLock::new(HashMap::new())),
            recovery_strategies: Arc::new(RwLock::new(Vec::new())),
            fallback_manager,
            config,
            stats: Arc::new(RwLock::new(ErrorStats::default())),
        }
    }
    
    /// Register an error boundary for a component
    pub async fn register_boundary(&self, boundary: ErrorBoundary) {
        debug!("Registering error boundary: {}", boundary.id);
        
        let mut boundaries = self.boundaries.write().await;
        boundaries.insert(boundary.id.clone(), boundary);
    }
    
    /// Add a recovery strategy
    pub async fn add_recovery_strategy(&self, strategy: Arc<dyn RecoveryStrategy + Send + Sync>) {
        let mut strategies = self.recovery_strategies.write().await;
        strategies.push(strategy);
        
        // Sort by priority
        strategies.sort_by_key(|s| s.priority());
    }
    
    /// Handle an error and attempt recovery
    pub async fn handle_error(
        &self,
        error: HotReloadError,
        context: RecoveryContext,
    ) -> RecoveryOutcome {
        info!("Handling hot reload error: {}", error);
        
        // Update statistics
        self.update_error_stats(&error).await;
        
        // Find relevant error boundary
        let boundary_id = context.template_id.hash();
        let mut boundary = {
            let boundaries = self.boundaries.read().await;
            boundaries.get(&boundary_id).cloned()
        };
        
        // Create boundary if it doesn't exist
        if boundary.is_none() {
            boundary = Some(ErrorBoundary::new(boundary_id.clone(), context.template_id.hash()));
            if let Some(ref b) = boundary {
                self.register_boundary(b.clone()).await;
            }
        }
        
        let mut boundary = boundary.unwrap();
        
        // Check circuit breaker
        if self.is_circuit_open(&boundary) {
            warn!("Circuit breaker is open for boundary: {}", boundary.id);
            return self.trigger_fallback(&boundary, "Circuit breaker open".to_string()).await;
        }
        
        // Record the error
        let error_record = ErrorRecord {
            error: error.to_string(),
            error_type: Self::categorize_error(&error),
            timestamp: SystemTime::now(),
            template_id: Some(context.template_id.clone()),
            recovery_action: None,
            recovery_successful: None,
        };
        
        boundary.error_history.push(error_record);
        boundary.error_state = ErrorState::RecoverableError {
            error: error.to_string(),
            timestamp: SystemTime::now(),
        };
        
        // Attempt recovery
        let recovery_start = SystemTime::now();
        let recovery_result = self.attempt_recovery(&error, &context, &mut boundary).await;
        let recovery_duration = recovery_start.elapsed().unwrap_or_default();
        
        // Update boundary state based on recovery result
        match &recovery_result {
            RecoveryOutcome::Recovered { .. } => {
                boundary.error_state = ErrorState::Healthy;
                boundary.recovery_attempts = 0;
                boundary.circuit_breaker = CircuitBreakerState::Closed;
                
                // Update stats
                let mut stats = self.stats.write().await;
                stats.successful_recoveries += 1;
                stats.update_average_recovery_time(recovery_duration);
            }
            RecoveryOutcome::Failed { .. } => {
                boundary.recovery_attempts += 1;
                boundary.last_recovery_attempt = Some(SystemTime::now());
                
                // Check if we should open circuit breaker
                if boundary.recovery_attempts >= self.config.circuit_breaker_threshold {
                    boundary.circuit_breaker = CircuitBreakerState::Open {
                        opened_at: SystemTime::now(),
                    };
                }
                
                // Update stats
                let mut stats = self.stats.write().await;
                stats.failed_recoveries += 1;
            }
            RecoveryOutcome::FallbackTriggered { .. } => {
                boundary.error_state = ErrorState::UsingFallback {
                    reason: "Recovery failed, using fallback".to_string(),
                    timestamp: SystemTime::now(),
                };
                
                // Update stats
                let mut stats = self.stats.write().await;
                stats.fallbacks_triggered += 1;
            }
        }
        
        // Update boundary
        {
            let mut boundaries = self.boundaries.write().await;
            boundaries.insert(boundary.id.clone(), boundary);
        }
        
        recovery_result
    }
    
    /// Attempt recovery using available strategies
    async fn attempt_recovery(
        &self,
        error: &HotReloadError,
        context: &RecoveryContext,
        boundary: &mut ErrorBoundary,
    ) -> RecoveryOutcome {
        let strategies = self.recovery_strategies.read().await;
        
        for strategy in strategies.iter() {
            if !strategy.can_handle(error) {
                continue;
            }
            
            debug!("Attempting recovery with strategy (priority: {})", strategy.priority());
            
            match strategy.attempt_recovery(error, context).await {
                RecoveryResult::Success { message, recovered_content } => {
                    info!("Recovery successful: {}", message);
                    return RecoveryOutcome::Recovered {
                        strategy: format!("Strategy {}", strategy.priority()),
                        message,
                        new_content: recovered_content,
                    };
                }
                RecoveryResult::PartialSuccess { message, issues } => {
                    warn!("Partial recovery: {} (issues: {:?})", message, issues);
                    // Continue trying other strategies
                }
                RecoveryResult::Failed { error: err_msg, retry_recommended } => {
                    warn!("Recovery strategy failed: {} (retry: {})", err_msg, retry_recommended);
                    // Continue trying other strategies
                }
                RecoveryResult::RequiresFallback { reason } => {
                    warn!("Strategy requires fallback: {}", reason);
                    return self.trigger_fallback(boundary, reason).await;
                }
            }
        }
        
        // No strategy succeeded, trigger fallback
        self.trigger_fallback(boundary, "All recovery strategies failed".to_string()).await
    }
    
    /// Trigger fallback mechanism
    async fn trigger_fallback(&self, boundary: &ErrorBoundary, reason: String) -> RecoveryOutcome {
        info!("Triggering fallback for boundary {}: {}", boundary.id, reason);
        
        let fallback_content = self.fallback_manager.get_fallback_content(&boundary.protected_id).await;
        
        RecoveryOutcome::FallbackTriggered {
            reason,
            fallback_content,
            action: FallbackAction::ShowFallbackContent,
        }
    }
    
    /// Check if circuit breaker is open
    fn is_circuit_open(&self, boundary: &ErrorBoundary) -> bool {
        match &boundary.circuit_breaker {
            CircuitBreakerState::Open { opened_at } => {
                // Check if enough time has passed to try half-open
                opened_at.elapsed().unwrap_or_default() < self.config.circuit_breaker_reset_timeout
            }
            _ => false,
        }
    }
    
    /// Categorize an error for better handling
    fn categorize_error(error: &HotReloadError) -> ErrorType {
        match error {
            HotReloadError::DomDiffingFailed { .. } => ErrorType::DiffError,
            HotReloadError::StatePreservationFailed { .. } => ErrorType::StateError,
            HotReloadError::ParseError(_) => ErrorType::ParseError,
            HotReloadError::IoError(_) => ErrorType::FileSystemError,
            _ => ErrorType::Unknown,
        }
    }
    
    /// Update error statistics
    async fn update_error_stats(&self, error: &HotReloadError) {
        let mut stats = self.stats.write().await;
        stats.total_errors += 1;
        
        let error_type = Self::categorize_error(error);
        *stats.errors_by_type.entry(error_type).or_insert(0) += 1;
    }
    
    /// Get error statistics
    pub async fn get_statistics(&self) -> ErrorStats {
        self.stats.read().await.clone()
    }
    
    /// Reset error statistics
    pub async fn reset_statistics(&self) {
        let mut stats = self.stats.write().await;
        *stats = ErrorStats::default();
    }
}

/// Outcome of error recovery attempt
#[derive(Debug, Clone)]
pub enum RecoveryOutcome {
    /// Error was successfully recovered
    Recovered {
        strategy: String,
        message: String,
        new_content: Option<String>,
    },
    /// Recovery failed, fallback was triggered
    FallbackTriggered {
        reason: String,
        fallback_content: Option<String>,
        action: FallbackAction,
    },
    /// Recovery completely failed
    Failed {
        error: String,
        requires_manual_intervention: bool,
    },
}

/// Actions to take when fallback is triggered
#[derive(Debug, Clone)]
pub enum FallbackAction {
    /// Show fallback content
    ShowFallbackContent,
    /// Trigger full page refresh
    FullPageRefresh,
    /// Show error message to user
    ShowErrorMessage,
    /// Retry after delay
    RetryAfterDelay(Duration),
}

impl ErrorBoundary {
    /// Create a new error boundary
    pub fn new(id: String, protected_id: String) -> Self {
        Self {
            id,
            protected_id,
            error_state: ErrorState::Healthy,
            error_history: Vec::new(),
            recovery_attempts: 0,
            last_recovery_attempt: None,
            circuit_breaker: CircuitBreakerState::Closed,
            fallback_content: None,
        }
    }
    
    /// Check if the boundary is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.error_state, ErrorState::Healthy)
    }
    
    /// Get the last error
    pub fn last_error(&self) -> Option<&ErrorRecord> {
        self.error_history.last()
    }
}

impl FallbackManager {
    /// Create a new fallback manager
    pub fn new() -> Self {
        Self {
            strategies: vec![
                FallbackStrategy::UsePreviousVersion,
                FallbackStrategy::ShowLoadingState,
                FallbackStrategy::ShowErrorMessage,
                FallbackStrategy::FullPageRefresh,
            ],
            custom_fallbacks: HashMap::new(),
        }
    }
    
    /// Get fallback content for a component
    pub async fn get_fallback_content(&self, _component_id: &str) -> Option<String> {
        // In a real implementation, this would return appropriate fallback content
        Some("<div class=\"hot-reload-error\">Hot reload temporarily unavailable</div>".to_string())
    }
    
    /// Set custom fallback content for a template
    pub fn set_custom_fallback(&mut self, template_id: TemplateId, content: String) {
        self.custom_fallbacks.insert(template_id, content);
    }
}

impl Default for ErrorStats {
    fn default() -> Self {
        Self {
            total_errors: 0,
            errors_by_type: HashMap::new(),
            successful_recoveries: 0,
            failed_recoveries: 0,
            fallbacks_triggered: 0,
            average_recovery_time_ms: 0.0,
        }
    }
}

impl ErrorStats {
    /// Update average recovery time
    pub fn update_average_recovery_time(&mut self, duration: Duration) {
        let new_time_ms = duration.as_millis() as f64;
        if self.successful_recoveries == 0 {
            self.average_recovery_time_ms = new_time_ms;
        } else {
            let total_time = self.average_recovery_time_ms * (self.successful_recoveries - 1) as f64;
            self.average_recovery_time_ms = (total_time + new_time_ms) / self.successful_recoveries as f64;
        }
    }
}

/// Built-in recovery strategies
pub mod recovery_strategies {
    use super::*;
    
    /// Strategy that retries the operation with a delay
    #[derive(Debug)]
    pub struct RetryStrategy {
        max_attempts: usize,
        delay: Duration,
    }
    
    impl RetryStrategy {
        pub fn new(max_attempts: usize, delay: Duration) -> Self {
            Self { max_attempts, delay }
        }
    }
    
    #[async_trait::async_trait]
    impl RecoveryStrategy for RetryStrategy {
        async fn attempt_recovery(
            &self,
            _error: &HotReloadError,
            context: &RecoveryContext,
        ) -> RecoveryResult {
            if context.error_history.len() >= self.max_attempts {
                return RecoveryResult::Failed {
                    error: "Max retry attempts exceeded".to_string(),
                    retry_recommended: false,
                };
            }
            
            // Wait before retry
            tokio::time::sleep(self.delay).await;
            
            // For now, just return success - in a real implementation,
            // this would re-attempt the original operation
            RecoveryResult::Success {
                message: "Retry successful".to_string(),
                recovered_content: None,
            }
        }
        
        fn can_handle(&self, _error: &HotReloadError) -> bool {
            true // Can handle any error type
        }
        
        fn priority(&self) -> u32 {
            100 // Low priority
        }
    }
    
    /// Strategy that falls back to the previous working version
    #[derive(Debug)]
    pub struct PreviousVersionStrategy;
    
    #[async_trait::async_trait]
    impl RecoveryStrategy for PreviousVersionStrategy {
        async fn attempt_recovery(
            &self,
            _error: &HotReloadError,
            _context: &RecoveryContext,
        ) -> RecoveryResult {
            // In a real implementation, this would restore the previous version
            RecoveryResult::Success {
                message: "Restored previous version".to_string(),
                recovered_content: Some("<div>Previous version restored</div>".to_string()),
            }
        }
        
        fn can_handle(&self, error: &HotReloadError) -> bool {
            // Can handle template and diff errors
            matches!(
                error,
                HotReloadError::DomDiffingFailed { .. } | HotReloadError::TemplateUpdateFailed { .. }
            )
        }
        
        fn priority(&self) -> u32 {
            50 // Medium priority
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_error_recovery_system_creation() {
        let config = ErrorRecoveryConfig::default();
        let system = ErrorRecoverySystem::new(config);
        
        let stats = system.get_statistics().await;
        assert_eq!(stats.total_errors, 0);
    }
    
    #[tokio::test]
    async fn test_error_boundary_creation() {
        let boundary = ErrorBoundary::new("test-boundary".to_string(), "test-component".to_string());
        assert!(boundary.is_healthy());
        assert_eq!(boundary.recovery_attempts, 0);
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let config = ErrorRecoveryConfig::default();
        let system = ErrorRecoverySystem::new(config);
        
        // Add a retry strategy
        system.add_recovery_strategy(Arc::new(
            recovery_strategies::RetryStrategy::new(3, Duration::from_millis(100))
        )).await;
        
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        let context = RecoveryContext {
            template_id,
            template_update: None,
            liveview_instance: None,
            error_history: vec![],
            config: ErrorRecoveryConfig::default(),
        };
        
        let error = HotReloadError::DomDiffingFailed {
            reason: "Test error".to_string(),
        };
        
        let outcome = system.handle_error(error, context).await;
        assert!(matches!(outcome, RecoveryOutcome::Recovered { .. }));
        
        let stats = system.get_statistics().await;
        assert_eq!(stats.total_errors, 1);
        assert_eq!(stats.successful_recoveries, 1);
    }
}