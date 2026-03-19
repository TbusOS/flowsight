# Changelog

All notable changes to FlowSight are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned

- C++ / Rust source file support
- LSP server mode for IDE integration
- More kernel subsystem scenarios (block I/O, crypto, IPC)
- `flowsight watch` for file system monitoring and incremental analysis
- Web-based interactive report viewer

## [0.3.0] - 2026-03-19

### Added

- **Recursive directory analysis** (`analyze -r`) with rayon parallel processing
- **Cross-file SQLite index** (`index build/query/stats/update`) with subsystem auto-detection
- **Kernel pattern detection** (`patterns`) - locking, error handling, lifecycle, async, memory, RCU
- **Execution flow diff** (`diff`) - compare flows between file versions using LCS algorithm
- **Kernel scenario engine** (`scenario`) - 10 built-in scenarios: USB enumeration, driver probe, IRQ handling, memory allocation, network packet RX, filesystem read, scheduler, platform device, clock framework, GPIO operations
- **Training data pipeline** (`train generate/stats`) - SFT, DPO, ChatML JSONL generation for LLM fine-tuning
- **Symbol search** (`search`) - direct scan and SQLite-indexed search with regex support
- **HTML report generation** (`report`) - self-contained dark-theme HTML with interactive tables and filtering
- **DOT graph export** (`graph`, `-F dot`) - Graphviz DOT output for full file call graphs with clustering
- **Project config** (`config init/show/path`) - `.flowsight.toml` with CLI-overrides-config merging
- **Shell completions** (`completions`) - bash, zsh, fish auto-completion generation
- Install script (`install.sh`) and Makefile
- GitHub Actions CI/CD with 4-platform release builds (Linux/macOS x86_64/aarch64)
- Issue and PR templates
- 54 CLI integration tests with realistic kernel driver fixtures
- `#![forbid(unsafe_code)]` on all 10 crates

### Fixed

- Hardcoded developer paths removed from test files (use `LINUX_KERNEL_PATH` env var)
- Python code injection vulnerability in training script (paths via env vars now)
- Unbounded recursion in flow tree rendering (MAX_RENDER_DEPTH = 50)
- Clippy `eq_op` error in scenario.rs
- Silent `Ok(())` return when function not found (now returns proper error)
- Unused Cargo dependencies removed from CLI crate

### Security

- `#![forbid(unsafe_code)]` enforced across all crates
- `.claude/settings.local.json` excluded from git tracking
- `Cargo.lock` now committed for reproducible binary builds
- Repository URL updated from placeholder to actual GitHub URL

## [0.2.0] - 2026-03-18

### Added

- Interactive REPL mode with logo, command history, and tab completion
- ASCII multi-lane sequence diagram output format (`-F sequence`)
- Knowledge base commands: `kb stats`, `kb query`, `kb chain`, `kb async-chain`, `kb match`
- Execution flow depth control (`--depth`) and kernel API filtering (`--no-kernel`)
- Async boundary expansion (`--expand-async`) for deferred execution tracing
- Markdown output format (`-F markdown`)
- 137 YAML knowledge base files covering Linux kernel subsystems
- Auto-launch REPL when invoked with no arguments
- Low-saturation color palette for REPL output

### Fixed

- CJK character alignment in sequence diagram columns
- Sequence diagram column overflow with truncation and 36-char column width

## [0.1.0] - 2026-03-13

### Added

- Modular CLI architecture with `clap` subcommands
- `analyze` command for file-level static analysis
- `flow` command for execution flow tracing
- `trace` command for ftrace-style function graph output
- `callers` and `callees` commands for call graph queries
- `async` command for listing async handlers (work queues, timers, IRQs, tasklets)
- `callbacks` command for detecting function pointer assignments in ops tables
- Text, JSON, and ftrace output formats via global `-F` flag
- Tree-sitter-based C parser with function pointer resolution
- Async mechanism tracker (INIT_WORK, setup_timer, request_irq, etc.)
- Callback analyzer for struct initializer and assignment patterns

[Unreleased]: https://github.com/TbusOS/flowsight/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/TbusOS/flowsight/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/TbusOS/flowsight/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/TbusOS/flowsight/releases/tag/v0.1.0
