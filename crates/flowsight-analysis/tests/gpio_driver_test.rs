//! 真实内核文件解析测试
//! 
//! 测试解析器和分析器对真实 GPIO 驱动文件的处理

use flowsight_analysis::Analyzer;
use flowsight_analysis::flow_builder::{FlowBuilder, FunctionInfo, BuildOptions};
use flowsight_parser::get_parser;
use std::path::PathBuf;

const GPIO_AMD8111_PATH: &str = "/Users/sky/linux-kernel/linux/drivers/gpio/gpio-amd8111.c";

/// 测试解析真实的 GPIO 驱动文件
#[test]
fn test_parse_real_gpio_driver() {
    let path = PathBuf::from(GPIO_AMD8111_PATH);
    
    if !path.exists() {
        println!("跳过测试：文件不存在 {}", GPIO_AMD8111_PATH);
        return;
    }
    
    let parser = get_parser();
    let parse_result = parser.parse_file(&path);
    
    match parse_result {
        Ok(result) => {
            println!("\n=== 解析结果 ===");
            println!("函数数量: {}", result.functions.len());
            println!("结构体数量: {}", result.structs.len());
            
            // 打印所有函数
            println!("\n=== 函数列表 ===");
            for (name, func) in &result.functions {
                println!("{}():", name);
                println!("  行号: {:?}", func.location.as_ref().map(|l| l.line));
                println!("  调用: {:?}", func.calls);
                println!("  是回调: {}", func.is_callback);
            }
            
            // 验证基本解析
            assert!(result.functions.len() > 0, "应该解析出函数");
            
            // 检查关键函数
            let expected_funcs = [
                "amd_gpio_request",
                "amd_gpio_free", 
                "amd_gpio_set",
                "amd_gpio_get",
            ];
            
            for func_name in &expected_funcs {
                if !result.functions.contains_key(*func_name) {
                    println!("警告: 未找到预期函数 {}", func_name);
                }
            }
            
            // 检查函数调用是否被提取
            let mut total_calls = 0;
            for (_, func) in &result.functions {
                total_calls += func.calls.len();
            }
            println!("\n总调用数: {}", total_calls);
            
            // 如果没有调用被提取，这是个问题
            if total_calls == 0 {
                println!("警告: 没有提取到任何函数调用！");
            }
        }
        Err(e) => {
            println!("解析失败: {}", e);
            panic!("解析失败: {}", e);
        }
    }
}

/// 测试执行流构建
#[test]
fn test_build_execution_flow_for_gpio() {
    let path = PathBuf::from(GPIO_AMD8111_PATH);
    
    if !path.exists() {
        println!("跳过测试：文件不存在 {}", GPIO_AMD8111_PATH);
        return;
    }
    
    let parser = get_parser();
    let mut parse_result = parser.parse_file(&path).expect("解析应该成功");
    
    let source = std::fs::read_to_string(&path).expect("读取文件应该成功");
    
    // 运行分析
    let mut analyzer = Analyzer::new();
    let analysis = analyzer.analyze(&source, &mut parse_result).expect("分析应该成功");
    
    println!("\n=== 分析结果 ===");
    println!("入口点: {:?}", analysis.entry_points);
    println!("异步绑定数: {}", analysis.async_bindings.len());
    println!("调用边数: {}", analysis.call_edges.len());
    println!("执行流树数: {}", analysis.flow_trees.len());
    
    // 构建 FlowBuilder
    let mut builder = FlowBuilder::new();
    builder.extract_async_bindings(&source);
    
    // 注册所有解析的函数
    for (name, func) in &parse_result.functions {
        builder.register_function(FunctionInfo {
            name: name.clone(),
            location: func.location.clone(),
            calls: func.calls.clone(),
            is_kernel: func.attributes.contains(&"__init".to_string()) 
                || func.attributes.contains(&"__exit".to_string())
                || name.starts_with("__"),
        });
    }
    
    // 尝试为第一个入口点构建执行流
    if let Some(entry) = analysis.entry_points.first() {
        println!("\n=== 为 {} 构建执行流 ===", entry);
        
        let flow = builder.build(entry, &BuildOptions::default());
        
        println!("入口函数: {}", flow.entry_function);
        println!("根节点: {}", flow.root.name);
        println!("根节点子节点数: {}", flow.root.children.len());
        println!("总节点数: {}", flow.analysis_info.total_nodes);
        println!("直接调用数: {}", flow.analysis_info.direct_calls);
        
        // 打印根节点的子节点
        println!("\n子节点:");
        for child in &flow.root.children {
            println!("  - {} ({:?})", child.name, child.node_type);
        }
        
        // 验证执行流不为空
        if flow.analysis_info.total_nodes <= 1 {
            println!("\n警告: 执行流只有 {} 个节点！", flow.analysis_info.total_nodes);
            println!("这意味着函数调用没有被正确提取。");
            
            // 检查原始函数的 calls 字段
            if let Some(func) = parse_result.functions.get(entry) {
                println!("\n{} 的原始调用列表: {:?}", entry, func.calls);
            }
        }
    } else {
        println!("警告: 没有找到入口点！");
    }
}

/// 测试简单代码片段的解析
#[test]
fn test_parse_simple_kernel_code() {
    use flowsight_parser::treesitter::TreeSitterParser;
    
    // 模拟 GPIO 驱动代码
    let code = r#"
#include <linux/gpio.h>

static int amd_gpio_request(struct gpio_chip *chip, unsigned offset)
{
    struct amd_gpio *agp = gpiochip_get_data(chip);
    
    agp->orig[offset] = ioread8(agp->pm + AMD_REG_GPIO(offset)) &
        (AMD_GPIO_DEBOUNCE | AMD_GPIO_MODE_MASK | AMD_GPIO_X_MASK);
    
    dev_dbg(&agp->pdev->dev, "Requested gpio %d\n", offset);
    
    return 0;
}

static void amd_gpio_set(struct gpio_chip *chip, unsigned offset, int value)
{
    struct amd_gpio *agp = gpiochip_get_data(chip);
    u8 temp;
    unsigned long flags;
    
    spin_lock_irqsave(&agp->lock, flags);
    temp = ioread8(agp->pm + AMD_REG_GPIO(offset));
    temp = (temp & ~AMD_GPIO_X_OUT_LOW) | (value ? AMD_GPIO_X_OUT_HI : AMD_GPIO_X_OUT_LOW);
    iowrite8(temp, agp->pm + AMD_REG_GPIO(offset));
    spin_unlock_irqrestore(&agp->lock, flags);
}

static int amd8111_gpio_probe(struct pci_dev *pdev)
{
    struct amd_gpio *agp;
    int err;
    
    agp = devm_kzalloc(&pdev->dev, sizeof(*agp), GFP_KERNEL);
    if (!agp)
        return -ENOMEM;
    
    err = gpiochip_add_data(&agp->chip, agp);
    if (err) {
        dev_err(&pdev->dev, "Failed to add gpio chip\n");
        return err;
    }
    
    pci_set_drvdata(pdev, agp);
    return 0;
}
"#;
    
    let mut parser = TreeSitterParser::new();
    let result = parser.parse_source(code, "test_gpio.c").expect("解析应该成功");
    
    println!("\n=== 模拟代码解析结果 ===");
    println!("函数数量: {}", result.functions.len());
    
    for (name, func) in &result.functions {
        println!("{}():", name);
        println!("  调用: {:?}", func.calls);
    }
    
    // 验证 amd8111_gpio_probe 的调用被正确提取
    let probe = result.functions.get("amd8111_gpio_probe").expect("应该找到 probe 函数");
    
    // 应该包含这些调用
    assert!(probe.calls.contains(&"devm_kzalloc".to_string()), 
            "应该提取 devm_kzalloc 调用，实际: {:?}", probe.calls);
    assert!(probe.calls.contains(&"gpiochip_add_data".to_string()),
            "应该提取 gpiochip_add_data 调用，实际: {:?}", probe.calls);
    assert!(probe.calls.contains(&"pci_set_drvdata".to_string()),
            "应该提取 pci_set_drvdata 调用，实际: {:?}", probe.calls);
    
    // 验证 amd_gpio_set 的调用
    let gpio_set = result.functions.get("amd_gpio_set").expect("应该找到 gpio_set 函数");
    assert!(gpio_set.calls.contains(&"spin_lock_irqsave".to_string()),
            "应该提取 spin_lock_irqsave 调用，实际: {:?}", gpio_set.calls);
    
    println!("\n所有调用都被正确提取！");
}
