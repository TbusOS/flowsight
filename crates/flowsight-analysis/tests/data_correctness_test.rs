//! 数据正确性测试
//!
//! 🔴 关键测试：确保解析结果与源代码一致
//!
//! 这些测试是防止 UI 显示假数据的关键防线。

use flowsight_analysis::Analyzer;
use flowsight_parser::treesitter::TreeSitterParser;
use std::collections::HashSet;

/// 简单的 GPIO 驱动代码（真实场景）
const GPIO_DRIVER: &str = r#"
#include <linux/module.h>
#include <linux/gpio/driver.h>
#include <linux/platform_device.h>
#include <linux/interrupt.h>

struct my_gpio_chip {
    struct gpio_chip gc;
    void __iomem *base;
    spinlock_t lock;
    struct work_struct irq_work;
    int irq;
};

static void my_gpio_irq_work(struct work_struct *work)
{
    struct my_gpio_chip *chip = container_of(work, struct my_gpio_chip, irq_work);
    // Process IRQ in process context
    spin_lock(&chip->lock);
    // ... handle
    spin_unlock(&chip->lock);
}

static irqreturn_t my_gpio_irq_handler(int irq, void *dev_id)
{
    struct my_gpio_chip *chip = dev_id;
    // Quick handler in IRQ context
    schedule_work(&chip->irq_work);
    return IRQ_HANDLED;
}

static int my_gpio_get(struct gpio_chip *gc, unsigned int offset)
{
    struct my_gpio_chip *chip = gpiochip_get_data(gc);
    return readl(chip->base + offset * 4) & 1;
}

static void my_gpio_set(struct gpio_chip *gc, unsigned int offset, int value)
{
    struct my_gpio_chip *chip = gpiochip_get_data(gc);
    unsigned long flags;
    u32 reg;

    spin_lock_irqsave(&chip->lock, flags);
    reg = readl(chip->base + offset * 4);
    if (value)
        reg |= 1;
    else
        reg &= ~1;
    writel(reg, chip->base + offset * 4);
    spin_unlock_irqrestore(&chip->lock, flags);
}

static int my_gpio_direction_input(struct gpio_chip *gc, unsigned int offset)
{
    struct my_gpio_chip *chip = gpiochip_get_data(gc);
    // Set direction to input
    writel(0, chip->base + 0x100 + offset * 4);
    return 0;
}

static int my_gpio_direction_output(struct gpio_chip *gc, unsigned int offset, int value)
{
    struct my_gpio_chip *chip = gpiochip_get_data(gc);
    // Set direction to output
    writel(1, chip->base + 0x100 + offset * 4);
    my_gpio_set(gc, offset, value);
    return 0;
}

static int my_gpio_probe(struct platform_device *pdev)
{
    struct my_gpio_chip *chip;
    struct resource *res;
    int ret;

    chip = devm_kzalloc(&pdev->dev, sizeof(*chip), GFP_KERNEL);
    if (!chip)
        return -ENOMEM;

    res = platform_get_resource(pdev, IORESOURCE_MEM, 0);
    chip->base = devm_ioremap_resource(&pdev->dev, res);
    if (IS_ERR(chip->base))
        return PTR_ERR(chip->base);

    spin_lock_init(&chip->lock);
    INIT_WORK(&chip->irq_work, my_gpio_irq_work);

    chip->gc.label = "my-gpio";
    chip->gc.owner = THIS_MODULE;
    chip->gc.parent = &pdev->dev;
    chip->gc.base = -1;
    chip->gc.ngpio = 32;
    chip->gc.get = my_gpio_get;
    chip->gc.set = my_gpio_set;
    chip->gc.direction_input = my_gpio_direction_input;
    chip->gc.direction_output = my_gpio_direction_output;

    chip->irq = platform_get_irq(pdev, 0);
    if (chip->irq > 0) {
        ret = devm_request_irq(&pdev->dev, chip->irq, my_gpio_irq_handler,
                               IRQF_SHARED, "my-gpio", chip);
        if (ret)
            return ret;
    }

    ret = devm_gpiochip_add_data(&pdev->dev, &chip->gc, chip);
    if (ret)
        return ret;

    platform_set_drvdata(pdev, chip);
    dev_info(&pdev->dev, "GPIO controller registered\n");
    return 0;
}

static int my_gpio_remove(struct platform_device *pdev)
{
    struct my_gpio_chip *chip = platform_get_drvdata(pdev);
    cancel_work_sync(&chip->irq_work);
    return 0;
}

static struct platform_driver my_gpio_driver = {
    .driver = {
        .name = "my-gpio",
    },
    .probe = my_gpio_probe,
    .remove = my_gpio_remove,
};

module_platform_driver(my_gpio_driver);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("My GPIO Driver");
"#;

/// 预期在代码中存在的函数列表
const EXPECTED_FUNCTIONS: &[&str] = &[
    "my_gpio_irq_work",
    "my_gpio_irq_handler",
    "my_gpio_get",
    "my_gpio_set",
    "my_gpio_direction_input",
    "my_gpio_direction_output",
    "my_gpio_probe",
    "my_gpio_remove",
];

/// 预期的调用关系
const EXPECTED_CALLS: &[(&str, &str)] = &[
    ("my_gpio_probe", "devm_kzalloc"),
    ("my_gpio_probe", "platform_get_resource"),
    ("my_gpio_probe", "spin_lock_init"),
    ("my_gpio_probe", "INIT_WORK"),
    ("my_gpio_probe", "platform_get_irq"),
    ("my_gpio_probe", "devm_request_irq"),
    ("my_gpio_probe", "devm_gpiochip_add_data"),
    ("my_gpio_set", "spin_lock_irqsave"),
    ("my_gpio_set", "readl"),
    ("my_gpio_set", "writel"),
    ("my_gpio_set", "spin_unlock_irqrestore"),
    ("my_gpio_direction_output", "my_gpio_set"),
    ("my_gpio_irq_handler", "schedule_work"),
    ("my_gpio_remove", "cancel_work_sync"),
];

/// 🔴 关键测试：解析器必须找到所有预期的函数
#[test]
fn test_parser_finds_all_expected_functions() {
    let mut parser = TreeSitterParser::new();
    let parse_result = parser.parse_source(GPIO_DRIVER, "gpio_driver.c").unwrap();

    let found_functions: HashSet<&str> =
        parse_result.functions.keys().map(|s| s.as_str()).collect();

    let missing: Vec<&str> = EXPECTED_FUNCTIONS
        .iter()
        .filter(|f| !found_functions.contains(*f))
        .copied()
        .collect();

    assert!(
        missing.is_empty(),
        "解析器未找到预期的函数: {:?}\n实际找到: {:?}",
        missing,
        found_functions
    );
}

/// 🔴 关键测试：解析的函数数量不能为 0
#[test]
fn test_parser_returns_nonzero_functions() {
    let mut parser = TreeSitterParser::new();
    let parse_result = parser.parse_source(GPIO_DRIVER, "gpio_driver.c").unwrap();

    assert!(
        !parse_result.functions.is_empty(),
        "🔴 解析器返回 0 个函数！这会导致 UI 显示问题"
    );

    // 应该至少有 8 个函数
    assert!(
        parse_result.functions.len() >= 8,
        "期望至少 8 个函数，实际: {}",
        parse_result.functions.len()
    );
}

/// 🔴 关键测试：解析的结构体数量不能为 0
#[test]
fn test_parser_returns_nonzero_structs() {
    let mut parser = TreeSitterParser::new();
    let parse_result = parser.parse_source(GPIO_DRIVER, "gpio_driver.c").unwrap();

    assert!(
        !parse_result.structs.is_empty(),
        "🔴 解析器返回 0 个结构体！这会导致 UI 显示问题"
    );

    // 应该至少找到 my_gpio_chip
    assert!(
        parse_result.structs.contains_key("my_gpio_chip"),
        "应该找到 my_gpio_chip 结构体"
    );
}

/// 🔴 关键测试：调用关系必须正确
#[test]
fn test_call_relationships_are_correct() {
    let mut parser = TreeSitterParser::new();
    let parse_result = parser.parse_source(GPIO_DRIVER, "gpio_driver.c").unwrap();

    for (caller, callee) in EXPECTED_CALLS {
        let func = parse_result.functions.get(*caller);
        assert!(func.is_some(), "调用者函数 {} 应该存在", caller);

        let func = func.unwrap();
        assert!(
            func.calls.contains(&callee.to_string()),
            "{} 应该调用 {}，但实际调用列表: {:?}",
            caller,
            callee,
            func.calls
        );
    }
}

/// 🔴 关键测试：异步回调必须被正确识别
#[test]
fn test_async_callbacks_are_detected() {
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(GPIO_DRIVER, "gpio_driver.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(GPIO_DRIVER, &mut parse_result).unwrap();

    // 应该检测到 INIT_WORK 绑定
    let has_work_binding = result
        .async_bindings
        .iter()
        .any(|b| b.handler == "my_gpio_irq_work");

    assert!(
        has_work_binding,
        "应该检测到 my_gpio_irq_work 的 INIT_WORK 绑定"
    );
}

/// 🔴 关键测试：入口点必须被正确识别
#[test]
fn test_entry_points_are_detected() {
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(GPIO_DRIVER, "gpio_driver.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(GPIO_DRIVER, &mut parse_result).unwrap();

    // probe 应该是入口点
    let has_probe = result.entry_points.iter().any(|e| e.contains("probe"));

    assert!(has_probe, "my_gpio_probe 应该被识别为入口点");
}

/// 🔴 关键测试：回调函数应该被标记
#[test]
fn test_callbacks_are_marked() {
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(GPIO_DRIVER, "gpio_driver.c").unwrap();

    let mut analyzer = Analyzer::new();
    let _ = analyzer.analyze(GPIO_DRIVER, &mut parse_result).unwrap();

    // IRQ handler 应该被标记为回调
    let irq_handler = parse_result.functions.get("my_gpio_irq_handler");
    assert!(
        irq_handler.map(|f| f.is_callback).unwrap_or(false),
        "my_gpio_irq_handler 应该被标记为回调函数"
    );

    // Work handler 应该被标记为回调
    let work_handler = parse_result.functions.get("my_gpio_irq_work");
    assert!(
        work_handler.map(|f| f.is_callback).unwrap_or(false),
        "my_gpio_irq_work 应该被标记为回调函数"
    );
}

/// 🔴 关键测试：执行流树不能为空
#[test]
fn test_flow_trees_are_not_empty() {
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(GPIO_DRIVER, "gpio_driver.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(GPIO_DRIVER, &mut parse_result).unwrap();

    assert!(
        !result.flow_trees.is_empty(),
        "🔴 执行流树为空！这会导致 UI 只显示一个节点"
    );

    // 至少一个流树应该有子节点
    let has_children = result
        .flow_trees
        .iter()
        .any(|tree| !tree.children.is_empty());

    assert!(
        has_children,
        "🔴 所有执行流树都没有子节点！调用关系未正确展开"
    );
}

/// 🔴 关键测试：probe 函数的执行流应该包含多个节点
#[test]
fn test_probe_flow_has_multiple_nodes() {
    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(GPIO_DRIVER, "gpio_driver.c").unwrap();

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(GPIO_DRIVER, &mut parse_result).unwrap();

    // 找到 probe 的流树
    let probe_tree = result.flow_trees.iter().find(|t| t.name.contains("probe"));

    assert!(probe_tree.is_some(), "应该有 probe 函数的执行流树");

    let tree = probe_tree.unwrap();

    // 计算总节点数（递归）
    fn count_nodes(node: &flowsight_core::FlowNode) -> usize {
        1 + node.children.iter().map(|c| count_nodes(c)).sum::<usize>()
    }

    let total_nodes = count_nodes(tree);

    assert!(
        total_nodes > 1,
        "🔴 probe 执行流只有 {} 个节点，应该有多个！",
        total_nodes
    );
}

/// 空代码测试：确保解析器能正确处理空/无效输入
#[test]
fn test_parser_handles_empty_input() {
    let mut parser = TreeSitterParser::new();

    // 空字符串
    let result = parser.parse_source("", "empty.c");
    assert!(result.is_ok(), "空文件不应该导致错误");
    let parse_result = result.unwrap();
    assert!(
        parse_result.functions.is_empty(),
        "空文件应该返回空函数列表"
    );

    // 只有注释
    let result = parser.parse_source("// just a comment", "comment.c");
    assert!(result.is_ok());

    // 无效语法（但不应该 panic）
    let result = parser.parse_source("this is not valid C code {{{", "invalid.c");
    // 解析器应该优雅处理，不崩溃
    assert!(result.is_ok() || result.is_err()); // 允许返回错误，但不能 panic
}

/// 大文件测试：确保解析器能处理大文件
#[test]
fn test_parser_handles_large_input() {
    let mut parser = TreeSitterParser::new();

    // 生成包含 100 个函数的大文件
    let mut large_source = String::new();
    large_source.push_str("#include <linux/kernel.h>\n\n");
    for i in 0..100 {
        large_source.push_str(&format!(
            "int function_{}(int arg) {{ return arg + {}; }}\n\n",
            i, i
        ));
    }

    let result = parser.parse_source(&large_source, "large.c");
    assert!(result.is_ok(), "大文件解析不应该失败");

    let parse_result = result.unwrap();
    assert!(
        parse_result.functions.len() >= 100,
        "应该解析出至少 100 个函数，实际: {}",
        parse_result.functions.len()
    );
}
