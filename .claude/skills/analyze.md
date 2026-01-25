# /sc:analyze - 代码分析

> FlowSight 代码分析技能

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "分析", "analyze" | 代码分析 |
| "审查", "review" | 代码审查 |
| "安全", "security" | 安全分析 |

## 使用方式

```
/sc:analyze "代码质量分析"
/sc:analyze --security "安全漏洞检测"
/sc:analyze "性能瓶颈分析"
```

## 分析类型

### 代码质量分析

- 代码复杂度评估
- 重复代码检测
- 编码规范检查
- 潜在 bug 识别

### 安全分析

```bash
# 安全检查
cargo audit

# 依赖漏洞扫描
cargo deny check

# 代码安全审查
# - 内存安全
# - 输入验证
# - 错误处理
```

### 性能分析

```bash
# CPU 性能分析
cargo flamegraph

# 内存分析
heaptrack ./target/release/flowsight

# 编译时间分析
cargo +nightly build -Z timings
```

## 分析维度

| 维度 | 检查内容 | 工具 |
|------|----------|------|
| 正确性 | 逻辑错误 | 单元测试 |
| 安全性 | 漏洞检测 | cargo audit |
| 性能 | 热点代码 | flamegraph |
| 可维护性 | 代码复杂度 | cargo clippy |
| 风格 | 编码规范 | cargo fmt |

## 分析报告

```markdown
## 代码分析报告

### 概览
- 总代码行数: XXX
- 复杂度得分: XX/100
- 发现问题: X 个

### 问题列表
1. [严重] 问题描述
   - 位置: file:line
   - 建议修复方案

2. [警告] 问题描述
   - 位置: file:line
   - 建议修复方案

### 改进建议
- 建议1
- 建议2
```

## 与其他 Skills 配合

```
1. /sc:analyze "代码分析"
2. /sc:improve "改进代码"
3. /sc:test "运行测试"
```

---

**快捷命令**:

```
/sc:analyze "代码分析"    # 代码分析
/sc:analyze --security    # 安全分析
```

---

> FlowSight 专用 - 代码分析
