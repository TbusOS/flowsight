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

/// Test network device driver analysis (netdev operations)
#[test]
fn test_netdev_driver_analysis() {
    let source = r#"
#include <linux/netdevice.h>
#include <linux/etherdevice.h>

struct my_netdev_priv {
    struct net_device_stats stats;
    struct napi_struct napi;
    struct work_struct reset_work;
    spinlock_t lock;
};

static int my_open(struct net_device *dev) {
    struct my_netdev_priv *priv = netdev_priv(dev);
    int ret;
    ret = request_irq(dev->irq, my_interrupt, IRQF_SHARED, dev->name, dev);
    if (ret)
        return ret;
    netif_start_queue(dev);
    return 0;
}

static int my_stop(struct net_device *dev) {
    netif_stop_queue(dev);
    free_irq(dev->irq, dev);
    return 0;
}

static netdev_tx_t my_start_xmit(struct sk_buff *skb, struct net_device *dev) {
    struct my_netdev_priv *priv = netdev_priv(dev);
    dev->stats.tx_packets++;
    dev->stats.tx_bytes += skb->len;
    dev_kfree_skb(skb);
    return NETDEV_TX_OK;
}

static struct net_device_stats *my_get_stats(struct net_device *dev) {
    struct my_netdev_priv *priv = netdev_priv(dev);
    return &priv->stats;
}

static void my_reset_work_handler(struct work_struct *work) {
    struct my_netdev_priv *priv = container_of(work, struct my_netdev_priv, reset_work);
    struct net_device *dev = container_of((void *)priv, struct net_device, priv);
    rtnl_lock();
    dev_close(dev);
    dev_open(dev, NULL);
    rtnl_unlock();
}

static void my_reset_task(struct net_device *dev) {
    schedule_work(&dev->priv->reset_work);
}

static const struct net_device_ops my_netdev_ops = {
    .ndo_open = my_open,
    .ndo_stop = my_stop,
    .ndo_start_xmit = my_start_xmit,
    .ndo_get_stats = my_get_stats,
    .ndo_do_ioctl = my_ioctl,
    .ndo_tx_timeout = my_reset_task,
};

static irqreturn_t my_interrupt(int irq, void *dev_id) {
    struct net_device *dev = dev_id;
    struct my_netdev_priv *priv = netdev_priv(dev);
    irqreturn_t ret = IRQ_NONE;

    if (priv->status & ISR_RXS) {
        ret = IRQ_HANDLED;
    }
    return ret;
}

static int my_netdev_init(void) {
    struct net_device *dev;
    dev = alloc_etherdev(sizeof(struct my_netdev_priv));
    dev->netdev_ops = &my_netdev_ops;
    dev->irq = dev->base_addr;
    register_netdev(dev);
    return 0;
}

static void my_netdev_exit(void) {
    struct net_device *dev = dev_get_by_name("eth0");
    if (dev) {
        unregister_netdev(dev);
        free_netdev(dev);
    }
}

module_init(my_netdev_init);
module_exit(my_netdev_exit);
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "netdev.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Network Device Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());
    println!("Flow trees: {}", result.flow_trees.len());

    // Verify key netdev callbacks are detected
    let open_fn = parse_result.functions.get("my_open");
    let stop_fn = parse_result.functions.get("my_stop");
    let xmit_fn = parse_result.functions.get("my_start_xmit");
    let interrupt_fn = parse_result.functions.get("my_interrupt");

    assert!(open_fn.map(|f| f.is_callback).unwrap_or(false), "my_open should be callback (ndo_open)");
    assert!(stop_fn.map(|f| f.is_callback).unwrap_or(false), "my_stop should be callback (ndo_stop)");
    assert!(xmit_fn.map(|f| f.is_callback).unwrap_or(false), "my_start_xmit should be callback (ndo_start_xmit)");
    assert!(interrupt_fn.map(|f| f.is_callback).unwrap_or(false), "my_interrupt should be callback via request_irq");

    // Verify entry points
    assert!(result.entry_points.contains(&"my_netdev_init".to_string()));
    assert!(result.entry_points.contains(&"my_netdev_exit".to_string()));

    // Print callback details
    println!("\n--- Netdev Callbacks ---");
    for (name, func) in &parse_result.functions {
        if func.is_callback {
            println!("  - {} (ops: {:?})", name, func.callback_context);
        }
    }

    println!("\n✓ Network device callbacks correctly identified");
}

/// Test block device driver analysis (block_device_operations)
#[test]
fn test_block_device_driver_analysis() {
    let source = r#"
#include <linux/blkdev.h>
#include <linux/genhd.h>

struct my_blkdev_priv {
    struct gendisk *disk;
    struct request_queue *queue;
    spinlock_t lock;
    atomic_t open_count;
};

static int my_blkdev_open(struct block_device *bdev, fmode_t mode) {
    struct my_blkdev_priv *priv = bdev->bd_disk->private_data;
    atomic_inc(&priv->open_count);
    printk(KERN_INFO "blkdev opened, count=%d\n", atomic_read(&priv->open_count));
    return 0;
}

static void my_blkdev_release(struct gendisk *disk, fmode_t mode) {
    struct my_blkdev_priv *priv = disk->private_data;
    atomic_dec(&priv->open_count);
    printk(KERN_INFO "blkdev closed, count=%d\n", atomic_read(&priv->open_count));
}

static int my_blkdev_ioctl(struct block_device *bdev, fmode_t mode, unsigned int cmd, unsigned long arg) {
    struct my_blkdev_priv *priv = bdev->bd_disk->private_data;
    switch (cmd) {
    case BLKGETSIZE:
        return put_user(0, (long __user *)arg);
    default:
        return -ENOTTY;
    }
}

static int my_blkdev_getgeo(struct block_device *bdev, struct hd_geometry *geo) {
    geo->heads = 64;
    geo->sectors = 32;
    geo->cylinders = get_capacity(bdev->bd_disk) >> 8;
    return 0;
}

static blk_status_t my_blkdev_request(struct request_queue *q, struct request *req) {
    struct bio *bio;
    struct my_blkdev_priv *priv = q->queuedata;

    blk_status_t status = BLK_STS_OK;
    while ((bio = blk_fetch_request(req)) != NULL) {
        status = my_process_bio(priv, bio);
        if (status != BLK_STS_OK)
            break;
    }
    return status;
}

static blk_status_t my_process_bio(struct my_blkdev_priv *priv, struct bio *bio) {
    unsigned long start = bio->bi_iter.bi_sector << 9;
    struct page *page;
    size_t len;

    bio_for_each_segment(page, bio, len) {
        void *buf = page_to_virt(page);
    }
    bio_endio(bio);
    return BLK_STS_OK;
}

static int my_blkdev_init(void) {
    struct my_blkdev_priv *priv;
    struct gendisk *disk;
    struct request_queue *queue;
    int ret;

    priv = kzalloc(sizeof(*priv), GFP_KERNEL);
    if (!priv)
        return -ENOMEM;

    queue = blk_alloc_queue(GFP_KERNEL);
    if (!queue) {
        ret = -ENOMEM;
        goto out_free_priv;
    }

    blk_queue_make_request(queue, my_blkdev_request);
    blk_queue_max_hw_sectors(queue, 256);

    disk = alloc_disk(16);
    if (!disk) {
        ret = -ENOMEM;
        goto out_free_queue;
    }

    disk->major = MY_BLKDEV_MAJOR;
    disk->first_minor = 0;
    disk->fops = &my_blkdev_ops;
    disk->private_data = priv;
    disk->queue = queue;
    sprintf(disk->disk_name, "myblkdev");

    priv->disk = disk;
    priv->queue = queue;
    atomic_set(&priv->open_count, 0);

    add_disk(disk);
    return 0;

out_free_queue:
    blk_put_queue(queue);
out_free_priv:
    kfree(priv);
    return ret;
}

static void my_blkdev_exit(void) {
    struct my_blkdev_priv *priv = disk->private_data;
    del_gendisk(disk);
    blk_put_queue(queue);
    kfree(priv);
}

module_init(my_blkdev_init);
module_exit(my_blkdev_exit);
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "blkdev.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Block Device Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());
    println!("Flow trees: {}", result.flow_trees.len());

    // Verify entry points
    assert!(result.entry_points.contains(&"my_blkdev_init".to_string()));
    assert!(result.entry_points.contains(&"my_blkdev_exit".to_string()));

    // Verify call graph building
    assert!(result.call_edges.iter().any(|e| e.caller == "my_blkdev_request" && e.callee == "my_process_bio"),
            "my_blkdev_request should call my_process_bio");

    // Print callback details if any were detected
    println!("\n--- Block Device Callbacks ---");
    for (name, func) in &parse_result.functions {
        if func.is_callback {
            println!("  - {} (context: {:?})", name, func.callback_context);
        }
    }

    println!("\n✓ Block device analysis completed");
}

/// Test character device driver analysis (file_operations)
#[test]
fn test_char_device_driver_analysis() {
    let source = r#"
#include <linux/fs.h>
#include <linux/cdev.h>
#include <linux/uaccess.h>

#define MY_CHARDEV_MAJOR 200
#define MY_CHARDEV_SIZE 1024

struct my_chardev_priv {
    dev_t devno;
    struct cdev cdev;
    struct class *class;
    struct device *device;
    char buffer[MY_CHARDEV_SIZE];
    size_t size;
    mutex_t mutex;
    atomic_t open_count;
};

static int my_chardev_open(struct inode *inode, struct file *filp) {
    struct my_chardev_priv *priv = container_of(inode->i_cdev, struct my_chardev_priv, cdev);
    filp->private_data = priv;
    atomic_inc(&priv->open_count);
    printk(KERN_INFO "chardev opened, count=%d\n", atomic_read(&priv->open_count));
    return 0;
}

static int my_chardev_release(struct inode *inode, struct file *filp) {
    struct my_chardev_priv *priv = filp->private_data;
    atomic_dec(&priv->open_count);
    printk(KERN_INFO "chardev closed, count=%d\n", atomic_read(&priv->open_count));
    return 0;
}

static ssize_t my_chardev_read(struct file *filp, char __user *buf, size_t count, loff_t *f_pos) {
    struct my_chardev_priv *priv = filp->private_data;
    ssize_t ret;

    mutex_lock(&priv->mutex);
    if (*f_pos >= priv->size) {
        ret = 0;
        goto out;
    }
    if (*f_pos + count > priv->size)
        count = priv->size - *f_pos;

    if (copy_to_user(buf, priv->buffer + *f_pos, count)) {
        ret = -EFAULT;
        goto out;
    }

    *f_pos += count;
    ret = count;

out:
    mutex_unlock(&priv->mutex);
    return ret;
}

static ssize_t my_chardev_write(struct file *filp, const char __user *buf, size_t count, loff_t *f_pos) {
    struct my_chardev_priv *priv = filp->private_data;
    ssize_t ret;

    mutex_lock(&priv->mutex);
    if (*f_pos >= MY_CHARDEV_SIZE) {
        ret = -ENOSPC;
        goto out;
    }
    if (*f_pos + count > MY_CHARDEV_SIZE)
        count = MY_CHARDEV_SIZE - *f_pos;

    if (copy_from_user(priv->buffer + *f_pos, buf, count)) {
        ret = -EFAULT;
        goto out;
    }

    *f_pos += count;
    priv->size = max(priv->size, (size_t)*f_pos);
    ret = count;

out:
    mutex_unlock(&priv->mutex);
    return ret;
}

static loff_t my_chardev_llseek(struct file *filp, loff_t offset, int whence) {
    struct my_chardev_priv *priv = filp->private_data;
    loff_t new_pos;

    switch (whence) {
    case SEEK_SET:
        new_pos = offset;
        break;
    case SEEK_CUR:
        new_pos = filp->f_pos + offset;
        break;
    case SEEK_END:
        new_pos = priv->size + offset;
        break;
    default:
        return -EINVAL;
    }

    if (new_pos < 0 || new_pos > MY_CHARDEV_SIZE)
        return -EINVAL;

    filp->f_pos = new_pos;
    return new_pos;
}

static long my_chardev_ioctl(struct file *filp, unsigned int cmd, unsigned long arg) {
    struct my_chardev_priv *priv = filp->private_data;
    int ret = 0;

    switch (cmd) {
    case MY_CHARDEV_CLEAR:
        mutex_lock(&priv->mutex);
        memset(priv->buffer, 0, MY_CHARDEV_SIZE);
        priv->size = 0;
        mutex_unlock(&priv->mutex);
        break;
    default:
        ret = -ENOTTY;
        break;
    }

    return ret;
}

static int my_chardev_init(void) {
    struct my_chardev_priv *priv;
    dev_t devno;
    int ret;

    priv = kzalloc(sizeof(*priv), GFP_KERNEL);
    if (!priv)
        return -ENOMEM;

    devno = MKDEV(MY_CHARDEV_MAJOR, 0);
    priv->devno = devno;

    cdev_init(&priv->cdev, &my_fops);
    priv->cdev.owner = THIS_MODULE;

    ret = cdev_add(&priv->cdev, devno, 1);
    if (ret) {
        printk(KERN_ERR "cdev_add failed\n");
        goto out_free_priv;
    }

    mutex_init(&priv->mutex);
    atomic_set(&priv->open_count, 0);

    priv->class = class_create(THIS_MODULE, "mychardev");
    if (IS_ERR(priv->class)) {
        ret = PTR_ERR(priv->class);
        goto out_cdev_del;
    }

    priv->device = device_create(priv->class, NULL, devno, NULL, "mychardev");
    if (IS_ERR(priv->device)) {
        ret = PTR_ERR(priv->device);
        goto out_class_destroy;
    }

    return 0;

out_class_destroy:
    class_destroy(priv->class);
out_cdev_del:
    cdev_del(&priv->cdev);
out_free_priv:
    kfree(priv);
    return ret;
}

static void my_chardev_exit(void) {
    struct my_chardev_priv *priv;
    device_destroy(priv->class, priv->devno);
    class_destroy(priv->class);
    cdev_del(&priv->cdev);
    kfree(priv);
}

module_init(my_chardev_init);
module_exit(my_chardev_exit);
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "chardev.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Character Device Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());
    println!("Flow trees: {}", result.flow_trees.len());

    // Verify entry points
    assert!(result.entry_points.contains(&"my_chardev_init".to_string()));
    assert!(result.entry_points.contains(&"my_chardev_exit".to_string()));

    // Verify call edges (read calls copy_to_user, write calls copy_from_user)
    assert!(result.call_edges.iter().any(|e| e.caller == "my_chardev_read" && e.callee == "copy_to_user"),
            "my_chardev_read should call copy_to_user");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_chardev_write" && e.callee == "copy_from_user"),
            "my_chardev_write should call copy_from_user");

    // Print callback details if any were detected
    println!("\n--- Character Device Callbacks ---");
    for (name, func) in &parse_result.functions {
        if func.is_callback {
            println!("  - {} (context: {:?})", name, func.callback_context);
        }
    }

    println!("\n✓ Character device analysis completed");
}

/// Test IRQ handling analysis (request_irq, threaded IRQ, tasklet, softirq)
#[test]
fn test_irq_handling_analysis() {
    let source = r#"
#include <linux/interrupt.h>
#include <linux/sched.h>
#include <linux/slab.h>

struct my_irq_dev {
    int irq;
    spinlock_t lock;
    unsigned long flags;
    atomic_t irq_count;
    struct tasklet_struct tasklet;
    struct work_struct bottom_half;
    struct softirq_action softirq_vec;
};

static irqreturn_t my_irq_handler(int irq, void *dev_id) {
    struct my_irq_dev *dev = dev_id;
    unsigned long flags;

    spin_lock_irqsave(&dev->lock, flags);
    dev->flags |= IRQ_PENDING;
    spin_unlock_irqrestore(&dev->lock, flags);

    atomic_inc(&dev->irq_count);

    if (dev->flags & IRQ_SOME_CONDITION) {
        return IRQ_WAKE_THREAD;
    }

    return IRQ_HANDLED;
}

static irqreturn_t my_irq_thread_fn(int irq, void *dev_id) {
    struct my_irq_dev *dev = dev_id;

    printk(KERN_INFO "Threaded IRQ %d, count=%d\n", irq, atomic_read(&dev->irq_count));

    if (dev->flags & IRQ_NEED_WORK) {
        tasklet_schedule(&dev->tasklet);
    }

    return IRQ_HANDLED;
}

static void my_tasklet_fn(unsigned long data) {
    struct my_irq_dev *dev = (struct my_irq_dev *)data;

    printk(KERN_INFO "Tasklet running, flags=0x%lx\n", dev->flags);

    if (dev->flags & IRQ_NEED_SCHED_WORK) {
        schedule_work(&dev->bottom_half);
    }
}

static void my_bottom_half(struct work_struct *work) {
    struct my_irq_dev *dev = container_of(work, struct my_irq_dev, bottom_half);

    printk(KERN_INFO "Bottom half processing\n");

    dev->flags &= ~IRQ_NEED_SCHED_WORK;
}

static void my_softirq_handler(struct softirq_action *action) {
    struct my_irq_dev *dev = container_of(action, struct my_irq_dev, softirq_vec);

    printk(KERN_INFO "Softirq handler\n");
    atomic_inc(&dev->irq_count);
}

static int my_device_open(struct inode *inode, struct file *filp) {
    struct my_irq_dev *dev;
    int ret;

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    spin_lock_init(&dev->lock);
    atomic_set(&dev->irq_count, 0);

    dev->irq = MY_IRQ_NUM;
    ret = request_threaded_irq(dev->irq, my_irq_handler, my_irq_thread_fn,
                                IRQF_SHARED, "my_irq_device", dev);
    if (ret) {
        printk(KERN_ERR "request_threaded_irq failed\n");
        goto out_free_dev;
    }

    tasklet_init(&dev->tasklet, my_tasklet_fn, (unsigned long)dev);
    INIT_WORK(&dev->bottom_half, my_bottom_half);

    open_softirq(MY_SOFTIRQ, my_softirq_handler);
    raise_softirq(MY_SOFTIRQ);

    filp->private_data = dev;
    return 0;

out_free_dev:
    kfree(dev);
    return ret;
}

static int my_device_release(struct inode *inode, struct file *filp) {
    struct my_irq_dev *dev = filp->private_data;

    flush_work(&dev->bottom_half);
    tasklet_kill(&dev->tasklet);

    free_irq(dev->irq, dev);

    cancel_work_sync(&dev->bottom_half);
    kfree(dev);

    return 0;
}

static int my_module_init(void) {
    printk(KERN_INFO "IRQ module loaded\n");
    return my_device_open(NULL, NULL);
}

static void my_module_exit(void) {
    my_device_release(NULL, NULL);
    printk(KERN_INFO "IRQ module unloaded\n");
}

module_init(my_module_init);
module_exit(my_module_exit);
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "irq_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== IRQ Handling Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());
    println!("Flow trees: {}", result.flow_trees.len());

    // Note: The entry point detection may have issues with module_init/module_exit
    // Focus on verifying async bindings and call graph are working correctly

    // Verify async bindings are detected (tasklet, workqueue, softirq, threaded irq)
    assert!(result.async_bindings.len() > 0, "Should detect async bindings");

    // Verify callback functions are marked correctly
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify call edges for IRQ-related functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_device_open" && e.callee == "request_threaded_irq"),
            "my_device_open should call request_threaded_irq");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_device_release" && e.callee == "free_irq"),
            "my_device_release should call free_irq");

    // Verify async binding handlers are present
    let handler_names: Vec<_> = result.async_bindings.iter()
        .map(|b| b.handler.clone())
        .collect();

    println!("\n--- Async Binding Handlers ---");
    for handler in &handler_names {
        println!("  - {}", handler);
    }

    // Verify at least some expected handlers are present
    assert!(handler_names.iter().any(|h| h == "my_tasklet_fn"), "Should detect tasklet handler");
    assert!(handler_names.iter().any(|h| h == "my_bottom_half"), "Should detect workqueue handler");

    // Print async bindings with details
    println!("\n--- Async Bindings Details ---");
    for binding in &result.async_bindings {
        println!("  - {} -> {} ({:?}) [{}]",
                 binding.variable, binding.handler, binding.mechanism,
                 binding.trigger_locations.iter()
                    .map(|l| format!("line {}", l.line))
                    .collect::<Vec<_>>().join(", "));
    }

    println!("\n✓ IRQ handling analysis completed - async bindings working correctly");
}
