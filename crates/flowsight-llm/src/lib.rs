#![forbid(unsafe_code)]
//! FlowSight LLM — Unified provider interface for multiple LLM backends
//!
//! Supports:
//! - OpenAI-compatible APIs (GPT, DeepSeek, LM Studio, vLLM, etc.)
//! - Anthropic Claude API
//! - Ollama local models
//! - Custom HTTP endpoints
//!
//! # Architecture
//!
//! ```text
//! flowsight ask "..." → ProviderRegistry → LlmProvider trait → HTTP API
//!                         ├── OpenAiProvider   → api.openai.com
//!                         ├── AnthropicProvider → api.anthropic.com
//!                         ├── OllamaProvider   → localhost:11434
//!                         └── CustomProvider    → user-configured
//! ```

pub mod config;
pub mod providers;
pub mod types;

pub use config::LlmConfig;
pub use providers::registry::ProviderRegistry;
pub use types::*;
