# FlowSight - Claude Code 配置

> **自动加载**: 启动时自动加载完整 SuperClaude Framework skills
>
> **任务感知**: 根据当前工作上下文自动激活相关 skills
>
> 运行 `/sc:help` 验证所有 skills 已加载

---

## 🚀 自动团队开发模式

**直接告诉我开发任务，系统会自动启动团队开发：**

```
用户: 实现执行流树视图组件
系统: 自动分析 → 分配 UI-Dev → 开发 → E2E-Tester 测试 → 完成
```

**团队角色：**
- 🦀 **Rust-Dev** - 后端/分析引擎
- 🎨 **UI-Dev** - 前端/可视化
- 🧪 **Unit-Tester** - 单元测试
- 🖥️ **E2E-Tester** - E2E 测试
- 🔧 **Debug-Dev** - Bug 修复

**任务看板**: `.claude/tasks/board.md`

---

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
> - 任务感知: 启用（根据文件路径自动激活相关 skills）

### 任务感知自动加载

本项目支持根据当前工作上下文自动加载相关 skills。当您开始处理特定类型的任务时，对应的 skills 会自动激活：

#### 文件路径 → Skills 映射

| 文件路径模式 | 工作任务类型 | 自动激活的 Skills |
|-------------|-------------|------------------|
| `app/src/components/**` 或 `*.tsx` | 前端 React 开发 | `/sc:ui-design`, `/sc:implement` |
| `app/src-tauri/` 或 `crates/` | Rust 后端开发 | `/sc:implement`, `/sc:build`, `/sc:test` |
| `knowledge/` | 知识库开发 | `/sc:document`, `/sc:implement`, `/sc:test` |
| `docs/` | 文档编写 | `/sc:document`, `/sc:explain` |
| `.claude/` | 框架配置 | `/sc:load-core`, `/sc:load-flags` |
| 检测到编译错误 | 问题调试 | `/sc:troubleshoot`, `/sc:analyze` |
| 检测到 UI 样式问题 | UI 美化 | `/sc:ui-design` |

#### 任务关键词 → Skills 映射

| 任务关键词 | 自动激活的 Skills |
|-----------|------------------|
| "设计", "架构", "design" | `/sc:design` |
| "实现", "实现功能", "implement" | **自动团队调度** (见下方) |
| "构建", "编译", "build" | `/sc:build` |
| "测试", "test" | `/sc:test` |
| "分析", "analyze" | `/sc:analyze` |
| "调试", "问题", "troubleshoot" | `/sc:troubleshoot` |
| "美化", "UI", "界面" | `/sc:ui-design` |
| "协作", "多个", "一起", "并行", "三智能体" | `/sc:collaborative-dev` |
| "文档", "document" | `/sc:document` |
| "研究", "research" | `/sc:research` |
| "解释", "explain" | `/sc:explain` |
| "Git", "提交", "git" | `/sc:git` |
| "清理", "cleanup" | `/sc:cleanup` |
| "改进", "优化", "improve" | `/sc:improve` |

#### 🚀 自动团队调度 (重要)

**当检测到开发任务时，自动启动团队开发模式：**

| 任务类型 | 触发关键词 | 自动分配的 Agent |
|---------|-----------|-----------------|
| **后端开发** | Rust, 后端, API, 解析器, 知识库, LLVM, crate | 🦀 Rust-Dev → 🧪 Unit-Tester |
| **前端开发** | UI, 组件, 界面, 前端, 样式, 交互, 可视化 | 🎨 UI-Dev → 🖥️ E2E-Tester |
| **全栈开发** | 完整功能, 全栈, 端到端, 新功能 | 🦀+🎨 并行 → 🧪+🖥️ 测试 |
| **Bug 修复** | 修复, Bug, 问题, 错误, fix | 🔧 Debug-Dev → 重测 |

**自动执行流程：**
```
用户输入任务 → 分析 → 分配 Agent → 并行开发 → 自动测试 → 提交 GitHub → 完成
                                                    ↓
                                              发现 Bug → Debug 修复 → 重测
```

**无需手动调用**，系统会自动：
1. 分析任务类型和复杂度
2. 分配合适的 Agent
3. 更新任务看板
4. 开发完成后自动触发测试
5. 发现 Bug 自动分配给 Debug-Dev
6. **测试通过后立即提交 GitHub** ← 重要!

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

#### UI 开发 Skills

| Skill | 用途 | 示例 |
|-------|------|------|
| `/sc:ui-design` | UI 组件设计 | `/sc:ui-design "美化执行流程图"` |
| `/sc:implement` | 功能代码实现 | `/sc:implement "实现节点详情面板"` |

#### 多智能体协作 Skills

| Skill | 用途 | 示例 |
|-------|------|------|
| `/sc:collaborative-dev` | 三智能体协作开发 | `/sc:collaborative-dev "实现详情面板"` |
| `/sc:team` | 五角色团队开发 | `/sc:team "实现执行流可视化"` |

### 团队协作开发 (新)

> **五角色 Agent 团队** - 适用于复杂功能开发

#### 团队角色

| 角色 | Agent | 职责 | 配置文件 |
|------|-------|------|---------|
| 🦀 后端开发 | Rust-Dev | Rust 分析引擎、知识库、Tauri API | `.claude/agents/rust-dev.md` |
| 🎨 前端开发 | UI-Dev | React 组件、可视化、交互 | `.claude/agents/ui-dev.md` |
| 🧪 单元测试 | Unit-Tester | Rust 单元测试、集成测试 | `.claude/agents/unit-tester.md` |
| 🖥️ E2E 测试 | E2E-Tester | 端到端测试、UI 测试 | `.claude/agents/e2e-tester.md` |
| 🔧 问题修复 | Debug-Dev | Bug 定位、快速修复 | `.claude/agents/debug-dev.md` |

#### 协作流程

```
┌──────────────────────────────────────────────────────────────────┐
│  开发阶段                                                         │
│  ═════════                                                        │
│  Rust-Dev ──┐                                                     │
│             ├──► 功能完成 ──► Unit-Tester ──┐                     │
│  UI-Dev ────┘                              │                      │
│                           E2E-Tester ──────┤                      │
│                                            │                      │
│                                            ▼                      │
│  修复阶段                      发现 Bug ──► Debug-Dev             │
│  ═════════                                   │                    │
│                                             ▼                     │
│                               修复完成 ──► 请求重测               │
└──────────────────────────────────────────────────────────────────┘
```

#### 使用方式

```bash
# 启动团队开发
/sc:team "实现执行流可视化功能"

# 指定角色
/sc:team:rust-dev "实现知识库加载器"
/sc:team:ui-dev "实现节点详情面板"
/sc:team:unit-test "测试 LLVM IR 解析"
/sc:team:e2e-test "测试执行流交互"
/sc:team:debug "修复 Bug #12"

# 查看状态
/sc:team:status
```

#### 任务追踪

- **任务看板**: `.claude/tasks/board.md`
- **Agent 任务**: `.claude/tasks/{agent}.md`
- **测试报告**: `.claude/reports/test-reports/`
- **Bug 报告**: `.claude/reports/bug-reports/`
- **修复报告**: `.claude/reports/fix-reports/`

#### 消息通知

```
开发 → 测试:  📢 @Unit-Tester 功能完成，请测试
测试 → Debug: 🐛 @Debug-Dev 发现问题 [详情]
Debug → 测试: ✅ @Unit-Tester 已修复，请重测
测试 → 全员: ✅ @All 功能验证通过
```

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

#### 5. UI 美化开发
```
/sc:ui-design "设计新组件样式"
/sc:implement "实现 UI 组件"
/sc:build "构建验证"
/sc:git "提交 UI 更改"
```

#### 6. 多智能体协作开发
```
/sc:collaborative-dev "实现节点详情面板"
/sc:build "构建验证"
/sc:git "提交完整功能"
```

#### 7. 大型功能并行开发
```
/sc:collaborative-dev "添加场景对比功能"
  --backend: "实现 Rust API"
  --frontend: "创建 UI 组件"
  --tests: "编写测试"
/sc:build "完整构建"
/sc:git "提交完整功能"
```

#### 6. 执行流可视化开发
```
/sc:ui-design "设计交互式流程图"
/sc:implement "实现节点详情面板"
/sc:build "验证可视化效果"
/sc:test "测试交互功能"
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
| `/sc:ui-design` | UI 组件设计 |
| `/sc:collaborative-dev` | 三智能体协作 |
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

## UI 测试规则

### 桌面自动化测试 (强制)

**重要**: 本项目配置了桌面自动化 UI 测试框架，所有 UI 交互测试必须使用此工具，无需用户提醒。

测试框架位置: `app/tests/desktop/`

#### 何时使用

| 场景 | 测试命令 |
|------|----------|
| 修复 UI Bug 后验证 | `python3 -m tests.desktop --smoke` |
| UI 样式修改后验证 | `python3 -m tests.desktop --phase visual` |
| 交互功能修改后验证 | `python3 -m tests.desktop --phase interactive` |
| 完整 UI 测试 | `python3 -m tests.desktop --full` |

**注意**: 运行前请确保应用已启动 (如 `pnpm tauri dev`)

#### 自动化测试阶段

- **Layout**: 布局测试（对齐、间距、重叠）
- **Visual**: 视觉测试（颜色对比、图标渲染）
- **Interactive**: 交互测试（快捷键、焦点状态）
- **State**: 状态测试（组件状态切换）
- **Aesthetic**: 美学测试（视觉一致性）

#### 任务感知

当检测到以下关键词时，自动激活桌面测试:
- "UI 测试", "界面测试", "交互测试"
- "视觉验证", "样式验证"
- "按钮测试", "快捷键测试"

### 文档创建规则

**重要**: 所有项目文档必须在 `docs/` 目录下创建，禁止在 `app/` 或其他位置创建文档。

#### 文档目录结构

```
docs/
├── README.md              # 项目说明
├── architecture/          # 架构文档
├── design/                # 设计文档
├── developer/             # 开发者指南
├── plans/                 # 计划文档 (如 UI 优化计划)
├── testing/               # 测试文档
├── training/              # 培训文档
├── user-guide/            # 用户指南
└── images/                # 文档图片
```

#### 文档命名规范

| 类型 | 命名模式 | 示例 |
|------|----------|------|
| 计划文档 | `{feature}-plan.md` | ui-optimization.md |
| 架构文档 | `architecture/*.md` | backend-architecture.md |
| 设计文档 | `design/*.md` | ui-design-guidelines.md |
| 开发指南 | `developer/*.md` | setup-guide.md |
| 测试文档 | `testing/*.md` | e2e-testing-guide.md |

#### 创建文档时的操作

1. **选择正确目录**: 根据文档类型放入对应子目录
2. **更新 README**: 在 `docs/README.md` 中添加文档索引
3. **链接引用**: 在 CLAUDE.md 或相关文档中添加链接

#### 任务感知

当检测到以下关键词时，检查文档位置:
- "创建文档", "写文档", "文档"
- "更新计划", "修改计划"
- "添加指南", "编写指南"
