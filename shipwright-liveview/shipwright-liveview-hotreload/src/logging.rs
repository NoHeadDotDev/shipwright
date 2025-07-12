//! Comprehensive logging and debugging support for hot reload operations
//! 
//! This module provides detailed logging, debugging tools, and observability
//! features to help developers understand what's happening during hot reload.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::protocol::{TemplateId, TemplateUpdate};
use crate::runtime::HotReloadError;
use crate::liveview_integration::{ActiveLiveView, HotReloadResult};

/// Hot reload debugging and logging system
#[derive(Debug)]
pub struct HotReloadLogger {
    /// Configuration for logging
    config: LoggingConfig,
    /// Event buffer for debugging
    event_buffer: Arc<RwLock<VecDeque<LogEvent>>>,
    /// Performance metrics
    metrics: Arc<RwLock<PerformanceMetrics>>,
    /// Debug session state
    debug_session: Arc<RwLock<Option<DebugSession>>>,
    /// Subscribers for real-time events
    subscribers: Arc<RwLock<Vec<Arc<dyn LogEventSubscriber + Send + Sync>>>>,
}

/// Configuration for the logging system
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Logging level
    pub level: LogLevel,
    /// Maximum number of events to buffer
    pub max_events: usize,
    /// Whether to include detailed timing information
    pub include_timing: bool,
    /// Whether to include stack traces for errors
    pub include_stack_traces: bool,
    /// Whether to log DOM patches
    pub log_dom_patches: bool,
    /// Whether to log state preservation details
    pub log_state_preservation: bool,
    /// Output format
    pub output_format: LogFormat,
    /// Whether to enable real-time debugging
    pub enable_debug_mode: bool,
    /// File to write logs to (optional)
    pub log_file: Option<String>,
}

/// Logging levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Log output formats
#[derive(Debug, Clone)]
pub enum LogFormat {
    /// Human-readable text
    Text,
    /// JSON format
    Json,
    /// Compact format for development
    Compact,
    /// Pretty-printed with colors
    Pretty,
}

/// Individual log event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    /// Event ID for tracking
    pub id: String,
    /// When the event occurred
    pub timestamp: SystemTime,
    /// Event type/category
    pub event_type: LogEventType,
    /// Log level
    pub level: LogLevel,
    /// Main message
    pub message: String,
    /// Additional context data
    pub context: HashMap<String, serde_json::Value>,
    /// Template ID if applicable
    pub template_id: Option<TemplateId>,
    /// Duration if this is a timed event
    pub duration: Option<Duration>,
    /// Related events (by ID)
    pub related_events: Vec<String>,
}

/// Types of log events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogEventType {
    /// File change detected
    FileChange,
    /// Template parsing started/completed
    TemplateParsing,
    /// DOM diff generation
    DomDiffing,
    /// Component update
    ComponentUpdate,
    /// State preservation
    StatePreservation,
    /// Error occurred
    Error,
    /// Recovery attempt
    ErrorRecovery,
    /// WebSocket communication
    WebSocketEvent,
    /// Performance measurement
    Performance,
    /// Debug information
    Debug,
}

/// Performance metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total number of updates processed
    pub total_updates: usize,
    /// Average update time
    pub average_update_time_ms: f64,
    /// Fastest update time
    pub fastest_update_ms: Option<f64>,
    /// Slowest update time
    pub slowest_update_ms: Option<f64>,
    /// Updates by hour for trend analysis
    pub updates_by_hour: HashMap<String, usize>,
    /// File change to DOM update latency
    pub change_to_update_latency_ms: VecDeque<f64>,
    /// DOM patch sizes
    pub patch_sizes: VecDeque<usize>,
    /// State preservation times
    pub state_preservation_times_ms: VecDeque<f64>,
}

/// Active debugging session
#[derive(Debug, Clone)]
pub struct DebugSession {
    /// Session ID
    pub id: String,
    /// Start time
    pub start_time: SystemTime,
    /// Templates being debugged
    pub watched_templates: Vec<TemplateId>,
    /// Components being debugged
    pub watched_components: Vec<String>,
    /// Breakpoints for debugging
    pub breakpoints: Vec<DebugBreakpoint>,
    /// Step-through mode
    pub step_mode: bool,
    /// Current step
    pub current_step: Option<String>,
}

/// Debug breakpoint
#[derive(Debug, Clone)]
pub struct DebugBreakpoint {
    /// Breakpoint ID
    pub id: String,
    /// Template to break on
    pub template_id: Option<TemplateId>,
    /// Event type to break on
    pub event_type: Option<LogEventType>,
    /// Condition for breaking
    pub condition: Option<String>,
    /// Whether this breakpoint is enabled
    pub enabled: bool,
}

/// Trait for subscribing to log events
pub trait LogEventSubscriber {
    /// Called when a new log event occurs
    fn on_log_event(&self, event: &LogEvent);
    
    /// Called when performance metrics are updated
    fn on_metrics_update(&self, metrics: &PerformanceMetrics);
    
    /// Called when a debug breakpoint is hit
    fn on_breakpoint_hit(&self, breakpoint: &DebugBreakpoint, event: &LogEvent);
}

/// Debug command for interactive debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DebugCommand {
    /// Start a debug session
    StartSession {
        templates: Vec<TemplateId>,
        components: Vec<String>,
    },
    /// Stop the current debug session
    StopSession,
    /// Add a breakpoint
    AddBreakpoint {
        template_id: Option<TemplateId>,
        event_type: Option<LogEventType>,
        condition: Option<String>,
    },
    /// Remove a breakpoint
    RemoveBreakpoint { id: String },
    /// Step to next event
    Step,
    /// Continue execution
    Continue,
    /// Get current state
    GetState,
    /// Get event history
    GetEventHistory { limit: Option<usize> },
    /// Get performance metrics
    GetMetrics,
}

impl HotReloadLogger {
    /// Create a new hot reload logger
    pub fn new(config: LoggingConfig) -> Self {
        Self {
            config,
            event_buffer: Arc::new(RwLock::new(VecDeque::new())),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            debug_session: Arc::new(RwLock::new(None)),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Initialize the logging system
    pub fn init_logging(&self) -> Result<(), Box<dyn std::error::Error>> {
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&format!("{:?}", self.config.level)))?;
        
        let fmt_layer = match self.config.output_format {
            LogFormat::Text => fmt::layer().boxed(),
            LogFormat::Json => fmt::layer().json().boxed(),
            LogFormat::Compact => fmt::layer().compact().boxed(),
            LogFormat::Pretty => fmt::layer().pretty().boxed(),
        };
        
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
        
        info!("Hot reload logging system initialized");
        Ok(())
    }
    
    /// Log a hot reload event
    pub async fn log_event(
        &self,
        event_type: LogEventType,
        level: LogLevel,
        message: String,
        context: Option<HashMap<String, serde_json::Value>>,
        template_id: Option<TemplateId>,
        duration: Option<Duration>,
    ) -> String {
        let event_id = uuid::Uuid::new_v4().to_string();
        
        let event = LogEvent {
            id: event_id.clone(),
            timestamp: SystemTime::now(),
            event_type: event_type.clone(),
            level,
            message: message.clone(),
            context: context.unwrap_or_default(),
            template_id: template_id.clone(),
            duration,
            related_events: Vec::new(),
        };
        
        // Log using tracing
        match level {
            LogLevel::Trace => tracing::trace!(
                event_type = ?event_type,
                template_id = ?template_id,
                duration_ms = ?duration.map(|d| d.as_millis()),
                "{}",
                message
            ),
            LogLevel::Debug => tracing::debug!(
                event_type = ?event_type,
                template_id = ?template_id,
                duration_ms = ?duration.map(|d| d.as_millis()),
                "{}",
                message
            ),
            LogLevel::Info => tracing::info!(
                event_type = ?event_type,
                template_id = ?template_id,
                duration_ms = ?duration.map(|d| d.as_millis()),
                "{}",
                message
            ),
            LogLevel::Warn => tracing::warn!(
                event_type = ?event_type,
                template_id = ?template_id,
                duration_ms = ?duration.map(|d| d.as_millis()),
                "{}",
                message
            ),
            LogLevel::Error => tracing::error!(
                event_type = ?event_type,
                template_id = ?template_id,
                duration_ms = ?duration.map(|d| d.as_millis()),
                "{}",
                message
            ),
        }
        
        // Add to buffer
        {
            let mut buffer = self.event_buffer.write().await;
            buffer.push_back(event.clone());
            
            // Trim buffer if needed
            while buffer.len() > self.config.max_events {
                buffer.pop_front();
            }
        }
        
        // Update metrics
        if let Some(duration) = duration {
            self.update_metrics(duration).await;
        }
        
        // Check for debug breakpoints
        self.check_breakpoints(&event).await;
        
        // Notify subscribers
        let subscribers = self.subscribers.read().await;
        for subscriber in subscribers.iter() {
            subscriber.on_log_event(&event);
        }
        
        event_id
    }
    
    /// Update performance metrics
    async fn update_metrics(&self, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        
        let duration_ms = duration.as_millis() as f64;
        
        metrics.total_updates += 1;
        
        // Update average
        if metrics.total_updates == 1 {
            metrics.average_update_time_ms = duration_ms;
        } else {
            let total_time = metrics.average_update_time_ms * (metrics.total_updates - 1) as f64;
            metrics.average_update_time_ms = (total_time + duration_ms) / metrics.total_updates as f64;
        }
        
        // Update fastest/slowest
        if metrics.fastest_update_ms.is_none() || duration_ms < metrics.fastest_update_ms.unwrap() {
            metrics.fastest_update_ms = Some(duration_ms);
        }
        if metrics.slowest_update_ms.is_none() || duration_ms > metrics.slowest_update_ms.unwrap() {
            metrics.slowest_update_ms = Some(duration_ms);
        }
        
        // Track hourly trends
        let hour_key = format!("{}", chrono::Utc::now().format("%Y-%m-%d %H:00"));
        *metrics.updates_by_hour.entry(hour_key).or_insert(0) += 1;
    }
    
    /// Check debug breakpoints
    async fn check_breakpoints(&self, event: &LogEvent) {
        let debug_session = self.debug_session.read().await;
        if let Some(ref session) = *debug_session {
            for breakpoint in &session.breakpoints {
                if !breakpoint.enabled {
                    continue;
                }
                
                let mut should_break = false;
                
                // Check template match
                if let Some(ref bp_template) = breakpoint.template_id {
                    if let Some(ref event_template) = event.template_id {
                        should_break = bp_template == event_template;
                    }
                } else if breakpoint.template_id.is_none() {
                    should_break = true;
                }
                
                // Check event type match
                if should_break {
                    if let Some(ref bp_event_type) = breakpoint.event_type {
                        should_break = bp_event_type == &event.event_type;
                    }
                }
                
                // Check condition (simplified - would need expression evaluation)
                if should_break && breakpoint.condition.is_some() {
                    // For now, just evaluate simple conditions
                    should_break = true;
                }
                
                if should_break {
                    info!("Debug breakpoint hit: {}", breakpoint.id);
                    
                    // Notify subscribers
                    let subscribers = self.subscribers.read().await;
                    for subscriber in subscribers.iter() {
                        subscriber.on_breakpoint_hit(breakpoint, event);
                    }
                    
                    // In step mode, pause execution
                    if session.step_mode {
                        // Would implement actual pause mechanism here
                        debug!("Pausing at breakpoint: {}", breakpoint.id);
                    }
                }
            }
        }
    }
    
    /// Add a log event subscriber
    pub async fn add_subscriber(&self, subscriber: Arc<dyn LogEventSubscriber + Send + Sync>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(subscriber);
    }
    
    /// Handle debug command
    pub async fn handle_debug_command(&self, command: DebugCommand) -> Result<serde_json::Value, String> {
        match command {
            DebugCommand::StartSession { templates, components } => {
                let session = DebugSession {
                    id: uuid::Uuid::new_v4().to_string(),
                    start_time: SystemTime::now(),
                    watched_templates: templates,
                    watched_components: components,
                    breakpoints: Vec::new(),
                    step_mode: false,
                    current_step: None,
                };
                
                let session_id = session.id.clone();
                {
                    let mut debug_session = self.debug_session.write().await;
                    *debug_session = Some(session);
                }
                
                info!("Started debug session: {}", session_id);
                Ok(serde_json::json!({ "session_id": session_id }))
            }
            DebugCommand::StopSession => {
                let mut debug_session = self.debug_session.write().await;
                *debug_session = None;
                
                info!("Stopped debug session");
                Ok(serde_json::json!({ "status": "stopped" }))
            }
            DebugCommand::AddBreakpoint { template_id, event_type, condition } => {
                let breakpoint = DebugBreakpoint {
                    id: uuid::Uuid::new_v4().to_string(),
                    template_id,
                    event_type,
                    condition,
                    enabled: true,
                };
                
                let breakpoint_id = breakpoint.id.clone();
                
                {
                    let mut debug_session = self.debug_session.write().await;
                    if let Some(ref mut session) = *debug_session {
                        session.breakpoints.push(breakpoint);
                    } else {
                        return Err("No active debug session".to_string());
                    }
                }
                
                info!("Added breakpoint: {}", breakpoint_id);
                Ok(serde_json::json!({ "breakpoint_id": breakpoint_id }))
            }
            DebugCommand::GetEventHistory { limit } => {
                let buffer = self.event_buffer.read().await;
                let events: Vec<_> = buffer
                    .iter()
                    .rev()
                    .take(limit.unwrap_or(100))
                    .collect();
                
                Ok(serde_json::to_value(events).unwrap_or_default())
            }
            DebugCommand::GetMetrics => {
                let metrics = self.metrics.read().await;
                Ok(serde_json::to_value(&*metrics).unwrap_or_default())
            }
            _ => {
                Ok(serde_json::json!({ "status": "not_implemented" }))
            }
        }
    }
    
    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Get recent log events
    pub async fn get_recent_events(&self, limit: usize) -> Vec<LogEvent> {
        let buffer = self.event_buffer.read().await;
        buffer.iter().rev().take(limit).cloned().collect()
    }
    
    /// Clear all logged events
    pub async fn clear_events(&self) {
        let mut buffer = self.event_buffer.write().await;
        buffer.clear();
        
        let mut metrics = self.metrics.write().await;
        *metrics = PerformanceMetrics::default();
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            max_events: 1000,
            include_timing: true,
            include_stack_traces: true,
            log_dom_patches: false,
            log_state_preservation: true,
            output_format: LogFormat::Pretty,
            enable_debug_mode: cfg!(debug_assertions),
            log_file: None,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_updates: 0,
            average_update_time_ms: 0.0,
            fastest_update_ms: None,
            slowest_update_ms: None,
            updates_by_hour: HashMap::new(),
            change_to_update_latency_ms: VecDeque::new(),
            patch_sizes: VecDeque::new(),
            state_preservation_times_ms: VecDeque::new(),
        }
    }
}

/// Helper functions for common logging operations
impl HotReloadLogger {
    /// Log a file change event
    pub async fn log_file_change(&self, file_path: &str, change_type: &str) -> String {
        let mut context = HashMap::new();
        context.insert("file_path".to_string(), serde_json::Value::String(file_path.to_string()));
        context.insert("change_type".to_string(), serde_json::Value::String(change_type.to_string()));
        
        self.log_event(
            LogEventType::FileChange,
            LogLevel::Debug,
            format!("File changed: {} ({})", file_path, change_type),
            Some(context),
            None,
            None,
        ).await
    }
    
    /// Log a template parsing event
    pub async fn log_template_parsing(&self, template_id: &TemplateId, duration: Duration, success: bool) -> String {
        let mut context = HashMap::new();
        context.insert("success".to_string(), serde_json::Value::Bool(success));
        
        let message = if success {
            format!("Template parsed successfully: {:?}", template_id)
        } else {
            format!("Template parsing failed: {:?}", template_id)
        };
        
        self.log_event(
            LogEventType::TemplateParsing,
            if success { LogLevel::Debug } else { LogLevel::Error },
            message,
            Some(context),
            Some(template_id.clone()),
            Some(duration),
        ).await
    }
    
    /// Log a component update
    pub async fn log_component_update(&self, instance_id: &str, result: &HotReloadResult) -> String {
        let mut context = HashMap::new();
        context.insert("instance_id".to_string(), serde_json::Value::String(instance_id.to_string()));
        
        let (message, level) = match result {
            HotReloadResult::Updated { patch_size, state_preserved, .. } => {
                context.insert("patch_size".to_string(), serde_json::Value::Number((*patch_size).into()));
                context.insert("state_preserved".to_string(), serde_json::Value::Bool(*state_preserved));
                (format!("Component updated: {} (patch size: {})", instance_id, patch_size), LogLevel::Info)
            }
            HotReloadResult::FullRefreshRequired { reason } => {
                context.insert("reason".to_string(), serde_json::Value::String(reason.clone()));
                (format!("Full refresh required for {}: {}", instance_id, reason), LogLevel::Warn)
            }
            HotReloadResult::Failed { error, .. } => {
                context.insert("error".to_string(), serde_json::Value::String(error.to_string()));
                (format!("Update failed for {}: {}", instance_id, error), LogLevel::Error)
            }
        };
        
        self.log_event(
            LogEventType::ComponentUpdate,
            level,
            message,
            Some(context),
            None,
            None,
        ).await
    }
    
    /// Log an error with detailed context
    pub async fn log_error(&self, error: &HotReloadError, context: Option<&str>) -> String {
        let mut ctx = HashMap::new();
        ctx.insert("error_type".to_string(), serde_json::Value::String(format!("{:?}", error)));
        
        if let Some(context_str) = context {
            ctx.insert("context".to_string(), serde_json::Value::String(context_str.to_string()));
        }
        
        if self.config.include_stack_traces {
            // Would include stack trace here in a real implementation
            ctx.insert("stack_trace".to_string(), serde_json::Value::String("Stack trace placeholder".to_string()));
        }
        
        self.log_event(
            LogEventType::Error,
            LogLevel::Error,
            format!("Hot reload error: {}", error),
            Some(ctx),
            None,
            None,
        ).await
    }
}

/// Built-in log event subscribers
pub mod subscribers {
    use super::*;
    
    /// Console subscriber that prints events to stdout
    #[derive(Debug)]
    pub struct ConsoleSubscriber {
        verbose: bool,
    }
    
    impl ConsoleSubscriber {
        pub fn new(verbose: bool) -> Self {
            Self { verbose }
        }
    }
    
    impl LogEventSubscriber for ConsoleSubscriber {
        fn on_log_event(&self, event: &LogEvent) {
            if self.verbose || event.level >= LogLevel::Info {
                println!("[{}] {} - {}", 
                    event.timestamp.duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default().as_secs(),
                    format!("{:?}", event.level).to_uppercase(),
                    event.message
                );
            }
        }
        
        fn on_metrics_update(&self, metrics: &PerformanceMetrics) {
            if self.verbose {
                println!("Metrics: {} updates, avg {:.2}ms", 
                    metrics.total_updates, 
                    metrics.average_update_time_ms
                );
            }
        }
        
        fn on_breakpoint_hit(&self, breakpoint: &DebugBreakpoint, event: &LogEvent) {
            println!("🔴 Breakpoint hit: {} - {}", breakpoint.id, event.message);
        }
    }
    
    /// File subscriber that writes events to a file
    #[derive(Debug)]
    pub struct FileSubscriber {
        file_path: String,
    }
    
    impl FileSubscriber {
        pub fn new(file_path: String) -> Self {
            Self { file_path }
        }
    }
    
    impl LogEventSubscriber for FileSubscriber {
        fn on_log_event(&self, event: &LogEvent) {
            // Would write to file in a real implementation
            debug!("Would write to {}: {}", self.file_path, event.message);
        }
        
        fn on_metrics_update(&self, _metrics: &PerformanceMetrics) {
            // Would write metrics to file
        }
        
        fn on_breakpoint_hit(&self, _breakpoint: &DebugBreakpoint, _event: &LogEvent) {
            // Would write breakpoint info to file
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_logger_creation() {
        let config = LoggingConfig::default();
        let logger = HotReloadLogger::new(config);
        
        let metrics = logger.get_metrics().await;
        assert_eq!(metrics.total_updates, 0);
    }
    
    #[tokio::test]
    async fn test_event_logging() {
        let config = LoggingConfig::default();
        let logger = HotReloadLogger::new(config);
        
        let event_id = logger.log_event(
            LogEventType::Debug,
            LogLevel::Info,
            "Test event".to_string(),
            None,
            None,
            Some(Duration::from_millis(100)),
        ).await;
        
        assert!(!event_id.is_empty());
        
        let events = logger.get_recent_events(10).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].message, "Test event");
    }
    
    #[tokio::test]
    async fn test_metrics_update() {
        let config = LoggingConfig::default();
        let logger = HotReloadLogger::new(config);
        
        // Log a few events with timing
        for i in 1..=5 {
            logger.log_event(
                LogEventType::Performance,
                LogLevel::Info,
                format!("Update {}", i),
                None,
                None,
                Some(Duration::from_millis(i * 100)),
            ).await;
        }
        
        let metrics = logger.get_metrics().await;
        assert_eq!(metrics.total_updates, 5);
        assert_eq!(metrics.fastest_update_ms, Some(100.0));
        assert_eq!(metrics.slowest_update_ms, Some(500.0));
    }
    
    #[tokio::test]
    async fn test_debug_commands() {
        let config = LoggingConfig::default();
        let logger = HotReloadLogger::new(config);
        
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        
        // Start debug session
        let result = logger.handle_debug_command(DebugCommand::StartSession {
            templates: vec![template_id],
            components: vec!["TestComponent".to_string()],
        }).await;
        
        assert!(result.is_ok());
        
        // Add breakpoint
        let result = logger.handle_debug_command(DebugCommand::AddBreakpoint {
            template_id: None,
            event_type: Some(LogEventType::ComponentUpdate),
            condition: None,
        }).await;
        
        assert!(result.is_ok());
        
        // Stop session
        let result = logger.handle_debug_command(DebugCommand::StopSession).await;
        assert!(result.is_ok());
    }
}