//! `flowsight self-test` — quality scoring for CLI readiness
//!
//! Runs all major commands against test fixtures and real kernel code,
//! scores each category, and produces an overall quality score (0-100).

use anyhow::Result;
use crossterm::style::{Color, Stylize};
use std::path::{Path, PathBuf};
use std::process::Command;

const C_TITLE: Color = Color::Rgb { r: 140, g: 185, b: 165 };
const C_OK: Color = Color::Rgb { r: 130, g: 175, b: 140 };
const C_WARN: Color = Color::Rgb { r: 170, g: 160, b: 120 };
const C_ERR: Color = Color::Rgb { r: 195, g: 120, b: 120 };
const C_DIM: Color = Color::Rgb { r: 110, g: 115, b: 120 };

/// Quality score for a category
#[derive(Debug)]
struct CategoryScore {
    name: String,
    passed: usize,
    total: usize,
    details: Vec<(String, bool, String)>, // (test_name, passed, message)
}

impl CategoryScore {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: 0,
            total: 0,
            details: Vec::new(),
        }
    }

    fn add(&mut self, name: &str, passed: bool, msg: &str) {
        self.total += 1;
        if passed {
            self.passed += 1;
        }
        self.details.push((name.to_string(), passed, msg.to_string()));
    }

    fn score(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.passed as f64 / self.total as f64) * 100.0
    }
}

/// Run the self-test
pub fn run(kernel_path: Option<&Path>) -> Result<()> {
    println!("{}", "FlowSight CLI Quality Assessment".with(C_TITLE).bold());
    println!("{}", "=================================".with(C_DIM));
    println!();

    let binary = std::env::current_exe()
        .ok()
        .or_else(|| which_flowsight())
        .unwrap_or_else(|| PathBuf::from("flowsight"));

    let fixture = find_test_fixture();
    let kernel_file = kernel_path
        .map(PathBuf::from)
        .or_else(find_kernel_test_file);

    let mut categories = Vec::new();

    // Category 1: Core commands (fixture-based)
    categories.push(test_core_commands(&binary, fixture.as_deref()));

    // Category 2: CFG & Error Analysis
    categories.push(test_cfg_analysis(&binary, fixture.as_deref()));

    // Category 3: Data Flow
    categories.push(test_data_flow(&binary, fixture.as_deref()));

    // Category 4: Output Formats
    categories.push(test_output_formats(&binary, fixture.as_deref()));

    // Category 5: Output Quality (noise filtering)
    categories.push(test_output_quality(&binary, fixture.as_deref()));

    // Category 6: Real Kernel Code (if available)
    if let Some(ref kf) = kernel_file {
        categories.push(test_kernel_code(&binary, kf));
    } else {
        let mut cat = CategoryScore::new("Real Kernel Code");
        cat.add("kernel_path", false, "No kernel path found. Set LINUX_KERNEL_PATH or pass --kernel-path");
        categories.push(cat);
    }

    // Category 7: LLM Integration
    categories.push(test_llm_integration(&binary));

    // Category 8: Knowledge Base
    categories.push(test_knowledge_base(&binary));

    // Print results
    println!();
    let mut total_score = 0.0;
    let mut total_weight = 0.0;

    let weights = [20.0, 15.0, 15.0, 10.0, 15.0, 15.0, 5.0, 5.0]; // Must sum to 100

    for (i, cat) in categories.iter().enumerate() {
        let score = cat.score();
        let weight = weights.get(i).copied().unwrap_or(10.0);
        let weighted = score * weight / 100.0;
        total_score += weighted;
        total_weight += weight;

        let color = if score >= 90.0 {
            C_OK
        } else if score >= 60.0 {
            C_WARN
        } else {
            C_ERR
        };

        println!(
            "  {:<30} {:>5.1}% ({}/{}) weight={:.0}%",
            cat.name.as_str().with(C_TITLE),
            format!("{:.1}", score).with(color),
            cat.passed,
            cat.total,
            weight,
        );

        for (name, passed, msg) in &cat.details {
            if !passed {
                println!(
                    "    {} {} {}",
                    "✗".with(C_ERR),
                    name.as_str().with(C_DIM),
                    msg.as_str().with(C_DIM),
                );
            }
        }
    }

    // Overall score
    let overall = if total_weight > 0.0 {
        total_score / total_weight * 100.0
    } else {
        0.0
    };

    println!();
    let overall_color = if overall >= 90.0 {
        C_OK
    } else if overall >= 70.0 {
        C_WARN
    } else {
        C_ERR
    };

    let grade = if overall >= 95.0 {
        "A+ (Production Ready)"
    } else if overall >= 90.0 {
        "A  (Release Candidate)"
    } else if overall >= 80.0 {
        "B  (Beta Quality)"
    } else if overall >= 70.0 {
        "C  (Alpha Quality)"
    } else if overall >= 50.0 {
        "D  (Development)"
    } else {
        "F  (Not Ready)"
    };

    println!(
        "  {}: {:.1}/100  {}",
        "Overall Score".with(C_TITLE).bold(),
        format!("{:.1}", overall).with(overall_color).bold(),
        grade.with(overall_color),
    );
    println!();

    if overall < 90.0 {
        println!("{}", "  To reach 100:".with(C_TITLE));
        for cat in &categories {
            if cat.score() < 100.0 {
                let failing: Vec<&str> = cat.details.iter()
                    .filter(|(_, p, _)| !p)
                    .map(|(n, _, _)| n.as_str())
                    .collect();
                if !failing.is_empty() {
                    println!(
                        "    - {}: fix {}",
                        cat.name.as_str().with(C_WARN),
                        failing.join(", ").with(C_DIM),
                    );
                }
            }
        }
    }

    Ok(())
}

// =========================================================================
// Test categories
// =========================================================================

fn test_core_commands(bin: &Path, fixture: Option<&Path>) -> CategoryScore {
    let mut cat = CategoryScore::new("Core Commands");
    let f = match fixture {
        Some(f) => f,
        None => {
            cat.add("fixture", false, "Test fixture not found");
            return cat;
        }
    };
    let f_str = f.to_string_lossy();

    cat.add("analyze", run_ok(bin, &["analyze", &f_str]), "");
    cat.add("flow", run_ok(bin, &["flow", &f_str, "my_device_probe"]), "");
    cat.add("trace", run_ok(bin, &["trace", &f_str, "my_device_probe"]), "");
    cat.add("callers", run_ok(bin, &["callers", &f_str, "my_device_probe"]), "");
    cat.add("callees", run_ok(bin, &["callees", &f_str, "my_device_probe"]), "");
    cat.add("async", run_ok(bin, &["async", &f_str]), "");
    cat.add("callbacks", run_ok(bin, &["callbacks", &f_str]), "");
    cat.add("patterns", run_ok(bin, &["patterns", &f_str]), "");
    cat.add("search", run_ok(bin, &["search", "probe", &f_str]), "");
    cat.add("kb_stats", run_ok(bin, &["kb", "stats"]), "");

    cat
}

fn test_cfg_analysis(bin: &Path, fixture: Option<&Path>) -> CategoryScore {
    let mut cat = CategoryScore::new("CFG & Error Analysis");
    let f = match fixture {
        Some(f) => f,
        None => { cat.add("fixture", false, ""); return cat; }
    };
    let f_str = f.to_string_lossy();

    cat.add("cfg_text", run_ok(bin, &["cfg", &f_str, "my_device_probe"]), "");
    cat.add("cfg_dot", run_ok(bin, &["-F", "dot", "cfg", &f_str, "my_device_probe"]), "");
    cat.add("cfg_json", run_ok(bin, &["-F", "json", "cfg", &f_str, "my_device_probe"]), "");
    cat.add("errors_all", run_ok(bin, &["errors", &f_str]), "");
    cat.add("errors_func", run_ok(bin, &["errors", &f_str, "my_device_probe"]), "");

    // Quality: errors should detect error paths
    let out = run_output(bin, &["errors", &f_str, "my_device_probe"]);
    cat.add("errors_detect_paths", out.contains("error path"), "Should detect error paths");

    // Quality: CFG should classify macros
    let cfg_out = run_output(bin, &["cfg", &f_str, "my_device_probe"]);
    cat.add("cfg_macro_async", cfg_out.contains("async"), "Should classify async macros");

    cat
}

fn test_data_flow(bin: &Path, fixture: Option<&Path>) -> CategoryScore {
    let mut cat = CategoryScore::new("Data Flow Analysis");
    let f = match fixture {
        Some(f) => f,
        None => { cat.add("fixture", false, ""); return cat; }
    };
    let f_str = f.to_string_lossy();

    cat.add("dataflow", run_ok(bin, &["dataflow", &f_str, "my_device_probe"]), "");
    cat.add("dataflow_var", run_ok(bin, &["dataflow", &f_str, "my_device_probe", "--var", "ret"]), "");
    cat.add("dataflow_json", run_ok(bin, &["-F", "json", "dataflow", &f_str, "my_device_probe"]), "");

    // Quality: should track parameters
    let out = run_output(bin, &["dataflow", &f_str, "my_device_probe"]);
    cat.add("tracks_params", out.contains("pdev"), "Should track parameters");

    // Quality: should find error returns
    cat.add("finds_error_returns", out.contains("[error]"), "Should find error returns");

    // Quality: def-use chains should exist
    let var_out = run_output(bin, &["dataflow", &f_str, "my_device_probe", "--var", "ret"]);
    cat.add("def_use_chains", var_out.contains("Def-use chains"), "Should show def-use chains");

    cat
}

fn test_output_formats(bin: &Path, fixture: Option<&Path>) -> CategoryScore {
    let mut cat = CategoryScore::new("Output Formats");
    let f = match fixture {
        Some(f) => f,
        None => { cat.add("fixture", false, ""); return cat; }
    };
    let f_str = f.to_string_lossy();

    // JSON should be valid
    let json_out = run_output(bin, &["-F", "json", "analyze", &f_str]);
    cat.add("json_valid", serde_json::from_str::<serde_json::Value>(&json_out).is_ok(), "JSON should be parseable");

    // DOT should be valid
    let dot_out = run_output(bin, &["-F", "dot", "cfg", &f_str, "my_device_probe"]);
    cat.add("dot_valid", dot_out.contains("digraph") && dot_out.contains("->"), "DOT should be valid");

    // Ftrace format
    cat.add("ftrace", run_ok(bin, &["trace", &f_str, "my_device_probe"]), "");

    // Sequence format (kb chain)
    cat.add("sequence", run_ok(bin, &["-F", "sequence", "kb", "chain", "usb_driver", "probe"]), "");

    cat
}

fn test_output_quality(bin: &Path, fixture: Option<&Path>) -> CategoryScore {
    let mut cat = CategoryScore::new("Output Quality");
    let f = match fixture {
        Some(f) => f,
        None => { cat.add("fixture", false, ""); return cat; }
    };
    let f_str = f.to_string_lossy();

    let flow_out = run_output(bin, &["flow", &f_str, "my_device_probe"]);

    // Should have reachability tags
    cat.add("has_reachability_tags",
        flow_out.contains("always") || flow_out.contains("error-path"),
        "Flow should annotate reachability");

    // IS_ERR/PTR_ERR should be filtered or tagged as helper
    let has_iserr_noise = flow_out.lines().any(|l| {
        let clean = strip_ansi(l);
        (clean.contains("IS_ERR()") || clean.contains("PTR_ERR()"))
            && !clean.contains("[helper]")
            && !clean.contains("[macro]")
    });
    cat.add("no_helper_noise", !has_iserr_noise,
        "IS_ERR/PTR_ERR should be filtered or tagged");

    // Should classify INIT_WORK as async
    cat.add("async_classification",
        flow_out.contains("async"),
        "INIT_WORK should be [async:WorkQueue]");

    // Calls should be in source order, not alphabetical
    // Find first two function names in output
    let clean_lines: Vec<String> = flow_out.lines()
        .map(|l| strip_ansi(l))
        .filter(|l| l.contains("()") && !l.contains("my_device_probe"))
        .collect();
    let is_alphabetical = clean_lines.windows(2).all(|w| w[0] <= w[1]);
    cat.add("source_order", !is_alphabetical || clean_lines.len() < 3,
        "Calls should be in source code order, not alphabetical");

    // Error-only filter should work
    let error_only = run_output(bin, &["flow", &f_str, "my_device_probe", "--error-only"]);
    let has_always = strip_ansi(&error_only).contains("[always]");
    cat.add("error_only_filter", !has_always,
        "--error-only should hide [always] calls");

    cat
}

fn test_kernel_code(bin: &Path, kernel_file: &Path) -> CategoryScore {
    let mut cat = CategoryScore::new("Real Kernel Code");
    let f_str = kernel_file.to_string_lossy();

    // Find a probe function in the file
    let analyze_out = run_output(bin, &["-F", "json", "analyze", &f_str]);
    let func_name = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&analyze_out) {
        json.get("entry_points")
            .and_then(|ep| ep.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("main")
            .to_string()
    } else {
        "main".to_string()
    };

    cat.add("analyze", run_ok(bin, &["analyze", &f_str]), "");
    cat.add("cfg", run_ok(bin, &["cfg", &f_str, &func_name]), "");
    cat.add("errors", run_ok(bin, &["errors", &f_str]), "");
    cat.add("dataflow", run_ok(bin, &["dataflow", &f_str, &func_name]), "");

    // Quality: should find functions
    let cfg_out = run_output(bin, &["cfg", &f_str, &func_name]);
    cat.add("finds_blocks", cfg_out.contains("blocks"), "CFG should find blocks in kernel code");

    cat
}

fn test_llm_integration(bin: &Path) -> CategoryScore {
    let mut cat = CategoryScore::new("LLM Integration");

    cat.add("llm_providers", run_ok(bin, &["llm-providers"]), "");

    // llm-test will likely fail if no provider is running, that's OK
    // Just check the command doesn't crash
    let test_result = run_output_stderr(bin, &["llm-test", "ollama"]);
    cat.add("llm_test_runs", !test_result.contains("panicked"), "llm-test should not panic");

    cat
}

fn test_knowledge_base(bin: &Path) -> CategoryScore {
    let mut cat = CategoryScore::new("Knowledge Base");

    cat.add("kb_stats", run_ok(bin, &["kb", "stats"]), "");
    cat.add("kb_query", run_ok(bin, &["kb", "query", "usb"]), "");
    cat.add("kb_chain_usb", run_ok(bin, &["kb", "chain", "usb_driver", "probe"]), "");

    // Quality: should have substantial coverage
    let stats = run_output(bin, &["kb", "stats"]);
    let has_frameworks = stats.contains("Frameworks:") && !stats.contains("Frameworks:     0");
    cat.add("has_frameworks", has_frameworks, "Should have framework definitions");

    cat
}

// =========================================================================
// Helpers
// =========================================================================

fn run_ok(bin: &Path, args: &[&str]) -> bool {
    Command::new(bin)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_output(bin: &Path, args: &[&str]) -> String {
    Command::new(bin)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

fn run_output_stderr(bin: &Path, args: &[&str]) -> String {
    Command::new(bin)
        .args(args)
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            format!("{}{}", stdout, stderr)
        })
        .unwrap_or_default()
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn find_test_fixture() -> Option<PathBuf> {
    // Try relative to current binary
    let candidates = [
        "cli/tests/fixtures/test_driver.c",
        "../cli/tests/fixtures/test_driver.c",
        "tests/fixtures/test_driver.c",
    ];
    for c in &candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn find_kernel_test_file() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("LINUX_KERNEL_PATH") {
        let test_file = PathBuf::from(&path).join("drivers/usb/gadget/udc/fsl_udc_core.c");
        if test_file.exists() {
            return Some(test_file);
        }
        let test_file2 = PathBuf::from(&path).join("arch/arm/mach-imx/anatop.c");
        if test_file2.exists() {
            return Some(test_file2);
        }
    }

    let default = PathBuf::from("/Users/sky/linux-kernel/linux/drivers/usb/gadget/udc/fsl_udc_core.c");
    if default.exists() {
        return Some(default);
    }

    None
}

fn which_flowsight() -> Option<PathBuf> {
    // Try to find the binary in target/debug
    let debug = PathBuf::from("target/debug/flowsight");
    if debug.exists() {
        return Some(debug);
    }
    None
}
