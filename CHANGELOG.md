# Changelog

All notable changes to FlowSight are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.2.0]: https://github.com/TbusOS/flowsight/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/TbusOS/flowsight/releases/tag/v0.1.0
