# Changelog

All notable changes to FlowSight will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-02-04

### Added

#### Documentation
- **快速开始指南**: 5 分钟上手教程 (`docs/user-guide/quick-start.md`)
- **功能说明文档**: 所有功能详细介绍 (`docs/user-guide/features.md`)
- **异步机制分析指南**: WorkQueue、Timer、IRQ 深入分析 (`docs/user-guide/async-analysis.md`)
- **知识库说明文档**: 内置知识库使用指南 (`docs/user-guide/knowledge-base.md`)
- **用户指南索引**: 更新用户指南目录结构

### Changed

- 更新文档中心索引，添加用户指南快速导航

---

## [0.2.0] - 2026-02-01

### Added

#### Features
- **执行流导出面板**: 支持 Mermaid、表格、文本、AI 格式化导出
- **AI 格式化模块**: FlowFormatter Rust 模块，支持多种输出格式
- **主题系统**: 6 个低饱和度主题 (Dark, Light, Slate, Forest, Sunset, Lavender)
- **LLVM IR 面板**: 查看函数的 LLVM IR 中间表示
- **响应式设计**: 移动端和桌面端自适应布局

#### Testing & CI
- **DeskPilot 测试框架**: 完整的桌面自动化测试框架
- **VLM 视觉测试**: AI 驱动的视觉断言和 UI 检查
- **CI 工作流**: GitHub Actions 自动化测试和 PR 视觉检查
- **12 Agent 协作系统**: 完整的多智能体开发团队

#### Knowledge Base
- **ARM32 知识库扩展**: IRQ、IMX、PM 子系统支持

### Changed

- 优化索引并发性能 (Mutex → RwLock)
- 统一图标系统 (Emoji → Lucide React)
- 改进 UI 间距和交互细节
- 重组文档结构

### Fixed

- 修复打开项目后显示 "0 文件" 的 Bug
- 修复 LLVM IR clang 头文件路径问题
- 修复按钮触摸目标不符合 WCAG 标准的问题
- 修复终端面板状态显示问题

### Security

- 更新 happy-dom 修复 RCE 漏洞 (Critical)
- 更新 vite 修复请求劫持漏洞 (Moderate)
- 更新 vitest 修复 esbuild 依赖漏洞
- 添加 lodash 版本覆盖修复原型污染漏洞

## [0.1.0] - 2026-01-15

### Added

- 初始版本发布
- 基础代码解析功能 (Tree-sitter)
- 执行流可视化 (@xyflow/react)
- Monaco 代码编辑器集成
- 符号索引和搜索
- 知识库系统 (YAML 配置)
- Tauri 桌面应用框架
- 命令面板和快捷键支持
