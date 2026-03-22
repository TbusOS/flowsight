//! Execution flow quality tests
//!
//! Tests that the `flow` command produces TRUE execution flow:
//! - Calls in source code order (not alphabetical)
//! - Branching shows if/else paths
//! - Error paths are separate from normal flow
//! - Async registrations are identified (not treated as calls)
//! - Error handlers (goto labels) shown as cleanup blocks

use std::process::Command;

fn flowsight() -> Command {
    Command::new(env!("CARGO_BIN_EXE_flowsight"))
}

fn test_driver() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test_driver.c")
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' { in_escape = true; }
        else if in_escape { if c == 'm' { in_escape = false; } }
        else { result.push(c); }
    }
    result
}

fn get_flow_output(file: &str, func: &str) -> String {
    let output = flowsight()
        .args(["flow", file, func])
        .output()
        .expect("failed to run flow");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    strip_ansi(&stdout)
}

fn get_flow_lines(file: &str, func: &str) -> Vec<String> {
    get_flow_output(file, func)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Extract function call names from flow output lines (strip tree chars and tags)
fn extract_call_names(lines: &[String]) -> Vec<String> {
    lines.iter()
        .filter_map(|line| {
            // Skip the root function line
            if !line.contains("├") && !line.contains("└") && !line.contains("│") {
                return None;
            }
            // Extract the function name: find "xxx()" pattern
            let clean = line.replace("├── ", "").replace("└── ", "").replace("│", "");
            let clean = clean.trim();
            // Remove reachability tags like [always] [error-path] etc.
            let after_tag = if let Some(pos) = clean.find(']') {
                clean[pos+1..].trim()
            } else {
                clean
            };
            // Get function name (up to "()")
            if let Some(pos) = after_tag.find("()") {
                let name = after_tag[..pos].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
            None
        })
        .collect()
}

// =========================================================================
// CRITICAL: Source code order (not alphabetical)
// =========================================================================

#[test]
fn flow_calls_in_source_order() {
    let lines = get_flow_lines(test_driver(), "my_device_probe");
    let calls = extract_call_names(&lines);

    // In the test driver, source order is:
    // devm_kzalloc (L52) → platform_get_resource (L56) → devm_ioremap_resource (L57)
    // → devm_clk_get (L61) → clk_prepare_enable (L65) → platform_get_irq (L69)
    // → spin_lock_init (L75) → INIT_WORK (L76) → devm_request_irq (L78)
    // → platform_set_drvdata (L83)
    //
    // If alphabetical: clk_* would come before devm_*, INIT_WORK before platform_*
    // If source order: devm_kzalloc is FIRST

    assert!(
        !calls.is_empty(),
        "Should have calls in output"
    );

    // devm_kzalloc should appear BEFORE devm_clk_get (source order)
    // In alphabetical order, devm_clk_get < devm_kzalloc
    let kzalloc_pos = calls.iter().position(|c| c.contains("devm_kzalloc"));
    let clk_get_pos = calls.iter().position(|c| c.contains("devm_clk_get"));

    if let (Some(kz), Some(cg)) = (kzalloc_pos, clk_get_pos) {
        assert!(
            kz < cg,
            "devm_kzalloc (L52) should appear before devm_clk_get (L61) in source order.\n\
             Got: kzalloc at position {}, clk_get at position {}.\n\
             Calls: {:?}",
            kz, cg, calls
        );
    }

    // platform_get_resource should appear before clk_prepare_enable
    let pgr_pos = calls.iter().position(|c| c.contains("platform_get_resource"));
    let cpe_pos = calls.iter().position(|c| c.contains("clk_prepare_enable"));
    if let (Some(pgr), Some(cpe)) = (pgr_pos, cpe_pos) {
        assert!(
            pgr < cpe,
            "platform_get_resource (L56) should appear before clk_prepare_enable (L65)"
        );
    }
}

// =========================================================================
// Error paths should be visible and separate
// =========================================================================

#[test]
fn flow_shows_error_paths() {
    let output = get_flow_output(test_driver(), "my_device_probe");

    // Should show error handling
    assert!(
        output.contains("error") || output.contains("err_clk") || output.contains("cleanup"),
        "Flow should show error handling paths.\nOutput:\n{}",
        output
    );
}

#[test]
fn flow_error_only_hides_normal() {
    let output = flowsight()
        .args(["flow", test_driver(), "my_device_probe", "--error-only"])
        .output()
        .expect("failed to run flow --error-only");
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));

    // Should NOT show [always] calls
    let has_always = stdout.lines().any(|l| l.contains("[always]"));
    assert!(
        !has_always,
        "--error-only should not show [always] calls.\nOutput:\n{}",
        stdout
    );
}

// =========================================================================
// Async registrations should be identified
// =========================================================================

#[test]
fn flow_identifies_async_registration() {
    let output = get_flow_output(test_driver(), "my_device_probe");

    assert!(
        output.contains("async") || output.contains("WorkQueue") || output.contains("IRQ"),
        "INIT_WORK/devm_request_irq should be identified as async registration.\nOutput:\n{}",
        output
    );
}

// =========================================================================
// Helper macros should be filtered
// =========================================================================

#[test]
fn flow_filters_helper_macros() {
    let output = get_flow_output(test_driver(), "my_device_probe");

    // IS_ERR and PTR_ERR should not appear as standalone calls
    let has_noise = output.lines().any(|l| {
        let clean = l.trim();
        (clean.contains("IS_ERR()") || clean.contains("PTR_ERR()"))
            && !clean.contains("[helper]")
    });

    assert!(
        !has_noise,
        "IS_ERR/PTR_ERR should be filtered from flow output.\nOutput:\n{}",
        output
    );
}

// =========================================================================
// Line numbers should be present
// =========================================================================

#[test]
fn flow_shows_line_numbers() {
    // The CFG output should show line numbers
    let output = flowsight()
        .args(["cfg", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run cfg");
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));

    // Should have line numbers (L52, L56, etc.)
    assert!(
        stdout.contains("L52") || stdout.contains("L56") || stdout.contains("L78"),
        "CFG output should show source line numbers.\nOutput:\n{}",
        stdout
    );
}

// =========================================================================
// Error handler goto labels should be detected
// =========================================================================

#[test]
fn flow_detects_goto_cleanup() {
    let output = flowsight()
        .args(["errors", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run errors");
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));

    assert!(
        stdout.contains("err_clk"),
        "Should detect err_clk goto label.\nOutput:\n{}",
        stdout
    );

    assert!(
        stdout.contains("clk_disable_unprepare"),
        "Should show cleanup function in error path.\nOutput:\n{}",
        stdout
    );
}

// =========================================================================
// Data flow should track variable assignments
// =========================================================================

#[test]
fn dataflow_tracks_ret_variable() {
    let output = flowsight()
        .args(["dataflow", test_driver(), "my_device_probe", "--var", "ret"])
        .output()
        .expect("failed to run dataflow");
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));

    // ret should have definitions from clk_prepare_enable() and devm_request_irq()
    assert!(
        stdout.contains("clk_prepare_enable"),
        "ret should be defined by clk_prepare_enable().\nOutput:\n{}",
        stdout
    );

    // ret should have uses in condition checks and return
    assert!(
        stdout.contains("condition") && stdout.contains("return"),
        "ret should be used in conditions and return.\nOutput:\n{}",
        stdout
    );
}
