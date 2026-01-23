# FlowSight - Claude Code 配置

> **自动加载**: 启动时自动加载完整 SuperClaude Framework skills
>
> 运行 `/sc:help` 验证所有 skills 已加载

## 项目概述
跨平台代码执行流可视化 IDE，帮助理解 Linux 内核等大型代码库的执行流程。

## 技术栈
- **后端**: Rust (Tokio, Tree-sitter, SQLite, Sled)
- **前端**: React + TypeScript + Tauri
- **可视化**: @xyflow/react (流程图), Monaco Editor

## 常用命令

```bash
# 后端构建与测试
cargo build --workspace
cargo test --workspace
cargo test --package flowsight-analysis  # 只测试分析模块

# 前端开发
cd app && pnpm install
cd app && pnpm tauri dev

# 完整构建
cd app && pnpm build

# 代码检查
cargo clippy
cargo fmt
```

## 项目结构

```
flowsight/
├── crates/                    # Rust 核心模块
│   ├── flowsight-core/       # 核心类型定义
│   ├── flowsight-parser/     # 代码解析器
│   ├── flowsight-analysis/   # 代码分析引擎 ← 当前开发重点
│   ├── flowsight-index/      # 符号索引
│   ├── flowsight-knowledge/  # 知识库
│   ├── flowsight-query/      # 查询引擎
│   └── flowsight-cli/        # CLI 工具
├── app/                       # 前端应用
│   ├── src/
│   │   ├── components/       # React 组件
│   │   ├── store/           # Zustand 状态管理
│   │   └── utils/           # 工具函数
│   └── src-tauri/           # Tauri 后端
└── knowledge/                # 知识库数据
```

## SuperClaude Framework 集成

> **重要**: 本项目配置了完整的 SuperClaude Framework skills
>
> - 自动加载: 是（通过 CLAUDE.md）
> - 框架模式: development
> - 可用 skills: 20+ 个

### 快速开始

```
/sc:load-core    # 加载完整框架（如果未自动加载）
/sc:help         # 验证 skills 是否可用
```

### 推荐 Skills

#### 核心开发 Skills

| Skill | 用途 | 示例 |
|-------|------|------|
| `/sc:design` | 系统架构设计 | `/sc:design "设计 LLVM 模块"` |
| `/sc:implement` | 功能代码实现 | `/sc:implement "实现 memory.yaml"` |
| `/sc:build` | 项目构建 | `/sc:build --release` |
| `/sc:test` | 测试执行 | `/sc:test -p flowsight-analysis` |
| `/sc:analyze` | 代码分析 | `/sc:analyze --security` |

#### 诊断与调试 Skills

| Skill | 用途 | 示例 |
|-------|------|------|
| `/sc:troubleshoot` | 问题诊断 | `/sc:troubleshoot "编译错误"` |
| `/sc:cleanup` | 代码清理 | `/sc:cleanup "移除死代码"` |
| `/sc:improve` | 代码改进 | `/sc:improve "性能优化"` |

#### 文档与研究 Skills

| Skill | 用途 | 示例 |
|-------|------|------|
| `/sc:document` | 文档生成 | `/sc:document "API 文档"` |
| `/sc:research` | 网络研究 | `/sc:research "LLVM IR 规范"` |
| `/sc:explain` | 代码解释 | `/sc:explain "解释这个函数"` |

#### 项目管理 Skills

| Skill | 用途 | 示例 |
|-------|------|------|
| `/sc:workflow` | 工作流生成 | `/sc:workflow "从计划生成"` |
| `/sc:estimate` | 任务估算 | `/sc:estimate "Phase 1 估算"` |
| `/sc:spawn` | 任务编排 | `/sc:spawn "分解任务"` |

#### 版本控制 Skills

| Skill | 用途 | 示例 |
|-------|------|------|
| `/sc:git` | Git 智能操作 | `/sc:git "提交更改"` |

### 详细指南

完整的使用指南请查看 [.claude/SKILLS-GUIDE.md](.claude/SKILLS-GUIDE.md)

### 推荐工作流

#### 1. 实现新功能
```
/sc:workflow "查看任务分解"  # 可选，从计划生成工作流
/sc:design "设计架构"        # 可选，需要设计时
/sc:implement "实现功能"
/sc:test "运行测试"
/sc:build "构建验证"
/sc:git "提交代码"
```

#### 2. 开发知识库
```
/sc:document "设计 knowledge.yaml 结构"
/sc:implement "实现知识库文件"
/sc:test "-p flowsight-knowledge"
/sc:git "添加知识库"
```

#### 3. 调试问题
```
/sc:troubleshoot "诊断问题"
/sc:analyze "分析根因"
/sc:implement "修复问题"
/sc:test "验证修复"
```

#### 4. 学习研究
```
/sc:research "查阅 LLVM/KLEE 文档"
/sc:document "记录学习笔记"
/sc:implement "应用到项目"
```

### 常用命令速查

#### 框架加载

| 命令 | 用途 |
|------|------|
| `/sc:load-core` | 加载完整框架 |
| `/sc:load-flags` | 加载配置模式 |
| `/sc:load-rules` | 加载开发规则 |

#### 核心 Skills

| 命令 | 用途 |
|------|------|
| `/sc:design` | 架构设计 |
| `/sc:implement` | 功能实现 |
| `/sc:build` | 项目构建 |
| `/sc:test` | 运行测试 |
| `/sc:troubleshoot` | 问题诊断 |
| `/sc:analyze` | 代码分析 |
| `/sc:document` | 文档生成 |
| `/sc:research` | 网络研究 |
| `/sc:workflow` | 工作流生成 |
| `/sc:git` | Git 智能操作 |

#### 辅助 Skills

| 命令 | 用途 |
|------|------|
| `/sc:help` | 查看所有 Skills |
| `/sc:cleanup` | 代码清理 |
| `/sc:improve` | 代码改进 |
| `/sc:explain` | 代码解释 |
| `/sc:estimate` | 任务估算 |
| `/sc:spawn` | 任务编排 |

### 文档位置

- **Skills 完整指南**: [.claude/SKILLS-GUIDE.md](.claude/SKILLS-GUIDE.md)
- **Skills 快速参考**: [.claude/SKILLS-QUICKREF.md](.claude/SKILLS-QUICKREF.md)
- **自动加载配置**: [.claude/AUTOLOAD.md](.claude/AUTOLOAD.md)
- **Agent 配置**: [.claude/agents/](.claude/agents/)
- **开发规则**: [.claude/DEV-RULES.md](.claude/DEV-RULES.md)

## 当前开发重点

### flowsight-analysis crate
- [x] 异步追踪 (async_tracker.rs)
- [x] 函数指针解析 (funcptr.rs)
- [x] 回调分析 (callback.rs)
- [x] 场景执行 (scenario.rs) ← 当前文件
- [ ] 约束传播增强
- [ ] 符号执行优化

## 质量标准

- 所有公开 API 必须有文档注释
- 关键逻辑需要单元测试覆盖
- 提交前运行 `cargo clippy`
- 遵循 Rust 所有权系统最佳实践
