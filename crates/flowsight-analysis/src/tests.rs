//! Tests for the FlowSight analysis engine

use super::*;
use flowsight_parser::treesitter::TreeSitterParser;

/// Test basic analyzer creation
#[test]
fn test_analyzer_creation() {
    let analyzer = Analyzer::new();
    assert!(true); // Analyzer created successfully
    drop(analyzer);
}

/// Test async tracking for INIT_WORK pattern
#[test]
fn test_async_tracking_work_struct() {
    let source = r#"
static void my_work_handler(struct work_struct *work) {
    printk("work done\n");
}

static int probe(struct usb_interface *intf) {
    struct my_device *dev = get_dev(intf);
    INIT_WORK(&dev->work, my_work_handler);
    schedule_work(&dev->work);
    return 0;
}
"#;
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "test.c").unwrap();
    
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result).unwrap();
    
    // Should find INIT_WORK binding
    assert!(!result.async_bindings.is_empty(), "Should find async bindings");
    
    let binding = result.async_bindings.iter()
        .find(|b| b.handler == "my_work_handler");
    assert!(binding.is_some(), "Should find my_work_handler binding");
}

/// Test async tracking for timer pattern
#[test]
fn test_async_tracking_timer() {
    let source = r#"
static void timer_fn(struct timer_list *t) {
    printk("timer fired\n");
}

static int init_device(struct my_device *dev) {
    timer_setup(&dev->timer, timer_fn, 0);
    mod_timer(&dev->timer, jiffies + HZ);
    return 0;
}
"#;
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "test.c").unwrap();
    
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result).unwrap();
    
    // Should find timer_setup binding
    let timer_binding = result.async_bindings.iter()
        .find(|b| b.handler == "timer_fn");
    assert!(timer_binding.is_some(), "Should find timer_fn binding");
}

/// Test entry point detection
#[test]
fn test_entry_point_detection() {
    let source = r#"
static int my_init(void) {
    return 0;
}

static void my_exit(void) {
}

module_init(my_init);
module_exit(my_exit);
"#;
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "test.c").unwrap();
    
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result).unwrap();
    
    assert!(result.entry_points.contains(&"my_init".to_string()));
    assert!(result.entry_points.contains(&"my_exit".to_string()));
}

/// Test call graph building
#[test]
fn test_call_graph() {
    let source = r#"
static void helper(void) {
    printk("helper\n");
}

static void caller(void) {
    helper();
}
"#;
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "test.c").unwrap();
    
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result).unwrap();
    
    // Should have call edge from caller to helper
    let edge = result.call_edges.iter()
        .find(|e| e.caller == "caller" && e.callee == "helper");
    assert!(edge.is_some(), "Should find caller->helper edge");
}

/// Test complete USB driver analysis
#[test]
fn test_usb_driver_analysis() {
    let source = r#"
struct my_usb_device {
    struct usb_device *udev;
    struct work_struct work;
    int status;
};

static void work_handler(struct work_struct *work) {
    struct my_usb_device *dev = container_of(work, struct my_usb_device, work);
    dev->status++;
}

static int my_probe(struct usb_interface *intf, const struct usb_device_id *id) {
    struct my_usb_device *dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    INIT_WORK(&dev->work, work_handler);
    usb_set_intfdata(intf, dev);
    return 0;
}

static void my_disconnect(struct usb_interface *intf) {
    struct my_usb_device *dev = usb_get_intfdata(intf);
    cancel_work_sync(&dev->work);
    kfree(dev);
}

module_init(my_probe);
"#;
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "test.c").unwrap();
    
    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result).unwrap();
    
    // Should find async binding
    assert!(!result.async_bindings.is_empty());
    
    // work_handler should be marked as callback
    let work_handler = parse_result.functions.get("work_handler").unwrap();
    assert!(work_handler.is_callback, "work_handler should be marked as callback");
}

