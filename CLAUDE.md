# FlowSight - Claude Code 配置

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

FlowSight 项目集成了 SuperClaude Framework，提供丰富的 Skills 来辅助开发。

### 加载框架

在会话开始时运行：
```
/sc:load-core
```

### 推荐 Skills

| 场景 | 推荐 Skill | 命令 |
|------|-----------|------|
| 设计新模块架构 | 架构设计 | `/sc:design "设计内容"` |
| 实现知识库/代码 | 功能实现 | `/sc:implement "实现内容"` |
| 编译项目 | 项目构建 | `/sc:build --debug` |
| 运行测试 | 测试执行 | `/sc:test -p <包名>` |
| 调试问题 | 问题诊断 | `/sc:troubleshoot "问题"` |
| 代码质量分析 | 代码分析 | `/sc:analyze` |
| 生成 API 文档 | 文档生成 | `/sc:document "内容"` |
| 查阅 LLVM/KLEE 文档 | 网络研究 | `/sc:research "主题"` |
| 项目规划 | 工作流生成 | `/sc:workflow` |
| Git 操作 | 智能 Git | `/sc:git "提交信息"` |

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

| 命令 | 用途 |
|------|------|
| `/sc:load-core` | 加载完整框架 |
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
| `/sc:help` | 查看所有 Skills |

### 文档位置

- **Skills 指南**: [.claude/SKILLS-GUIDE.md](.claude/SKILLS-GUIDE.md)
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
