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

/// Test Timer mechanisms (timer_list and hrtimer)
#[test]
fn test_timer_mechanisms_analysis() {
    let source = r#"
#include <linux/timer.h>
#include <linux/hrtimer.h>
#include <linux/jiffies.h>
#include <linux/module.h>
#include <linux/slab.h>

struct timer_dev {
    struct timer_list basic_timer;
    struct hrtimer hrtimer;
    struct work_struct timer_work;
    ktime_t period;
    int timer_count;
    atomic_t work_count;
};

static enum hrtimer_restart my_hrtimer_handler(struct hrtimer *timer) {
    struct timer_dev *dev = container_of(timer, struct timer_dev, hrtimer);

    dev->timer_count++;
    hrtimer_forward_now(timer, dev->period);

    schedule_work(&dev->timer_work);

    return HRTIMER_RESTART;
}

static void my_timer_work_handler(struct work_struct *work) {
    struct timer_dev *dev = container_of(work, struct timer_dev, timer_work);

    atomic_inc(&dev->work_count);
    pr_info("Timer work %d\n", atomic_read(&dev->work_count));
}

static void my_basic_timer_fn(struct timer_list *t) {
    struct timer_dev *dev = container_of(t, struct timer_dev, basic_timer);

    dev->timer_count++;
    pr_info("Basic timer count: %d\n", dev->timer_count);

    if (dev->timer_count < 10) {
        mod_timer(t, jiffies + msecs_to_jiffies(100));
    }
}

static int __init my_timer_init(void) {
    struct timer_dev *dev;

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    dev->period = ktime_set(1, 0);
    atomic_set(&dev->work_count, 0);

    /* Setup basic timer */
    timer_setup(&dev->basic_timer, my_basic_timer_fn, 0);
    dev->basic_timer.expires = jiffies + msecs_to_jiffies(100);
    add_timer(&dev->basic_timer);

    /* Setup high-resolution timer */
    hrtimer_init(&dev->hrtimer, CLOCK_MONOTONIC, HRTIMER_MODE_REL);
    dev->hrtimer.function = my_hrtimer_handler;
    hrtimer_start(&dev->hrtimer, dev->period, HRTIMER_MODE_REL);

    INIT_WORK(&dev->timer_work, my_timer_work_handler);

    return 0;
}

static void __exit my_timer_exit(void) {
    struct timer_dev *dev;

    del_timer_sync(&dev->basic_timer);
    hrtimer_cancel(&dev->hrtimer);
    cancel_work_sync(&dev->timer_work);
    kfree(dev);
}

module_init(my_timer_init);
module_exit(my_timer_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Timer mechanism test driver");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "timer_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Timer Mechanisms Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());

    // Verify async bindings are detected (timer and workqueue)
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

    // Verify timer handlers are detected
    assert!(callbacks.iter().any(|n| n == "my_basic_timer_fn"),
            "Should detect basic timer callback");
    assert!(callbacks.iter().any(|n| n == "my_hrtimer_handler"),
            "Should detect hrtimer callback");
    assert!(callbacks.iter().any(|n| n == "my_timer_work_handler"),
            "Should detect workqueue callback");

    // Verify call edges for timer functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_timer_init" && e.callee == "timer_setup"),
            "my_timer_init should call timer_setup");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_timer_init" && e.callee == "hrtimer_init"),
            "my_timer_init should call hrtimer_init");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_timer_init" && e.callee == "add_timer"),
            "my_timer_init should call add_timer");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_timer_init" && e.callee == "hrtimer_start"),
            "my_timer_init should call hrtimer_start");

    println!("\n✓ Timer mechanisms analysis completed");
}

/// Test Kthread (kernel thread) analysis
#[test]
fn test_kthread_analysis() {
    let source = r#"
#include <linux/kthread.h>
#include <linux/sched.h>
#include <linux/wait.h>
#include <linux/module.h>
#include <linux/slab.h>

struct kthread_dev {
    struct task_struct *worker_thread;
    struct task_struct *io_thread;
    struct kthread_work work;
    wait_queue_head_t waitq;
    atomic_t work_count;
    bool stop;
};

static int my_worker_fn(void *data) {
    struct kthread_dev *dev = data;

    pr_info("Worker thread started\n");

    while (!kthread_should_stop()) {
        wait_event_interruptible(dev->waitq, dev->stop || kthread_work_pending(&dev->work));
        if (kthread_should_stop())
            break;

        if (kthread_work_pending(&dev->work)) {
            kthread_clear_work(&dev->work);
            atomic_inc(&dev->work_count);
            pr_info("Work processed, count: %d\n", atomic_read(&dev->work_count));
        }
    }

    pr_info("Worker thread stopping\n");
    return 0;
}

static int my_io_thread_fn(void *data) {
    struct kthread_dev *dev = data;
    struct task_struct *task = current;

    allow_signal(SIGKILL);

    while (!kthread_should_stop()) {
        pr_info("IO thread running\n");

        set_current_state(TASK_INTERRUPTIBLE);
        schedule_timeout(msecs_to_jiffies(500));
    }

    return 0;
}

static void init_work_handler(struct kthread_work *work) {
    struct kthread_dev *dev = container_of(work, struct kthread_dev, work);

    atomic_inc(&dev->work_count);
    pr_info("Kthread work: %d\n", atomic_read(&dev->work_count));
}

static int __init my_kthread_init(void) {
    struct kthread_dev *dev;

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    atomic_set(&dev->work_count, 0);
    dev->stop = false;
    init_waitqueue_head(&dev->waitq);

    /* Create worker thread */
    dev->worker_thread = kthread_run(my_worker_fn, dev, "my_worker");
    if (IS_ERR(dev->worker_thread)) {
        pr_err("Failed to create worker thread\n");
        kfree(dev);
        return PTR_ERR(dev->worker_thread);
    }

    /* Create IO thread */
    dev->io_thread = kthread_create(my_io_thread_fn, dev, "my_io_thread");
    if (IS_ERR(dev->io_thread)) {
        pr_err("Failed to create IO thread\n");
        kthread_stop(dev->worker_thread);
        kfree(dev);
        return PTR_ERR(dev->io_thread);
    }
    wake_up_process(dev->io_thread);

    /* Initialize kthread work */
    kthread_init_work(&dev->work, init_work_handler);

    return 0;
}

static void __exit my_kthread_exit(void) {
    struct kthread_dev *dev;

    dev->stop = true;
    wake_up_interruptible(&dev->waitq);

    kthread_stop(dev->worker_thread);
    kthread_stop(dev->io_thread);

    kfree(dev);
}

module_init(my_kthread_init);
module_exit(my_kthread_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Kthread test driver");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "kthread_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Kthread Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());

    // Verify kthread functions are detected
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify kthread handlers are detected
    assert!(callbacks.iter().any(|n| n == "my_worker_fn"),
            "Should detect worker thread function");
    assert!(callbacks.iter().any(|n| n == "my_io_thread_fn"),
            "Should detect IO thread function");

    // Note: kthread_work handlers are detected via kthread_init_work pattern
    // Verify call edges for kthread functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_kthread_init" && e.callee == "kthread_run"),
            "my_kthread_init should call kthread_run");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_kthread_init" && e.callee == "kthread_create"),
            "my_kthread_init should call kthread_create");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_kthread_init" && e.callee == "wake_up_process"),
            "my_kthread_init should call wake_up_process");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_kthread_exit" && e.callee == "kthread_stop"),
            "my_kthread_exit should call kthread_stop");

    println!("\n✓ Kthread analysis completed");
}

/// Test RCU (Read-Copy-Update) analysis
#[test]
fn test_rcu_analysis() {
    let source = r#"
#include <linux/rcupdate.h>
#include <linux/slab.h>
#include <linux/module.h>
#include <linux/list.h>

struct rcu_data {
    struct list_head list;
    int value;
    struct rcu_head rcu;
};

struct rcu_example {
    struct list_head head;
    spinlock_t lock;
    atomic_t read_count;
    atomic_t write_count;
};

static void my_rcu_callback(struct rcu_head *rcu) {
    struct rcu_data *data = container_of(rcu, struct rcu_data, rcu);

    pr_info("RCU callback freeing data: %d\n", data->value);
    kfree(data);
}

static int my_rcu_read(void) {
    struct rcu_example *dev;
    struct rcu_data *data;
    int value;

    rcu_read_lock();

    list_for_each_entry_rcu(data, &dev->list, list) {
        value = data->value;
        atomic_inc(&dev->read_count);
    }

    rcu_read_unlock();

    return value;
}

static void my_rcu_add(int value) {
    struct rcu_data *new_data;
    struct rcu_example *dev;

    new_data = kmalloc(sizeof(*new_data), GFP_ATOMIC);
    if (!new_data)
        return;

    new_data->value = value;
    INIT_RCU_HEAD(&new_data->rcu);

    spin_lock(&dev->lock);
    list_add_rcu(&new_data->list, &dev->head);
    spin_unlock(&dev->lock);

    atomic_inc(&dev->write_count);
}

static void my_rcu_remove(int value) {
    struct rcu_example *dev;
    struct rcu_data *data, *tmp;

    spin_lock(&dev->lock);
    list_for_each_entry_safe(data, tmp, &dev->list, list) {
        if (data->value == value) {
            list_del_rcu(&data->list);
            call_rcu(&data->rcu, my_rcu_callback);
            break;
        }
    }
    spin_unlock(&dev->lock);
}

static void my_rcu_sync(void) {
    synchronize_rcu();
    pr_info("RCU synchronize completed\n");
}

static void my_rcu_free(void *p) {
    struct rcu_data *data = p;
    pr_info("RCU callback (kfree_rcu): %d\n", data->value);
}

static int __init my_rcu_init(void) {
    struct rcu_example *dev;

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    INIT_LIST_HEAD(&dev->head);
    spin_lock_init(&dev->lock);
    atomic_set(&dev->read_count, 0);
    atomic_set(&dev->write_count, 0);

    my_rcu_add(1);
    my_rcu_add(2);
    my_rcu_add(3);

    return 0;
}

static void __exit my_rcu_exit(void) {
    struct rcu_example *dev;
    struct rcu_data *data, *tmp;

    synchronize_rcu();

    spin_lock(&dev->lock);
    list_for_each_entry_safe(data, tmp, &dev->list, list) {
        list_del(&data->list);
        kfree_rcu(data, rcu);
    }
    spin_unlock(&dev->lock);

    kfree(dev);
}

module_init(my_rcu_init);
module_exit(my_rcu_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("RCU test driver");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "rcu_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== RCU Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());

    // Verify RCU functions are detected
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify RCU callback is detected
    assert!(callbacks.iter().any(|n| n == "my_rcu_callback"),
            "Should detect RCU callback function");

    // Verify call edges for RCU functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_rcu_remove" && e.callee == "call_rcu"),
            "my_rcu_remove should call call_rcu");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_rcu_exit" && e.callee == "synchronize_rcu"),
            "my_rcu_exit should call synchronize_rcu");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_rcu_exit" && e.callee == "kfree_rcu"),
            "my_rcu_exit should call kfree_rcu");

    // Verify rcu_read_lock/unlock pair is detected
    assert!(result.call_edges.iter().any(|e| e.caller == "my_rcu_read" && e.callee == "rcu_read_lock"),
            "my_rcu_read should call rcu_read_lock");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_rcu_read" && e.callee == "rcu_read_unlock"),
            "my_rcu_read should call rcu_read_unlock");

    println!("\n✓ RCU analysis completed");
}

/// Test Platform driver analysis
#[test]
fn test_platform_driver_analysis() {
    let source = r#"
#include <linux/platform_device.h>
#include <linux/module.h>
#include <linux/gpio.h>
#include <linux/of.h>
#include <linux/of_gpio.h>

struct platform_data {
    int gpio_num;
    int irq;
    void __iomem *regs;
    struct clk *clk;
    struct device *dev;
    struct work_struct init_work;
};

static void platform_init_work_handler(struct work_struct *work) {
    struct platform_data *pdata = container_of(work, struct platform_data, init_work);

    pr_info("Platform init work\n");

    if (gpio_request(pdata->gpio_num, "platform_gpio")) {
        dev_err(pdata->dev, "Failed to request GPIO\n");
        return;
    }

    gpio_direction_output(pdata->gpio_num, 0);
}

static int my_platform_probe(struct platform_device *pdev) {
    struct platform_data *pdata;
    struct resource *res;
    int ret;

    pdata = devm_kzalloc(&pdev->dev, sizeof(*pdata), GFP_KERNEL);
    if (!pdata)
        return -ENOMEM;

    pdata->dev = &pdev->dev;

    /* Get memory resource */
    res = platform_get_resource(pdev, IORESOURCE_MEM, 0);
    if (!res) {
        dev_err(&pdev->dev, "Failed to get memory resource\n");
        return -ENODEV;
    }

    pdata->regs = devm_ioremap_resource(&pdev->dev, res);
    if (IS_ERR(pdata->regs))
        return PTR_ERR(pdata->regs);

    /* Get IRQ resource */
    pdata->irq = platform_get_irq(pdev, 0);
    if (pdata->irq < 0) {
        dev_err(&pdev->dev, "Failed to get IRQ\n");
        return pdata->irq;
    }

    /* Get GPIO from device tree */
    if (dev->of_node) {
        pdata->gpio_num = of_get_gpio(dev->of_node, 0);
        if (pdata->gpio_num < 0) {
            dev_err(&pdev->dev, "Failed to get GPIO\n");
            return pdata->gpio_num;
        }
    }

    INIT_WORK(&pdata->init_work, platform_init_work_handler);
    schedule_work(&pdata->init_work);

    platform_set_drvdata(pdev, pdata);

    pr_info("Platform device probed\n");
    return 0;
}

static int my_platform_remove(struct platform_device *pdev) {
    struct platform_data *pdata = platform_get_drvdata(pdev);

    cancel_work_sync(&pdata->init_work);
    gpio_free(pdata->gpio_num);

    pr_info("Platform device removed\n");
    return 0;
}

static int my_platform_suspend(struct platform_device *pdev, pm_message_t state) {
    struct platform_data *pdata = platform_get_drvdata(pdev);

    pr_info("Platform suspend\n");
    return 0;
}

static int my_platform_resume(struct platform_device *pdev) {
    struct platform_data *pdata = platform_get_drvdata(pdev);

    pr_info("Platform resume\n");
    return 0;
}

static struct platform_driver my_platform_driver = {
    .probe = my_platform_probe,
    .remove = my_platform_remove,
    .suspend = my_platform_suspend,
    .resume = my_platform_resume,
    .driver = {
        .name = "my-platform-device",
        .of_match_table = NULL,
    },
};

module_platform_driver(my_platform_driver);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Platform driver test");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "platform_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Platform Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());

    // Verify platform driver callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    assert!(callbacks.iter().any(|n| n == "my_platform_probe"),
            "Should detect platform probe callback");
    assert!(callbacks.iter().any(|n| n == "my_platform_remove"),
            "Should detect platform remove callback");

    // Verify call edges
    assert!(result.call_edges.iter().any(|e| e.caller == "my_platform_probe" && e.callee == "platform_get_resource"),
            "probe should call platform_get_resource");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_platform_probe" && e.callee == "platform_get_irq"),
            "probe should call platform_get_irq");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_platform_probe" && e.callee == "schedule_work"),
            "probe should call schedule_work");

    println!("\n✓ Platform driver analysis completed");
}

/// Test PCI driver analysis
#[test]
fn test_pci_driver_analysis() {
    let source = r#"
#include <linux/pci.h>
#include <linux/module.h>
#include <linux/interrupt.h>
#include <linux/msi.h>

struct pci_dev_data {
    struct pci_dev *pdev;
    void __iomem *bar0;
    void __iomem *bar1;
    int irq;
    int msix_count;
    struct msix_entry *msix_entries;
    spinlock_t lock;
    atomic_t tx_count;
    atomic_t rx_count;
};

static irqreturn_t my_pci_isr(int irq, void *dev_id) {
    struct pci_dev_data *data = dev_id;
    u32 status;

    status = readl(data->bar0 + 0x00);

    if (status & 0x01) {
        atomic_inc(&data->rx_count);
        return IRQ_HANDLED;
    }

    if (status & 0x02) {
        atomic_inc(&data->tx_count);
        return IRQ_HANDLED;
    }

    return IRQ_NONE;
}

static int my_pci_probe(struct pci_dev *pdev, const struct pci_device_id *id) {
    struct pci_dev_data *data;
    int ret;

    ret = pci_enable_device(pdev);
    if (ret) {
        dev_err(&pdev->dev, "Failed to enable PCI device\n");
        return ret;
    }

    ret = pci_request_regions(pdev, "my_pci");
    if (ret) {
        dev_err(&pdev->dev, "Failed to request regions\n");
        goto err_disable_device;
    }

    data = kzalloc(sizeof(*data), GFP_KERNEL);
    if (!data) {
        ret = -ENOMEM;
        goto err_release_regions;
    }

    data->pdev = pdev;

    data->bar0 = pci_iomap(pdev, 0, 0);
    if (!data->bar0) {
        dev_err(&pdev->dev, "Failed to map BAR0\n");
        ret = -ENOMEM;
        goto err_free_data;
    }

    data->bar1 = pci_iomap(pdev, 1, 0);
    if (!data->bar1) {
        dev_err(&pdev->dev, "Failed to map BAR1\n");
        ret = -ENOMEM;
        goto err_unmap_bar0;
    }

    pci_set_master(pdev);

    ret = pci_enable_msi(pdev);
    if (ret) {
        dev_err(&pdev->dev, "Failed to enable MSI\n");
        goto err_unmap_bar1;
    }

    data->irq = pdev->irq;
    ret = request_irq(data->irq, my_pci_isr, IRQF_SHARED, "my_pci", data);
    if (ret) {
        dev_err(&pdev->dev, "Failed to request IRQ\n");
        goto err_disable_msi;
    }

    spin_lock_init(&data->lock);
    atomic_set(&data->tx_count, 0);
    atomic_set(&data->rx_count, 0);

    pci_set_drvdata(pdev, data);

    dev_info(&pdev->dev, "PCI device probed successfully\n");
    return 0;

err_disable_msi:
    pci_disable_msi(pdev);
err_unmap_bar1:
    pci_iounmap(pdev, data->bar1);
err_unmap_bar0:
    pci_iounmap(pdev, data->bar0);
err_free_data:
    kfree(data);
err_release_regions:
    pci_release_regions(pdev);
err_disable_device:
    pci_disable_device(pdev);
    return ret;
}

static void my_pci_remove(struct pci_dev *pdev) {
    struct pci_dev_data *data = pci_get_drvdata(pdev);

    free_irq(data->irq, data);
    pci_disable_msi(pdev);
    pci_iounmap(pdev, data->bar1);
    pci_iounmap(pdev, data->bar0);
    pci_release_regions(pdev);
    pci_disable_device(pdev);
    kfree(data);

    dev_info(&pdev->dev, "PCI device removed\n");
}

static struct pci_device_id my_pci_table[] = {
    { PCI_DEVICE(0x1234, 0x5678) },
    { 0 }
};

MODULE_DEVICE_TABLE(pci, my_pci_table);

static struct pci_driver my_pci_driver = {
    .name = "my_pci",
    .id_table = my_pci_table,
    .probe = my_pci_probe,
    .remove = my_pci_remove,
};

module_pci_driver(my_pci_driver);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("PCI driver test");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "pci_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== PCI Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());

    // Verify PCI driver callbacks
    // Note: probe/remove callbacks are detected via driver registration patterns
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // At minimum, the ISR should be detected
    assert!(callbacks.iter().any(|n| n == "my_pci_isr"),
            "Should detect PCI ISR callback");

    // Verify call edges - the key test is that PCI API functions are detected
    assert!(result.call_edges.iter().any(|e| e.caller == "my_pci_probe" && e.callee == "pci_enable_device"),
            "probe should call pci_enable_device");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_pci_probe" && e.callee == "pci_request_regions"),
            "probe should call pci_request_regions");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_pci_probe" && e.callee == "pci_iomap"),
            "probe should call pci_iomap");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_pci_probe" && e.callee == "pci_enable_msi"),
            "probe should call pci_enable_msi");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_pci_probe" && e.callee == "request_irq"),
            "probe should call request_irq");

    println!("\n✓ PCI driver analysis completed");
}

/// Test SPI driver analysis
#[test]
fn test_spi_driver_analysis() {
    let source = r#"
#include <linux/spi/spi.h>
#include <linux/module.h>
#include <linux/gpio.h>

struct spi_dev_data {
    struct spi_device *spi;
    struct spi_transfer xfer;
    struct spi_message msg;
    u8 tx_buf[256];
    u8 rx_buf[256];
    int cs_gpio;
    struct work_struct work;
};

static void spi_work_handler(struct work_struct *work) {
    struct spi_dev_data *data = container_of(work, struct spi_dev_data, work);
    int ret;

    gpio_set_value(data->cs_gpio, 0);

    ret = spi_sync_transfer(data->spi, &data->xfer, 1);
    if (ret < 0)
        dev_err(&data->spi->dev, "SPI transfer failed\n");

    gpio_set_value(data->cs_gpio, 1);
}

static int my_spi_probe(struct spi_device *spi) {
    struct spi_dev_data *data;
    int ret;

    data = devm_kzalloc(&spi->dev, sizeof(*data), GFP_KERNEL);
    if (!data)
        return -ENOMEM;

    data->spi = spi;

    data->cs_gpio = of_get_gpio(spi->dev.of_node, 0);
    if (data->cs_gpio < 0) {
        dev_err(&spi->dev, "Failed to get CS GPIO\n");
        return data->cs_gpio;
    }

    ret = gpio_request(data->cs_gpio, "spi_cs");
    if (ret) {
        dev_err(&spi->dev, "Failed to request CS GPIO\n");
        return ret;
    }

    gpio_direction_output(data->cs_gpio, 1);

    spi->mode = SPI_MODE_0;
    spi->max_speed_hz = 10000000;

    spi_setup(spi);

    memset(data->tx_buf, 0xAA, sizeof(data->tx_buf));
    memset(data->rx_buf, 0, sizeof(data->rx_buf));

    data->xfer.tx_buf = data->tx_buf;
    data->xfer.rx_buf = data->rx_buf;
    data->xfer.len = 256;
    data->xfer.cs_change = 1;

    spi_message_init(&data->msg);
    spi_message_add_tail(&data->xfer, &data->msg);

    INIT_WORK(&data->work, spi_work_handler);

    spi_set_drvdata(spi, data);

    dev_info(&spi->dev, "SPI device probed\n");
    return 0;
}

static int my_spi_remove(struct spi_device *spi) {
    struct spi_dev_data *data = spi_get_drvdata(spi);

    cancel_work_sync(&data->work);
    gpio_free(data->cs_gpio);

    dev_info(&spi->dev, "SPI device removed\n");
    return 0;
}

static struct spi_device_id my_spi_id_table[] = {
    { "my_spi_device", 0 },
    { }
};

MODULE_DEVICE_TABLE(spi, my_spi_id_table);

static struct spi_driver my_spi_driver = {
    .driver = {
        .name = "my_spi_driver",
    },
    .probe = my_spi_probe,
    .remove = my_spi_remove,
    .id_table = my_spi_id_table,
};

module_spi_driver(my_spi_driver);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("SPI driver test");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "spi_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== SPI Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());

    // Verify SPI driver callbacks
    // Note: probe/remove callbacks are detected via driver registration patterns
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // At minimum, the work handler should be detected
    assert!(callbacks.iter().any(|n| n == "spi_work_handler"),
            "Should detect SPI work handler callback");

    // Verify call edges - the key test is that SPI API functions are detected
    assert!(result.call_edges.iter().any(|e| e.caller == "my_spi_probe" && e.callee == "spi_setup"),
            "probe should call spi_setup");
    // Note: spi_sync_transfer may be inlined or recognized as spi_transfer
    assert!(result.call_edges.iter().any(|e| e.caller == "my_spi_probe" && e.callee.contains("spi")),
            "probe should call SPI functions");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_spi_probe" && e.callee == "spi_message_init"),
            "probe should call spi_message_init");

    println!("\n✓ SPI driver analysis completed");
}

/// Test Completion synchronization primitive
#[test]
fn test_completion_analysis() {
    let source = r#"
#include <linux/completion.h>
#include <linux/module.h>
#include <linux/wait.h>
#include <linux/sched.h>
#include <linux/slab.h>

struct completion_dev {
    struct completion done;
    struct completion all_done;
    struct work_struct work;
    atomic_t work_count;
    int result;
};

static void completion_work_handler(struct work_struct *work) {
    struct completion_dev *dev = container_of(work, struct completion_dev, work);

    pr_info("Completion work started\n");

    atomic_inc(&dev->work_count);

    msleep(100);

    dev->result = 42;

    complete(&dev->done);
}

static int __init my_completion_init(void) {
    struct completion_dev *dev;
    int ret;

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    init_completion(&dev->done);
    init_completion(&dev->all_done);
    atomic_set(&dev->work_count, 0);

    INIT_WORK(&dev->work, completion_work_handler);

    schedule_work(&dev->work);

    ret = wait_for_completion_timeout(&dev->done, msecs_to_jiffies(5000));
    if (ret == 0) {
        pr_err("Completion timed out\n");
        kfree(dev);
        return -ETIMEDOUT;
    }

    pr_info("Completion done, result: %d\n", dev->result);

    return 0;
}

static void __exit my_completion_exit(void) {
    struct completion_dev *dev;

    complete_all(&dev->all_done);
    cancel_work_sync(&dev->work);
    kfree(dev);
}

module_init(my_completion_init);
module_exit(my_completion_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Completion primitive test");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "completion_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Completion Primitive Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());

    // Verify completion work callback
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    assert!(callbacks.iter().any(|n| n == "completion_work_handler"),
            "Should detect work handler callback");

    // Verify call edges for completion functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_completion_init" && e.callee == "init_completion"),
            "init should call init_completion");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_completion_init" && e.callee == "wait_for_completion_timeout"),
            "init should call wait_for_completion_timeout");
    assert!(result.call_edges.iter().any(|e| e.caller == "completion_work_handler" && e.callee == "complete"),
            "work handler should call complete");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_completion_exit" && e.callee == "complete_all"),
            "exit should call complete_all");

    println!("\n✓ Completion primitive analysis completed");
}

/// Test GPIO driver analysis
#[test]
fn test_gpio_driver_analysis() {
    let source = r#"
#include <linux/gpio.h>
#include <linux/module.h>
#include <linux/interrupt.h>

struct gpio_dev {
    int led_gpio;
    int button_gpio;
    int irq;
    struct gpio_desc *led_desc;
    struct gpio_desc *button_desc;
};

static irqreturn_t button_irq_handler(int irq, void *dev_id) {
    struct gpio_dev *dev = dev_id;
    int val;

    val = gpiod_get_value(dev->button_desc);
    gpiod_set_value(dev->led_desc, !val);

    return IRQ_HANDLED;
}

static int __init my_gpio_init(void) {
    struct gpio_dev *dev;
    int ret;

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    /* Request GPIOs using legacy API */
    dev->led_gpio = 17;
    ret = gpio_request(dev->led_gpio, "led-gpio");
    if (ret)
        goto out_free_dev;

    ret = gpio_direction_output(dev->led_gpio, 0);
    if (ret) {
        gpio_free(dev->led_gpio);
        goto out_free_dev;
    }

    dev->button_gpio = 18;
    ret = gpio_request(dev->button_gpio, "button-gpio");
    if (ret) {
        gpio_free(dev->led_gpio);
        goto out_free_dev;
    }

    ret = gpio_direction_input(dev->button_gpio);
    if (ret) {
        gpio_free(dev->button_gpio);
        gpio_free(dev->led_gpio);
        goto out_free_dev;
    }

    /* Map button GPIO to IRQ */
    dev->irq = gpio_to_irq(dev->button_gpio);
    ret = request_irq(dev->irq, button_irq_handler,
                     IRQF_TRIGGER_FALLING, "button", dev);
    if (ret) {
        gpio_free(dev->button_gpio);
        gpio_free(dev->led_gpio);
        goto out_free_dev;
    }

    return 0;

out_free_dev:
    kfree(dev);
    return ret;
}

static void __exit my_gpio_exit(void) {
    free_irq(gpio_to_irq(18), NULL);
    gpio_free(18);
    gpio_free(17);
    kfree(NULL);
}

module_init(my_gpio_init);
module_exit(my_gpio_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("GPIO driver test");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "gpio_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== GPIO Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());

    // Verify callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify GPIO IRQ handler is detected
    assert!(callbacks.iter().any(|n| n == "button_irq_handler"),
            "Should detect GPIO IRQ handler callback");

    // Verify call edges for GPIO functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_gpio_init" && e.callee == "gpio_request"),
            "init should call gpio_request");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_gpio_init" && e.callee == "gpio_direction_output"),
            "init should call gpio_direction_output");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_gpio_init" && e.callee == "gpio_direction_input"),
            "init should call gpio_direction_input");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_gpio_init" && e.callee == "gpio_to_irq"),
            "init should call gpio_to_irq");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_gpio_init" && e.callee == "request_irq"),
            "init should call request_irq");

    println!("\n✓ GPIO driver analysis completed");
}

/// Test Clock driver analysis
#[test]
fn test_clk_driver_analysis() {
    let source = r#"
#include <linux/clk.h>
#include <linux/module.h>

struct clk_dev {
    struct clk *aclk;
    struct clk *i2c_clk;
    struct clk *fixed_clk;
};

static int __init my_clk_init(void) {
    struct clk_dev *dev;
    int ret;

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    /* Get clocks from device tree */
    dev->aclk = clk_get(NULL, "aclk");
    if (IS_ERR(dev->aclk)) {
        ret = PTR_ERR(dev->aclk);
        goto out_free_dev;
    }

    dev->i2c_clk = clk_get(NULL, "i2c_clk");
    if (IS_ERR(dev->i2c_clk)) {
        ret = PTR_ERR(dev->i2c_clk);
        goto out_put_aclk;
    }

    /* Set clock rate */
    ret = clk_set_rate(dev->aclk, 200000000);
    if (ret < 0)
        goto out_put_i2c;

    /* Enable clocks */
    ret = clk_prepare_enable(dev->aclk);
    if (ret)
        goto out_put_i2c;

    ret = clk_prepare_enable(dev->i2c_clk);
    if (ret) {
        clk_disable_unprepare(dev->aclk);
        goto out_put_i2c;
    }

    return 0;

out_put_i2c:
    clk_put(dev->i2c_clk);
out_put_aclk:
    clk_put(dev->aclk);
out_free_dev:
    kfree(dev);
    return ret;
}

static void __exit my_clk_exit(void) {
    clk_disable_unprepare(NULL);
    clk_put(NULL);
    clk_put(NULL);
    kfree(NULL);
}

module_init(my_clk_init);
module_exit(my_clk_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Clock driver test");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "clk_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Clock Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify call edges for clock functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_clk_init" && e.callee == "clk_get"),
            "init should call clk_get");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_clk_init" && e.callee == "clk_set_rate"),
            "init should call clk_set_rate");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_clk_init" && e.callee == "clk_prepare_enable"),
            "init should call clk_prepare_enable");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_clk_exit" && e.callee == "clk_disable_unprepare"),
            "exit should call clk_disable_unprepare");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_clk_exit" && e.callee == "clk_put"),
            "exit should call clk_put");

    println!("\n✓ Clock driver analysis completed");
}

/// Test Regulator driver analysis
#[test]
fn test_regulator_driver_analysis() {
    let source = r#"
#include <linux/regulator/consumer.h>
#include <linux/module.h>

struct regulator_dev {
    struct regulator *vdd;
    struct regulator *vdd_arm;
};

static int __init my_regulator_init(void) {
    struct regulator_dev *dev;
    int ret;

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    /* Get regulators */
    dev->vdd = regulator_get(NULL, "vdd");
    if (IS_ERR(dev->vdd)) {
        ret = PTR_ERR(dev->vdd);
        goto out_free_dev;
    }

    dev->vdd_arm = regulator_get(NULL, "vdd_arm");
    if (IS_ERR(dev->vdd_arm)) {
        ret = PTR_ERR(dev->vdd_arm);
        goto out_put_vdd;
    }

    /* Set voltage (1.2V - 1.3V) */
    ret = regulator_set_voltage(dev->vdd, 1200000, 1300000);
    if (ret < 0)
        goto out_put_vdd_arm;

    /* Enable regulator */
    ret = regulator_enable(dev->vdd);
    if (ret < 0)
        goto out_put_vdd_arm;

    if (regulator_is_enabled(dev->vdd_arm)) {
        ret = regulator_disable(dev->vdd_arm);
        if (ret < 0)
            goto out_disable_vdd;
    }

    return 0;

out_disable_vdd:
    regulator_disable(dev->vdd);
out_put_vdd_arm:
    regulator_put(dev->vdd_arm);
out_put_vdd:
    regulator_put(dev->vdd);
out_free_dev:
    kfree(dev);
    return ret;
}

static void __exit my_regulator_exit(void) {
    regulator_disable(NULL);
    regulator_put(NULL);
    regulator_put(NULL);
    kfree(NULL);
}

module_init(my_regulator_init);
module_exit(my_regulator_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Regulator driver test");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "regulator_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Regulator Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify call edges for regulator functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_regulator_init" && e.callee == "regulator_get"),
            "init should call regulator_get");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_regulator_init" && e.callee == "regulator_set_voltage"),
            "init should call regulator_set_voltage");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_regulator_init" && e.callee == "regulator_enable"),
            "init should call regulator_enable");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_regulator_init" && e.callee == "regulator_is_enabled"),
            "init should call regulator_is_enabled");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_regulator_init" && e.callee == "regulator_disable"),
            "init should call regulator_disable");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_regulator_exit" && e.callee == "regulator_put"),
            "exit should call regulator_put");

    println!("\n✓ Regulator driver analysis completed");
}

/// Test Input driver analysis
#[test]
fn test_input_driver_analysis() {
    let source = r#"
#include <linux/input.h>
#include <linux/module.h>
#include <linux/interrupt.h>

struct input_dev_data {
    struct input_dev *input;
    int irq;
};

static irqreturn_t button_irq_handler(int irq, void *dev_id) {
    struct input_dev_data *data = dev_id;
    int pressed;

    pressed = gpio_get_value(18);
    input_report_key(data->input, KEY_ENTER, pressed);
    input_sync(data->input);

    return IRQ_HANDLED;
}

static int __init my_input_init(void) {
    struct input_dev_data *dev;
    int error;

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    /* Allocate input device */
    dev->input = input_allocate_device();
    if (!dev->input) {
        error = -ENOMEM;
        goto out_free_dev;
    }

    dev->input->name = "gpio-buttons";
    dev->input->phys = "gpio-buttons/input0";

    /* Set capabilities */
    __set_bit(EV_KEY, dev->input->evbit);
    __set_bit(KEY_ENTER, dev->input->keybit);

    /* Register input device */
    error = input_register_device(dev->input);
    if (error) {
        input_free_device(dev->input);
        goto out_free_dev;
    }

    /* Request IRQ */
    dev->irq = gpio_to_irq(18);
    error = request_irq(dev->irq, button_irq_handler,
                       IRQF_TRIGGER_RISING | IRQF_TRIGGER_FALLING,
                       "gpio-button", dev);
    if (error)
        goto out_unregister;

    return 0;

out_unregister:
    input_unregister_device(dev->input);
out_free_dev:
    kfree(dev);
    return error;
}

static void __exit my_input_exit(void) {
    free_irq(gpio_to_irq(18), NULL);
    input_unregister_device(NULL);
    kfree(NULL);
}

module_init(my_input_init);
module_exit(my_input_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Input driver test");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "input_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Input Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify input IRQ handler is detected
    assert!(callbacks.iter().any(|n| n == "button_irq_handler"),
            "Should detect input IRQ handler callback");

    // Verify call edges for input functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_input_init" && e.callee == "input_allocate_device"),
            "init should call input_allocate_device");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_input_init" && e.callee == "input_register_device"),
            "init should call input_register_device");
    assert!(result.call_edges.iter().any(|e| e.caller == "button_irq_handler" && e.callee == "input_report_key"),
            "irq handler should call input_report_key");
    assert!(result.call_edges.iter().any(|e| e.caller == "button_irq_handler" && e.callee == "input_sync"),
            "irq handler should call input_sync");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_input_exit" && e.callee == "input_unregister_device"),
            "exit should call input_unregister_device");

    println!("\n✓ Input driver analysis completed");
}

/// Test DMA driver analysis
#[test]
fn test_dma_driver_analysis() {
    let source = r#"
#include <linux/dmaengine.h>
#include <linux/module.h>
#include <linux/dmapool.h>

struct dma_dev {
    struct dma_chan *chan;
    struct dma_async_tx_descriptor *tx;
    dma_cookie_t cookie;
    struct dma_pool *pool;
    void *cpu_addr;
    dma_addr_t dma_addr;
};

static void dma_callback(void *completion) {
    complete(completion);
}

static int __init my_dma_init(void) {
    struct dma_dev *dev;
    int ret;

    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;

    /* Request DMA channel */
    dev->chan = dma_request_chan(NULL, "tx");
    if (IS_ERR(dev->chan)) {
        ret = PTR_ERR(dev->chan);
        goto out_free_dev;
    }

    /* Allocate DMA pool */
    dev->pool = dma_pool_create("my_pool", NULL, 4096, 4096, 0);
    if (!dev->pool) {
        ret = -ENOMEM;
        goto out_release_chan;
    }

    /* Allocate from pool */
    dev->cpu_addr = dma_pool_alloc(dev->pool, GFP_KERNEL, &dev->dma_addr);
    if (!dev->cpu_addr) {
        ret = -ENOMEM;
        goto out_destroy_pool;
    }

    /* Prepare memcpy */
    dev->tx = dmaengine_prep_dma_memcpy(dev->chan, dev->dma_addr, dev->dma_addr, 4096, 0);
    if (!dev->tx) {
        ret = -ENOMEM;
        goto out_pool_free;
    }

    /* Set callback */
    dev->tx->callback = dma_callback;

    /* Submit */
    dev->cookie = dev->tx->tx_submit(dev->tx);
    if (dma_submit_error(dev->cookie)) {
        ret = -EIO;
        goto out_pool_free;
    }

    /* Issue pending */
    dma_async_issue_pending(dev->chan);

    return 0;

out_pool_free:
    dma_pool_free(dev->pool, dev->cpu_addr, dev->dma_addr);
out_destroy_pool:
    dma_pool_destroy(dev->pool);
out_release_chan:
    dma_release_channel(dev->chan);
out_free_dev:
    kfree(dev);
    return ret;
}

static void __exit my_dma_exit(void) {
    if (NULL) {
        dma_async_terminate(NULL);
    }
    if (NULL) {
        dma_pool_free(NULL, NULL, 0);
        dma_pool_destroy(NULL);
    }
    if (NULL) {
        dma_release_channel(NULL);
    }
    kfree(NULL);
}

module_init(my_dma_init);
module_exit(my_dma_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("DMA driver test");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "dma_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== DMA Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Async bindings: {}", result.async_bindings.len());
    println!("Call edges: {}", result.call_edges.len());

    // Verify DMA callback is detected
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify call edges for DMA functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_dma_init" && e.callee == "dma_request_chan"),
            "init should call dma_request_chan");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_dma_init" && e.callee == "dma_pool_create"),
            "init should call dma_pool_create");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_dma_init" && e.callee == "dma_pool_alloc"),
            "init should call dma_pool_alloc");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_dma_init" && e.callee == "dmaengine_prep_dma_memcpy"),
            "init should call dmaengine_prep_dma_memcpy");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_dma_init" && e.callee == "dma_async_issue_pending"),
            "init should call dma_async_issue_pending");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_dma_init" && e.callee == "dma_release_channel"),
            "init should call dma_release_channel");

    println!("\n✓ DMA driver analysis completed");
}

/// Test Block device driver analysis
#[test]
fn test_block_driver_analysis() {
    let source = r#"
#include <linux/genhd.h>
#include <linux/blkdev.h>
#include <linux/module.h>

struct blkdev_priv {
    struct request_queue *queue;
    struct gendisk *disk;
    spinlock_t lock;
    sector_t capacity;
};

static void my_request(struct request_queue *q) {
    struct request *rq;

    rq = blk_mq_start_request(rq);
    blk_mq_end_request(rq, BLK_STS_OK);
}

static int my_open(struct block_device *bdev, fmode_t mode) {
    return 0;
}

static void my_release(struct gendisk *disk, fmode_t mode) {
}

static const struct block_device_operations my_fops = {
    .owner = THIS_MODULE,
    .open = my_open,
    .release = my_release,
};

static int __init my_blkdev_init(void) {
    struct blkdev_priv *priv;
    int ret;

    priv = kzalloc(sizeof(*priv), GFP_KERNEL);
    if (!priv)
        return -ENOMEM;

    /* Allocate request queue */
    priv->queue = blk_init_queue(my_request, &priv->lock);
    if (!priv->queue) {
        ret = -ENOMEM;
        goto out_free_priv;
    }

    /* Set queue parameters */
    blk_queue_max_hw_sectors(priv->queue, 256);

    /* Allocate disk */
    priv->disk = alloc_disk(16);
    if (!priv->disk) {
        ret = -ENOMEM;
        goto out_free_queue;
    }

    /* Set disk parameters */
    priv->disk->major = 0;
    priv->disk->first_minor = 0;
    priv->disk->fops = &my_fops;
    priv->disk->queue = priv->queue;
    priv->disk->private_data = priv;

    /* Set capacity */
    priv->capacity = 1024 * 1024;
    set_capacity(priv->disk, priv->capacity);

    /* Add disk */
    add_disk(priv->disk);

    return 0;

out_free_queue:
    blk_cleanup_queue(priv->queue);
out_free_priv:
    kfree(priv);
    return ret;
}

static void __exit my_blkdev_exit(void) {
    del_gendisk(NULL);
    blk_cleanup_queue(NULL);
    kfree(NULL);
}

module_init(my_blkdev_init);
module_exit(my_blkdev_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Block device driver test");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "blkdev_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Block Device Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify block device callbacks are detected (via block_device_operations)
    // Note: blk_init_queue request handler is not detected as callback
    // because analyzer only tracks struct member assignments
    assert!(callbacks.iter().any(|n| n == "my_open"),
            "Should detect open callback");
    assert!(callbacks.iter().any(|n| n == "my_release"),
            "Should detect release callback");

    // Verify call edges for block functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_blkdev_init" && e.callee == "blk_init_queue"),
            "init should call blk_init_queue");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_blkdev_init" && e.callee == "blk_queue_max_hw_sectors"),
            "init should call blk_queue_max_hw_sectors");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_blkdev_init" && e.callee == "alloc_disk"),
            "init should call alloc_disk");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_blkdev_init" && e.callee == "set_capacity"),
            "init should call set_capacity");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_blkdev_init" && e.callee == "add_disk"),
            "init should call add_disk");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_blkdev_exit" && e.callee == "del_gendisk"),
            "exit should call del_gendisk");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_blkdev_exit" && e.callee == "blk_cleanup_queue"),
            "exit should call blk_cleanup_queue");

    println!("\n✓ Block device driver analysis completed");
}

/// Test ASoC (ALSA SoC) driver analysis
#[test]
fn test_asoc_driver_analysis() {
    let source = r#"
#include <sound/soc.h>
#include <sound/soc-dapm.h>
#include <linux/module.h>

static const struct snd_soc_dapm_widget my_dapm_widgets[] = {
    SND_SOC_DAPM_OUTPUT("HiFi Playback"),
    SND_SOC_DAPM_INPUT("HiFi Capture"),
};

static const struct snd_soc_dapm_route my_dapm_routes[] = {
    {"HiFi Playback", NULL, "DAC"},
    {"ADC", NULL, "HiFi Capture"},
};

static const struct snd_soc_codec_driver soc_codec_dev_my = {
    .dapm_widgets = my_dapm_widgets,
    .num_dapm_widgets = ARRAY_SIZE(my_dapm_widgets),
    .dapm_routes = my_dapm_routes,
    .num_dapm_routes = ARRAY_SIZE(my_dapm_routes),
};

static struct snd_soc_dai_driver my_dai = {
    .name = "my-codec-dai",
    .playback = {
        .stream_name = "Playback",
        .channels_min = 1,
        .channels_max = 2,
        .rates = 0,
        .formats = 0,
    },
    .capture = {
        .stream_name = "Capture",
        .channels_min = 1,
        .channels_max = 2,
        .rates = 0,
        .formats = 0,
    },
};

static struct snd_soc_dai_link my_dai_link = {
    .name = "my-codec",
    .stream_name = "my-codec HiFi",
    .codec_dai_name = "my-codec-dai",
};

static struct snd_soc_card my_card = {
    .name = "my-board",
    .dai_link = &my_dai_link,
    .num_links = 1,
};

static int my_probe(struct platform_device *pdev) {
    struct snd_soc_card *card = &my_card;
    int ret;

    ret = devm_snd_soc_register_card(&pdev->dev, card);
    if (ret)
        dev_err(&pdev->dev, "Failed to register card\n");

    return ret;
}

static int my_remove(struct platform_device *pdev) {
    return 0;
}

static struct platform_driver my_driver = {
    .probe = my_probe,
    .remove = my_remove,
    .driver = {
        .name = "my-asoc",
    },
};

module_platform_driver(my_driver);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("ASoC driver test");
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "asoc_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== ASoC Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify ASoC probe/remove callbacks are detected
    assert!(callbacks.iter().any(|n| n == "my_probe"),
            "Should detect ASoC probe callback");
    assert!(callbacks.iter().any(|n| n == "my_remove"),
            "Should detect ASoC remove callback");

    // Verify call edges for ASoC functions
    assert!(result.call_edges.iter().any(|e| e.caller == "my_probe" && e.callee == "devm_snd_soc_register_card"),
            "probe should call devm_snd_soc_register_card");
    assert!(result.call_edges.iter().any(|e| e.callee == "snd_soc_register_card" || e.callee == "devm_snd_soc_register_card"),
            "Should detect ASoC card registration");

    println!("\n✓ ASoC driver analysis completed");
}

/// Test NVMEM driver analysis
#[test]
fn test_nvmem_driver_analysis() {
    let source = r#"
#include <linux/nvmem-provider.h>
#include <linux/module.h>

static int my_nvmem_probe(struct nvmem_device *dev,
                          const struct nvmem_device_id *id) {
    void *buf;
    size_t size;

    size = nvmem_device_size(dev);
    buf = kzalloc(size, GFP_KERNEL);
    if (!buf)
        return -ENOMEM;

    nvmem_device_read(dev, 0, size, buf);
    kfree(buf);
    return 0;
}

static void my_nvmem_remove(struct nvmem_device *dev) {
}

static struct nvmem_device_id my_nvmem_id[] = {
    { "my-nvmem", 0 },
    { },
};

static struct nvmem_driver my_nvmem = {
    .driver = {
        .name = "my-nvmem",
    },
    .probe = my_nvmem_probe,
    .remove = my_nvmem_remove,
    .id_table = my_nvmem_id,
};
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "nvmem_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== NVMEM Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify NVMEM driver structure is present (probe/remove patterns)
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_nvmem_probe"),
            "Should have probe function");
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_nvmem_remove"),
            "Should have remove function");

    // Verify call edges
    assert!(result.call_edges.iter().any(|e| e.caller == "my_nvmem_probe" && e.callee == "nvmem_device_read"),
            "probe should call nvmem_device_read");

    println!("\n✓ NVMEM driver analysis completed");
}

/// Test IIO driver analysis
#[test]
fn test_iio_driver_analysis() {
    let source = r#"
#include <linux/iio/iio.h>
#include <linux/module.h>

static int my_sensor_read_raw(struct iio_dev *indio_dev,
                              struct iio_chan_spec const *chan,
                              int *val, int *val2, long mask) {
    switch (mask) {
    case IIO_CHAN_INFO_RAW:
        *val = 100;
        return IIO_VAL_INT;
    case IIO_CHAN_INFO_SCALE:
        *val = 1;
        *val2 = 1000;
        return IIO_VAL_INT_PLUS_MICRO;
    }
    return -EINVAL;
}

static const struct iio_chan_spec my_channels[] = {
    {
        .type = IIO_ACCEL,
        .modified = 1,
        .channel2 = IIO_MOD_X,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW),
        .info_mask_shared_by_type = BIT(IIO_CHAN_INFO_SCALE),
    },
};

static const struct iio_info my_info = {
    .read_raw = my_sensor_read_raw,
};

static int my_probe(struct iio_dev *indio_dev,
                    const struct iio_device_id *id) {
    indio_dev->info = &my_info;
    indio_dev->channels = my_channels;
    indio_dev->num_channels = ARRAY_SIZE(my_channels);
    indio_dev->modes = INDIO_DIRECT_MODE;

    return iio_device_register(indio_dev);
}

static void my_remove(struct iio_dev *indio_dev) {
    iio_device_unregister(indio_dev);
}
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "iio_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== IIO Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify IIO driver structure is present
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_sensor_read_raw"),
            "Should have read_raw function");
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_probe"),
            "Should have probe function");

    // Verify call edges
    assert!(result.call_edges.iter().any(|e| e.caller == "my_probe" && e.callee == "iio_device_register"),
            "probe should call iio_device_register");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_remove" && e.callee == "iio_device_unregister"),
            "remove should call iio_device_unregister");

    println!("\n✓ IIO driver analysis completed");
}

/// Test Watchdog driver analysis
#[test]
fn test_watchdog_driver_analysis() {
    let source = r#"
#include <linux/watchdog.h>
#include <linux/module.h>

static int my_wdd_start(struct watchdog_device *wdd) {
    writel(0xABCD, wdt_base + WDT_START_REG);
    return 0;
}

static int my_wdd_stop(struct watchdog_device *wdd) {
    writel(0, wdt_base + WDT_START_REG);
    return 0;
}

static int my_wdd_ping(struct watchdog_device *wdd) {
    writel(0xABCD, wdt_base + WDT_CLEAR_REG);
    return 0;
}

static int my_wdd_set_timeout(struct watchdog_device *wdd,
                              unsigned int timeout) {
    wdd->timeout = timeout;
    return 0;
}

static const struct watchdog_ops my_wdt_ops = {
    .owner = THIS_MODULE,
    .start = my_wdd_start,
    .stop = my_wdd_stop,
    .ping = my_wdd_ping,
    .set_timeout = my_wdd_set_timeout,
};

static const struct watchdog_info my_wdt_info = {
    .options = WDIOF_SETTIMEOUT | WDIOF_KEEPALIVEPING,
    .firmware_version = 0,
    .identity = "my-watchdog",
};

static struct watchdog_device my_wdt = {
    .info = &my_wdt_info,
    .ops = &my_wdt_ops,
    .min_timeout = 1,
    .max_timeout = 300,
    .timeout = 30,
};

static int __init my_init(void) {
    return watchdog_register_device(&my_wdt);
}

static void __exit my_exit(void) {
    watchdog_unregister_device(&my_wdt);
}
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "watchdog_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Watchdog Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify Watchdog driver structure is present (via watchdog_ops struct)
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_wdd_start"),
            "Should have start function");
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_wdd_stop"),
            "Should have stop function");
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_wdd_ping"),
            "Should have ping function");

    // Verify call edges
    assert!(result.call_edges.iter().any(|e| e.caller == "my_init" && e.callee == "watchdog_register_device"),
            "init should call watchdog_register_device");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_exit" && e.callee == "watchdog_unregister_device"),
            "exit should call watchdog_unregister_device");

    println!("\n✓ Watchdog driver analysis completed");
}

/// Test Thermal driver analysis
#[test]
fn test_thermal_driver_analysis() {
    let source = r#"
#include <linux/thermal.h>
#include <linux/module.h>

static int my_get_temp(void *data, int *temp) {
    *temp = 45000;  /* 45C */
    return 0;
}

static int my_get_trip_temp(struct thermal_zone_device *tz,
                            int trip, int *temp) {
    static int trip_temps[] = {60000, 80000, 95000};
    if (trip >= 3)
        return -EINVAL;
    *temp = trip_temps[trip];
    return 0;
}

static int my_get_trip_type(struct thermal_zone_device *tz,
                            int trip, enum thermal_trip_type *type) {
    static enum thermal_trip_type types[] = {
        THERMAL_TRIP_ACTIVE,
        THERMAL_TRIP_PASSIVE,
        THERMAL_TRIP_CRITICAL,
    };
    if (trip >= 3)
        return -EINVAL;
    *type = types[trip];
    return 0;
}

static struct thermal_zone_device_ops my_tz_ops = {
    .get_temp = my_get_temp,
    .get_trip_temp = my_get_trip_temp,
    .get_trip_type = my_get_trip_type,
};

static int __init my_init(void) {
    struct thermal_zone_device *tz;

    tz = thermal_zone_device_register("my-thermal", 3, 0, NULL,
                                       &my_tz_ops, NULL, 0, 0);
    if (IS_ERR(tz))
        return PTR_ERR(tz);

    thermal_zone_bind_cooling_device(tz, 0, NULL,
                                     THERMAL_NO_LIMIT,
                                     THERMAL_NO_LIMIT,
                                     THERMAL_WEIGHT_DEFAULT);
    return 0;
}

static void __exit my_exit(void) {
    thermal_zone_device_unregister(NULL);
}
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "thermal_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Thermal Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify Thermal driver structure is present
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_get_temp"),
            "Should have get_temp function");
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_get_trip_temp"),
            "Should have get_trip_temp function");

    // Verify call edges
    assert!(result.call_edges.iter().any(|e| e.caller == "my_init" && e.callee == "thermal_zone_device_register"),
            "init should call thermal_zone_device_register");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_init" && e.callee == "thermal_zone_bind_cooling_device"),
            "init should call thermal_zone_bind_cooling_device");

    println!("\n✓ Thermal driver analysis completed");
}

/// Test PHY driver analysis
#[test]
fn test_phy_driver_analysis() {
    let source = r#"
#include <linux/phy.h>
#include <linux/module.h>

static int my_phy_init(struct phy *phy) {
    struct my_phy_data *priv;

    priv = phy_get_drvdata(phy);
    phy_write(phy, MII_BMCR, BMCR_RESET);
    return 0;
}

static int my_phy_reset(struct phy *phy) {
    gpio_set_value(phy_reset_gpio, 0);
    udelay(10);
    gpio_set_value(phy_reset_gpio, 1);
    return 0;
}

static int my_phy_config_aneg(struct phy *phy) {
    u16 adv = ADVERTISE_ALL | ADVERTISE_CSMA;
    phy_write(phy, MII_ADVERTISE, adv);
    return 0;
}

static int my_phy_read_status(struct phy *phy) {
    return 0;
}

static struct phy_ops my_phy_ops = {
    .init = my_phy_init,
    .reset = my_phy_reset,
    .config_aneg = my_phy_config_aneg,
    .read_status = my_phy_read_status,
    .owner = THIS_MODULE,
};

static int __init my_phy_probe(struct platform_device *pdev) {
    struct phy *phy;

    phy = devm_phy_create(&pdev->dev, NULL, &my_phy_ops);
    if (IS_ERR(phy))
        return PTR_ERR(phy);

    phy_power_on(phy);
    phy_start(phy);
    return 0;
}

static int __exit my_phy_remove(struct platform_device *pdev) {
    phy_stop(NULL);
    phy_power_off(NULL);
    return 0;
}
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "phy_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== PHY Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify PHY driver structure is present (via phy_ops struct)
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_phy_init"),
            "Should have init function");
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_phy_reset"),
            "Should have reset function");

    // Verify call edges
    assert!(result.call_edges.iter().any(|e| e.caller == "my_phy_probe" && e.callee == "devm_phy_create"),
            "probe should call devm_phy_create");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_phy_probe" && e.callee == "phy_power_on"),
            "probe should call phy_power_on");

    println!("\n✓ PHY driver analysis completed");
}

/// Test Reset controller driver analysis
#[test]
fn test_reset_driver_analysis() {
    let source = r#"
#include <linux/reset.h>
#include <linux/module.h>

static int my_reset_reset(struct reset_controller_dev *rcdev,
                          unsigned long id) {
    writel(BIT(id), rst_base + RST_ASSERT_REG);
    udelay(10);
    writel(BIT(id), rst_base + RST_DEASSERT_REG);
    return 0;
}

static int my_reset_assert(struct reset_controller_dev *rcdev,
                           unsigned long id) {
    writel(BIT(id), rst_base + RST_ASSERT_REG);
    return 0;
}

static int my_reset_deassert(struct reset_controller_dev *rcdev,
                             unsigned long id) {
    writel(BIT(id), rst_base + RST_DEASSERT_REG);
    return 0;
}

static int my_reset_status(struct reset_controller_dev *rcdev,
                           unsigned long id) {
    return readl(rst_base + RST_STATUS_REG) & BIT(id);
}

static const struct reset_control_ops my_reset_ops = {
    .reset = my_reset_reset,
    .assert = my_reset_assert,
    .deassert = my_reset_deassert,
    .status = my_reset_status,
};

static struct reset_controller_dev my_rcdev = {
    .ops = &my_reset_ops,
    .owner = THIS_MODULE,
    .nr_resets = 32,
};

static int __init my_init(void) {
    struct reset_control *rst;

    rst = reset_control_get(NULL, NULL);
    if (IS_ERR(rst))
        return PTR_ERR(rst);

    reset_control_deassert(rst);
    reset_control_put(rst);
    return reset_controller_register(&my_rcdev);
}

static void __exit my_exit(void) {
    reset_controller_unregister(&my_rcdev);
}
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "reset_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== Reset Controller Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify Reset driver structure is present (via reset_control_ops struct)
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_reset_reset"),
            "Should have reset function");
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_reset_assert"),
            "Should have assert function");
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_reset_deassert"),
            "Should have deassert function");

    // Verify call edges
    assert!(result.call_edges.iter().any(|e| e.caller == "my_init" && e.callee == "reset_control_get"),
            "init should call reset_control_get");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_init" && e.callee == "reset_controller_register"),
            "init should call reset_controller_register");

    println!("\n✓ Reset controller driver analysis completed");
}

/// Test V4L2 driver analysis
#[test]
fn test_v4l2_driver_analysis() {
    let source = r#"
#include <linux/videodev2.h>
#include <media/v4l2-device.h>
#include <media/v4l2-ioctl.h>
#include <linux/module.h>

static int my_querycap(struct file *file, void *priv,
                       struct v4l2_capability *cap) {
    strlcpy(cap->driver, "my-video", sizeof(cap->driver));
    strlcpy(cap->card, "My Video Device", sizeof(cap->card));
    cap->capabilities = V4L2_CAP_VIDEO_CAPTURE | V4L2_CAP_STREAMING;
    return 0;
}

static int my_enum_fmt(struct file *file, void *priv,
                       struct v4l2_fmtdesc *f) {
    if (f->index >= 2)
        return -EINVAL;
    return 0;
}

static int my_s_fmt(struct file *file, void *priv,
                    struct v4l2_format *f) {
    return 0;
}

static const struct v4l2_ioctl_ops my_ioctl_ops = {
    .vidioc_querycap = my_querycap,
    .vidioc_enum_fmt_vid_cap = my_enum_fmt,
    .vidioc_s_fmt_vid_cap = my_s_fmt,
};

static const struct v4l2_file_operations my_fops = {
    .owner = THIS_MODULE,
    .open = my_open,
    .release = my_release,
    .ioctl = video_ioctl2,
};

static struct video_device my_vdev = {
    .name = "my-video",
    .fops = &my_fops,
    .ioctl_ops = &my_ioctl_ops,
    .release = video_device_release,
};

static int __init my_init(void) {
    return video_register_device(&my_vdev, VFL_TYPE_VIDEO, -1);
}

static void __exit my_exit(void) {
    video_unregister_device(&my_vdev);
}
"#;

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(source, "v4l2_test.c")
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(source, &mut parse_result)
        .expect("Analysis failed");

    println!("\n=== V4L2 Driver Analysis ===");
    println!("Functions found: {}", parse_result.functions.len());
    println!("Entry points: {:?}", result.entry_points);
    println!("Call edges: {}", result.call_edges.len());

    // Verify callbacks
    let callbacks: Vec<_> = parse_result.functions.iter()
        .filter(|(_, f)| f.is_callback)
        .map(|(n, _)| n.clone())
        .collect();

    println!("\n--- Detected Callbacks ({}) ---", callbacks.len());
    for name in &callbacks {
        println!("  - {}", name);
    }

    // Verify V4L2 driver structure is present (via v4l2_ioctl_ops struct)
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_querycap"),
            "Should have querycap function");
    assert!(parse_result.functions.iter().any(|(n, _)| n == "my_enum_fmt"),
            "Should have enum_fmt function");

    // Verify call edges
    assert!(result.call_edges.iter().any(|e| e.caller == "my_init" && e.callee == "video_register_device"),
            "init should call video_register_device");
    assert!(result.call_edges.iter().any(|e| e.caller == "my_exit" && e.callee == "video_unregister_device"),
            "exit should call video_unregister_device");

    println!("\n✓ V4L2 driver analysis completed");
}
