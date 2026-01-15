# FlowSight 改进计划

> 本文档记录从 OpenCode 项目借鉴的改进计划

## 背景

参考 [OpenCode](https://github.com/anomalyco/opencode) 项目的最佳实践，为 FlowSight 添加以下功能：

## 1. Agent 系统

### 概述
添加专门的 Agent 用于代码分析、解释和搜索。

### Agent 类型

| Agent | 模式 | 功能 |
|-------|------|------|
| `analyze` | 只读 | 执行流分析、生成报告 |
| `explain` | 只读 | 解释代码逻辑、自然语言描述 |
| `search` | 只读 | 语义搜索代码模式 |

### 工具权限控制

```json
{
  "tools": {
    "read_file": true,
    "search_symbols": true,
    "analyze_code": true,
    "write_file": false,
    "run_commands": false
  }
}
```

### 文件变更

- `.claude/agents/analyze.md` - 分析 Agent
- `.claude/agents/explain.md` - 解释 Agent
- `.claude/agents/search.md` - 搜索 Agent
- `.claude/opencode.jsonc` - Agent 配置

---

## 2. 安装脚本

### 目标
添加一键安装脚本，简化用户安装流程。

### 功能

- 支持 `curl | bash` 一键安装
- 自动检测平台（Linux/macOS/Windows）
- 支持 Homebrew、Scoop、apt 等包管理器
- 自动添加到 PATH

### 文件变更

- `install` - 主安装脚本（Bash）
- `install.ps1` - Windows PowerShell 版本

---

## 3. 配置系统增强

### 配置项

```jsonc
{
  "agent": {
    "analyze": {
      "model": "claude-sonnet-4",
      "tools": ["read_file", "search_symbols", "analyze_code"]
    }
  },
  "mcp": {
    "context7": {
      "type": "remote",
      "url": "https://mcp.context7.com/mcp"
    }
  },
  "knowledge": {
    "linux_kernel_version": "6.1",
    "enabled_patterns": ["usb", "pci", "netdev"]
  }
}
```

### 文件变更

- `.claude/opencode.jsonc` - 主配置
- `.claude/env.d.ts` - 类型定义

---

## 4. UI/UX 改进

### 改进项

| 功能 | 当前状态 | 改进方向 |
|------|----------|----------|
| 命令面板 | 已有 | 增强，支持 Agent 命令 |
| 快速打开 | 已有 | 支持搜索函数、符号 |
| 键盘快捷键 | 基础 | 添加更多快捷操作 |
| 暗色主题 | 基础 | 参考 OpenCode 的主题设计 |

### 文件变更

- `app/src/components/CommandPalette/CommandPalette.tsx`
- `app/src/components/QuickOpen/QuickOpen.tsx`
- `app/src/styles/` - 主题样式

---

## 5. 文档和规范

### 文件变更

- `STYLE_GUIDE.md` - 代码风格指南
- `AGENTS.md` - Agent 说明文档
- 更新 `CONTRIBUTING.md`

---

## 实施计划

| Phase | 内容 | 状态 |
|-------|------|------|
| 1 | Agent 系统 | 待实施 |
| 2 | 安装脚本 | 待实施 |
| 3 | 配置系统 | 待实施 |
| 4 | UI/UX 改进 | 待实施 |
| 5 | 文档规范 | 待实施 |

---

## 验证方式

1. 运行 `cargo test --workspace` 验证后端测试
2. 运行 `pnpm test` 验证前端测试
3. 手动测试 Agent 功能
4. 测试安装脚本
5. 检查 UI 组件渲染

---

## 参考

- [OpenCode GitHub](https://github.com/anomalyco/opencode)
- [OpenCode 配置格式](https://opencode.ai/config.json)
