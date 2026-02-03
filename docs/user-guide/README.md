# FlowSight 用户指南

欢迎使用 FlowSight！本目录包含完整的用户文档，帮助您快速上手并深入使用。

## 📚 文档列表

### 入门指南

| 文档 | 描述 | 阅读时间 |
|------|------|----------|
| [快速开始](./quick-start.md) | 5 分钟上手 FlowSight | 5 分钟 |
| [功能说明](./features.md) | 所有功能详细介绍 | 15 分钟 |
| [异步机制分析](./async-analysis.md) | WorkQueue、Timer、IRQ 分析指南 | 20 分钟 |

### 进阶指南

| 文档 | 描述 | 状态 |
|------|------|------|
| [知识库说明](./knowledge-base.md) | 内置知识库使用指南 | 📝 编写中 |
| [快捷键参考](./keyboard-shortcuts.md) | 完整快捷键列表 | 📝 编写中 |
| [导出格式](./export-formats.md) | 各种导出格式说明 | 📝 编写中 |

## 🎯 目标用户

FlowSight 专为以下开发者设计：

| 用户类型 | 典型场景 |
|----------|----------|
| **Linux 内核开发者** | 分析驱动代码、理解异步机制 |
| **嵌入式开发者** | 追踪硬件回调、中断处理流程 |
| **Android 系统开发者** | 分析 Framework、Native 层代码 |
| **C/C++ 项目维护者** | 理解大型遗留代码库 |

## 🚀 快速入门

```bash
# 1. 下载安装
# 访问 https://github.com/TbusOS/flowsight/releases

# 2. 启动 FlowSight

# 3. 打开项目
#    Cmd+K → 打开项目 → 选择 Linux 内核源码目录

# 4. 开始分析
#    打开 .c 文件 → 点击函数 → 查看执行流
```

## ⌨️ 常用快捷键

| 快捷键 | 功能 |
|--------|------|
| `Cmd/Ctrl+K` | 打开命令面板 |
| `Cmd/Ctrl+P` | 快速打开文件 |
| `F12` | 跳转到定义 |
| `Alt+←` / `Alt+→` | 后退/前进导航 |

完整列表：[快捷键参考](./keyboard-shortcuts.md)

## 📊 执行流颜色含义

| 颜色 | 上下文 | 可睡眠 |
|------|--------|--------|
| 🟢 绿色 | 进程上下文 | ✅ |
| 🔵 蓝色 | 工作队列 | ✅ |
| 🟠 橙色 | 软中断 | ❌ |
| 🔴 红色 | 硬中断 | ❌ |
| 🟣 紫色 | 定时器 | ❌ |

## 📖 其他资源

| 资源 | 链接 |
|------|------|
| API 参考 | [Tauri Commands](../api/tauri-commands.md) |
| 架构设计 | [Architecture](../architecture/README.md) |
| 开发者指南 | [Developer Guide](../developer/README.md) |
| 变更日志 | [CHANGELOG](../../CHANGELOG.md) |

## 🐛 反馈与支持

- **问题报告**: [GitHub Issues](https://github.com/TbusOS/flowsight/issues)
- **功能建议**: [GitHub Discussions](https://github.com/TbusOS/flowsight/discussions)
- **贡献代码**: [Contributing Guide](../../CONTRIBUTING.md)

---

**FlowSight 版本**: 0.2.0  
**文档更新**: 2026-02-04
