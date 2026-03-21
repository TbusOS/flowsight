//! Anthropic Claude API provider
//!
//! Uses the Messages API (https://docs.anthropic.com/en/api/messages)

use crate::config::ProviderConfig;
use crate::types::*;
use async_trait::async_trait;
use futures::StreamExt;
use std::pin::Pin;

/// Provider for Anthropic Claude API
pub struct AnthropicProvider {
    client: reqwest::Client,
    config: ProviderConfig,
}

impl AnthropicProvider {
    pub fn from_config(config: &ProviderConfig) -> Result<Self, LlmError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| LlmError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    fn api_url(&self) -> String {
        let base = self.config.effective_base_url();
        format!("{}/v1/messages", base.trim_end_matches('/'))
    }

    fn build_request_body(
        &self,
        request: &CompletionRequest,
        stream: bool,
    ) -> serde_json::Value {
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .filter(|m| m.role != Role::System) // System handled separately
            .map(|msg| {
                serde_json::json!({
                    "role": match msg.role {
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::System => "user", // shouldn't reach here
                    },
                    "content": msg.content,
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "stream": stream,
        });

        // Anthropic uses top-level "system" field
        if let Some(ref system) = request.system {
            body["system"] = serde_json::json!(system);
        }

        // Temperature (Anthropic accepts 0-1)
        if request.temperature > 0.0 {
            body["temperature"] = serde_json::json!(request.temperature);
        }

        body
    }

    fn get_api_key(&self) -> Result<String, LlmError> {
        self.config
            .resolve_api_key()
            .ok_or_else(|| LlmError::AuthError(
                "Anthropic API key not found. Set ANTHROPIC_API_KEY environment variable.".to_string(),
            ))
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let api_key = self.get_api_key()?;
        let body = self.build_request_body(request, false);
        let url = self.api_url();

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        let response_text = response
            .text()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if status == 401 {
            return Err(LlmError::AuthError("Invalid Anthropic API key.".to_string()));
        }
        if status == 429 {
            return Err(LlmError::RateLimited { retry_after_secs: 60 });
        }
        if status >= 400 {
            return Err(LlmError::ApiError { status, message: response_text });
        }

        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| LlmError::InvalidResponse(format!("JSON parse error: {}", e)))?;

        // Anthropic returns content as array of content blocks
        let content = json["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|block| block["text"].as_str())
            .unwrap_or("")
            .to_string();

        let usage = json.get("usage").map(|u| TokenUsage {
            prompt_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (u["input_tokens"].as_u64().unwrap_or(0)
                + u["output_tokens"].as_u64().unwrap_or(0)) as u32,
        });

        Ok(CompletionResponse {
            content,
            model: json["model"].as_str().unwrap_or(&self.config.model).to_string(),
            usage,
        })
    }

    async fn stream(
        &self,
        request: &CompletionRequest,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<StreamChunk, LlmError>> + Send>>, LlmError>
    {
        let api_key = self.get_api_key()?;
        let body = self.build_request_body(request, true);
        let url = self.api_url();

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        if status >= 400 {
            let text = response.text().await.map_err(|e| LlmError::RequestFailed(e.to_string()))?;
            return Err(LlmError::ApiError { status, message: text });
        }

        let byte_stream = response.bytes_stream();

        let stream = byte_stream.map(|chunk| {
            let bytes = chunk.map_err(|e| LlmError::StreamError(e.to_string()))?;
            let text = String::from_utf8_lossy(&bytes);

            for line in text.lines() {
                let line = line.trim();
                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        let event_type = json["type"].as_str().unwrap_or("");

                        match event_type {
                            "content_block_delta" => {
                                let delta = json["delta"]["text"]
                                    .as_str()
                                    .unwrap_or("")
                                    .to_string();
                                return Ok(StreamChunk { delta, done: false });
                            }
                            "message_stop" => {
                                return Ok(StreamChunk {
                                    delta: String::new(),
                                    done: true,
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }

            Ok(StreamChunk {
                delta: String::new(),
                done: false,
            })
        });

        Ok(Box::pin(stream))
    }

    async fn health_check(&self) -> Result<bool, LlmError> {
        // Anthropic doesn't have a lightweight health endpoint,
        // just verify we have an API key
        self.get_api_key()?;
        Ok(true)
    }
}
