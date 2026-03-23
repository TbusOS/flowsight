//! Integration tests for the FlowSight CLI.
//!
//! These tests exercise every subcommand by spawning the actual binary
//! and inspecting stdout/stderr/exit-code.  They rely on small fixture
//! files shipped in `tests/fixtures/` so they work in CI without
//! access to a real Linux kernel tree.

use std::process::Command;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `Command` pointing at the compiled `flowsight` binary.
fn flowsight() -> Command {
    Command::new(env!("CARGO_BIN_EXE_flowsight"))
}

/// Path to the realistic test driver fixture.
fn test_driver() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test_driver.c")
}

/// Strip ANSI escape sequences from output for assertion matching
fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

/// Path to the (nearly) empty C file fixture.
fn empty_file() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/empty.c")
}

/// Path to the non-C text file fixture.
fn not_c_file() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/not_c_code.txt")
}

/// Path to the fixtures directory (used for directory-level commands).
fn fixtures_dir() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")
}

// =========================================================================
// 1. Help & Version
// =========================================================================

#[test]
fn help_shows_usage() {
    let output = flowsight()
        .arg("--help")
        .output()
        .expect("failed to run flowsight --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "exit code should be 0");
    assert!(
        stdout.contains("flowsight") || stdout.contains("Usage"),
        "help output should mention the program name or Usage"
    );
    assert!(
        stdout.contains("analyze"),
        "help output should list the analyze command"
    );
    assert!(
        stdout.contains("flow"),
        "help output should list the flow command"
    );
}

#[test]
fn version_shows_semver() {
    let output = flowsight()
        .arg("--version")
        .output()
        .expect("failed to run flowsight --version");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    // Should contain something like "flowsight 0.2.0"
    assert!(
        stdout.contains("flowsight"),
        "version output should contain the binary name"
    );
}

// =========================================================================
// 2. Analyze Command
// =========================================================================

#[test]
fn analyze_valid_file() {
    let output = flowsight()
        .args(["analyze", test_driver()])
        .output()
        .expect("failed to run analyze");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "analyze should succeed: {stdout}");
    // The driver defines several functions -- output should mention at least some.
    assert!(
        stdout.contains("function") || stdout.contains("Function"),
        "analyze output should mention functions: {stdout}"
    );
}

#[test]
fn analyze_json_output() {
    let output = flowsight()
        .args(["-F", "json", "analyze", test_driver()])
        .output()
        .expect("failed to run analyze --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");
    assert!(parsed.is_object(), "JSON root should be an object");
}

#[test]
fn analyze_missing_file() {
    let output = flowsight()
        .args(["analyze", "/nonexistent/path/foo.c"])
        .output()
        .expect("failed to run analyze on missing file");

    assert!(
        !output.status.success(),
        "analyze on missing file should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to read") || stderr.contains("No such file"),
        "stderr should explain the failure: {stderr}"
    );
}

#[test]
fn analyze_empty_file() {
    let output = flowsight()
        .args(["analyze", empty_file()])
        .output()
        .expect("failed to run analyze on empty file");

    // Should succeed (or at least not panic).
    // An empty file simply has zero functions -- that is fine.
    assert!(
        output.status.success(),
        "analyze of an empty file should not crash"
    );
}

#[test]
fn analyze_directory_requires_recursive_flag() {
    let output = flowsight()
        .args(["analyze", fixtures_dir()])
        .output()
        .expect("failed to run analyze on directory");

    assert!(
        !output.status.success(),
        "analyze on a directory without -r should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--recursive") || stderr.contains("-r"),
        "error should suggest --recursive flag: {stderr}"
    );
}

#[test]
fn analyze_directory_recursive() {
    let output = flowsight()
        .args(["analyze", "-r", fixtures_dir()])
        .output()
        .expect("failed to run analyze -r");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "recursive analyze should succeed: {stdout}"
    );
}

#[test]
fn analyze_directory_recursive_json() {
    let output = flowsight()
        .args(["-F", "json", "analyze", "-r", fixtures_dir()])
        .output()
        .expect("failed to run analyze -r --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("recursive JSON output should be valid");
    assert!(parsed.is_object());
}

// =========================================================================
// 3. Flow Command
// =========================================================================

#[test]
fn flow_probe_function() {
    let output = flowsight()
        .args(["flow", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run flow");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "flow should succeed: {stdout}"
    );
    assert!(
        stdout.contains("my_device_probe"),
        "output should contain the function name: {stdout}"
    );
}

#[test]
fn flow_json_output() {
    let output = flowsight()
        .args(["-F", "json", "flow", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run flow --json");

    let combined = format!(
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    assert!(output.status.success(), "flow -F json should succeed: {combined}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // JSON output may be a flow tree or a simple listing.
    // It should at least be parseable.
    let _parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect(&format!("flow JSON should be valid: {stdout}"));
}

#[test]
fn flow_with_depth_limit() {
    let output = flowsight()
        .args(["flow", test_driver(), "my_device_probe", "--depth", "1"])
        .output()
        .expect("failed to run flow --depth");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("my_device_probe"),
        "depth-limited output should still show the root function: {stdout}"
    );
}

#[test]
fn flow_missing_function_reports_available() {
    let output = flowsight()
        .args(["flow", test_driver(), "nonexistent_function"])
        .output()
        .expect("failed to run flow with nonexistent function");

    // The command currently prints to stderr and exits 0, listing available functions.
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("not found") || combined.contains("Available"),
        "should indicate the function was not found: {combined}"
    );
}

#[test]
fn flow_ftrace_format() {
    let output = flowsight()
        .args(["-F", "ftrace", "flow", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run flow -F ftrace");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("my_device_probe"),
        "ftrace output should mention the function: {stdout}"
    );
}

#[test]
fn flow_irq_handler() {
    let output = flowsight()
        .args(["flow", test_driver(), "my_irq_handler"])
        .output()
        .expect("failed to run flow for IRQ handler");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("my_irq_handler"),
        "should show the IRQ handler: {stdout}"
    );
}

#[test]
fn flow_work_handler() {
    let output = flowsight()
        .args(["flow", test_driver(), "my_work_handler"])
        .output()
        .expect("failed to run flow for work handler");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("my_work_handler"),
        "should show the work handler: {stdout}"
    );
}

// =========================================================================
// 4. Trace Command
// =========================================================================

#[test]
fn trace_function() {
    let output = flowsight()
        .args(["trace", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run trace");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("my_device_probe"),
        "trace output should contain the function: {stdout}"
    );
}

#[test]
fn trace_json_format() {
    let output = flowsight()
        .args(["trace", test_driver(), "my_device_probe", "--trace-format", "json"])
        .output()
        .expect("failed to run trace --trace-format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("trace JSON should be valid");
}

#[test]
fn trace_markdown_format() {
    let output = flowsight()
        .args(["trace", test_driver(), "my_device_probe", "--trace-format", "markdown"])
        .output()
        .expect("failed to run trace --trace-format markdown");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("my_device_probe"),
        "markdown trace should mention the function: {stdout}"
    );
}

#[test]
fn trace_invalid_format() {
    let output = flowsight()
        .args(["trace", test_driver(), "my_device_probe", "--trace-format", "bogus"])
        .output()
        .expect("failed to run trace with invalid format");

    assert!(
        !output.status.success(),
        "trace with invalid format should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown trace format") || stderr.contains("bogus"),
        "should report the bad format: {stderr}"
    );
}

// =========================================================================
// 5. Callers / Callees
// =========================================================================

#[test]
fn callers_of_work_handler() {
    // my_work_handler is registered via INIT_WORK, so callers should
    // mention async/work_struct or at least report something meaningful.
    let output = flowsight()
        .args(["callers", test_driver(), "my_work_handler"])
        .output()
        .expect("failed to run callers");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Callers of my_work_handler"),
        "should print header: {stdout}"
    );
}

#[test]
fn callers_of_probe() {
    let output = flowsight()
        .args(["callers", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run callers for probe");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Callers of my_device_probe"),
        "should print callers header: {stdout}"
    );
}

#[test]
fn callees_of_probe() {
    let output = flowsight()
        .args(["callees", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run callees");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("my_device_probe() calls"),
        "should show callees header: {stdout}"
    );
    // The probe function calls several kernel APIs.
    assert!(
        stdout.contains("devm_kzalloc")
            || stdout.contains("platform_get_resource")
            || stdout.contains("devm_ioremap_resource")
            || stdout.contains("clk_prepare_enable"),
        "should list at least one kernel API call: {stdout}"
    );
}

#[test]
fn callees_of_remove() {
    let output = flowsight()
        .args(["callees", test_driver(), "my_device_remove"])
        .output()
        .expect("failed to run callees for remove");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cancel_work_sync") || stdout.contains("clk_disable_unprepare"),
        "remove should call cleanup APIs: {stdout}"
    );
}

#[test]
fn callees_missing_function() {
    let output = flowsight()
        .args(["callees", test_driver(), "does_not_exist"])
        .output()
        .expect("failed to run callees for missing fn");

    // Current behaviour: prints "Function 'does_not_exist' not found" to stderr.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found"),
        "should report function not found: {stderr}"
    );
}

// =========================================================================
// 6. Async Command
// =========================================================================

#[test]
fn async_lists_handlers() {
    let output = flowsight()
        .args(["async", test_driver()])
        .output()
        .expect("failed to run async");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Async Handlers"),
        "should show async handlers header: {stdout}"
    );
    // The driver has INIT_WORK and devm_request_irq.
    // At minimum the work handler or irq handler should be detected.
    assert!(
        stdout.contains("my_work_handler") || stdout.contains("my_irq_handler"),
        "should detect at least one async handler: {stdout}"
    );
}

#[test]
fn async_empty_file() {
    let output = flowsight()
        .args(["async", empty_file()])
        .output()
        .expect("failed to run async on empty file");

    assert!(
        output.status.success(),
        "async on empty file should not crash"
    );
}

// =========================================================================
// 7. Callbacks Command
// =========================================================================

#[test]
fn callbacks_lists_callbacks() {
    let output = flowsight()
        .args(["callbacks", test_driver()])
        .output()
        .expect("failed to run callbacks");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Callbacks"),
        "should show callbacks header: {stdout}"
    );
}

#[test]
fn callbacks_empty_file() {
    let output = flowsight()
        .args(["callbacks", empty_file()])
        .output()
        .expect("failed to run callbacks on empty file");

    assert!(
        output.status.success(),
        "callbacks on empty file should not crash"
    );
}

// =========================================================================
// 8. KB (Knowledge Base) Commands
// =========================================================================

#[test]
fn kb_stats_text() {
    let output = flowsight()
        .args(["kb", "stats"])
        .output()
        .expect("failed to run kb stats");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Knowledge Base Statistics"),
        "should show KB stats header: {stdout}"
    );
    assert!(
        stdout.contains("Frameworks"),
        "should list frameworks: {stdout}"
    );
}

#[test]
fn kb_stats_json() {
    let output = flowsight()
        .args(["-F", "json", "kb", "stats"])
        .output()
        .expect("failed to run kb stats --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("kb stats JSON should be valid");
    assert!(
        parsed.get("frameworks").is_some(),
        "JSON should have 'frameworks' key: {parsed}"
    );
    assert!(
        parsed.get("async_patterns").is_some(),
        "JSON should have 'async_patterns' key: {parsed}"
    );
}

#[test]
fn kb_query_platform_driver() {
    let output = flowsight()
        .args(["kb", "query", "platform_driver"])
        .output()
        .expect("failed to run kb query");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should find the platform_driver framework.
    assert!(
        stdout.contains("platform_driver") || stdout.contains("Platform"),
        "should find platform_driver in KB: {stdout}"
    );
}

#[test]
fn kb_query_json() {
    let output = flowsight()
        .args(["-F", "json", "kb", "query", "platform_driver"])
        .output()
        .expect("failed to run kb query --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // JSON output may be multiple JSON objects (one per result).
    // At minimum the first object should parse.
    if !stdout.trim().is_empty() {
        // Try parsing the first line/object.
        let first_obj = stdout
            .trim()
            .lines()
            .collect::<Vec<_>>()
            .join("\n");
        let _parsed: serde_json::Value = serde_json::from_str(&first_obj)
            .expect(&format!("kb query JSON should be valid: {first_obj}"));
    }
}

#[test]
fn kb_query_no_results() {
    let output = flowsight()
        .args(["kb", "query", "zzz_nonexistent_zzz"])
        .output()
        .expect("failed to run kb query with no results");

    // Should succeed but report no results.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No results"),
        "should report no results: {stderr}"
    );
}

#[test]
fn kb_query_work_struct() {
    let output = flowsight()
        .args(["kb", "query", "work_struct"])
        .output()
        .expect("failed to run kb query work_struct");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("work") || stdout.contains("Work"),
        "should find work-related entries: {stdout}"
    );
}

#[test]
fn kb_match_driver() {
    let output = flowsight()
        .args(["kb", "match", test_driver()])
        .output()
        .expect("failed to run kb match");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Knowledge Base Match") || stdout.contains("match"),
        "should show match results: {stdout}"
    );
}

#[test]
fn kb_match_json() {
    let output = flowsight()
        .args(["-F", "json", "kb", "match", test_driver()])
        .output()
        .expect("failed to run kb match --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("kb match JSON should be valid");
    assert!(parsed.is_object());
    assert!(
        parsed.get("file").is_some(),
        "JSON should have 'file' key: {parsed}"
    );
}

// =========================================================================
// 9. Error Handling & Edge Cases
// =========================================================================

#[test]
fn nonexistent_file_error() {
    let output = flowsight()
        .args(["flow", "/tmp/flowsight_no_such_file_xyz.c", "main"])
        .output()
        .expect("failed to run flow on missing file");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to read") || stderr.contains("No such file"),
        "should report read failure: {stderr}"
    );
}

#[test]
fn invalid_subcommand() {
    let output = flowsight()
        .args(["not_a_command"])
        .output()
        .expect("failed to run with invalid subcommand");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("unrecognized"),
        "should report invalid subcommand: {stderr}"
    );
}

#[test]
fn analyze_non_c_file_gracefully() {
    // A plain text file is not valid C, but the parser should not panic.
    let output = flowsight()
        .args(["analyze", not_c_file()])
        .output()
        .expect("failed to run analyze on non-C file");

    // It may succeed (tree-sitter is lenient) or fail gracefully.
    // The key assertion: it must not crash/panic.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "should not panic on non-C input: {stderr}"
    );
}

#[test]
fn flow_non_c_file_gracefully() {
    let output = flowsight()
        .args(["flow", not_c_file(), "main"])
        .output()
        .expect("failed to run flow on non-C file");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "should not panic on non-C input: {stderr}"
    );
}

// =========================================================================
// 10. Global Format Flag Combinations
// =========================================================================

#[test]
fn global_format_text_explicit() {
    let output = flowsight()
        .args(["-F", "text", "analyze", test_driver()])
        .output()
        .expect("failed with explicit -F text");

    assert!(output.status.success());
}

#[test]
fn global_format_markdown_flow() {
    let output = flowsight()
        .args(["-F", "markdown", "flow", test_driver(), "my_device_probe"])
        .output()
        .expect("failed with -F markdown flow");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("my_device_probe"),
        "markdown flow output should contain the function: {stdout}"
    );
}

// =========================================================================
// 11. Train Commands (subcommand structure only - no large corpus needed)
// =========================================================================

#[test]
fn train_help() {
    let output = flowsight()
        .args(["train", "--help"])
        .output()
        .expect("failed to run train --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("generate") || stdout.contains("Generate"),
        "train help should list generate subcommand: {stdout}"
    );
}

#[test]
fn train_generate_fixtures_dir() {
    let output = flowsight()
        .args(["train", "generate", fixtures_dir()])
        .output()
        .expect("failed to run train generate");

    // Should either succeed (producing JSONL) or fail gracefully.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "train generate should not panic: {stderr}"
    );
}

// =========================================================================
// 12. Completions (hidden command)
// =========================================================================

#[test]
fn completions_bash() {
    let output = flowsight()
        .args(["completions", "bash"])
        .output()
        .expect("failed to run completions bash");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("flowsight") || stdout.contains("complete"),
        "bash completions should contain shell completion code: {stdout}"
    );
}

#[test]
fn completions_zsh() {
    let output = flowsight()
        .args(["completions", "zsh"])
        .output()
        .expect("failed to run completions zsh");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.is_empty(),
        "zsh completions should produce output"
    );
}

// =========================================================================
// 13. Verbose Flag
// =========================================================================

#[test]
fn verbose_flag_accepted() {
    let output = flowsight()
        .args(["-v", "analyze", test_driver()])
        .output()
        .expect("failed with -v flag");

    // Verbose flag should be accepted without error.
    assert!(
        output.status.success(),
        "verbose flag should not cause failure"
    );
}

// =========================================================================
// 14. KB Chain & AsyncChain (may or may not have data for platform_driver)
// =========================================================================

#[test]
fn kb_chain_platform_driver_probe() {
    let output = flowsight()
        .args(["kb", "chain", "platform_driver", "probe"])
        .output()
        .expect("failed to run kb chain");

    // May succeed or print "No call chain found" -- both are acceptable.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.contains("panicked"),
        "kb chain should not panic: {combined}"
    );
}

#[test]
fn kb_chain_json() {
    let output = flowsight()
        .args(["-F", "json", "kb", "chain", "platform_driver", "probe"])
        .output()
        .expect("failed to run kb chain --json");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "kb chain JSON should not panic: {stderr}"
    );
}

#[test]
fn kb_async_chain_work_struct() {
    let output = flowsight()
        .args(["kb", "async-chain", "work_struct"])
        .output()
        .expect("failed to run kb async-chain");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "kb async-chain should not panic: {stderr}"
    );
}

#[test]
fn kb_async_chain_json() {
    let output = flowsight()
        .args(["-F", "json", "kb", "async-chain", "work_struct"])
        .output()
        .expect("failed to run kb async-chain --json");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "kb async-chain JSON should not panic: {stderr}"
    );
}

// =========================================================================
// 15. Multiple Functions in Same File
// =========================================================================

#[test]
fn callees_irq_handler() {
    let output = flowsight()
        .args(["callees", test_driver(), "my_irq_handler"])
        .output()
        .expect("failed to run callees for irq handler");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // IRQ handler calls readl and schedule_work.
    assert!(
        stdout.contains("readl") || stdout.contains("schedule_work"),
        "IRQ handler should call readl or schedule_work: {stdout}"
    );
}

#[test]
fn callees_work_handler() {
    let output = flowsight()
        .args(["callees", test_driver(), "my_work_handler"])
        .output()
        .expect("failed to run callees for work handler");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Work handler calls spin_lock_irqsave, writel, spin_unlock_irqrestore.
    assert!(
        stdout.contains("spin_lock_irqsave")
            || stdout.contains("writel")
            || stdout.contains("spin_unlock_irqrestore"),
        "work handler should call spinlock/IO APIs: {stdout}"
    );
}

// =========================================================================
// CFG: Control flow graph commands
// =========================================================================

#[test]
fn cfg_text_output() {
    let output = flowsight()
        .args(["cfg", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run cfg");

    assert!(output.status.success(), "cfg should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Strip ANSI codes for assertion
    let clean = strip_ansi(&stdout);
    assert!(
        clean.contains("CFG:") && clean.contains("my_device_probe"),
        "should show CFG header: {clean}"
    );
    assert!(
        clean.contains("blocks") && clean.contains("edges"),
        "should show block/edge counts: {clean}"
    );
    assert!(
        clean.contains("Calls:"),
        "should list calls: {clean}"
    );
}

#[test]
fn cfg_dot_output() {
    let output = flowsight()
        .args(["-F", "dot", "cfg", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run cfg -F dot");

    assert!(output.status.success(), "cfg dot should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("digraph") && stdout.contains("my_device_probe"),
        "DOT output should be valid: {stdout}"
    );
    assert!(
        stdout.contains("->"),
        "DOT should have edges: {stdout}"
    );
}

#[test]
fn cfg_json_output() {
    let output = flowsight()
        .args(["-F", "json", "cfg", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run cfg -F json");

    assert!(output.status.success(), "cfg json should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("should be valid JSON");
    assert_eq!(
        parsed["function_name"], "my_device_probe",
        "JSON should contain function name"
    );
    assert!(
        parsed["blocks"].as_array().map(|a| a.len()).unwrap_or(0) > 0,
        "JSON should have blocks"
    );
}

#[test]
fn cfg_detects_error_paths() {
    let output = flowsight()
        .args(["cfg", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run cfg");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The test_driver probe function should have error handling
    assert!(
        stdout.contains("error") || stdout.contains("Error"),
        "should detect error paths or error-related blocks: {stdout}"
    );
}

#[test]
fn cfg_classifies_macros() {
    let output = flowsight()
        .args(["cfg", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run cfg");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should classify INIT_WORK as async registration
    assert!(
        stdout.contains("async") || stdout.contains("INIT_WORK"),
        "should classify async macros: {stdout}"
    );
}

#[test]
fn errors_lists_error_paths() {
    let output = flowsight()
        .args(["errors", test_driver()])
        .output()
        .expect("failed to run errors");

    assert!(output.status.success(), "errors command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("error path") || stdout.contains("Error path"),
        "should show error path info: {stdout}"
    );
}

#[test]
fn errors_single_function() {
    let output = flowsight()
        .args(["errors", test_driver(), "my_device_probe"])
        .output()
        .expect("failed to run errors for single function");

    assert!(output.status.success(), "errors for function should succeed");
}

#[test]
fn errors_json_output() {
    let output = flowsight()
        .args(["-F", "json", "errors", test_driver()])
        .output()
        .expect("failed to run errors -F json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("should be valid JSON");
    assert!(
        parsed.as_array().is_some(),
        "errors JSON should be an array"
    );
}

#[test]
fn cfg_nonexistent_function_fails() {
    let output = flowsight()
        .args(["cfg", test_driver(), "nonexistent_function_xyz"])
        .output()
        .expect("failed to run cfg with bad function");

    assert!(
        !output.status.success(),
        "cfg with nonexistent function should fail"
    );
}

// ---------------------------------------------------------------------------
// quality (AQS) command
// ---------------------------------------------------------------------------

#[test]
fn quality_single_file_text() {
    let output = flowsight()
        .args(["quality", test_driver()])
        .output()
        .expect("failed to run quality");

    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    assert!(
        stdout.contains("Analysis Quality Score"),
        "should show AQS header"
    );
    assert!(
        stdout.contains("Direct call resolution"),
        "should show dimension breakdown"
    );
}

#[test]
fn quality_single_file_json() {
    let output = flowsight()
        .args(["-F", "json", "quality", test_driver()])
        .output()
        .expect("failed to run quality -F json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("quality JSON should be valid");
    let obj = parsed.as_object().expect("quality JSON should be object");
    assert!(obj.contains_key("score"), "should have score field");
    assert!(obj.contains_key("dimensions"), "should have dimensions");
    assert!(obj.contains_key("stats"), "should have stats");

    let score = obj["score"].as_f64().expect("score should be a number");
    assert!(score >= 0.0 && score <= 1.0, "score should be 0.0-1.0");
}

#[test]
fn quality_directory_text() {
    let fixtures_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    let output = flowsight()
        .args(["quality", fixtures_dir])
        .output()
        .expect("failed to run quality on directory");

    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    assert!(
        stdout.contains("Directory AQS Report"),
        "should show directory report header"
    );
    assert!(
        stdout.contains("Per-File Scores"),
        "should show per-file breakdown"
    );
}

#[test]
fn quality_directory_json() {
    let fixtures_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    let output = flowsight()
        .args(["-F", "json", "quality", fixtures_dir])
        .output()
        .expect("failed to run quality -F json on directory");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("quality JSON should be valid");
    let obj = parsed.as_object().expect("should be object");
    assert!(obj.contains_key("aggregate"), "should have aggregate");
    assert!(obj.contains_key("per_file"), "should have per_file");
    assert!(obj.contains_key("files_analyzed"), "should have files_analyzed");
}

#[test]
fn quality_budget_json() {
    let fixtures_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    let output = flowsight()
        .args(["-F", "json", "quality", fixtures_dir, "--budget", "60"])
        .output()
        .expect("failed to run quality --budget");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("budgeted quality JSON should be valid");
    let obj = parsed.as_object().expect("should be object");
    assert!(obj.contains_key("budget_seconds"), "should have budget_seconds");
    assert!(obj.contains_key("elapsed_seconds"), "should have elapsed_seconds");
    assert!(obj.contains_key("aggregate_aqs"), "should have aggregate_aqs");
    assert!(obj.contains_key("coverage_pct"), "should have coverage_pct");
    assert!(obj.contains_key("per_file"), "should have per_file");
}

#[test]
fn experiment_list_empty() {
    let output = flowsight()
        .args(["experiment", "list"])
        .output()
        .expect("failed to run experiment list");

    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    assert!(
        stdout.contains("No experiments found"),
        "should show empty message"
    );
}

#[test]
fn experiment_list_json() {
    let output = flowsight()
        .args(["-F", "json", "experiment", "list"])
        .output()
        .expect("failed to run experiment list -F json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("experiment list JSON should be valid");
    assert!(parsed.as_array().is_some(), "should be an array");
}

#[test]
fn review_requires_llm() {
    // Review needs a configured LLM provider — should fail gracefully without one
    let output = flowsight()
        .args(["review", test_driver(), "--no-stream"])
        .output()
        .expect("failed to run review");

    // Should exit non-zero since no LLM is configured
    // (but should not panic — graceful error message)
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Either fails with "no provider" or succeeds if user has config
    assert!(
        !output.status.success() || stderr.contains("Reviewing"),
        "review should either fail gracefully or succeed with LLM"
    );
}

#[test]
fn review_help() {
    let output = flowsight()
        .args(["review", "--help"])
        .output()
        .expect("failed to run review --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AI-powered code review"));
    assert!(stdout.contains("--focus"));
    assert!(stdout.contains("--provider"));
}

#[test]
fn bench_json() {
    let fixtures_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    let output = flowsight()
        .args(["-F", "json", "bench", fixtures_dir, "--budget", "60"])
        .output()
        .expect("failed to run bench");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("bench JSON should be valid");
    let obj = parsed.as_object().expect("should be object");

    let aqs = obj["aggregate_aqs"]["score"].as_f64().expect("score should be number");
    assert!(aqs >= 0.0 && aqs <= 1.0, "AQS should be 0.0-1.0");
    assert!(obj["files_total"].as_u64().unwrap() > 0, "should find files");
}
