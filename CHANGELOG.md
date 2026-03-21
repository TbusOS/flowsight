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

## [0.6.0] - 2026-03-21

### Added

- **LLM integration** (`flowsight-llm` crate) — unified provider interface
  - OpenAI-compatible API provider (GPT-4o, DeepSeek, LM Studio, vLLM, Ollama /v1)
  - Anthropic Claude API provider (Messages API with SSE streaming)
  - Provider registry with lazy creation and caching
  - TOML-serializable config with env var API key resolution
- **`flowsight ask <query>`** — natural language code query with LLM
  - `--provider` flag to select LLM backend
  - `--file` + `--function` for injecting CFG analysis context
  - Streaming output by default, `--no-stream` for blocking mode
- **`flowsight explain <file> <function>`** — AI-powered function explanation
- **`flowsight llm-providers`** — list configured providers
- **`flowsight llm-test [provider]`** — test provider connectivity
- REPL `ask` and `explain` commands
- `[llm]` config section in `.flowsight.toml` with provider presets

## [0.4.0] - 2026-03-21

### Added

- **`flowsight-cfg` crate** — Control Flow Graph construction from tree-sitter AST (18 tests)
  - Basic block identification: if/else, switch/case, while/for/do, goto/label, return
  - Error path detection: goto err_* chains, early return -EXXX, IS_ERR patterns
  - Reachability annotation: Always / Conditional / ErrorPath per call site
  - Kernel macro semantics table: 80+ macros classified (async registration, context change, declaration, iterator, etc.)
  - DOT graph output for Graphviz visualization
- **`flowsight cfg <file> <function>`** — control flow graph display (text/DOT/JSON)
- **`flowsight errors <file> [function]`** — list error handling paths
- **`flowsight flow` enhanced** with CFG-aware reachability tags
  - `[always]` / `[conditional]` / `[error-path]` annotations
  - `--show-conditions` flag for branch condition display
  - `--error-only` flag for error paths only
  - `--happy-path` flag for normal path only
  - `--cross-file --index <db>` for cross-file call expansion
- **`flowsight callers <func> --index <db>`** — cross-file caller analysis
  - `--group-by-subsystem` for subsystem-grouped output
- **`flowsight callees <func> --index <db>`** — cross-file callee analysis
- **`flowsight path --from A --to B --index <db>`** — BFS call chain path finding
  - `--all` flag for finding up to 20 paths (DFS)
  - `--max-depth N` configurable search depth
- **`flowsight subsystem-deps --index <db>`** — subsystem dependency graph (DOT/JSON)
- 9 new CLI integration tests for CFG commands

### Fixed

- Stack overflow on large kernel trees (rayon thread stack increased to 32MB)

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
