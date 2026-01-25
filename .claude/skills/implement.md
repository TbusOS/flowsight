# /sc:implement - 功能代码实现

> FlowSight 代码实现技能

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "实现", "implement" | 代码实现任务 |
| "添加", "add" | 添加新功能 |
| "创建", "create" | 创建新模块 |

## 使用方式

```
/sc:implement "实现 memory.yaml 解析器"
/sc:implement "添加 LLVM IR 语法高亮"
/sc:implement "创建节点详情面板组件"
```

## 实现规范

### Rust 后端

- 所有公开 API 必须有文档注释
- 关键逻辑需要单元测试覆盖
- 提交前运行 `cargo clippy`
- 遵循 cargo 项目结构

```rust
/// 功能描述
///
/// # Examples
///
/// ```
/// let result = function();
/// ```
pub fn public_api() -> Result<(), Error> {
    // 实现代码
}
```

### React 前端

- 使用 TypeScript 类型定义
- 组件 Props 有类型注解
- 遵循项目现有样式规范
- 提交前运行类型检查

## 工作流程

1. **理解需求** - 分析任务描述
2. **设计接口** - 定义数据结构
3. **实现代码** - 编写功能逻辑
4. **添加测试** - 覆盖关键场景
5. **运行检查** - clippy / 类型检查

## 与其他 Skills 配合

```
1. /sc:design "设计接口"    # 可选
2. /sc:implement "实现功能"
3. /sc:test "运行测试"
4. /sc:build "构建验证"
```

---

**快捷命令**:

```
/sc:impl "实现功能"         # 代码实现
/sc:implement "创建组件"    # 组件开发
```

---

> FlowSight 专用 - 功能代码实现
