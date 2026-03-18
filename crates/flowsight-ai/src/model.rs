//! Local Model Management
//!
//! Manages the local AI model for inference.

use super::AiConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Model errors
#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Model not found: {0}")]
    NotFound(String),

    #[error("Failed to load model: {0}")]
    LoadError(String),

    #[error("Inference error: {0}")]
    InferenceError(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Local model wrapper
#[derive(Debug)]
pub struct LocalModel {
    /// Model path
    path: PathBuf,
    /// Model name
    name: String,
    /// Configuration
    _config: AiConfig,
    /// Loaded model handle
    handle: Option<ModelHandle>,
}

/// Model handle (platform-specific)
#[derive(Debug)]
pub struct ModelHandle {
    /// For now, we store metadata about the model
    _path: PathBuf,
    /// Model file size in bytes
    file_size: u64,
    /// Whether it's loaded
    is_loaded: bool,
}

impl LocalModel {
    /// Create a new model wrapper
    pub fn new(path: impl AsRef<std::path::Path>, config: &AiConfig) -> Result<Self, ModelError> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(ModelError::NotFound(path.display().to_string()));
        }

        let _file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            path,
            name,
            _config: config.clone(),
            handle: None,
        })
    }

    /// Load the model into memory
    pub fn load(&mut self) -> Result<(), ModelError> {
        // For now, this is a placeholder
        // In a full implementation, this would use llama.cpp bindings

        self.handle = Some(ModelHandle {
            _path: self.path.clone(),
            file_size: std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0),
            is_loaded: true,
        });

        Ok(())
    }

    /// Unload the model
    pub fn unload(&mut self) {
        self.handle = None;
    }

    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        self.handle.as_ref().map(|h| h.is_loaded).unwrap_or(false)
    }

    /// Run inference
    pub async fn inference(&self, _prompt: &str, _max_tokens: usize) -> Result<String, ModelError> {
        // Placeholder implementation
        // In production, this would use llama.cpp bindings

        if !self.is_loaded() {
            return Err(ModelError::InferenceError("Model not loaded".to_string()));
        }

        // For testing, return a placeholder response
        // This simulates what the AI model would return
        Ok(format!(
            r#"```json
{{
  "business_meaning": "占位响应 - 实际需要加载 llama.cpp 模型",
  "trigger_scenario": "AI 模型未配置",
  "confidence": 0.0
}}
```

> Note: This is a placeholder response. To enable AI features:
> 1. Download the model: `flowsight-ai download-model`
> 2. Or place `flowsight-linux-1.3b-q4_k_m.gguf` in `~/.flowsight/models/`"#,
        ))
    }

    /// Get model name
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get model path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get model size in bytes
    pub fn file_size(&self) -> u64 {
        self.handle.as_ref().map(|h| h.file_size).unwrap_or(0)
    }
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model name
    pub name: String,
    /// Model size (human readable)
    pub size_human: String,
    /// File size in bytes
    pub file_size: u64,
    /// Whether it's loaded
    pub is_loaded: bool,
    /// Model format
    pub format: String,
    /// Quantization type
    pub quantization: String,
}

impl ModelMetadata {
    /// Create from model
    pub fn from_model(model: &LocalModel) -> Self {
        let size_human = if model.file_size() > 1024 * 1024 * 1024 {
            format!(
                "{:.1} GB",
                model.file_size() as f64 / (1024.0 * 1024.0 * 1024.0)
            )
        } else if model.file_size() > 1024 * 1024 {
            format!("{:.1} MB", model.file_size() as f64 / (1024.0 * 1024.0))
        } else {
            format!("{} KB", model.file_size() / 1024)
        };

        let format = model
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string();

        let quantization = model
            .name()
            .split('-')
            .last()
            .unwrap_or("unknown")
            .to_string();

        Self {
            name: model.name(),
            size_human,
            file_size: model.file_size(),
            is_loaded: model.is_loaded(),
            format,
            quantization,
        }
    }
}

/// Model repository
#[derive(Debug)]
pub struct ModelRepository {
    /// Local model directory
    model_dir: PathBuf,
}

impl ModelRepository {
    /// Create new repository
    pub fn new(model_dir: impl AsRef<std::path::Path>) -> Self {
        Self {
            model_dir: model_dir.as_ref().to_path_buf(),
        }
    }

    /// List available models
    pub fn list_models(&self) -> Vec<ModelMetadata> {
        let mut models = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.model_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "gguf" || ext == "bin" || ext == "model" {
                            if let Ok(metadata) = std::fs::metadata(&path) {
                                let name = path
                                    .file_stem()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();

                                let size_human = if metadata.len() > 1024 * 1024 * 1024 {
                                    format!(
                                        "{:.1} GB",
                                        metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0)
                                    )
                                } else if metadata.len() > 1024 * 1024 {
                                    format!("{:.1} MB", metadata.len() as f64 / (1024.0 * 1024.0))
                                } else {
                                    format!("{} KB", metadata.len() / 1024)
                                };

                                models.push(ModelMetadata {
                                    name,
                                    size_human,
                                    file_size: metadata.len(),
                                    is_loaded: false,
                                    format: ext.to_str().unwrap_or("unknown").to_string(),
                                    quantization: "unknown".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        models
    }

    /// Get default model
    pub fn get_default_model(&self) -> Option<PathBuf> {
        // Check for known model filenames
        let candidates = [
            "flowsight-linux-1.3b-q4_k_m.gguf",
            "flowsight-linux-1.3b.gguf",
            "FlowSight-Linux-1.3B-Q4_K_M.gguf",
            "model.gguf",
        ];

        for candidate in &candidates {
            let path = self.model_dir.join(candidate);
            if path.exists() {
                return Some(path);
            }
        }

        // Return first gguf file
        self.list_models().first().and_then(|m| {
            self.model_dir
                .join(format!("{}.gguf", m.name))
                .exists()
                .then_some(self.model_dir.join(format!("{}.gguf", m.name)))
        })
    }
}
