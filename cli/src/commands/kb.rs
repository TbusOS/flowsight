//! `flowsight kb` command group - knowledge base query and inspection

use crate::context::AnalysisContext;
use crate::output::{json, OutputFormat};
use anyhow::Result;
use std::path::Path;

/// Show knowledge base statistics
pub fn run_stats(format: &OutputFormat) -> Result<()> {
    let ctx = AnalysisContext::new();
    let kb = ctx.knowledge_base();

    let framework_count = kb.frameworks.len();
    let async_count = kb.async_patterns.len();
    let api_count = kb.kernel_apis.len();

    let total_callbacks: usize = kb
        .frameworks
        .values()
        .map(|f| f.callbacks.len())
        .sum();

    let chains_count: usize = kb
        .frameworks
        .values()
        .flat_map(|f| f.callbacks.values())
        .filter(|cb| cb.call_chain.is_some())
        .count();

    match format {
        OutputFormat::Json => {
            let result = serde_json::json!({
                "frameworks": framework_count,
                "callbacks": total_callbacks,
                "call_chains": chains_count,
                "async_patterns": async_count,
                "kernel_apis": api_count,
            });
            println!("{}", json::to_pretty_json(&result)?);
        }
        _ => {
            println!("Knowledge Base Statistics:");
            println!();
            println!("  Frameworks:     {}", framework_count);
            println!("  Callbacks:      {}", total_callbacks);
            println!("  Call chains:    {}", chains_count);
            println!("  Async patterns: {}", async_count);
            println!("  Kernel APIs:    {}", api_count);
            println!();
            println!("Frameworks:");
            let mut frameworks: Vec<_> = kb.frameworks.iter().collect();
            frameworks.sort_by_key(|(name, _)| (*name).clone());
            for (name, fw) in &frameworks {
                println!(
                    "  {:<30} {} callbacks  {}",
                    name,
                    fw.callbacks.len(),
                    fw.description.chars().take(50).collect::<String>()
                );
            }
            println!();
            println!("Async Patterns:");
            let mut patterns: Vec<_> = kb.async_patterns.iter().collect();
            patterns.sort_by_key(|(name, _)| (*name).clone());
            for (name, pat) in &patterns {
                let ctx_str = pat.context.description();
                println!("  {:<20} {}", name, ctx_str);
            }
            println!();
            println!("Kernel APIs: {} registered", api_count);
        }
    }

    Ok(())
}

/// Search knowledge base for a term
pub fn run_query(term: &str, format: &OutputFormat) -> Result<()> {
    let ctx = AnalysisContext::new();
    let kb = ctx.knowledge_base();
    let term_lower = term.to_lowercase();

    let mut found = false;

    // Search frameworks
    for (name, fw) in &kb.frameworks {
        if name.to_lowercase().contains(&term_lower)
            || fw.description.to_lowercase().contains(&term_lower)
        {
            found = true;
            match format {
                OutputFormat::Json => {
                    let result = serde_json::json!({
                        "type": "framework",
                        "name": name,
                        "description": fw.description,
                        "header": fw.header,
                        "callbacks": fw.callbacks.keys().collect::<Vec<_>>(),
                    });
                    println!("{}", json::to_pretty_json(&result)?);
                }
                _ => {
                    println!("[Framework] {}", name);
                    println!("  Description: {}", fw.description);
                    if let Some(ref header) = fw.header {
                        println!("  Header: {}", header);
                    }
                    println!("  Callbacks:");
                    for (cb_name, cb) in &fw.callbacks {
                        let chain_marker = if cb.call_chain.is_some() { " [chain]" } else { "" };
                        println!("    .{:<20} {}{}", cb_name, cb.description, chain_marker);
                    }
                    println!();
                }
            }
        }
    }

    // Search callbacks
    for (fw_name, fw) in &kb.frameworks {
        for (cb_name, cb) in &fw.callbacks {
            if cb_name.to_lowercase().contains(&term_lower)
                || cb.description.to_lowercase().contains(&term_lower)
            {
                if !fw_name.to_lowercase().contains(&term_lower) {
                    found = true;
                    match format {
                        OutputFormat::Json => {
                            let result = serde_json::json!({
                                "type": "callback",
                                "framework": fw_name,
                                "name": cb_name,
                                "description": cb.description,
                                "trigger": cb.trigger,
                                "context": format!("{:?}", cb.context),
                                "has_call_chain": cb.call_chain.is_some(),
                            });
                            println!("{}", json::to_pretty_json(&result)?);
                        }
                        _ => {
                            println!("[Callback] {}.{}", fw_name, cb_name);
                            println!("  Description: {}", cb.description);
                            println!("  Trigger: {}", cb.trigger);
                            println!("  Context: {}", cb.context.description());
                            if let Some(ref sig) = cb.signature {
                                println!("  Signature: {}", sig);
                            }
                            println!();
                        }
                    }
                }
            }
        }
    }

    // Search async patterns
    for (name, pat) in &kb.async_patterns {
        if name.to_lowercase().contains(&term_lower)
            || pat.description.to_lowercase().contains(&term_lower)
        {
            found = true;
            match format {
                OutputFormat::Json => {
                    let result = serde_json::json!({
                        "type": "async_pattern",
                        "name": name,
                        "description": pat.description,
                        "context": format!("{:?}", pat.context),
                        "has_timeline": pat.timeline.is_some(),
                        "has_call_chain": pat.handler_call_chain.is_some(),
                    });
                    println!("{}", json::to_pretty_json(&result)?);
                }
                _ => {
                    println!("[Async] {}", name);
                    println!("  Description: {}", pat.description);
                    println!("  Context: {}", pat.context.description());
                    if let Some(ref sig) = pat.handler_signature {
                        println!("  Handler: {}", sig);
                    }
                    println!();
                }
            }
        }
    }

    // Search kernel APIs
    for (name, api) in &kb.kernel_apis {
        if name.to_lowercase().contains(&term_lower) {
            found = true;
            match format {
                OutputFormat::Json => {
                    let result = serde_json::json!({
                        "type": "kernel_api",
                        "name": name,
                        "description": api.description,
                        "can_sleep": api.can_sleep,
                        "can_fail": api.can_fail,
                    });
                    println!("{}", json::to_pretty_json(&result)?);
                }
                _ => {
                    let sleep = if api.can_sleep { "can sleep" } else { "no sleep" };
                    let fail = if api.can_fail { "can fail" } else { "no fail" };
                    println!("[API] {}() - {} [{}, {}]", name, api.description, sleep, fail);
                }
            }
        }
    }

    if !found {
        eprintln!("No results found for '{}'", term);
    }

    Ok(())
}

/// Show complete kernel call chain for a framework callback
pub fn run_chain(framework: &str, callback: &str, format: &OutputFormat) -> Result<()> {
    let ctx = AnalysisContext::new();
    let kb = ctx.knowledge_base();

    // Try exact match first
    let chain = kb.get_callback_call_chain(framework, callback);

    // If not found, try searching by partial match
    let (fw_name, cb_name, chain) = if let Some(chain) = chain {
        (framework.to_string(), callback.to_string(), chain)
    } else {
        // Search for partial match
        let mut found = None;
        for (fname, fw) in &kb.frameworks {
            if fname.contains(framework) {
                for (cname, cb) in &fw.callbacks {
                    if cname.contains(callback) {
                        if let Some(ref chain) = cb.call_chain {
                            found = Some((fname.clone(), cname.clone(), chain));
                            break;
                        }
                    }
                }
            }
            if found.is_some() {
                break;
            }
        }
        match found {
            Some((f, c, chain)) => (f, c, chain),
            None => {
                eprintln!(
                    "No call chain found for {}.{}",
                    framework, callback
                );
                eprintln!();
                eprintln!("Available chains:");
                for (fname, fw) in &kb.frameworks {
                    for (cname, cb) in &fw.callbacks {
                        if cb.call_chain.is_some() {
                            eprintln!("  {} {}", fname, cname);
                        }
                    }
                }
                // Also check async patterns
                for (name, pat) in &kb.async_patterns {
                    if pat.handler_call_chain.is_some() {
                        eprintln!("  [async] {}", name);
                    }
                }
                return Ok(());
            }
        }
    };

    match format {
        OutputFormat::Json => {
            println!("{}", json::to_pretty_json(chain)?);
        }
        _ => {
            println!("{}", chain.name);
            println!("Trigger: {}", chain.trigger_source);
            println!();
            print_call_chain(&chain.nodes, &fw_name, &cb_name);
        }
    }

    Ok(())
}

/// Show async handler call chain
pub fn run_async_chain(pattern: &str, format: &OutputFormat) -> Result<()> {
    let ctx = AnalysisContext::new();
    let kb = ctx.knowledge_base();

    // Search async patterns
    let (name, pat) = if let Some(pat) = kb.get_async_pattern(pattern) {
        (pattern.to_string(), pat)
    } else {
        // Partial match
        let mut found = None;
        for (n, p) in &kb.async_patterns {
            if n.contains(pattern) {
                found = Some((n.clone(), p));
                break;
            }
        }
        match found {
            Some((n, p)) => (n, p),
            None => {
                eprintln!("Async pattern '{}' not found", pattern);
                eprintln!();
                eprintln!("Available patterns:");
                for name in kb.async_patterns.keys() {
                    eprintln!("  {}", name);
                }
                return Ok(());
            }
        }
    };

    match format {
        OutputFormat::Json => {
            let result = serde_json::json!({
                "name": name,
                "description": pat.description,
                "context": format!("{:?}", pat.context),
                "handler_call_chain": pat.handler_call_chain,
                "timeline": pat.timeline,
            });
            println!("{}", json::to_pretty_json(&result)?);
        }
        _ => {
            println!("[Async] {} - {}", name, pat.description);
            println!("Context: {}", pat.context.description());
            println!();

            if let Some(ref chain) = pat.handler_call_chain {
                println!("{}", chain.name);
                println!("Trigger: {}", chain.trigger_source);
                println!();
                for (i, node) in chain.nodes.iter().enumerate() {
                    let indent = "  ".repeat(i);
                    let marker = if node.is_user_entry { " <-- your code" } else { "" };
                    let file = node.file.as_deref().unwrap_or("");
                    println!(
                        "{}{}(){}",
                        indent, node.function, marker
                    );
                    if !file.is_empty() {
                        println!("{}  // {}", indent, file);
                    }
                    if let Some(ref desc) = node.description {
                        println!("{}  // {}", indent, desc);
                    }
                }
            }

            if let Some(ref timeline) = pat.timeline {
                println!();
                println!("Timeline: {}", timeline.name);
                println!();
                println!("  Phase 1: {} [{}]", timeline.phase1.name, timeline.phase1.context.description());
                println!("  ------- {} -------", timeline.separation);
                println!("  Phase 2: {} [{}]", timeline.phase2.name, timeline.phase2.context.description());
            }
        }
    }

    Ok(())
}

/// Match a source file against knowledge base patterns
pub fn run_match(file: &Path, format: &OutputFormat) -> Result<()> {
    let mut ctx = AnalysisContext::new();
    let result = ctx.analyze_file(file)?;
    let kb = ctx.knowledge_base();

    match format {
        OutputFormat::Json => {
            let mut matches = Vec::new();

            for (name, func) in &result.parse_result.functions {
                if func.is_callback {
                    if let Some((fw, cb, callback)) = kb.identify_callback(name, &result.source) {
                        matches.push(serde_json::json!({
                            "function": name,
                            "framework": fw,
                            "callback": cb,
                            "trigger": callback.trigger,
                            "context": format!("{:?}", callback.context),
                            "has_call_chain": callback.call_chain.is_some(),
                        }));
                    }
                }
            }

            let output = serde_json::json!({
                "file": file.to_string_lossy(),
                "matches": matches,
                "async_bindings": result.analysis.async_bindings.len(),
                "entry_points": result.analysis.entry_points,
            });
            println!("{}", json::to_pretty_json(&output)?);
        }
        _ => {
            println!("Knowledge Base Match: {}", file.display());
            println!();

            let mut match_count = 0;

            // Check callback matches
            for (name, func) in &result.parse_result.functions {
                if func.is_callback {
                    if let Some((fw, cb, callback)) = kb.identify_callback(name, &result.source) {
                        match_count += 1;
                        let chain_marker = if callback.call_chain.is_some() {
                            " [has chain]"
                        } else {
                            ""
                        };
                        println!(
                            "  {}() -> {}.{} [{}]{}",
                            name,
                            fw,
                            cb,
                            callback.context.description(),
                            chain_marker
                        );
                        println!("    Trigger: {}", callback.trigger);
                    }
                }
            }

            // Show async bindings
            if !result.analysis.async_bindings.is_empty() {
                println!();
                println!("  Async mechanisms:");
                for binding in &result.analysis.async_bindings {
                    let ctx_str = if binding.context.can_sleep() {
                        "can sleep"
                    } else {
                        "cannot sleep"
                    };
                    println!(
                        "    {}() via {:?} [{}]",
                        binding.handler, binding.mechanism, ctx_str
                    );
                }
                match_count += result.analysis.async_bindings.len();
            }

            if match_count == 0 {
                println!("  No framework patterns matched");
            }
        }
    }

    Ok(())
}

fn print_call_chain(
    nodes: &[flowsight_knowledge::CallChainNode],
    _fw_name: &str,
    _cb_name: &str,
) {
    for (i, node) in nodes.iter().enumerate() {
        let is_last = i == nodes.len() - 1;

        // Build visual prefix
        let tree_prefix: String = if i == 0 {
            String::new()
        } else {
            let mut s = String::new();
            for _ in 0..i - 1 {
                s.push_str("      ");
            }
            if is_last {
                s.push_str("  └── ");
            } else {
                s.push_str("  ├── ");
            }
            s
        };

        let marker = if node.is_user_entry {
            " <-- YOUR CODE"
        } else {
            ""
        };

        println!("{}{}(){}", tree_prefix, node.function, marker);

        // Show file and description on next lines
        let info_prefix: String = {
            let mut s = String::new();
            for _ in 0..i {
                s.push_str("      ");
            }
            s
        };

        if let Some(ref file) = node.file {
            println!("{}  // {}", info_prefix, file);
        }
        if let Some(ref desc) = node.description {
            println!("{}  // {}", info_prefix, desc);
        }
    }
}
