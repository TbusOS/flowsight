# FlowSight 系统架构设计

> 版本: 1.0
> 更新日期: 2025-01-24
> 状态: 初稿

## 1. 概述

### 1.1 项目愿景

FlowSight 是一个函数执行流分析工具，旨在帮助开发者理解复杂代码库（特别是 Linux 内核等超大型项目）中函数的真正执行过程。传统 IDE 在面对异步机制、函数指针、回调注册等模式时会"迷路"，而 FlowSight 通过结合 LLVM IR 静态分析、知识库语义理解和按需符号执行，提供精准的执行流追踪能力。

### 1.2 核心设计原则

| 原则 | 描述 |
|------|------|
| **精准优先** | 不追求分析覆盖率，追求分析结果的准确性。宁可报告"未知"也不给出错误结论 |
| **分层分析** | 源代码层（语法分析）→ IR 层（类型/指针分析）→ 语义层（知识库）→ 约束层（符号执行） |
| **按需计算** | 复杂的符号执行只在用户需要精确分支条件时才触发 |
| **知识驱动** | 内置 Linux 内核 API 调用链知识库，100% 准确的语义信息 |
| **本地优先** | 所有分析在本地完成，数据不出用户机器 |

### 1.3 技术栈总览

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          FlowSight 技术栈                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                      前端 (Tauri 桌面应用)                       │    │
│  │  ┌──────────┬──────────┬──────────┬──────────┬──────────┐      │    │
│  │  │  React   │ TypeScript│ Monaco   │  @xyflow │  Zustand │      │    │
│  │  │    18    │    5.x   │  Editor  │  /react  │   5.x    │      │    │
│  │  └──────────┴──────────┴──────────┴──────────┴──────────┘      │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                    │                                     │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                       后端 (Rust 分析引擎)                       │    │
│  │  ┌──────────┬──────────┬──────────┬──────────┬──────────┐      │    │
│  │  │Tree-sitter│  inkwell │  KLEE   │  SQLite  │   Sled   │      │    │
│  │  │   0.22   │(LLVM 17) │ Symbolic │  0.31    │  0.34    │      │    │
│  │  │          │  Binding │ Execution│          │  Graph DB│      │    │
│  │  └──────────┴──────────┴──────────┴──────────┴──────────┘      │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                    │                                     │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                          知识库系统                              │    │
│  │  ┌─────────────────────────────────────────────────────────┐    │    │
│  │  │  YAML 格式 - Linux 内核 API 调用链、回调触发模式、执行上下文 │    │
│  │  └─────────────────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. 模块划分与职责

### 2.1 模块架构总览

```
flowsight/
├── app/                                      # 前端应用 (Tauri)
│   ├── src/
│   │   ├── components/                       # React 组件
│   │   │   ├── Editor/                       # 代码编辑器 (Monaco)
│   │   │   ├── FlowView/                     # 执行流可视化 (@xyflow/react)
│   │   │   ├── CallGraph/                    # 调用图面板
│   │   │   ├── Explorer/                     # 文件浏览器
│   │   │   ├── Outline/                      # 大纲视图
│   │   │   ├── CommandPalette/               # 命令面板
│   │   │   └── ...
│   │   ├── store/                            # Zustand 状态管理
│   │   │   ├── editorStore.ts                # 编辑器状态
│   │   │   ├── analysisStore.ts              # 分析结果状态
│   │   │   └── uiStore.ts                    # UI 状态
│   │   ├── utils/                            # 工具函数
│   │   └── types.ts                          # 类型定义
│   └── src-tauri/                            # Tauri 后端命令
│       ├── commands.rs                       # Tauri 命令实现
│       └── lib.rs                            # Rust FFI 接口
│
├── crates/                                   # Rust 核心模块
│   ├── flowsight-core/                       # 核心类型与接口
│   │   ├── config.rs                         # 配置类型
│   │   ├── error.rs                          # 错误类型
│   │   ├── location.rs                       # 位置信息
│   │   └── types.rs                          # 核心数据结构
│   │
│   ├── flowsight-parser/                     # 源代码解析器
│   │   ├── c_parser.rs                       # C 语言解析 (tree-sitter)
│   │   ├── ast.rs                            # AST 定义
│   │   └── types.rs                          # 解析结果类型
│   │
│   ├── flowsight-llvm/                       # LLVM IR 分析模块
│   │   ├── ir_parser.rs                      # IR 解析器
│   │   ├── types.rs                          # IR 类型定义
│   │   └── lib.rs                            # 公共接口
│   │
│   ├── flowsight-index/                      # 符号索引系统
│   │   ├── indexer.rs                        # 索引构建器
│   │   ├── symbol_table.rs                   # 符号表
│   │   └── storage.rs                        # 持久化存储
│   │
│   ├── flowsight-analysis/                   # 代码分析引擎
│   │   ├── async_tracker.rs                  # 异步机制追踪
│   │   ├── funcptr.rs                        # 函数指针解析
│   │   ├── callgraph.rs                      # 调用图构建
│   │   ├���─ constraint.rs                     # 约束传播
│   │   ├── evaluator.rs                      # 表达式求值
│   │   ├── scenario.rs                       # 场景分析
│   │   └── learning.rs                       # 用户学习反馈
│   │
│   ├── flowsight-knowledge/                  # 知识库系统
│   │   ├── loader.rs                         # YAML 加载器
│   │   ├── matcher.rs                        # 模式匹配器
│   │   └── lib.rs                            # 公共接口
│   │
│   ├── flowsight-symbolic/                   # 符号执行引擎
│   │   ├── executor.rs                       # KLEE 执行器
│   │   ├── solver.rs                         # 约束求解器
│   │   └── lib.rs                            # 公共接口
│   │
│   ├── flowsight-query/                      # 查询引擎
│   │   ├── parser.rs                         # 查询语法解析
│   │   ├── engine.rs                         # 查询执行
│   │   └── lib.rs                            # 公共接口
│   │
│   ├── flowsight-ai/                         # AI 辅助分析 (未来)
│   │   └── ...
│   │
│   ├── flowsight-learning/                   # 机器学习模块 (未来)
│   │   └── ...
│   │
│   └── flowsight-cli/                        # CLI 工具
│       └── main.rs
│
└── knowledge/                                # 知识库数据
    ├── languages/                            # 语言级模式定义
    │   └── c/
    ├── platforms/                            # 平台级框架定义
    │   └── linux-kernel/
    │       ├── core/                         # 核心机制
    │       │   ├── workqueue.yaml
    │       │   ├── timer.yaml
    │       │   ├── irq.yaml
    │       │   ├── rcu.yaml
    │       │   └── kthread.yaml
    │       ├── drivers/                      # 驱动框架
    │       │   ├── usb.yaml
    │       │   ├── char_dev.yaml
    │       │   └── platform.yaml
    │       └── fs/                           # 文件系统
    │           └── vfs.yaml
    └── schemas/                              # YAML Schema 定义
```

### 2.2 核心模块详解

#### 2.2.1 flowsight-core (核心类型模块)

**职责**: 定义整个系统的核心数据类型和接口。

**关键类型**:

```rust
// 核心数据结构定义

/// 执行上下文 - 描述代码运行的硬件/软件环境
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionContext {
    Process,          // 普通进程上下文
    Softirq,          // 软中断上下文
    Hardirq,          // 硬中断上下文
    Interrupt,        // 中断处理
    KernelThread,     // 内核线程
    RcUReadSide,      // RCU 读临界区
    UserSpace,        // 用户空间
}

/// 位置信息 - 源代码或 IR 中的位置
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub file: String,      // 文件路径
    pub line: u32,         // 行号
    pub column: u32,       // 列号
    pub ir_location: Option<IrLocation>,  // 可选的 IR 位置
}

/// IR 位置信息
#[derive(Debug, Clone, PartialEq)]
pub struct IrLocation {
    pub function: String,  // IR 函数名
    pub basic_block: String,  // 基本块 ID
    pub instruction: u32,  // 指令偏移
}

/// 函数定义
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<Parameter>,
    pub location: Option<Location>,
    pub is_callback: bool,
    pub callback_context: Option<String>,
    pub execution_context: ExecutionContext,
    pub attributes: Vec<String>,  // static, inline, etc.
}

/// 调用边
#[derive(Debug, Clone)]
pub struct CallEdge {
    pub caller: String,       // 调用者函数
    pub callee: String,       // 被调用函数
    pub location: Location,   // 调用位置
    pub confidence: Confidence,
    pub edge_type: CallEdgeType,
}

/// 置信度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    Certain,     // 100% 确定
    Possible,    // 可能的
    Unknown,     // 未知
}

/// 调用边类型
#[derive(Debug, Clone, PartialEq)]
pub enum CallEdgeType {
    DirectCall,      // 直接调用
    AsyncCallback,   // 异步回调
    FunctionPointer, // 函数指针
    VirtualCall,     // 虚函数调用
    KernelApi,       // 内核 API
}

/// 异步绑定 - 描述异步机制的注册和触发关系
#[derive(Debug, Clone)]
pub struct AsyncBinding {
    pub mechanism: AsyncMechanism,  // 异步机制类型
    pub variable: String,           // 变量名 (work_struct, timer, etc.)
    pub handler: String,            // 处理函数
    pub bind_location: Location,    // 注册位置
    pub trigger_location: Option<Location>,  // 触发位置 (如果可识别)
    pub trigger_conditions: Vec<TriggerCondition>,  // 触发条件
}

/// 异步机制类型
#[derive(Debug, Clone, PartialEq)]
pub enum AsyncMechanism {
    WorkQueue {
        work_struct: String,
        queue: Option<String>,
    },
    Timer {
        timer_name: String,
        timer_type: TimerType,
    },
    Tasklet {
        tasklet_name: String,
    },
    Softirq {
        type_name: String,
    },
    Irq {
        irq_name: String,
        flags: Option<String>,
    },
    Completion {
        completion_name: String,
    },
    Rcu {
        rcu_type: String,
    },
    Kthread {
        kthread_name: String,
    },
    Custom(String),  // 自定义机制
}

/// 触发条件
#[derive(Debug, Clone)]
pub struct TriggerCondition {
    pub description: String,  // 条件描述
    pub confidence: Confidence,
    pub source: ConditionSource,
}

/// 条件来源
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionSource {
    KnowledgeBase,     // 知识库
    StaticAnalysis,    // 静态分析
    SymbolicExecution, // 符号执行
    UserAnnotation,    // 用户注解
}

/// 执行流节点
#[derive(Debug, Clone)]
pub struct FlowNode {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub node_type: FlowNodeType,
    pub location: Option<Location>,
    pub children: Vec<FlowNode>,
    pub description: Option<String>,
    pub confidence: Option<CallConfidence>,
}

/// 节点类型
#[derive(Debug, Clone, PartialEq)]
pub enum FlowNodeType {
    Function,
    EntryPoint,
    AsyncCallback { mechanism: AsyncMechanism },
    KernelApi,
    External,
    Unknown,
}
```

#### 2.2.2 flowsight-llvm (LLVM IR 分析模块)

**职责**: 解析 LLVM IR (.bc/.ll 文件)，提取准确的类型和指针信息，补充源代码分析的不足。

**核心组件**:

```
┌─────────────────────────────────────────────────────────────────┐
│                      LLVM IR Parser                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐     │
│  │  BitcodeReader │  │   TypeParser   │  │ FunctionParser │     │
│  │  (.bc files)   │  │   (.ll files)  │  │                │     │
│  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘     │
│          │                   │                   │               │
│          └───────────────────┼───────────────────┘               │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    IR Analysis Engine                    │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐        │    │
│  │  │ Pointer     │ │ Call        │ │ Type        │        │    │
│  │  │ Analyzer    │ │ Resolver    │ │ Inferencer  │        │    │
│  │  └─────────────┘ └─────────────┘ └─────────────┘        │    │
│  └────────────────────────────┬──────────────────────────────┘    │
│                               ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Output Types                          │    │
│  │  IrFunction, IrCall, IrType, IrBasicBlock, IrInstruction │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**关键数据结构**:

```rust
/// LLVM IR 函数
#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub mangled_name: Option<String>,
    pub return_type: IrType,
    pub parameters: Vec<IrParameter>,
    pub blocks: Vec<IrBasicBlock>,
    pub is_declaration: bool,    // 是否只是声明
    pub linkage_type: LinkageType,
    pub visibility: Visibility,
    pub source_file: Option<String>,
    pub source_line: Option<u32>,
}

/// LLVM IR 基本块
#[derive(Debug, Clone)]
pub struct IrBasicBlock {
    pub id: String,
    pub name: String,
    pub instructions: Vec<IrInstruction>,
    pub predecessors: Vec<String>,   // 前驱基本块 ID
    pub successors: Vec<String>,     // 后继基本块 ID
    pub termiantor: Option<IrTerminator>,
}

/// LLVM IR 调用指令
#[derive(Debug, Clone)]
pub struct IrCall {
    pub instruction_id: String,
    pub caller_function: String,
    pub caller_basic_block: String,
    pub callee: String,           // 解析后的函数名
    pub callee_raw: String,       // 原始调用目标
    pub arguments: Vec<IrValue>,
    pub return_value: Option<IrValue>,
    pub is_indirect: bool,        // 是否是间接调用
    pub pointer_source: Option<PointerSource>,  // 指针来源
    pub source_location: Option<SourceLocation>,
}

/// LLVM IR 类型
#[derive(Debug, Clone)]
pub enum IrType {
    Void,
    Integer { bits: u32 },
    Float,
    Double,
    Pointer { pointee: Box<IrType>, address_space: u32 },
    Array { element: Box<IrType>, length: u64 },
    Vector { element: Box<IrType>, length: u32 },
    Struct { name: Option<String>, fields: Vec<IrType> },
    Function { return_type: Box<IrType>, params: Vec<IrType> },
    Opaque { name: String },  // 不透明类型
}

/// 指针来源分析结果
#[derive(Debug, Clone)]
pub struct PointerSource {
    pub source_type: PointerSourceType,
    pub source_value: String,
    pub analysis_confidence: Confidence,
    pub analysis_method: AnalysisMethod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerSourceType {
    ConstantAddress,      // 常量地址
    FunctionArgument,     // 函数参数
    GlobalVariable,       // 全局变量
    LocalAllocation,      // 局部分配
    LoadFromPointer,      // 从指针加载
    Bitcast,              // 类型转换
    Gep,                  // GetElementPtr
}

/// 分析方法
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnalysisMethod {
    TypeBased,            // 基于类型的分析
    ConstantPropagation,  // 常量传播
    AliasAnalysis,        // 别名分析
    UserAnnotation,       // 用户注解
}
```

**主要功能**:

| 功能 | 描述 | 准确性 |
|------|------|--------|
| 函数提取 | 从 .bc 文件提取所有函数定义 | 100% |
| 调用关系 | 解析 direct call 指令 | 100% |
| 间接调用分析 | 分析 function pointer 调用目标 | 高 (基于类型) |
| 类型信息 | 提取结构体、指针、数组类型 | 100% |
| 基本块分析 | 获取控制流图 (CFG) | 100% |
| 分支条件 | 提取 switch/branch 条件 | 100% |

#### 2.2.3 flowsight-knowledge (知识库系统)

**职责**: 存储和管理 Linux 内核 API 调用链知识，提供 100% 准确的语义信息。

**知识库结构**:

```
knowledge/
├── platforms/linux-kernel/
│   ├── core/
│   │   ├── workqueue.yaml       # 工作队列调用链
│   │   ├── timer.yaml           # 定时器调用链
│   │   ├── irq.yaml             # 中断处理调用链
│   │   ├── rcu.yaml             # RCU 机制调用链
│   │   ├── kthread.yaml         # 内核线程调用链
│   │   └── memory.yaml          # 内存管理调用链
│   ├── drivers/
│   │   ├── usb.yaml             # USB 驱动框架
│   │   ├── char_dev.yaml        # 字符设备
│   │   ├── platform.yaml        # 平台驱动
│   │   └── pci.yaml             # PCI 驱动
│   └── fs/
│       └── vfs.yaml             # VFS 文件系统
│
└── schemas/
    └── knowledge.schema.json    # YAML 格式规范
```

**知识库格式示例**:

```yaml
# workqueue.yaml - 工作队列调用链知识
workqueue:
  description: "Linux Kernel Work Queue Mechanism"

  bind_patterns:
    - name: INIT_WORK
      pattern: 'INIT_WORK\s*\(\s*&?(?P<work>\w+),\s*(?P<handler>\w+)\s*\)'
      parameters:
        work: "work_struct pointer"
        handler: "work handler function"

    - name: DECLARE_WORK
      pattern: 'DECLARE_WORK\s*\(\s*(?P<work>\w+),\s*(?P<handler>\w+)\s*\)'

  trigger_patterns:
    - name: schedule_work
      pattern: 'schedule_work\s*\(\s*&?(?P<work>\w+)\s*\)'

    - name: schedule_work_on
      pattern: 'schedule_work_on\s*\(\s*\w+,\s*&?(?P<work>\w+)\s*\)'

    - name: queue_work
      pattern: 'queue_work\s*\(\s*(?P<wq>\w+),\s*&?(?P<work>\w+)\s*\)'

  execution_chain:
    - trigger: "schedule_work() called"
      kernel_call: "insert_work()"
      kernel_call: "wake_up_process()"
      kernel_call: "worker_thread()"
      kernel_call: "process_one_work()"
      kernel_call: "worker->func (user handler)"

  context:
    execution_context: "Process"
    preemption: "may_sleep"

  examples:
    - code: |
        INIT_WORK(&dev->work, my_work_handler);
        schedule_work(&dev->work);
      flow:
        - "USB driver calls schedule_work()"
        - "Kernel schedules work to default workqueue"
        - "Worker thread processes work"
        - "my_work_handler() executes in process context"
```

```yaml
# irq.yaml - 中断处理调用链知识
irq:
  description: "Linux Kernel Interrupt Handling"

  registration_patterns:
    - name: request_irq
      pattern: 'request_irq\s*\(\s*(?P<irq>\w+),\s*(?P<handler>\w+),\s*(?P<flags>[^,]+),\s*(?P<devname>[^,]+),\s*(?P<dev_id>\w+)\s*\)'
      parameters:
        irq: "IRQ number or hwirq"
        handler: "interrupt handler function"
        flags: "IRQ flags (IRQF_SHARED, etc.)"
        devname: "device name"
        dev_id: "device identifier for shared IRQs"

  handler_types:
    top_half:
      - name: hardirq_ctx
        context: "Hardirq"
        description: "Runs with interrupts disabled"
        maximum_latency: "very low (microseconds)"

    bottom_half:
      - name: tasklet
        context: "Softirq"
        description: "Deferred from hardirq"

      - name: workqueue
        context: "Process"
        description: "Can sleep"

      - name: threaded_irq
        context: "KernelThread"
        description: "Runs in dedicated thread"

  execution_chain:
    - scenario: "Top half only"
      flow:
        - "Hardware interrupt fires"
        - "CPU jumps to interrupt vector"
        - "handle_irq() finds IRQ number"
        - "handler() executes (top half)"
        - "irq_exit() returns"

    - scenario: "Top half + tasklet"
      flow:
        - "Hardware interrupt fires"
        - "handle_irq() finds IRQ number"
        - "handler() calls tasklet_schedule()"
        - "irq_exit() schedules tasklet"
        - "Softirq handler runs tasklet"
        - "tasklet_action() calls tasklet func"

    - scenario: "Threaded IRQ"
      flow:
        - "Hardware interrupt fires"
        - "Kernel creates kernel thread for handler"
        - "Thread runs handler_threaded_irq()"
        - "handler() executes in thread context"
```

```yaml
# vfs.yaml - VFS 文件系统调用链
vfs:
  description: "Virtual File System Operations"

  file_operations:
    name: "struct file_operations"
    fields:
      - name: open
        pattern: '\.open\s*=\s*(?P<handler>\w+)'
        signature: "int (*open)(struct inode *, struct file *)"

      - name: read
        pattern: '\.read\s*=\s*(?P<handler>\w+)'
        signature: "ssize_t (*read)(struct file *, char __user *, size_t, loff_t *)"

      - name: write
        pattern: '\.write\s*=\s*(?P<handler>\w+)'
        signature: "ssize_t (*write)(struct file *, const char __user *, size_t, loff_t *)"

      - name: ioctl
        pattern: '\.unlocked_ioctl\s*=\s*(?P<handler>\w+)'
        signature: "long (*unlocked_ioctl)(struct file *, unsigned int, unsigned long)"

      - name: mmap
        pattern: '\.mmap\s*=\s*(?P<handler>\w+)'
        signature: "int (*mmap)(struct file *, struct vm_area_struct *)"

  operation_sequences:
    read_flow:
      - "User space calls read()"
      - "sys_read() system call"
      - "vfs_read()"
      - "fd->f_op->read()"
      - "filesystem specific read()"
      - "block device or character device read()"

    write_flow:
      - "User space calls write()"
      - "sys_write() system call"
      - "vfs_write()"
      - "fd->f_op->write()"
      - "filesystem specific write()"
```

#### 2.2.4 flowsight-analysis (代码分析引擎)

**职责**: 整合 LLVM IR 分析、知识库和静态分析，提供完整的执行流分析能力。

**核心组件架构**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Analysis Engine                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                      输入处理层                                  │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │    │
│  │  │  Source  │ │   LLVM   │ │ Knowledge│ │   User   │           │    │
│  │  │   Code   │ │  Module  │ │   Base   │ │Settings  │           │    │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘           │    │
│  │       │            │            │            │                   │    │
│  │       └────────────┴─────┬──────┴────────────┘                   │    │
│  │                          ▼                                        │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                    │                                     │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                      分析管道 (Pipeline)                         │    │
│  │                                                                     │    │
│  │  ┌─────────────┐                                                 │    │
│  │  │   Parser    │ ──► AST + 符号表                                │    │
│  │  └──────┬──────┘                                                 │    │
│  │         ▼                                                        │    │
│  │  ┌─────────────┐                                                 │    │
│  │  │ AsyncTracker│ ──► 异步绑定列表                                │    │
│  │  └──────┬──────┘                                                 │    │
│  │         ▼                                                        │    │
│  │  ┌─────────────┐                                                 │    │
│  │  │ FuncPtrResolver│ ──► 函数指针映射                             │    │
│  │  └──────┬──────┘                                                 │    │
│  │         ▼                                                        │    │
│  │  ┌─────────────┐                                                 │    │
│  │  │  CallGraph  │ ──► 调用图                                     │    │
│  │  │  Builder    │                                                 │    │
│  │  └──────┬──────┘                                                 │    │
│  │         ▼                                                        │    │
│  │  ┌─────────────┐                                                 │    │
│  │  │   Pointer   │ ──► 指针分析结果                               │    │
│  │  │   Analyzer  │                                                 │    │
│  │  └──────┬──────┘                                                 │    │
│  │         ▼                                                        │    │
│  │  ┌─────────────┐                                                 │    │
│  │  │  Constraint │ ──► 约束传播                                   │    │
│  │  │ Propagation │                                                 │    │
│  │  └──────┬──────┘                                                 │    │
│  │         ▼                                                        │    │
│  │  ┌─────────────┐                                                 │    │
│  │  │  Scenario   │ ──► 场景分析                                   │    │
│  │  │  Analyzer   │                                                 │    │
│  │  └──────┬──────┘                                                 │    │
│  │         ▼                                                        │    │
│  │  ┌─────────────┐                                                 │    │
│  │  │  Knowledge  │ ──► 知识库注入                                 │    │
│  │  │  Injector   │                                                 │    │
│  │  └──────┬──────┘                                                 │    │
│  │         ▼                                                        │    │
│  │  ┌─────────────┐                                                 │    │
│  │  │   Result    │ ──► 最终分析结果                               │    │
│  │  │  Classifier │                                                 │    │
│  │  └─────────────┘                                                 │    │
│  │                                                                     │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                    │                                     │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                        输出                                      │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │    │
│  ��  │FlowTrees │ │ CallEdges│ │ Async    │ │ Symbol   │           │    │
│  │  │          │ │          │ │Bindings  │ │Table     │           │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**核心分析器**:

```rust
/// 异步机制追踪器
pub struct AsyncTracker {
    /// 知识库实例
    knowledge_base: KnowledgeBase,
    /// 已识别的异步绑定
    bindings: Vec<AsyncBinding>,
}

impl AsyncTracker {
    /// 分析源代码中的异步机制
    pub fn analyze(&mut self, source: &str, functions: &HashMap<String, FunctionDef>) -> Vec<AsyncBinding> {
        // 1. 从知识库加载所有异步模式
        let patterns = self.knowledge_base.load_async_patterns();

        // 2. 对每种模式进行正则匹配
        for pattern in patterns {
            if let Some(matches) = self.match_pattern(source, &pattern) {
                for m in matches {
                    let binding = self.create_binding(&m, &pattern, functions);
                    self.bindings.push(binding);
                }
            }
        }

        self.bindings.clone()
    }
}

/// 函数指针解析器
pub struct FuncPtrResolver {
    /// 已知 ops 表定义
    known_ops: HashMap<String, OpsTable>,
    /// 已解析的函数指针
    resolved: HashMap<String, ResolvedCall>,
}

impl FuncPtrResolver {
    /// 分析 ops 表赋值
    pub fn analyze_ops_tables(&self, source: &str, functions: &HashMap<String, FunctionDef>) -> Vec<(String, String)> {
        let mut results = Vec::new();

        // 1. 匹配 struct file_operations 等定义
        for (ops_name, ops_def) in &self.known_ops {
            if let Some(assignments) = self.match_ops_assignment(source, ops_name, ops_def) {
                for (field, handler) in assignments {
                    // 验证 handler 确实存在
                    if functions.contains_key(&handler) {
                        results.push((format!("{}.{}", ops_name, field), handler));
                    }
                }
            }
        }

        results
    }
}

/// 场景分析器 - 分析特定执行场景
pub struct ScenarioAnalyzer {
    /// 符号执行引擎
    symbolic_executor: Option<SymbolicExecutor>,
    /// 约束求解器
    constraint_solver: ConstraintSolver,
    /// 场景缓存
    scenarios: HashMap<String, ScenarioResult>,
}

impl ScenarioAnalyzer {
    /// 分析特定分支条件
    pub fn analyze_branch(
        &mut self,
        branch: &BranchCondition,
        context: &AnalysisContext,
    ) -> Result<BranchResult> {
        match self.symbolic_executor {
            Some(ref mut executor) => {
                // 使用 KLEE 进行精确分析
                executor.analyze_branch(branch, context)
            }
            None => {
                // 使用启发式分析
                self.heuristic_analysis(branch, context)
            }
        }
    }
}
```

#### 2.2.5 flowsight-symbolic (符号执行引擎)

**职责**: 按需调用 KLEE 进行符号执行，提供精确的分支条件分析。

**架构设计**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Symbolic Execution Engine                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                       KLEE 集成层                                │    │
│  │                                                                  │    │
│  │  ┌────────────────┐    ┌────────────────┐    ┌────────────────┐ │    │
│  │  │  KLEE Binary   │    │  KLEE Config   │    │  Output Parser │ │    │
│  │  │  Executor      │    │  Generator     │    │                │ │    │
│  │  └────────┬───────┘    └───────┬────────┘    └───────┬────────┘ │    │
│  │           │                    │                     │          │    │
│  │           └────────────────────┼─────────────────────┘          │    │
│  │                                ▼                                 │    │
│  │  ┌─────────────────────────────────────────────────────────┐    │    │
│  │  │            KLEE Invocation Wrapper                      │    │    │
│  │  │  - Process management (spawn/terminate)                 │    │    │
│  │  │  - Output streaming and parsing                         │    │    │
│  │  │  - Timeout handling                                     │    │    │
│  │  │  - Error recovery                                       │    │    │
│  │  └─────────────────────────────────────────────────────────┘    │    │
│  │                                                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                    │                                     │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                      符号执行 API                                │    │
│  │                                                                  │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │    │
│  │  │  analyze_    │  │  evaluate_    │  │  find_       │          │    │
│  │  │  branch()    │  │  condition()  │  │  constraints()│          │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘          │    │
│  │                                                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                    │                                     │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                      模拟执行模式                                │    │
│  │                                                                  │    │
│  │  当 KLEE 不可用时，使用启发式分析作为后备:                       │    │
│  │  - 条件常量传播                                                 │    │
│  │  - 范围分析 (区间运算)                                           │    │
│  │  - 模式匹配 (常见条件模式)                                       │    │
│  │                                                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**接口设计**:

```rust
/// 符号执行引擎 trait
pub trait SymbolicExecutor {
    /// 分析分支条件
    fn analyze_branch(
        &mut self,
        branch: &BranchCondition,
        context: &ExecutionContext,
    ) -> Result<BranchResult>;

    /// 评估条件表达式
    fn evaluate_condition(
        &self,
        condition: &Condition,
        state: &ExecutionState,
    ) -> Result<ConditionResult>;

    /// 查找满足条件的路径
    fn find_satisfying_path(
        &self,
        constraints: &[Constraint],
    ) -> Result<PathResult>;

    /// 获取当前执行状态
    fn get_state(&self) -> &ExecutionState;
}

/// KLEE 执行器
pub struct KleeExecutor {
    /// KLEE 可执行文件路径
    klee_path: PathBuf,
    /// 工作目录
    work_dir: PathBuf,
    /// 超时设置
    timeout: Duration,
    /// 当前执行状态
    state: ExecutionState,
}

impl KleeExecutor {
    /// 创建 KLEE 执行器
    pub fn new(klee_path: PathBuf, work_dir: PathBuf) -> Self {
        Self {
            klee_path,
            work_dir,
            timeout: Duration::from_secs(30),
            state: ExecutionState::new(),
        }
    }

    /// 生成 KLEE 可执行的 C 代码
    fn generate_klee_code(&self, branch: &BranchCondition) -> String {
        // 将 LLVM IR 条件转换为 KLEE 可执行的 C 代码
        format!(r#"
            #include <klee/klee.h>

            int main() {{
                // 符号化条件变量
                klee_make_symbolic(&condition, sizeof(condition), "condition");

                // 添加约束
                {}

                // 检查分支条件
                if (condition) {{
                    // True 分支
                    return 0;
                }} else {{
                    // False 分支
                    return 1;
                }}
            }}
        "#, self.format_constraints(&branch.constraints))
    }
}

/// 模拟执行器 (KLEE 不可用时的后备)
pub struct SimulatedExecutor {
    /// 约束求解器
    solver: SimulatedSolver,
    /// 执行状态
    state: ExecutionState,
}

impl SymbolicExecutor for SimulatedExecutor {
    fn analyze_branch(
        &mut self,
        branch: &BranchCondition,
        _context: &ExecutionContext,
    ) -> Result<BranchResult> {
        // 使用启发式分析
        let result = self.solver.solve(&branch.constraints);

        match result {
            ConstraintSolution::Satisfiable(paths) => Ok(BranchResult {
                true_branch: BranchInfo {
                    is_feasible: true,
                    conditions: paths.true_conditions,
                    confidence: Confidence::Certain,
                },
                false_branch: BranchInfo {
                    is_feasible: true,
                    conditions: paths.false_conditions,
                    confidence: Confidence::Certain,
                },
            }),
            ConstraintSolution::Unknown => Ok(BranchResult {
                true_branch: BranchInfo {
                    is_feasible: true,
                    conditions: Vec::new(),
                    confidence: Confidence::Unknown,
                },
                false_branch: BranchInfo {
                    is_feasible: true,
                    conditions: Vec::new(),
                    confidence: Confidence::Unknown,
                },
            }),
            _ => Ok(BranchResult {
                true_branch: BranchInfo {
                    is_feasible: false,
                    conditions: Vec::new(),
                    confidence: Confidence::Certain,
                },
                false_branch: BranchInfo {
                    is_feasible: true,
                    conditions: Vec::new(),
                    confidence: Confidence::Certain,
                },
            }),
        }
    }
}
```

#### 2.2.6 flowsight-query (查询引擎)

**职责**: 提供强大的查询语言，支持用户自定义查询和过滤。

**查询语言设计**:

```rust
/// 查询表达式
pub enum QueryExpr {
    /// 函数查询
    Function {
        name_pattern: Pattern,
        filters: Vec<Filter>,
    },
    /// 调用链查询
    CallChain {
        from: Pattern,
        to: Pattern,
        depth: Option<u32>,
    },
    /// 路径查询
    Path {
        source: Pattern,
        target: Pattern,
        constraints: Vec<Constraint>,
    },
    /// 场景查询
    Scenario {
        mechanism: Pattern,
        trigger: Pattern,
    },
}

/// 查询结果
pub struct QueryResult {
    /// 匹配的函数
    pub functions: Vec<FunctionMatch>,
    /// 匹配的调用边
    pub call_edges: Vec<CallEdgeMatch>,
    /// 执行路径
    pub paths: Vec<PathMatch>,
    /// 统计信息
    pub statistics: QueryStats,
}

/// 查询执行器
pub struct QueryEngine {
    /// 索引系统
    index: SymbolIndex,
    /// 分析结果缓存
    analysis_cache: HashMap<PathBuf, AnalysisResult>,
    /// 知识库
    knowledge_base: KnowledgeBase,
}

impl QueryEngine {
    /// 执行查询
    pub fn execute(&self, query: &QueryExpr) -> Result<QueryResult> {
        match query {
            QueryExpr::Function { name_pattern, filters } => {
                self.query_functions(name_pattern, filters)
            }
            QueryExpr::CallChain { from, to, depth } => {
                self.query_call_chain(from, to, *depth)
            }
            QueryExpr::Path { source, target, constraints } => {
                self.query_path(source, target, constraints)
            }
            QueryExpr::Scenario { mechanism, trigger } => {
                self.query_scenario(mechanism, trigger)
            }
        }
    }

    /// 查询所有调用特定函数的函数
    pub fn query_callers(&self, func_name: &str) -> Vec<String> {
        self.index
            .get_reverse_callers(func_name)
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// 查询调用特定函数的所有路径
    pub fn query_call_paths(&self, source: &str, target: &str) -> Vec<CallPath> {
        let mut paths = Vec::new();
        self.find_all_paths(source, target, &mut paths, 10);
        paths
    }
}
```

### 2.3 前端模块 (React + TypeScript)

**职责**: 提供交互式 UI，展示分析结果，支持用户探索执行流。

```
app/src/
├── components/
│   ├── Editor/
│   │   ├── CodeEditor.tsx        # Monaco Editor 封装
│   │   ├── DiffView.tsx          # 差异对比视图
│   │   └── GoToLine.tsx          # 跳转行
│   │
│   ├── FlowView/
│   │   ├── FlowView.tsx          # 主流程图组件 (@xyflow/react)
│   │   ├── FlowNode.tsx          # 自定义节点
│   │   ├── FlowEdge.tsx          # 自定义边
│   │   └── FlowControls.tsx      # 流程图控制
│   │
│   ├── CallGraph/
│   │   ├── CallGraphView.tsx     # 调用图可视化
│   │   ├── CallGraphNode.tsx     # 节点组件
│   │   └── CallGraphEdge.tsx     # 边组件
│   │
│   ├── Explorer/
│   │   ├── FileTree.tsx          # 文件树
│   │   └── FunctionList.tsx      # 函数列表
│   │
│   ├── Outline/
│   │   └── Outline.tsx           # 大纲视图
│   │
│   ├── CommandPalette/
│   │   └── CommandPalette.tsx    # 命令面板
│   │
│   ├── ScenarioPanel/
│   │   └── ScenarioPanel.tsx     # 场景分析面板
│   │
│   └── ...
│
├── store/
│   ├── editorStore.ts            # 编辑器状态
│   │   ├── tabs: TabData[]       # 打开的标签页
│   │   ├── activeTab: string     # 当前活动标签
│   │   ├── openFile(path)        # 打开文件
│   │   ├── closeTab(id)          # 关闭标签
│   │   └── setActive(id)         # 切换活动标签
│   │
│   ├── analysisStore.ts          # 分析状态
│   │   ├── results: AnalysisResult[]  # 分析结果
│   │   ├── selectedFunction: string   # 选中的函数
│   │   ├── currentFlow: FlowTreeNode[]  # 当前执行流
│   │   ├── analyze(file)          # 分析文件
│   │   ├── selectFunction(name)   # 选择函数
│   │   └── setDisplayMode(mode)   # 设置显示模式
│   │
│   └── uiStore.ts                # UI 状态
│       ├── theme: Theme
│       ├── viewMode: ViewMode
│       ├── sidebarVisible: boolean
│       ├── toggleSidebar()
│       └── setViewMode(mode)
│
├── utils/
│   ├── format.ts                 # 格式化工具
│   ├── parser.ts                 # 解析工具
│   └── rpc.ts                    # Tauri RPC 调用
│
└── types.ts                      # 共享类型定义
```

---

## 3. 数据流设计

### 3.1 整体数据流

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           整体数据流                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   用户操作                         分析流程                          UI 渲染 │
│                                                                          │
│      │                              │                                  │
│      ▼                              ▼                                  │
│   ┌──────────┐                 ┌─────────────┐                      │
│   │  加载项目  │ ─────────────► │  索引构建    │                      │
│   │  或文件    │                 │  Indexer    │                      │
│   └──────────┘                 └──────┬──────┘                      │
│                                        │                              │
│                                        ▼                              │
│   ┌─────────────────────────────────────────────────────────────────┐  │
│   │                        分析管道                                  │  │
│   │                                                                  │  │
│   │   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐    │  │
│   │   │  Parser  │──►│  LLVM   │──►│ Knowledge│──►│ Analyzer │    │  │
│   │   │(C Parser)│   │   IR    │   │   Base   │   │(FlowTree)│    │  │
│   │   └──────────┘   └──────────┘   └──────────┘   └────┬─────┘    │  │
│   │                                                     │          │  │
│   │                                                     ▼          │  │
│   │   ┌────────────────────────────────────────────────────────┐  │  │
│   │   │              Symbol Execution (按需)                    │  │  │
│   │   │     KLEE ──► Branch Analysis ──► Constraint Solver    │  │  │
│   │   └────────────────────────────────────────────────────────┘  │  │
│   │                                                                  │  │
│   └──────────────────────────────────────────────────────────────────┘  │
│                                        │                                  │
│                                        ▼                                  │
│                                 ┌──────────────┐                         │
│                                 │  分析结果    │                         │
│                                 │  序列化      │                         │
│                                 └──────┬───────┘                         │
│                                        │                                 │
│                                        ▼                                 │
│   ┌─────────────────────────────────────────────────────────────────┐  │
│   │                     Tauri IPC 传输                              │  │
│   │            JSON/JSON-RPC ──► WebSocket/Channel                  │  │
│   └─────────────────────────────────────────────────────────────────┘  │
│                                        │                                 │
│                                        ▼                                 │
│   ┌─────────────────────────────────────────────────────────────────┐  │
│   │                     Zustand 状态管理                            │  │
│   │         analysisStore ──► uiStore ──► components               │  │
│   └─────────────────────────────────────────────────────────────────┘  │
│                                        │                                 │
│                                        ▼                                 │
│                              ┌──────────────────┐                        │
│                              │    React UI 渲染  │                        │
│                              │  Monaco + @xyflow │                        │
│                              └──────────────────┘                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 分析管道详细数据流

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       分析管道详细数据流                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  输入阶段                                                                     │
│  ═══════════                                                                 │
│                                                                          │
│    源代码 (.c/.h)                                                           │
│         │                                                                   │
│         ▼                                                                   │
│    ┌─────────────────────────────────────────┐                            │
│    │         Tree-sitter C Parser            │                            │
│    │  输出: AST, 符号表, 位置信息            │                            │
│    └────────────────────┬────────────────────┘                            │
│                         │                                                  │
│                         ▼                                                  │
│    LLVM Bitcode (.bc) ──────┐                                              │
│         │                   │                                              │
│         ▼                   ▼                                              │
│    ┌───────────────────┐ ┌───────────────────┐                            │
│    │  LLVM IR Parser   │ │   类型合并器      │                            │
│    │ - 函数定义        │ │ - 源类型 + IR 类型│                            │
│    │ - 调用指令        │ │ - 指针类型解析    │                            │
│    │ - 类型信息        │ │ - 完整类型信息    │                            │
│    │ - 基本块          │ │                   │                            │
│    └───────┬───────────┘ └───────────────────┘                            │
│            │                                                              │
│            ▼                                                              │
│    ┌─────────────────────────────────────────┐                            │
│    │         合并后的符号表                   │                            │
│    │  - 函数签名 (含完整类型)                 │                            │
│    │  - 全局变量                             │                            │
│    │  - 结构体定义                           │                            │
│    │  - 位置映射 (源 <-> IR)                 │                            │
│    └────────────────────┬────────────────────┘                            │
│                         │                                                  │
│  ════════════════════════════════════════════════════════════════════    │
│  分析阶段                                                                     │
│  ═══════════                                                                 │
│                                                                          │
│            │                                                              │
│            ▼                                                              │
│    ┌─────────────────────────────────────────┐                            │
│    │        Async Mechanism Tracker          │                            │
│    │  输入: 源代码 + 符号表 + 知识库          │                            │
│    │  处理:                                   │                            │
│    │    1. 匹配 bind 模式 (INIT_WORK 等)     │                            │
│    │    2. 匹配 trigger 模式 (schedule_work) │                            │
│    │    3. 验证 handler 存在性               │                            │
│    │  输出: AsyncBinding[]                   │                            │
│    └────────────────────┬────────────────────┘                            │
│                         │                                                  │
│                         ▼                                                  │
│    ┌─────────────────────────────────────────┐                            │
│    │        Function Pointer Resolver        │                            │
│    │  输入: 源代码 + 符号表 + 知识库 (ops 表) │                            │
│    │  处理:                                   │                            │
│    │    1. 识别 ops 结构体定义               │                            │
│    │    2. 匹配字段赋值 (.read = handler)    │                            │
│    │    3. 基于类型的函数匹配               │                            │
│    │    4. LLVM IR 间接调用验证             │                            │
│    │  输出: FuncPtrBinding[]                 │                            │
│    └────────────────────┬────────────────────┘                            │
│                         │                                                  │
│            ┌────────────┴────────────┐                                    │
│            ▼                         ▼                                    │
│    ┌───────────────────┐   ┌───────────────────┐                          │
│    │  Call Graph       │   │  Pointer Analysis │                          │
│    │  Builder          │   │  (Andersen-style) │                          │
│    │  - 直接调用边    │   │  - 指针别名分析   │                          │
│    │  - 回调调用边    │   │  - 指针传递链     │                          │
│    │  - 函数指针边    │   │  - 堆分配追踪     │                          │
│    └───────┬───────────┘   └───────────────────┘                          │
│            │                                                              │
│            ▼                                                              │
│    ┌─────────────────────────────────────────┐                            │
│    │         Call Graph with Annotations     │                            │
│    │  - 调用边类型标注                       │                            │
│    │  - 置信度评估                           │                            │
│    │  - 执行上下文标记                       │                            │
│    └────────────────────┬────────────────────┘                            │
│                         │                                                  │
│  ════════════════════════════════════════════════════════════════════    │
│  知识注入阶段                                                                 │
│  ═══════════════                                                         │
│                                                                          │
│            │                                                              │
│            ▼                                                              │
│    ┌─────────────────────────────────────────┐                            │
│    │        Knowledge Base Injector          │                            │
│    │  输入: CallGraph + 知识库 (API 调用链)  │                            │
│    │  处理:                                   │                            │
│    │    1. 识别 API 调用 (copy_from_user 等) │                            │
│    │    2. 查找对应的内核调用链              │                            │
│    │    3. 注入完整执行路径                  │                            │
│    │    4. 标注执行上下文                    │                            │
│    │  输出: AnnotatedCallGraph               │                            │
│    └────────────────────┬────────────────────┘                            │
│                         │                                                  │
│  ════════════════════════════════════════════════════════════════════    │
│  可选: 符号执行阶段                                                         │
│  ═══════════════════                                                     │
│                                                                          │
│            │ (用户请求精确分支分析)                                         │
│            ▼                                                              │
│    ┌─────────────────────────────────────────┐                            │
│    │        Symbolic Executor (KLEE)         │                            │
│    │  输入: 分支条件 + 约束                   │                            │
│    │  处理:                                   │                            │
│    │    1. 生成 KLEE 可执行代码              │                            │
│    │    2. 符号化条件变量                    │                            │
│    │    3. 求解约束条件                      │                            │
│    │    4. 验证分支可达性                    │                            │
│    │  输出: BranchAnalysisResult             │                            │
│    └────────────────────┬────────────────────┘                            │
│                         │                                                  │
│  ════════════════════════════════════════════════════════════════════    │
│  输出阶段                                                                     │
│  ═══════                                                                 │
│                                                                          │
│            │                                                              │
│            ▼                                                              │
│    ┌─────────────────────────────────────────┐                            │
│    │         Result Formatter                │                            │
│    │  输出格式:                               │                            │
│    │    - JSON (API)                         │                            │
│    │    - FlowTree (UI)                      │                            │
│    │    - DOT (Graphviz)                     │                            │
│    │    - Mermaid (文档)                     │                            │
│    └────────────────────┬────────────────────┘                            │
│                         │                                                  │
│                         ▼                                                  │
│    ┌─────────────────────────────────────────┐                            │
│    │         状态更新 (Zustand)              │                            │
│    │  - analysisStore.results = new_result   │                            │
│    │  - uiStore.triggerRender()              │                            │
│    └─────────────────────────────────────────┘                            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.3 Tauri IPC 数据流

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Tauri IPC 数据流                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  前端 (React)                              后端 (Rust)                   │
│                                                                          │
│     │                                           │                        │
│     │  invoke('analyzeFile', { path })          │                        │
│     │ ─────────────────────────────────────────►│                        │
│     │                                           │                        │
│     │                    ┌────────────────────┐  │                        │
│     │                    │  Command Handler   │  │                        │
│     │                    │  commands.rs       │  │                        │
│     │                    └─────────┬──────────┘  │                        │
│     │                              │             │                        │
│     │                              ▼             │                        │
│     │                    ┌────────────────────┐  │                        │
│     │                    │  Analysis Pipeline │  │                        │
│     │                    │                    │  │                        │
│     │                    │  Parser → LLVM →  │  │                        │
│     │                    │  Knowledge →      │  │                        │
│     │                    │  Analyzer → Query │  │                        │
│     │                    └─────────┬──────────┘  │                        │
│     │                              │             │                        │
│     │                              ▼             │                        │
│     │                    ┌────────────────────┐  │                        │
│     │                    │  Result Serializer │  │                        │
│     │                    │  (serde_json)      │  │                        │
│     │                    └─────────┬──────────┘  │                        │
│     │                              │             │                        │
│     │  invoke return ◄─────────────┼─────────────│                        │
│     │ ◄────────────────────────────│─────────────│                        │
│     │                              │             │                        │
│     │                              ▼             │                        │
│     │                    ┌────────────────────┐  │                        │
│     │                    │  State Update      │  │                        │
│     │                    │  (Zustand)         │  │                        │
│     │                    └────────────────────┘  │                        │
│     │                                           │                        │
│     │  React re-render                         │                        │
│     │ ◄─────────────────────────────────────────                        │
│     │                                           │                        │
│                                                                          │
│  ═══════════════════════════════════════════════════════════════════    │
│  消息格式                                                                 │
│  ═══════                                                             │
│                                                                          │
│  请求消息:                                                               │
│  {                                                                       │
│    "command": "analyzeFile",                                            │
│    "args": {                                                             │
│      "path": "/path/to/file.c",                                         │
│      "options": {                                                       │
│        "enableSymbolic": true,                                          │
│        "knowledgeBase": "linux-kernel-6.1"                              │
│      }                                                                  │
│    }                                                                     │
│  }                                                                       │
│                                                                          │
│  响应消息 (成功):                                                        │
│  {                                                                       │
│    "status": "ok",                                                      │
│    "result": {                                                          │
│      "functions": [...],                                                │
│      "callEdges": [...],                                                │
│      "asyncBindings": [...],                                            │
│      "flowTrees": [...]                                                 │
│    }                                                                     │
│  }                                                                       │
│                                                                          │
│  响应消息 (错误):                                                        │
│  {                                                                       │
│    "status": "error",                                                   │
│    "error": {                                                           │
│      "code": "PARSE_ERROR",                                             │
│      "message": "Failed to parse file.c",                               │
│      "details": {...}                                                   │
│    }                                                                     │
│  }                                                                       │
│                                                                          │
│  ═══════════════════════════════════════════════════════════════════    │
│  进度通知 (使用事件)                                                      │
│  ════════════════════════════════════════════════                       │
│                                                                          │
│  // 分析开始                                                             │
│  {                                                                       │
│    "event": "analysis.started",                                         │
│    "data": { "file": "/path/to/file.c" }                                │
│  }                                                                       │
│                                                                          │
│  // 进度更新                                                             │
│  {                                                                       │
│    "event": "analysis.progress",                                        │
│    "data": {                                                            │
│      "stage": "parsing",                                                │
│      "percent": 45,                                                     │
│      "message": "Parsing source code..."                                │
│    }                                                                     │
│  }                                                                       │
│                                                                          │
│  // 分析完成                                                             │
│  {                                                                       │
│    "event": "analysis.completed",                                       │
│    "data": { "result": {...} }                                          │
│  }                                                                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 4. 组件交互关系

### 4.1 核心组件依赖图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        组件依赖关系图                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│                              ┌────────────────┐                          │
│                              │ flowsight-core │                          │
│                              │   (无依赖)     │                          │
│                              └───────┬────────┘                          │
│                                      │                                   │
│                                      │ types, error, config              │
│                                      ▼                                   │
│   ┌───────────────────────────────────────────────────────────────┐     │
│   │                       分析层                                   │     │
│   │                                                                    │     │
│   │   ┌─────────────┐              ┌─────────────┐                  │     │
│   │   │flow-parser  │              │flow-llvm   │                  │     │
│   │   │             │              │             │                  │     │
│   │   └──────┬──────┘              └──────┬──────┘                  │     │
│   │          │                            │                          │     │
│   │          └──────────┬─────────────────┘                          │     │
│   │                     │                                            │     │
│   │                     ▼                                            │     │
│   │          ┌─────────────────────┐                                 │     │
│   │          │ flow-analysis       │                                 │     │
│   │          │                     │                                 │     │
│   │          └──────────┬──────────┘                                 │     │
│   │                     │                                            │     │
│   │                     │ depends on                                 │     │
│   │                     ▼                                            │     │
│   │          ┌─────────────────────┐                                 │     │
│   │          │ flow-knowledge      │                                 │     │
│   │          │                     │                                 │     │
│   │          └──────────┬──────────┘                                 │     │
│   │                     │                                            │     │
│   └─────────────────────┼────────────────────────────────────────────┘     │
│                         │                                              │
│                         │ depends on                                    │
│                         ▼                                              │
│   ┌───────────────────────────────────────────────────────────────┐     │
│   │                       增强层                                   │     │
│   │                                                                    │     │
│   │   ┌─────────────┐              ┌─────────────┐                  │     │
│   │   │flow-symbolic│              │flow-query  │                  │     │
│   │   │             │              │             │                  │     │
│   │   │ depends on: │              │ depends on: │                  │     │
│   │   │ - parser    │              │ - index     │                  │     │
│   │   │ - knowledge │              │ - analysis  │                  │     │
│   │   │             │              │ - knowledge │                  │     │
│   │   └──────┬──────┘              └──────┬──────┘                  │     │
│   │          │                            │                          │     │
│   │          └────────────────────────────┘                          │     │
│   │                                                                    │     │
│   └───────────────────────────────────────────────────────────────────┘     │
│                         │                                              │
│                         │ depends on                                    │
│                         ▼                                              │
│   ┌───────────────────────────────────────────────────────────────┐     │
│   │                       索引层                                   │     │
│   │                                                                    │     │
│   │   ┌─────────────┐                                                │     │
│   │   │flow-index   │                                                │     │
│   │   │             │                                                │     │
│   │   │ depends on: │                                                │     │
│   │   │ - core      │                                                │     │
│   │   │ - parser    │                                                │     │
│   │   │ - llvm      │                                                │     │
│   │   │ - analysis  │                                                │                                                │
│   │   └─────────────┘                                                │     │
│   │                                                                    │     │
│   └───────────────────────────────────────────────────────────────────┘     │
│                         │                                              │
│                         │ serves                                       │
│                         ▼                                              │
│   ┌───────────────────────────────────────────────────────────────┐     │
│   │                       应用层                                   │     │
│   │                                                                    │     │
│   │   ┌─────────────┐              ┌─────────────┐                  │     │
│   │   │flow-cli     │              │app/src-tauri│                  │     │
│   │   │             │              │             │                  │     │
│   │   └─────────────┘              └──────┬──────┘                  │     │
│   │                                       │                          │     │
│   │                                       │ serves                   │     │
│   │                                       ▼                          │     │
│   │                          ┌─────────────────────┐                  │     │
│   │                          │     React Frontend  │                  │     │
│   │                          │                     │                  │     │
│   │                          └─────────────────────┘                  │     │
│   │                                                                    │     │
│   └───────────────────────────────────────────────────────────────────┘     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.2 关键交互场景

#### 场景 1: 用户点击函数，查看完整执行流

```
┌─────────────────────────────────────────────────────────────────────────┐
│                   场景 1: 查看函数执行流                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 用户在 UI 中点击函数 "usb_probe"                                     │
│     │                                                                     │
│     ▼                                                                     │
│  2. React 组件触发 Zustand action                                        │
│     useAnalysisStore.getState().selectFunction("usb_probe")             │
│     │                                                                     │
│     ▼                                                                     │
│  3. Zustand 更新状态，触发 re-render                                     │
│     │                                                                     │
│     ▼                                                                     │
│  4. FlowView 组件监听 selectedFunction 变化                              │
│     │                                                                     │
│     ▼                                                                     │
│  5. Tauri invoke 调用 Rust 后端                                          │
│     invoke('getFunctionFlow', { name: "usb_probe" })                    │
│     │                                                                     │
│     ▼                                                                     │
│  6. Rust 后端处理                                                        │
│     ┌─────────────────────────────────────────┐                          │
│     │  QueryEngine.get_function_flow()        │                          │
│     │                                          │                          │
│     │  1. 从索引中查找函数定义                 │                          │
│     │  2. 从分析结果中获取调用图              │                          │
│     │  3. 从知识库注入内核 API 调用链         │                          │
│     │  4. 构建 FlowTree 结构                  │                          │
│     │  5. 返回序列化结果                      │                          │
│     └────────────────────┬────────────────────┘                          │
│                          │                                                │
│                          ▼                                                │
│  7. 结果返回前端，更新 Zustand                                            │
│     analysisStore.setCurrentFlow(flowTree)                              │
│     │                                                                     │
│     ▼                                                                     │
│  8. FlowView 渲染执行流图                                                │
│     - @xyflow/react 渲染节点和边                                         │
│     - 高亮直接调用                                                        │
│     - 虚线显示异步回调                                                    │
│     - 节点可点击跳转                                                      │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 场景 2: 分析分支条件 (按需符号执行)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                场景 2: 精确分支条件分析                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 用户点击分支节点，请求 "分析条件"                                      │
│     │                                                                     │
│     ▼                                                                     │
│  2. 前端发送分析请求                                                      │
│     invoke('analyzeBranch', { branchId: "node_123" })                   │
│     │                                                                     │
│     ▼                                                                     │
│  3. Rust 后端检查 KLEE 可用性                                            │
│     ┌─────────────────────────────────────────┐                          │
│     │  SymbolicExecutor::new()                │                          │
│     │                                          │                          │
│     │  if KLEE available:                     │                          │
│     │    return KleeExecutor::new()           │                          │
│     │  else:                                  │                          │
│     │    return SimulatedExecutor::new()      │                          │
│     └────────────────────┬────────────────────┘                          │
│                          │                                                │
│                          ▼                                                │
│  4. 提取分支条件                                                         │
│     ┌─────────────────────────────────────────┐                          │
│     │  从 LLVM IR 或分析结果获取:             │                          │
│     │                                          │                          │
│     │  - 条件变量                             │                          │
│     │  - 约束表达式                           │                          │
│     │  - 执行上下文                           │                          │
│     └────────────────────┬────────────────────┘                          │
│                          │                                                │
│                          ▼                                                │
│  5. 执行符号分析                                                         │
│     ┌─────────────────────────────────────────┐                          │
│     │  executor.analyze_branch(condition)     │                          │
│     │                                          │                          │
│     │  KLEE 模式:                             │                          │
│     │    1. 生成 KLEE C 代码                 │                          │
│     │    2. 写入临时文件                      │                          │
│     │    3. spawn KLEE 进程                  │                          │
│     │    4. 读取输出，解析结果                │                          │
│     │    5. 清理临时文件                      │                          │
│     │                                          │                          │
│     │  模拟模式:                              │                          │
│     │    1. 约束传播分析                     │                          │
│     │    2. 区间运算求解                     │                          │
│     │    3. 返回启发式结果                   │                          │
│     └────────────────────┬────────────────────┘                          │
│                          │                                                │
│                          ▼                                                │
│  6. 返回分析结果                                                         │
│     {                                                                  │
│       "trueBranch": {                                                   │
│         "isFeasible": true,                                             │
│         "conditions": ["ptr != NULL"],                                  │
│         "confidence": "Certain"                                         │
│       },                                                                │
│       "falseBranch": {                                                  │
│         "isFeasible": true,                                             │
│         "conditions": ["ptr == NULL"],                                  │
│         "confidence": "Certain"                                         │
│       }                                                                 │
│     }                                                                   │
│     │                                                                     │
│     ▼                                                                     │
│  7. 前端更新 UI                                                          │
│     - 在分支节点旁边显示条件                                              │
│     - 使用不同颜色标记可行/不可行分支                                     │
│     - 显示置信度标记                                                     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 场景 3: 知识库模式匹配

```
┌─────────────────────────────────────────────────────────────────────────┐
│                  场景 3: 知识库模式匹配                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. AsyncTracker.analyze() 被调用                                        │
│     │                                                                     │
│     ▼                                                                     │
│  2. 加载知识库                                                           │
│     ┌─────────────────────────────────────────┐                          │
│     │  KnowledgeBase.load_async_patterns()    │                          │
│     │                                          │                          │
│     │  从 YAML 文件加载:                       │                          │
│     │  - workqueue.yaml                       │                          │
│     │  - timer.yaml                           │                          │
│     │  - irq.yaml                             │                          │
│     │  - ...                                  │                          │
│     │                                          │                          │
│     │  解析为 Pattern 结构:                   │                          │
│     │  - name: "INIT_WORK"                    │                          │
│     │  - regex: r'INIT_WORK\s*\(&?(\w+),...'  │                          │
│     │  - parameters: [...]                    │                          │
│     │  - execution_chain: [...]               │                          │
│     └────────────────────┬────────────────────┘                          │
│                          │                                                │
│                          ▼                                                │
│  3. 遍历所有模式进行匹配                                                  │
│     ┌─────────────────────────────────────────┐                          │
│     │  for pattern in patterns:               │                          │
│     │                                          │                          │
│     │    // 匹配绑定模式                       │                          │
│     │    for bind_match in pattern.bind_matches │                        │
│     │      .find_all(source):                 │                          │
│     │        extract: var, handler            │                          │
│     │        if functions.contains(handler):  │                          │
│     │          create AsyncBinding            │                          │
│     │                                          │                          │
│     │    // 匹配触发模式                       │                          │
│     │    for trigger_match in pattern.trigger_matches│                    │
│     │      .find_all(source):                 │                          │
│     │        record trigger location          │                          │
│     │                                          │                          │
│     └────────────────────┬────────────────────┘                          │
│                          │                                                │
│                          ▼                                                │
│  4. 构建 AsyncBinding                                                   │
│     ┌─────────────────────────────────────────┐                          │
│     │  AsyncBinding {                         │                          │
│     │    mechanism: WorkQueue {               │                          │
│     │      work_struct: "dev->work",          │                          │
│     │      queue: None                        │                          │
│     │    },                                   │                          │
│     │    handler: "my_work_handler",          │                          │
│     │    bind_location: Location {...},       │                          │
│     │    trigger_location: Location {...},    │                          │
│     │    trigger_conditions: [...]            │                          │
│     │  }                                      │                          │
│     │                                          │                          │
│     │  // 从知识库注入执行链                   │                          │
│     │  // (schedule_work -> worker_thread ->  │                          │
│     │  //  process_one_work -> handler)       │                          │
│     └────────────────────┬────────────────────┘                          │
│                          │                                                │
│                          ▼                                                │
│  5. 返回结果，包含完整执行链                                              │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 5. 关键技术决策

### 5.1 架构决策记录 (ADR)

#### ADR-001: 采用 Rust + Tree-sitter + LLVM IR 混合分析

**状态**: 已批准

**背景**:
- 纯静态分析难以处理函数指针和间接调用
- 纯语义分析覆盖率不足
- 需要在准确性和覆盖率之间取得平衡

**决策**:
- 使用 Tree-sitter 进行快速的语法解析，获取 AST 和符号表
- 使用 LLVM IR 分析获取准确的类型和指针信息
- 使用知识库提供 100% 准确的 API 语义信息
- 按需使用 KLEE 进行符号执行

**权衡分析**:

| 方案 | 优点 | 缺点 |
|------|------|------|
| 纯静态分析 | 快速、全面 | 不准确，无法处理函数指针 |
| 纯 IR 分析 | 准确 | 丢失源代码语义，需要编译产物 |
| **混合方案 (选定)** | **准确 + 语义完整** | **实现复杂** |
| 纯符号执行 | 最精确 | 性能差，无法处理大程序 |

#### ADR-002: 前端选择 React + Monaco + @xyflow/react

**状态**: 已批准

**背景**: 需要现代化的 UI 框架，支持复杂的代码编辑和图形可视化

**决策**:
- React 18: 成熟的组件化框架，丰富的生态系统
- Monaco Editor: VS Code 同款编辑器，专业级代码编辑体验
- @xyflow/react: 灵活的流程图库，支持自定义节点和边

**替代方案考虑**:
- CodeMirror vs Monaco: Monaco 功能更全面，但体积较大 → Monaco 胜出
- React Flow vs @xyflow/react: 同一库的新名称，选择最新版本
- Electron vs Tauri: Tauri 体积小、性能好、安全性高 → Tauri 胜出

#### ADR-003: 后端采用 Rust async runtime (Tokio)

**状态**: 已批准

**背景**: 需要处理大量并发分析任务，同时保持响应性

**决策**:
- Tokio 作为 async runtime
- rayon 用于 CPU 并行任务 (如并行文件解析)
- 明确的异步边界：I/O 密集型用 Tokio，CPU 密集型用 rayon

**配置**:

```rust
// Cargo.toml
tokio = { version = "1.35", features = ["full"] }
rayon = "1.8"

// 使用场景
let results: Vec<Result<()>> = rayon::ParallelIterator::map(
    files.par_iter(),
    |file| analyze_file(file)
).collect();
```

#### ADR-004: 存储方案选择 SQLite + Sled

**状态**: 已批准

**背景**: 需要同时支持关系查询和图遍历

**决策**:
- SQLite: 存储符号表、函数元数据、位置信息 (关系型查询)
- Sled: 存储调用图、依赖关系 (键值存储，图遍历)

**使用场景**:

```rust
// SQLite - 函数查询
let stmt = conn.prepare(
    "SELECT * FROM functions WHERE name LIKE ?"
)?;
let func: FunctionRow = stmt.query_row(["%usb%"], |r| r.try_into())?;

// Sled - 调用图遍历
let graph = sled_db.open_tree("call_graph")?;
let callers = graph.get(format!("{}->", func_name))?;
```

#### ADR-005: 知识库使用 YAML 格式

**状态**: 已批准

**背景**: 知识库需要人类可读、可版本控制、易于维护

**决策**:
- YAML 格式：人类可读、结构化、支持注释
- JSON Schema 验证：保证格式正确性
- 易于贡献：社区可以轻松添加新模式

**替代方案**:
- JSON: 不支持注释，维护困难
- Protobuf: 二进制格式，不便手动编辑
- Lua 脚本: 太灵活，难以验证

#### ADR-006: KLEE 符号执行按需触发

**状态**: 已批准

**背景**: 符号执行计算成本高，不需要每次分析都触发

**决策**:
- 默认使用启发式分析作为后备
- 只有用户明确请求或高置信度需求时才触发 KLEE
- 支持超时设置，防止长时间运行
- 支持模拟模式 (KLEE 不可用时)

**配置选项**:

```rust
pub struct SymbolicConfig {
    pub enabled: bool,           // 是否启用
    pub use_klee: bool,          // 是否优先使用 KLEE
    pub timeout: Duration,       // 超时时间
    pub fallback_to_simulated: bool,  // KLEE 不可用时回退
}

impl Default for SymbolicConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_klee: false,  // 默认使用模拟模式
            timeout: Duration::from_secs(30),
            fallback_to_simulated: true,
        }
    }
}
```

### 5.2 性能优化策略

| 优化点 | 策略 | 预期收益 |
|--------|------|----------|
| 并行解析 | 使用 rayon 并行解析多个文件 | 10x 加速 (多核) |
| 结果缓存 | 相同文件 + 相同配置 = 缓存结果 | 再次打开秒开 |
| 懒加载 | 按需加载 LLVM IR 模块 | 减少内存占用 |
| 流式处理 | 边解析边输出 | 更快看到初步结果 |
| 增量分析 | 文件修改后只重新分析受影响部分 | 大项目增量更新 |
| 增量渲染 | FlowView 只更新变化的节点 | UI 响应更快 |

### 5.3 扩展性设计

#### 模块化架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        插件架构设计                                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Core (flowsight-core)                                                  │
│      │                                                                   │
│      │  定义插件接口                                                     │
│      ▼                                                                   │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                     Plugin System                                │    │
│  │                                                                   │    │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │    │
│  │   │  Language   │  │  Platform   │  │   Feature   │             │    │
│  │   │  Plugin     │  │  Plugin     │  │   Plugin    │             │    │
│  │   │             │  │             │  │             │             │    │
│  │   │ - C Parser  │  │ - Linux KB  │  │ - Trace     │             │    │
│  │   │ - C++ Parser│  │ - Android KB│  │ - Coverage  │             │    │
│  │   │ - Rust Parser│  │ - RTOS KB   │  │ - Profiling │             │    │
│  │   └─────────────┘  └─────────────┘  └─────────────┘             │    │
│  │                                                                   │    │
│  └───────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│  插件接口定义:                                                           │
│                                                                          │
│  pub trait LanguagePlugin {                                             │
│      fn name(&self) -> &str;                                            │
│      fn extensions(&self) -> &[&str];                                   │
│      fn create_parser(&self) -> Box<dyn Parser>;                        │
│  }                                                                       │
│                                                                          │
│  pub trait PlatformKnowledge {                                          │
│      fn name(&self) -> &str;                                            │
│      fn load_patterns(&self) -> Vec<AsyncPattern>;                      │
│      fn get_execution_chain(&self, api: &str) -> Option<ExecutionChain>;│
│  }                                                                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.4 错误处理与恢复

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        错误处理架构                                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  错误分类                                                                │
│  ════════                                                                │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  Error Severity                                                  │    │
│  │                                                                   │    │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐             │    │
│  │  │  Fatal  │  │  Error  │  │Warning  │  │  Info   │             │    │
│  │  │         │  │         │  │         │  │         │             │    │
│  │  │ 分析终止 │  │ 部分失败│  │ 结果可疑│  │ 进度信息│             │    │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘             │    │
│  │                                                                   │    │
│  └───────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│  错误处理策略                                                            │
│  ══════════════                                                        │
│                                                                          │
│  Fatal:                                                                  │
│  - 立即停止分析                                                          │
│  - 显示错误对话框                                                        │
│  - 建议用户检查输入                                                      │
│                                                                          │
│  Error:                                                                  │
│  - 记录错误日志                                                          │
│  - 返回部分结果                                                          │
│  - 在 UI 中显示警告图标                                                  │
│                                                                          │
│  Warning:                                                                │
│  - 记录日志                                                              │
│  - 继续分析                                                              │
│  - 在结果中标记置信度                                                    │
│                                                                          │
│  Info:                                                                   │
│  - 仅记录进度                                                            │
│  - 显示在进度条中                                                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 6. 部署架构

### 6.1 开发环境

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        开发环境架构                                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   ┌─────────────────────────────────────────────────────────────────┐    │
│   │                      本地开发环境                                │    │
│   │                                                                   │    │
│   │   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │    │
│   │   │   Rust   │  │  Node.js │  │   Git    │  │  VS Code │       │    │
│   │   │  1.75+   │  │   20+    │  │           │  │  + Ext   │       │    │
│   │   └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘       │    │
│   │        │             │             │             │               │    │
│   │        └─────────────┴─────────────┴─────────────┘               │    │
│   │                              │                                    │    │
│   │                              ▼                                    │    │
│   │   ┌─────────────────────────────────────────────────────────┐    │    │
│   │   │                    pnpm workspace                        │    │    │
│   │   │                                                       │    │    │
│   │   │   pnpm tauri dev    →  开发模式运行                    │    │    │
│   │   │   pnpm tauri build  →  构建发布版本                    │    │    │
│   │   │   cargo test       →  运行测试                         │    │    │
│   │   │                                                       │    │    │
│   │   └─────────────────────────────────────────────────────────┘    │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│   依赖服务 (可选)                                                         │
│   ═════════════════                                                      │
│                                                                          │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐                             │
│   │  KLEE    │  │ LLVM 17  │  │  Docker  │                             │
│   │ Symbolic │  │  Dev     │  │ (可选)   │                             │
│   │ Execution│  │  Files   │  │          │                             │
│   └──────────┘  └──────────┘  └──────────┘                             │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.2 生产环境

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        生产环境架构                                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   ┌─────────────────────────────────────────────────────────────────┐    │
│   │                     用户桌面环境                                 │    │
│   │                                                                   │    │
│   │   ┌─────────────────────────────────────────────────────────┐    │    │
│   │   │                    Tauri 应用                           │    │    │
│   │   │                                                          │    │    │
│   │   │   ┌────────────────────────────────────────────────┐    │    │    │
│   │   │   │              React Frontend                    │    │    │    │
│   │   │   │         (bundled in .asar)                     │    │    │    │
│   │   │   └────────────────────────────────────────────────┘    │    │    │
│   │   │                        │                               │    │    │
│   │   │   ┌────────────────────────────────────────────────┐    │    │    │
│   │   │   │              Rust Backend                      │    │    │    │
│   │   │   │         (native binary)                        │    │    │    │
│   │   │   └────────────────────────────────────────────────┘    │    │    │
│   │   │                        │                               │    │    │
│   │   │   ┌────────────────────────────────────────────────┐    │    │    │
│   │   │   │              Local Storage                     │    │    │    │
│   │   │   │    SQLite + Sled (in ~/.flowsight)            │    │    │    │
│   │   │   └────────────────────────────────────────────────┘    │    │    │
│   │   │                                                          │    │    │
│   │   └──────────────────────────────────────────────────────────┘    │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│   资源使用                                                                │
│   ═════════                                                              │
│                                                                          │
│   内存: ~200MB (空闲) → ~1GB (分析大型项目)                               │
│   磁盘: ~100MB (应用) + ~500MB (知识库) + 用户数据                        │
│   CPU: 多核优化，分析时使用全部核心                                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 7. 安全考虑

### 7.1 安全设计原则

| 原则 | 实现方式 |
|------|----------|
| 本地优先 | 所有分析在本地完成，不上传任何代码 |
| 沙箱隔离 | Tauri 权限系统限制文件系统访问 |
| 输入验证 | 严格验证所有用户输入和文件路径 |
| 内存安全 | Rust 语言保证内存安全，无 GC 暂停 |
| 依赖审计 | 定期扫描依赖漏洞 (cargo audit) |

### 7.2 权限控制

```json
// app/src-tauri/capabilities/default.json
{
  "permissions": [
    "core:default",
    "core:path:default",
    "core:shell:open",
    {
      "identifier": "fs:read-write",
      "allow": [
        {
          "name": "home",
          "path": "**/*"
        }
      ],
      "deny": [
        {
          "name": "config",
          "path": "~/.ssh/**"
        },
        {
          "name": "etc",
          "path": "/etc/shadow"
        }
      ]
    }
  ]
}
```

---

## 8. 未来演进

### 8.1 短期规划 (v1.0)

- [ ] 完善 C 语言分析
- [ ] 完成 Linux 内核知识库 (v1.0)
- [ ] 优化性能 (并行分析、缓存)
- [ ] 完善 UI (快捷键、主题)
- [ ] 稳定版发布

### 8.2 中期规划 (v2.0)

- [ ] Android 系统知识库
- [ ] C++ 语言支持
- [ ] Rust 语言支持
- [ ] AI 辅助分析 (机器学习)
- [ ] 远程分析 (可选服务器模式)

### 8.3 长期规划 (v3.0+)

- [ ] 多语言支持 (Go, Java, Python)
- [ ] 实时分析 (增量更新)
- [ ] 协作功能 (多用户分析会话)
- [ ] 插件系统 (第三方扩展)
- [ ] 云端知识库共享

---

## 9. 附录

### 9.1 术语表

| 术语 | 定义 |
|------|------|
| IR | Intermediate Representation，中间表示 |
| CFG | Control Flow Graph，控制流图 |
| AST | Abstract Syntax Tree，抽象语法树 |
| KLEE | 符号执行引擎 |
| LLVM | 编译器基础设施 |
| Tree-sitter | 增量解析器 |
| 知识库 | 预定义的 API 调用链和模式集合 |
| 执行流 | 函数调用的完整路径，包括异步回调 |
| 置信度 | 分析结果的可信程度 (Certain/Possible/Unknown) |

### 9.2 参考资料

- [LLVM Language Reference](https://llvm.org/docs/LangRef.html)
- [KLEE Symbolic Execution Engine](https://klee.github.io/)
- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
- [Tauri Documentation](https://tauri.app/)
- [Linux Kernel Source](https://github.com/torvalds/linux)

### 9.3 文档版本历史

| 版本 | 日期 | 修改内容 |
|------|------|----------|
| 1.0 | 2025-01-24 | 初始版本 |
