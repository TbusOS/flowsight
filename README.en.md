<p align="center">
  <img src="docs/images/logo.svg" alt="FlowSight" width="140"/>
</p>

<h1 align="center">FlowSight</h1>

<p align="center">
  <strong>Static execution flow analyzer for Linux kernel code</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/flowsight-cli"><img src="https://img.shields.io/crates/v/flowsight-cli.svg" alt="crates.io"/></a>
  <a href="https://github.com/TbusOS/flowsight/actions"><img src="https://img.shields.io/github/actions/workflow/status/TbusOS/flowsight/ci.yml?branch=main" alt="CI"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"/></a>
  <img src="https://img.shields.io/badge/rust-1.75+-orange.svg" alt="Rust 1.75+"/>
</p>

<p align="center">
  <a href="#features">Features</a> |
  <a href="#installation">Installation</a> |
  <a href="#quick-start">Quick Start</a> |
  <a href="#output-formats">Output Formats</a> |
  <a href="#knowledge-base">Knowledge Base</a> |
  <a href="#contributing">Contributing</a>
</p>

---

FlowSight is a command-line tool that statically analyzes C source code to trace execution flows, resolve function pointers, detect async handlers, and map callback chains. It is purpose-built for navigating large codebases like the Linux kernel, where indirect calls, deferred work, and framework callbacks make control flow hard to follow.

## Features

- **Execution flow tracing** -- Follow a function's call tree with depth control and kernel API filtering
- **ftrace-style output** -- Render call graphs in the same format as Linux `function_graph` tracer
- **Async handler detection** -- Find work queues, timers, tasklets, IRQ handlers, and kthreads
- **Callback analysis** -- Identify function pointer assignments in ops tables and struct initializers
- **Caller/callee graphs** -- Show who calls a function and what it calls
- **Knowledge base** -- 137 YAML definitions covering Linux kernel subsystems (drivers, mm, net, fs, sync, arch)
- **Sequence diagrams** -- ASCII multi-lane diagrams showing async execution across subsystem boundaries
- **Interactive REPL** -- Explore code with history, tab completion, and inline help
- **Multiple output formats** -- Text, JSON, ftrace, sequence diagram, and Markdown

## Installation

### From crates.io

```bash
cargo install flowsight-cli
```

### From source

```bash
git clone https://github.com/TbusOS/flowsight.git
cd flowsight
cargo build --release --package flowsight-cli
# Binary is at target/release/flowsight
```

Requires Rust 1.75 or later.

## Quick Start

Analyze a kernel source file:

```bash
flowsight analyze drivers/usb/gadget/udc/fsl_udc_core.c
```

Trace the execution flow of a function:

```bash
flowsight flow drivers/usb/gadget/udc/fsl_udc_core.c fsl_udc_probe
```

Limit depth and hide kernel API calls:

```bash
flowsight flow --depth 3 --no-kernel arch/arm/mach-imx/pm-imx6.c imx6q_pm_init
```

Show ftrace-style output:

```bash
flowsight trace arch/arm/mach-imx/clk-imx6q.c imx6q_clocks_init
```

List async handlers in a file:

```bash
flowsight async drivers/usb/gadget/udc/fsl_udc_core.c
```

List callbacks and ops table assignments:

```bash
flowsight callbacks drivers/usb/gadget/udc/fsl_udc_core.c
```

Show callers and callees:

```bash
flowsight callers drivers/usb/gadget/udc/fsl_udc_core.c fsl_udc_probe
flowsight callees drivers/usb/gadget/udc/fsl_udc_core.c fsl_udc_probe
```

Query the knowledge base:

```bash
flowsight kb stats
flowsight kb query work_struct
flowsight kb chain usb_driver probe
flowsight kb async-chain work_struct
flowsight kb match drivers/usb/gadget/udc/fsl_udc_core.c
```

Launch the interactive REPL:

```bash
flowsight interactive
# or just:
flowsight
```

## Output Formats

Use the global `-F` flag to switch formats:

```bash
flowsight -F json  analyze file.c          # Structured JSON
flowsight -F ftrace flow file.c func       # ftrace function_graph style
flowsight -F sequence flow file.c func     # ASCII sequence diagram
flowsight -F markdown analyze file.c       # Markdown tables
```

### ftrace output example

```
 0)               |  fsl_udc_probe() {
 0)               |    usb_add_gadget_udc() {
 0)   0.000 us    |      device_register();
 0)               |    }
 0)               |    INIT_WORK() {
 0)               |      /* deferred -> fsl_udc_work_handler */
 0)               |    }
 0)               |  }
```

### Sequence diagram output example

```
  fsl_udc_probe        workqueue            IRQ
  -------------        ---------            ---
       |                   |                  |
       |--INIT_WORK------->|                  |
       |                   |                  |
       |--request_irq------------------------->
       |                   |                  |
       |  schedule_work--->|                  |
       |                   |--handler()       |
       |                   |                  |
```

## Knowledge Base

FlowSight ships with 137 YAML knowledge files covering:

| Area | Examples |
|------|----------|
| Core | workqueue, timer, kthread, softirq, RCU, signals |
| Memory | page_alloc, vmalloc, slab, OOM, DMA, ioremap |
| Drivers | USB, I2C, SPI, GPIO, platform, DRM, input, clk |
| Networking | TCP, UDP, socket, netfilter, XDP, ARP, ICMP |
| Filesystems | VFS, ext4, procfs, sysfs, tmpfs |
| Sync | spinlock, mutex, rwlock, semaphore, completion |
| Arch | ARM32, ARM64, x86, RISC-V |

The knowledge base maps framework registration macros to their kernel call chains, enabling FlowSight to show how a driver's `.probe` function gets called through the device model.

## Architecture

```
flowsight-cli           CLI frontend (clap + REPL)
    |
flowsight-analysis      Execution flow, async tracking, callback resolution
flowsight-knowledge     YAML knowledge base loader and matcher
flowsight-query         Cross-file query engine
flowsight-index         Symbol table and call graph index
flowsight-parser        Tree-sitter C parser
flowsight-core          Shared types and data structures
```

All crates live under `crates/` and share a Cargo workspace.

## Contributing

Contributions are welcome. To get started:

```bash
git clone https://github.com/TbusOS/flowsight.git
cd flowsight
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
```

Before submitting a pull request:

1. Run `cargo fmt` and `cargo clippy`
2. Add tests for new functionality
3. Keep commits focused and use [conventional commit](https://www.conventionalcommits.org/) messages

See [docs/developer/](docs/developer/) for architecture details.

## License

MIT -- see [LICENSE](LICENSE) for details.
