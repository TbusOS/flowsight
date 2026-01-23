//! Knowledge Delta Tracking
//!
//! Tracks incremental knowledge updates from user feedback.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Knowledge entry from user feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// Unique ID
    pub id: String,
    /// Code pattern
    pub pattern: String,
    /// Business meaning
    pub meaning: String,
    /// Trigger condition
    pub trigger: String,
    /// Category
    pub category: String,
    /// Source function
    pub source_function: String,
    /// Confidence (based on user feedback)
    pub confidence: f32,
    /// Usage count
    pub usage_count: usize,
    /// When added
    pub added_at: chrono::DateTime<chrono::Utc>,
    /// Last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Anonymized knowledge for server upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedKnowledge {
    /// Anonymous ID (random)
    pub anonymous_id: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Entries (without source code)
    pub entries: Vec<AnonymizedEntry>,
    /// Statistics
    pub stats: KnowledgeStats,
}

impl AnonymizedKnowledge {
    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Anonymized entry (no source code)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedEntry {
    /// Category
    pub category: String,
    /// Pattern type
    pub pattern_type: String,
    /// Business meaning (anonymized)
    pub meaning: String,
    /// Trigger type
    pub trigger_type: String,
    /// Confidence score
    pub confidence: f32,
}

/// Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeStats {
    pub total_entries: usize,
    pub categories: HashMap<String, usize>,
    pub avg_confidence: f32,
}

/// Knowledge delta tracker
#[derive(Debug)]
pub struct KnowledgeDelta {
    /// All knowledge entries
    entries: HashMap<String, KnowledgeEntry>,
    /// Data directory
    data_dir: PathBuf,
}

impl KnowledgeDelta {
    /// Create new delta tracker
    pub fn new(data_dir: impl AsRef<std::path::Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir).ok();

        // Load existing entries
        let entries = Self::load_entries(&data_dir);

        Self { entries, data_dir }
    }

    /// Load entries from disk
    fn load_entries(data_dir: &PathBuf) -> HashMap<String, KnowledgeEntry> {
        let path = data_dir.join("knowledge_delta.json");

        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(entries) = serde_json::from_str::<HashMap<String, KnowledgeEntry>>(&content) {
                return entries;
            }
        }

        HashMap::new()
    }

    /// Add feedback and extract knowledge
    pub fn add_feedback(&mut self, feedback: &super::UserFeedback) {
        match &feedback.content {
            super::FeedbackContent::KnowledgeSupplement {
                pattern,
                explanation,
                trigger_condition,
                ..
            } => {
                let id = uuid::Uuid::new_v4().to_string();

                let entry = KnowledgeEntry {
                    id: id.clone(),
                    pattern: pattern.clone(),
                    meaning: explanation.clone(),
                    trigger: trigger_condition.clone(),
                    category: feedback.function.clone(),
                    source_function: feedback.function.clone(),
                    confidence: 0.7, // Initial confidence from user
                    usage_count: 0,
                    added_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                self.entries.insert(id, entry);
                self.save();
            }
            super::FeedbackContent::PatternDiscovery {
                snippet: _,
                pattern_type,
                semantics,
                framework,
            } => {
                let id = uuid::Uuid::new_v4().to_string();
                let category = framework.clone().unwrap_or_else(|| "unknown".into());

                let entry = KnowledgeEntry {
                    id: id.clone(),
                    pattern: pattern_type.clone(),
                    meaning: semantics.clone(),
                    trigger: format!("{} pattern detected", pattern_type),
                    category,
                    source_function: feedback.function.clone(),
                    confidence: 0.8,
                    usage_count: 0,
                    added_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                self.entries.insert(id, entry);
                self.save();
            }
            _ => {
                // Other feedback types don't add to knowledge base
            }
        }
    }

    /// Get all entries
    pub fn entries(&self) -> impl Iterator<Item = &KnowledgeEntry> {
        self.entries.values()
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Find matching entry
    pub fn find_matching(&self, code: &str) -> Option<&KnowledgeEntry> {
        self.entries.values()
            .find(|entry| code.contains(&entry.pattern))
            .map(|entry| {
                // Increment usage count (clone for now)
                entry
            })
    }

    /// Save to disk
    fn save(&self) {
        let path = self.data_dir.join("knowledge_delta.json");

        if let Ok(json) = serde_json::to_string_pretty(&self.entries) {
            std::fs::write(&path, json).ok();
        }
    }

    /// Get anonymized knowledge for upload
    pub fn get_anonymized(&self) -> AnonymizedKnowledge {
        let anonymous_id = uuid::Uuid::new_v4().to_string();

        let mut categories = HashMap::new();
        let mut total_confidence = 0.0f32;

        let entries: Vec<_> = self.entries.values()
            .map(|entry| {
                // Update categories
                *categories.entry(entry.category.clone()).or_insert(0) += 1;
                total_confidence += entry.confidence;

                AnonymizedEntry {
                    category: entry.category.clone(),
                    pattern_type: entry.pattern.clone(),
                    meaning: Self::anonymize_meaning(&entry.meaning),
                    trigger_type: Self::anonymize_trigger(&entry.trigger),
                    confidence: entry.confidence,
                }
            })
            .collect();

        let entry_count = entries.len();
        let avg_confidence = if entry_count == 0 {
            0.0
        } else {
            total_confidence / entry_count as f32
        };

        AnonymizedKnowledge {
            anonymous_id,
            timestamp: chrono::Utc::now(),
            entries,
            stats: KnowledgeStats {
                total_entries: entry_count,
                categories,
                avg_confidence,
            },
        }
    }

    /// Anonymize meaning text (remove specific identifiers)
    fn anonymize_meaning(meaning: &str) -> String {
        // Remove specific function/variable names
        let re = regex::Regex::new(r"[a-zA-Z_]\w*").unwrap();
        re.replace_all(meaning, "[name]").to_string()
    }

    /// Anonymize trigger text
    fn anonymize_trigger(trigger: &str) -> String {
        // Keep general structure, remove specifics
        let re = regex::Regex::new(r"[0-9]+").unwrap();
        re.replace_all(trigger, "[N]").to_string()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;
    use super::super::UserFeedback;

    #[test]
    fn test_knowledge_delta() {
        let dir = TempDir::new().unwrap();

        let mut delta = KnowledgeDelta::new(dir.path());

        // Add feedback
        let fb = UserFeedback::new(
            super::super::FeedbackType::KnowledgeSupplement,
            "my_driver_init",
            "INIT_WORK(&dev->work, work_handler);",
            super::super::FeedbackContent::KnowledgeSupplement {
                pattern: "INIT_WORK".to_string(),
                explanation: "初始化工作队列，将处理函数绑定到工作项".to_string(),
                trigger_condition: "在 probe 或 init 函数中调用".to_string(),
                kb_entry: None,
            },
        );

        delta.add_feedback(&fb);

        assert_eq!(delta.len(), 1);

        // Get anonymized
        let anon = delta.get_anonymized();
        assert_eq!(anon.entries.len(), 1);
        assert_eq!(anon.stats.total_entries, 1);
    }
}
