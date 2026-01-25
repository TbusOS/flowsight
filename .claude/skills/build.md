# /sc:build - 项目构建

> FlowSight 项目构建技能

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "构建", "build" | 项目构建任务 |
| "编译", "compile" | 代码编译 |
| "打包", "package" | 打包发布 |

## 使用方式

```
/sc:build              # 默认构建
/sc:build --release    # 发布构建
/sc:build "完整构建"    # 前后端构建
```

## 构建命令

### Rust 后端

```bash
# 开发构建
cargo build --workspace

# 发布构建
cargo build --release --workspace

# 只构建特定 crate
cargo build -p flowsight-analysis
```

### 前端应用

```bash
# 开发构建
cd app && pnpm tauri dev

# 生产构建
cd app && pnpm build

# 只构建前端
cd app && pnpm tauri build
```

### 完整构建

```bash
# Rust + 前端完整构建
cargo build --release --workspace && cd app && pnpm build
```

## 构建检查

在构建前自动执行：

1. **代码格式检查** - `cargo fmt`
2. **静态分析** - `cargo clippy`
3. **类型检查** - TypeScript 编译

## 故障排查

### 常见问题

| 问题 | 解决方案 |
|------|----------|
| 依赖冲突 | `cargo update` |
| 编译错误 | 查看具体错误信息 |
| 前端构建失败 | `cd app && pnpm install` |

### 使用 /sc:troubleshoot 配合

```
/sc:build "构建失败"
/sc:troubleshoot "诊断构建问题"
```

## 与其他 Skills 配合

```
1. /sc:implement "实现功能"
2. /sc:test "运行测试"
3. /sc:build "构建验证"
4. /sc:git "提交代码"
```

---

**快捷命令**:

```
/sc:build           # 构建项目
/sc:build --release # 发布构建
```

---

> FlowSight 专用 - 项目构建
