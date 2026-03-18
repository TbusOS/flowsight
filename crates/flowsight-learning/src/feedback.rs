//! Feedback Collection and Storage
//!
//! Manages user feedback for self-learning.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Feedback errors
#[derive(Debug, Error)]
pub enum FeedbackError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Invalid feedback: {0}")]
    Invalid(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Type of user feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackType {
    /// Correctness feedback
    Correctness,
    /// Path selection
    PathSelection,
    /// Knowledge supplement
    KnowledgeSupplement,
    /// Pattern discovery
    PatternDiscovery,
    /// Translation correction
    TranslationCorrection,
}

/// User feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    /// Unique ID
    pub id: String,
    /// Type of feedback
    pub feedback_type: FeedbackType,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Function name
    pub function: String,
    /// Code snippet
    pub code_snippet: String,
    /// Feedback content (type-specific)
    pub content: FeedbackContent,
    /// User notes
    pub notes: Option<String>,
}

/// Feedback content (union type)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FeedbackContent {
    /// Correctness feedback
    #[serde(rename = "correctness")]
    Correctness {
        /// Whether the analysis was correct
        is_correct: bool,
        /// The correct answer
        correct_answer: Option<String>,
        /// What was wrong
        error_description: Option<String>,
    },

    /// Path selection
    #[serde(rename = "path_selection")]
    PathSelection {
        /// Which path was taken
        path_taken: String,
        /// Why this path
        reason: String,
        /// Alternative paths not taken
        alternatives: Vec<String>,
    },

    /// Knowledge supplement
    #[serde(rename = "knowledge")]
    KnowledgeSupplement {
        /// Code pattern
        pattern: String,
        /// Explanation of the pattern
        explanation: String,
        /// Trigger condition
        trigger_condition: String,
        /// Related knowledge base entry
        kb_entry: Option<String>,
    },

    /// Pattern discovery
    #[serde(rename = "pattern")]
    PatternDiscovery {
        /// Code snippet
        snippet: String,
        /// Pattern type
        pattern_type: String,
        /// Semantics
        semantics: String,
        /// Framework if applicable
        framework: Option<String>,
    },

    /// Translation correction
    #[serde(rename = "translation")]
    TranslationCorrection {
        /// Original translation
        original: String,
        /// Correct translation
        corrected: String,
        /// Why it was wrong
        reason: String,
    },
}

/// Feedback collector with SQLite storage
#[derive(Debug)]
pub struct FeedbackCollector {
    /// Database path
    _db_path: PathBuf,
    /// Database connection
    conn: Connection,
}

impl FeedbackCollector {
    /// Create new collector
    pub fn new(data_dir: impl AsRef<std::path::Path>) -> Result<Self, FeedbackError> {
        let data_dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("feedback.db");
        let conn = Connection::open(&db_path)?;

        // Initialize schema
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS feedback (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                function TEXT NOT NULL,
                code_snippet TEXT NOT NULL,
                content TEXT NOT NULL,
                notes TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_feedback_function ON feedback(function);
            CREATE INDEX IF NOT EXISTS idx_feedback_type ON feedback(type);
            CREATE INDEX IF NOT EXISTS idx_feedback_timestamp ON feedback(timestamp);
        "#,
        )?;

        Ok(Self { _db_path: db_path, conn })
    }

    /// Add feedback
    pub fn add(&self, feedback: &UserFeedback) -> Result<(), FeedbackError> {
        let content_json = serde_json::to_string(&feedback.content)?;

        self.conn.execute(
            r#"INSERT INTO feedback
               (id, type, timestamp, function, code_snippet, content, notes)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
            (
                &feedback.id,
                format!("{:?}", feedback.feedback_type),
                feedback.timestamp.to_rfc3339(),
                &feedback.function,
                &feedback.code_snippet,
                &content_json,
                &feedback.notes,
            ),
        )?;

        Ok(())
    }

    /// Get all feedback
    pub fn get_all(&self) -> Result<Vec<UserFeedback>, FeedbackError> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM feedback ORDER BY timestamp DESC")?;
        let rows = stmt.query_map((), |row| {
            let content_json: String = row.get(5)?;
            let content: FeedbackContent =
                serde_json::from_str(&content_json).unwrap_or(FeedbackContent::Correctness {
                    is_correct: true,
                    correct_answer: None,
                    error_description: None,
                });

            Ok(UserFeedback {
                id: row.get(0)?,
                feedback_type: serde_json::from_str(&row.get::<_, String>(1)?)
                    .unwrap_or(FeedbackType::Correctness),
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .unwrap_or_else(|_| {
                        // Create a FixedOffset from UTC (0 seconds offset)
                        let fixed = chrono::FixedOffset::east_opt(0).unwrap();
                        chrono::DateTime::from_timestamp(0, 0)
                            .map(|ts| ts.with_timezone(&fixed))
                            .unwrap_or_else(|| chrono::Utc::now().with_timezone(&fixed))
                    })
                    .into(),
                function: row.get(3)?,
                code_snippet: row.get(4)?,
                content,
                notes: row.get::<_, Option<String>>(6)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// Get feedback count
    pub fn count(&self) -> Result<usize, FeedbackError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM feedback", (), |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Get training samples
    pub fn get_training_samples(&self) -> Result<Vec<TrainingSample>, FeedbackError> {
        let all = self.get_all()?;
        let samples = all
            .into_iter()
            .filter_map(|fb| fb.to_training_sample())
            .collect();

        Ok(samples)
    }

    /// Get feedback by function
    pub fn by_function(&self, function: &str) -> Result<Vec<UserFeedback>, FeedbackError> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM feedback WHERE function = ? ORDER BY timestamp DESC")?;
        let rows = stmt.query_map([function], |row| {
            let content_json: String = row.get(5)?;
            let content: FeedbackContent =
                serde_json::from_str(&content_json).unwrap_or(FeedbackContent::Correctness {
                    is_correct: true,
                    correct_answer: None,
                    error_description: None,
                });

            Ok(UserFeedback {
                id: row.get(0)?,
                feedback_type: serde_json::from_str(&row.get::<_, String>(1)?)
                    .unwrap_or(FeedbackType::Correctness),
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .unwrap_or_else(|_| {
                        // Create a FixedOffset from UTC (0 seconds offset)
                        let fixed = chrono::FixedOffset::east_opt(0).unwrap();
                        chrono::DateTime::from_timestamp(0, 0)
                            .map(|ts| ts.with_timezone(&fixed))
                            .unwrap_or_else(|| chrono::Utc::now().with_timezone(&fixed))
                    })
                    .into(),
                function: row.get(3)?,
                code_snippet: row.get(4)?,
                content,
                notes: row.get::<_, Option<String>>(6)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// Delete old feedback
    pub fn delete_old(&self, keep: usize) -> Result<usize, FeedbackError> {
        // Get IDs to keep using a prepared statement
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM feedback ORDER BY timestamp DESC LIMIT ?")?;
        let ids_to_keep: Result<Vec<String>, rusqlite::Error> =
            stmt.query_map([keep as i64], |row| row.get(0))?.collect();

        let ids: Vec<String> = ids_to_keep?;
        let count = if ids.is_empty() {
            0
        } else {
            // Build the DELETE query with proper parameter handling
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            let query = format!("DELETE FROM feedback WHERE id NOT IN ({})", placeholders);

            // Use a prepared statement with the IDs
            let mut delete_stmt = self.conn.prepare(&query)?;
            let params: Vec<&dyn rusqlite::ToSql> =
                ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
            delete_stmt.execute(&*params)? as usize
        };

        Ok(count)
    }

    /// Clear all feedback
    pub fn clear(&self) -> Result<(), FeedbackError> {
        self.conn.execute("DELETE FROM feedback", ())?;
        Ok(())
    }
}

impl UserFeedback {
    /// Create new feedback
    pub fn new(
        feedback_type: FeedbackType,
        function: impl Into<String>,
        code_snippet: impl Into<String>,
        content: FeedbackContent,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            feedback_type,
            timestamp: chrono::Utc::now(),
            function: function.into(),
            code_snippet: code_snippet.into(),
            content,
            notes: None,
        }
    }

    /// Convert to training sample
    pub fn to_training_sample(self) -> Option<TrainingSample> {
        match self.content {
            FeedbackContent::Correctness {
                is_correct,
                correct_answer,
                error_description,
            } => Some(TrainingSample {
                instruction: format!("分析以下代码中函数的执行是否正确：{}", self.function),
                input: self.code_snippet,
                output: correct_answer.unwrap_or_else(|| {
                    if is_correct {
                        "正确".into()
                    } else {
                        error_description.unwrap_or("错误".into())
                    }
                }),
                feedback_type: "correctness".into(),
            }),
            FeedbackContent::KnowledgeSupplement {
                pattern,
                explanation,
                trigger_condition,
                ..
            } => Some(TrainingSample {
                instruction: format!("解释代码模式 '{}' 的业务含义和触发条件", pattern),
                input: self.code_snippet,
                output: format!("{}\n触发条件: {}", explanation, trigger_condition),
                feedback_type: "knowledge".into(),
            }),
            FeedbackContent::PatternDiscovery {
                snippet: _,
                pattern_type,
                semantics,
                ..
            } => Some(TrainingSample {
                instruction: format!("识别以下代码中的 {} 模式", pattern_type),
                input: self.code_snippet,
                output: semantics,
                feedback_type: "pattern".into(),
            }),
            FeedbackContent::PathSelection {
                path_taken, reason, ..
            } => Some(TrainingSample {
                instruction: format!("分析 {} 函数中哪条执行路径会被采用", self.function),
                input: self.code_snippet,
                output: format!("路径: {}\n原因: {}", path_taken, reason),
                feedback_type: "path".into(),
            }),
            FeedbackContent::TranslationCorrection {
                original,
                corrected,
                reason: _,
            } => Some(TrainingSample {
                instruction: format!("翻译以下代码条件约束：{}", original),
                input: self.code_snippet,
                output: corrected,
                feedback_type: "translation".into(),
            }),
        }
    }
}

/// Training sample for LoRA training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    pub instruction: String,
    pub input: String,
    pub output: String,
    pub feedback_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_feedback_collector() -> Result<(), FeedbackError> {
        let dir = TempDir::new().unwrap();
        let collector = FeedbackCollector::new(dir.path())?;

        // Add some feedback
        let fb = UserFeedback::new(
            FeedbackType::Correctness,
            "my_probe",
            "int ret = usb_control_msg(...);",
            FeedbackContent::Correctness {
                is_correct: true,
                correct_answer: Some("USB 控制消息发送成功".into()),
                error_description: None,
            },
        );
        collector.add(&fb)?;

        // Check count
        assert_eq!(collector.count()?, 1);

        // Get training samples
        let samples = collector.get_training_samples()?;
        assert_eq!(samples.len(), 1);
        assert!(samples[0].output.contains("成功"));

        Ok(())
    }
}
