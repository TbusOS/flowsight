# FlowSight Skills 速查卡

> FlowSight 项目 - SuperClaude Framework Skills 快速参考

## 加载框架

```bash
/sc:load-core
```

## Skills 速查

| 场景 | Skill | 命令 |
|------|-------|------|
| 架构设计 | Design | `/sc:design "设计内容"` |
| 功能实现 | Implement | `/sc:implement "实现内容"` |
| 项目构建 | Build | `/sc:build --debug` |
| 运行测试 | Test | `/sc:test -p flowsight-analysis` |
| 问题诊断 | Troubleshoot | `/sc:troubleshoot "问题"` |
| 代码分析 | Analyze | `/sc:analyze` |
| 文档生成 | Document | `/sc:document "内容"` |
| 网络研究 | Research | `/sc:research "主题"` |
| 工作流生成 | Workflow | `/sc:workflow` |
| Git 操作 | Git | `/sc:git "提交信息"` |

## 常用命令

### 后端开发
```bash
cargo build --workspace          # 构建所有 crates
cargo test --workspace           # 运行所有测试
cargo test -p flowsight-analysis # 测试分析模块
cargo clippy                     # 代码检查
cargo doc --no-deps              # 生成文档
```

### 前端开发
```bash
cd app && pnpm install           # 安装依赖
cd app && pnpm tauri dev         # 开发模式启动
cd app && pnpm build             # 构建前端
```

## FlowSight 专用场景

### 场景 1: 开发知识库
```bash
/sc:document "设计 memory.yaml 结构"
/sc:implement "实现 knowledge/platforms/linux-kernel/core/memory.yaml"
/sc:test "-p flowsight-knowledge"
/sc:git "添加 memory 知识库"
```

### 场景 2: 集成 LLVM
```bash
/sc:design "设计 flowsight-llvm 与分析引擎集成"
/sc:implement "实现 LLVM IR 解析器"
/sc:test "-p flowsight-llvm"
/sc:build "--debug"
```

### 场景 3: 调试问题
```bash
/sc:troubleshoot "诊断编译错误"
/sc:analyze "--security" "--performance"
/sc:implement "修复问题"
/sc:test "验证修复"
```

### 场景 4: 学习研究
```bash
/sc:research "LLVM IR 语法规范"
/sc:research "KLEE 符号执行"
/sc:document "记录学习笔记"
```

## 路径速查

```
flowsight/
├── crates/                    # Rust crates
│   ├── flowsight-core/       # 核心类型
│   ├── flowsight-parser/     # 解析器
│   ├── flowsight-analysis/   # 分析引擎
│   ├── flowsight-knowledge/  # 知识库
│   ├── flowsight-query/      # 查询引擎
│   ├── flowsight-symbolic/   # 符号执行
│   ├── flowsight-llvm/       # LLVM IR (新建)
│   ├── flowsight-klee/       # KLEE 集成 (计划)
│   └── flowsight-cli/        # CLI
├── app/                       # 前端
│   ├── src/
│   │   ├── components/       # React 组件
│   │   ├── store/           # 状态管理
│   │   └── utils/           # 工具
│   └── src-tauri/           # Tauri 后端
├── knowledge/                # 知识库
│   ├── platforms/
│   │   └── linux-kernel/
│   │       ├── core/        # 核心机制
│   │       ├── drivers/     # 驱动框架
│   │       └── sync/        # 同步原语
│   └── languages/           # 语言模式
└── docs/
    ├── design/              # 设计文档
    └── architecture/        # 架构文档
```

## 关键文件

| 文件 | 用途 |
|------|------|
| `CLAUDE.md` | 主配置 |
| `.claude/SKILLS-GUIDE.md` | Skills 详细指南 |
| `.claude/DEV-RULES.md` | 开发规则 |
| `docs/design/PROJECT-PLAN-V2.md` | 项目计划 |

## 质量标准

- 所有公开 API 有文档注释
- 关键逻辑有单元测试
- 提交前运行 `cargo clippy`
- 遵循 Rust 所有权最佳实践
- 不要添加 Co-Authored-By

---

*运行 `/sc:help` 查看所有可用 Skills*
