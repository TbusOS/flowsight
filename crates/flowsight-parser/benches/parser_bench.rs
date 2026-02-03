//! Performance benchmarks for flowsight-parser
//!
//! Run with: `cargo bench --package flowsight-parser`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use flowsight_parser::treesitter::TreeSitterParser;
use flowsight_parser::Parser;

/// Small C file (~100 lines) - typical header file
const SMALL_CODE: &str = r#"
#include <linux/module.h>
#include <linux/kernel.h>

static int counter = 0;

struct my_device {
    int id;
    char name[32];
    void *private_data;
};

static int my_init(void)
{
    counter = 0;
    printk(KERN_INFO "Module initialized\n");
    return 0;
}

static void my_exit(void)
{
    printk(KERN_INFO "Module exited\n");
}

static int my_open(struct inode *inode, struct file *file)
{
    counter++;
    return 0;
}

static int my_release(struct inode *inode, struct file *file)
{
    counter--;
    return 0;
}

static ssize_t my_read(struct file *file, char __user *buf, size_t count, loff_t *ppos)
{
    return 0;
}

static ssize_t my_write(struct file *file, const char __user *buf, size_t count, loff_t *ppos)
{
    return count;
}

static const struct file_operations my_fops = {
    .owner = THIS_MODULE,
    .open = my_open,
    .release = my_release,
    .read = my_read,
    .write = my_write,
};

module_init(my_init);
module_exit(my_exit);
MODULE_LICENSE("GPL");
MODULE_AUTHOR("Test");
MODULE_DESCRIPTION("Test module");
"#;

/// Medium C file (~500 lines) - typical driver file
fn generate_medium_code() -> String {
    let mut code = String::from(
        r#"
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/gpio.h>
#include <linux/interrupt.h>
#include <linux/platform_device.h>

"#,
    );

    // Generate 50 functions
    for i in 0..50 {
        code.push_str(&format!(
            r#"
static int function_{i}(struct device *dev, int param)
{{
    int result = 0;
    if (param > 0) {{
        result = param * 2;
    }} else {{
        result = -EINVAL;
    }}
    dev_info(dev, "function_{i} called with %d, returning %d\n", param, result);
    return result;
}}
"#
        ));
    }

    // Generate probe/remove
    code.push_str(
        r#"
static int my_probe(struct platform_device *pdev)
{
    struct device *dev = &pdev->dev;
    int ret;
"#,
    );

    for i in 0..20 {
        code.push_str(&format!("    ret = function_{i}(dev, {i});\n"));
        code.push_str("    if (ret < 0) return ret;\n");
    }

    code.push_str(
        r#"
    return 0;
}

static int my_remove(struct platform_device *pdev)
{
    return 0;
}

static struct platform_driver my_driver = {
    .probe = my_probe,
    .remove = my_remove,
    .driver = {
        .name = "my_driver",
    },
};

module_platform_driver(my_driver);
MODULE_LICENSE("GPL");
"#,
    );

    code
}

/// Large C file (~2000 lines) - complex driver
fn generate_large_code() -> String {
    let mut code = String::from(
        r#"
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/gpio.h>
#include <linux/interrupt.h>
#include <linux/platform_device.h>
#include <linux/workqueue.h>
#include <linux/timer.h>
#include <linux/spinlock.h>

"#,
    );

    // Generate 100 structures
    for i in 0..20 {
        code.push_str(&format!(
            r#"
struct data_struct_{i} {{
    int field_a;
    int field_b;
    char name[64];
    void *private;
    struct list_head list;
}};
"#
        ));
    }

    // Generate 200 functions
    for i in 0..200 {
        code.push_str(&format!(
            r#"
static int complex_function_{i}(struct device *dev, int param1, int param2)
{{
    int result = 0;
    unsigned long flags;
    
    spin_lock_irqsave(&dev->lock, flags);
    
    if (param1 > 0 && param2 > 0) {{
        result = param1 * param2;
    }} else if (param1 < 0) {{
        result = -EINVAL;
    }} else {{
        result = param2;
    }}
    
    spin_unlock_irqrestore(&dev->lock, flags);
    
    dev_dbg(dev, "complex_function_{i}(%d, %d) = %d\n", param1, param2, result);
    return result;
}}
"#
        ));
    }

    // Generate IRQ handlers
    for i in 0..10 {
        code.push_str(&format!(
            r#"
static irqreturn_t irq_handler_{i}(int irq, void *dev_id)
{{
    struct device *dev = dev_id;
    dev_info(dev, "IRQ {i} triggered\n");
    return IRQ_HANDLED;
}}
"#
        ));
    }

    // Generate workqueue handlers
    for i in 0..10 {
        code.push_str(&format!(
            r#"
static void work_handler_{i}(struct work_struct *work)
{{
    pr_info("Work {i} executed\n");
}}
"#
        ));
    }

    code.push_str(
        r#"
static int driver_probe(struct platform_device *pdev)
{
    struct device *dev = &pdev->dev;
    int ret;

    dev_info(dev, "Probing device\n");
"#,
    );

    for i in 0..50 {
        code.push_str(&format!(
            "    ret = complex_function_{i}(dev, {i}, {});\n",
            i * 2
        ));
        code.push_str("    if (ret < 0) goto err;\n");
    }

    code.push_str(
        r#"
    return 0;
err:
    dev_err(dev, "Probe failed: %d\n", ret);
    return ret;
}

static int driver_remove(struct platform_device *pdev)
{
    dev_info(&pdev->dev, "Removing device\n");
    return 0;
}

static struct platform_driver my_complex_driver = {
    .probe = driver_probe,
    .remove = driver_remove,
    .driver = {
        .name = "complex_driver",
    },
};

module_platform_driver(my_complex_driver);
MODULE_LICENSE("GPL");
MODULE_AUTHOR("FlowSight Team");
MODULE_DESCRIPTION("Complex driver for benchmarking");
"#,
    );

    code
}

fn bench_parse_small(c: &mut Criterion) {
    let parser = TreeSitterParser::new();

    c.bench_function("parse_small_100lines", |b| {
        b.iter(|| {
            let result = parser.parse(black_box(SMALL_CODE), "test.c");
            black_box(result)
        })
    });
}

fn bench_parse_medium(c: &mut Criterion) {
    let parser = TreeSitterParser::new();
    let code = generate_medium_code();

    let mut group = c.benchmark_group("parse_medium");
    group.throughput(Throughput::Bytes(code.len() as u64));

    group.bench_function("500lines", |b| {
        b.iter(|| {
            let result = parser.parse(black_box(&code), "test.c");
            black_box(result)
        })
    });

    group.finish();
}

fn bench_parse_large(c: &mut Criterion) {
    let parser = TreeSitterParser::new();
    let code = generate_large_code();

    let mut group = c.benchmark_group("parse_large");
    group.throughput(Throughput::Bytes(code.len() as u64));

    group.bench_function("2000lines", |b| {
        b.iter(|| {
            let result = parser.parse(black_box(&code), "test.c");
            black_box(result)
        })
    });

    group.finish();
}

fn bench_parse_scaling(c: &mut Criterion) {
    let parser = TreeSitterParser::new();

    let mut group = c.benchmark_group("parse_scaling");

    for size in [10, 50, 100, 200].iter() {
        let code = generate_code_with_functions(*size);
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("functions", size), &code, |b, code| {
            b.iter(|| {
                let result = parser.parse(black_box(code), "test.c");
                black_box(result)
            })
        });
    }

    group.finish();
}

fn generate_code_with_functions(count: usize) -> String {
    let mut code = String::from("#include <linux/kernel.h>\n\n");

    for i in 0..count {
        code.push_str(&format!(
            r#"
static int func_{i}(int x)
{{
    return x + {i};
}}
"#
        ));
    }

    code
}

criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_medium,
    bench_parse_large,
    bench_parse_scaling
);
criterion_main!(benches);
