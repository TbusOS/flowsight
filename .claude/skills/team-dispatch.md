# FlowSight 团队自动调度 Skill

> **核心调度 Skill** - 自动分析任务并分配给对应 Agent 团队
>
> **自动激活**: 当检测到开发任务时自动启动

## 自动触发条件

当任务描述包含以下关键词时，**自动激活团队开发模式**：

| 触发关键词 | 分配的 Agent |
|-----------|-------------|
| "实现", "开发", "创建", "添加" | 根据内容分析分配 |
| "Rust", "后端", "API", "解析器", "知识库" | Rust-Dev |
| "UI", "组件", "界面", "前端", "样式" | UI-Dev |
| "测试", "验证", "覆盖率" | Unit-Tester / E2E-Tester |
| "修复", "Bug", "问题", "错误" | Debug-Dev |
| "完整功能", "全栈", "端到端" | 全部 Agent |

## 任务分析引擎

```
用户输入任务
      │
      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    任务分析引擎                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Step 1: 关键词检测                                              │
│  ═══════════════════                                             │
│  检测任务描述中的关键词，确定任务类型                            │
│                                                                  │
│  Step 2: 文件路径分析                                            │
│  ═══════════════════                                             │
│  如果涉及特定文件，根据路径判断技术栈                            │
│  • crates/* → Rust-Dev                                          │
│  • app/src/components/* → UI-Dev                                │
│  • app/src-tauri/* → Rust-Dev                                   │
│  • tests/* → Tester                                             │
│                                                                  │
│  Step 3: 复杂度评估                                              │
│  ═══════════════════                                             │
│  • 简单任务 → 单 Agent                                          │
│  • 中等任务 → 开发 + 测试                                       │
│  • 复杂任务 → 全部 Agent                                        │
│                                                                  │
│  Step 4: 分配并启动                                              │
│  ═══════════════════                                             │
│  创建任务文件，启动对应 Agent                                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 任务分配规则

### 规则 1: 后端任务 → Rust-Dev + Unit-Tester

触发条件:
- 关键词: Rust, 后端, API, 解析器, 知识库, LLVM, Tauri, crate
- 文件路径: `crates/*`, `app/src-tauri/*`, `knowledge/*`

分配:
```
Rust-Dev: 实现功能
Unit-Tester: 准备测试 (开发完成后自动触发)
```

### 规则 2: 前端任务 → UI-Dev + E2E-Tester

触发条件:
- 关键词: UI, 组件, 界面, 前端, 样式, 交互, React, 可视化
- 文件路径: `app/src/components/*`, `app/src/styles/*`

分配:
```
UI-Dev: 实现组件
E2E-Tester: 准备 E2E 测试 (开发完成后自动触发)
```

### 规则 3: 完整功能 → 全部 Agent

触发条件:
- 关键词: 完整功能, 全栈, 端到端, 新功能
- 涉及前后端: 同时有 Rust 和 React 代码

分配:
```
Rust-Dev: 后端 API (并行)
UI-Dev: 前端组件 (并行)
Unit-Tester: 单元测试 (开发完成后)
E2E-Tester: E2E 测试 (开发完成后)
Debug-Dev: 待命 (发现 Bug 后激活)
```

### 规则 4: Bug 修复 → Debug-Dev + Tester

触发条件:
- 关键词: 修复, Bug, 问题, 错误, fix
- 来源: 测试报告

分配:
```
Debug-Dev: 定位并修复
Unit-Tester/E2E-Tester: 重测 (修复完成后)
```

## 完整工作流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        FlowSight 开发流程 (自动化)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  📋 任务输入                                                                 │
│      │                                                                       │
│      ▼                                                                       │
│  🔍 任务分析 ─────► 分配 Agent                                              │
│      │                                                                       │
│      ▼                                                                       │
│  🦀🎨 并行开发 ──► 开发完成                                                  │
│      │                                                                       │
│      ▼                                                                       │
│  🧪🖥️ 并行测试                                                               │
│      │                                                                       │
│      ├─── 测试通过 ──► 📝 Git Commit ──► 🚀 Push GitHub ──► ✅ 完成         │
│      │                                                                       │
│      └─── 发现 Bug ──► 🔧 Debug-Dev 修复 ──► 重测 ──► (循环)                │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**关键点：小功能完成 + 测试通过 = 立即提交 GitHub**

## 自动执行流程

### 1. 任务接收

当用户输入任务时，系统自动：

```
📋 收到任务: [任务描述]

🔍 分析中...
   ├─ 关键词: [检测到的关键词]
   ├─ 类型: [后端/前端/全栈/修复]
   ├─ 复杂度: [简单/中等/复杂]
   └─ 分配: [Agent 列表]

🚀 启动团队...
```

### 2. Agent 启动

根据分配结果，自动更新任务文件并启动 Agent：

```
📝 更新任务看板: .claude/tasks/board.md
📝 分配 Rust-Dev: .claude/tasks/rust-dev.md
📝 分配 UI-Dev: .claude/tasks/ui-dev.md

🦀 Rust-Dev 开始工作...
🎨 UI-Dev 开始工作...
```

### 3. 自动流转

开发完成后，自动触发测试：

```
✅ Rust-Dev 完成
   └─ 自动通知: @Unit-Tester 请测试

✅ UI-Dev 完成
   └─ 自动通知: @E2E-Tester 请测试
```

测试发现问题，自动触发修复：

```
🐛 Unit-Tester 发现 Bug
   └─ 自动分配: @Debug-Dev 请修复
```

### 4. 测试通过 → 自动提交

测试通过后，**立即提交到 GitHub**：

```
✅ 测试通过
   │
   ▼
📝 生成 commit message
   │
   ▼
🔄 git add + git commit
   │
   ▼
🚀 git push origin <branch>
   │
   ▼
✅ 功能完成，已推送到 GitHub
```

**Commit 规范**：
```
<type>(<scope>): <description>

feat(knowledge): 实现 WorkQueue 模式匹配
fix(ui): 修复节点展开动画卡顿
test(analysis): 添加执行流构建测试
```

**Type 类型**：
- `feat`: 新功能
- `fix`: Bug 修复
- `test`: 测试
- `refactor`: 重构
- `style`: 样式
- `docs`: 文档

## 使用示例

### 示例 1: 后端任务

**用户输入**:
```
实现 WorkQueue 知识库模式匹配
```

**系统响应**:
```
📋 收到任务: 实现 WorkQueue 知识库模式匹配

🔍 分析结果:
   ├─ 关键词: 实现, 知识库, 模式匹配
   ├─ 类型: 后端开发
   ├─ 复杂度: 中等
   └─ 分配: Rust-Dev → Unit-Tester

🚀 启动 Rust-Dev...

🦀 Rust-Dev 开始工作:
   任务: 实现 WorkQueue 知识库模式匹配
   文件: crates/flowsight-knowledge/src/matcher.rs
```

### 示例 2: 前端任务

**用户输入**:
```
实现执行流树视图组件
```

**系统响应**:
```
📋 收到任务: 实现执行流树视图组件

🔍 分析结果:
   ├─ 关键词: 实现, 执行流, 视图, 组件
   ├─ 类型: 前端开发
   ├─ 复杂度: 中等
   └─ 分配: UI-Dev → E2E-Tester

🚀 启动 UI-Dev...

🎨 UI-Dev 开始工作:
   任务: 实现执行流树视图组件
   文件: app/src/components/FlowView/FlowTextView.tsx
```

### 示例 3: 完整功能

**用户输入**:
```
实现节点详情面板，包含后端 API 和前端组件
```

**系统响应**:
```
📋 收到任务: 实现节点详情面板

🔍 分析结果:
   ├─ 关键词: 实现, 后端, API, 前端, 组件
   ├─ 类型: 全栈开发
   ├─ 复杂度: 复杂
   └─ 分配: Rust-Dev + UI-Dev → Unit-Tester + E2E-Tester

🚀 启动团队 (并行开发)...

🦀 Rust-Dev 开始工作:
   任务: 实现节点详情 API
   文件: app/src-tauri/src/commands/node_detail.rs

🎨 UI-Dev 开始工作:
   任务: 实现节点详情面板组件
   文件: app/src/components/NodeDetail/NodeDetailPanel.tsx

🧪 Unit-Tester 待命: 等待后端完成
🖥️ E2E-Tester 待命: 等待前端完成
🔧 Debug-Dev 待命: 等待测试结果
```

## 进度追踪

任务执行过程中，可以查看进度：

```
📊 团队状态:

🦀 Rust-Dev: 开发中 - 节点详情 API (60%)
🎨 UI-Dev: 开发中 - 详情面板组件 (45%)
🧪 Unit-Tester: 待命 - 等待后端完成
🖥️ E2E-Tester: 待命 - 等待前端完成
🔧 Debug-Dev: 空闲

📋 任务看板:
├─ 开发中: 2
├─ 待测试: 0
├─ Bug 修复: 0
└─ 已完成: 0
```

## 配置

### 启用自动调度

在 CLAUDE.md 中已配置，无需额外操作。

### 自定义规则

可以在 `.claude/config/dispatch-rules.yaml` 中添加自定义规则（可选）。

---

> FlowSight 团队自动调度 - 智能任务分析，自动 Agent 分配
