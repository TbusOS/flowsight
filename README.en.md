# 🔭 FlowSight

<p align="center">
  <img src="docs/images/logo.svg" alt="FlowSight Logo" width="180"/>
</p>

<p align="center">
  <strong>See Your Code Flow — A Cross-Platform IDE for Visualizing Execution Flow</strong>
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
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#documentation">Documentation</a> •
  <a href="#contributing">Contributing</a>
</p>

---

## 🎯 What is FlowSight?

When you're reading a large codebase like the Linux kernel (20M+ lines), existing IDEs fall short:

```c
// 😵 Traditional IDEs lose track here
INIT_WORK(&dev->work, my_handler);    // Binding
schedule_work(&dev->work);             // Trigger → ??? Who gets called?

request_irq(irq, irq_handler, ...);    // Register → ??? When executed?

static struct file_operations fops = {
    .read = my_read,                    // Assignment → ??? Who calls .read?
};
```

**FlowSight** solves this by understanding code semantics:

| Feature | Description |
|---------|-------------|
| 🔍 **Static Analysis** | No code execution needed |
| 🧠 **Semantic Understanding** | Understands async mechanisms, callbacks, function pointers |
| 📊 **Visualization** | Complete execution flow graphs |
| 🖥️ **Cross-Platform** | Windows (primary) / Linux / macOS |
| 🌐 **Internationalization** | 简体中文 + English |

---

## ✨ Features

### Core Capabilities

| Feature | Description |
|---------|-------------|
| **Execution Flow Visualization** | See how code flows through async handlers, callbacks, and function pointers |
| **Function Pointer Resolution** | Track ops tables, variable assignments, type-based matching |
| **Async Mechanism Tracking** | Work queues, timers, interrupts, tasklets, kthreads |
| **Call Graph Analysis** | Interactive call graph with filtering and search |
| **Struct Relationship View** | Visualize data structure relationships |
| **Knowledge Base Driven** | Built-in understanding of common frameworks |

### Supported Languages

| Language | Status |
|----------|--------|
| C | ✅ Full Support |
| C++ | 🚧 Planned |
| Rust | 🚧 Planned |
| Java/Kotlin (Android) | 📅 v2.0 |
| Go | 📅 Future |

### Knowledge Base Priority

| Priority | Platform | Version |
|----------|----------|---------|
| P0 | Linux Kernel | v1.0 |
| P1 | Android System | v2.0+ |
| P2 | Others | Future |

---

## 🖼️ UI Preview

> 🚧 **Under Development** - Below is the planned UI layout:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  📁 File  📝 Edit  🔍 View  📊 Analyze  ❓ Help                         │
├──────┬──────────────────────────────────────────────────┬───────────────┤
│      │                                                  │               │
│ 📁   │  ┌──────────────────────────────────────────┐   │  📋 Outline   │
│ File │  │  // usb_driver.c                          │   │  ├─ probe     │
│ Exp- │  │  static int usb_probe(struct usb_device)  │   │  ├─ disconnect│
│ lorer│  │  {                                        │   │  └─ suspend   │
│      │  │      INIT_WORK(&dev->work, handler);      │   │               │
│      │  │      ...                                  │   ├───────────────┤
│      │  └──────────────────────────────────────────┘   │  📊 Flow View │
│      │                                                  │  ┌───────────┐│
│      │  ┌──────────────────────────────────────────┐   │  │ probe     ││
│      │  │         🔗 Execution Flow View            │   │  │   ↓       ││
│      │  │    ┌─────────┐      ┌─────────┐          │   │  │ INIT_WORK ││
│      │  │    │  probe  │ ───→ │ handler │          │   │  │   ↓       ││
│      │  │    └─────────┘      └─────────┘          │   │  │ schedule  ││
│      │  └──────────────────────────────────────────┘   │  └───────────┘│
├──────┴──────────────────────────────────────────────────┴───────────────┤
│  ✅ Indexing complete: 15,234 symbols | 📊 Analysis ready               │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Installation

### Download Pre-built Binaries

> ⏳ Coming Soon

Visit the [Releases](https://github.com/TbusOS/flowsight/releases) page to download:

| Platform | File |
|----------|------|
| Windows | `flowsight-x.x.x-windows.msi` |
| Linux | `flowsight-x.x.x-linux.AppImage` or `.deb` |
| macOS | `flowsight-x.x.x-macos.dmg` |

### Build from Source

```bash
# Prerequisites
# - Rust 1.75+
# - Node.js 20+
# - pnpm

# Clone
git clone https://github.com/TbusOS/flowsight.git
cd flowsight

# Install dependencies
pnpm install

# Build and run
cargo tauri dev
```

---

## 🎮 Quick Start

### 1. Open a Project

```
File → Open Folder → Select your source code directory
```

### 2. Wait for Indexing

FlowSight will automatically index your project. For large projects like the Linux kernel, this may take a few minutes.

### 3. Explore Execution Flow

- **Right-click** on a function → "Show Execution Flow"
- **Ctrl+Click** on a function to jump to definition
- Use the **Flow View** panel to see async call chains

### 4. Understand Async Patterns

FlowSight automatically detects:
- Work queue handlers
- Timer callbacks
- Interrupt handlers
- Function pointer assignments

---

## 📖 Documentation

| Document | Description |
|----------|-------------|
| [Project Plan](docs/design/PROJECT-PLAN.md) | Complete project plan and roadmap |
| [User Guide](docs/user-guide/README.md) | Usage tutorials (WIP) |
| [Developer Guide](docs/developer/README.md) | Development and contribution guide |
| [Architecture](docs/architecture/README.md) | Technical architecture docs |
| [i18n](docs/developer/I18N.md) | Internationalization and translation |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         FlowSight                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Tauri Desktop App (React + TypeScript + Monaco)          │  │
│  │  Languages: 简体中文 | English                             │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Rust Analysis Engine                                      │  │
│  │  ├── flowsight-parser    (tree-sitter + libclang)         │  │
│  │  ├── flowsight-analysis  (async tracking, func ptr)       │  │
│  │  ├── flowsight-index     (symbol table, call graph)       │  │
│  │  └── flowsight-knowledge (pattern matching)               │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Storage: SQLite (symbols) + sled (graphs)                │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repo
git clone https://github.com/TbusOS/flowsight.git
cd flowsight

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js dependencies
pnpm install

# Run in development mode
cargo tauri dev

# Run tests
cargo test --workspace
```

### Areas for Contribution

| Type | Description |
|------|-------------|
| 🐛 Bug Fixes | Fix known issues |
| 📚 Documentation | Improve docs and translations |
| 🔧 Parsers | Add new language support |
| ✨ UI/UX | Interface improvements |
| 🧪 Testing | Increase test coverage |
| 🌐 Translation | Add new language packs |

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- [tree-sitter](https://tree-sitter.github.io/) - Incremental parsing
- [Tauri](https://tauri.app/) - Desktop app framework
- [Monaco Editor](https://microsoft.github.io/monaco-editor/) - Code editor
- Linux Kernel community - Inspiration for understanding complex codebases

---

<p align="center">
  Made with ❤️ for developers who want to truly understand their code
</p>

