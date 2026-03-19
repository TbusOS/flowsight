# FlowSight Roadmap

> Last updated: 2026-03-19

FlowSight is a static execution flow analyzer for Linux kernel code. This document tracks completed milestones and planned features.

## Current Status: v0.3.0 (CLI Feature-Complete)

17 commands, 310 tests, 6 output formats, 10 kernel scenarios, SQLite cross-file index, LLM training data pipeline.

---

## Completed

### v0.1.0 — CLI Foundation (2026-03-13)

- [x] Modular CLI with clap subcommands
- [x] `analyze` - single file static analysis
- [x] `flow` - execution flow tree tracing
- [x] `trace` - ftrace-style output
- [x] `callers` / `callees` - call graph queries
- [x] `async` / `callbacks` - async handler & callback detection
- [x] Tree-sitter C parser with function pointer resolution
- [x] Text, JSON, ftrace output formats

### v0.2.0 — Interactive & Knowledge Base (2026-03-18)

- [x] Interactive REPL with history, tab completion, ASCII logo
- [x] ASCII sequence diagram output format
- [x] Knowledge base: 865 frameworks, 696 callbacks, 11 call chains
- [x] `kb` command suite (stats, query, chain, async-chain, match)
- [x] `--depth`, `--no-kernel`, `--expand-async` flow options
- [x] CJK character alignment in sequence diagrams
- [x] Low-saturation color palette

### v0.3.0 — Production-Ready (2026-03-19)

- [x] `analyze -r` — recursive directory analysis with parallel processing (rayon)
- [x] `index build/query/stats/update` — SQLite cross-file index with subsystem detection
- [x] `patterns` — kernel pattern detection (locking, error handling, lifecycle, async, memory, RCU)
- [x] `diff` — execution flow comparison between file versions (LCS algorithm)
- [x] `scenario list/run` — 10 built-in kernel scenarios (USB, IRQ, MM, Net, FS, Sched, etc.)
- [x] `train generate/stats` — LLM training data pipeline (SFT, DPO, ChatML JSONL)
- [x] `search` — symbol search (direct scan + SQLite indexed mode, regex support)
- [x] `report` — self-contained HTML report (dark theme, interactive tables, filtering)
- [x] `graph` — full file call graph with DOT/Graphviz export
- [x] `config` — `.flowsight.toml` project configuration
- [x] Shell completions (bash, zsh, fish)
- [x] GitHub Actions CI/CD (4 platforms)
- [x] 54 CLI integration tests
- [x] `#![forbid(unsafe_code)]` on all crates
- [x] Security audit & hardening
- [x] README, LICENSE (MIT), CHANGELOG, install.sh, Makefile

---

## Planned

### v0.4.0 — Cross-File Intelligence

- [ ] Cross-file callers/callees using index (resolve calls across entire kernel tree)
- [ ] `flowsight trace` with cross-file expansion
- [ ] Subsystem dependency graph generation
- [ ] Call chain path finding (`graph path --from A --to B`)
- [ ] Index-backed pattern detection across directories
- [ ] Performance optimization for 30,000+ file kernel trees

### v0.5.0 — Language Expansion

- [ ] C++ source file support (classes, namespaces, templates)
- [ ] Rust source file support (traits, impl blocks, async/await)
- [ ] Header file analysis (`.h` function declarations, macro expansion)
- [ ] Preprocessor-aware analysis (conditional compilation, `#ifdef`)

### v0.6.0 — Developer Experience

- [ ] `flowsight watch` — file system monitoring with incremental re-analysis
- [ ] LSP server mode for IDE integration (VS Code, Neovim)
- [ ] Web-based interactive report viewer (local server)
- [ ] Man page generation
- [ ] Structured logging with `tracing` instrumentation
- [ ] Plugin system for custom analyzers

### v0.7.0 — AI Integration

- [ ] `flowsight explain <file> <function>` — AI-powered code explanation
- [ ] `flowsight suggest` — pattern-based improvement suggestions
- [ ] Training data quality scoring and filtering
- [ ] Fine-tuned kernel expert model integration
- [ ] DPO feedback collection from user corrections

### v1.0.0 — Stable Release

- [ ] Stable public API for all commands
- [ ] Comprehensive documentation site
- [ ] Binary releases on crates.io
- [ ] Docker image for CI/CD integration
- [ ] Benchmark suite with reproducible performance results
- [ ] 90%+ test coverage

---

## Architecture

```
flowsight/
├── cli/              # CLI tool (current focus)
│   ├── src/
│   │   ├── main.rs           # Command dispatcher + config loading
│   │   ├── config.rs         # .flowsight.toml support
│   │   ├── context.rs        # Shared AnalysisContext
│   │   ├── index_db.rs       # SQLite index layer
│   │   ├── repl.rs           # Interactive REPL
│   │   ├── commands/         # One file per command
│   │   │   ├── analyze.rs    # File/directory analysis
│   │   │   ├── flow.rs       # Execution flow + ftrace
│   │   │   ├── graph.rs      # Call graph + DOT
│   │   │   ├── diff.rs       # Flow comparison
│   │   │   ├── patterns.rs   # Kernel pattern detection
│   │   │   ├── search.rs     # Symbol search
│   │   │   ├── index.rs      # Cross-file index
│   │   │   ├── scenario.rs   # Kernel scenarios
│   │   │   ├── train/        # Training data pipeline
│   │   │   ├── report.rs     # HTML report
│   │   │   ├── kb.rs         # Knowledge base
│   │   │   └── config.rs     # Config management
│   │   └── output/           # Output formatters
│   │       ├── text.rs       # Human-readable
│   │       ├── json.rs       # JSON
│   │       ├── dot.rs        # Graphviz DOT
│   │       └── sequence.rs   # ASCII sequence diagrams
│   └── tests/
│       ├── integration_test.rs
│       └── fixtures/         # Test kernel driver
├── crates/                   # Shared analysis engine
│   ├── flowsight-core/       # Core types
│   ├── flowsight-parser/     # Tree-sitter C parser
│   ├── flowsight-analysis/   # Static analysis engine
│   ├── flowsight-knowledge/  # Knowledge base (137 YAML)
│   ├── flowsight-index/      # Symbol indexing
│   └── ...
├── knowledge/                # Kernel knowledge YAML files
├── app/                      # Tauri desktop app (paused)
└── docs/                     # Documentation
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.
