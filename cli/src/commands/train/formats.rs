//! Training data JSONL format definitions
//!
//! Supports three output formats:
//! - SFT (Alpaca-style): instruction/input/output
//! - DPO: prompt/chosen/rejected preference pairs
//! - ChatML: OpenAI multi-turn message format

use serde::Serialize;

use super::TrainCategory;

/// Metadata attached to every training example
#[derive(Debug, Clone, Serialize)]
pub struct ExampleMeta {
    /// Source file path
    pub source_file: String,
    /// Function name (if applicable)
    pub function: Option<String>,
    /// Generation category
    pub category: String,
    /// Estimated token count (chars / 4)
    pub estimated_tokens: usize,
}

/// SFT (Supervised Fine-Tuning) example in Alpaca format
#[derive(Debug, Clone, Serialize)]
pub struct SftExample {
    pub instruction: String,
    pub input: String,
    pub output: String,
    pub meta: ExampleMeta,
}

/// DPO (Direct Preference Optimization) example
#[derive(Debug, Clone, Serialize)]
pub struct DpoExample {
    pub prompt: String,
    pub chosen: String,
    pub rejected: String,
    pub meta: ExampleMeta,
}

/// A single chat message for ChatML format
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// ChatML format example
#[derive(Debug, Clone, Serialize)]
pub struct ChatMlExample {
    pub messages: Vec<ChatMessage>,
    pub meta: ExampleMeta,
}

/// Build metadata for a training example
pub fn build_meta(
    source_file: &str,
    function: Option<&str>,
    category: &TrainCategory,
    content_length: usize,
) -> ExampleMeta {
    ExampleMeta {
        source_file: source_file.to_string(),
        function: function.map(String::from),
        category: category.to_string(),
        estimated_tokens: content_length / 4,
    }
}

/// Convert an SFT example to a DPO example by generating a shallow rejected answer
pub fn sft_to_dpo(sft: &SftExample) -> DpoExample {
    let rejected = build_shallow_answer(&sft.instruction);
    DpoExample {
        prompt: format_dpo_prompt(&sft.instruction, &sft.input),
        chosen: sft.output.clone(),
        rejected,
        meta: sft.meta.clone(),
    }
}

/// Convert an SFT example to ChatML format
pub fn sft_to_chatml(sft: &SftExample) -> ChatMlExample {
    let system = ChatMessage {
        role: "system".to_string(),
        content: "You are a Linux kernel expert specializing in code execution flow analysis, \
                  async mechanisms, callback patterns, and driver architecture."
            .to_string(),
    };

    let user_content = if sft.input.is_empty() {
        sft.instruction.clone()
    } else {
        format!("{}\n\n```c\n{}\n```", sft.instruction, sft.input)
    };

    let user = ChatMessage {
        role: "user".to_string(),
        content: user_content,
    };

    let assistant = ChatMessage {
        role: "assistant".to_string(),
        content: sft.output.clone(),
    };

    ChatMlExample {
        messages: vec![system, user, assistant],
        meta: sft.meta.clone(),
    }
}

/// Format instruction + input into a DPO prompt string
fn format_dpo_prompt(instruction: &str, input: &str) -> String {
    if input.is_empty() {
        instruction.to_string()
    } else {
        format!("{}\n\n```c\n{}\n```", instruction, input)
    }
}

/// Generate a shallow/superficial rejected answer for DPO training
fn build_shallow_answer(instruction: &str) -> String {
    format!(
        "This function performs some operations in the Linux kernel. \
         It is called during system operation and does what its name suggests. \
         For more details, refer to the kernel source code. \
         The question was: {}",
        truncate_str(instruction, 80)
    )
}

/// Truncate a string to a maximum character count, adding ellipsis if needed
fn truncate_str(s: &str, max_chars: usize) -> &str {
    if s.len() <= max_chars {
        s
    } else {
        let boundary = s
            .char_indices()
            .nth(max_chars)
            .map(|(idx, _)| idx)
            .unwrap_or(s.len());
        &s[..boundary]
    }
}

/// Serialize any training example to a JSONL line (no trailing newline)
pub fn to_jsonl_line<T: Serialize>(example: &T) -> anyhow::Result<String> {
    serde_json::to_string(example).map_err(|e| anyhow::anyhow!("JSON serialization failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_meta_estimates_tokens() {
        let meta = build_meta("test.c", Some("foo"), &TrainCategory::Flow, 400);
        assert_eq!(meta.estimated_tokens, 100);
        assert_eq!(meta.category, "flow");
    }

    #[test]
    fn test_sft_to_dpo_preserves_chosen() {
        let sft = SftExample {
            instruction: "Explain foo".to_string(),
            input: "void foo() {}".to_string(),
            output: "Detailed answer".to_string(),
            meta: build_meta("t.c", Some("foo"), &TrainCategory::Flow, 100),
        };
        let dpo = sft_to_dpo(&sft);
        assert_eq!(dpo.chosen, "Detailed answer");
        assert!(dpo.rejected.contains("some operations"));
    }

    #[test]
    fn test_sft_to_chatml_has_three_messages() {
        let sft = SftExample {
            instruction: "Explain bar".to_string(),
            input: String::new(),
            output: "Answer".to_string(),
            meta: build_meta("t.c", None, &TrainCategory::Async, 50),
        };
        let chatml = sft_to_chatml(&sft);
        assert_eq!(chatml.messages.len(), 3);
        assert_eq!(chatml.messages[0].role, "system");
        assert_eq!(chatml.messages[1].role, "user");
        assert_eq!(chatml.messages[2].role, "assistant");
    }

    #[test]
    fn test_to_jsonl_line_produces_single_line() {
        let sft = SftExample {
            instruction: "Q".to_string(),
            input: "I".to_string(),
            output: "A".to_string(),
            meta: build_meta("x.c", None, &TrainCategory::Patterns, 10),
        };
        let line = to_jsonl_line(&sft).unwrap();
        assert!(!line.contains('\n'));
    }
}
