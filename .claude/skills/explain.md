# /sc:explain - 代码解释

> FlowSight 代码解释技能

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "解释", "explain" | 代码解释 |
| "说明", "describe" | 功能说明 |
| "理解", "understand" | 理解代码 |

## 使用方式

```
/sc:explain "解释这个函数"
/sc:explain "说明模块功能"
/sc:explain "解释异步代码"
```

## 解释内容

### 函数解释

```rust
/// 计算斐波那契数列
///
/// 使用动态规划算法，避免递归重复计算
/// 时间复杂度 O(n)，空间复杂度 O(1)
///
/// # Arguments
///
/// * `n` - 要求第 n 项
///
/// # Examples
///
/// ```
/// let result = fibonacci(10);
/// assert_eq!(result, 55);
/// ```
pub fn fibonacci(n: u32) -> u32 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    a
}
```

### 模块解释

```markdown
## flowsight-analysis 模块

### 功能概述
提供代码分析能力，包括：
- LLVM IR 解析
- 函数调用分析
- 符号执行支持

### 核心类型
- `Analyzer`: 主分析器
- `IrParser`: IR 解析器
- `SymbolicExecutor`: 符号执行器

### 使用示例
```rust
let analyzer = Analyzer::new();
let result = analyzer.analyze_function(func_id);
```
```

### 概念解释

- 异步编程概念
- 符号执行原理
- 流程图数据结构

## 解释深度

| 场景 | 解释深度 |
|------|----------|
| 快速理解 | 高层概述 |
| 学习研究 | 详细分析 |
| 调试问题 | 执行流程 |

## 输出格式

### 代码解释

```markdown
## 函数名

### 功能
简要描述函数作用

### 实现原理
- 关键算法
- 数据结构
- 边界处理

### 调用关系
- 被谁调用
- 调用哪些函数

### 使用注意事项
- 参数要求
- 异常情况
```

## 与其他 Skills 配合

```
1. /sc:explain "解释代码"
2. /sc:design "理解架构"
3. /sc:implement "实现功能"
```

---

**快捷命令**:

```
/sc:explain "解释函数"    # 代码解释
```

---

> FlowSight 专用 - 代码解释
