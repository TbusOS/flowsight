//! Core types for LLM provider interface

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Unified LLM Provider trait
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider name (e.g., "openai", "anthropic", "ollama")
    fn name(&self) -> &str;

    /// Current model ID
    fn model(&self) -> &str;

    /// Send a completion request and return full response
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, LlmError>;

    /// Send a completion request and return streaming response
    async fn stream(
        &self,
        request: &CompletionRequest,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<StreamChunk, LlmError>> + Send>>, LlmError>;

    /// Check if the provider is reachable
    async fn health_check(&self) -> Result<bool, LlmError>;
}

/// Completion request (provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// System prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Conversation messages
    pub messages: Vec<Message>,
    /// Temperature (0.0 - 1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Max tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

fn default_temperature() -> f32 {
    0.3
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            system: None,
            messages: vec![],
            temperature: 0.3,
            max_tokens: None,
        }
    }
}

/// A single message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Generated text
    pub content: String,
    /// Model that generated the response
    pub model: String,
    /// Token usage
    pub usage: Option<TokenUsage>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Streaming chunk
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Incremental text delta
    pub delta: String,
    /// Whether this is the final chunk
    pub done: bool,
}

/// LLM provider error
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("API returned error {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Provider not configured: {0}")]
    NotConfigured(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}
