//! `flowsight ask` — natural language query with LLM
//!
//! Sends analysis context + user question to configured LLM provider.

use crate::context::AnalysisContext;
use anyhow::Result;
use crossterm::style::{Color, Stylize};
use flowsight_cfg::{CfgBuilder, ErrorPathDetector};
use flowsight_llm::config::LlmConfig;
use flowsight_llm::providers::registry::ProviderRegistry;
use flowsight_llm::types::*;
use futures::StreamExt;
use std::io::Write;

const C_TITLE: Color = Color::Rgb { r: 140, g: 185, b: 165 };
const C_DIM: Color = Color::Rgb { r: 110, g: 115, b: 120 };
const C_ERR: Color = Color::Rgb { r: 195, g: 120, b: 120 };

/// Options for the ask command
pub struct AskOptions {
    /// Provider name override
    pub provider: Option<String>,
    /// Model name override
    #[allow(dead_code)]
    pub model: Option<String>,
    /// Source file for context
    pub file: Option<std::path::PathBuf>,
    /// Function for focused context
    pub function: Option<String>,
    /// Disable streaming (wait for full response)
    pub no_stream: bool,
}

/// Run the `ask` command
pub fn run(
    query: &str,
    llm_config: &LlmConfig,
    opts: &AskOptions,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_async(query, llm_config, opts))
}

async fn run_async(
    query: &str,
    llm_config: &LlmConfig,
    opts: &AskOptions,
) -> Result<()> {
    let mut registry = ProviderRegistry::new(llm_config.clone());
    let provider = registry
        .get(opts.provider.as_deref())
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Build context from file if provided
    let system_prompt = build_system_prompt(opts);
    let user_message = build_user_message(query, opts);

    let request = CompletionRequest {
        system: Some(system_prompt),
        messages: vec![Message {
            role: Role::User,
            content: user_message,
        }],
        temperature: llm_config.temperature,
        max_tokens: llm_config.max_tokens,
    };

    eprintln!(
        "{} ({} / {})",
        "Analyzing...".with(C_DIM),
        provider.name().with(C_TITLE),
        provider.model().with(C_DIM),
    );
    eprintln!();

    if opts.no_stream {
        // Non-streaming mode
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
        // Streaming mode
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

/// Build system prompt with FlowSight context
fn build_system_prompt(opts: &AskOptions) -> String {
    let mut prompt = String::from(
        "You are a Linux kernel code analysis expert. You help developers understand \
         execution flows, error handling patterns, and kernel subsystem interactions.\n\n\
         When analyzing code, focus on:\n\
         - Execution flow and call chains\n\
         - Error handling paths (goto cleanup, early returns)\n\
         - Async mechanisms (work queues, timers, IRQ handlers)\n\
         - Locking and execution context (atomic vs process context)\n\
         - Memory management patterns\n\n\
         Be concise and technical. Use function names and line numbers when relevant.",
    );

    // Add file context if available
    if let Some(ref file) = opts.file {
        if let Ok(source) = std::fs::read_to_string(file) {
            let filename = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Add analysis context
            if let Some(ref func_name) = opts.function {
                // Build CFG for the specific function
                let builder = CfgBuilder::new();
                if let Ok(mut cfg) = builder.build_function_cfg(&source, func_name) {
                    ErrorPathDetector::analyze(&mut cfg);
                    let stats = cfg.stats();

                    prompt.push_str(&format!(
                        "\n\nAnalysis context for {}() in {}:\n\
                         - {} basic blocks, {} edges, {} error paths\n\
                         - Calls: {} always, {} conditional, {} error-path\n",
                        func_name,
                        filename,
                        stats.block_count,
                        stats.edge_count,
                        stats.error_path_count,
                        stats.always_calls,
                        stats.conditional_calls,
                        stats.error_calls,
                    ));

                    // Add error path details
                    if !cfg.error_paths.is_empty() {
                        prompt.push_str("- Error paths:\n");
                        for ep in &cfg.error_paths {
                            prompt.push_str(&format!(
                                "  L{}: {} -> {:?}\n",
                                ep.check_line, ep.check_expression, ep.strategy
                            ));
                        }
                    }
                }

                // Extract function source
                let mut ctx = AnalysisContext::new();
                if let Ok(result) = ctx.analyze_file(file) {
                    if let Some(func) = result.parse_result.functions.get(func_name) {
                        if let Some(loc) = &func.location {
                            let lines: Vec<&str> = source.lines().collect();
                            let start = (loc.line as usize).saturating_sub(1);
                            let end = (loc.end_line as usize).min(lines.len());
                            let snippet: String = lines[start..end].join("\n");
                            // Limit to ~200 lines
                            let truncated = if snippet.lines().count() > 200 {
                                format!(
                                    "{}\n... ({} more lines)",
                                    snippet.lines().take(200).collect::<Vec<_>>().join("\n"),
                                    snippet.lines().count() - 200
                                )
                            } else {
                                snippet
                            };
                            prompt.push_str(&format!(
                                "\nSource code of {}():\n```c\n{}\n```\n",
                                func_name, truncated
                            ));
                        }
                    }
                }
            } else {
                // Add file-level summary
                let mut ctx = AnalysisContext::new();
                if let Ok(result) = ctx.analyze_file(file) {
                    prompt.push_str(&format!(
                        "\n\nFile: {} ({} functions, {} async handlers, {} entry points)\n",
                        filename,
                        result.parse_result.functions.len(),
                        result.analysis.async_bindings.len(),
                        result.analysis.entry_points.len(),
                    ));
                }
            }
        }
    }

    prompt
}

/// Build user message with the query
fn build_user_message(query: &str, _opts: &AskOptions) -> String {
    query.to_string()
}

/// List configured providers
pub fn run_list_providers(llm_config: &LlmConfig) -> Result<()> {
    let registry = ProviderRegistry::new(llm_config.clone());
    let providers = registry.list_providers();
    let default = registry.default_name().unwrap_or("(none)");

    println!("{}", "Configured LLM Providers:".with(C_TITLE));
    println!();

    for (name, ptype, model) in &providers {
        let is_default = *name == default;
        let marker = if is_default { " (default)" } else { "" };
        println!(
            "  {}{} ({} / {})",
            name.with(C_TITLE),
            marker.with(C_DIM),
            ptype.with(C_DIM),
            model.with(C_DIM),
        );
    }

    if providers.is_empty() {
        println!("  {}", "(none configured)".with(C_DIM));
    }

    Ok(())
}

/// Test provider connectivity
pub fn run_test_provider(llm_config: &LlmConfig, provider_name: Option<&str>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut registry = ProviderRegistry::new(llm_config.clone());
        let provider = registry
            .get(provider_name)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        eprint!(
            "Testing {} ({})... ",
            provider.name().with(C_TITLE),
            provider.model().with(C_DIM),
        );

        match provider.health_check().await {
            Ok(true) => {
                eprintln!("{}", "OK".with(Color::Rgb { r: 130, g: 175, b: 140 }));
            }
            Ok(false) => {
                eprintln!("{}", "FAILED".with(C_ERR));
            }
            Err(e) => {
                eprintln!("{} ({})", "FAILED".with(C_ERR), e);
            }
        }

        Ok(())
    })
}
