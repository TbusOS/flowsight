//! LLM provider configuration
//!
//! Loaded from `[llm]` section in `.flowsight.toml` or environment variables.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level LLM configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Default provider name
    #[serde(default)]
    pub default_provider: Option<String>,
    /// Temperature for all providers (overridable per-provider)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Max tokens (overridable per-provider)
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Enable streaming by default
    #[serde(default = "default_stream")]
    pub stream: bool,
    /// Provider configurations
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

fn default_temperature() -> f32 {
    0.3
}

fn default_stream() -> bool {
    true
}

/// Configuration for a single LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type: "openai", "anthropic", "ollama", "custom"
    #[serde(default = "default_provider_type")]
    pub provider_type: String,
    /// API key (read from env var named here, never stored as plaintext)
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// API key value (only for runtime, never serialize)
    #[serde(skip)]
    pub api_key: Option<String>,
    /// Model ID
    pub model: String,
    /// Base URL (for OpenAI-compatible or custom endpoints)
    #[serde(default)]
    pub base_url: Option<String>,
    /// Ollama endpoint (shorthand for base_url)
    #[serde(default)]
    pub endpoint: Option<String>,
    /// Provider-specific temperature override
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Provider-specific max_tokens override
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

fn default_provider_type() -> String {
    "openai".to_string()
}

impl ProviderConfig {
    /// Resolve the API key from environment variable
    pub fn resolve_api_key(&self) -> Option<String> {
        if let Some(ref key) = self.api_key {
            return Some(key.clone());
        }
        if let Some(ref env_var) = self.api_key_env {
            return std::env::var(env_var).ok();
        }
        None
    }

    /// Get the effective base URL
    pub fn effective_base_url(&self) -> String {
        if let Some(ref url) = self.base_url {
            if !url.is_empty() {
                return url.clone();
            }
        }
        if let Some(ref endpoint) = self.endpoint {
            return endpoint.clone();
        }
        // Defaults based on provider type
        match self.provider_type.as_str() {
            "anthropic" => "https://api.anthropic.com".to_string(),
            "ollama" => "http://localhost:11434".to_string(),
            _ => "https://api.openai.com".to_string(),
        }
    }
}

impl LlmConfig {
    /// Create config with common provider presets
    pub fn with_defaults() -> Self {
        let mut providers = HashMap::new();

        providers.insert("openai".to_string(), ProviderConfig {
            provider_type: "openai".to_string(),
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            api_key: None,
            model: "gpt-4o".to_string(),
            base_url: None,
            endpoint: None,
            temperature: None,
            max_tokens: None,
        });

        providers.insert("claude".to_string(), ProviderConfig {
            provider_type: "anthropic".to_string(),
            api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
            api_key: None,
            model: "claude-sonnet-4-20250514".to_string(),
            base_url: None,
            endpoint: None,
            temperature: None,
            max_tokens: None,
        });

        providers.insert("ollama".to_string(), ProviderConfig {
            provider_type: "ollama".to_string(),
            api_key_env: None,
            api_key: None,
            model: "llama3.2".to_string(),
            base_url: None,
            endpoint: Some("http://localhost:11434".to_string()),
            temperature: None,
            max_tokens: None,
        });

        Self {
            default_provider: Some("openai".to_string()),
            temperature: 0.3,
            max_tokens: Some(4096),
            stream: true,
            providers,
        }
    }

    /// Get provider config by name, falling back to default
    pub fn get_provider(&self, name: Option<&str>) -> Option<&ProviderConfig> {
        let provider_name = name
            .or(self.default_provider.as_deref())?;
        self.providers.get(provider_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let config = LlmConfig::with_defaults();
        assert_eq!(config.temperature, 0.3);
        assert!(config.providers.contains_key("openai"));
        assert!(config.providers.contains_key("claude"));
        assert!(config.providers.contains_key("ollama"));
    }

    #[test]
    fn test_resolve_api_key_from_env() {
        let config = ProviderConfig {
            provider_type: "openai".to_string(),
            api_key_env: Some("FLOWSIGHT_TEST_KEY_NONEXISTENT".to_string()),
            api_key: None,
            model: "gpt-4o".to_string(),
            base_url: None,
            endpoint: None,
            temperature: None,
            max_tokens: None,
        };
        // Should return None since env var doesn't exist
        assert!(config.resolve_api_key().is_none());
    }

    #[test]
    fn test_effective_base_url() {
        let openai = ProviderConfig {
            provider_type: "openai".to_string(),
            api_key_env: None,
            api_key: None,
            model: "gpt-4o".to_string(),
            base_url: None,
            endpoint: None,
            temperature: None,
            max_tokens: None,
        };
        assert_eq!(openai.effective_base_url(), "https://api.openai.com");

        let ollama = ProviderConfig {
            provider_type: "ollama".to_string(),
            api_key_env: None,
            api_key: None,
            model: "llama3.2".to_string(),
            base_url: None,
            endpoint: Some("http://localhost:11434".to_string()),
            temperature: None,
            max_tokens: None,
        };
        assert_eq!(ollama.effective_base_url(), "http://localhost:11434");

        let custom = ProviderConfig {
            provider_type: "openai".to_string(),
            api_key_env: None,
            api_key: None,
            model: "deepseek-chat".to_string(),
            base_url: Some("https://api.deepseek.com".to_string()),
            endpoint: None,
            temperature: None,
            max_tokens: None,
        };
        assert_eq!(custom.effective_base_url(), "https://api.deepseek.com");
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = LlmConfig::with_defaults();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: LlmConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.temperature, config.temperature);
        assert_eq!(parsed.providers.len(), config.providers.len());
    }
}
