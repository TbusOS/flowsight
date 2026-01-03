# 🔭 FlowSight

<p align="center">
  <img src="docs/images/logo.svg" alt="FlowSight Logo" width="200"/>
</p>

<p align="center">
  <strong>See Your Code Flow — A Cross-Platform IDE for Visualizing Execution Flow</strong>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#documentation">Documentation</a> •
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"/>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg" alt="Platform"/>
  <img src="https://img.shields.io/badge/rust-1.75+-orange.svg" alt="Rust"/>
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

- 🔍 **Static Analysis** — No code execution needed
- 🧠 **Semantic Understanding** — Understands async mechanisms, callbacks, function pointers
- 📊 **Visualization** — See complete execution flow graphs
- 🖥️ **Cross-Platform** — Windows / Linux / macOS

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
| **Knowledge Base** | Built-in understanding of common frameworks |

### Supported Languages

| Language | Status |
|----------|--------|
| C | ✅ Full Support |
| C++ | 🚧 Planned |
| Rust | 🚧 Planned |
| Java | 📅 Future |
| Go | 📅 Future |

---

## 📸 Screenshots

<p align="center">
  <img src="docs/images/flow-view.png" alt="Execution Flow View" width="80%"/>
  <br>
  <em>Execution Flow Visualization</em>
</p>

<p align="center">
  <img src="docs/images/struct-view.png" alt="Struct Relationship View" width="80%"/>
  <br>
  <em>Struct Relationship Graph</em>
</p>

---

## 🚀 Installation

### Download Pre-built Binaries

Visit the [Releases](https://github.com/user/flowsight/releases) page to download:

- **Windows**: `flowsight-x.x.x-windows.msi`
- **Linux**: `flowsight-x.x.x-linux.AppImage` or `.deb`
- **macOS**: `flowsight-x.x.x-macos.dmg`

### Build from Source

```bash
# Prerequisites
# - Rust 1.75+
# - Node.js 20+
# - pnpm

# Clone
git clone https://github.com/user/flowsight.git
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

- [User Guide](docs/user-guide/README.md)
- [Developer Guide](docs/developer/README.md)
- [API Reference](docs/api/README.md)
- [Project Plan](docs/PROJECT-PLAN.md)

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         FlowSight                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Tauri Desktop App (React + TypeScript + Monaco)          │  │
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
git clone https://github.com/user/flowsight.git
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

- 🐛 Bug fixes
- 📚 Documentation
- 🔧 New language parsers
- ✨ UI/UX improvements
- 🧪 Test coverage

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

