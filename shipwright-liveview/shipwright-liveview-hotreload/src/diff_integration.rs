//! Integration of template diffing with the hot reload system
//!
//! This module provides the glue between the template diffing engine and the
//! hot reload infrastructure, determining when changes can be hot-reloaded.

use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::protocol::{TemplateUpdate, TemplateId, HotReloadMessage};
use crate::template_diff::{TemplateDiffer, DiffResult, CompatibilityChecker, BatchOperationBuilder, DeltaOperation};
use crate::template_cache::TemplateCache;

/// Hot reload decision maker that uses the template differ
pub struct HotReloadDecisionMaker {
    /// Template differ for AST-level comparison
    differ: TemplateDiffer,
    /// Compatibility checker
    compatibility_checker: CompatibilityChecker,
    /// Previous template states for comparison
    previous_states: HashMap<String, TemplateUpdate>,
}

impl HotReloadDecisionMaker {
    /// Create a new decision maker
    pub fn new() -> Self {
        Self {
            differ: TemplateDiffer::new(),
            compatibility_checker: CompatibilityChecker::new(),
            previous_states: HashMap::new(),
        }
    }

    /// Analyze template updates and determine hot reload eligibility
    pub fn analyze_updates(&mut self, updates: Vec<TemplateUpdate>) -> Result<HotReloadAnalysis> {
        let mut hot_reloadable = Vec::new();
        let mut require_rebuild = Vec::new();
        let mut delta_operations = HashMap::new();

        for update in updates {
            let hash = update.hash.clone();
            
            // Check if we have a previous version to compare
            if let Some(previous) = self.previous_states.get(&hash) {
                // Diff the templates
                match self.differ.diff_updates(previous, &update) {
                    Ok(diff_result) => {
                        if diff_result.compatible {
                            debug!(
                                "Template {} is hot-reloadable with {} changes",
                                hash,
                                diff_result.changes.len()
                            );
                            
                            // Store delta operations for this template
                            if !diff_result.delta_ops.is_empty() {
                                delta_operations.insert(hash.clone(), diff_result.delta_ops);
                            }
                            
                            hot_reloadable.push(update.clone());
                        } else {
                            info!(
                                "Template {} requires rebuild due to: {:?}",
                                hash,
                                diff_result.compatibility_issues
                            );
                            require_rebuild.push((update.clone(), diff_result.compatibility_issues));
                        }
                    }
                    Err(e) => {
                        warn!("Failed to diff template {}: {}", hash, e);
                        require_rebuild.push((update.clone(), vec![]));
                    }
                }
            } else {
                // New template - can be hot-reloaded
                debug!("Template {} is new, marking as hot-reloadable", hash);
                hot_reloadable.push(update.clone());
            }

            // Update previous state
            self.previous_states.insert(hash, update);
        }

        Ok(HotReloadAnalysis {
            hot_reloadable,
            require_rebuild,
            delta_operations,
        })
    }

    /// Clear previous states (e.g., after a full rebuild)
    pub fn clear_states(&mut self) {
        self.previous_states.clear();
    }

    /// Get statistics about the decision maker
    pub fn get_stats(&self) -> DecisionMakerStats {
        DecisionMakerStats {
            cached_templates: self.previous_states.len(),
        }
    }
}

/// Result of analyzing template updates for hot reload
#[derive(Debug)]
pub struct HotReloadAnalysis {
    /// Templates that can be hot-reloaded
    pub hot_reloadable: Vec<TemplateUpdate>,
    /// Templates that require a full rebuild (with reasons)
    pub require_rebuild: Vec<(TemplateUpdate, Vec<crate::template_diff::CompatibilityIssue>)>,
    /// Delta operations for each hot-reloadable template
    pub delta_operations: HashMap<String, Vec<DeltaOperation>>,
}

/// Statistics about the decision maker
#[derive(Debug)]
pub struct DecisionMakerStats {
    /// Number of cached templates
    pub cached_templates: usize,
}

/// Enhanced protocol message with delta operations
#[derive(Debug, Clone)]
pub enum EnhancedHotReloadMessage {
    /// Standard template update
    Standard(HotReloadMessage),
    /// Delta update with operations
    DeltaUpdate {
        template_id: TemplateId,
        hash: String,
        operations: Vec<DeltaOperation>,
    },
    /// Batch delta update
    BatchDeltaUpdate {
        updates: Vec<(TemplateId, String, Vec<DeltaOperation>)>,
    },
}

/// Convert hot reload analysis to protocol messages
pub fn analysis_to_messages(analysis: HotReloadAnalysis) -> Vec<EnhancedHotReloadMessage> {
    let mut messages = Vec::new();

    // Process hot-reloadable templates
    if !analysis.hot_reloadable.is_empty() {
        // Check if we have delta operations for these templates
        let mut delta_updates = Vec::new();
        let mut standard_updates = Vec::new();

        for update in analysis.hot_reloadable {
            if let Some(ops) = analysis.delta_operations.get(&update.hash) {
                delta_updates.push((update.id, update.hash, ops.clone()));
            } else {
                standard_updates.push(update);
            }
        }

        // Send delta updates if available
        if !delta_updates.is_empty() {
            messages.push(EnhancedHotReloadMessage::BatchDeltaUpdate {
                updates: delta_updates,
            });
        }

        // Send standard updates
        if !standard_updates.is_empty() {
            messages.push(EnhancedHotReloadMessage::Standard(
                HotReloadMessage::BatchUpdate {
                    updates: standard_updates,
                },
            ));
        }
    }

    // Process templates requiring rebuild
    if !analysis.require_rebuild.is_empty() {
        let error_messages: Vec<String> = analysis
            .require_rebuild
            .iter()
            .map(|(update, issues)| {
                format!(
                    "Template {} requires rebuild: {:?}",
                    update.id.file.display(),
                    issues
                )
            })
            .collect();

        messages.push(EnhancedHotReloadMessage::Standard(
            HotReloadMessage::Error {
                message: error_messages.join("\n"),
            },
        ));
    }

    messages
}

/// Integration with the template cache for diff-aware caching
pub struct DiffAwareTemplateCache {
    /// Underlying template cache
    cache: TemplateCache,
    /// Decision maker for hot reload eligibility
    decision_maker: HotReloadDecisionMaker,
}

impl DiffAwareTemplateCache {
    /// Create a new diff-aware cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: TemplateCache::new(max_size),
            decision_maker: HotReloadDecisionMaker::new(),
        }
    }

    /// Insert templates with hot reload analysis
    pub fn insert_with_analysis(&mut self, updates: Vec<TemplateUpdate>) -> Result<HotReloadAnalysis> {
        // First, analyze the updates
        let analysis = self.decision_maker.analyze_updates(updates.clone())?;

        // Insert hot-reloadable templates into cache
        for update in &analysis.hot_reloadable {
            self.cache.insert(update.clone());
        }

        Ok(analysis)
    }

    /// Get a template from cache
    pub fn get(&self, id: &TemplateId) -> Option<&TemplateUpdate> {
        self.cache.get(id)
    }

    /// Clear the cache and decision maker states
    pub fn clear(&mut self) {
        self.cache.clear();
        self.decision_maker.clear_states();
    }

    /// Get cache statistics
    pub fn stats(&self) -> (crate::template_cache::CacheStats, DecisionMakerStats) {
        (self.cache.stats(), self.decision_maker.get_stats())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{DynamicPart, DynamicKind};
    use std::path::PathBuf;

    fn create_test_update(id: &str, html: &str) -> TemplateUpdate {
        TemplateUpdate {
            id: TemplateId::new(PathBuf::from("test.rs"), 1, 1),
            hash: id.to_string(),
            content_hash: format!("{}_content", id),
            html: html.to_string(),
            dynamic_parts: vec![],
        }
    }

    #[test]
    fn test_hot_reload_decision_new_template() {
        let mut decision_maker = HotReloadDecisionMaker::new();
        
        let update = create_test_update("template1", "<div>Hello</div>");
        let analysis = decision_maker.analyze_updates(vec![update.clone()]).unwrap();
        
        assert_eq!(analysis.hot_reloadable.len(), 1);
        assert_eq!(analysis.require_rebuild.len(), 0);
    }

    #[test]
    fn test_hot_reload_decision_compatible_change() {
        let mut decision_maker = HotReloadDecisionMaker::new();
        
        // First update
        let update1 = create_test_update("template1", "<div>Hello</div>");
        decision_maker.analyze_updates(vec![update1]).unwrap();
        
        // Compatible change
        let update2 = create_test_update("template1", "<div>World</div>");
        let analysis = decision_maker.analyze_updates(vec![update2]).unwrap();
        
        assert_eq!(analysis.hot_reloadable.len(), 1);
        assert_eq!(analysis.require_rebuild.len(), 0);
    }

    #[test]
    fn test_diff_aware_cache() {
        let mut cache = DiffAwareTemplateCache::new(100);
        
        let update1 = create_test_update("template1", "<div>Hello</div>");
        let update2 = create_test_update("template2", "<span>World</span>");
        
        let analysis = cache.insert_with_analysis(vec![update1.clone(), update2.clone()]).unwrap();
        
        assert_eq!(analysis.hot_reloadable.len(), 2);
        assert!(cache.get(&update1.id).is_some());
        assert!(cache.get(&update2.id).is_some());
    }

    #[test]
    fn test_enhanced_messages() {
        let analysis = HotReloadAnalysis {
            hot_reloadable: vec![create_test_update("t1", "<div>Test</div>")],
            require_rebuild: vec![],
            delta_operations: HashMap::new(),
        };
        
        let messages = analysis_to_messages(analysis);
        assert_eq!(messages.len(), 1);
        
        match &messages[0] {
            EnhancedHotReloadMessage::Standard(HotReloadMessage::BatchUpdate { updates }) => {
                assert_eq!(updates.len(), 1);
            }
            _ => panic!("Expected standard batch update"),
        }
    }
}