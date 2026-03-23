//! `flowsight review` — AI-powered code review with analysis context
//!
//! Sends source code + CFG + AQS + error paths to LLM for code review.
//! Injects FlowSight's static analysis context to produce deeper reviews
//! than generic LLM code review.

use crate::context::AnalysisContext;
use anyhow::Result;
use crossterm::style::{Color, Stylize};
use flowsight_cfg::{CfgBuilder, ErrorPathDetector};
use flowsight_evolve::aqs::AnalysisQualityScore;
use flowsight_llm::config::LlmConfig;
use flowsight_llm::providers::registry::ProviderRegistry;
use flowsight_llm::types::*;
use futures::StreamExt;
use std::io::Write;
use std::path::Path;

const C_TITLE: Color = Color::Rgb {
    r: 140,
    g: 185,
    b: 165,
};
const C_DIM: Color = Color::Rgb {
    r: 110,
    g: 115,
    b: 120,
};
const C_ERR: Color = Color::Rgb {
    r: 195,
    g: 120,
    b: 120,
};
const C_FILE: Color = Color::Rgb {
    r: 155,
    g: 160,
    b: 185,
};

/// Options for the review command
pub struct ReviewOptions {
    /// Provider name override
    pub provider: Option<String>,
    /// Specific function to review (otherwise reviews entire file)
    pub function: Option<String>,
    /// Disable streaming
    pub no_stream: bool,
    /// Focus area: "security", "error-handling", "performance", or "all"
    pub focus: String,
}

/// Run the review command
pub fn run(
    file: &Path,
    llm_config: &LlmConfig,
    opts: &ReviewOptions,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_async(file, llm_config, opts))
}

async fn run_async(
    file: &Path,
    llm_config: &LlmConfig,
    opts: &ReviewOptions,
) -> Result<()> {
    let mut registry = ProviderRegistry::new(llm_config.clone());
    let provider = registry
        .get(opts.provider.as_deref())
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let (system_prompt, user_message) = build_review_prompts(file, opts)?;

    let request = CompletionRequest {
        system: Some(system_prompt),
        messages: vec![Message {
            role: Role::User,
            content: user_message,
        }],
        temperature: llm_config.temperature,
        max_tokens: llm_config.max_tokens,
    };

    let filename = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    eprintln!(
        "{} {} ({} / {})",
        "Reviewing".with(C_DIM),
        filename.with(C_FILE),
        provider.name().with(C_TITLE),
        provider.model().with(C_DIM),
    );
    eprintln!();

    if opts.no_stream {
        match provider.complete(&request).await {
            Ok(response) => {
                println!("{}", response.content);
                if let Some(usage) = &response.usage {
                    eprintln!();
                    eprintln!(
                        "{}",
                        format!(
                            "tokens: {} prompt + {} completion = {} total",
                            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                        )
                        .with(C_DIM)
                    );
                }
            }
            Err(e) => {
                eprintln!("{} {}", "Error:".with(C_ERR), e);
                return Err(anyhow::anyhow!("{}", e));
            }
        }
    } else {
        match provider.stream(&request).await {
            Ok(mut stream) => {
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(c) => {
                            if !c.delta.is_empty() {
                                print!("{}", c.delta);
                                std::io::stdout().flush().ok();
                            }
                            if c.done {
                                println!();
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("\n{} {}", "Stream error:".with(C_ERR), e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{} {}", "Error:".with(C_ERR), e);
                return Err(anyhow::anyhow!("{}", e));
            }
        }
    }

    Ok(())
}

/// Build system prompt and user message for code review
fn build_review_prompts(
    file: &Path,
    opts: &ReviewOptions,
) -> Result<(String, String)> {
    let source = std::fs::read_to_string(file)?;
    let filename = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Parse and analyze the file
    let mut ctx = AnalysisContext::new();
    let file_analysis = ctx.analyze_file(file)?;
    let kb = ctx.knowledge_base();

    // Compute AQS
    let aqs = AnalysisQualityScore::compute(
        &source,
        &file_analysis.parse_result,
        &file_analysis.analysis,
        kb,
    );

    // Build system prompt with analysis context
    let focus_instructions = match opts.focus.as_str() {
        "security" => {
            "Focus on SECURITY issues: buffer overflows, integer overflows, \
             use-after-free, double-free, null pointer dereferences, TOCTOU races, \
             missing bounds checks, unvalidated user input."
        }
        "error-handling" => {
            "Focus on ERROR HANDLING: missing error checks, resource leaks on error paths, \
             goto cleanup patterns, return value checking, proper unwinding."
        }
        "performance" => {
            "Focus on PERFORMANCE: unnecessary allocations, lock contention, \
             cache-unfriendly access patterns, redundant operations, hot path optimization."
        }
        _ => {
            "Review comprehensively: correctness, error handling, security, \
             performance, kernel coding style, and API usage."
        }
    };

    let system_prompt = format!(
        "You are an expert Linux kernel code reviewer. You review C code with deep \
         knowledge of kernel internals, driver frameworks, and common bug patterns.\n\n\
         {}\n\n\
         Format your review as:\n\
         1. **Summary**: One-line assessment\n\
         2. **Critical Issues**: Must-fix bugs or security problems\n\
         3. **Improvements**: Recommended changes\n\
         4. **Notes**: Minor observations\n\n\
         Reference specific line numbers. Be concise and actionable.",
        focus_instructions
    );

    // Build user message with code + analysis context
    let mut user_msg = String::new();

    // Analysis Quality Score context
    user_msg.push_str(&format!(
        "## Analysis Quality Score: {:.2} ({:.0}%)\n\
         - Direct call resolution: {:.0}% ({}/{})\n\
         - KB coverage: {:.0}% ({}/{})\n\
         - Error path coverage: {:.0}% ({}/{})\n\n",
        aqs.score,
        aqs.score * 100.0,
        aqs.dimensions.direct_call_resolution * 100.0,
        aqs.stats.direct_calls_resolved,
        aqs.stats.total_calls,
        aqs.dimensions.kb_coverage * 100.0,
        aqs.stats.kernel_api_covered,
        aqs.stats.kernel_api_calls,
        aqs.dimensions.error_path_coverage * 100.0,
        aqs.stats.error_paths_detected,
        aqs.stats.functions_with_error_potential,
    ));

    // If reviewing a specific function, add CFG context
    if let Some(ref func_name) = opts.function {
        let builder = CfgBuilder::new();
        if let Ok(mut cfg) = builder.build_function_cfg(&source, func_name) {
            ErrorPathDetector::analyze(&mut cfg);
            let stats = cfg.stats();

            user_msg.push_str(&format!(
                "## CFG Analysis for {}()\n\
                 - {} basic blocks, {} edges\n\
                 - {} error paths detected\n\
                 - Calls: {} always, {} conditional, {} error-path\n",
                func_name,
                stats.block_count,
                stats.edge_count,
                stats.error_path_count,
                stats.always_calls,
                stats.conditional_calls,
                stats.error_calls,
            ));

            if !cfg.error_paths.is_empty() {
                user_msg.push_str("- Error paths:\n");
                for ep in &cfg.error_paths {
                    user_msg.push_str(&format!(
                        "  L{}: {} -> {:?}\n",
                        ep.check_line, ep.check_expression, ep.strategy
                    ));
                }
            }
            user_msg.push('\n');
        }

        // Extract just the function source
        if let Some(func) = file_analysis.parse_result.functions.get(func_name) {
            if let Some(loc) = &func.location {
                let lines: Vec<&str> = source.lines().collect();
                let start = (loc.line as usize).saturating_sub(1);
                let end = (loc.end_line as usize).min(lines.len());
                let snippet: String = lines[start..end]
                    .iter()
                    .enumerate()
                    .map(|(i, line)| format!("{:>4} {}", start + i + 1, line))
                    .collect::<Vec<_>>()
                    .join("\n");
                user_msg.push_str(&format!(
                    "## Review this function: {}() in {}\n```c\n{}\n```\n",
                    func_name, filename, snippet
                ));
            }
        } else {
            anyhow::bail!("Function '{}' not found in {}", func_name, filename);
        }
    } else {
        // Review entire file (truncate if too long)
        let numbered: String = source
            .lines()
            .enumerate()
            .map(|(i, line)| format!("{:>4} {}", i + 1, line))
            .collect::<Vec<_>>()
            .join("\n");

        let truncated = if numbered.lines().count() > 500 {
            format!(
                "{}\n... ({} more lines, showing first 500)",
                numbered.lines().take(500).collect::<Vec<_>>().join("\n"),
                numbered.lines().count() - 500
            )
        } else {
            numbered
        };

        user_msg.push_str(&format!(
            "## Review this file: {}\n```c\n{}\n```\n",
            filename, truncated
        ));
    }

    Ok((system_prompt, user_msg))
}
