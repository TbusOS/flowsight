# /sc:troubleshoot - 问题诊断

> FlowSight 问题诊断技能

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "调试", "debug" | 问题调试 |
| "问题", "issue" | 问题诊断 |
| "错误", "error" | 错误排查 |
| "失败", "fail" | 失败分析 |

## 使用方式

```
/sc:troubleshoot "编译错误"
/sc:troubleshoot "测试失败"
/sc:troubleshoot "运行时崩溃"
```

## 诊断流程

### 1. 问题收集

- 收集错误信息
- 记录复现步骤
- 查看相关日志

### 2. 原因分析

- 分析错误堆栈
- 检查依赖版本
- 审查最近变更

### 3. 解决方案

- 制定修复计划
- 执行修复
- 验证效果

## 常见问题类型

### 编译错误

```
症状: cargo build 失败
解决:
1. 检查依赖版本
2. 运行 cargo update
3. 查看具体错误信息
```

### 测试失败

```
症状: cargo test 失败
解决:
1. 分析失败原因
2. 修复实现或测试
3. 重新运行测试
```

### 运行时错误

```
症状: 程序崩溃/异常
解决:
1. 查看日志
2. 使用调试器
3. 检查边界条件
```

### 性能问题

```
症状: 运行缓慢
解决:
1. 使用 profiling 工具
2. 分析瓶颈
3. 优化热点代码
```

## 诊断工具

### Rust 诊断

```bash
# 详细错误信息
cargo build -vv

# 内存检测
valgrind ./target/release/flowsight

# 线程分析
cargo flamegraph
```

### 前端诊断

```bash
# 开发模式
cd app && pnpm tauri dev --debug

# 检查控制台错误
# 查看浏览器开发者工具
```

## 与其他 Skills 配合

```
1. /sc:troubleshoot "诊断问题"
2. /sc:analyze "分析根因"
3. /sc:implement "修复问题"
4. /sc:test "验证修复"
```

---

**快捷命令**:

```
/sc:debug "诊断问题"     # 问题诊断
/sc:troubleshoot "错误"  # 错误排查
```

---

> FlowSight 专用 - 问题诊断
