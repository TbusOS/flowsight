# /sc:test - 测试执行

> FlowSight 测试执行技能

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "测试", "test" | 测试执行任务 |
| "验证", "verify" | 功能验证 |
| "覆盖率", "coverage" | 代码覆盖率 |

## 使用方式

```
/sc:test              # 运行所有测试
/sc:test "测试分析模块" # 测试特定模块
/sc:test -p flowsight-analysis  # 测试特定 crate
```

## 测试命令

### 所有测试

```bash
cargo test --workspace
```

### 特定 crate 测试

```bash
# 测试分析模块
cargo test -p flowsight-analysis

# 测试核心模块
cargo test -p flowsight-core

# 测试解析器
cargo test -p flowsight-parser
```

### 前端测试

```bash
cd app && pnpm test
```

## 测试覆盖率

```bash
# 生成覆盖率报告
cargo tarpaulin --workspace
```

## 测试类型

| 类型 | 说明 | 命令 |
|------|------|------|
| 单元测试 | 测试单个函数 | `cargo test` |
| 集成测试 | 测试模块间交互 | `cargo test --test` |
| 文档测试 | 验证文档示例 | `cargo test --doc` |

## 测试规范

### Rust 测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        let result = function();
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_case() {
        // 边界条件测试
    }
}
```

## 测试失败处理

1. 分析失败原因
2. 修复��现代码
3. 重新运行测试
4. 验证修复效果

## 与其他 Skills 配合

```
1. /sc:implement "实现功能"
2. /sc:test "运行测试"
3. /sc:build "构建验证"
```

---

**快捷命令**:

```
/sc:test              # 运行所有测试
/sc:test -p <crate>   # 测试特定 crate
```

---

> FlowSight 专用 - 测试执行
