# FlowSight 任务看板

> 自动更新 | 最后更新: 2026-01-29

## 当前迭代: Phase 2 - Tauri 命令集成

### 📋 待开发 (Backlog)

| ID | 任务 | 负责人 | 优先级 |
|----|------|--------|--------|
| T-014 | 本地 AI 模型下载集成 | 🦀 Rust-Dev | P3 |

### 🔄 开发中 (In Progress)

| ID | 任务 | 负责人 | 进度 | 开始时间 |
|----|------|--------|------|---------|
| - | - | - | - | - |

### 🧪 待测试 (Testing)

| ID | 任务 | 测试人 | 类型 |
|----|------|--------|------|
| - | - | - | - |

### 🐛 Bug 修复 (Fixing)

| Bug ID | 标题 | 严重度 | 负责人 |
|--------|------|--------|--------|
| - | - | - | - |

### ✅ 已完成 (Done) - Phase 1 & 2

| ID | 任务 | 完成时间 |
|----|------|---------|
| T-001 | ExecutionFlow 顶层数据结构 | 2026-01-29 |
| T-002 | 知识库模式匹配器增强 | 2026-01-29 |
| T-003 | 执行流构建器 | 2026-01-29 |
| T-004 | FlowTextView 增强 | 2026-01-29 |
| T-006 | Tauri 命令: build_execution_flow | 2026-01-29 |
| T-007 | 前端 ExecutionFlow 状态管理 | 2026-01-29 |
| T-008 | FlowView 与新 API 集成 | 2026-01-29 |
| T-009 | ARM32 知识库扩展 (irq/imx/pm) | 2026-01-29 |
| T-011 | 命令面板: 打开项目/文件功能 | 2026-01-29 |
| T-012 | ARM32 内核分析 E2E 测试 | 2026-01-29 |
| T-010 | AI 推断模块集成 (Tauri 命令) | 2026-01-29 |

---

## 统计

- 待开发: 1
- 开发中: 0
- 待测试: 0
- Bug 修复中: 0
- 已完成: 11

---

## 当前工作

### 🦀 Rust-Dev 已完成:
- ✅ T-006: Tauri 命令 build_execution_flow
- ✅ T-009: ARM32 知识库扩展
- ✅ T-010: AI 推断模块 Tauri 命令

### 🎨 UI-Dev 已完成:
- ✅ T-007: 前端 ExecutionFlow 状态管理
- ✅ T-008: FlowView 与新 API 集成
- ✅ T-011: 命令面板打开项目/文件功能

### 🧪 Unit-Tester 待命:
- 等待测试任务

### 🖥️ E2E-Tester 已完成:
- ✅ Playwright 桌面测试: 23/23 通过 (最新)
  - UI Components (4/4) ✅
  - Command Palette (5/5) ✅
  - Layout Structure (2/2) ✅
  - Color System (2/2) ✅
  - Navigation (1/1) ✅
  - Interactive Tests (2/2) ✅
  - ARM32 Kernel Analysis (5/5) ✅
  - ARM32 Kernel File Paths (2/2) ✅

### 🔧 Debug-Dev 待命:
- 无待处理 Bug

---

## 下一步工作

| ID | 任务 | 优先级 | 描述 |
|----|------|--------|------|
| T-010 | AI 推断模块集成 | P3 | 集成 AI 代码解释功能 |
| T-013 | 真实内核文件分析测试 | P2 | 在桌面应用中打开 ARM32 文件测试 |

---

## 测试内核配置

- **路径**: `/Users/sky/linux-kernel/linux`
- **优先架构**: ARM32 (`arch/arm/`)
- **优先子目录**: `arch/arm/mach-imx/`
- **测试文件**:
  - `mach-imx/mach-imx6q.c`
  - `mach-imx/clk.c`
  - `mach-imx/pm-imx6.c`
  - `mach-imx/src.c`
  - `kernel/irq.c`
