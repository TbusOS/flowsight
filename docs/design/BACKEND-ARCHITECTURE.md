# FlowSight 后端架构设计

> **版本**: 1.0.0
> **最后更新**: 2025-01-24
> **作者**: FlowSight Team

## 1. 概述

FlowSight 是一个基于 Rust 的代码分析引擎，用于在不运行代码的情况下精准预测函数执行流程。本文档详细描述其后端架构设计，涵盖工作空间结构、crate 职责划分、核心数据结构、模块间接口以及错误处理策略。

### 1.1 设计目标

- **高可靠性**: 优先保证数据完整性和分析准确性
- **可观测性**: 内置日志、指标和监控能力
- **可扩展性**: 支持新增分析模块和知识库
- **模块化**: 各模块职责清晰，依赖关系明确
- **类型安全**: 充分利用 Rust 的类型系统保证正确性

### 1.2 技术栈

| 组件 | 技术选型 | 版本要求 |
|------|----------|----------|
| 编程语言 | Rust | 1.75+ |
| LLVM IR 解析 | inkwell | LLVM 17+ |
| 源码解析 | Tree-sitter | 0.22+ |
| 符号执行 | KLEE | 最新版 |
| 关系型存储 | SQLite | bundled |
| 键值存储 | Sled | 0.34+ |
| 异步运行时 | Tokio | 1.35+ |

---

## 2. Cargo Workspace 结构

### 2.1 工作空间概览

```
flowsight/
├── Cargo.toml                    # Workspace 根配置
├── crates/                       # 核心 crate 集合
│   ├── flowsight-core/          # 核心类型与接口定义
│   ├── flowsight-parser/        # 源码解析器
│   ├── flowsight-llvm/          # LLVM IR 解析器
│   ├── flowsight-analysis/      # 静态分析引擎
│   ├── flowsight-symbolic/      # 符号执行引擎 (KLEE)
│   ├── flowsight-knowledge/     # 知识库系统
│   ├── flowsight-index/         # 符号索引系统
│   ├── flowsight-query/         # 查询引擎
│   ├── flowsight-ai/            # AI 辅助分析
│   ├── flowsight-learning/      # 机器学习模块
│   └── flowsight-cli/           # CLI 工具
├── app/                         # 前端应用 (Tauri)
├── knowledge/                   # 知识库数据文件
├── docs/                        # 文档
└── tests/                       # 集成测试
```

### 2.2 Cargo.toml 配置

```toml
[workspace]
resolver = "2"
members = [
    "crates/flowsight-core",
    "crates/flowsight-parser",
    "crates/flowsight-llvm",
    "crates/flowsight-analysis",
    "crates/flowsight-symbolic",
    "crates/flowsight-knowledge",
    "crates/flowsight-index",
    "crates/flowsight-query",
    "crates/flowsight-ai",
    "crates/flowsight-learning",
    "crates/flowsight-cli",
    "app/src-tauri",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["FlowSight Team"]
license = "MIT"
repository = "https://github.com/user/flowsight"
description = "A cross-platform IDE for visualizing code execution flow"

[workspace.dependencies]
# Internal crates (defined below)

# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Parsing
tree-sitter = "0.22"
tree-sitter-c = "0.21"

# Database
rusqlite = { version = "0.31", features = ["bundled"] }
sled = "0.34"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# CLI
clap = { version = "4.4", features = ["derive"] }

# Regex
regex = "1.10"

# Path handling
walkdir = "2.4"
globset = "0.4"

# Parallelism
rayon = "1.8"
```

### 2.3 依赖关系图

```
flowsight-cli
    │
    └── flowsight-query
            │
            ├── flowsight-analysis
            │       │
            │       ├── flowsight-llvm
            │       │       │
            │       │       └── flowsight-core
            │       │
            │       ├── flowsight-knowledge
            │       │
            │       └── flowsight-symbolic
            │               │
            │               └── flowsight-core
            │
            ├── flowsight-index
            │       │
            │       └── flowsight-parser
            │               │
            │               └── flowsight-core
            │
            └── flowsight-ai
                    │
                    └── flowsight-core
```

---

## 3. Crate 职责划分

### 3.1 flowsight-core

**职责**: 定义核心类型、错误类型和共享接口。

**对外提供**:
- `Error` / `Result<T>` - 统一的错误处理类型
- `Location` - 源码位置表示
- `FunctionDef` / `StructDef` / `Parameter` - 核心数据结构
- `CallEdge` / `CallType` / `Confidence` - 调用图相关类型
- `AsyncBinding` / `AsyncMechanism` / `ExecutionContext` - 异步机制类型
- `FlowNode` / `FlowNodeType` - 执行流可视化类型

**模块结构**:
```
flowsight-core/src/
├── lib.rs              # 主入口，导出所有公共类型
├── error.rs            # 错误类型定义
├── location.rs         # 位置类型定义
├── types.rs            # 核心数据类型定义
└── config.rs           # 配置类型定义
```

### 3.2 flowsight-parser

**职责**: 源码解析，使用 Tree-sitter 进行快速增量解析。

**核心功能**:
- 增量解析 C/C++ 源码
- AST 提取与遍历
- 函数定义识别
- 结构体定义解析
- 并行文件解析

**模块结构**:
```
flowsight-parser/src/
├── lib.rs              # 主入口，Parser trait 定义
├── treesitter/         # Tree-sitter 解析器实现
├── ast/                # AST 类型与遍历
├── cache/              # 解析结果缓存 (LRU)
├── parallel/           # 并行解析
├── preprocessor/       # 预处理器集成
└── tests/
```

**关键接口**:
```rust
pub trait Parser: Send + Sync {
    fn parse(&self, source: &str, filename: &str) -> Result<ParseResult>;
    fn parse_file(&self, path: &Path) -> Result<ParseResult>;
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
}
```

### 3.3 flowsight-llvm

**职责**: LLVM IR 解析，提取类型信息和精确的指针信息。

**核心功能**:
- 解析 `.bc` (bitcode) 和 `.ll` (text IR) 文件
- 提取函数定义和参数类型
- 提取函数调用关系
- 提取结构体和类型信息
- 构建基本块和控制流图

**模块结构**:
```
flowsight-llvm/src/
├── lib.rs              # 主入口，LlvmParser
├── ir_parser/          # IR 解析器实现
└── types.rs            # IR 类型定义
```

**关键类型**:
```rust
pub struct LlvmParser {
    knowledge_base: Option<ir_parser::KnowledgeBase>,
}

pub struct IrParseResult {
    pub functions: HashMap<String, IrFunction>,
    pub calls: Vec<IrCall>,
    pub types: HashMap<String, IrType>,
    pub module_name: String,
    pub source_files: Vec<String>,
}
```

### 3.4 flowsight-analysis

**职责**: 静态分析引擎，构建调用图和分析控制流。

**核心功能**:
- 异步机制追踪 (WorkQueue, Timer, Interrupt)
- 函数指针解析
- 回调函数识别
- 调用图构建
- 场景化符号执行
- 表达式求值
- 数据流分析
- 结果分类 (Certain/Possible/Unknown)

**模块结构**:
```
flowsight-analysis/src/
├── lib.rs              # 主入口，Analyzer
├── async_tracker.rs    # 异步机制追踪
├── callback.rs         # 回调分析
├── callgraph.rs        # 调用图构建
├── classification.rs   # 结果分类
├── constraint.rs       # 约束求解
├── evaluator.rs        # 表达式求值
├── funcptr.rs          # 函数指针解析
├── learning.rs         # 学习模块
├── pointer.rs          # 指针分析
├── propagation.rs      # 数据流传播
├── scenario.rs         # 场景执行
└── types.rs            # 分析类型
```

### 3.5 flowsight-symbolic

**职责**: 符号执行引擎，集成 KLEE 进行路径探索。

**核心功能**:
- KLEE 执行器管理
- 路径探索与约束生成
- 测试用例生成
- 约束翻译 (AI 辅助)

**模块结构**:
```
flowsight-symbolic/src/
├── lib.rs              # 主入口
├── executor/           # KLEE 执行器
├── path/               # 执行路径
└── constraint/         # 符号约束
```

### 3.6 flowsight-knowledge

**职责**: 知识库系统，加载 YAML 知识库文件并匹配调用模式。

**核心功能**:
- 框架模式定义 (USB driver, file_operations, etc.)
- 异步模式定义 (WorkQueue, Timer, etc.)
- 内核 API 信息
- **完整的内核调用链注入**

**核心概念**:
- `KnowledgeBase` - 知识库主类型
- `Framework` - 框架定义 (如 USB driver)
- `FrameworkCallback` - 框架回调 (如 probe, disconnect)
- `CallChain` - **完整的内核调用链**
- `AsyncPattern` - 异步模式定义
- `AsyncTimeline` - 异步时间线关系
- `KernelApi` - 内核 API 信息

**模块结构**:
```
flowsight-knowledge/src/
└── lib.rs              # 主入口，所有知识定义
```

### 3.7 flowsight-index

**职责**: 符号索引系统，提供持久化存储和增量索引。

**核心功能**:
- 符号持久化存储 (SQLite)
- 文件版本追踪
- 增量索引
- 批量索引

**模块结构**:
```
flowsight-index/src/
├── lib.rs              # 主入口
├── batch_indexer.rs    # 批量索引
├── file_tracker.rs     # 文件版本追踪
├── storage.rs          # SQLite 存储
└── tree_cache.rs       # 语法树缓存
```

### 3.8 flowsight-query

**职责**: 查询引擎，支持对分析结果进行查询。

**核心功能**:
- 执行流查询
- 调用路径查询
- 函数关系查询

### 3.9 flowsight-ai

**职责**: AI 辅助分析，提供自然语言解释和智能推理。

**核心功能**:
- 约束条件自然语言翻译
- 执行条件解释
- 未知路径推理

### 3.10 flowsight-learning

**职责**: 机器学习模块，从用户反馈中学习。

**核心功能**:
- 函数指针目标学习
- 回调匹配学习
- 准确度提升

### 3.11 flowsight-cli

**职责**: 命令行工具，提供 CLI 接口。

---

## 4. 核心数据结构

### 4.1 执行流核心类型

#### FunctionDef - 函数定义

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    /// 函数名
    pub name: String,
    /// 返回类型
    pub return_type: String,
    /// 参数列表
    pub params: Vec<Parameter>,
    /// 源码位置
    pub location: Option<Location>,
    /// 直接调用的函数
    pub calls: Vec<String>,
    /// 调用此函数的函数
    pub called_by: Vec<String>,
    /// 是否为回调函数
    pub is_callback: bool,
    /// 回调上下文 (如 "usb_driver.probe")
    pub callback_context: Option<String>,
    /// 属性列表 (static, inline, __init 等)
    pub attributes: Vec<String>,
}
```

#### CallEdge - 调用边

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    /// 调用者函数
    pub caller: String,
    /// 被调用函数
    pub callee: String,
    /// 调用位置
    pub location: Option<Location>,
    /// 调用类型
    pub call_type: CallType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallType {
    /// 直接函数调用
    Direct,
    /// 通过函数指针的间接调用
    Indirect { confidence: Confidence },
    /// 异步调用 (WorkQueue, Timer 等)
    Async { mechanism: AsyncMechanism },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
}
```

#### FlowNode - 执行流节点

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    /// 唯一 ID
    pub id: String,
    /// 函数名
    pub name: String,
    /// 显示名称 (带装饰)
    pub display_name: String,
    /// 位置
    pub location: Option<Location>,
    /// 节点类型
    pub node_type: FlowNodeType,
    /// 子节点
    pub children: Vec<FlowNode>,
    /// 描述
    pub description: Option<String>,
    /// 调用置信度
    pub confidence: Option<CallConfidence>,
    /// 执行上下文 (process/softirq/hardirq)
    pub execution_context: Option<ExecutionContext>,
    /// 是否可睡眠
    pub can_sleep: Option<bool>,
    /// 源文件路径 (内核函数)
    pub source_file: Option<String>,
    /// 是否是内核内部调用链
    pub is_kernel_internal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowNodeType {
    /// 普通函数调用
    Function,
    /// 入口点 (回调)
    EntryPoint,
    /// 异步回调
    AsyncCallback { mechanism: AsyncMechanism },
    /// 内核 API
    KernelApi,
    /// 外部函数
    External,
}
```

### 4.2 知识库核心类型

#### CallChain - 完整的内核调用链

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallChain {
    /// 调用链名称
    pub name: String,
    /// 触发源头
    pub trigger_source: String,
    /// 调用链节点 (从触发源到用户代码)
    pub nodes: Vec<CallChainNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallChainNode {
    /// 函数名
    pub function: String,
    /// 所属文件
    pub file: Option<String>,
    /// 执行上下文
    pub context: ExecutionContext,
    /// 节点说明
    pub description: Option<String>,
    /// 是否是用户代码入口点
    pub is_user_entry: bool,
}
```

#### Framework - 框架定义

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Framework {
    /// 框架描述
    pub description: String,
    /// 头文件
    pub header: Option<String>,
    /// 回调定义
    pub callbacks: HashMap<String, FrameworkCallback>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkCallback {
    /// 回调描述
    pub description: String,
    /// 触发时机
    pub trigger: String,
    /// 执行上下文
    pub context: ExecutionContext,
    /// 函数签名
    pub signature: Option<String>,
    /// 完整的内核调用链
    pub call_chain: Option<CallChain>,
}
```

### 4.3 LLVM IR 类型

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrFunction {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<IrParameter>,
    pub blocks: Vec<IrBasicBlock>,
    pub is_callback: bool,
    pub callback_context: Option<String>,
    pub execution_context: ExecutionContext,
    pub source_file: Option<String>,
    pub source_line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrBasicBlock {
    pub name: String,
    pub instructions: Vec<IrInstruction>,
    pub successors: Vec<String>,  // 后继基本块
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrCall {
    pub caller_function: String,
    pub callee: String,
    pub location: Option<Location>,
    pub is_direct: bool,
}
```

---

## 5. 模块间接口设计

### 5.1 数据流图

```
┌─────────────────────────────────────────────────────────────────────┐
│                          用户输入                                     │
└───────────────────────────────────┬─────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│  flowsight-cli / Tauri Backend                                      │
│  - 命令行参数解析                                                    │
│  - 配置文件加载                                                      │
└───────────────────────────────────┬─────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│  flowsight-query (查询引擎)                                         │
│  - 执行流查询                                                        │
│  - 调用路径查询                                                      │
└───────────────────────────────────┬─────────────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
        ┌──────────────────┐ ┌──────────────┐ ┌──────────────────┐
        │ flowsight-index  │ │ flowsight-ai │ │ flowsight-llvm   │
        │ (符号索引)        │ │ (AI 辅助)    │ │ (IR 解析)        │
        └────────┬─────────┘ └──────┬───────┘ └────────┬─────────┘
                 │                  │                  │
                 │                  │                  │
                 ▼                  ▼                  ▼
        ┌──────────────────────────────────────────────────────────┐
        │              flowsight-analysis (分析引擎)                │
        │  - 异步追踪      - 回调分析    - 函数指针解析             │
        │  - 调用图构建    - 数据流分析  - 约束求解                 │
        └────────────────────────────────┬─────────────────────────┘
                                         │
                                         ▼
        ┌──────────────────────────────────────────────────────────┐
        │            flowsight-knowledge (知识库)                   │
        │  - 框架模式匹配  - 调用链注入    - 内核 API 信息          │
        └────────────────────────────────┬─────────────────────────┘
                                         │
                                         ▼
        ┌──────────────────────────────────────────────────────────┐
        │           flowsight-symbolic (符号执行)                   │
        │  - KLEE 执行      - 路径探索      - 约束生成             │
        └────────────────────────────────┬─────────────────────────┘
                                         │
                                         ▼
        ┌──────────────────────────────────────────────────────────┐
        │              flowsight-parser (源码解析)                  │
        │  - Tree-sitter 解析  - AST 提取    - 并行处理            │
        └────────────────────────────────┬─────────────────────────┘
                                         │
                                         ▼
        ┌──────────────────────────────────────────────────────────┐
        │                    输出结果                               │
        │  - 执行流树    - 调用图    - 路径条件    - 可视化数据     │
        └──────────────────────────────────────────────────────────┘
```

### 5.2 核心接口定义

#### 分析流水线接口

```rust
/// 分析流水线配置
#[derive(Debug, Clone)]
pub struct AnalysisPipelineConfig {
    /// 是否启用 LLVM IR 解析
    pub enable_llvm_analysis: bool,
    /// 是否启用符号执行
    pub enable_symbolic_execution: bool,
    /// 是否启用 AI 辅助
    pub enable_ai_assistance: bool,
    /// 最大分析超时 (秒)
    pub timeout_seconds: u64,
    /// 知识库路径
    pub knowledge_base_path: Option<PathBuf>,
}

/// 分析流水线
pub struct AnalysisPipeline {
    config: AnalysisPipelineConfig,
    parser: Box<dyn Parser>,
    llvm_parser: Option<LlvmParser>,
    analyzer: Analyzer,
    symbolic_executor: Option<KleeExecutor>,
    knowledge_base: KnowledgeBase,
    index_manager: IndexManager,
}

impl AnalysisPipeline {
    /// 创建新流水线
    pub fn new(config: AnalysisPipelineConfig) -> Result<Self> {
        // 加载知识库
        let knowledge_base = match config.knowledge_base_path {
            Some(ref path) if path.exists() => {
                KnowledgeBase::load_yaml(path)?
            }
            _ => KnowledgeBase::builtin(),
        };

        Ok(Self {
            config,
            parser: get_parser(),
            llvm_parser: Some(LlvmParser::with_knowledge_base(
                knowledge_base.clone(),
            )),
            analyzer: Analyzer::with_knowledge_base(knowledge_base.clone()),
            symbolic_executor: None,
            knowledge_base,
            index_manager: IndexManager::new(),
        })
    }

    /// 执行完整分析
    pub async fn analyze(
        &mut self,
        source: SourceInput,
    ) -> Result<AnalysisResult> {
        // 1. 解析源码
        let parse_result = self.parse_source(&source)?;

        // 2. 解析 LLVM IR (如果可用)
        let ir_result = if self.config.enable_llvm_analysis {
            self.parse_llvm_ir(&source)?
        } else {
            None
        };

        // 3. 索引符号
        self.index_symbols(&parse_result)?;

        // 4. 执行静态分析
        let mut analysis_result = self.analyzer.analyze(
            source.content(),
            &mut parse_result.clone(),
        )?;

        // 5. 执行符号执行 (如果启用)
        if self.config.enable_symbolic_execution {
            analysis_result = self.run_symbolic_execution(
                analysis_result,
                &parse_result,
                ir_result.as_ref(),
            )?;
        }

        Ok(analysis_result)
    }
}

/// 源码输入
pub enum SourceInput {
    /// 源码文件
    File(PathBuf),
    /// 源码内容
    Content { filename: String, content: String },
    /// LLVM IR 文件
    IrFile(PathBuf),
}

impl SourceInput {
    pub fn filename(&self) -> &str { /* ... */ }
    pub fn content(&self) -> Option<&str> { /* ... */ }
    pub fn path(&self) -> Option<&Path> { /* ... */ }
}
```

#### 知识库接口

```rust
impl KnowledgeBase {
    /// 获取框架回调的完整内核调用链
    pub fn get_callback_call_chain(
        &self,
        framework: &str,
        callback: &str,
    ) -> Option<&CallChain>;

    /// 获取异步模式的 handler 调用链
    pub fn get_async_handler_chain(&self, pattern_name: &str) -> Option<&CallChain>;

    /// 获取异步模式的时间线关系
    pub fn get_async_timeline(&self, pattern_name: &str) -> Option<&AsyncTimeline>;

    /// 识别回调函数
    pub fn identify_callback(
        &self,
        function_name: &str,
        code_context: &str,
    ) -> Option<(&str, &str, &FrameworkCallback)>;
}
```

#### 调用图构建接口

```rust
impl callgraph {
    /// 构建调用边
    pub fn build_call_edges(
        parse_result: &ParseResult,
        async_bindings: &[AsyncBinding],
    ) -> Vec<CallEdge>;

    /// 构建完整的执行流树 (含内核调用链注入)
    pub fn build_full_flow_tree(
        entry_point: &str,
        parse_result: &ParseResult,
        async_bindings: &[AsyncBinding],
        knowledge_base: &KnowledgeBase,
    ) -> Option<FlowNode>;
}
```

### 5.3 事件流接口

```rust
/// 分析事件
#[derive(Debug)]
pub enum AnalysisEvent {
    /// 开始解析
    ParsingStarted { source: String },
    /// 解析完成
    ParsingCompleted { functions_found: usize },
    /// 开始分析
    AnalysisStarted { entry_point: String },
    /// 分析进度
    AnalysisProgress { current: usize, total: usize },
    /// 发现异步绑定
    AsyncBindingFound { variable: String, handler: String },
    /// 发现函数指针调用
    FuncPtrResolved { caller: String, callee: String, confidence: Confidence },
    /// 内核调用链注入
    KernelCallChainInjected { trigger: String, chain_length: usize },
    /// 分析完成
    AnalysisCompleted { duration_ms: u64 },
    /// 错误发生
    ErrorOccurred { stage: String, error: String },
}

/// 分析事件监听器
pub trait AnalysisEventListener: Send {
    fn on_event(&self, event: &AnalysisEvent);
}

/// 带事件监听的分析器
pub struct ListeningAnalyzer<L: AnalysisEventListener> {
    analyzer: Analyzer,
    listeners: Vec<L>,
}

impl<L: AnalysisEventListener> ListeningAnalyzer<L> {
    pub fn add_listener(&mut self, listener: L) { /* ... */ }

    pub fn analyze_with_events(
        &mut self,
        source: &str,
        parse_result: &mut ParseResult,
    ) -> Result<AnalysisResult> {
        // 分析过程中通过 listeners 报告事件
    }
}
```

---

## 6. 错误处理策略

### 6.1 错误层次结构

```
┌─────────────────────────────────────────────────────────────────┐
│                        FlowSight Error                          │
└─────────────────────────────────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        ▼                       ▼                       ▼
┌───────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Core Error  │     │   Parser Error  │     │   KLEE Error    │
│  (thiserror)  │     │  (thiserror)    │     │  (thiserror)    │
└───────┬───────┘     └────────┬────────┘     └────────┬────────┘
        │                      │                       │
        ▼                      ▼                       ▼
┌───────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ - Io          │     │ - ParseFailed   │     │ - KleeNotFound  │
│ - Config      │     │ - Unsupported   │     │ - KleeFailed    │
│ - FileNotFound│     │ - AstError      │     │ - Timeout       │
│ - Other       │     │                 │     │ - NoPaths       │
└───────────────┘     └─────────────────┘     └─────────────────┘
```

### 6.2 核心错误定义 (flowsight-core)

```rust
use thiserror::Error;

/// FlowSight 核心错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 解析错误
    #[error("Parse error: {0}")]
    Parse(String),

    /// 索引错误
    #[error("Index error: {0}")]
    Index(String),

    /// 查询错误
    #[error("Query error: {0}")]
    Query(String),

    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(String),

    /// 文件未找到
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// 不支持的语言
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// LLVM 相关错误
    #[error("LLVM error: {0}")]
    Llvm(String),

    /// KLEE 相关错误
    #[error("KLEE error: {0}")]
    Klee(String),

    /// 知识库错误
    #[error("Knowledge base error: {0}")]
    Knowledge(String),

    /// 超时错误
    #[error("Analysis timeout: {0}")]
    Timeout(String),

    /// 内部错误
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result 类型别名
pub type Result<T> = std::result::Result<T, Error>;
```

### 6.3 模块特定错误

#### Parser 错误 (flowsight-parser)

```rust
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Parse failed at {location}: {message}")]
    ParseFailed {
        location: Location,
        message: String,
    },

    #[error("Unsupported language: {language}")]
    UnsupportedLanguage { language: String },

    #[error("Tree-sitter error: {0}")]
    TreeSitterError(#[from] tree_sitter::Error),

    #[error("Preprocessor error: {0}")]
    PreprocessorError(String),

    #[error("File too large: {size} bytes (max: {max_size})")]
    FileTooLarge { size: u64, max_size: u64 },
}
```

#### LLVM 错误 (flowsight-llvm)

```rust
#[derive(Error, Debug)]
pub enum LlvmError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid IR format: {0}")]
    InvalidFormat(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("LLVM error: {0}")]
    LlvmError(String),

    #[error("Type resolution failed: {0}")]
    TypeResolutionFailed(String),

    #[error("Metadata error: {0}")]
    MetadataError(String),
}
```

#### KLEE 错误 (flowsight-symbolic)

```rust
#[derive(Error, Debug)]
pub enum KleeError {
    #[error("KLEE not found: {path}")]
    KleeNotFound { path: String },

    #[error("KLEE execution failed: {output}")]
    KleeFailed { output: String },

    #[error("Analysis timeout after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("No paths found")]
    NoPaths,

    #[error("Constraint translation failed: {0}")]
    TranslationFailed(String),

    #[error("Invalid test case: {0}")]
    InvalidTestCase(String),
}
```

### 6.4 错误处理策略

#### 1. 结果类型与错误的分离

对于可能部分失败的分析，使用 `Result` 和 `AnalysisResult` 分离：

```rust
/// 分析结果 (可能包含部分错误信息)
#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    /// 成功的分析结果
    pub flow_trees: Vec<FlowNode>,
    pub call_edges: Vec<CallEdge>,
    pub async_bindings: Vec<AsyncBinding>,
    pub entry_points: Vec<String>,

    /// 非致命错误 (继续返回结果)
    pub warnings: Vec<AnalysisWarning>,
    /// 解析错误 (非致命)
    pub parse_errors: Vec<ParseErrorInfo>,
}

#[derive(Debug, Clone)]
pub struct AnalysisWarning {
    pub stage: String,
    pub message: String,
    pub location: Option<Location>,
}

#[derive(Debug, Clone)]
pub struct ParseErrorInfo {
    pub file: PathBuf,
    pub error: String,
    pub line: u32,
}
```

#### 2. 错误恢复策略

```rust
impl Analyzer {
    /// 尝试分析，允许部分失败
    pub fn analyze_with_recovery(
        &mut self,
        source: &str,
        parse_result: &mut ParseResult,
    ) -> (AnalysisResult, Vec<Error>) {
        let mut result = AnalysisResult::default();
        let mut errors = Vec::new();

        // 尝试构建调用图 (出错时不中断)
        if let Err(e) = self.build_callgraph(parse_result, &mut result) {
            errors.push(e);
            // 使用空的调用图继续
        }

        // 尝试追踪异步机制
        if let Err(e) = self.track_async_mechanisms(source, parse_result, &mut result) {
            errors.push(e);
        }

        // 尝试构建执行流
        if let Err(e) = self.build_flow_trees(&mut result) {
            errors.push(e);
        }

        (result, errors)
    }
}
```

#### 3. 错误传播与上下文增强

```rust
/// 带上下文的错误
#[derive(Debug)]
pub struct ContextualError {
    pub inner: Error,
    pub context: String,
    pub location: Option<Location>,
    pub suggestion: Option<String>,
}

impl std::fmt::Display for ContextualError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.context, self.inner)?;
        if let Some(ref loc) = self.location {
            write!(f, " (at {})", loc)?;
        }
        if let Some(ref sug) = self.suggestion {
            write!(f, "\nSuggestion: {}", sug)?;
        }
        Ok(())
    }
}

impl From<Error> for ContextualError {
    fn from(e: Error) -> Self {
        Self {
            inner: e,
            context: "Unknown context".to_string(),
            location: None,
            suggestion: None,
        }
    }
}

/// 增强错误上下文的工具
pub fn with_context<E: Into<Error>>(
    error: E,
    context: impl Into<String>,
) -> ContextualError {
    ContextualError {
        inner: error.into(),
        context: context.into(),
        location: None,
        suggestion: None,
    }
}
```

### 6.5 可观测性集成

#### 1. 结构化日志

```rust
use tracing::{info, warn, error, debug, instrument};

// 带有追踪的分析函数
#[instrument(level = "info", skip(self, source, parse_result))]
pub fn analyze(
    &mut self,
    source: &str,
    parse_result: &mut ParseResult,
) -> Result<AnalysisResult> {
    debug!("Starting analysis for source");

    let functions_count = parse_result.functions.len();
    debug!("Parsed {} functions", functions_count);

    // ... 分析逻辑 ...

    info!("Analysis completed: {} flow trees generated", result.flow_trees.len());
    Ok(result)
}
```

#### 2. 指标收集

```rust
/// 分析指标
#[derive(Debug, Default)]
pub struct AnalysisMetrics {
    pub parse_duration_ms: u64,
    pub analysis_duration_ms: u64,
    pub functions_analyzed: usize,
    pub edges_found: usize,
    pub async_bindings_found: usize,
    pub warnings_count: usize,
    pub errors_count: usize,
}

impl AnalysisMetrics {
    pub fn record(&mut self, event: &AnalysisEvent) {
        match event {
            AnalysisEvent::ParsingCompleted { functions_found } => {
                self.functions_analyzed = *functions_found;
            }
            AnalysisEvent::AsyncBindingFound { .. } => {
                self.async_bindings_found += 1;
            }
            AnalysisEvent::ErrorOccurred { .. } => {
                self.errors_count += 1;
            }
            _ => {}
        }
    }
}
```

---

## 7. 配置管理

### 7.1 配置结构

```rust
/// FlowSight 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 分析配置
    pub analysis: AnalysisConfig,
    /// 解析配置
    pub parsing: ParsingConfig,
    /// 知识库配置
    pub knowledge_base: KnowledgeBaseConfig,
    /// KLEE 配置
    pub klee: KleeConfig,
    /// 输出配置
    pub output: OutputConfig,
    /// 日志配置
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// 是否启用 LLVM IR 分析
    pub enable_llvm: bool,
    /// 是否启用符号执行
    pub enable_symbolic: bool,
    /// 是否启用 AI 辅助
    pub enable_ai: bool,
    /// 最大分析超时 (秒)
    pub timeout_seconds: u64,
    /// 并行分析线程数
    pub parallel_threads: usize,
    /// 置信度阈值
    pub confidence_threshold: Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseConfig {
    /// 知识库路径
    pub path: Option<PathBuf>,
    /// 是否加载内置知识库
    pub load_builtin: bool,
    /// 自定义框架定义
    pub custom_frameworks: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KleeConfig {
    /// KLEE 可执行文件路径
    pub klee_path: PathBuf,
    /// KLEE 参数
    pub klee_args: Vec<String>,
    /// 最大路径数
    pub max_paths: usize,
    /// 最大内存 (MB)
    pub max_memory_mb: usize,
    /// 超时时间 (秒)
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// 输出格式
    pub format: OutputFormat,
    /// 输出路径
    pub output_path: Option<PathBuf>,
    /// 是否包含源位置
    pub include_locations: bool,
    /// 是否包含置信度
    pub include_confidence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
   Yaml,
    Dot,
    Json5,
}
```

### 7.2 配置加载

```rust
impl Config {
    /// 从文件加载配置
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// 从字符串加载配置
    pub fn from_str(content: &str) -> Result<Self> {
        // 尝试多种格式
        if let Ok(config) = toml::from_str(content) {
            return Ok(config);
        }
        if let Ok(config) = serde_json::from_str(content) {
            return Ok(config);
        }
        if let Ok(config) = serde_yaml::from_str(content) {
            return Ok(config);
        }
        Err(Error::Config("Unable to parse config file".to_string()))
    }

    /// 获取默认配置
    pub fn default() -> Self {
        Self {
            analysis: AnalysisConfig {
                enable_llvm: true,
                enable_symbolic: false,
                enable_ai: true,
                timeout_seconds: 300,
                parallel_threads: num_cpus::get(),
                confidence_threshold: Confidence::Low,
            },
            parsing: ParsingConfig {
                max_file_size_mb: 10,
                cache_enabled: true,
                cache_size: 100,
            },
            knowledge_base: KnowledgeBaseConfig {
                path: None,
                load_builtin: true,
                custom_frameworks: vec![],
            },
            klee: KleeConfig {
                klee_path: PathBuf::from("klee"),
                klee_args: vec!["--max-sym-array-size=1024".to_string()],
                max_paths: 1000,
                max_memory_mb: 4096,
                timeout_seconds: 600,
            },
            output: OutputConfig {
                format: OutputFormat::Json,
                output_path: None,
                include_locations: true,
                include_confidence: true,
            },
            logging: LoggingConfig {
                level: tracing::Level::INFO,
                format: LoggingFormat::Compact,
            },
        }
    }
}
```

---

## 8. 总结

本文档描述了 FlowSight 后端的完整架构设计，包括：

1. **Cargo Workspace 结构**: 清晰的 crate 划分和依赖关系
2. **Crate 职责划分**: 每个 crate 职责明确，边界清晰
3. **核心数据结构**: 定义了函数定义、调用图、执行流等核心类型
4. **模块间接口设计**: 提供了流水线式的数据流和事件机制
5. **错误处理策略**: 层次化的错误类型和恢复机制

该架构设计遵循以下原则：
- **可靠性优先**: 错误处理和恢复机制完善
- **模块化设计**: 各模块职责清晰，便于维护和扩展
- **类型安全**: 充分利用 Rust 的类型系统
- **可观测性**: 内置日志和指标支持

未来可根据需要添加新的分析模块或扩展现有功能。
