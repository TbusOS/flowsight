# FlowSight CLI — 技术设计文档

> **版本**: v0.4.0 规划
> **日期**: 2026-03-20
> **状态**: 已评审，待实施
> **目标读者**: 后续开发者（人类或 AI Agent）

---

## 目录

1. [项目定位与最终目标](#1-项目定位与最终目标)
2. [当前架构 (v0.3.0)](#2-当前架构-v030)
3. [问题诊断：现有方案的根本缺陷](#3-问题诊断现有方案的根本缺陷)
4. [业界调研：值得借鉴的项目与方法](#4-业界调研值得借鉴的项目与方法)
5. [技术路线图 (v0.4.0 — v0.7.0)](#5-技术路线图-v040--v070)
6. [Phase 1: CFG + 错误路径 + 宏语义 (v0.4.0)](#6-phase-1-cfg--错误路径--宏语义-v040)
7. [Phase 2: CPG + 跨文件智能 (v0.5.0)](#7-phase-2-cpg--跨文件智能-v050)
8. [Phase 3: LLM 集成层 (v0.6.0)](#8-phase-3-llm-集成层-v060)
9. [Phase 4: 语言扩展 + DPO 闭环 (v0.7.0)](#9-phase-4-语言扩展--dpo-闭环-v070)
10. [开发规范与约束](#10-开发规范与约束)
11. [附录](#11-附录)

---

## 1. 项目定位与最终目标

FlowSight 是一个 **Linux 内核代码静态执行流分析工具**，最终目标：

```
输入: C 源代码（重点是 Linux 内核）
输出: 函数的完整执行流（包括条件分支、错误路径、异步回调、跨文件调用）
增强: 支持自然语言查询 + 本地内核专家模型 + 外部 LLM API
```

### 核心使命

1. **精确的执行流分析** — 不是调用树，而是真正的控制流感知执行流
2. **内核领域专家** — 理解 RCU、spinlock、中断上下文、goto 错误处理等内核特有模式
3. **训练数据生成** — 为微调 Linux 内核专家 LLM 提供高质量结构化数据
4. **自然语言交互** — 支持接入多种 LLM（Claude / GPT / Ollama 本地模型）

### 当前状态

- CLI v0.3.0 已完成 17 个命令、310 tests、6 种输出格式
- 基础分析能力可用，但 **执行流分析存在根本性设计缺陷**（见第 3 节）
- 训练数据管道可生成 SFT/DPO/ChatML，但数据质量受限于分析精度
- 无 LLM 集成能力

---

## 2. 当前架构 (v0.3.0)

### 2.1 系统组成

```
flowsight/
├── cli/                          # CLI 工具 (本文档重点)
│   ├── src/
│   │   ├── main.rs               # 命令分发 + 全局配置
│   │   ├── context.rs            # AnalysisContext (parser + analyzer)
│   │   ├── index_db.rs           # SQLite 跨文件索引
│   │   ├── repl.rs               # 交互式 REPL
│   │   ├── config.rs             # .flowsight.toml 配置
│   │   ├── commands/             # 17 个子命令
│   │   └── output/               # 6 种输出格式化器
│   └── tests/                    # 54 个集成测试
├── crates/                       # 共享分析引擎 (10 个 crate)
│   ├── flowsight-core/           # 核心类型 (FlowNode, CallEdge...)
│   ├── flowsight-parser/         # Tree-sitter C 解析器
│   ├── flowsight-analysis/       # 静态分析引擎 ← 需要重大升级
│   ├── flowsight-index/          # 符号索引
│   ├── flowsight-knowledge/      # 内核知识库 (137 YAML)
│   └── ...
└── knowledge/                    # 知识库数据文件
```

### 2.2 分析流水线（当前）

```
C 源码 → Tree-sitter 解析 → AST → 函数提取 → 调用列表提取 → 递归拼装调用树
                                                                      ↑
                                                              这就是全部。没有 CFG。
```

### 2.3 数据模型

当前 `FlowNode` 定义（简化）：

```rust
pub struct FlowNode {
    pub name: String,           // 函数名
    pub call_type: CallType,    // Direct / Indirect / KernelApi / Callback
    pub children: Vec<FlowNode>, // 子调用（平铺列表，无分支信息）
    pub depth: usize,
    pub line: Option<usize>,
}
```

**问题**：`children` 是简单的 `Vec`，不区分条件分支、错误路径、循环体。

### 2.4 索引数据库

SQLite 表结构：

```sql
files    (id, path, subsystem, hash, indexed_at, updated_at)
symbols  (id, name, kind, file_id, start_line, end_line, is_exported, signature)
calls    (id, caller_id, callee_name, call_line, is_indirect)
async_handlers (id, symbol_id, mechanism, variable, target, can_sleep)
```

**问题**：`calls` 表存了跨文件关系，但没有任何 CLI 命令使用它。

---

## 3. 问题诊断：现有方案的根本缺陷

### 3.1 致命问题：「调用树」≠「执行流」

当前分析器把源码中所有函数调用平铺成线性列表，递归拼装。**不区分条件分支、错误路径、循环。**

```c
// 当前分析器看到的执行流：
// probe() → usb_submit_urb() → request_irq() → schedule_work()
//
// 实际执行流（应该看到的）：
int probe(struct usb_device *dev) {
    if (!dev)
        return -ENOMEM;              // [错误路径] 后续全部不执行

    ret = usb_submit_urb(urb);
    if (ret < 0)
        goto err_free;               // [错误路径] 跳到清理代码

    #ifdef CONFIG_PM
    request_irq(...);                // [条件编译] 仅在 CONFIG_PM 启用时
    #endif

    INIT_WORK(&dev->work, handler);  // [不是调用] 是异步回调注册
    return 0;

err_free:
    kfree(dev);                      // [清理路径] 仅在错误时执行
    return ret;
}
```

### 3.2 缺陷清单

| # | 缺陷 | 严重度 | 影响 |
|---|------|--------|------|
| 1 | **无控制流图 (CFG)** | 致命 | 不知道哪些调用是条件的、哪些必定执行 |
| 2 | **无错误路径分析** | 致命 | 内核代码 30%+ 是 `goto err_*` / `return -EXXX`，全部丢失 |
| 3 | **无宏语义理解** | 致命 | `INIT_WORK` 是注册回调不是调用，`list_for_each` 是迭代器不是函数 |
| 4 | **无 #ifdef 处理** | 严重 | 所有条件编译分支都当作必执行代码 |
| 5 | **无数据流** | 严重 | 不跟踪变量值、返回值传播、结构体字段访问 |
| 6 | **异步检测靠正则** | 严重 | `INIT_WORK(w, handler)` 间接变量赋值就失败 |
| 7 | **函数指针只认 ops 表** | 中等 | 运行时赋值、跨文件赋值、间接赋值全部漏掉 |
| 8 | **跨文件索引未接入** | 中等 | SQLite 存了跨文件关系但没有命令使用 |
| 9 | **知识库只是查表** | 中等 | 不参与实际分析推理，仅做标签匹配 |
| 10 | **场景引擎与分析器隔离** | 低 | 符号执行和静态分析完全分离，不能组合 |

### 3.3 根因分析

```
正确的分析流水线：

源代码 → 预处理(宏/ifdef) → AST → CFG构建 → 数据流分析 → 执行流生成
                                      ↑
                              当前停在这里之前

当前的流水线：

源代码 → Tree-sitter AST → 提取函数调用列表 → 递归拼装调用树
                                                    ↑
                                           这不是执行流
```

### 3.4 训练数据质量影响

因为分析引擎的缺陷，当前 `train generate` 产出的 SFT/DPO 数据存在系统性错误：

- 执行流不区分条件分支 → 训练数据教 LLM 所有调用都会执行
- 错误路径丢失 → LLM 学不到内核的错误处理模式
- 宏被当成函数调用 → LLM 对 `INIT_WORK` 等宏的理解会出错
- 跨文件调用标记为 `[External]` → LLM 不知道外部函数做了什么

**结论：分析引擎是一切的基础，必须先修正，否则 LLM 集成建立在错误数据之上。**

---

## 4. 业界调研：值得借鉴的项目与方法

### 4.1 核心结论

**不依赖 LLM 也能做执行流分析。** 业界最强的执行流工具全部基于结构化分析：

| 工具 | 方法 | 是否用 LLM | 内核规模 | 效果 |
|------|------|-----------|---------|------|
| **Smatch** | 状态跟踪 + 插件式流分析 | 否 | ✅ | 20 年内核 bug hunter，Dan Carpenter 维护 |
| **Joern** | CPG (AST+CFG+PDG) | 否 | ✅ | 学术界漏洞检测标杆，2.7k stars |
| **Clang Analyzer** | 符号执行 + 路径敏感 | 否 | ✅ | 生产级，路径全探索 |
| **CodeQL** | 代码→关系数据库 + QL | 否 | ✅ | Google 内核堆漏洞研究 |
| **Facebook Infer** | 分离逻辑 + 组合式分析 | 否 | ✅ | 百万行级 |
| **NASA IKOS** | 抽象解释 | 否 | ⚠️ | 证明无运行时错误 |

**2025 年趋势**：结构化分析 + LLM 推理组合效果最佳。IRIS 论文显示 CodeQL + GPT-4 漏洞检出率是单独使用的 2 倍。**但前提是结构化分析本身要准确。**

### 4.2 最值得借鉴的开源项目

#### Tier 1: 直接借鉴（技术栈兼容 / 算法可移植）

| 项目 | Stars | 语言 | 借鉴点 | 链接 |
|------|-------|------|--------|------|
| **tree-climber** | ~100 | Python | tree-sitter → CFG 构建算法，基于 Joern 的 CfgCreator | [github.com/bstee615/tree-climber](https://github.com/bstee615/tree-climber) |
| **stack-graphs** (GitHub) | ~3k | **Rust** | 增量式跨文件符号解析，tree-sitter + Rust，驱动 GitHub 精确导航 | [github.com/github/stack-graphs](https://github.com/github/stack-graphs) |
| **weggli** (Google P0) | ~2.3k | **Rust** | tree-sitter + Rust 语义搜索，查询语法像 C 代码 | [github.com/weggli-rs/weggli](https://github.com/weggli-rs/weggli) |
| **tree-sitter-graph** | ~400 | **Rust** | tree-sitter 官方 DSL，声明式图构建替代手写 Rust visitor | [github.com/tree-sitter/tree-sitter-graph](https://github.com/tree-sitter/tree-sitter-graph) |
| **rust-code-analysis** (Mozilla) | ~600 | **Rust** | 多语言 tree-sitter 指标提取，架构参考 | [github.com/mozilla/rust-code-analysis](https://github.com/mozilla/rust-code-analysis) |

#### Tier 2: 架构参考（不同语言但方法论重要）

| 项目 | Stars | 语言 | 借鉴点 | 链接 |
|------|-------|------|--------|------|
| **Joern** | ~2.7k | Scala | CPG 规范（AST+CFG+PDG 三合一图），查询式分析 | [github.com/joernio/joern](https://github.com/joernio/joern) |
| **IBM COMEX** | ~200 | Python | tree-sitter → 15+ 代码视图 (CFG/DFG/PDG)，面向 ML 训练 | [github.com/IBM/tree-sitter-codeviews](https://github.com/IBM/tree-sitter-codeviews) |
| **SVF** | ~1.6k | C++ | LLVM IR 上的值流分析，多层图架构 | [github.com/SVF-tools/SVF](https://github.com/SVF-tools/SVF) |
| **Semgrep** | ~10k | OCaml | tree-sitter GLR 解析 + 通用 AST 层，多语言架构参考 | [github.com/semgrep/semgrep](https://github.com/semgrep/semgrep) |
| **uftrace** | ~3k | C | 动态函数追踪，ftrace 格式鼻祖，过滤和深度限制 UX | [github.com/namhyung/uftrace](https://github.com/namhyung/uftrace) |

#### Tier 3: 内核专用工具（领域知识）

| 项目 | 维护者 | 借鉴点 | 链接 |
|------|--------|--------|------|
| **Smatch** | Dan Carpenter | 状态跟踪 + hook 架构，内核级流分析唯一开源实现 | [lwn.net/Articles/1023646](https://lwn.net/Articles/1023646/) |
| **Coccinelle** | Julia Lawall (INRIA) | SmPL 语义模式匹配，大规模内核 API 迁移 | [github.com/coccinelle/coccinelle](https://github.com/coccinelle/coccinelle) |
| **Sparse** | Linus Torvalds 发起 | 注解系统 (`__user`, `__kernel`)，内核语义标注方法 | kernel.org |

#### Tier 4: LLM + 代码分析（未来集成参考）

| 项目 | 借鉴点 | 链接 |
|------|--------|------|
| **Aider repo map** | tree-sitter 符号提取 → PageRank 排序 → LLM 上下文优化 | [github.com/Aider-AI/aider](https://github.com/Aider-AI/aider) |
| **MCPtrace** | Rust MCP server 暴露分析能力给 AI 助手 | [github.com/eunomia-bpf/MCPtrace](https://github.com/eunomia-bpf/MCPtrace) |
| **code-survey** | LLM 系统性分析内核子系统演进 | [github.com/eunomia-bpf/code-survey](https://github.com/eunomia-bpf/code-survey) |
| **IRIS 论文** | CodeQL + GPT-4 组合漏洞检测，检出率翻倍 | [arxiv.org/abs/2405.17238](https://arxiv.org/abs/2405.17238) |
| **LLMxCPG** | CPG 引导 LLM 上下文，USENIX Security 2025 | [arxiv.org/html/2507.16585v1](https://arxiv.org/html/2507.16585v1) |

### 4.3 关键技术洞察

#### tree-sitter 能不能建 CFG？

**能。** tree-climber 和 IBM COMEX 都证明了这一点。核心算法：

1. 遍历函数体 AST
2. 遇到 `if/else/switch/for/while/goto/return` 创建分支节点
3. 为每个基本块 (basic block) 创建 CFG 节点
4. 连接控制流边（fall-through / branch-taken / branch-not-taken）

tree-sitter 的限制：
- 不展开宏 → 用「宏语义表」补偿（我们自己维护常见内核宏的语义）
- 不处理 `#ifdef` → 对于内核代码，将所有分支标记为 `[conditional]`
- 不理解类型 → 用知识库补偿（函数签名 + 上下文类型推断）

#### Smatch 的状态跟踪模型

Smatch 是最接近 FlowSight 需求的参考。核心概念：

```
函数 → 基本块序列 → 每个块维护变量状态集合

状态示例：
  ptr = kmalloc(...)  → ptr: {non-null, null}  (两种可能)
  if (!ptr)           → 分支1: ptr={null}  分支2: ptr={non-null}
  return -ENOMEM      → 终止分支1
  ptr->field          → 在分支2中，ptr 确定非 null
```

FlowSight 不需要实现完整的 Smatch，但应该借鉴其**状态跟踪架构**来标注执行流的分支条件。

#### CPG (Code Property Graph) 概念

Joern 的核心创新。将三种图合并为一个：

```
CPG = AST (抽象语法树)
    + CFG (控制流图)
    + PDG (程序依赖图 = 数据依赖 + 控制依赖)
```

好处：一个图能回答所有问题 — 调用关系、控制流路径、数据依赖、可达性。

FlowSight 当前的 AST / 调用图 / 知识库是分离的数据结构，未来应该统一到类似 CPG 的模型。

---

## 5. 技术路线图 (v0.4.0 — v0.7.0)

### 总览

```
v0.4.0  CFG + 错误路径 + 宏语义          ← 分析引擎补课（最高优先级）
  │     借鉴: tree-climber, Smatch
  │
  v
v0.5.0  CPG + 跨文件智能                 ← 完整代码属性图 + 索引接入
  │     借鉴: Joern, stack-graphs
  │
  v
v0.6.0  LLM Provider + 自然语言 + 本地模型 ← AI 集成层
  │     借鉴: Aider repo map, MCPtrace
  │
  v
v0.7.0  C++/Rust 支持 + DPO 闭环         ← 语言扩展 + 反馈循环
        借鉴: Semgrep 通用 AST, IRIS
```

### 为什么这个顺序？

1. **没有 CFG，训练数据是错的** → 先修分析引擎
2. **没有跨文件，分析是不完整的** → 接索引到命令
3. **有了准确的结构化分析，LLM 才能发挥作用** → 最后加 AI
4. **LLM 集成后可以做 DPO 反馈闭环** → 持续改进

---

## 6. Phase 1: CFG + 错误路径 + 宏语义 (v0.4.0)

### 6.1 目标

在不更换 tree-sitter 的前提下，为分析引擎补上 CFG 能力，使执行流区分条件分支和错误路径。

### 6.2 新增 crate

```
crates/
└── flowsight-cfg/                # 新 crate: 控制流图
    ├── Cargo.toml
    └── src/
        ├── lib.rs                # 公开 API
        ├── builder.rs            # AST → CFG 构建器
        ├── basic_block.rs        # 基本块定义
        ├── edge.rs               # CFG 边（fall-through / branch / goto / return）
        ├── error_path.rs         # 错误路径检测 (goto err_*, return -EXXX)
        ├── macro_semantics.rs    # 内核宏语义表
        └── walker.rs             # CFG 遍历器
```

### 6.3 核心数据结构

```rust
/// 控制流图
pub struct ControlFlowGraph {
    pub function_name: String,
    pub blocks: Vec<BasicBlock>,
    pub edges: Vec<CfgEdge>,
    pub entry: BlockId,
    pub exits: Vec<BlockId>,         // 可能有多个出口（正常 return + 错误 return）
}

/// 基本块 — CFG 的最小单元
pub struct BasicBlock {
    pub id: BlockId,
    pub statements: Vec<Statement>,  // 块内的语句序列
    pub calls: Vec<CallSite>,        // 块内的函数调用
    pub line_range: (usize, usize),  // 源码行范围
    pub block_type: BlockType,       // Normal / ErrorHandler / CleanupGoto / LoopBody
}

/// 调用点 — 比当前的 CallEdge 更丰富
pub struct CallSite {
    pub callee: String,
    pub line: usize,
    pub call_type: CallType,         // Direct / Indirect / MacroExpansion / KernelApi
    pub condition: Option<String>,   // 所在分支的条件表达式
    pub reachability: Reachability,  // Always / Conditional / ErrorPath / ConditionalCompilation
}

/// 可达性标注
pub enum Reachability {
    Always,                          // 必定执行
    Conditional(String),             // 条件执行（附带条件表达式）
    ErrorPath,                       // 仅在错误路径执行
    ConditionalCompilation,          // #ifdef 条件编译
}

/// CFG 边
pub struct CfgEdge {
    pub from: BlockId,
    pub to: BlockId,
    pub edge_type: EdgeType,
}

pub enum EdgeType {
    FallThrough,                     // 顺序执行
    BranchTrue(String),              // 条件为真 (附带条件表达式)
    BranchFalse(String),             // 条件为假
    Goto(String),                    // goto 跳转 (附带标签名)
    Return,                          // 函数返回
    SwitchCase(String),              // switch case (附带 case 值)
    SwitchDefault,                   // switch default
    LoopBack,                        // 循环回边
    LoopExit,                        // 循环退出
}

/// 基本块类型
pub enum BlockType {
    Normal,                          // 普通代码块
    ErrorHandler,                    // goto err_xxx 标签后的清理块
    CleanupGoto,                     // goto 跳转前的错误检查块
    LoopBody,                        // 循环体
    ConditionalCompilation,          // #ifdef 块
}
```

### 6.4 CFG 构建算法

移植自 tree-climber (Joern CfgCreator)，适配 tree-sitter Rust binding：

```
输入: tree-sitter 解析后的函数 AST 节点
输出: ControlFlowGraph

算法:
1. 创建 ENTRY 块和 EXIT 块
2. 遍历函数体的 compound_statement 子节点
3. 对每个语句:
   a. 普通语句 → 加入当前基本块
   b. if_statement →
      - 创建条件块、true 分支块、false 分支块
      - 连接边: current → condition → true_branch / false_branch → merge
   c. for/while/do_while →
      - 创建循环头块、循环体块、循环出口块
      - 连接边: current → loop_head → loop_body → loop_head (回边)
      - 连接边: loop_head → loop_exit
   d. switch_statement →
      - 为每个 case 创建块
      - 处理 fall-through 和 break
   e. goto_statement →
      - 创建跳转边到对应 labeled_statement
      - 标记目标块为 ErrorHandler (如果标签名匹配 err_*/fail_*/out_*/cleanup_*)
   f. return_statement →
      - 连接到 EXIT 块
      - 如果返回负值 (return -EXXX)，标记当前路径为错误路径
   g. labeled_statement →
      - 创建新基本块，记录标签名
4. 收集所有 goto 跳转，解析标签引用
5. 标注错误路径: 从 ErrorHandler 块反向传播
```

### 6.5 内核宏语义表

不做完整的预处理器，用查表法处理高频内核宏：

```rust
/// 宏的语义分类
pub enum MacroSemantics {
    /// 函数调用语义 — 和普通调用一样处理
    FunctionCall,

    /// 异步回调注册 — 提取 handler 参数
    AsyncRegistration {
        handler_arg_index: usize,  // handler 是第几个参数
        mechanism: AsyncMechanism, // WorkQueue / Timer / IRQ / Tasklet
    },

    /// 迭代器宏 — 展开为循环
    Iterator {
        body_is_loop: bool,
    },

    /// 声明宏 — 不产生运行时调用
    Declaration,

    /// 上下文变更 — 改变执行上下文
    ContextChange {
        new_context: ExecutionContext, // 如 spin_lock → atomic context
    },

    /// 分支提示 — 不改变控制流
    BranchHint,

    /// 内存屏障 — 不是函数调用
    MemoryBarrier,

    /// 类型转换 / 指针运算 — 不是函数调用
    TypeCast,
}

// 内置宏语义表 (可通过知识库 YAML 扩展)
fn default_macro_table() -> HashMap<&'static str, MacroSemantics> {
    map! {
        // 异步注册
        "INIT_WORK"             => AsyncRegistration { handler: 1, mechanism: WorkQueue },
        "INIT_DELAYED_WORK"     => AsyncRegistration { handler: 1, mechanism: WorkQueue },
        "setup_timer"           => AsyncRegistration { handler: 1, mechanism: Timer },
        "timer_setup"           => AsyncRegistration { handler: 1, mechanism: Timer },
        "request_irq"           => AsyncRegistration { handler: 1, mechanism: IRQ },
        "request_threaded_irq"  => AsyncRegistration { handler: 2, mechanism: IRQ },
        "tasklet_init"          => AsyncRegistration { handler: 1, mechanism: Tasklet },

        // 迭代器
        "list_for_each"         => Iterator { body_is_loop: true },
        "list_for_each_entry"   => Iterator { body_is_loop: true },
        "for_each_netdev"       => Iterator { body_is_loop: true },
        "for_each_possible_cpu" => Iterator { body_is_loop: true },

        // 声明
        "DEFINE_MUTEX"          => Declaration,
        "DEFINE_SPINLOCK"       => Declaration,
        "DECLARE_WAIT_QUEUE_HEAD" => Declaration,
        "MODULE_LICENSE"        => Declaration,
        "MODULE_AUTHOR"         => Declaration,
        "module_param"          => Declaration,

        // 入口点注册
        "module_init"           => AsyncRegistration { handler: 0, mechanism: ModuleInit },
        "module_exit"           => AsyncRegistration { handler: 0, mechanism: ModuleExit },
        "module_usb_driver"     => AsyncRegistration { handler: 0, mechanism: DriverRegister },
        "module_platform_driver" => AsyncRegistration { handler: 0, mechanism: DriverRegister },

        // 上下文变更
        "spin_lock"             => ContextChange { new_context: Atomic },
        "spin_lock_irqsave"     => ContextChange { new_context: Atomic },
        "spin_unlock"           => ContextChange { new_context: Process },
        "spin_unlock_irqrestore" => ContextChange { new_context: Process },
        "rcu_read_lock"         => ContextChange { new_context: RcuRead },
        "rcu_read_unlock"       => ContextChange { new_context: Process },
        "local_irq_disable"     => ContextChange { new_context: Atomic },
        "local_irq_enable"      => ContextChange { new_context: Process },
        "preempt_disable"       => ContextChange { new_context: Atomic },
        "preempt_enable"        => ContextChange { new_context: Process },
        "mutex_lock"            => ContextChange { new_context: MutexHeld },
        "mutex_unlock"          => ContextChange { new_context: Process },

        // 分支提示
        "likely"                => BranchHint,
        "unlikely"              => BranchHint,

        // 内存屏障
        "barrier"               => MemoryBarrier,
        "mb"                    => MemoryBarrier,
        "rmb"                   => MemoryBarrier,
        "wmb"                   => MemoryBarrier,
        "smp_mb"                => MemoryBarrier,
        "smp_rmb"               => MemoryBarrier,
        "smp_wmb"               => MemoryBarrier,

        // 类型转换
        "container_of"          => TypeCast,
        "to_usb_device"         => TypeCast,
        "to_platform_device"    => TypeCast,
        "dev_get_drvdata"       => FunctionCall,  // 这个确实是函数调用
    }
}
```

### 6.6 错误路径检测

Linux 内核的错误处理有固定模式：

```c
// 模式 1: goto 链式错误处理
ret = step1();
if (ret < 0)
    goto err_step1;
ret = step2();
if (ret < 0)
    goto err_step2;
return 0;

err_step2:
    undo_step2();
err_step1:
    undo_step1();
    return ret;

// 模式 2: 提前返回
if (!ptr)
    return -ENOMEM;

// 模式 3: IS_ERR 检查
clk = clk_get(dev, "fck");
if (IS_ERR(clk))
    return PTR_ERR(clk);
```

检测规则：

```rust
fn is_error_label(label: &str) -> bool {
    let error_prefixes = ["err_", "error_", "fail_", "out_", "cleanup_",
                          "free_", "unwind_", "undo_", "bail_"];
    error_prefixes.iter().any(|p| label.starts_with(p))
}

fn is_error_return(expr: &str) -> bool {
    // return -EXXX
    // return PTR_ERR(x)
    // return ERR_PTR(x)
    // return ERR_CAST(x)
    expr.starts_with("-E") ||
    expr.contains("PTR_ERR") ||
    expr.contains("ERR_PTR") ||
    expr.contains("ERR_CAST") ||
    expr == "ret" || expr == "err" || expr == "rc" || expr == "status"
}
```

### 6.7 增强后的 FlowNode

```rust
/// v0.4.0 增强版执行流节点
pub struct FlowNode {
    pub name: String,
    pub call_type: CallType,
    pub reachability: Reachability,       // 新增: 可达性标注
    pub condition: Option<String>,        // 新增: 所在分支条件
    pub context: Option<ExecutionContext>, // 新增: 执行上下文
    pub children: Vec<FlowBranch>,        // 变更: 从 Vec<FlowNode> 改为分支结构
    pub depth: usize,
    pub line: Option<usize>,
}

/// 执行流分支
pub enum FlowBranch {
    /// 顺序执行的子调用
    Sequential(Vec<FlowNode>),

    /// 条件分支
    Conditional {
        condition: String,
        true_branch: Vec<FlowNode>,
        false_branch: Vec<FlowNode>,
    },

    /// 错误处理分支
    ErrorHandling {
        check: String,          // "ret < 0", "!ptr", "IS_ERR(clk)"
        error_path: Vec<FlowNode>,   // goto err_xxx 或 return -EXXX
        normal_path: Vec<FlowNode>,  // 继续执行
    },

    /// 循环
    Loop {
        condition: String,
        body: Vec<FlowNode>,
    },
}
```

### 6.8 输出示例（增强后）

```
flowsight flow drivers/usb/gadget/udc/fsl_udc_core.c fsl_udc_probe

fsl_udc_probe()
├── [always] platform_get_resource()
├── [always] devm_ioremap()
├── [error-check: !res] return -ENOMEM
│   └── [error-path] 直接返回，不执行后续
├── [always] usb_add_gadget_udc()
├── [error-check: ret < 0] goto err_free_irq
│   └── [error-path]
│       ├── free_irq()
│       └── return ret
├── [conditional: CONFIG_PM] device_init_wakeup()
├── [async-register: WorkQueue] INIT_WORK(&udc->work, fsl_udc_work)
│   └── [deferred] fsl_udc_work()        ← 异步回调，不在此路径执行
│       ├── usb_gadget_giveback_request()
│       └── ...
└── [always] return 0
```

### 6.9 CLI 命令变更

```bash
# 现有命令增强
flowsight flow <file> <function>              # 默认显示所有路径
flowsight flow <file> <function> --error-only # 仅显示错误路径
flowsight flow <file> <function> --happy-path # 仅显示正常路径
flowsight flow <file> <function> --show-conditions  # 显示分支条件

# 新增命令
flowsight cfg <file> <function>               # 输出 CFG (DOT 格式)
flowsight cfg <file> <function> -F json       # CFG JSON 格式
flowsight errors <file> <function>            # 列出所有错误处理路径
flowsight context <file> <function>           # 显示执行上下文变化
```

### 6.10 测试策略

使用真实内核代码测试：

```bash
# 测试内核路径
LINUX_KERNEL_PATH=/Users/sky/linux-kernel/linux

# 测试文件优先级
# 1. ARM32 平台代码 (熟悉的测试场景)
$LINUX_KERNEL_PATH/arch/arm/mach-imx/clk-imx6q.c

# 2. USB 驱动 (丰富的错误处理和异步模式)
$LINUX_KERNEL_PATH/drivers/usb/gadget/udc/fsl_udc_core.c

# 3. 网络驱动 (复杂的中断和回调模式)
$LINUX_KERNEL_PATH/drivers/net/ethernet/intel/e1000/e1000_main.c
```

每个新功能的测试必须覆盖：
- CFG 构建正确性（基本块数量、边数量、入口/出口）
- 错误路径识别准确率
- 宏语义分类正确性
- 条件分支标注正确性
- 与真实内核代码的兼容性

---

## 7. Phase 2: CPG + 跨文件智能 (v0.5.0)

### 7.1 目标

1. 将 AST + CFG + 调用图合并为统一的 Code Property Graph
2. 接通 SQLite 索引到 CLI 命令，实现跨文件分析
3. 子系统级依赖图和路径查找

### 7.2 CPG 数据模型

```rust
/// 代码属性图 — 统一表示
pub struct CodePropertyGraph {
    pub nodes: Vec<CpgNode>,
    pub edges: Vec<CpgEdge>,
}

pub enum CpgNode {
    Function(FunctionNode),     // 函数定义
    BasicBlock(BasicBlockNode), // CFG 基本块
    Statement(StatementNode),   // 语句
    Expression(ExprNode),       // 表达式
    Variable(VarNode),          // 变量
    Symbol(SymbolNode),         // 跨文件符号引用
}

pub enum CpgEdge {
    Ast(AstEdge),              // AST 父子关系
    Cfg(CfgEdge),              // 控制流边
    Call(CallEdge),            // 函数调用边（可跨文件）
    DataDep(DataDepEdge),      // 数据依赖
    ControlDep(ControlDepEdge), // 控制依赖
}
```

### 7.3 跨文件分析架构

```
                    ┌─────────────────────────┐
                    │     SQLite 索引          │
                    │  files / symbols / calls │
                    └──────────┬──────────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
         ┌────┴───┐      ┌────┴───┐      ┌────┴───┐
         │ file1.c │      │ file2.c │      │ file3.c │
         │  CFG    │      │  CFG    │      │  CFG    │
         └────┬───┘      └────┬───┘      └────┬───┘
              │                │                │
              └────────────────┼────────────────┘
                               │
                    ┌──────────┴──────────┐
                    │   跨文件 CPG 查询    │
                    │  callers / path /    │
                    │  dependency graph    │
                    └─────────────────────┘
```

### 7.4 新增/增强命令

```bash
# 跨文件调用者 (使用索引)
flowsight callers <function> --index <db>
flowsight callers usb_submit_urb --index kernel.db --group-by subsystem

# 跨文件调用链路径查找
flowsight path --from usb_submit_urb --to dma_map_single --index kernel.db
flowsight path --from sys_read --to ext4_readpage --index kernel.db --max-depth 10

# 子系统依赖图
flowsight subsystem-deps --index kernel.db -F dot
flowsight subsystem-deps --index kernel.db --focus usb,pci

# 跨文件执行流
flowsight flow <file> <function> --cross-file --index kernel.db

# 增强: 索引构建加入 CFG 信息
flowsight index build <dir> --with-cfg    # 索引时同时构建 CFG
```

### 7.5 性能考虑

Linux 内核规模：~30,000 .c 文件，~700,000 函数

| 操作 | 目标性能 |
|------|---------|
| 索引构建 (全内核) | < 10 分钟 (rayon 并行) |
| 单符号查询 | < 10ms (SQLite 索引) |
| 跨文件路径查找 (depth=10) | < 1 秒 |
| 子系统依赖图生成 | < 5 秒 |

关键优化：
- 惰性加载 CFG（只在需要跨文件展开时解析目标文件）
- SQLite 索引缓存常用查询
- 参考 stack-graphs 的增量更新策略

---

## 8. Phase 3: LLM 集成层 (v0.6.0)

### 8.1 目标

1. 统一的 LLM Provider 接口，支持多种模型后端
2. 自然语言查询分析结果
3. 支持本地训练的内核专家模型
4. 分析结果 → LLM 上下文的智能构建

### 8.2 新增 crate

```
crates/
└── flowsight-llm/                    # 新 crate: LLM 集成
    ├── Cargo.toml
    └── src/
        ├── lib.rs                    # 公开 API
        ├── provider.rs               # Provider trait 定义
        ├── providers/
        │   ├── mod.rs
        │   ├── openai.rs             # OpenAI 兼容 API (GPT-4o, DeepSeek 等)
        │   ├── anthropic.rs          # Anthropic Claude API
        │   ├── ollama.rs             # Ollama 本地模型
        │   ├── lmstudio.rs           # LM Studio 本地模型
        │   └── custom.rs             # 自定义 HTTP 端点
        ├── context_builder.rs        # 分析结果 → LLM prompt 构建
        ├── config.rs                 # Provider 配置 + API key 管理
        ├── streaming.rs              # SSE 流式输出
        └── feedback.rs               # DPO 反馈收集
```

### 8.3 Provider 接口

```rust
/// 统一的 LLM Provider 接口
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider 名称
    fn name(&self) -> &str;

    /// 单次完成
    async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse>;

    /// 流式完成 (SSE)
    async fn stream(
        &self,
        request: &CompletionRequest
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>>;

    /// 列出可用模型
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    /// 检查连接
    async fn health_check(&self) -> Result<bool>;
}

/// 与具体 provider 无关的请求格式
pub struct CompletionRequest {
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub stop_sequences: Vec<String>,
}

pub struct Message {
    pub role: Role,          // System / User / Assistant
    pub content: String,
}

pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    pub usage: TokenUsage,
}

pub struct StreamChunk {
    pub delta: String,       // 增量文本
    pub done: bool,          // 是否完成
}
```

### 8.4 支持的 Provider

| Provider | 接口协议 | 用途 | 配置 |
|----------|---------|------|------|
| **OpenAI** | OpenAI API v1 | GPT-4o, GPT-4-turbo | api_key + model |
| **Anthropic** | Messages API | Claude Sonnet/Opus | api_key + model |
| **Ollama** | Ollama REST API | 本地模型 (Llama, Mistral, 自训练) | endpoint + model |
| **LM Studio** | OpenAI 兼容 | 本地模型 (GUI 管理) | endpoint + model |
| **Custom** | OpenAI 兼容 | 任意兼容端点 (DeepSeek, 自部署) | endpoint + api_key + model |

### 8.5 LLM 上下文构建器（核心组件）

不是把源码直接给 LLM，而是把 **分析结果结构化后** 作为上下文：

```rust
/// 智能上下文构建
pub struct ContextBuilder {
    analysis: AnalysisResult,
    cfg: Option<ControlFlowGraph>,
    index: Option<IndexDb>,
    knowledge: KnowledgeBase,
}

impl ContextBuilder {
    /// 构建 LLM 上下文
    pub fn build(&self, file: &Path, function: &str) -> LlmContext {
        LlmContext {
            // 源代码片段
            source_code: self.extract_function_source(file, function),

            // 结构化分析结果 (Phase 1 的 CFG 输出)
            flow_tree: self.analysis.flow_tree_for(function),
            cfg_summary: self.cfg.as_ref().map(|c| c.summarize()),
            error_paths: self.cfg.as_ref().map(|c| c.error_paths()),

            // 异步和回调信息
            async_bindings: self.analysis.async_bindings_for(function),
            callbacks: self.analysis.callbacks_for(function),

            // 内核模式
            patterns: self.analysis.patterns_for(function),
            context_changes: self.cfg.as_ref().map(|c| c.context_changes()),

            // 知识库上下文
            kb_context: self.knowledge.match_function(function),
            subsystem: self.knowledge.detect_subsystem(file),

            // 跨文件信息 (Phase 2)
            cross_file_callers: self.index.as_ref()
                .map(|idx| idx.query_callers(function)),
            cross_file_callees: self.index.as_ref()
                .map(|idx| idx.query_callees(function)),
        }
    }

    /// PageRank 排序上下文元素 (借鉴 Aider)
    /// 按重要性排序，优先放入 token 预算内
    pub fn rank_context_elements(&self, budget: usize) -> Vec<ContextElement> {
        // 1. 构建符号依赖图
        // 2. 计算 PageRank
        // 3. 按重要性排序
        // 4. 在 token 预算内选择最重要的元素
        todo!()
    }
}
```

### 8.6 CLI 新命令

```bash
# 自然语言查询
flowsight ask "这个驱动的 probe 函数有几条错误处理路径？"
flowsight ask "schedule_work 注册的回调在什么上下文执行？"
flowsight ask "从 sys_read 到 ext4_readpage 经过哪些关键函数？"

# 指定 provider
flowsight ask --provider claude "解释这个函数的执行流"
flowsight ask --provider ollama --model flowsight-kernel "分析这个驱动的生命周期"
flowsight ask --provider openai --model gpt-4o "这段代码有什么潜在 bug？"

# AI 驱动的分析命令
flowsight explain <file> <function>           # LLM 解释执行流
flowsight explain <file> <function> --deep    # 深度解释（含跨文件上下文）
flowsight review <file>                       # AI 代码审查
flowsight suggest <file> <function>           # 改进建议

# 本地内核专家模型
flowsight ask --provider ollama --model flowsight-kernel "..."

# Provider 管理
flowsight config set llm.default_provider claude
flowsight config set llm.providers.claude.api_key_env ANTHROPIC_API_KEY
flowsight config set llm.providers.ollama.endpoint http://localhost:11434
flowsight config set llm.providers.ollama.model flowsight-kernel
flowsight llm list-providers                  # 列出配置的 providers
flowsight llm test                            # 测试连接
```

### 8.7 配置文件扩展

`.flowsight.toml` 新增 `[llm]` 段：

```toml
[llm]
default_provider = "claude"
temperature = 0.3                      # 技术分析偏低温度
max_tokens = 4096
stream = true                          # 默认流式输出

[llm.providers.claude]
api_key_env = "ANTHROPIC_API_KEY"      # 不存明文，用环境变量
model = "claude-sonnet-4-20250514"

[llm.providers.openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o"
base_url = ""                          # 空 = 默认 OpenAI 端点

[llm.providers.deepseek]
api_key_env = "DEEPSEEK_API_KEY"
model = "deepseek-chat"
base_url = "https://api.deepseek.com"  # OpenAI 兼容端点

[llm.providers.ollama]
endpoint = "http://localhost:11434"
model = "flowsight-kernel"             # 本地训练的内核专家模型

[llm.providers.custom]
endpoint = "http://my-server:8080/v1"
api_key_env = "CUSTOM_API_KEY"
model = "my-kernel-model"
```

### 8.8 REPL 增强

```
flowsight> ask 这个函数的错误处理模式是什么？

分析中... (使用 claude / claude-sonnet-4-20250514)

fsl_udc_probe() 使用典型的 Linux 内核 goto 链式错误处理模式：

1. 资源获取阶段按顺序执行 platform_get_resource → devm_ioremap → ...
2. 每步检查返回值，失败时 goto 到对应清理标签
3. 清理标签按反序排列 (err_free_irq → err_unmap → err_release)，
   确保只释放已成功获取的资源

该模式在 Linux 内核中称为 "centralized exit" 或 "goto chain cleanup"。

flowsight> set provider ollama
flowsight> set model flowsight-kernel
已切换到 ollama / flowsight-kernel

flowsight> ask 同样的问题

分析中... (使用 ollama / flowsight-kernel)
...
```

### 8.9 MCP Server 模式（可选）

未来可将 FlowSight 作为 MCP server 暴露给 Claude Code 等 AI 工具：

```bash
flowsight serve --mcp                    # 启动 MCP server
flowsight serve --mcp --port 3000        # 指定端口
```

暴露的 MCP tools：
- `analyze_file(path)` — 分析文件
- `trace_flow(path, function)` — 追踪执行流
- `search_symbol(name, index_path)` — 搜索符号
- `query_callers(function, index_path)` — 查询调用者
- `explain_function(path, function)` — 解释函数

---

## 9. Phase 4: 语言扩展 + DPO 闭环 (v0.7.0)

### 9.1 C++ 支持

```rust
// 新增 C++ 特有的 AST 节点处理
pub enum CppConstruct {
    Class { name: String, methods: Vec<Method>, bases: Vec<String> },
    Namespace { name: String, members: Vec<Symbol> },
    Template { params: Vec<String>, body: Box<CppConstruct> },
    VirtualMethod { class: String, method: String, is_override: bool },
    OperatorOverload { op: String },
    Lambda { captures: Vec<String>, body: FunctionBody },
}
```

借鉴 Semgrep 的「通用 AST 层」：C 和 C++ 共享大部分 CFG 构建逻辑，C++ 特有部分（虚函数、模板、lambda）做增量扩展。

### 9.2 Rust 支持

```rust
pub enum RustConstruct {
    Trait { name: String, methods: Vec<TraitMethod> },
    ImplBlock { type_name: String, trait_name: Option<String>, methods: Vec<Method> },
    AsyncFn { name: String, body: FunctionBody },
    Match { arms: Vec<MatchArm> },
    ResultHandling { ok_path: FlowBranch, err_path: FlowBranch },
}
```

Rust 的 `Result<T, E>` 和 `?` 操作符类似内核的 goto 错误处理，可以复用 Phase 1 的错误路径分析框架。

### 9.3 DPO 反馈闭环

```
用户提问 → FlowSight 分析 + LLM 生成回答 → 用户评分/纠正
                                                    │
                                                    ▼
                                            自动生成 DPO 训练对
                                            chosen = 用户认可的回答
                                            rejected = 被纠正的回答
                                                    │
                                                    ▼
                                            累积到训练数据集
                                            定期微调本地模型
```

```bash
# 反馈命令
flowsight feedback --good          # 标记上一次回答为好
flowsight feedback --bad "原因"    # 标记为差并说明原因
flowsight feedback export          # 导出 DPO 训练对
flowsight feedback stats           # 反馈统计
```

### 9.4 训练数据质量提升

Phase 1-2 完成后，训练数据质量显著提升：

| 维度 | v0.3.0 (当前) | v0.7.0 (目标) |
|------|-------------|-------------|
| 执行流准确性 | 仅调用列表 | CFG 感知 + 分支标注 |
| 错误路径 | 不识别 | 完整 goto chain + return -EXXX |
| 宏理解 | 当作函数调用 | 语义分类（注册/声明/迭代器...） |
| 跨文件上下文 | 无 | 完整调用链 + 子系统关联 |
| 执行上下文 | 不跟踪 | 原子/进程/中断上下文变化 |
| DPO 反馈 | 自动生成差答案 | 真实用户纠正 |

---

## 10. 开发规范与约束

### 10.1 技术约束

- **语言**: Rust，`#![forbid(unsafe_code)]`
- **解析器**: 继续使用 tree-sitter（不切换到 libclang/LLVM）
- **宏处理**: 查表法 + 知识库 YAML 扩展（不做完整预处理）
- **异步运行时**: Tokio（用于 LLM API 调用）
- **数据库**: SQLite（索引）
- **测试**: 使用真实 Linux 内核代码 (`/Users/sky/linux-kernel/linux`)
- **CI**: GitHub Actions，4 平台交叉编译

### 10.2 代码风格

- 文件 < 800 行，函数 < 50 行
- 每个命令独立文件
- 不可变数据模式优先
- 所有公开 API 有文档注释
- `cargo clippy` 和 `cargo fmt` 必须通过

### 10.3 测试要求

- 每个新 crate 需要单元测试
- CLI 命令需要集成测试（在 `cli/tests/`）
- 使用真实内核文件作为测试夹具
- 优先测试 ARM32 平台代码 (`arch/arm/mach-imx/`)

### 10.4 配色规范

CLI 终端输出使用低饱和度色板，禁止高亮 Cyan/Green/Blue/Yellow：

| 用途 | 名称 | RGB |
|------|------|-----|
| Logo | 钢蓝 | (130, 160, 190) |
| 标题 | 灰绿 sage | (140, 185, 165) |
| 成功 | 淡绿 | (130, 175, 140) |
| 文件名 | 薰衣草灰 | (155, 160, 185) |
| 提示符 | 柔绿 | (150, 175, 155) |
| 错误 | 烟粉红 | (195, 120, 120) |
| 标题栏 | 沙色 | (190, 170, 130) |
| 暗提示 | 暖灰 | (110, 115, 120) |

---

## 11. 附录

### A. 参考文献

#### 论文
- **IRIS** (2024): LLM + 静态分析漏洞检测 — [arxiv.org/abs/2405.17238](https://arxiv.org/abs/2405.17238)
- **LLMxCPG** (USENIX Security 2025): CPG 引导 LLM — [arxiv.org/html/2507.16585v1](https://arxiv.org/html/2507.16585v1)
- **COMEX** (ASE 2023, IBM): tree-sitter CFG/DFG — [arxiv.org/pdf/2307.04693](https://arxiv.org/pdf/2307.04693)
- **Coccinelle 10 Years** (USENIX ATC'18): [usenix.org/system/files/conference/atc18/atc18-lawall.pdf](https://www.usenix.org/system/files/conference/atc18/atc18-lawall.pdf)

#### 开源项目
- **tree-climber** (CFG from tree-sitter): [github.com/bstee615/tree-climber](https://github.com/bstee615/tree-climber)
- **stack-graphs** (GitHub, Rust): [github.com/github/stack-graphs](https://github.com/github/stack-graphs)
- **weggli** (Google P0, Rust): [github.com/weggli-rs/weggli](https://github.com/weggli-rs/weggli)
- **Joern** (CPG): [github.com/joernio/joern](https://github.com/joernio/joern)
- **IBM COMEX**: [github.com/IBM/tree-sitter-codeviews](https://github.com/IBM/tree-sitter-codeviews)
- **Aider**: [github.com/Aider-AI/aider](https://github.com/Aider-AI/aider)
- **MCPtrace**: [github.com/eunomia-bpf/MCPtrace](https://github.com/eunomia-bpf/MCPtrace)
- **SVF**: [github.com/SVF-tools/SVF](https://github.com/SVF-tools/SVF)
- **Semgrep**: [github.com/semgrep/semgrep](https://github.com/semgrep/semgrep)
- **uftrace**: [github.com/namhyung/uftrace](https://github.com/namhyung/uftrace)
- **tree-sitter-graph**: [github.com/tree-sitter/tree-sitter-graph](https://github.com/tree-sitter/tree-sitter-graph)
- **rust-code-analysis** (Mozilla): [github.com/mozilla/rust-code-analysis](https://github.com/mozilla/rust-code-analysis)

#### 内核工具
- **Smatch**: [lwn.net/Articles/1023646](https://lwn.net/Articles/1023646/) | [Oracle blog](https://blogs.oracle.com/linux/post/smatch-static-analysis-tool-overview-by-dan-carpenter)
- **Coccinelle**: [github.com/coccinelle/coccinelle](https://github.com/coccinelle/coccinelle)
- **Sparse**: kernel.org
- **Coverity**: [scan.coverity.com/projects/linux-next-weekly-scan](https://scan.coverity.com/projects/linux-next-weekly-scan)

### B. 术语表

| 术语 | 含义 |
|------|------|
| **CFG** | Control Flow Graph — 控制流图，表示程序中所有可能的执行路径 |
| **CPG** | Code Property Graph — 代码属性图，AST + CFG + PDG 的合并图 |
| **PDG** | Program Dependence Graph — 程序依赖图 |
| **DFG** | Data Flow Graph — 数据流图 |
| **SSA** | Static Single Assignment — 静态单赋值形式 |
| **SFT** | Supervised Fine-Tuning — 监督微调 |
| **DPO** | Direct Preference Optimization — 直接偏好优化 |
| **ChatML** | Chat Markup Language — OpenAI 对话训练格式 |
| **MCP** | Model Context Protocol — Anthropic 的模型上下文协议 |
| **REPL** | Read-Eval-Print Loop — 交互式命令行 |
| **RCU** | Read-Copy-Update — Linux 内核的无锁同步机制 |
| **SmPL** | Semantic Patch Language — Coccinelle 的语义补丁语言 |

### C. 文档版本历史

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-03-20 | v1.0 | 初始版本：完整技术评审 + 路线图 |
