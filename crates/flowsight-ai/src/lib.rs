//! FlowSight Local AI Module
//!
//! Provides local AI model inference for:
//! - Constraint condition translation (symbolic → business meaning)
//! - Business semantics explanation
//! - Execution context annotation
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    AI Inference Engine                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
//! │  │ Condition   │  │ Business    │  │ Execution Context   │  │
//! │  │ Translator  │  │ Explainer   │  │ Annotator           │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────────┘  │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//!                            │
//!                            ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Model Manager                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │  Local Model (~0.8GB GGUF)                           │   │
//! │  │  - FlowSight-Linux-1.3B-Q4_K_M.gguf                 │   │
//! │  │  - Downloaded on first use                           │   │
//! │  │  - Optionally embedded in installer                  │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod explainer;
pub mod formatter;
pub mod model;
pub mod translator;

pub use explainer::{BusinessExplainer, ExecutionContextAnnotator};
pub use formatter::{
    DisplayAsyncPattern, DisplayFlowData, DisplayNode, DisplayStats, FlowFormatter, OutputFormat,
};
pub use model::{LocalModel, ModelMetadata, ModelRepository};
pub use translator::{ConditionTranslator, TranslationResult};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// AI task types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiTask {
    /// Translate constraint conditions to business meaning
    TranslateCondition {
        /// Code context
        code: String,
        /// Constraint expression
        constraint: String,
        /// Function context
        function: String,
    },
    /// Explain business semantics of a function
    ExplainBusiness {
        /// Function code
        code: String,
        /// Function name
        function_name: String,
        /// Execution context
        context: String,
    },
    /// Annotate execution context
    AnnotateContext {
        /// Async mechanism
        mechanism: String,
        /// Handler code
        handler_code: String,
    },
    /// Generate call chain description
    DescribeCallChain {
        /// Call chain nodes
        nodes: Vec<String>,
    },
    /// Answer a specific question about code
    AnswerQuestion {
        /// Question
        question: String,
        /// Code context
        code: String,
    },
}

/// AI inference result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResult {
    /// The generated response
    pub response: String,
    /// Whether the response is complete
    pub is_complete: bool,
    /// Inference time in milliseconds
    pub inference_time_ms: u64,
    /// Model used
    pub model: String,
    /// Token count
    pub token_count: usize,
}

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Model path
    pub model_path: Option<PathBuf>,
    /// Model size (for display)
    pub model_size: String,
    /// Context length
    pub context_length: usize,
    /// Max tokens to generate
    pub max_tokens: usize,
    /// Temperature (0.0 - 1.0)
    pub temperature: f32,
    /// Top-p sampling
    pub top_p: f32,
    /// Top-k sampling
    pub top_k: usize,
    /// Batch size for inference
    pub batch_size: usize,
    /// Use GPU acceleration if available
    pub use_gpu: bool,
    /// Number of GPU layers (for llama.cpp)
    pub gpu_layers: usize,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            model_path: None,
            model_size: "1.3B".into(),
            context_length: 4096,
            max_tokens: 1024,
            temperature: 0.1, // Low temperature for deterministic outputs
            top_p: 0.9,
            top_k: 10,
            batch_size: 1,
            use_gpu: false,
            gpu_layers: 0,
        }
    }
}

impl AiConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set model path
    pub fn with_model_path(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.model_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max: usize) -> Self {
        self.max_tokens = max;
        self
    }

    /// Enable GPU
    pub fn with_gpu(mut self, layers: usize) -> Self {
        self.use_gpu = true;
        self.gpu_layers = layers;
        self
    }
}

/// Main AI interface
#[derive(Debug)]
pub struct FlowSightAi {
    /// Model manager
    model: Option<LocalModel>,
    /// Configuration
    config: AiConfig,
    /// Knowledge base for context
    knowledge_base: Option<PathBuf>,
}

impl FlowSightAi {
    /// Create new AI instance
    pub fn new(config: AiConfig) -> Self {
        Self {
            model: None,
            config,
            knowledge_base: None,
        }
    }

    /// Set knowledge base path
    pub fn with_knowledge_base(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.knowledge_base = Some(path.as_ref().to_path_buf());
        self
    }

    /// Initialize the AI model
    pub async fn initialize(&mut self) -> Result<(), anyhow::Error> {
        // Check if model exists
        let model_path = if let Some(path) = &self.config.model_path {
            path.clone()
        } else {
            get_default_model_path()?
        };

        if !model_path.exists() {
            // Download model
            tracing::info!("Model not found, downloading...");
            download_model(&model_path).await?;
        }

        // Load model
        self.model = Some(LocalModel::new(&model_path, &self.config)?);

        tracing::info!("AI model initialized: {}", model_path.display());
        Ok(())
    }

    /// Check if model is loaded
    pub fn is_ready(&self) -> bool {
        self.model.is_some()
    }

    /// Run inference
    pub async fn infer(&mut self, task: AiTask) -> Result<AiResult, anyhow::Error> {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| anyhow::Error::msg("Model not initialized. Call initialize() first."))?;

        let prompt = self.build_prompt(&task);
        let start = std::time::Instant::now();

        let response = model.inference(&prompt, self.config.max_tokens).await?;
        let token_count = response.split_whitespace().count();

        let elapsed = start.elapsed();

        Ok(AiResult {
            response,
            is_complete: true,
            inference_time_ms: elapsed.as_millis() as u64,
            model: model.name(),
            token_count,
        })
    }

    /// Build prompt for the task
    fn build_prompt(&self, task: &AiTask) -> String {
        match task {
            AiTask::TranslateCondition {
                code,
                constraint,
                function,
            } => {
                format!(
                    r#"### Task: 翻译代码约束条件为业务语义

### 代码:
```
{code}
```

### 约束条件:
{constraint}

### 函数:
{function}

### 输出格式 (JSON):
```json
{{
  "original": "原始条件表达式",
  "business_meaning": "这个条件在业务上代表什么",
  "trigger_scenario": "这个条件满足时会发生什么",
  "confidence": 0.95
}}
```

### Response:"#
                )
            }
            AiTask::ExplainBusiness {
                code,
                function_name,
                context,
            } => {
                format!(
                    r#"### Task: 解释函数的业务语义

### 代码:
```
{code}
```

### 函数名:
{function_name}

### 执行上下文:
{context}

### 输出格式 (JSON):
```json
{{
  "trigger_condition": "函数在什么条件下被调用",
  "business_meaning": "这个函数在业务上做什么",
  "execution_result": "执行这个函数会产生什么结果",
  "related_functions": ["相关的函数列表"],
  "common_errors": ["常见的错误情况"]
}}
```

### Response:"#
                )
            }
            AiTask::AnnotateContext {
                mechanism,
                handler_code,
            } => {
                format!(
                    r#"### Task: 标注异步执行上下文

### 异步机制:
{mechanism}

### Handler 代码:
```
{handler_code}
```

### 输出格式:
- 执行上下文: (进程/软中断/硬中断)
- 是否可睡眠: (是/否)
- 触发时机: 描述何时会调用这个 handler
- 注意事项: 开发时需要注意的点

### Response:"#
                )
            }
            AiTask::DescribeCallChain { nodes } => {
                format!(
                    r#"### Task: 描述调用链

### 调用链节点:
{}

### 描述:
请按时间顺序描述这个调用链，从触发源到最终用户代码。

### Response:"#,
                    nodes
                        .iter()
                        .map(|n| format!("- {}", n))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            AiTask::AnswerQuestion { question, code } => {
                format!(
                    r#"### Question:
{question}

### Code:
```
{code}
```

### Answer:"#
                )
            }
        }
    }
}

/// Get default model path
fn get_default_model_path() -> Result<PathBuf, anyhow::Error> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;

    let model_dir = PathBuf::from(home).join(".flowsight").join("models");

    Ok(model_dir.join("flowsight-linux-1.3b-q4_k_m.gguf"))
}

/// Download model file
async fn download_model(path: &PathBuf) -> Result<(), anyhow::Error> {
    // Create parent directories
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // TODO: Implement actual download from release assets
    // For now, create a placeholder
    tracing::warn!("Model download not implemented yet. Please download manually:");
    tracing::warn!("  URL: https://github.com/TbusOS/flowsight/releases");
    tracing::warn!("  Path: {}", path.display());

    // Create a small placeholder file
    std::fs::write(path, "# Model placeholder - please download from releases")?;

    Ok(())
}
