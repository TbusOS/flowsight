# SuperClaude Framework - FlowSight 完整配置

## 自动加载配置

本配置确保每次开发 FlowSight 项目时自动加载完整的 SuperClaude Framework skills。

## 加载方式

### 方式 1: 会话开始时自动加载 (推荐)

在 Claude Code 中，项目根目录的 `CLAUDE.md` 会在会话开始时自动加载。
当前配置已包含：
- `/sc:load-core` 指令
- **任务感知自动加载**: 根据文件路径和任务关键词自动激活相关 skills

### 方式 2: 手动加载

在会话中输入：
```
/sc:load-core
```

## 任务感知自动加载

FlowSight 项目支持任务感知的自动 skill 加载：

### 文件路径触发

| 文件路径模式 | 工作任务类型 | 自动激活的 Skills |
|-------------|-------------|------------------|
| `app/src/components/**` 或 `*.tsx` | 前端 React 开发 | `/sc:ui-design`, `/sc:implement` |
| `app/src-tauri/` 或 `crates/` | Rust 后端开发 | `/sc:implement`, `/sc:build`, `/sc:test` |
| `knowledge/` | 知识库开发 | `/sc:document`, `/sc:implement`, `/sc:test` |
| `docs/` | 文档编写 | `/sc:document`, `/sc:explain` |
| `.claude/` | 框架配置 | `/sc:load-core`, `/sc:load-flags` |

### 任务关键词触发

当用户的任务描述包含以下关键词时，自动激活对应 skills：

| 任务关键词 | 自动激活的 Skills |
|-----------|------------------|
| "美化", "UI", "界面" | `/sc:ui-design` |
| "交互", "UX", "用户体验", "流程", "布局" | `/sc:interaction-design` |
| "协作", "多个", "一起", "并行", "三智能体" | `/sc:collaborative-dev` |
| "设计", "架构", "design" | `/sc:design` |
| "实现", "implement" | `/sc:implement` |
| "构建", "编译", "build" | `/sc:build` |
| "测试", "test" | `/sc:test` |
| "调试", "问题" | `/sc:troubleshoot` |

## 完整 Skills 列表

### 核心开发 Skills

| Skill | 用途 | 使用场景 |
|-------|------|---------|
| `/sc:design` | 系统架构设计 | 设计新模块、API、数据结构 |
| `/sc:implement` | 功能代码实现 | 实现功能、添加新特性 |
| `/sc:build` | 项目构建 | 编译、构建、发布 |
| `/sc:test` | 测试执行 | 单元测试、集成测试 |
| `/sc:analyze` | 代码分析 | 质量审计、安全检查 |
| `/sc:ui-design` | UI 组件设计 | 美化界面、创建组件 |
| `/sc:interaction-design` | 交互设计 | 用户流程、信息架构、面板系统 |
| `/sc:collaborative-dev` | 三智能体协作 | 并行开发、后端+前端+测试同步 |

### 诊断与调试 Skills

| Skill | 用途 | 使用场景 |
|-------|------|---------|
| `/sc:troubleshoot` | 问题诊断 | 编译错误、运行时错误 |
| `/sc:cleanup` | 代码清理 | 重构、移除死代码 |
| `/sc:improve` | 代码改进 | 性能优化、质量改进 |

### 文档与研究 Skills

| Skill | 用途 | 使用场景 |
|-------|------|---------|
| `/sc:document` | 文档生成 | API 文档、设计文档 |
| `/sc:research` | 网络研究 | 查阅文档、学习新技术 |
| `/sc:explain` | 代码解释 | 解释复杂代码逻辑 |

### 项目管理 Skills

| Skill | 用途 | 使用场景 |
|-------|------|---------|
| `/sc:workflow` | 工作流生成 | 从 PRD 生成开发计划 |
| `/sc:estimate` | 任务估算 | 评估开发工作量 |
| `/sc:spawn` | 任务编排 | 复杂任务分解与委派 |

### 版本控制 Skills

| Skill | 用途 | 使用场景 |
|-------|------|---------|
| `/sc:git` | Git 智能操作 | 提交、合并、代码审查 |

### 辅助 Skills

| Skill | 用途 | 使用场景 |
|-------|------|---------|
| `/sc:help` | 帮助信息 | 查看所有可用 Skills |
| `/sc:load-core` | 加载框架 | 加载完整功能 |
| `/sc:load-flags` | 加载配置 | 加载指定模式配置 |
| `/sc:load-rules` | 加载规则 | 加载开发规则 |

## FlowSight 专用 Skills 映射

### 场景 → 推荐 Skills

| 开发场景 | 推荐 Skills 组合 |
|---------|-----------------|
| 实现知识库 YAML | `/sc:document` → `/sc:implement` → `/sc:test` → `/sc:git` |
| 集成 LLVM IR | `/sc:design` → `/sc:implement` → `/sc:test` → `/sc:build` → `/sc:git` |
| 集成 KLEE | `/sc:design` → `/sc:research` → `/sc:implement` → `/sc:test` → `/sc:git` |
| 前端组件开发 | `/sc:design` → `/sc:implement` → `/sc:build` → `/sc:git` |
| UI 美化设计 | `/sc:ui-design` → `/sc:build` → `/sc:git` |
| 大型功能开发 | `/sc:collaborative-dev` → `/sc:build` → `/sc:git` |
| 调试编译错误 | `/sc:troubleshoot` → `/sc:analyze` → `/sc:implement` → `/sc:test` |
| 代码质量检查 | `/sc:analyze` → `/sc:improve` → `/sc:test` → `/sc:git` |
| 学习新技术 | `/sc:research` → `/sc:document` → `/sc:implement` |
| 项目规划 | `/sc:workflow` → `/sc:estimate` → `/sc:spawn` |

## 高级用法

### 并行执行多个 Skills

```
/sc:implement "功能A"
/sc:implement "功能B"
```

### 链式工作流

```
/sc:workflow "从 PROJECT-PLAN-V2.md 生成 Phase 1 工作流"
```

### 带参数调用

```
/sc:build "--release"                    # 发布构建
/sc:test "-p flowsight-analysis --lib"   # 只运行库测试
/sc:research "LLVM IR 语法" --depth medium
/sc:analyze "--security --performance"
```

## 配置文件结构

```
.claude/
├── CLAUDE.md              # 主配置（自动加载）
├── SKILLS-GUIDE.md        # Skills 详细指南
├── SKILLS-QUICKREF.md     # Skills 快速参考
├── DEV-RULES.md           # 开发规则
├── AUTOLOAD.md            # 本文件（自动加载配置）
├── agents/                # 内置 Agent 配置
│   ├── analyze.md
│   ├── explain.md
│   └── search.md
└── skills/                # 自定义 Skills
    ├── ui-design.md       # UI 设计技能
    ├── interaction-design.md  # 交互设计技能
    └── collaborative-dev.md   # 多智能体协作技能
```

## 会话初始化流程

```
1. 用户启动 Claude Code
2. 自动加载项目根目录 CLAUDE.md
3. 执行 /sc:load-core 加载完整框架
4. 所有 Skills 可用
5. 开始开发
```

## 验证 Skills 是否加载

运行以下命令验证：
```
/sc:help
```

输出应显示所有可用的 Skills 列表。

## 故障排除

### Skills 无法使用

1. 确保运行了 `/sc:load-core`
2. 检查 CLAUDE.md 是否在项目根目录
3. 重启 Claude Code 会话

### 命令未找到

1. 确认 SuperClaude Framework 已安装
2. 检查 MCP 服务器配置
3. 联系框架支持

## 相关资源

- SuperClaude Framework: https://github.com/superclaude-ai/superclaude-framework
- FlowSight 项目: https://github.com/TbusOS/flowsight
- Skills 文档: [.claude/SKILLS-GUIDE.md](SKILLS-GUIDE.md)

---

*本配置由 FlowSight 项目自动加载*
