# /sc:load-rules - 加载开发规则

> SuperClaude Framework 开发规则加载技能

## 使用方式

```
/sc:load-rules    # 加载开发规则
```

## 功能说明

此命令用于加载项目的开发规则和质量标准。

### 加载内容

- Rust 编码规范
- TypeScript 编码规范
- 代码审查标准
- 提交规范
- 测试要求

## 开发规则

### 代码质量规则

1. **Rust 规则**
   - 所有公开 API 必须有文档注释
   - 关键逻辑需要单元测试覆盖
   - 提交前运行 `cargo clippy`
   - 遵循 Rust 所有权系统最佳实践

2. **前端规则**
   - 使用 TypeScript 类型定义
   - 组件 Props 有类型注解
   - 遵循项目现有样式规范

3. **提交规则**
   - 使用约定式提交格式
   - 提交前运行测试
   - 确保代码通过 lint 检查

## 与其他 Skills 配合

```
/sc:load-core     # 加载核心框架
/sc:load-flags    # 加载配置模式
/sc:load-rules    # 加载开发规则
```

---

**快捷命令**:

```
/sc:load-rules    # 加载开发规则
```

---

> SuperClaude Framework - 开发规则加载
