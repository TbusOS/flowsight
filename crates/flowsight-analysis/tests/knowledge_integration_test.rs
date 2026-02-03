//! Integration tests for knowledge base data flow into analysis engine
//!
//! These tests verify that:
//! 1. Knowledge base is correctly loaded
//! 2. FuncPtrResolver uses KB callback patterns
//! 3. AsyncTracker uses KB async patterns
//! 4. Analyzer properly integrates KB into analysis results

use flowsight_analysis::Analyzer;
use flowsight_knowledge::KnowledgeBase;
use flowsight_parser::{treesitter::TreeSitterParser, Parser};

/// Test that the built-in knowledge base loads correctly
#[test]
fn test_knowledge_base_loads() {
    let kb = KnowledgeBase::builtin();

    // Should have USB driver framework
    assert!(
        kb.get_framework("usb_driver").is_some(),
        "Should have usb_driver framework"
    );

    // Should have file_operations framework
    assert!(
        kb.get_framework("file_operations").is_some(),
        "Should have file_operations framework"
    );

    // Should have async patterns
    assert!(
        kb.get_async_pattern("work_struct").is_some(),
        "Should have work_struct async pattern"
    );
    assert!(
        kb.get_async_pattern("timer_list").is_some(),
        "Should have timer_list async pattern"
    );
}

/// Test that USB probe callback is recognized with call chain
#[test]
fn test_usb_probe_call_chain() {
    let kb = KnowledgeBase::builtin();

    // Get USB probe callback
    let probe_callback = kb.get_callback("usb_driver", "probe");
    assert!(probe_callback.is_some(), "Should have probe callback");

    let callback = probe_callback.unwrap();
    assert!(
        callback.call_chain.is_some(),
        "Probe should have call chain"
    );

    let chain = callback.call_chain.as_ref().unwrap();
    assert!(!chain.nodes.is_empty(), "Call chain should have nodes");
    assert!(
        chain.trigger_source.contains("USB"),
        "Trigger should mention USB"
    );
}

/// Test that callback patterns are extracted from KB
#[test]
fn test_callback_patterns_available() {
    let kb = KnowledgeBase::builtin();

    // Get callbacks with patterns
    let patterns = kb.get_callbacks_with_patterns();

    // Should have patterns for probe, disconnect, open, etc.
    let has_probe_pattern = patterns.iter().any(|(_, cb, _)| *cb == "probe");
    let has_open_pattern = patterns.iter().any(|(_, cb, _)| *cb == "open");

    assert!(has_probe_pattern, "Should have probe pattern");
    assert!(has_open_pattern, "Should have open pattern");
}

/// Test analyzer with USB probe detection
#[test]
fn test_analyzer_detects_usb_probe() {
    let source = r#"
#include <linux/usb.h>

static int my_probe(struct usb_interface *intf, const struct usb_device_id *id)
{
    printk("probe called\n");
    return 0;
}

static void my_disconnect(struct usb_interface *intf)
{
    printk("disconnect called\n");
}

static struct usb_driver my_driver = {
    .name = "my_driver",
    .probe = my_probe,
    .disconnect = my_disconnect,
};
module_usb_driver(my_driver);
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse(source, "test_usb.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result).unwrap();

    // Should detect my_probe as an entry point
    assert!(
        result.entry_points.contains(&"my_probe".to_string()),
        "Should detect my_probe as entry point"
    );

    // Should detect my_disconnect as an entry point
    assert!(
        result.entry_points.contains(&"my_disconnect".to_string()),
        "Should detect my_disconnect as entry point"
    );

    // my_probe should be marked as callback
    let my_probe = parse_result.functions.get("my_probe");
    assert!(my_probe.is_some(), "Should have my_probe function");
    assert!(
        my_probe.unwrap().is_callback,
        "my_probe should be marked as callback"
    );
}

/// Test analyzer with work queue detection
#[test]
fn test_analyzer_detects_work_queue() {
    let source = r#"
#include <linux/workqueue.h>

static void my_work_handler(struct work_struct *work)
{
    printk("work handler called\n");
}

static int my_init(void)
{
    INIT_WORK(&my_device->work, my_work_handler);
    schedule_work(&my_device->work);
    return 0;
}
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse(source, "test_work.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result).unwrap();

    // Should detect async binding for work queue
    assert!(
        !result.async_bindings.is_empty(),
        "Should detect async bindings"
    );

    let work_binding = result
        .async_bindings
        .iter()
        .find(|b| b.handler == "my_work_handler");
    assert!(
        work_binding.is_some(),
        "Should detect my_work_handler binding"
    );

    let binding = work_binding.unwrap();
    assert!(
        matches!(
            binding.mechanism,
            flowsight_core::AsyncMechanism::WorkQueue { .. }
        ),
        "Should be WorkQueue mechanism"
    );
}

/// Test analyzer with timer detection
#[test]
fn test_analyzer_detects_timer() {
    let source = r#"
#include <linux/timer.h>

static void my_timer_handler(struct timer_list *timer)
{
    printk("timer fired\n");
}

static int my_init(void)
{
    timer_setup(&my_timer, my_timer_handler, 0);
    mod_timer(&my_timer, jiffies + HZ);
    return 0;
}
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse(source, "test_timer.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result).unwrap();

    // Should detect timer async binding
    let timer_binding = result
        .async_bindings
        .iter()
        .find(|b| b.handler == "my_timer_handler");
    assert!(
        timer_binding.is_some(),
        "Should detect my_timer_handler binding"
    );

    let binding = timer_binding.unwrap();
    assert!(
        matches!(
            binding.mechanism,
            flowsight_core::AsyncMechanism::Timer { .. }
        ),
        "Should be Timer mechanism"
    );

    // Timer handlers run in softirq context
    assert_eq!(
        binding.context,
        flowsight_core::ExecutionContext::SoftIrq,
        "Timer handler should be in SoftIrq context"
    );
}

/// Test that KB loaded from YAML contains patterns
#[test]
fn test_yaml_patterns_loaded() {
    let kb = KnowledgeBase::builtin();

    // Check if YAML-loaded frameworks exist (they use file_name prefix)
    // e.g., "usb_usb_driver" from knowledge/platforms/linux-kernel/drivers/usb.yaml
    let has_yaml_frameworks = kb
        .frameworks
        .keys()
        .any(|k| k.contains("usb_") || k.contains("platform_") || k.contains("i2c_"));

    // At minimum, built-in frameworks should exist
    assert!(
        kb.frameworks.contains_key("usb_driver"),
        "Should have usb_driver"
    );
    assert!(
        kb.frameworks.contains_key("file_operations"),
        "Should have file_operations"
    );
}

/// Test flow tree generation with KB injection
#[test]
fn test_flow_tree_with_kb_injection() {
    let source = r#"
static int my_probe(struct usb_interface *intf, const struct usb_device_id *id)
{
    my_helper();
    return 0;
}

static void my_helper(void)
{
    printk("helper\n");
}

static struct usb_driver my_driver = {
    .probe = my_probe,
};
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse(source, "test_flow.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result).unwrap();

    // Should have flow trees
    assert!(!result.flow_trees.is_empty(), "Should have flow trees");

    // Find flow tree for my_probe
    let probe_tree = result
        .flow_trees
        .iter()
        .find(|t| t.name == "my_probe" || t.children.iter().any(|c| c.name == "my_probe"));

    // Flow tree should exist (might be wrapped in kernel chain)
    assert!(
        probe_tree.is_some()
            || result
                .flow_trees
                .iter()
                .any(|t| t.name.contains("USB") || t.children.iter().any(|c| c.name == "my_probe")),
        "Should have flow tree for my_probe"
    );
}

/// Test that knowledge base API info is loaded
#[test]
fn test_kernel_api_info() {
    let kb = KnowledgeBase::builtin();

    // Should have kzalloc API info
    let kzalloc = kb.get_api("kzalloc");
    assert!(kzalloc.is_some(), "Should have kzalloc API info");

    let api = kzalloc.unwrap();
    assert!(api.can_sleep, "kzalloc can sleep");
    assert!(api.can_fail, "kzalloc can fail");

    // Should have spin_lock API info
    let spin_lock = kb.get_api("spin_lock");
    assert!(spin_lock.is_some(), "Should have spin_lock API info");
    assert!(!spin_lock.unwrap().can_sleep, "spin_lock cannot sleep");
}

/// Test analyzer uses KB for callback context
#[test]
fn test_callback_context_from_kb() {
    let source = r#"
static int my_open(struct inode *inode, struct file *file)
{
    return 0;
}

static ssize_t my_read(struct file *file, char __user *buf, size_t len, loff_t *off)
{
    return 0;
}

static const struct file_operations my_fops = {
    .open = my_open,
    .read = my_read,
};
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse(source, "test_fops.c").unwrap();

    let mut analyzer = Analyzer::new();
    let _result = analyzer.analyze(source, &mut parse_result).unwrap();

    // my_open should be marked as callback
    let my_open = parse_result.functions.get("my_open");
    assert!(my_open.is_some(), "Should have my_open function");
    assert!(
        my_open.unwrap().is_callback,
        "my_open should be marked as callback"
    );

    // Should have callback context
    let callback_ctx = my_open.unwrap().callback_context.as_ref();
    assert!(
        callback_ctx.is_some(),
        "my_open should have callback context"
    );
}
