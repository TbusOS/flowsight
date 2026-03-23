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

> **技术设计详情**: [cli/docs/TECHNICAL-DESIGN.md](cli/docs/TECHNICAL-DESIGN.md)

### v0.4.0 — CFG + Error Path + Macro Semantics + Cross-File (2026-03-21)

> 分析引擎补课 + 跨文件分析。借鉴 tree-climber / Smatch / Joern / stack-graphs。

- [x] `flowsight-cfg` crate — Control Flow Graph construction from tree-sitter AST (18 tests)
- [x] Basic block identification (if/else, switch, for/while, goto/label)
- [x] Error path detection (goto err_*, return -EXXX, IS_ERR patterns)
- [x] Reachability annotation: Always / Conditional / ErrorPath / ConditionalCompilation
- [x] Kernel macro semantics table — 80+ macros (INIT_WORK, list_for_each, DEFINE_MUTEX, spin_lock, etc.)
- [x] Execution context tracking (spin_lock → atomic, rcu_read_lock → RCU read)
- [x] `flowsight cfg <file> <function>` — CFG output (text/DOT/JSON)
- [x] `flowsight errors <file> [function]` — error path listing
- [x] `flowsight flow --error-only / --happy-path / --show-conditions` flags
- [x] Cross-file callers/callees using SQLite index (`--index` flag)
- [x] `flowsight callers <func> --index <db> --group-by-subsystem`
- [x] `flowsight path --from A --to B --index <db>` — BFS call chain path finding
- [x] `flowsight path --from A --to B --index <db> --all` — all paths (DFS)
- [x] `flowsight subsystem-deps --index <db>` — subsystem dependency graph (DOT/JSON)
- [x] `flowsight flow --cross-file --index <db>` — cross-file flow expansion
- [x] 9 new integration tests (72 total across workspace)

### v0.5.0 — CPG + Data Flow (2026-03-22)

> 代码属性图 + 数据流分析。借鉴 Joern CPG。

- [x] `flowsight-cpg` crate — Code Property Graph (8 tests)
- [x] Variable definition extraction (declarations, assignments, call returns, field assignments)
- [x] Variable use extraction (conditions, arguments, return values, dereferences, field access)
- [x] Reaching definitions: worklist algorithm on CFG (fixed-point iteration)
- [x] Def-use chains: each definition → all its uses
- [x] Backward/forward slicing
- [x] `flowsight dataflow <file> <function>` — full data flow summary
- [x] `flowsight dataflow --var <name>` — single variable tracking with chains
- [ ] `flowsight index build --with-cfg` — index with CFG + data flow info
- [ ] Performance optimization for 30,000+ file kernel trees

### v0.6.0 — LLM Integration Layer (2026-03-21)

> 多 Provider 接入 + 自然语言查询 + 本地模型。借鉴 Aider repo map / MCPtrace。

- [x] `flowsight-llm` crate — unified LLM Provider trait (8 tests)
- [x] Provider: OpenAI-compatible API (GPT-4o, DeepSeek, LM Studio, vLLM, Ollama /v1)
- [x] Provider: Anthropic Claude API (Messages API + SSE streaming)
- [x] Provider: Ollama (local models, including self-trained kernel expert)
- [x] `flowsight ask "<query>"` — natural language query with CFG context injection
- [x] `flowsight explain <file> <function>` — AI-powered function explanation
- [x] `flowsight llm-providers` — list configured providers
- [x] `flowsight llm-test [provider]` — test provider connectivity
- [x] SSE streaming output (default) + --no-stream blocking mode
- [x] `[llm]` section in `.flowsight.toml` for provider configuration
- [x] REPL `ask` and `explain` commands with --provider flag
- [x] `flowsight review <file> [function]` — AI code review with CFG + AQS context
- [ ] Smart context builder with PageRank ranking
- [ ] MCP server mode (optional)

### v0.7.0 — Language Expansion + DPO Feedback Loop

> C++/Rust 支持 + 用户反馈闭环。借鉴 Semgrep generic AST / IRIS paper。

- [ ] C++ source file support (classes, namespaces, templates, virtual methods)
- [ ] Rust source file support (traits, impl blocks, async/await, Result/?)
- [ ] Generic AST layer for multi-language CFG construction
- [ ] `flowsight feedback --good / --bad` — DPO feedback collection
- [ ] DPO training pair auto-generation from user corrections
- [ ] Training data quality scoring and filtering
- [ ] `flowsight watch` — file system monitoring with incremental re-analysis

### v0.8.0 — Autonomous Analysis Evolution (AutoResearch Pattern)

> 自主分析进化系统。借鉴 Karpathy autoresearch 的约束驱动自动化范式：固定预算、单一标量指标、keep/discard 循环。

#### 分析预算系统

- [x] `flowsight quality <dir> --budget <seconds>` — 固定时间预算分析
- [x] 优先级调度：入口函数 (probe/init/open) > 回调函数 > 工具函数
- [x] 预算耗尽时输出覆盖率报告（已分析 X/Y 文件，Z% 覆盖）
- [x] `flowsight bench <dir>` — 固定预算基准测试，输出可比指标
- [x] 自适应降级：Full → Fast → Degraded → Skipped

#### 分析质量标量 (Analysis Quality Score, AQS)

- [x] 单一标量指标 `aqs`（0.0 — 1.0），衡量分析深度和准确度
- [x] 维度加权：调用解析率 × 0.3 + 间接调用识别率 × 0.2 + 知识库命中率 × 0.2 + 错误路径覆盖率 × 0.15 + 跨文件解析率 × 0.15
- [x] `flowsight quality <file|dir>` — 输出 AQS 分数 + 各维度明细
- [x] `flowsight quality --baseline` — 对 arm/mach-imx 建立基线 AQS

#### 知识库自进化循环 (KB Evolution Loop)

- [ ] `flowsight kb evolve <dir>` — 自动发现缺失的内核 API 并生成候选 YAML
- [ ] LLM 生成候选条目 → 真实内核代码验证 → AQS 提升则 keep，否则 discard
- [ ] 进化日志：TSV 格式记录每次迭代（commit, aqs_before, aqs_after, status, description）
- [ ] `flowsight kb evolve --rounds <N>` — 运行 N 轮自主进化
- [ ] 策略文件 `.flowsight-strategy.md` — 人类编写的进化策略指引（类似 autoresearch 的 program.md）

#### 分析降级与容错

- [x] 预算调度器中自动降级 (Full → Fast → Degraded → Skipped)
- [x] 解析失败时跳过并记录（继续下一个文件）
- [x] OOM/超时视为信息（记录 AQS=0，继续下一个文件）
- [ ] 间接调用解析失败标记为 `Unknown { reason, confidence: 0.0 }` 继续分析
- [ ] 跨文件索引缺失时标注 `[unindexed]` 而非报错

#### 实验跟踪

- [x] `flowsight experiment start <name>` — 创建实验 + 基线 AQS 快照
- [x] `flowsight experiment log` — 查看实验历史（TSV 格式）
- [x] `flowsight experiment best` — 输出最佳配置
- [x] `flowsight experiment list` — 列出所有实验
- [ ] `flowsight experiment run` — 自动执行修改 → 测量 AQS → keep/discard

### v1.0.0 — Stable Release

- [ ] Stable public API for all commands
- [ ] Comprehensive documentation site
- [ ] Binary releases on crates.io
- [ ] Docker image for CI/CD integration
- [ ] Benchmark suite with reproducible performance results
- [ ] 90%+ test coverage
- [ ] LSP server mode for IDE integration

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
│   │   └── output/           # Output formatters
│   ├── tests/
│   └── docs/
│       └── TECHNICAL-DESIGN.md  # Technical design & roadmap details
├── crates/                   # Shared analysis engine
│   ├── flowsight-core/       # Core types (FlowNode, CallEdge...)
│   ├── flowsight-parser/     # Tree-sitter C parser
│   ├── flowsight-analysis/   # Static analysis engine
│   ├── flowsight-cfg/        # [v0.4.0] Control flow graph construction
│   ├── flowsight-knowledge/  # Knowledge base (137 YAML)
│   ├── flowsight-index/      # Symbol indexing
│   ├── flowsight-llm/        # [v0.6.0] LLM provider integration
│   └── ...
├── knowledge/                # Kernel knowledge YAML files
├── app/                      # Tauri desktop app (paused)
└── docs/                     # Documentation
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.
