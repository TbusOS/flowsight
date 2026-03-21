//! Provider registry — creates and manages provider instances

use crate::config::{LlmConfig, ProviderConfig};
use crate::providers::anthropic::AnthropicProvider;
use crate::providers::openai_compat::OpenAiCompatProvider;
use crate::types::{LlmError, LlmProvider};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry that creates and caches provider instances
pub struct ProviderRegistry {
    config: LlmConfig,
    cache: HashMap<String, Arc<dyn LlmProvider>>,
}

impl ProviderRegistry {
    /// Create from LLM config
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
        }
    }

    /// Get or create a provider by name
    pub fn get(&mut self, name: Option<&str>) -> Result<Arc<dyn LlmProvider>, LlmError> {
        let provider_name = name
            .or(self.config.default_provider.as_deref())
            .ok_or_else(|| LlmError::NotConfigured(
                "No default provider configured. Set llm.default_provider in .flowsight.toml or use --provider flag.".to_string(),
            ))?
            .to_string();

        // Return cached instance
        if let Some(cached) = self.cache.get(&provider_name) {
            return Ok(Arc::clone(cached));
        }

        // Create new instance
        let provider_config = self
            .config
            .providers
            .get(&provider_name)
            .ok_or_else(|| LlmError::NotConfigured(format!(
                "Provider '{}' not configured. Available: {:?}",
                provider_name,
                self.config.providers.keys().collect::<Vec<_>>()
            )))?;

        let provider = create_provider(&provider_name, provider_config)?;
        self.cache.insert(provider_name, Arc::clone(&provider));
        Ok(provider)
    }

    /// List all configured provider names
    pub fn list_providers(&self) -> Vec<(&str, &str, &str)> {
        self.config
            .providers
            .iter()
            .map(|(name, cfg)| {
                (name.as_str(), cfg.provider_type.as_str(), cfg.model.as_str())
            })
            .collect()
    }

    /// Get default provider name
    pub fn default_name(&self) -> Option<&str> {
        self.config.default_provider.as_deref()
    }
}

/// Create a provider instance from config
fn create_provider(
    name: &str,
    config: &ProviderConfig,
) -> Result<Arc<dyn LlmProvider>, LlmError> {
    match config.provider_type.as_str() {
        "anthropic" | "claude" => {
            let provider = AnthropicProvider::from_config(config)?;
            Ok(Arc::new(provider))
        }
        "openai" | "deepseek" | "lmstudio" | "custom" | "ollama" => {
            // All OpenAI-compatible endpoints use the same provider
            let provider = OpenAiCompatProvider::from_config(name, config)?;
            Ok(Arc::new(provider))
        }
        other => Err(LlmError::NotConfigured(format!(
            "Unknown provider type: '{}'. Supported: openai, anthropic, ollama, deepseek, lmstudio, custom",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_providers() {
        let config = LlmConfig::with_defaults();
        let registry = ProviderRegistry::new(config);

        let providers = registry.list_providers();
        assert!(providers.len() >= 3, "Should have openai, claude, ollama");

        let names: Vec<&str> = providers.iter().map(|(n, _, _)| *n).collect();
        assert!(names.contains(&"openai"));
        assert!(names.contains(&"claude"));
        assert!(names.contains(&"ollama"));
    }

    #[test]
    fn test_get_nonexistent_provider() {
        let config = LlmConfig::with_defaults();
        let mut registry = ProviderRegistry::new(config);

        let result = registry.get(Some("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_openai_provider() {
        let config = LlmConfig::with_defaults();
        let mut registry = ProviderRegistry::new(config);

        // Should create successfully (doesn't need valid API key for creation)
        let provider = registry.get(Some("openai"));
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "openai");
    }

    #[test]
    fn test_create_ollama_provider() {
        let config = LlmConfig::with_defaults();
        let mut registry = ProviderRegistry::new(config);

        let provider = registry.get(Some("ollama"));
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "ollama");
    }
}
