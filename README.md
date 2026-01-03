# 🔭 FlowSight

<p align="center">
  <strong>看见代码的"灵魂" — 跨平台执行流可视化 IDE</strong>
</p>

<p align="center">
  <a href="README.md">中文</a> | <a href="README.en.md">English</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"/>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg" alt="Platform"/>
  <img src="https://img.shields.io/badge/rust-1.75+-orange.svg" alt="Rust"/>
  <img src="https://img.shields.io/badge/i18n-简体中文%20%7C%20English-green.svg" alt="Languages"/>
</p>

<p align="center">
  <a href="#特性">特性</a> •
  <a href="#安装">安装</a> •
  <a href="#快速开始">快速开始</a> •
  <a href="#文档">文档</a> •
  <a href="#贡献">贡献</a>
</p>

---

## 🎯 FlowSight 是什么？

当你阅读像 Linux 内核这样的超大型代码库（2000万+ 行）时，现有 IDE 都会"迷路"：

```c
// 😵 传统 IDE 在这里就断了
INIT_WORK(&dev->work, my_handler);    // 绑定
schedule_work(&dev->work);             // 触发 → ??? 谁被调用？

request_irq(irq, irq_handler, ...);    // 注册 → ??? 何时执行？

static struct file_operations fops = {
    .read = my_read,                    // 赋值 → ??? 谁调用 .read？
};
```

**FlowSight** 通过理解代码语义来解决这个问题：

| 特性 | 描述 |
|------|------|
| 🔍 **静态分析** | 不需要运行代码，纯代码阅读 |
| 🧠 **语义理解** | 理解异步机制、回调模式、函数指针 |
| 📊 **可视化** | 完整的执行流程图 |
| 🖥️ **跨平台** | Windows (首选) / Linux / macOS |
| 🌐 **多语言界面** | 简体中文 + English |

---

## ✨ 特性

### 核心能力

| 功能 | 描述 |
|------|------|
| **执行流可视化** | 查看代码如何通过异步处理程序、回调和函数指针流动 |
| **函数指针解析** | 追踪 ops 表、变量赋值、基于类型的匹配 |
| **异步机制追踪** | 工作队列、定时器、中断、tasklet、kthreads |
| **调用图分析** | 交互式调用图，支持过滤和搜索 |
| **结构体关系图** | 可视化数据结构之间的关系 |
| **知识库驱动** | 内置对常见框架的理解 |

### 语言支持

| 语言 | 状态 |
|------|------|
| C | ✅ 完整支持 |
| C++ | 🚧 计划中 |
| Rust | 🚧 计划中 |
| Java/Kotlin (Android) | 📅 v2.0 |
| Go | 📅 未来 |

### 知识库优先级

| 优先级 | 平台 | 版本 |
|--------|------|------|
| P0 | Linux Kernel | v1.0 |
| P1 | Android System | v2.0+ |
| P2 | 其他平台 | 未来 |

---

## 🖼️ 界面预览

> 🚧 **开发中** - 以下是计划的界面布局：

```
┌─────────────────────────────────────────────────────────────────────────┐
│  📁 文件  📝 编辑  🔍 视图  📊 分析  ❓ 帮助                            │
├──────┬──────────────────────────────────────────────────┬───────────────┤
│      │                                                  │               │
│ 📁   │  ┌──────────────────────────────────────────┐   │  📋 大纲      │
│ 文件 │  │  // usb_driver.c                          │   │  ├─ probe     │
│ 浏览 │  │  static int usb_probe(struct usb_device)  │   │  ├─ disconnect│
│ 器   │  │  {                                        │   │  └─ suspend   │
│      │  │      INIT_WORK(&dev->work, handler);      │   │               │
│      │  │      ...                                  │   ├───────────────┤
│      │  └──────────────────────────────────────────┘   │  📊 执行流    │
│      │                                                  │  ┌───────────┐│
│      │  ┌──────────────────────────────────────────┐   │  │ probe     ││
│      │  │         🔗 执行流视图                     │   │  │   ↓       ││
│      │  │    ┌─────────┐      ┌─────────┐          │   │  │ INIT_WORK ││
│      │  │    │  probe  │ ───→ │ handler │          │   │  │   ↓       ││
│      │  │    └─────────┘      └─────────┘          │   │  │ schedule  ││
│      │  └──────────────────────────────────────────┘   │  └───────────┘│
├──────┴──────────────────────────────────────────────────┴───────────────┤
│  ✅ 索引完成: 15,234 符号 | 📊 分析就绪                                  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 安装

### 下载预编译版本

> ⏳ 即将发布

访问 [Releases](https://github.com/TbusOS/flowsight/releases) 页面下载：

| 平台 | 文件 |
|------|------|
| Windows | `flowsight-x.x.x-windows.msi` |
| Linux | `flowsight-x.x.x-linux.AppImage` 或 `.deb` |
| macOS | `flowsight-x.x.x-macos.dmg` |

### 从源码构建

```bash
# 前置条件
# - Rust 1.75+
# - Node.js 20+
# - pnpm

# 克隆仓库
git clone https://github.com/TbusOS/flowsight.git
cd flowsight

# 安装依赖
pnpm install

# 开发模式运行
cargo tauri dev
```

---

## 🎮 快速开始

### 1. 打开项目

```
文件 → 打开文件夹 → 选择源代码目录
```

### 2. 等待索引

FlowSight 会自动索引项目。对于 Linux 内核这样的大型项目，可能需要几分钟。

### 3. 探索执行流

- **右键点击** 函数 → "显示执行流"
- **Ctrl+Click** 跳转到定义
- 使用 **执行流视图** 面板查看异步调用链

### 4. 理解异步模式

FlowSight 自动检测：
- 工作队列处理函数
- 定时器回调
- 中断处理程序
- 函数指针赋值

---

## 📖 文档

| 文档 | 描述 |
|------|------|
| [项目计划](docs/design/PROJECT-PLAN.md) | 完整的项目规划和路线图 |
| [用户指南](docs/user-guide/README.md) | 使用教程 (开发中) |
| [开发者指南](docs/developer/README.md) | 开发与贡献指南 |
| [架构设计](docs/architecture/README.md) | 技术架构文档 |
| [国际化](docs/developer/I18N.md) | 多语言支持与翻译贡献 |

---

## 🏗️ 技术架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         FlowSight                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Tauri 桌面应用 (React + TypeScript + Monaco)            │  │
│  │  支持语言: 简体中文 | English                             │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Rust 分析引擎                                            │  │
│  │  ├── flowsight-parser    (tree-sitter + libclang)         │  │
│  │  ├── flowsight-analysis  (异步追踪, 函数指针解析)         │  │
│  │  ├── flowsight-index     (符号表, 调用图)                 │  │
│  │  └── flowsight-knowledge (模式匹配, 知识库)               │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  存储: SQLite (符号) + sled (图)                          │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🤝 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

### 开发环境

```bash
# 克隆仓库
git clone https://github.com/TbusOS/flowsight.git
cd flowsight

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装前端依赖
pnpm install

# 开发模式运行
cargo tauri dev

# 运行测试
cargo test --workspace
```

### 贡献方向

| 类型 | 描述 |
|------|------|
| 🐛 Bug 修复 | 修复已知问题 |
| 📚 文档 | 改进文档和翻译 |
| 🔧 解析器 | 添加新语言支持 |
| ✨ UI/UX | 界面改进 |
| 🧪 测试 | 增加测试覆盖 |
| 🌐 翻译 | 添加新语言包 |

---

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE)

---

## 🙏 致谢

- [tree-sitter](https://tree-sitter.github.io/) - 增量解析
- [Tauri](https://tauri.app/) - 桌面应用框架
- [Monaco Editor](https://microsoft.github.io/monaco-editor/) - 代码编辑器
- Linux Kernel 社区 - 理解复杂代码库的灵感来源

---

<p align="center">
  用 ❤️ 为想要真正理解代码的开发者打造
</p>
