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

### 加载框架
在会话开始时运行：
```
/sc:load-core
```

### 推荐工作流

#### 1. 实现新功能
```
/sc:design "设计新功能"      # 设计架构
/sc:workflow                # 生成工作流
/sc:implement               # 实现代码
/sc:test                    # 运行测试
/sc:analyze                 # 分析质量
```

#### 2. 代码改进
```
/sc:improve                 # 改进代码质量
/sc:cleanup                 # 清理代码
```

#### 3. 调试问题
```
/sc:troubleshoot            # 诊断问题
/sc:git                     # 智能 Git 操作
```

### 常用命令速查

| 命令 | 用途 |
|------|------|
| `/sc:load-core` | 加载完整框架 |
| `/sc:implement` | 实现功能代码 |
| `/sc:test` | 运行测试 |
| `/sc:analyze` | 代码分析 |
| `/sc:workflow` | 生成工作流 |
| `/sc:document` | 生成文档 |
| `/sc:git` | Git 智能操作 |

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
