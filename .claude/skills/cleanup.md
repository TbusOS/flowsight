# /sc:cleanup - 代码清理

> FlowSight 代码清理技能

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "清理", "cleanup" | 代码清理 |
| "移除", "remove" | 删除死代码 |
| "重构", "refactor" | 代码重构 |

## 使用方式

```
/sc:cleanup "移除死代码"
/sc:cleanup "清理未使用的依赖"
/sc:cleanup "重构模块结构"
```

## 清理内容

### 代码清理

- 删除未使用的函数/变量
- 移除重复代码
- 简化复杂逻辑
- 统一代码风格

### 依赖清理

- 移除未使用的 crate
- 更新过时依赖
- 解决依赖冲突

### 文件清理

- 删除临时文件
- 清理构建产物
- 整理目录结构

## 清理工具

### Rust 代码清理

```bash
# 检查未使用代码
cargo clippy -- -D warnings

# 自动格式化
cargo fmt

# 移除未导入
cargo +nightly rustfmt --check
```

### 依赖检查

```bash
# 查找未使用依赖
cargo tree -i unused_crate

# 检查依赖版本
cargo outdated
```

## 注意事项

1. **备份代码** - 清理前确保版本控制
2. **小步清理** - 每次清理少量代码
3. **验证测试** - 清理后运行测试

## 与其他 Skills 配合

```
1. /sc:cleanup "清理代码"
2. /sc:test "运行测试"
3. /sc:build "构建验证"
```

---

**快捷命令**:

```
/sc:cleanup "清理代码"    # 代码清理
```

---

> FlowSight 专用 - 代码清理
