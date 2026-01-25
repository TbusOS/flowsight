# /sc:document - 文档生成

> FlowSight 文档生成技能

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "文档", "document" | 文档编写 |
| "注释", "comment" | 代码注释 |
| "README" | 项目文档 |

## 使用方式

```
/sc:document "API 文档"
/sc:document "模块说明"
/sc:document "更新 README"
```

## 文档类型

### 代码文档

```rust
/// 模块功能描述
///
/// # Examples
///
/// ```
/// use module_name;
///
/// let result = module_name::function();
/// ```
///
/// # Panics
///
/// 函数在以下情况会 panic:
///
/// - 参数无效时
///
/// # Errors
///
/// 返回 [`Error`] 类型错误：
///
/// - [`ErrorKind::InvalidInput`]
pub fn public_function() -> Result<()> {
    // 实现
}
```

### API 文档

- 函数接口说明
- 参数含义解释
- 返回值说明
- 错误类型定义

### 模块文档

- 模块目的说明
- 主要类型/函数列表
- 使用示例
- 相关模块链接

## 文档工具

### Rust 文档

```bash
# 生成 HTML 文档
cargo doc --no-deps

# 打开本地文档
cargo doc --open

# 检查文档测试
cargo test --doc
```

### 项目文档

- 更新 README.md
- 编写 API 文档
- 创建使用指南

## 文档规范

| 类型 | 要求 |
|------|------|
| 公开 API | 必须有文档注释 |
| 复杂函数 | 需要示例代码 |
| 错误类型 | 说明错误场景 |
| 模块 | 说明模块职责 |

## 与其他 Skills 配合

```
1. /sc:implement "实现功能"
2. /sc:document "添加文档"
3. /sc:test "验证文档测试"
```

---

**快捷命令**:

```
/sc:document "API 文档"    # 文档生成
```

---

> FlowSight 专用 - 文档生成
