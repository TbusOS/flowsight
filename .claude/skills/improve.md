# /sc:improve - 代码改进

> FlowSight 代码改进技能

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "改进", "improve" | 代码改进 |
| "优化", "optimize" | 性能优化 |
| "增强", "enhance" | 功能增强 |

## 使用方式

```
/sc:improve "性能优化"
/sc:improve "代码质量提升"
/sc:improve "错误处理增强"
```

## 改进类型

### 性能优化

- 减少内存分配
- 优化算法复杂度
- 使用更高效的数据结构
- 并行化处理

### 代码质量

- 增强类型安全
- 改进错误处理
- 添加文档注释
- 提高测试覆盖率

### 可维护性

- 简化复杂代码
- 提取公共逻辑
- 改善模块结构
- 统一代码风格

## 优化工具

### 性能分析

```bash
# CPU 性能分析
cargo flamegraph

# 内存分析
cargo valgrind

# 编译时间
cargo +nightly build -Z timings
```

### 代码质量检查

```bash
# Clippy 建议
cargo clippy

# 代码复杂度
cargo machete
```

## 改进策略

1. **基准测试** - 测量当前性能
2. **定位瓶颈** - 识别热点代码
3. **实施优化** - 改进实现
4. **验证效果** - 确认性能提升

## 与其他 Skills 配合

```
1. /sc:analyze "分析代码质量"
2. /sc:improve "性能优化"
3. /sc:test "性能测试"
```

---

**快捷命令**:

```
/sc:improve "优化性能"    # 性能优化
/sc:improve "提升质量"    # 质量改进
```

---

> FlowSight 专用 - 代码改进
