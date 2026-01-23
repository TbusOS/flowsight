# FlowSight Skills 配置指南

> FlowSight 项目专用的 SuperClaude Framework Skills 配置
>
> **自动加载**: 支持任务感知的自动 skill 激活

## 加载框架

在每个 Claude Code 会话开始时，框架会自动加载。也可以手动加载：

```
/sc:load-core
```

或者使用简化版本：

```
/sc:load-flags --mode development
```

## 任务感知自动加载

FlowSight 项目支持任务感知的自动 skill 加载，无需手动选择：

### 文件路径触发

当您开始编辑以下路径的文件时，对应的 skills 会自动激活：

| 文件路径模式 | 工作任务类型 | 自动激活的 Skills |
|-------------|-------------|------------------|
| `app/src/components/**` 或 `*.tsx` | 前端 React 开发 | `/sc:ui-design`, `/sc:implement` |
| `app/src-tauri/` 或 `crates/` | Rust 后端开发 | `/sc:implement`, `/sc:build`, `/sc:test` |
| `knowledge/` | 知识库开发 | `/sc:document`, `/sc:implement` |
| `docs/` | 文档编写 | `/sc:document`, `/sc:explain` |

### 任务关键词触发

当您的任务描述包含以下关键词时，自动激活对应 skills：

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

---

## 推荐 Skills 速查

| 场景 | 推荐 Skill | 命令 |
|------|-----------|------|
| 设计新模块架构 | `/sc:design` | 设计 LLVM/KLEE 模块架构 |
| 实现知识库 YAML | `/sc:implement` | 生成 knowledge 文件 |
| 编译项目 | `/sc:build` | 完整构建流程 |
| 运行测试 | `/sc:test` | 执行单元测试 |
| 调试编译错误 | `/sc:troubleshoot` | 诊断问题 |
| 代码质量分析 | `/sc:analyze` | 分析代码质量 |
| 生成 API 文档 | `/sc:document` | 为模块生成文档 |
| 查阅 LLVM/KLEE 文档 | `/sc:research` | 网络研究 |
| 项目规划 | `/sc:workflow` | 从 PRD 生成工作流 |

---

## Skills 详细使用指南

### 1. `/sc:design` - 系统架构设计

**适用场景**：
- 设计 flowsight-llvm 模块架构
- 设计 flowsight-klee 模块架构
- 设计知识库结构
- 设计前端组件架构

**使用示例**：

```
/sc:design "设计 flowsight-llvm 模块架构"
```

**输出内容**：
- 模块结构设计
- 组件接口定义
- 数据流设计
- 技术选型建议

**针对 FlowSight 的提示**：
- 重点关注 LLVM IR 解析器和知识库的集成
- 考虑与现有 flowsight-analysis 的交互
- 遵循 Rust 最佳实践和项目编码规范

---

### 2. `/sc:implement` - 功能实现

**适用场景**：
- 实现 LLVM IR 解析器
- 编写知识库 YAML 文件
- 实现 KLEE 集成模块
- 添加前端组件

**使用示例**：

```
/sc:implement "实现 memory.yaml 知识库文件"
```

**工作流程**：
1. 加载项目上下文
2. 分析现有代码结构
3. 生成符合规范的代码
4. 集成到项目

**针对 FlowSight 的提示**：
- 知识库文件位于 `knowledge/platforms/linux-kernel/`
- 遵循现有知识库的 YAML 结构（参考 workqueue.yaml）
- Rust 代码遵循 `crates/` 下的模块结构

---

### 3. `/sc:build` - 项目构建

**适用场景**：
- 完整项目构建
- 调试构建
- 发布构建

**使用示例**：

```
/sc:build "--debug"        # 调试构建
/sc:build "--release"      # 发布构建
/sc:build "--target x86_64-unknown-linux-gnu"  # 交叉编译
```

**完整构建流程**：
```bash
# Rust 后端
cargo build --workspace

# 前端
cd app && pnpm install && pnpm build

# Tauri 应用
cd app && pnpm tauri build
```

---

### 4. `/sc:test` - 测试执行

**适用场景**：
- 运行所有测试
- 运行特定模块测试
- 生成测试报告

**使用示例**：

```
/sc:test "--workspace"           # 所有测试
/sc:test "-p flowsight-analysis" # 分析模块
/sc:test "--lib --release"       # 库测试发布模式
```

**常用命令**：

```bash
# 单元测试
cargo test --workspace

# 特定包测试
cargo test -p flowsight-analysis
cargo test -p flowsight-parser
cargo test -p flowsight-knowledge

# 测试覆盖率
cargo tarpaulin --workspace

# 前端测试
cd app && pnpm test
```

---

### 5. `/sc:troubleshoot` - 问题诊断

**适用场景**：
- 编译错误诊断
- 运行时错误诊断
- 性能问题诊断
- 内存问题诊断

**使用示例**：

```
/sc:troubleshoot "inkwell 编译错误"
/sc:troubleshoot "KLEE 集成失败"
/sc:troubleshoot "前端构建错误"
```

**诊断流程**：
1. 收集错误信息
2. 分析根本原因
3. 提供解决方案
4. 验证修复效果

---

### 6. `/sc:analyze` - 代码分析

**适用场景**：
- 代码质量评估
- 安全漏洞检测
- 性能瓶颈分析
- 代码重复检测

**使用示例**：

```
/sc:analyze "分析 flowsight-llvm 代码质量"
/sc:analyze "--security" "--performance" "代码审计"
```

**分析维度**：
- 代码复杂度
- 依赖管理
- 错误处理
- 内存安全
- 并发安全

---

### 7. `/sc:document` - 文档生成

**适用场景**：
- API 文档生成
- 模块文档编写
- 设计文档生成

**使用示例**：

```
/sc:document "为 flowsight-llvm 生成 API 文档"
/sc:document "编写 memory.yaml 知识库说明"
```

**文档类型**：
- Rust doc comments → `cargo doc`
- API 文档 → Markdown
- 设计文档 → architecture docs

---

### 8. `/sc:research` - 网络研究

**适用场景**：
- 查阅 LLVM IR 文档
- 查阅 KLEE 文档
- 查阅 Linux 内核文档
- 查找最佳实践

**使用示例**：

```
/sc:research "LLVM IR 语法规范"
/sc:research "KLEE 符号执行教程"
/sc:research "inkwell Rust 绑定用法"
```

**研究深度**：
- `--depth quick` - 快速概览
- `--depth medium` - 详细研究
- `--depth very-thorough` - 深度研究

---

### 9. `/sc:workflow` - 工作流生成

**适用场景**：
- 从项目计划生成开发工作流
- 规划 Phase 实施步骤
- 任务分解

**使用示例**：

```
/sc:workflow "从 PROJECT-PLAN-V2.md 生成 Phase 1 工作流"
```

**输出内容**：
- 详细任务分解
- 依赖关系
- 时间估算
- 验收标准

---

### 10. `/sc:git` - Git 智能操作

**适用场景**：
- 智能提交
- 分支管理
- 代码审查

**使用示例**：

```
/sc:git "提交 LLVM 解析器实现"
/sc:git "--push" "推送更改"
```

---

### 11. `/sc:ui-design` - UI 组件设计（FlowSight 专用）

**适用场景**：
- 为 FlowSight 创建美观的前端组件
- 美化现有 UI 界面
- 设计新的交互面板
- 统一项目视觉风格

**使用示例**：

```
/sc:ui-design "创建节点详情面板组件"
/sc:ui-design "美化工具栏样式"
/sc:ui-design "设计场景对比视图"
/sc:ui-design "改进执行流图节点样式"
```

**设计规范**：
- 遵循 FlowSight 深色主题配色（主色 #3b82f6）
- 使用玻璃态效果 (backdrop-blur)
- 添加微光和渐变效果
- 统一的动画曲线和过渡时长

**输出内容**：
- 完整的 React 组件代码
- TypeScript 类型定义
- CSS/Tailwind 样式
- Framer Motion 动画
- 使用示例

**示例：创建节点详情面板**

```
/sc:ui-design "创建节点详情面板，支持函数/分支/语句三种视图切换"
```

这将生成一个包含：
- 标签页切换（函数/分支/语句）
- 玻璃态背景
- 平滑动画过渡
- 符合 FlowSight 主题的组件

---

### 12. `/sc:interaction-design` - 交互设计（FlowSight 专用）

**适用场景**：
- 设计用户操作流程和信息架构
- 设计交互面板和导航系统
- 优化用户体验和操作效率
- 制定交互规范和模式库

**使用示例**：

```
/sc:interaction-design "设计符号执行配置流程"
/sc:interaction-design "设计知识库编辑交互"
/sc:interaction-design "优化结果分析面板交互"
/sc:interaction-design "设计错误恢复流程"
```

**交互规范**：
- 用户流程设计（3 步以内完成核心操作）
- 信息架构设计（清晰的层级结构）
- 交互模式库（拖放、搜索、展开、面板等）
- 键盘交互规范（全键盘可操作）
- 反馈系统（成功/错误/警告/加载状态）

**输出内容**：
- 交互规范文档
- 用户流程图
- 线框图/低保真原型
- 交互说明文档
- 验收标准

**示例：符号执行配置流程设计**

```
/sc:interaction-design "设计符号执行配置的交互流程"
```

这将输出：
- 任务流程（选择目标 → 配置参数 → 确认执行 → 开始执行）
- 信息架构（全局导航 → 功能模块 → 详细面板）
- 交互模式（拖放选择、实时验证、分步展开）
- 反馈设计（参数验证、执行进度、完成后通知）

---

### 13. `/sc:collaborative-dev` - 三智能体协作开发（FlowSight 专用）

**适用场景**：
- 大型功能开发（需要前后端同步）
- UI 美化与功能开发并行
- 需要测试保障的复杂任务
- 追求开发效率的最大化

**使用示例**：

```
/sc:collaborative-dev "实现节点详情面板"
/sc:collaborative-dev "添加场景对比功能"
/sc:collaborative-dev "美化执行流图节点样式"
```

**三个智能体**：

| 智能体 | 角色 | 专注领域 |
|--------|------|----------|
| 🤖 Core-Agent | 核心开发 | Rust 后端、API、核心逻辑 |
| 🎨 UI-Agent | 前端美化 | React 组件、交互设计、视觉效果 |
| 🧪 Test-Agent | 测试保障 | 单元测试、集成测试、问题诊断 |

**协作机制**：
- 共享上下文，实时同步
- 消息通信，任务委派
- 并行开发，顺序合并

**高级用法**：

```
# 只用特定 Agent
/sc:collaborative-dev "优化性能" --only: "core"

# 强制顺序执行
/sc:collaborative-dev "添加功能" --sequential: true

# 指定任务描述
/sc:collaborative-dev "实现详情面板"
  --backend: "添加 Rust API"
  --frontend: "创建 UI 组件"
  --tests: "编写测试"
```

---

## FlowSight 专用场景

### 场景 1: 实现新的知识库文件

```
/sc:document "设计 netdev.yaml 网络设备知识库结构"
/sc:implement "实现 knowledge/platforms/linux-kernel/drivers/netdev.yaml"
/sc:test "-p flowsight-knowledge" "测试知识库加载"
/sc:git "添加 netdev.yaml 网络设备知识库"
```

### 场景 2: 集成 LLVM IR 解析

```
/sc:design "设计 flowsight-llvm 模块与现有分析引擎的集成"
/sc:implement "实现 LLVM IR 解析器"
/sc:build "--debug" "构建验证"
/sc:test "-p flowsight-llvm" "运行单元测试"
/sc:git "集成 LLVM IR 解析模块"
```

### 场景 3: 集成 KLEE 符号执行

```
/sc:design "设计 flowsight-klee 按需符号执行架构"
/sc:implement "实现 KLEE 测试代码生成器"
/sc:troubleshoot "调试 KLEE 集成问题"
/sc:git "添加 KLEE 符号执行支持"
```

### 场景 4: 前端组件开发

```
/sc:design "设计新的 FlowView 组件架构"
/sc:implement "实现 FlowTextView 增强功能"
/sc:build "--debug" "前端构建"
/sc:git "增强执行流展示组件"
```

### 场景 5: UI 美化与设计

```
/sc:ui-design "创建节点详情面板组件"
/sc:ui-design "美化工具栏样式"
/sc:ui-design "设计场景对比视图"
/sc:build "--debug" "验证 UI 构建"
/sc:git "添加 UI 组件"
```

---

## 命令速查表

### 框架加载

| 命令 | 用途 |
|------|------|
| `/sc:load-core` | 加载完整框架 |
| `/sc:load-flags --mode development` | 开发模式 |
| `/sc:load-rules` | 加载开发规则 |

### 核心 Skills

| 命令 | 用途 | 常用参数 |
|------|------|---------|
| `/sc:design` | 架构设计 | `"设计内容"` |
| `/sc:implement` | 功能实现 | `"实现内容"` |
| `/sc:build` | 项目构建 | `--debug`, `--release` |
| `/sc:test` | 运行测试 | `-p <包名>`, `--workspace` |
| `/sc:troubleshoot` | 问题诊断 | `"问题描述"` |
| `/sc:analyze` | 代码分析 | `--security`, `--performance` |
| `/sc:document` | 文档生成 | `"文档内容"` |
| `/sc:research` | 网络研究 | `"研究主题"`, `--depth` |
| `/sc:workflow` | 工作流生成 | `"输入文件"` |
| `/sc:ui-design` | UI 组件设计 | `"创建组件描述"` |
| `/sc:collaborative-dev` | 三智能体协作 | `"任务描述"`, `--parallel` |
| `/sc:git` | Git 操作 | `"提交信息"`, `--push` |

### 辅助 Skills

| 命令 | 用途 |
|------|------|
| `/sc:improve` | 代码改进 |
| `/sc:cleanup` | 代码清理 |
| `/sc:explain` | 代码解释 |
| `/sc:estimate` | 任务估算 |

---

## 配置文件

### SuperClaude 配置

配置文件位于项目根目录的 `.claude/` 目录：

| 文件 | 用途 |
|------|------|
| `CLAUDE.md` | 主配置和常用命令 |
| `AGENTS.md` | 内置 Agent 配置 |
| `DEV-RULES.md` | 开发规则 |
| `SKILLS-GUIDE.md` | Skills 使用指南（本文档） |
| `opencode.jsonc` | Agent 系统配置 |

### 加载方式

在 Claude Code 会话中：

1. **自动加载**: 项目根目录有 `CLAUDE.md` 时自动加载
2. **手动加载**: 运行 `/sc:load-core` 加载完整框架
3. **部分加载**: 运行 `/sc:load-flags --mode <mode>`

---

## 最佳实践

### 1. 开发流程

```
开始开发
    │
    ▼
/sc:workflow "查看任务分解"
    │
    ▼
/sc:design "设计架构" (如需要)
    │
    ▼
/sc:implement "实现功能"
    │
    ▼
/sc:test "运行测试"
    │
    ▼
/sc:build "构建验证"
    │
    ▼
/sc:git "提交代码"
```

### 2. 问题处理

```
发现问题
    │
    ▼
/sc:troubleshoot "诊断问题"
    │
    ▼
/sc:analyze "分析根因"
    │
    ▼
/sc:implement "修复问题"
    │
    ▼
/sc:test "验证修复"
```

### 3. 学习研究

```
需要学习新知识
    │
    ▼
/sc:research "研究主题"
    │
    ▼
/sc:document "记录笔记"
    │
    ▼
/sc:implement "应用到项目"
```

---

## 常见问题

### Q: 如何开始使用 Skills？

A: 在每个新会话开始时，运行 `/sc:load-core` 加载完整框架。

### Q: Skills 可以一起使用吗？

A: 可以。Skills 可以串联使用，形成完整的工作流。

### Q: 如何查看所有可用的 Skills？

A: 运行 `/sc:help` 查看所有可用 Skills。

### Q: 如何自定义 Skills 行为？

A: 通过命令行参数自定义行为，如 `/sc:build --release`。

### Q: Skills 支持离线使用吗？

A: 大部分 Skills 支持离线，但 `/sc:research` 需要网络连接。

---

## 参考资源

- [SuperClaude Framework 文档](https://github.com/superclaude-ai/superclaude-framework)
- [FlowSight 项目计划](docs/design/PROJECT-PLAN-V2.md)
- [LLVM IR 文档](https://llvm.org/docs/LangRef.html)
- [KLEE 文档](https://klee.github.io/)
- [inkwell 文档](https://github.com/TheDan64/inkwell)

---

*本文档由 SuperClaude Framework 自动生成*
