# 用户使用指南

本目录包含 FlowSight 的用户使用文档。

## 📄 文档列表

> 🚧 文档编写中...

## 📋 计划文档

| 文档 | 描述 | 状态 |
|------|------|------|
| INSTALLATION.md | 安装指南（Windows/Linux/macOS） | 待编写 |
| QUICK-START.md | 快速开始教程 | 待编写 |
| CREATE-PROJECT.md | 创建和配置项目 | 待编写 |
| NAVIGATION.md | 代码导航与执行流查看 | 待编写 |
| CALL-GRAPH.md | 调用关系图使用 | 待编写 |
| ASYNC-FLOW.md | 异步执行流分析 | 待编写 |
| KEYBOARD-SHORTCUTS.md | 键盘快捷键 | 待编写 |
| FAQ.md | 常见问题解答 | 待编写 |

## 🎯 目标用户

- **Linux 内核开发者** - 分析驱动代码、理解异步机制
- **嵌入式开发者** - 追踪硬件回调、中断处理流程
- **Android 系统开发者** - 分析 Framework、Native 层代码
- **C/C++ 项目维护者** - 理解大型遗留代码库

## 💡 快速入门（预览）

```bash
# 1. 下载安装包
# Windows: flowsight-x.x.x-windows.msi
# Linux:   flowsight-x.x.x-linux.AppImage
# macOS:   flowsight-x.x.x-macos.dmg

# 2. 启动 FlowSight

# 3. 创建项目
#    File → New Project → 选择 Linux Kernel 项目类型

# 4. 导入源代码
#    指向 Linux 源码目录，自动检测 compile_commands.json

# 5. 开始分析
#    右键函数 → Show Execution Flow
```

