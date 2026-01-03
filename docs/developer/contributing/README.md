# 贡献指南

本目录包含 FlowSight 的贡献相关文档。

> 🚧 文档编写中

## 📋 计划文档

| 文档 | 描述 |
|------|------|
| CONTRIBUTING.md | 完整贡献流程 |
| CODE-STYLE.md | Rust/TypeScript 代码风格 |
| BUILD.md | 构建与测试 |
| RELEASE.md | 发布流程 |

## 🚀 快速贡献

### 1. Fork 仓库

```bash
git clone https://github.com/YOUR_NAME/flowsight.git
cd flowsight
git remote add upstream https://github.com/TbusOS/flowsight.git
```

### 2. 创建分支

```bash
git checkout -b feature/your-feature-name
```

### 3. 开发与测试

```bash
# 运行测试
cargo test
pnpm test

# 代码检查
cargo clippy
pnpm lint
```

### 4. 提交 PR

- 清晰的 commit message
- 更新相关文档
- 添加必要的测试

## 📝 Commit 规范

```
type(scope): description

- feat: 新功能
- fix: Bug 修复
- docs: 文档更新
- refactor: 重构
- test: 测试相关
- chore: 构建/工具
```

示例:
```
feat(parser): add Kotlin coroutine pattern support
fix(analyzer): resolve false positive in pointer analysis
docs(knowledge): add USB driver framework documentation
```

## 🎯 贡献方向

### 高优先级

- [ ] Linux 内核知识库扩展
- [ ] 指针分析算法优化
- [ ] UI/UX 改进

### 中优先级

- [ ] Android 框架知识
- [ ] 性能优化
- [ ] 测试覆盖

### 欢迎贡献

- [ ] 新语言支持
- [ ] 文档翻译
- [ ] Bug 报告与修复

