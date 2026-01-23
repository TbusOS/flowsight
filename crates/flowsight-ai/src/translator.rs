//! Condition Translator
//!
//! Translates symbolic execution constraints to business semantics.

use super::{AiTask, FlowSightAi, AiConfig, AiResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of constraint translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    /// Original constraint expression
    pub original: String,
    /// Business meaning in Chinese
    pub business_meaning: String,
    /// Trigger scenario description
    pub trigger_scenario: String,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    /// Source of translation
    pub source: TranslationSource,
    /// Related knowledge base entries
    pub related_patterns: Vec<String>,
}

/// Source of translation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TranslationSource {
    /// From AI model inference
    AiModel,
    /// From knowledge base lookup
    KnowledgeBase,
    /// From heuristic rules
    Heuristic,
    /// User provided
    User,
}

/// Common constraint patterns with their business meanings
const CONSTRAINT_PATTERNS: &[(&str, &str, &str)] = &[
    // NULL checks
    (r"ptr\s*==\s*NULL", "指针为空", "指针未分配内存或指向无效地址"),
    (r"ptr\s*!=\s*NULL", "指针非空", "指针已成功分配内存"),
    (r"!ptr", "指针为空", "指针未初始化或已释放"),

    // Error codes
    (r"ret\s*==\s*-?\d+", "返回值检查", "根据返回值判断操作结果"),
    (r"ret\s*<\s*0", "返回错误", "操作执行失败"),
    (r"ret\s*>=\s*0", "返回成功", "操作执行成功"),
    (r"errno\s*==\s*\w+", "错误码检查", "根据错误码判断具体失败原因"),

    // Buffer checks
    (r"len\s*>\s*0", "长度大于零", "缓冲区大小有效"),
    (r"len\s*<=\s*\d+", "长度限制", "缓冲区大小在允许范围内"),
    (r"size\s*>\s*sizeof", "大小检查", "分配的内存足够容纳数据"),

    // Index checks
    (r"index\s*<\s*\w+_count", "索引有效", "索引在数组范围内"),
    (r"index\s*>=\s*0", "索引非负", "索引值有效"),
    (r"i\s*<\s*n", "循环条件", "循环继续执行"),

    // USB specific
    (r"dev\s*==\s*NULL", "设备为空", "USB 设备未连接或初始化失败"),
    (r"interface\s*!=\s*NULL", "接口有效", "USB 接口已获取"),
    (r"urb\s*!=\s*NULL", "URB 有效", "USB 请求块已分配"),

    // File operations
    (r"fd\s*<\s*0", "文件描述符无效", "文件打开失败"),
    (r"file\s*!=\s*NULL", "文件有效", "文件已成功打开"),

    // Memory allocation
    (r"ptr\s*==\s*NULL", "分配失败", "内存分配请求未被满足"),
    (r"size\s*==\s*0", "零大小分配", "请求分配零字节内存"),

    // Mutex/spinlock
    (r"lock\s*!=\s*NULL", "锁有效", "锁对象已初始化"),
    (r"down_interruptible", "可中断等待", "等待锁时可被信号中断"),
];

/// Condition translator
#[derive(Debug)]
pub struct ConditionTranslator {
    /// AI instance for complex translations
    ai: FlowSightAi,
    /// Cache of recent translations
    cache: HashMap<String, TranslationResult>,
    /// Maximum cache size
    cache_size: usize,
}

impl ConditionTranslator {
    /// Create new translator
    pub fn new(ai_config: AiConfig) -> Self {
        Self {
            ai: FlowSightAi::new(ai_config),
            cache: HashMap::new(),
            cache_size: 100,
        }
    }

    /// Initialize the translator
    pub async fn initialize(&mut self) -> Result<(), anyhow::Error> {
        self.ai.initialize().await?;
        Ok(())
    }

    /// Translate a constraint
    pub async fn translate(
        &mut self,
        constraint: &str,
        code_context: &str,
        function: &str,
    ) -> Result<TranslationResult, anyhow::Error> {
        // Check cache first
        if let Some(cached) = self.cache.get(constraint) {
            return Ok(cached.clone());
        }

        // Try heuristic first (fast path)
        if let Some(result) = self.translate_heuristic(constraint) {
            self.cache_result(constraint.to_string(), result.clone());
            return Ok(result);
        }

        // Fall back to AI translation
        let ai_result = self.translate_ai(constraint, code_context, function).await?;

        self.cache_result(constraint.to_string(), ai_result.clone());
        Ok(ai_result)
    }

    /// Translate using heuristic rules
    fn translate_heuristic(&self, constraint: &str) -> Option<TranslationResult> {
        for (pattern, meaning, scenario) in CONSTRAINT_PATTERNS {
            if regex::Regex::new(pattern)
                .ok()?
                .is_match(constraint)
            {
                return Some(TranslationResult {
                    original: constraint.to_string(),
                    business_meaning: meaning.to_string(),
                    trigger_scenario: scenario.to_string(),
                    confidence: 0.9,
                    source: TranslationSource::Heuristic,
                    related_patterns: vec![pattern.to_string()],
                });
            }
        }

        None
    }

    /// Translate using AI model
    async fn translate_ai(
        &mut self,
        constraint: &str,
        code_context: &str,
        function: &str,
    ) -> Result<TranslationResult, anyhow::Error> {
        let task = AiTask::TranslateCondition {
            code: code_context.to_string(),
            constraint: constraint.to_string(),
            function: function.to_string(),
        };

        let result = self.ai.infer(task).await?;

        // Parse JSON response
        self.parse_translation_response(&result.response)
    }

    /// Parse AI response into TranslationResult
    fn parse_translation_response(&self, response: &str) -> Result<TranslationResult, anyhow::Error> {
        // Extract JSON from response
        let json_start = response.find("{");
        let json_end = response.rfind("}");

        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &response[start..=end];
            let parsed: serde_json::Value = serde_json::from_str(json_str)?;

            return Ok(TranslationResult {
                original: parsed["original"].as_str().unwrap_or("").to_string(),
                business_meaning: parsed["business_meaning"].as_str().unwrap_or("未知").to_string(),
                trigger_scenario: parsed["trigger_scenario"].as_str().unwrap_or("").to_string(),
                confidence: parsed["confidence"].as_f64().unwrap_or(0.5),
                source: TranslationSource::AiModel,
                related_patterns: Vec::new(),
            });
        }

        // Fallback: use response as is
        Ok(TranslationResult {
            original: response.to_string(),
            business_meaning: response.to_string(),
            trigger_scenario: "".to_string(),
            confidence: 0.3,
            source: TranslationSource::AiModel,
            related_patterns: Vec::new(),
        })
    }

    /// Cache a translation result
    fn cache_result(&mut self, constraint: String, result: TranslationResult) {
        if self.cache.len() >= self.cache_size {
            // Remove oldest entry
            if let Some(key) = self.cache.keys().next().cloned() {
                self.cache.remove(&key);
            }
        }
        self.cache.insert(constraint, result);
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

/// Batch translation helper
pub async fn translate_batch(
    translator: &mut ConditionTranslator,
    constraints: &[(&str, &str, &str)],
) -> Vec<(String, Result<TranslationResult, anyhow::Error>)> {
    let mut results = Vec::new();

    for (constraint, context, function) in constraints {
        let result = translator.translate(constraint, context, function).await;
        results.push((constraint.to_string(), result));
    }

    results
}
