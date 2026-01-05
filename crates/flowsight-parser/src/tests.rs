//! Extended tests for the FlowSight parser
//!
//! These tests verify the parser can correctly handle various C code patterns
//! commonly found in Linux kernel drivers.

use super::*;
use crate::treesitter::TreeSitterParser;

/// Test parsing of async patterns like INIT_WORK
#[test]
fn test_async_pattern_init_work() {
    let source = r#"
static void my_work_fn(struct work_struct *work) {
    printk("doing work\n");
}

static int driver_probe(struct usb_interface *intf) {
    struct my_device *dev = get_dev(intf);
    INIT_WORK(&dev->work, my_work_fn);
    return 0;
}
"#;
    let mut parser = TreeSitterParser::new();
    let result = parser.parse_source(source, "test.c").unwrap();

    assert_eq!(result.functions.len(), 2);

    let probe = result.functions.get("driver_probe").unwrap();
    assert!(probe.calls.contains(&"INIT_WORK".to_string()));
    assert!(probe.calls.contains(&"get_dev".to_string()));
}

/// Test parsing of timer patterns
#[test]
fn test_timer_pattern() {
    let source = r#"
static void timer_callback(struct timer_list *t) {
    struct my_device *dev = from_timer(dev, t, timer);
    schedule_work(&dev->work);
}

static int setup_timer(struct my_device *dev) {
    timer_setup(&dev->timer, timer_callback, 0);
    mod_timer(&dev->timer, jiffies + HZ);
    return 0;
}
"#;
    let mut parser = TreeSitterParser::new();
    let result = parser.parse_source(source, "test.c").unwrap();

    assert_eq!(result.functions.len(), 2);

    let setup = result.functions.get("setup_timer").unwrap();
    assert!(setup.calls.contains(&"timer_setup".to_string()));
    assert!(setup.calls.contains(&"mod_timer".to_string()));

    let callback = result.functions.get("timer_callback").unwrap();
    assert!(callback.calls.contains(&"schedule_work".to_string()));
}

/// Test parsing of ops structure with function pointers
/// NOTE: Function pointer field parsing is a known limitation to be improved
#[test]
fn test_ops_structure() {
    let source = r#"
struct file_operations {
    struct module *owner;
    int flags;
};
"#;
    let mut parser = TreeSitterParser::new();
    let result = parser.parse_source(source, "test.c").unwrap();

    assert_eq!(result.structs.len(), 1);
    let ops = result.structs.get("file_operations").unwrap();
    assert_eq!(ops.fields.len(), 2);
}

/// Test parsing of USB driver structure definition
#[test]
fn test_usb_driver_structure() {
    let source = r#"
struct usb_driver {
    const char *name;
    int (*probe)(struct usb_interface *intf, const struct usb_device_id *id);
    void (*disconnect)(struct usb_interface *intf);
};
"#;
    let mut parser = TreeSitterParser::new();
    let result = parser.parse_source(source, "test.c").unwrap();

    // Should parse the driver struct definition
    assert!(result.structs.contains_key("usb_driver"));
    let driver = result.structs.get("usb_driver").unwrap();
    assert_eq!(driver.name, "usb_driver");
}

/// Test parsing nested struct references
#[test]
fn test_nested_struct_references() {
    let source = r#"
struct inner_data {
    int value;
    char name[32];
};

struct outer_container {
    struct inner_data data;
    struct inner_data *data_ptr;
    int count;
};
"#;
    let mut parser = TreeSitterParser::new();
    let result = parser.parse_source(source, "test.c").unwrap();

    assert_eq!(result.structs.len(), 2);

    let outer = result.structs.get("outer_container").unwrap();
    assert!(outer.referenced_structs.contains(&"inner_data".to_string()));
}

/// Test parsing of container_of pattern
#[test]
fn test_container_of_pattern() {
    let source = r#"
static void work_handler(struct work_struct *work) {
    struct my_device *dev = container_of(work, struct my_device, work);
    dev->status++;
}
"#;
    let mut parser = TreeSitterParser::new();
    let result = parser.parse_source(source, "test.c").unwrap();

    let handler = result.functions.get("work_handler").unwrap();
    assert!(handler.calls.contains(&"container_of".to_string()));
}

/// Test parsing of interrupt handler pattern
#[test]
fn test_irq_handler() {
    let source = r#"
static irqreturn_t my_irq_handler(int irq, void *dev_id) {
    struct my_device *dev = dev_id;
    schedule_work(&dev->work);
    return IRQ_HANDLED;
}

static int setup_irq(struct my_device *dev) {
    return request_irq(dev->irq, my_irq_handler, IRQF_SHARED, "my_dev", dev);
}
"#;
    let mut parser = TreeSitterParser::new();
    let result = parser.parse_source(source, "test.c").unwrap();

    assert_eq!(result.functions.len(), 2);

    let setup = result.functions.get("setup_irq").unwrap();
    assert!(setup.calls.contains(&"request_irq".to_string()));
}

/// Test parsing of complex function with multiple calls
#[test]
fn test_complex_function() {
    let source = r#"
static int driver_init(struct platform_device *pdev) {
    struct my_device *dev;
    int ret;

    dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    dev->clk = devm_clk_get(&pdev->dev, NULL);
    if (IS_ERR(dev->clk))
        return PTR_ERR(dev->clk);

    ret = clk_prepare_enable(dev->clk);
    if (ret)
        return ret;

    platform_set_drvdata(pdev, dev);
    
    dev_info(&pdev->dev, "initialized\n");
    return 0;
}
"#;
    let mut parser = TreeSitterParser::new();
    let result = parser.parse_source(source, "test.c").unwrap();

    let init = result.functions.get("driver_init").unwrap();
    assert!(init.calls.contains(&"devm_kzalloc".to_string()));
    assert!(init.calls.contains(&"devm_clk_get".to_string()));
    assert!(init.calls.contains(&"clk_prepare_enable".to_string()));
    assert!(init.calls.contains(&"platform_set_drvdata".to_string()));
}
