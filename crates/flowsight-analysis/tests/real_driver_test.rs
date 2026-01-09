//! Integration tests with real USB/I2C driver code patterns
//!
//! Tests the analysis engine against realistic kernel driver code.

use flowsight_analysis::Analyzer;
use flowsight_analysis::callback::CallbackAnalyzer;
use flowsight_analysis::scenario::{Scenario, SymbolicValue, ScenarioExecutor, ScenarioOptions};
use flowsight_parser::treesitter::TreeSitterParser;

/// Realistic USB driver code (based on usb-skeleton.c pattern)
const USB_SKELETON_DRIVER: &str = r#"
#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/usb.h>
#include <linux/slab.h>

#define USB_SKEL_VENDOR_ID  0xfff0
#define USB_SKEL_PRODUCT_ID 0xfff0

struct usb_skel {
    struct usb_device *udev;
    struct usb_interface *interface;
    struct semaphore limit_sem;
    struct usb_anchor submitted;
    struct urb *bulk_in_urb;
    unsigned char *bulk_in_buffer;
    size_t bulk_in_size;
    size_t bulk_in_filled;
    size_t bulk_in_copied;
    __u8 bulk_in_endpointAddr;
    __u8 bulk_out_endpointAddr;
    int errors;
    bool ongoing_read;
    spinlock_t err_lock;
    struct kref kref;
    struct mutex io_mutex;
    unsigned long disconnected:1;
    wait_queue_head_t bulk_in_wait;
    struct work_struct work;
};

static struct usb_device_id skel_table[] = {
    { USB_DEVICE(USB_SKEL_VENDOR_ID, USB_SKEL_PRODUCT_ID) },
    { }
};
MODULE_DEVICE_TABLE(usb, skel_table);

static void skel_work_handler(struct work_struct *work)
{
    struct usb_skel *dev = container_of(work, struct usb_skel, work);

    mutex_lock(&dev->io_mutex);
    if (dev->disconnected) {
        mutex_unlock(&dev->io_mutex);
        return;
    }

    // Do actual work here
    dev->errors = 0;
    mutex_unlock(&dev->io_mutex);
}

static void skel_read_bulk_callback(struct urb *urb)
{
    struct usb_skel *dev = urb->context;
    unsigned long flags;

    spin_lock_irqsave(&dev->err_lock, flags);

    if (urb->status) {
        if (urb->status == -ENOENT ||
            urb->status == -ECONNRESET ||
            urb->status == -ESHUTDOWN) {
            dev->errors = urb->status;
        }
    } else {
        dev->bulk_in_filled = urb->actual_length;
    }
    dev->ongoing_read = false;
    spin_unlock_irqrestore(&dev->err_lock, flags);

    wake_up_interruptible(&dev->bulk_in_wait);
}

static void skel_write_bulk_callback(struct urb *urb)
{
    struct usb_skel *dev = urb->context;

    if (urb->status) {
        if (urb->status == -ENOENT ||
            urb->status == -ECONNRESET ||
            urb->status == -ESHUTDOWN) {
            dev_err(&dev->interface->dev,
                "%s - nonzero write bulk status received: %d\n",
                __func__, urb->status);
        }
        spin_lock(&dev->err_lock);
        dev->errors = urb->status;
        spin_unlock(&dev->err_lock);
    }

    usb_free_coherent(urb->dev, urb->transfer_buffer_length,
                      urb->transfer_buffer, urb->transfer_dma);
}

static void skel_delete(struct kref *kref)
{
    struct usb_skel *dev = container_of(kref, struct usb_skel, kref);

    usb_free_urb(dev->bulk_in_urb);
    usb_put_intf(dev->interface);
    usb_put_dev(dev->udev);
    kfree(dev->bulk_in_buffer);
    kfree(dev);
}

static int skel_probe(struct usb_interface *interface,
                      const struct usb_device_id *id)
{
    struct usb_skel *dev;
    struct usb_endpoint_descriptor *bulk_in, *bulk_out;
    int retval;

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    kref_init(&dev->kref);
    sema_init(&dev->limit_sem, 8);
    mutex_init(&dev->io_mutex);
    spin_lock_init(&dev->err_lock);
    init_usb_anchor(&dev->submitted);
    init_waitqueue_head(&dev->bulk_in_wait);
    INIT_WORK(&dev->work, skel_work_handler);

    dev->udev = usb_get_dev(interface_to_usbdev(interface));
    dev->interface = usb_get_intf(interface);

    retval = usb_find_common_endpoints(interface->cur_altsetting,
                                       &bulk_in, &bulk_out, NULL, NULL);
    if (retval) {
        dev_err(&interface->dev, "Could not find endpoints\n");
        goto error;
    }

    dev->bulk_in_size = usb_endpoint_maxp(bulk_in);
    dev->bulk_in_endpointAddr = bulk_in->bEndpointAddress;
    dev->bulk_in_buffer = kmalloc(dev->bulk_in_size, GFP_KERNEL);
    if (!dev->bulk_in_buffer) {
        retval = -ENOMEM;
        goto error;
    }

    dev->bulk_in_urb = usb_alloc_urb(0, GFP_KERNEL);
    if (!dev->bulk_in_urb) {
        retval = -ENOMEM;
        goto error;
    }

    dev->bulk_out_endpointAddr = bulk_out->bEndpointAddress;

    usb_set_intfdata(interface, dev);

    if (id->idVendor == 0x1234) {
        schedule_work(&dev->work);
    }

    retval = usb_register_dev(interface, &skel_class);
    if (retval) {
        dev_err(&interface->dev, "Not able to get a minor\n");
        usb_set_intfdata(interface, NULL);
        goto error;
    }

    dev_info(&interface->dev,
             "USB Skeleton device now attached to USBSkel-%d",
             interface->minor);
    return 0;

error:
    kref_put(&dev->kref, skel_delete);
    return retval;
}

static void skel_disconnect(struct usb_interface *interface)
{
    struct usb_skel *dev;
    int minor = interface->minor;

    dev = usb_get_intfdata(interface);

    usb_deregister_dev(interface, &skel_class);

    mutex_lock(&dev->io_mutex);
    dev->disconnected = 1;
    mutex_unlock(&dev->io_mutex);

    usb_kill_anchored_urbs(&dev->submitted);
    cancel_work_sync(&dev->work);

    kref_put(&dev->kref, skel_delete);

    dev_info(&interface->dev, "USB Skeleton #%d now disconnected", minor);
}

static struct usb_driver skel_driver = {
    .name       = "skeleton",
    .probe      = skel_probe,
    .disconnect = skel_disconnect,
    .id_table   = skel_table,
};

module_usb_driver(skel_driver);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Your Name");
MODULE_DESCRIPTION("USB Skeleton Driver");
"#;

/// I2C driver code pattern
const I2C_DRIVER: &str = r#"
#include <linux/i2c.h>
#include <linux/module.h>
#include <linux/workqueue.h>
#include <linux/delay.h>

struct my_i2c_device {
    struct i2c_client *client;
    struct work_struct init_work;
    struct delayed_work poll_work;
    int status;
    bool initialized;
};

static void i2c_init_work_handler(struct work_struct *work)
{
    struct my_i2c_device *dev = container_of(work, struct my_i2c_device, init_work);

    // Initialize the device
    dev->status = i2c_smbus_read_byte_data(dev->client, 0x00);
    if (dev->status >= 0) {
        dev->initialized = true;
    }
}

static void i2c_poll_work_handler(struct work_struct *work)
{
    struct delayed_work *dwork = to_delayed_work(work);
    struct my_i2c_device *dev = container_of(dwork, struct my_i2c_device, poll_work);

    if (!dev->initialized)
        return;

    dev->status = i2c_smbus_read_byte_data(dev->client, 0x01);

    // Re-schedule the poll
    schedule_delayed_work(&dev->poll_work, msecs_to_jiffies(1000));
}

static int my_i2c_probe(struct i2c_client *client, const struct i2c_device_id *id)
{
    struct my_i2c_device *dev;

    dev = devm_kzalloc(&client->dev, sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    dev->client = client;
    dev->initialized = false;

    INIT_WORK(&dev->init_work, i2c_init_work_handler);
    INIT_DELAYED_WORK(&dev->poll_work, i2c_poll_work_handler);

    i2c_set_clientdata(client, dev);

    // Start initialization
    schedule_work(&dev->init_work);

    return 0;
}

static void my_i2c_remove(struct i2c_client *client)
{
    struct my_i2c_device *dev = i2c_get_clientdata(client);

    cancel_work_sync(&dev->init_work);
    cancel_delayed_work_sync(&dev->poll_work);
}

static const struct i2c_device_id my_i2c_id[] = {
    { "my_i2c_device", 0 },
    { }
};
MODULE_DEVICE_TABLE(i2c, my_i2c_id);

static struct i2c_driver my_i2c_driver = {
    .driver = {
        .name = "my_i2c_device",
    },
    .probe = my_i2c_probe,
    .remove = my_i2c_remove,
    .id_table = my_i2c_id,
};

module_i2c_driver(my_i2c_driver);

MODULE_LICENSE("GPL");
"#;

/// Test USB driver analysis
#[test]
fn test_usb_driver_analysis() {
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(USB_SKELETON_DRIVER, "usb_skeleton.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(USB_SKELETON_DRIVER, &mut parse_result).unwrap();

    // Should detect async bindings
    assert!(!result.async_bindings.is_empty(), "Should find async bindings in USB driver");

    // Should find INIT_WORK binding for skel_work_handler
    let work_binding = result.async_bindings.iter()
        .find(|b| b.handler == "skel_work_handler");
    assert!(work_binding.is_some(), "Should find skel_work_handler binding");

    // Should mark callback functions
    assert!(parse_result.functions.get("skel_work_handler")
        .map(|f| f.is_callback)
        .unwrap_or(false), "skel_work_handler should be marked as callback");

    // Should find entry points
    assert!(result.entry_points.iter().any(|e| e.contains("probe") || e.contains("disconnect")),
        "Should find probe/disconnect as entry points");
}

/// Test I2C driver analysis
#[test]
fn test_i2c_driver_analysis() {
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(I2C_DRIVER, "i2c_driver.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(I2C_DRIVER, &mut parse_result).unwrap();

    // Should detect async bindings
    assert!(!result.async_bindings.is_empty(), "Should find async bindings in I2C driver");

    // Should find INIT_WORK binding
    let init_work = result.async_bindings.iter()
        .find(|b| b.handler == "i2c_init_work_handler");
    assert!(init_work.is_some(), "Should find i2c_init_work_handler binding");

    // Should find INIT_DELAYED_WORK binding
    let delayed_work = result.async_bindings.iter()
        .find(|b| b.handler == "i2c_poll_work_handler");
    assert!(delayed_work.is_some(), "Should find i2c_poll_work_handler binding");
}

/// Test callback pattern recognition on USB driver
#[test]
fn test_usb_driver_callback_patterns() {
    let mut analyzer = CallbackAnalyzer::new();
    analyzer.set_functions(vec![
        "skel_work_handler".to_string(),
        "skel_read_bulk_callback".to_string(),
        "skel_write_bulk_callback".to_string(),
        "skel_delete".to_string(),
        "skel_probe".to_string(),
        "skel_disconnect".to_string(),
    ]);

    let result = analyzer.analyze(USB_SKELETON_DRIVER);

    // Should detect queue patterns (schedule_work)
    assert!(!result.queue_patterns.is_empty(), "Should find queue patterns");

    let schedule_work = result.queue_patterns.iter()
        .find(|p| p.enqueue_func == "schedule_work");
    assert!(schedule_work.is_some(), "Should find schedule_work pattern");
}

/// Test scenario-based analysis with USB probe
#[test]
fn test_usb_probe_scenario() {
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(USB_SKELETON_DRIVER, "usb_skeleton.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(USB_SKELETON_DRIVER, &mut parse_result).unwrap();

    // Create a scenario for skel_probe
    let mut scenario = Scenario::new("probe_vendor_0x1234", "skel_probe");
    scenario
        .bind("id->idVendor", SymbolicValue::Integer(0x1234))
        .bind("id->idProduct", SymbolicValue::Integer(0x5678))
        .bind("interface", SymbolicValue::Pointer { is_null: false, size: None });

    // Find the flow tree for skel_probe
    let probe_tree = result.flow_trees.iter()
        .find(|t| t.name == "skel_probe");

    if let Some(tree) = probe_tree {
        let mut executor = ScenarioExecutor::new(ScenarioOptions::default());
        let exec_result = executor.execute(&scenario, tree);

        assert!(exec_result.completed, "Scenario execution should complete");
        assert!(!exec_result.states.is_empty(), "Should have execution states");
    }
}

/// Test flow tree construction
#[test]
fn test_flow_tree_construction() {
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(USB_SKELETON_DRIVER, "usb_skeleton.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(USB_SKELETON_DRIVER, &mut parse_result).unwrap();

    // Should have flow trees for entry points
    assert!(!result.flow_trees.is_empty(), "Should have flow trees");

    // Check that flow trees have children (call hierarchy)
    let has_children = result.flow_trees.iter()
        .any(|t| !t.children.is_empty());
    assert!(has_children, "At least one flow tree should have children");
}

/// Test call graph edges
#[test]
fn test_call_graph_edges() {
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(USB_SKELETON_DRIVER, "usb_skeleton.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(USB_SKELETON_DRIVER, &mut parse_result).unwrap();

    // Should have call edges
    assert!(!result.call_edges.is_empty(), "Should have call edges");

    // skel_probe should call various functions
    let probe_edges: Vec<_> = result.call_edges.iter()
        .filter(|e| e.caller == "skel_probe")
        .collect();

    assert!(!probe_edges.is_empty(), "skel_probe should have call edges");
}
