//! FlowSight Self-Learning Module
//!
//! Provides user feedback collection and local model adaptation:
//! - User feedback storage and management
//! - Local LoRA weight updates
//! - Knowledge base delta tracking
//! - Anonymous knowledge upload (optional)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      User Feedback Flow                          │
//├─────────────────────────────────────────────────────────────────┤
//!                                                                  │
//!  User uses IDE                                                   │
//!       │                                                          │
//!       ▼                                                          │
//!  ┌─────────────────────────────────────────────────────────┐     │
//!  │  Feedback Collector                                      │     │
//!  │  • Correctness feedback (👍/👎)                        │     │
//!  │  • Path selection                                       │     │
//!  │  • Knowledge supplement                                 │     │
//!  │  • Pattern discovery                                    │     │
//!  └─────────────────────────────────────────────────────────┘     │
//!       │                                                          │
//!       ▼                                                          │
//!  ┌─────────────────────────────────────────────────────────┐     │
//!  │  Feedback Store (SQLite)                                 │     │
//!  │  • Local storage of all feedback                         │     │
//!  │  • Organized by function/path                            │     │
//!  └─────────────────────────────────────────────────────────┘     │
//!       │                                                          │
//!       ▼                                                          │
//!  ┌─────────────────────────────────────────────────────────┐     │
//!  │  Local Trainer (LoRA)                                    │     │
//!  │  • Collect 100+ feedbacks                                │     │
//!  │  • Run incremental training                              │     │
//!  │  • Update local model weights                            │     │
//!  └─────────────────────────────────────────────────────────┘     │
//!       │                                                          │
//!       ▼                                                          │
//!  ┌─────────────────────────────────────────────────────────┐     │
//!  │  Knowledge Uploader (Optional)                           │     │
//!  │  • Anonymize feedback                                    │     │
//!  │  • Upload to server                                      │     │
//!  │  • Help improve global model                             │     │
//!  └─────────────────────────────────────────────────────────┘     │
//!                                                                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

pub mod feedback;
pub mod trainer;
pub mod knowledge_delta;

pub use feedback::{FeedbackCollector, UserFeedback, FeedbackType, FeedbackContent, TrainingSample};
pub use trainer::{LocalTrainer, TrainingConfig};
pub use knowledge_delta::{KnowledgeDelta, AnonymizedKnowledge};

/// Self-learning configuration
#[derive(Debug, Clone)]
pub struct LearningConfig {
    /// User data directory
    pub data_dir: PathBuf,
    /// Minimum feedbacks before training
    pub min_feedbacks: usize,
    /// Maximum feedbacks to keep
    pub max_feedbacks: usize,
    /// Enable anonymous upload
    pub enable_upload: bool,
    /// Upload server URL
    pub upload_url: String,
    /// LoRA rank
    pub lora_rank: usize,
    /// Learning rate for LoRA
    pub learning_rate: f32,
    /// Batch size
    pub batch_size: usize,
    /// Epochs for training
    pub epochs: usize,
}

use std::path::PathBuf;

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            data_dir: get_default_data_dir(),
            min_feedbacks: 100,
            max_feedbacks: 10000,
            enable_upload: false,
            upload_url: "https://api.flowsight.dev/knowledge-upload".into(),
            lora_rank: 16,
            learning_rate: 2e-4,
            batch_size: 4,
            epochs: 1,
        }
    }
}

impl LearningConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set data directory
    pub fn with_data_dir(mut self, dir: impl AsRef<std::path::Path>) -> Self {
        self.data_dir = dir.as_ref().to_path_buf();
        self
    }

    /// Set minimum feedbacks
    pub fn with_min_feedbacks(mut self, n: usize) -> Self {
        self.min_feedbacks = n;
        self
    }

    /// Enable upload
    pub fn with_upload_enabled(mut self, enabled: bool) -> Self {
        self.enable_upload = enabled;
        self
    }
}

/// Get default data directory
fn get_default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".flowsight")
        .join("learning")
}

/// Main self-learning manager
#[derive(Debug)]
pub struct SelfLearning {
    /// Configuration
    config: LearningConfig,
    /// Feedback collector
    feedback: FeedbackCollector,
    /// Local trainer
    trainer: Option<LocalTrainer>,
    /// Knowledge delta tracker
    knowledge_delta: KnowledgeDelta,
    /// Feedback count since last training
    feedback_count: usize,
}

impl SelfLearning {
    /// Create new self-learning manager
    pub fn new(config: LearningConfig) -> Result<Self, anyhow::Error> {
        let data_dir = config.data_dir.clone();
        std::fs::create_dir_all(&data_dir)?;

        let feedback = FeedbackCollector::new(&data_dir)?;
        let knowledge_delta = KnowledgeDelta::new(&data_dir);

        Ok(Self {
            config,
            feedback,
            trainer: None,
            knowledge_delta,
            feedback_count: 0,
        })
    }

    /// Collect user feedback
    pub async fn collect_feedback(&mut self, fb: UserFeedback) -> Result<(), anyhow::Error> {
        // Store feedback
        self.feedback.add(&fb)?;

        // Update knowledge delta if applicable
        self.knowledge_delta.add_feedback(&fb);

        // Increment counter
        self.feedback_count += 1;

        // Check if we should train
        if self.feedback_count >= self.config.min_feedbacks {
            self.check_and_train().await?;
        }

        Ok(())
    }

    /// Check if training is needed and run it
    async fn check_and_train(&mut self) -> Result<(), anyhow::Error> {
        // Check if we have enough feedback
        let count = self.feedback.count()?;
        if count < self.config.min_feedbacks {
            tracing::info!("Not enough feedback for training: {}/{}",
                count, self.config.min_feedbacks);
            return Ok(());
        }

        tracing::info!("Starting local training with {} feedbacks", count);

        // Create trainer if needed
        if self.trainer.is_none() {
            let training_config = TrainingConfig {
                model_path: self.config.data_dir.join("models"),
                output_path: self.config.data_dir.join("lora"),
                r: self.config.lora_rank,
                alpha: self.config.lora_rank * 2,
                dropout: 0.05,
                learning_rate: self.config.learning_rate,
                batch_size: self.config.batch_size,
                epochs: self.config.epochs,
                warmup_steps: 100,
                max_steps: 500,
                save_steps: 250,
            };
            self.trainer = Some(LocalTrainer::new(&training_config)?);
        }

        // Get training data
        let samples = self.feedback.get_training_samples()?;

        if samples.is_empty() {
            tracing::warn!("No valid training samples found");
            return Ok(());
        }

        // Run training
        if let Some(trainer) = self.trainer.as_mut() {
            trainer.train(&samples).await?;
        }

        // Reset counter
        self.feedback_count = 0;

        // Optionally upload anonymized knowledge
        if self.config.enable_upload {
            self.upload_anonymized_knowledge().await?;
        }

        Ok(())
    }

    /// Upload anonymized knowledge to server
    async fn upload_anonymized_knowledge(&self) -> Result<(), anyhow::Error> {
        let knowledge = self.knowledge_delta.get_anonymized();

        if knowledge.is_empty() {
            return Ok(());
        }

        tracing::info!("Uploading {} anonymized knowledge entries", knowledge.len());

        let client = reqwest::blocking::Client::new();
        let _response = client
            .post(&self.config.upload_url)
            .json(&knowledge)
            .send()
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;

        tracing::info!("Knowledge upload successful");
        Ok(())
    }

    /// Get current statistics
    pub fn stats(&self) -> LearningStats {
        LearningStats {
            total_feedback: self.feedback.count().unwrap_or(0),
            pending_training: self.feedback_count,
            min_required: self.config.min_feedbacks,
            knowledge_entries: self.knowledge_delta.len(),
            has_model: self.trainer.is_some(),
        }
    }
}

/// Learning statistics
#[derive(Debug, Clone)]
pub struct LearningStats {
    /// Total feedback count
    pub total_feedback: usize,
    /// Pending training count
    pub pending_training: usize,
    /// Minimum required for training
    pub min_required: usize,
    /// Knowledge delta entries
    pub knowledge_entries: usize,
    /// Whether model is trained
    pub has_model: bool,
}
