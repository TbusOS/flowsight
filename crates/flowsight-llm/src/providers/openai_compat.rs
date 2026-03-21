//! OpenAI-compatible API provider
//!
//! Supports: OpenAI, DeepSeek, LM Studio, vLLM, Ollama (via /v1), and any
//! server implementing the OpenAI Chat Completions API.

use crate::config::ProviderConfig;
use crate::types::*;
use async_trait::async_trait;
use futures::StreamExt;
use std::pin::Pin;

/// Provider for OpenAI-compatible APIs
pub struct OpenAiCompatProvider {
    client: reqwest::Client,
    config: ProviderConfig,
    provider_name: String,
}

impl OpenAiCompatProvider {
    /// Create from provider config
    pub fn from_config(name: &str, config: &ProviderConfig) -> Result<Self, LlmError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| LlmError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            client,
            config: config.clone(),
            provider_name: name.to_string(),
        })
    }

    fn api_url(&self) -> String {
        let base = self.config.effective_base_url();
        // Ollama uses /api/chat, but also supports /v1/chat/completions
        if base.contains("11434") && !base.contains("/v1") {
            format!("{}/v1/chat/completions", base.trim_end_matches('/'))
        } else {
            format!("{}/v1/chat/completions", base.trim_end_matches('/'))
        }
    }

    fn build_request_body(
        &self,
        request: &CompletionRequest,
        stream: bool,
    ) -> serde_json::Value {
        let mut messages = Vec::new();

        if let Some(ref system) = request.system {
            messages.push(serde_json::json!({
                "role": "system",
                "content": system,
            }));
        }

        for msg in &request.messages {
            messages.push(serde_json::json!({
                "role": match msg.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                "content": msg.content,
            }));
        }

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": request.temperature,
            "stream": stream,
        });

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        body
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatProvider {
    fn name(&self) -> &str {
        &self.provider_name
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let body = self.build_request_body(request, false);
        let url = self.api_url();

        let mut req = self.client.post(&url).json(&body);

        if let Some(api_key) = self.config.resolve_api_key() {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        if status == 401 {
            return Err(LlmError::AuthError(
                "Invalid API key. Check your API key environment variable.".to_string(),
            ));
        }
        if status == 429 {
            return Err(LlmError::RateLimited {
                retry_after_secs: 30,
            });
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if status >= 400 {
            return Err(LlmError::ApiError {
                status,
                message: response_text,
            });
        }

        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| LlmError::InvalidResponse(format!("JSON parse error: {}", e)))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = json.get("usage").map(|u| TokenUsage {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        let model = json["model"]
            .as_str()
            .unwrap_or(&self.config.model)
            .to_string();

        Ok(CompletionResponse {
            content,
            model,
            usage,
        })
    }

    async fn stream(
        &self,
        request: &CompletionRequest,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<StreamChunk, LlmError>> + Send>>, LlmError>
    {
        let body = self.build_request_body(request, true);
        let url = self.api_url();

        let mut req = self.client.post(&url).json(&body);

        if let Some(api_key) = self.config.resolve_api_key() {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        if status >= 400 {
            let text = response
                .text()
                .await
                .map_err(|e| LlmError::RequestFailed(e.to_string()))?;
            return Err(LlmError::ApiError {
                status,
                message: text,
            });
        }

        let byte_stream = response.bytes_stream();

        let stream = byte_stream
            .map(|chunk| {
                let bytes = chunk.map_err(|e| LlmError::StreamError(e.to_string()))?;
                let text = String::from_utf8_lossy(&bytes);
                let mut deltas = Vec::new();

                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() || line == "data: [DONE]" {
                        if line == "data: [DONE]" {
                            deltas.push(StreamChunk {
                                delta: String::new(),
                                done: true,
                            });
                        }
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            let delta = json["choices"][0]["delta"]["content"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            let finish = json["choices"][0]["finish_reason"]
                                .as_str()
                                .is_some();
                            deltas.push(StreamChunk {
                                delta,
                                done: finish,
                            });
                        }
                    }
                }

                // Return the last meaningful delta
                Ok(deltas.into_iter().last().unwrap_or(StreamChunk {
                    delta: String::new(),
                    done: false,
                }))
            });

        Ok(Box::pin(stream))
    }

    async fn health_check(&self) -> Result<bool, LlmError> {
        let base = self.config.effective_base_url();

        // For Ollama, check /api/tags
        if base.contains("11434") {
            let url = format!("{}/api/tags", base.trim_end_matches('/'));
            let resp = self
                .client
                .get(&url)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
                .map_err(|e| LlmError::ConnectionFailed(e.to_string()))?;
            return Ok(resp.status().is_success());
        }

        // For OpenAI-compatible, check /v1/models
        let url = format!("{}/v1/models", base.trim_end_matches('/'));
        let mut req = self.client.get(&url)
            .timeout(std::time::Duration::from_secs(5));

        if let Some(api_key) = self.config.resolve_api_key() {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| LlmError::ConnectionFailed(e.to_string()))?;

        Ok(resp.status().is_success())
    }
}
