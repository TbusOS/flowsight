// Integration test for FlowSight with Linux kernel code
// Run with: cargo test --package flowsight-analysis --test kernel_analysis_test

use flowsight_analysis::Analyzer;
use flowsight_parser::treesitter::TreeSitterParser;

#[test]
fn test_usb_storage_driver_analysis() {
    // Test with USB storage driver file
    let test_file = "/home/parallels/github/linux_kernel/drivers/usb/storage/debug.c";
    let source = std::fs::read_to_string(test_file)
        .expect("Failed to read test file");

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(&source, test_file)
        .expect("Failed to parse file");

    println!("=== USB Storage Driver Analysis ===");
    println!("File: {}", test_file);
    println!("Functions found: {}", parse_result.functions.len());
    println!("Structs found: {}", parse_result.structs.len());

    // Print function details
    println!("\n--- Functions ---");
    for (name, func) in &parse_result.functions {
        println!("  - {} (line {})", name, func.location.as_ref().map(|l| l.line).unwrap_or(0));
        if !func.calls.is_empty() {
            println!("    Calls: {:?}", func.calls.iter().take(5).collect::<Vec<_>>());
        }
    }

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(&source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n--- Analysis Results ---");
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());
    println!("Flow trees: {}", result.flow_trees.len());

    // Verify some basic assertions
    assert!(parse_result.functions.len() > 0, "Should find functions");
    assert!(result.flow_trees.len() > 0, "Should build flow trees");
}

#[test]
fn test_kernel_callchain_injection() {
    // Test that kernel call chains are properly injected for callbacks
    let source = r#"
#include <linux/usb.h>

struct my_usb_dev {
    struct usb_device *udev;
    struct work_struct work;
    struct timer_list timer;
};

static void my_work_handler(struct work_struct *work) {
    struct my_usb_dev *dev = container_of(work, struct my_usb_dev, work);
    dev->status = 1;
}

static void my_timer_fn(struct timer_list *t) {
    struct my_usb_dev *dev = container_of(t, struct my_usb_dev, timer);
    dev->status = 2;
}

static int my_probe(struct usb_interface *intf, const struct usb_device_id *id) {
    struct my_usb_dev *dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    INIT_WORK(&dev->work, my_work_handler);
    timer_setup(&dev->timer, my_timer_fn, 0);
    usb_set_intfdata(intf, dev);
    return 0;
}

static void my_disconnect(struct usb_interface *intf) {
    struct my_usb_dev *dev = usb_get_intfdata(intf);
    cancel_work_sync(&dev->work);
    del_timer_sync(&dev->timer);
    kfree(dev);
}

static struct usb_device_id my_table[] = {
    { USB_DEVICE(0x1234, 0x5678) },
    { }
};
MODULE_DEVICE_TABLE(usb, my_table);

static struct usb_driver my_driver = {
    .name = "my_driver",
    .id_table = my_table,
    .probe = my_probe,
    .disconnect = my_disconnect,
};

module_usb_driver(my_driver);
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "test_usb.c")
        .expect("Failed to parse");

    println!("\n=== Kernel Call Chain Injection Test ===");
    println!("Functions found: {}", parse_result.functions.len());

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n--- Results ---");
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings:");
    for binding in &result.async_bindings {
        let triggers = binding.trigger_locations.iter()
            .map(|l| format!("line {}", l.line))
            .collect::<Vec<_>>();
        println!("  - {} -> {} ({:?}) [triggers: {}]",
                 binding.variable, binding.handler, binding.mechanism,
                 triggers.join(", "));
    }

    // Verify callbacks are detected
    let work_handler = parse_result.functions.get("my_work_handler");
    let timer_fn = parse_result.functions.get("my_timer_fn");
    let probe = parse_result.functions.get("my_probe");
    let disconnect = parse_result.functions.get("my_disconnect");

    assert!(work_handler.map(|f| f.is_callback).unwrap_or(false), "work_handler should be callback");
    assert!(timer_fn.map(|f| f.is_callback).unwrap_or(false), "timer_fn should be callback");
    assert!(probe.map(|f| f.is_callback).unwrap_or(false), "probe should be callback");
    assert!(disconnect.map(|f| f.is_callback).unwrap_or(false), "disconnect should be callback");

    println!("\n✓ All callbacks correctly identified");
}

#[test]
fn test_simple_driver_execution_flow() {
    // Test the execution flow visualization for a simple driver
    let source = r#"
static int my_init(void) {
    printk("init\n");
    return 0;
}

static void my_exit(void) {
    printk("exit\n");
}

static void irq_handler(unsigned int irq, void *dev_id) {
    printk("irq\n");
    return IRQ_HANDLED;
}

static int probe_handler(struct platform_device *dev) {
    int ret;
    ret = request_irq(irq, irq_handler, IRQF_SHARED, "my_irq", NULL);
    if (ret)
        return ret;
    return 0;
}

static int remove_handler(struct platform_device *dev) {
    free_irq(irq, NULL);
    return 0;
}

static struct platform_driver my_driver = {
    .probe = probe_handler,
    .remove = remove_handler,
};

module_init(my_init);
module_exit(my_exit);
module_platform_driver(my_driver);
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "simple_driver.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Simple Driver Execution Flow Test ===");
    println!("Entry points: {:?}", result.entry_points);
    println!("Flow trees: {}", result.flow_trees.len());

    // Print flow tree structure
    for tree in &result.flow_trees {
        println!("\n--- Flow Tree: {} ---", tree.name);
        print_flow_tree(tree, 0);
    }

    // Verify entry points
    assert!(result.entry_points.contains(&"my_init".to_string()));
    assert!(result.entry_points.contains(&"my_exit".to_string()));
    assert!(result.entry_points.contains(&"probe_handler".to_string()));
    assert!(result.entry_points.contains(&"remove_handler".to_string()));
}

fn print_flow_tree(node: &flowsight_core::FlowNode, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}├── {} [{}]", indent, node.name, node.display_name);
    for child in &node.children {
        print_flow_tree(child, depth + 1);
    }
}
