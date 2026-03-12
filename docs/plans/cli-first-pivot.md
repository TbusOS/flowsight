# FlowSight CLI-First 转型方案

> 版本: v1.0
> 创建日期: 2026-03-13
> 状态: 执行中
> 前置版本: PROJECT-PLAN-V3 (IDE 版本)

---

## 1. 转型背景

### 1.1 为什么从 IDE 转向 CLI

FlowSight 的**核心价值**是函数执行流分析和内核专家模型训练，而不是一个漂亮的 IDE 界面。

| 问题 | 影响 |
|------|------|
| IDE 开发大量时间花在 UI 上 | React 组件 105+ 文件，36 个组件目录，偏离核心 |
| Tauri 桌面应用调试困难 | 前后端联调、原生 API 测试、跨平台 |
| 分析引擎能力无法快速验证 | 每次改动都要启动完整桌面应用 |
| 训练数据生成需要批处理 | IDE 面向单文件交互，不适合批量管道 |

### 1.2 CLI-First 的优势

- **快速落地**: 直接调用分析引擎，无 UI 层
- **管道友好**: `flowsight analyze dir/ --format json | jq`
- **批量处理**: 一条命令分析整个内核子系统
- **训练数据管道**: 直接生成 JSONL 用于模型微调
- **可组合**: 与 shell 工具链无缝集成

### 1.3 IDE 的定位

IDE (`app/` 目录) **不删除**，保留在 workspace 中。CLI 和 IDE 共享同一套 `crates/` 分析引擎。待 CLI 功能成熟后，IDE 可以作为可视化前端恢复开发。

---

## 2. 项目结构

```
flowsight/
├── cli/                       # CLI 入口 (当前开发重点)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # 薄分发器 + 全局选项
│       ├── context.rs         # 共享 AnalysisContext
│       ├── commands/          # 命令实现
│       │   ├── analyze.rs     # 文件/目录分析
│       │   ├── flow.rs        # 执行流 + ftrace
│       │   ├── graph.rs       # 调用图 (callers/callees)
│       │   ├── async_cmd.rs   # 异步机制 + 回调
│       │   ├── kb.rs          # 知识库查询 (Phase 2)
│       │   ├── scenario.rs    # 场景执行 (Phase 2)
│       │   ├── index.rs       # 符号索引 (Phase 2)
│       │   └── train.rs       # 训练数据生成 (Phase 3)
│       └── output/            # 输出格式化
│           ├── mod.rs         # OutputFormat enum
│           ├── text.rs        # 文本/ftrace 格式
│           ├── json.rs        # JSON 格式
│           └── training.rs    # 训练数据格式 (Phase 3)
├── crates/                    # 共享 Rust 分析库 (CLI + IDE 共用)
│   ├── flowsight-core/       # 核心类型
│   ├── flowsight-parser/     # Tree-sitter 代码解析
│   ├── flowsight-analysis/   # 分析引擎 (12,380 行)
│   ├── flowsight-knowledge/  # 知识库 (137 YAML)
│   ├── flowsight-ai/         # 本地 AI 推理
│   ├── flowsight-learning/   # 反馈学习系统
│   ├── flowsight-symbolic/   # KLEE 符号执行
│   ├── flowsight-llvm/       # LLVM IR 分析
│   ├── flowsight-index/      # 符号索引
│   └── flowsight-query/      # 查询接口
├── app/                       # IDE 入口 (暂停，保留)
│   ├── src/                  # React 前端
│   └── src-tauri/            # Tauri 后端
├── knowledge/                 # 知识库数据 (共享)
│   └── platforms/linux-kernel/  # 137 个 YAML 文件
└── Cargo.toml                 # Workspace
```

---

## 3. 分阶段实施计划

### Phase 1: CLI 基础重构 [已完成 2026-03-13]

**目标**: 将 473 行 main.rs 重构为模块化架构

| 任务 | 状态 | 说明 |
|------|------|------|
| `cli/` 提升到顶级目录 | done | 从 `crates/flowsight-cli/` 移出 |
| `context.rs` 共享上下文 | done | Parser + Analyzer 一次初始化 |
| `output/` 输出模块 | done | 5 种格式: text/json/ftrace/mermaid/markdown |
| `commands/` 命令拆分 | done | analyze/flow/graph/async_cmd 四个模块 |
| 全局选项 `-F/--format` | done | 所有命令共享输出格式参数 |
| 真实内核文件验证 | done | fsl_udc_core.c, pm-imx6.c 通过 |

### Phase 2: 增强分析命令 [待实施]

**目标**: 暴露所有分析引擎能力到 CLI

| 命令 | 功能 | 依赖 |
|------|------|------|
| `analyze <dir> --recursive` | 目录级批量分析 + 进度条 | ParallelParser |
| `flow --depth --expand-async` | 深度限制 + 异步展开 | - |
| `graph full --format dot` | 完整调用图 + Graphviz 输出 | - |
| `graph path --from A --to B` | 函数间路径查找 | - |
| `kb stats` | 知识库统计 | KnowledgeBase |
| `kb query <term>` | 知识库搜索 | KnowledgeBase |
| `kb chain <framework> <cb>` | 完整内核调用链 | KnowledgeBase |
| `kb match <file>` | 文件的框架匹配 | KnowledgeBase + Analyzer |
| `scenario run --bind "ptr=null"` | 场景执行 + 值绑定 | ScenarioEngine |
| `scenario explore` | 多路径探索 | ScenarioEngine |
| `index build <dir>` | 持久化符号索引 | BatchIndexer + SQLite |
| `index search <pattern>` | 索引查询 | SymbolIndex |

### Phase 3: 训练数据管道 [待实施]

**目标**: CLI 成为内核专家模型训练数据的生成工具

#### 3.1 训练数据格式

```jsonl
# Alpaca SFT 格式
{"instruction": "分析 fsl_udc_irq 的完整执行流程", "input": "drivers/usb/gadget/udc/fsl_udc_core.c", "output": "fsl_udc_irq() 是 USB UDC 驱动的中断处理函数..."}

# ChatML 格式
{"messages": [{"role": "system", "content": "你是 Linux 内核专家"}, {"role": "user", "content": "..."}, {"role": "assistant", "content": "..."}]}

# DPO 偏好优化格式
{"prompt": "...", "chosen": "准确的分析结果", "rejected": "不准确的结果"}
```

#### 3.2 Q/A 生成类别

| 类别 | 示例问题 | 数据来源 |
|------|---------|---------|
| 函数执行流 | "fsl_udc_irq 的完整调用链？" | flow_trees |
| 异步机制 | "这个 WorkQueue 在什么上下文执行？" | async_bindings |
| 内核调用链 | "USB probe 从设备插入到 probe 回调的路径？" | knowledge base |
| 回调识别 | "这个文件实现了哪些框架回调？" | funcptr resolution |
| 错误路径 | "参数为 NULL 时会发生什么？" | scenario engine |
| 上下文安全 | "这个函数能 sleep 吗？" | KB + analysis |

#### 3.3 命令设计

```bash
# 从目录生成 SFT 训练数据
flowsight train generate /path/to/kernel --format sft --output train.jsonl

# 只生成特定类别
flowsight train generate /path/to/kernel --categories flow,async,chains

# 生成统计报告
flowsight train generate /path/to/kernel --format sft --stats

# 人工反馈 (用于 DPO)
flowsight train feedback add --file x.c --function foo --correct
flowsight train feedback export --format dpo
```

### Phase 4: 交互与配置 [待实施]

| 任务 | 优先级 | 说明 |
|------|--------|------|
| REPL 模式 `flowsight interactive <dir>` | P2 | rustyline, Tab 补全, 一次加载多次查询 |
| `.flowsight.toml` 项目配置 | P2 | 内核路径/glob/分析参数 |
| Shell 补全 (bash/zsh/fish) | P2 | clap_complete |
| AI 解释命令 `flowsight explain` | P3 | 依赖 flowsight-ai 本地模型 |
| TUI (ratatui) | 跳过 | 投入产出比不划算 |

---

## 4. 核心架构

### 4.1 数据流

```
源代码 (.c)
    │
    ▼
Parser (tree-sitter)  ──────►  ParseResult (functions, structs, calls)
    │
    ▼
Analyzer
    ├── AsyncTracker    ──────►  async_bindings (WorkQueue, Timer, IRQ...)
    ├── FuncPtrResolver ──────►  function pointer → concrete function
    ├── CallbackAnalyzer ─────►  framework callback identification
    ├── CallGraphBuilder ─────►  call_edges (direct + indirect + async)
    └── FlowBuilder     ─────►  flow_trees (with kernel chain injection)
            │
            ├── KnowledgeBase (137 YAML) ──► 注入内核调用链
            └── ScenarioEngine ────────────► 多路径探索
    │
    ▼
AnalysisResult
    │
    ├──► CLI 输出 (text/json/ftrace/mermaid/markdown)
    ├──► 训练数据 (JSONL SFT/ChatML/DPO)
    └──► IDE 可视化 (暂停)
```

### 4.2 知识库结构

```yaml
# knowledge/platforms/linux-kernel/drivers/usb.yaml
usb_driver:
  description: "USB device driver framework"
  header: "linux/usb.h"
  callbacks:
    probe:
      trigger: "USB 设备插入并且 ID 匹配"
      context: process
      can_sleep: true
      call_chain:
        - function: usb_hub_port_connect
          file: drivers/usb/core/hub.c
          context: process
        - function: usb_new_device
          ...
        - function: drv->probe()
          is_user_entry: true
```

### 4.3 训练数据管道架构

```
flowsight train generate <dir>
    │
    ├── ParallelParser   ──► 批量解析所有 .c 文件
    ├── Analyzer         ──► 每个文件的 AnalysisResult
    ├── QA Generator     ──► 从分析结果生成 Q/A 对
    │   ├── FlowQuestions      (from flow_trees)
    │   ├── AsyncQuestions      (from async_bindings)
    │   ├── ChainQuestions      (from knowledge base)
    │   ├── CallbackQuestions   (from funcptr resolution)
    │   ├── ErrorPathQuestions  (from scenario engine)
    │   └── ContextQuestions    (from KB + analysis)
    ├── Formatter        ──► 转换为 JSONL/ChatML/DPO
    └── Stats Reporter   ──► 统计覆盖率/分布/质量
```

---

## 5. 测试策略

### 5.1 测试用内核

| 配置项 | 值 |
|--------|-----|
| 内核路径 | `/Users/sky/linux-kernel/linux` |
| 优先架构 | ARM32 (`arch/arm/`) |
| 首选目录 | `arch/arm/mach-imx/` (142 文件) |
| 备选目录 | `drivers/usb/`, `drivers/net/` |

### 5.2 验证命令

```bash
# 单文件分析
flowsight analyze /path/to/kernel/drivers/usb/gadget/udc/fsl_udc_core.c

# 执行流追踪
flowsight flow /path/to/kernel/drivers/usb/gadget/udc/fsl_udc_core.c fsl_udc_irq

# JSON 管道
flowsight -F json analyze file.c | jq '.entry_points'

# 批量分析 (Phase 2)
flowsight analyze /path/to/kernel/arch/arm/mach-imx/ --recursive --summary

# 训练数据 (Phase 3)
flowsight train generate /path/to/kernel/drivers/usb/ --format sft | wc -l
```

---

## 6. 里程碑

| 里程碑 | 内容 | 状态 |
|--------|------|------|
| M1: CLI 重构 | Phase 1 完成，模块化架构 | done |
| M2: 增强分析 | Phase 2 完成，所有分析能力 CLI 化 | 待实施 |
| M3: 训练管道 | Phase 3 完成，可生成训练数据 | 待实施 |
| M4: 首批训练 | 用 ARM32 子系统生成首批训练数据 | 待实施 |
| M5: 模型微调 | 用生成的数据微调内核专家模型 | 待实施 |

---

## 7. 风险与缓解

| 风险 | 级别 | 缓解 |
|------|------|------|
| ParallelParser 在大目录上不稳定 | 中 | 用 mach-imx/ (142 文件) 集成测试 |
| 训练数据质量不够自然 | 中 | 人工反馈 + DPO，只用 Certain 置信度 |
| AI 模型下载/初始化脆弱 | 中 | `--no-ai` 回退到模板生成 |
| 知识库加载慢 (137 YAML) | 低 | 二进制缓存 |

---

## 附录

### 相关文档

- [PROJECT-PLAN-V3.md](../design/PROJECT-PLAN-V3.md) - 原始 IDE 版项目计划
- [EXECUTION-FLOW-DESIGN.md](../design/EXECUTION-FLOW-DESIGN.md) - 执行流设计
- [KNOWLEDGE-BASE-SCHEMA.md](../architecture/KNOWLEDGE-BASE-SCHEMA.md) - 知识库架构
- [LOCAL-AI-DESIGN.md](../architecture/LOCAL-AI-DESIGN.md) - 本地 AI 设计
- [STATIC-ANALYSIS-DESIGN.md](../architecture/STATIC-ANALYSIS-DESIGN.md) - 静态分析设计
