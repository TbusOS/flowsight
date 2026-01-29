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

#### 测试反馈规则 (强制执行)

**E2E-Tester 和 Unit-Tester 必须遵守以下规则：**

1. **测试覆盖不足时**：
   - 必须向 UI-Dev 或 Rust-Dev 提出需求，要求开发新的测试工具/功能
   - 在 `.claude/tasks/board.md` 中创建测试工具需求任务
   - 示例：`📋 需求: E2E-Tester 无法测试原生对话框，需要 UI-Dev 实现测试模式`

2. **发现 Bug 时**：
   - 立即创建 Bug 报告并分配给 Debug-Dev
   - 提供：复现步骤、截图、日志、预期行为
   - Debug-Dev 修复后必须请求重测

3. **测试不到的功能**：
   - 记录到 `.claude/reports/test-gaps.md`
   - 分析原因（缺少工具？权限问题？环境问题？）
   - 向相关开发 Agent 提出解决方案

4. **自动执行原则**：
   - 每次代码修改后，相关 Tester 自动运行测试
   - 不需要用户提醒，根据修改内容自动选择测试方式
   - MCP Browser 测试纯 Web UI，桌面自动化测试 Tauri 原生功能

5. **测试工具不足时的处理**：
   ```
   E2E-Tester 发现问题 → 分析原因 → 提出工具需求 → UI-Dev/Rust-Dev 开发 → 重测
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

## 测试内核配置 (强制)

**重要**: 所有功能测试必须使用以下 Linux 内核源码，无需用户每次提醒。

| 配置项 | 值 |
|--------|-----|
| **内核路径** | `/Users/sky/linux-kernel/linux` |
| **优先架构** | ARM32 (`arch/arm/`) |
| **测试目录** | `arch/arm/mach-imx/` (142 个文件) |
| **备选目录** | `drivers/usb/`, `drivers/net/` |

### 测试用例选择优先级

1. **ARM32 平台代码**: `arch/arm/mach-*/` 
2. **IMX SoC**: `arch/arm/mach-imx/` (推荐首选)
3. **USB 驱动**: `drivers/usb/`
4. **网络驱动**: `drivers/net/`

### 测试命令示例

```bash
# IDE 测试 - 打开 ARM32 内核代码
pnpm tauri dev
# 然后打开项目: /Users/sky/linux-kernel/linux

# 单元测试 - 使用内核文件
cargo test --package flowsight-analysis -- --test-threads=1

# 分析测试文件
/Users/sky/linux-kernel/linux/arch/arm/mach-imx/clk-imx6q.c
/Users/sky/linux-kernel/linux/arch/arm/mach-imx/pm-imx6.c
/Users/sky/linux-kernel/linux/drivers/usb/gadget/udc/fsl_udc_core.c
```

### 任务感知

当检测到以下关键词时，自动使用测试内核:
- "测试", "验证", "功能测试"
- "分析内核", "解析驱动"
- "ARM", "arm32", "IMX"

## UI 测试规则 (自动执行)

**重要**: 开发完成后必须自动执行测试，无需用户提醒。根据任务类型自动选择测试方式。

### 自动测试选择策略

| 任务类型 | 测试方式 | 原因 |
|----------|----------|------|
| 纯 Web UI 修改 | MCP Browser | 快速验证 DOM/样式 |
| Tauri 原生功能 | Playwright 桌面测试 | 需要完整桌面环境 |
| 文件对话框/系统集成 | Playwright 桌面测试 | 原生 API 测试 |
| 快捷键/命令面板 | Playwright 桌面测试 | 键盘事件测试 |
| 复杂交互流程 | Playwright + MCP 组合 | 全面覆盖 |

### 自动测试触发规则

**开发完成后自动执行：**
1. 修改 `app/src/components/` → 运行 Playwright UI 测试
2. 修改 `app/src-tauri/` → 运行功能测试
3. 修改命令面板 → 测试 ⌘K 打开和命令执行
4. 修改样式/布局 → MCP Browser 快速验证 + Playwright 截图

**自动测试命令：**
```bash
# 快速测试（MCP Browser 不可用时）
cd app && npx playwright test tests/desktop/playwright.spec.ts --config=tests/desktop/playwright.config.ts --reporter=list

# 完整测试
cd app && npx playwright test tests/desktop/ --config=tests/desktop/playwright.config.ts
```

### 工具不足时的处理

如果现有测试工具无法覆盖测试场景，需要：
1. 在 `app/tests/desktop/playwright.spec.ts` 中添加新测试用例
2. 或扩展 `app/tests/desktop/` 框架功能
3. 确保新增测试可复用

---

### MCP Browser 自动化测试 (Web UI 快速验证)

**重要**: 本项目优先使用 Cursor MCP Browser 进行 UI 测试，可直接在开发过程中自动验证功能。

#### MCP Browser 测试流程

```bash
# 1. 启动应用 (后台运行)
cd app && pnpm tauri dev

# 2. 使用 MCP Browser 工具测试
# - browser_navigate: 导航到 http://localhost:5173
# - browser_snapshot: 获取页面快照
# - browser_click: 点击元素
# - browser_fill: 填充输入框
# - browser_press_key: 按键操作
# - browser_take_screenshot: 截图保存
```

#### 常用测试操作

| 操作 | MCP 工具 | 示例 |
|------|----------|------|
| 打开应用 | `browser_navigate` | `{"url": "http://localhost:5173"}` |
| 获取状态 | `browser_snapshot` | `{}` |
| 点击按钮 | `browser_click` | `{"ref": "e1"}` |
| 输入文本 | `browser_fill` | `{"ref": "e40", "value": "test"}` |
| 按键 | `browser_press_key` | `{"key": "Escape"}` |
| 截图 | `browser_take_screenshot` | `{}` |

#### 测试 ARM32 内核文件分析

**MCP Browser 限制**: 由于 MCP Browser 在纯浏览器环境运行，无法调用 Tauri 原生 API（如文件对话框）。以下为测试策略：

**可用 MCP Browser 测试的功能**:
- UI 布局和样式
- 命令面板搜索和导航
- 键盘快捷键
- 视图切换
- 状态管理响应

**需要手动测试的功能**:
- 打开项目/文件（原生对话框）
- 文件保存
- 桌面窗口行为

**自动化测试流程**:
```
1. browser_navigate → http://localhost:5173
2. browser_snapshot → 获取 UI 状态
3. browser_click → 点击命令面板按钮 (⌘K)
4. browser_snapshot → 验证命令菜单项（打开项目、打开文件等）
5. browser_press_key → 测试键盘导航
6. browser_snapshot → 验证状态变化
```

**手动测试步骤（Tauri 桌面应用）**:
```
1. 启动: pnpm tauri dev
2. 按 ⌘K 打开命令面板
3. 选择"打开项目"
4. 选择 /Users/sky/linux-kernel/linux/arch/arm/mach-imx
5. 等待索引完成
6. 打开 .c 文件验证分析功能
```

#### 任务感知

当检测到以下关键词时，自动使用 MCP Browser 测试:
- "E2E 测试", "UI 测试", "界面测试"
- "验证功能", "功能测试"
- "浏览器测试", "自动化测试"

### Playwright 桌面自动化测试 (推荐)

测试框架位置: `app/tests/desktop/`

#### 运行 Playwright 测试（首选方案）

```bash
# 确保应用已启动
cd app && pnpm tauri dev

# 运行桌面 UI 测试
cd app && npx playwright test tests/desktop/playwright.spec.ts --config=tests/desktop/playwright.config.ts --reporter=list

# 查看截图结果
ls app/test-results/flowsight/
```

#### 测试用例

| 测试组 | 测试内容 | 状态 |
|--------|----------|------|
| UI Components | Header, Sidebar, Main, Footer | ✅ 通过 |
| Command Palette | Cmd+K 打开, 搜索输入框 | ✅ 通过 |
| Layout Structure | 完整布局截图, Flexbox | ✅ 通过 |
| Color System | 背景色, 文字色 | ⚠️ 部分通过 |
| Navigation | 侧边栏按钮 | ⚠️ Hover 超时 |

#### 添加新测试

在 `app/tests/desktop/playwright.spec.ts` 中添加：

```typescript
test('my new feature test', async ({ page }) => {
  await page.goto('http://localhost:5173/', { waitUntil: 'networkidle' });
  
  // 测试打开项目命令
  await page.keyboard.press('Meta+K');
  await page.waitForTimeout(500);
  const openProjectOption = page.locator('text=打开项目');
  await expect(openProjectOption).toBeVisible();
  
  await page.screenshot({ path: 'test-results/flowsight/my-test.png' });
});
```

### Python 桌面自动化测试 (备选)

#### 何时使用

| 场景 | 测试命令 |
|------|----------|
| 修复 UI Bug 后验证 | `python3 -m tests.desktop --smoke` |
| UI 样式修改后验证 | `python3 -m tests.desktop --phase visual` |
| 交互功能修改后验证 | `python3 -m tests.desktop --phase interactive` |
| 完整 UI 测试 | `python3 -m tests.desktop --full` |

**macOS 依赖**: `pip3 install pyobjc-framework-Quartz Pillow`

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
