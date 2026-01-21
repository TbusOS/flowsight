# FlowSight 技术设计文档

> 版本: v2.0
> 更新日期: 2025-01-21
> 状态: 规划中

---

## 目录

1. [概述](#1-概述)
2. [系统架构](#2-系统架构)
3. [静态分析模块](#3-静态分析模块)
4. [符号执行模块](#4-符号执行模块)
5. [AI 辅助模块](#5-ai-辅助模块)
6. [自学习模块](#6-自学习模块)
7. [IDE 展示层](#7-ide-展示层)
8. [数据格式](#8-数据格式)
9. [API 设计](#9-api-设计)
10. [性能优化](#10-性能优化)
11. [测试策略](#11-测试策略)

---

## 1. 概述

### 1.1 设计目标

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          技术设计目标                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 精准分析：不运行代码，精准呈现函数执行过程                           │
│                                                                          │
│  2. 高效执行：单函数分析 < 5 秒，内存 < 2GB                              │
│                                                                          │
│  3. 本地部署：用户下载即用，不依赖外部 API                               │
│                                                                          │
│  4. 可扩展架构：支持新平台、新语言、新分析技术                           │
│                                                                          │
│  5. 自学习能力：从用户反馈中持续改进                                     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 技术路线

```
输入
  │
  ├──► 静态分析 ──► 调用图 ──► 回调识别 ──► 异步追踪
  │         │
  │         └──► 知识库 ──► 模式匹配 ──► 语义补充
  │
  └──► 符号执行 (可选) ──► 路径分析 ──► 约束条件
                                      │
                                      ▼
                              ┌─────────────────┐
                              │  AI 条件翻译    │
                              │  (本地模型)     │
                              └─────────────────┘
                                      │
                                      ▼
                              ┌─────────────────┐
                              │  执行流构建     │
                              └─────────────────┘
                                      │
                                      ▼
                              ┌─────────────────┐
                              │  IDE 展示       │
                              │  ftrace 风格    │
                              └─────────────────┘
```

### 1.3 技术选型

| 层级 | 技术选型 | 选型理由 |
|-----|---------|---------|
| 代码解析 | Tree-sitter | 高性能、跨语言、支持增量更新 |
| 静态分析 | Rust 实现 | 内存安全、高性能、易于维护 |
| 符号执行 | KLEE (LLVM) | 成熟、精确、社区活跃 |
| AI 推理 | llama.cpp | 本地运行、支持量化、社区活跃 |
| 前端框架 | React + TypeScript | 生态丰富、类型安全 |
| 桌面框架 | Tauri | 轻量、安全、跨平台 |
| 代码编辑 | Monaco Editor | VSCode 同款、体验好 |
| 图形可视化 | @xyflow/react | 功能强大、易于定制 |

---

## 2. 系统架构

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           FlowSight 系统架构                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                      客户端层 (Tauri App)                        │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │    │
│  │  │  Frontend   │  │  Backend    │  │  System Tray           │ │    │
│  │  │  (React)    │  │  (Rust)     │  │  (Tray Icon)           │ │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘ │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                    │                                     │
│                          Tauri IPC (JSON-RPC)                            │
│                                    ▼                                     │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                     分析引擎层                                    │    │
│  │                                                                  │    │
│  │  ┌──────────────────────────────────────────────────────────┐  │    │
│  │  │                  flowsight-parser                        │  │    │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────────┐│  │    │
│  │  │  │ C Parser│ │Rust Parser│ │SymbolTable│ │TypeInference ││  │    │
│  │  │  └─────────┘ └─────────┘ └─────────┘ └─────────────────┘│  │    │
│  │  └──────────────────────────────────────────────────────────┘  │    │
│  │                              │                                   │    │
│  │                              ▼                                   │    │
│  │  ┌──────────────────────────────────────────────────────────┐  │    │
│  │  │                  flowsight-analysis                      │  │    │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────────┐│  │    │
│  │  │  │CallGraph│ │Callback │ │ Async   │ │ Pointer         ││  │    │
│  │  │  │Analyzer │ │Detector │ │Tracker  │ │ Analyzer        ││  │    │
│  │  │  └─────────┘ └─────────┘ └─────────┘ └─────────────────┘│  │    │
│  │  └──────────────────────────────────────────────────────────┘  │    │
│  │                              │                                   │    │
│  │                              ▼                                   │    │
│  │  ┌──────────────────────────────────────────────────────────┐  │    │
│  │  │                  flowsight-symbolic                      │  │    │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────────────────────────┐│  │    │
│  │  │  │KLEE     │ │Path     │ │ Constraint                  ││  │    │
│  │  │  │Executor │ │Analyzer │ │ Generator                   ││  │    │
│  │  │  └─────────┘ └─────────┘ └─────────────────────────────┘│  │    │
│  │  └──────────────────────────────────────────────────────────┘  │    │
│  │                                                                  │    │
│  └──────────────────────────────────────────────────────────────────┘    │
│                                    │                                     │
│                                    ▼                                     │
│  ┌──────────────────────────────────────────────────────────────────┐    │
│  │                       AI 辅助层                                   │    │
│  │                                                                  │    │
│  │  ┌──────────────────────────────────────────────────────────┐  │    │
│  │  │                  flowsight-ai                            │  │    │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────────────────────────┐│  │    │
│  │  │  │Local    │ │Prompt   │ │ Batch                       ││  │    │
│  │  │  │Model    │ │Manager  │ │ Inference                   ││  │    │
│  │  │  └─────────┘ └─────────┘ └─────────────────────────────┘│  │    │
│  │  └──────────────────────────────────────────────────────────┘  │    │
│  │                              │                                   │    │
│  │                              ▼                                   │    │
│  │  ┌──────────────────────────────────────────────────────────┐  │    │
│  │  │                  flowsight-learning                      │  │    │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────────────────────────┐│  │    │
│  │  │  │Feedback │ │ Local   │ │ Knowledge                   ││  │    │
│  │  │  │Collector│ │ Trainer │ │ Uploader                    ││  │    │
│  │  │  └─────────┘ └─────────┘ └─────────────────────────────┘│  │    │
│  │  └──────────────────────────────────────────────────────────┘  │    │
│  │                                                                  │    │
│  └──────────────────────────────────────────────────────────────────┘    │
│                                    │                                     │
│                                    ▼                                     │
│  ┌──────────────────────────────────────────────────────────────────┐    │
│  │                        知识库层                                   │    │
│  │                                                                  │    │
│  │  knowledge/                                                      │    │
│  │  ├── linux-kernel/                                               │    │
│  │  │   ├── core/                                                   │    │
│  │  │   │   ├── workqueue.yaml                                      │    │
│  │  │   │   ├── irq.yaml                                            │    │
│  │  │   │   ├── vfs.yaml                                            │    │
│  │  │   │   └── memory.yaml                                         │    │
│  │  │   ├── drivers/                                                │    │
│  │  │   │   ├── usb.yaml                                            │    │
│  │  │   │   ├── i2c.yaml                                            │    │
│  │  │   │   ├── pci.yaml                                            │    │
│  │  │   │   └── platform.yaml                                       │    │
│  │  │   └── sync/                                                   │    │
│  │  │       ├── spinlock.yaml                                       │    │
│  │  │       └── mutex.yaml                                          │    │
│  │  └── execution_semantics.yaml                                    │    │
│  │                                                                      │    │
│  └──────────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 模块依赖关系

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           模块依赖图                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  flowsight-core (核心类型定义)                                           │
│         │                                                                    │
│         ├──► flowsight-parser                                              │
│         │         │                                                         │
│         │         └──► tree-sitter-c                                       │
│         │                                                                    │
│         ├──► flowsight-analysis                                            │
│         │         │                                                         │
│         │         ├──► flowsight-parser                                     │
│         │         ├──► flowsight-knowledge                                  │
│         │         └──► flowsight-core                                       │
│         │                                                                    │
│         ├──► flowsight-symbolic                                            │
│         │         │                                                         │
│         │         ├──► flowsight-analysis                                   │
│         │         ├──► flowsight-core                                       │
│         │         └──► KLEE (外部依赖)                                      │
│         │                                                                    │
│         ├──► flowsight-ai                                                  │
│         │         │                                                         │
│         │         ├──► flowsight-knowledge                                  │
│         │         ├──► flowsight-core                                       │
│         │         └──► llama.cpp (外部依赖)                                 │
│         │                                                                    │
│         └──► flowsight-learning                                            │
│                  │                                                          │
│                  ├──► flowsight-ai                                          │
│                  ├──► flowsight-knowledge                                  │
│                  └──► flowsight-core                                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. 静态分析模块

### 3.1 架构设计

```rust
// flowsight-analysis/src/lib.rs

/// 静态分析器主入口
pub struct StaticAnalyzer {
    /// 代码解析器
    parser: Parser,
    /// 符号表
    symbol_table: SymbolTable,
    /// 知识库
    knowledge_base: KnowledgeBase,
    /// 配置
    config: AnalyzerConfig,
}

impl StaticAnalyzer {
    /// 分析函数执行流
    pub fn analyze_function(
        &self,
        file: &Path,
        function_name: &str,
    ) -> Result<FunctionAnalysis, AnalyzerError> {
        // 1. 解析代码
        let ast = self.parser.parse_file(file)?;

        // 2. 构建符号表
        self.symbol_table.build(&ast)?;

        // 3. 提取函数信息
        let func_info = self.symbol_table.get_function(function_name)?;

        // 4. 构建调用图
        let call_graph = self.build_call_graph(&func_info)?;

        // 5. 检测回调
        let callbacks = self.detect_callbacks(&func_info, &ast)?;

        // 6. 追踪异步机制
        let async_bindings = self.track_async_mechanisms(&func_info, &ast)?;

        // 7. 合并结果
        let analysis = FunctionAnalysis {
            function: func_info,
            call_graph,
            callbacks,
            async_bindings,
            execution_context: self.determine_context(&func_info)?,
        };

        Ok(analysis)
    }
}

/// 函数分析结果
pub struct FunctionAnalysis {
    /// 函数信息
    pub function: FunctionInfo,
    /// 调用图
    pub call_graph: CallGraph,
    /// 回调列表
    pub callbacks: Vec<CallbackBinding>,
    /// 异步绑定
    pub async_bindings: Vec<AsyncBinding>,
    /// 执行上下文
    pub execution_context: ExecutionContext,
}
```

### 3.2 调用图构建

```rust
// flowsight-analysis/src/callgraph.rs

/// 调用图构建器
pub struct CallGraphBuilder<'a> {
    /// AST
    ast: &'a AST,
    /// 符号表
    symbol_table: &'a SymbolTable,
    /// 知识库
    knowledge_base: &'a KnowledgeBase,
}

impl<'a> CallGraphBuilder<'a> {
    /// 构建函数的调用图
    pub fn build(&self, func: &Function) -> Result<CallGraph, CallGraphError> {
        let mut call_graph = CallGraph::new();

        // 1. 收集直接调用
        self.collect_direct_calls(func, &mut call_graph)?;

        // 2. 收集间接调用（函数指针）
        self.collect_indirect_calls(func, &mut call_graph)?;

        // 3. 注入知识库调用链
        self.inject_knowledge_base_calls(func, &mut call_graph)?;

        Ok(call_graph)
    }

    /// 收集直接函数调用
    fn collect_direct_calls(
        &self,
        func: &Function,
        call_graph: &mut CallGraph,
    ) -> Result<(), CallGraphError> {
        for call in func.function_calls() {
            let callee = self.symbol_table.resolve_function(call.name())?;
            if let Some(callee) = callee {
                call_graph.add_edge(func.name(), callee.name(), CallType::Direct);
            }
        }
        Ok(())
    }

    /// 收集间接函数调用（函数指针）
    fn collect_indirect_calls(
        &self,
        func: &Function,
        call_graph: &mut CallGraph,
    ) -> Result<(), CallGraphError> {
        for indirect_call in func.indirect_calls() {
            // 尝试解析函数指针目标
            let targets = self.resolve_function_pointer(&indirect_call)?;

            for target in targets {
                call_graph.add_edge(
                    func.name(),
                    target.name,
                    CallType::Indirect {
                        confidence: target.confidence,
                        source: target.source,
                    },
                );
            }
        }
        Ok(())
    }

    /// 注入知识库中的调用链
    fn inject_knowledge_base_calls(
        &self,
        func: &Function,
        call_graph: &mut CallGraph,
    ) -> Result<(), CallGraphError> {
        // 检查函数是否是已知回调
        if let Some(callback_info) = self.knowledge_base.get_callback_info(func.name()) {
            // 注入从内核框架到用户回调的调用链
            for kernel_call in &callback_info.kernel_call_chain {
                call_graph.add_edge(kernel_call, func.name(), CallType::KernelFramework);
            }
        }

        // 检查函数是否触发异步机制
        if let Some(async_info) = self.knowledge_base.get_async_info(func.name()) {
            // 注入从触发到执行的调用链
            for async_call in &async_info.trigger_to_handler_chain {
                call_graph.add_edge(async_call.trigger, async_call.handler, CallType::Async);
            }
        }

        Ok(())
    }
}

/// 调用类型
pub enum CallType {
    /// 直接调用
    Direct,
    /// 间接调用（函数指针）
    Indirect {
        confidence: Confidence,
        source: TargetSource,
    },
    /// 内核框架调用
    KernelFramework,
    /// 异步调用
    Async,
}

/// 置信度
#[derive(Debug, Clone, Copy)]
pub enum Confidence {
    High = 100,
    Medium = 75,
    Low = 50,
    Uncertain = 25,
    Unknown = 0,
}

/// 目标来源
#[derive(Debug, Clone)]
pub enum TargetSource {
    /// 模式匹配
    PatternMatch,
    /// 结构体字段
    StructField,
    /// 类型匹配
    TypeMatch,
    /// 知识库
    KnowledgeBase,
    /// 未知
    Unknown,
}
```

### 3.3 回调检测

```rust
// flowsight-analysis/src/callback.rs

/// 回调检测器
pub struct CallbackDetector<'a> {
    ast: &'a AST,
    symbol_table: &'a SymbolTable,
    knowledge_base: &'a KnowledgeBase,
}

impl<'a> CallbackDetector<'a> {
    /// 检测函数中的回调注册
    pub fn detect_callbacks(&self, func: &Function) -> Result<Vec<CallbackBinding>, CallbackError> {
        let mut bindings = Vec::new();

        // 1. 检测 ops 表注册
        bindings.extend(self.detect_ops_registration(func)?);

        // 2. 检测异步机制注册
        bindings.extend(self.detect_async_registration(func)?);

        // 3. 检测事件监听注册
        bindings.extend(self.detect_event_registration(func)?);

        Ok(bindings)
    }

    /// 检测 ops 表注册
    fn detect_ops_registration(
        &self,
        func: &Function,
    ) -> Result<Vec<CallbackBinding>, CallbackError> {
        let mut bindings = Vec::new();

        for assignment in func.struct_assignments() {
            // 检查是否是已知的 ops 类型
            if let Some(ops_type) = self.knowledge_base.get_ops_type(&assignment.type_name) {
                // 解析 ops 表字段
                for field_assignment in &assignment.field_assignments {
                    if let Some(callback_name) = ops_type.get_callback_field(&field_assignment.field) {
                        // 检查是否是函数
                        if let Some(target) = self.symbol_table.get_function(&field_assignment.value) {
                            bindings.push(CallbackBinding {
                                callback: target.name.clone(),
                                mechanism: format!("{} ops", ops_type.name),
                                trigger: format!("{} operation", field_assignment.field),
                                registration_line: assignment.line,
                            });
                        }
                    }
                }
            }
        }

        Ok(bindings)
    }

    /// 检测异步机制注册
    fn detect_async_registration(
        &self,
        func: &Function,
    ) -> Result<Vec<CallbackBinding>, CallbackError> {
        let mut bindings = Vec::new();

        for macro_call in func.macro_calls() {
            // 检测 INIT_WORK
            if let Some(captures) = self.knowledge_base.match_pattern("INIT_WORK", &macro_call) {
                bindings.push(CallbackBinding {
                    callback: captures["handler"].clone(),
                    mechanism: "WorkQueue".to_string(),
                    trigger: "schedule_work()".to_string(),
                    registration_line: macro_call.line,
                });
            }

            // 检测 timer_setup
            if let Some(captures) = self.knowledge_base.match_pattern("timer_setup", &macro_call) {
                bindings.push(CallbackBinding {
                    callback: captures["handler"].clone(),
                    mechanism: "Timer".to_string(),
                    trigger: "mod_timer()".to_string(),
                    registration_line: macro_call.line,
                });
            }

            // 检测 request_irq
            if let Some(captures) = self.knowledge_base.match_pattern("request_irq", &macro_call) {
                bindings.push(CallbackBinding {
                    callback: captures["handler"].clone(),
                    mechanism: "IRQ".to_string(),
                    trigger: "硬件中断".to_string(),
                    registration_line: macro_call.line,
                });
            }
        }

        Ok(bindings)
    }
}

/// 回调绑定
pub struct CallbackBinding {
    /// 回调函数名
    pub callback: String,
    /// 机制类型
    pub mechanism: String,
    /// 触发方式
    pub trigger: String,
    /// 注册行号
    pub registration_line: u32,
}
```

### 3.4 异步追踪

```rust
// flowsight-analysis/src/async_tracker.rs

/// 异步机制追踪器
pub struct AsyncTracker<'a> {
    ast: &'a AST,
    symbol_table: &'a SymbolTable,
    knowledge_base: &'a KnowledgeBase,
}

impl<'a> AsyncTracker<'a> {
    /// 追踪异步机制
    pub fn track_async_mechanisms(&self, func: &Function) -> Result<Vec<AsyncBinding>, AsyncError> {
        let mut bindings = Vec::new();

        // 1. 追踪 WorkQueue
        bindings.extend(self.track_workqueue(func)?);

        // 2. 追踪 Timer
        bindings.extend(self.track_timer(func)?);

        // 3. 追踪 IRQ
        bindings.extend(self.track_irq(func)?);

        // 4. 追踪 Threaded IRQ
        bindings.extend(self.track_threaded_irq(func)?);

        Ok(bindings)
    }

    /// 追踪 WorkQueue
    fn track_workqueue(&self, func: &Function) -> Result<Vec<AsyncBinding>, AsyncError> {
        let mut bindings = Vec::new();

        // 查找 INIT_WORK 绑定
        for init_work in func.find_macro_calls("INIT_WORK") {
            let work_var = init_work.get_arg(0)?;
            let handler = init_work.get_arg(1)?;

            bindings.push(AsyncBinding {
                mechanism: AsyncMechanism::WorkQueue,
                variable: work_var.to_string(),
                handler: handler.to_string(),
                context: ExecutionContext::Process,
                trigger: "schedule_work()".to_string(),
                timeline: AsyncTimeline {
                    registration: init_work.line,
                    trigger_event: "schedule_work() 调用后立即返回",
                    execution: "kworker 线程在进程上下文执行",
                },
            });
        }

        // 查找 INIT_DELAYED_WORK
        for init_work in func.find_macro_calls("INIT_DELAYED_WORK") {
            let work_var = init_work.get_arg(0)?;
            let handler = init_work.get_arg(1)?;

            bindings.push(AsyncBinding {
                mechanism: AsyncMechanism::DelayedWork,
                variable: work_var.to_string(),
                handler: handler.to_string(),
                context: ExecutionContext::Process,
                trigger: "queue_delayed_work()".to_string(),
                timeline: AsyncTimeline {
                    registration: init_work.line,
                    trigger_event: "指定时间后触发",
                    execution: "kworker 线程在进程上下文执行",
                },
            });
        }

        Ok(bindings)
    }

    /// 追踪 Timer
    fn track_timer(&self, func: &Function) -> Result<Vec<AsyncBinding>, AsyncError> {
        let mut bindings = Vec::new();

        for timer_setup in func.find_macro_calls("timer_setup") {
            let timer_var = timer_setup.get_arg(0)?;
            let handler = timer_setup.get_arg(1)?;

            bindings.push(AsyncBinding {
                mechanism: AsyncMechanism::Timer,
                variable: timer_var.to_string(),
                handler: handler.to_string(),
                context: ExecutionContext::SoftIRQ,
                trigger: "mod_timer()".to_string(),
                timeline: AsyncTimeline {
                    registration: timer_setup.line,
                    trigger_event: "定时器到期时触发",
                    execution: "TIMER_SOFTIRQ 在软中断上下文执行",
                },
            });
        }

        Ok(bindings)
    }

    /// 追踪 IRQ
    fn track_irq(&self, func: &Function) -> Result<Vec<AsyncBinding>, AsyncError> {
        let mut bindings = Vec::new();

        for request_irq in func.find_function_calls("request_irq") {
            let irq = request_irq.get_arg(0)?;
            let handler = request_irq.get_arg(1)?;

            // 检查是否有 Threaded IRQ 标志
            let is_threaded = request_irq.has_arg_with_value("IRQF_ONESHOT");

            bindings.push(AsyncBinding {
                mechanism: if is_threaded {
                    AsyncMechanism::ThreadedIRQ
                } else {
                    AsyncMechanism::HardIRQ
                },
                variable: format!("irq_{}", irq),
                handler: handler.to_string(),
                context: if is_threaded {
                    ExecutionContext::Process
                } else {
                    ExecutionContext::HardIRQ
                },
                trigger: "硬件中断".to_string(),
                timeline: AsyncTimeline {
                    registration: request_irq.line,
                    trigger_event: "外设产生中断",
                    execution: if is_threaded {
                        "request_threaded_irq 创建内核线程执行"
                    } else {
                        "硬中断上下文，快速处理后返回"
                    },
                },
            });
        }

        Ok(bindings)
    }
}

/// 异步机制类型
pub enum AsyncMechanism {
    WorkQueue,
    DelayedWork,
    Timer,
    HRTimer,
    Tasklet,
    SoftIRQ,
    HardIRQ,
    ThreadedIRQ,
    Completion,
}

/// 异步绑定
pub struct AsyncBinding {
    /// 机制类型
    pub mechanism: AsyncMechanism,
    /// 变量名
    pub variable: String,
    /// 处理函数
    pub handler: String,
    /// 执行上下文
    pub context: ExecutionContext,
    /// 触发方式
    pub trigger: String,
    /// 时序信息
    pub timeline: AsyncTimeline,
}

/// 执行上下文
pub enum ExecutionContext {
    /// 进程上下文（可以睡眠）
    Process,
    /// 软中断上下文（不能睡眠）
    SoftIRQ,
    /// 硬中断上下文（不能睡眠，快速处理）
    HardIRQ,
}

/// 异步时序
pub struct AsyncTimeline {
    /// 注册时机
    pub registration: u32,
    /// 触发事件
    pub trigger_event: String,
    /// 执行时机
    pub execution: String,
}
```

---

## 4. 符号执行模块

### 4.1 KLEE 集成

```rust
// flowsight-symbolic/src/klee.rs

/// KLEE 执行器
pub struct KleeExecutor {
    /// KLEE 路径
    klee_path: PathBuf,
    /// LLVM 路径
    llvm_path: PathBuf,
    /// 工作目录
    work_dir: PathBuf,
    /// 配置
    config: KleeConfig,
}

impl KleeExecutor {
    /// 分析函数的执行路径
    pub fn analyze_function(
        &self,
        llvm_ir: &Path,
        target_function: &str,
        args: &[SymbolicArg],
    ) -> Result<Vec<ExecutionPath>, KleeError> {
        // 1. 生成 KLEE 测试代码
        let test_code = self.generate_klee_test(llvm_ir, target_function, args)?;

        // 2. 编译测试代码
        let test_bc = self.compile_test(&test_code)?;

        // 3. 运行 KLEE
        let klee_output = self.run_klee(&test_bc)?;

        // 4. 解析输出
        let paths = self.parse_output(klee_output)?;

        Ok(paths)
    }

    /// 生成 KLEE 测试代码
    fn generate_klee_test(
        &self,
        llvm_ir: &Path,
        target_function: &str,
        args: &[SymbolicArg],
    ) -> Result<String, KleeError> {
        let mut code = String::new();

        // 头文件
        code.push_str(r#"
#include <klee/klee.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// 外部声明目标函数
extern int target_function(int arg0, int arg1, int arg2);

int main() {
"#);

        // 符号变量声明
        for (i, arg) in args.iter().enumerate() {
            let name = format!("arg_{}", i);

            match &arg.ctype {
                SymbolicType::Int => {
                    code.push_str(&format!("    int {};\n", name));
                    code.push_str(&format!(
                        "    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n",
                        name, name, name
                    ));
                }
                SymbolicType::Ptr => {
                    code.push_str(&format!("    void *{};\n", name));
                    code.push_str(&format!(
                        "    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n",
                        name, name, name
                    ));
                }
                SymbolicType::Array(len) => {
                    code.push_str(&format!("    char {}[{}];\n", name, len));
                    code.push_str(&format!(
                        "    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n",
                        name, name, name
                    ));
                }
            }
        }

        // 调用目标函数
        code.push_str("\n    // 调用目标函数\n");
        let args_str = (0..args.len())
            .map(|i| format!("arg_{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        code.push_str(&format!("    int result = target_function({});\n", args_str));

        // 输出结果
        code.push_str(r#"
    printf("RESULT: %d\n", result);
    printf("PATHS_COMPLETE\n");
    return 0;
}
"#);

        Ok(code)
    }

    /// 编译测试代码
    fn compile_test(&self, code: &str) -> Result<PathBuf, KleeError> {
        // 写入临时文件
        let test_c = self.work_dir.join("test.c");
        std::fs::write(&test_c, code)?;

        // 编译为 LLVM IR
        let test_bc = self.work_dir.join("test.bc");
        let output = std::process::Command::new("clang")
            .args(&[
                "-emit-llvm",
                "-c",
                "-o",
                test_bc.to_str().unwrap(),
                test_c.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(KleeError::CompileFailed(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        Ok(test_bc)
    }

    /// 运行 KLEE
    fn run_klee(&self, test_bc: &Path) -> Result<KleeOutput, KleeError> {
        let klee_out_dir = self.work_dir.join("klee_out");

        let output = std::process::Command::new(&self.klee_path)
            .args(&[
                "--max-paths=100",        // 限制路径数量
                "--max-time=30s",         // 限制时间
                "--output-dir",
                klee_out_dir.to_str().unwrap(),
                test_bc.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(KleeError::RunFailed(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        // 读取 KLEE 输出
        let klee_output = std::fs::read_to_string(klee_out_dir.join("info"))?;
        let messages = std::fs::read_to_string(klee_out_dir.join("messages.txt"))?;

        Ok(KleeOutput {
            stdout: output.stdout,
            info: klee_output,
            messages,
            out_dir: klee_out_dir,
        })
    }

    /// 解析 KLEE 输出
    fn parse_output(&self, klee_output: KleeOutput) -> Result<Vec<ExecutionPath>, KleeError> {
        let mut paths = Vec::new();

        // 解析每条路径的测试用例
        for entry in std::fs::read_dir(klee_output.out_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && path.file_name().unwrap().to_string_lossy().starts_with("test") {
                // 读取 .ktest 文件
                let ktest_file = path.join("test000001.ktest");
                if ktest_file.exists() {
                    let test_case = self.parse_ktest(&ktest_file)?;
                    let path_info = self.parse_info_file(&path.join("info"))?;

                    paths.push(ExecutionPath {
                        id: path.file_name().unwrap().to_string_lossy().to_string(),
                        constraints: test_case.constraints,
                        concrete_args: test_case.values,
                        return_value: path_info.return_value,
                        is_error: path_info.is_error,
                    });
                }
            }
        }

        Ok(paths)
    }
}

/// KLEE 配置
pub struct KleeConfig {
    /// 最大路径数
    pub max_paths: usize,
    /// 最大时间（秒）
    pub max_time: u64,
    /// 最大内存（MB）
    pub max_memory: usize,
    /// 搜索策略
    pub search_strategy: SearchStrategy,
}

/// 搜索策略
pub enum SearchStrategy {
    DFS,
    BFS,
    RandomPath,
    NRandom,
    Coverage,
}
```

### 4.2 路径分析

```rust
// flowsight-symbolic/src/path.rs

/// 路径分析器
pub struct PathAnalyzer {
    /// 知识库
    knowledge_base: KnowledgeBase,
    /// AI 翻译器
    ai_translator: Option<AiTranslator>,
}

impl PathAnalyzer {
    /// 分析执行路径
    pub fn analyze_paths(&self, paths: &[ExecutionPath]) -> Result<PathAnalysis, PathError> {
        let mut analysis = PathAnalysis {
            paths: Vec::new(),
            total_paths: paths.len(),
            unique_constraints: Vec::new(),
        };

        for path in paths {
            let analyzed_path = self.analyze_single_path(path)?;
            analysis.paths.push(analyzed_path);
        }

        // 提取唯一约束
        analysis.unique_constraints = self.extract_unique_constraints(&analysis.paths)?;

        Ok(analysis)
    }

    /// 分析单条路径
    fn analyze_single_path(&self, path: &ExecutionPath) -> Result<AnalyzedPath, PathError> {
        let mut analyzed = AnalyzedPath {
            id: path.id.clone(),
            constraints: Vec::new(),
            condition_translations: Vec::new(),
            steps: self.extract_steps(&path.concrete_args)?,
            return_value: path.return_value.clone(),
            is_error: path.is_error,
        };

        // 翻译约束条件
        for constraint in &path.constraints {
            let translation = self.translate_constraint(constraint)?;
            analyzed.constraints.push(constraint.clone());
            analyzed.condition_translations.push(translation);
        }

        Ok(analyzed)
    }

    /// 翻译约束条件
    fn translate_constraint(&self, constraint: &Constraint) -> Result<ConditionTranslation, PathError> {
        // 如果有 AI 翻译器，使用 AI 翻译
        if let Some(ref translator) = self.ai_translator {
            return translator.translate(constraint);
        }

        // 否则使用规则翻译
        Ok(self.translate_by_rules(constraint))
    }

    /// 提取执行步骤（从具体的参数值推断）
    fn extract_steps(&self, concrete_args: &HashMap<String, String>) -> Result<Vec<ExecutionStep>, PathError> {
        let mut steps = Vec::new();

        // TODO: 从具体值推断执行了哪些分支
        // 这需要结合静态分析的结果

        Ok(steps)
    }

    /// 提取唯一约束
    fn extract_unique_constraints(
        &self,
        paths: &[AnalyzedPath],
    ) -> Result<Vec<ConstraintPattern>, PathError> {
        let mut patterns = Vec::new();

        // 合并所有路径的约束，提取模式
        for path in paths {
            for constraint in &path.constraints {
                let pattern = self.extract_pattern(constraint);
                if !patterns.iter().any(|p| p == &pattern) {
                    patterns.push(pattern);
                }
            }
        }

        Ok(patterns)
    }

    fn extract_pattern(&self, constraint: &Constraint) -> ConstraintPattern {
        ConstraintPattern {
            variable_type: self.infer_variable_type(&constraint.variable),
            operator: constraint.operator.clone(),
            value_pattern: self.extract_value_pattern(&constraint.value),
        }
    }

    fn infer_variable_type(&self, var: &str) -> String {
        // 从变量名推断类型
        if var.contains("ret") || var.contains("result") {
            "返回值".to_string()
        } else if var.contains("status") || var.contains("err") {
            "状态码".to_string()
        } else if var.contains("len") || var.contains("size") {
            "长度".to_string()
        } else {
            "未知".to_string()
        }
    }

    fn extract_value_pattern(&self, value: &str) -> String {
        // 提取值的模式
        match value {
            "-ENODEV" => "设备不存在",
            "-EINVAL" => "参数无效",
            "-ENOMEM" => "内存不足",
            "0" => "成功",
            _ => value,
        }.to_string()
    }

    fn translate_by_rules(&self, constraint: &Constraint) -> ConditionTranslation {
        let meaning = match (constraint.variable.as_str(), constraint.operator.as_str(), constraint.value.as_str()) {
            (_, "==", "-ENODEV") => "设备不匹配",
            (_, "==", "-EINVAL") => "参数无效",
            (_, "==", "-ENOMEM") => "内存不足",
            (_, "==", "0") => "操作成功",
            (_, "!=", "0") => "操作失败",
            (_, ">", "0") => "值大于0",
            (_, "<", "0") => "值小于0",
            _ => format!("{} {} {}", constraint.variable, constraint.operator, constraint.value),
        };

        ConditionTranslation {
            original: format!("{} {} {}", constraint.variable, constraint.operator, constraint.value),
            business_meaning: meaning,
            confidence: Confidence::Low,
        }
    }
}

/// 执行路径
pub struct ExecutionPath {
    pub id: String,
    pub constraints: Vec<Constraint>,
    pub concrete_args: HashMap<String, String>,
    pub return_value: Option<String>,
    pub is_error: bool,
}

/// 约束条件
pub struct Constraint {
    pub variable: String,
    pub operator: String,
    pub value: String,
}

/// 约束翻译
pub struct ConditionTranslation {
    pub original: String,
    pub business_meaning: String,
    pub confidence: Confidence,
}

/// 约束模式
pub struct ConstraintPattern {
    pub variable_type: String,
    pub operator: String,
    pub value_pattern: String,
}

/// 路径分析结果
pub struct PathAnalysis {
    pub paths: Vec<AnalyzedPath>,
    pub total_paths: usize,
    pub unique_constraints: Vec<ConstraintPattern>,
}

/// 分析后的路径
pub struct AnalyzedPath {
    pub id: String,
    pub constraints: Vec<Constraint>,
    pub condition_translations: Vec<ConditionTranslation>,
    pub steps: Vec<ExecutionStep>,
    pub return_value: Option<String>,
    pub is_error: bool,
}

/// 执行步骤
pub struct ExecutionStep {
    pub function: String,
    pub line: u32,
    pub is_kernel: bool,
    pub is_user_code: bool,
}
```

---

## 5. AI 辅助模块

### 5.1 本地模型管理

```rust
// flowsight-ai/src/local_model.rs

/// 本地 AI 模型管理器
pub struct LocalModelManager {
    /// 模型路径
    model_path: PathBuf,
    /// 模型类型
    model_type: ModelType,
    /// llama.cpp 上下文
    llama_ctx: Option<LlamaContext>,
    /// 知识库
    knowledge_base: KnowledgeBase,
    /// 配置
    config: ModelConfig,
}

impl LocalModelManager {
    /// 初始化模型
    pub fn initialize(&mut self) -> Result<(), ModelError> {
        // 检查模型是否存在
        if self.model_path.exists() {
            // 加载本地模型
            self.load_model()?;
        } else {
            // 下载模型
            self.download_model()?;
        }

        Ok(())
    }

    /// 加载本地模型
    fn load_model(&mut self) -> Result<(), ModelError> {
        let params = LlamaModelParams::default()
            .n_ctx(2048)           // 上下文长度
            .n_batch(512)          // 批处理大小
            .n_threads(4);         // 线程数

        let model = LlamaModel::load(
            &self.model_path,
            params,
        )?;

        self.llama_ctx = Some(model.create_context(
            self.model_path.clone(),
            LlamaSamplingParams::default()
                .temperature(0.1)  // 低温度，保证确定性
                .top_k(10)
                .top_p(0.9)
                .repeat_penalty(1.1),
        ));

        Ok(())
    }

    /// 下载模型
    fn download_model(&mut self) -> Result<(), ModelError> {
        // 从服务器下载模型
        let url = "https://models.flowsight.dev/flowsight-linux-1.3b.gguf";

        let response = reqwest::blocking::get(url)?;
        let total_size = response.content_length().unwrap_or(0);

        let mut file = std::fs::File::create(&self.model_path)?;
        let mut downloaded = 0u64;

        let mut reader = response;
        let mut buffer = vec![0u8; 8192];

        while let Ok(bytes) = reader.read(&mut buffer) {
            if bytes == 0 {
                break;
            }
            file.write_all(&buffer[..bytes])?;
            downloaded += bytes as u64;

            // 报告进度
            if let Some(total) = total_size.checked_div(100) {
                if downloaded % (total * 10) == 0 {
                    log::info!("Downloaded {}%", downloaded * 100 / total);
                }
            }
        }

        // 验证文件
        self.verify_model()?;

        // 加载模型
        self.load_model()?;

        Ok(())
    }

    /// 验证模型文件
    fn verify_model(&self) -> Result<(), ModelError> {
        // 检查 SHA256
        let expected_hash = std::fs::read_to_string(self.model_path.with_extension("sha256"))?;
        let actual_hash = self.calculate_sha256()?;

        if expected_hash.trim() != actual_hash {
            return Err(ModelError::HashMismatch);
        }

        Ok(())
    }

    /// 翻译约束条件
    pub fn translate_constraints(
        &self,
        code: &str,
        constraints: &[Constraint],
        context: &str,
    ) -> Result<Vec<ConditionTranslation>, ModelError> {
        let prompt = self.build_translation_prompt(code, constraints, context);

        let response = self.inference(&prompt)?;

        self.parse_translation_response(&response)
    }

    /// 解释业务语义
    pub fn explain_business_semantics(
        &self,
        code: &str,
        function_name: &str,
        execution_context: &str,
    ) -> Result<BusinessExplanation, ModelError> {
        let prompt = self.build_semantics_prompt(code, function_name, execution_context);

        let response = self.inference(&prompt)?;

        self.parse_semantics_response(&response)
    }

    /// 推理
    fn inference(&self, prompt: &str) -> Result<String, ModelError> {
        let ctx = self.llama_ctx.as_ref().expect("Model not initialized");

        let mut output = String::new();

        ctx.inference(prompt, |token| {
            output.push_str(&token);
            true
        })?;

        // 提取 JSON
        self.extract_json(&output)
    }

    /// 构建翻译 Prompt
    fn build_translation_prompt(
        &self,
        code: &str,
        constraints: &[Constraint],
        context: &str,
    ) -> String {
        let constraints_str = constraints
            .iter()
            .map(|c| format!("{} {} {}", c.variable, c.operator, c.value))
            .collect::<Vec<_>>()
            .join("\n");

        let kb_context = self.knowledge_base.get_relevant_context(code);

        format!(
            r###"
### Task: 翻译代码约束条件为业务语义

### Code Context:
```
{}
```

### Constraints:
{}

### Knowledge Base Context:
{}

### Output Format (JSON):
```json
{{
  "translations": [
    {{
      "original": "原始条件",
      "business_meaning": "业务含义",
      "trigger_scenario": "触发场景",
      "confidence": 0.95
    }}
  ]
}}
```

### Response:
"###,
            code, constraints_str, kb_context
        )
    }

    /// 构建语义解释 Prompt
    fn build_semantics_prompt(
        &self,
        code: &str,
        function_name: &str,
        execution_context: &str,
    ) -> String {
        format!(
            r###"
### Task: 解释函数的业务语义

### Function: {}

### Code:
```
{}
```

### Execution Context: {}

### Output Format (JSON):
```json
{{
  "trigger_condition": "函数在什么条件下被调用",
  "business_meaning": "这个函数在业务上做什么",
  "execution_result": "执行结果及含义",
  "related_scenarios": ["相关场景1", "相关场景2"]
}}
```

### Response:
"###,
            function_name, code, execution_context
        )
    }

    /// 解析翻译响应
    fn parse_translation_response(&self, response: &str) -> Result<Vec<ConditionTranslation>, ModelError> {
        // 提取 JSON 部分
        let json = self.extract_json(response)?;

        // 解析 JSON
        let parsed: Translations = serde_json::from_str(&json)?;

        Ok(parsed.translations.into_iter().map(|t| ConditionTranslation {
            original: t.original,
            business_meaning: t.meaning,
            confidence: t.confidence,
        }).collect())
    }

    /// 提取 JSON
    fn extract_json(&self, text: &str) -> Result<String, ModelError> {
        // 查找 ```json ... ``` 或直接的 JSON
        if let Some(start) = text.find("```json") {
            let start = start + 7;
            if let Some(end) = text[start..].find("```") {
                return Ok(text[start..start + end].trim().to_string());
            }
        }

        if let Some(start) = text.find("{") {
            let mut depth = 0;
            let mut in_string = false;
            let mut escaped = false;

            for (i, c) in text[start..].char_indices() {
                if escaped {
                    escaped = false;
                    continue;
                }

                match c {
                    '\\' if in_string => escaped = true,
                    '"' if !escaped => in_string = !in_string,
                    '{' if !in_string => depth += 1,
                    '}' if !in_string => {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(text[start..=start + i].to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        Err(ModelError::ParseFailed("Failed to extract JSON".to_string()))
    }
}

/// 模型配置
pub struct ModelConfig {
    /// 上下文长度
    pub context_length: usize,
    /// 批处理大小
    pub batch_size: usize,
    /// 线程数
    pub threads: usize,
    /// 温度
    pub temperature: f32,
}

/// 翻译响应
#[derive(Deserialize)]
struct Translations {
    translations: Vec<TranslationItem>,
}

#[derive(Deserialize)]
struct TranslationItem {
    original: String,
    business_meaning: String,
    trigger_scenario: String,
    confidence: f64,
}
```

### 5.2 Prompt 管理

```rust
// flowsight-ai/src/prompts.rs

/// Prompt 模板管理器
pub struct PromptManager {
    /// 模板存储
    templates: HashMap<String, PromptTemplate>,
    /// 知识库
    knowledge_base: KnowledgeBase,
}

impl PromptManager {
    /// 获取条件翻译 Prompt
    pub fn get_translation_prompt(
        &self,
        code: &str,
        constraints: &[Constraint],
    ) -> String {
        let template = self.templates.get("translation").expect("Template not found");

        let constraints_str = constraints
            .iter()
            .map(|c| format!("{} {} {}", c.variable, c.operator, c.value))
            .join("\n");

        let kb_context = self.knowledge_base.get_relevant_context(code);

        template.render(serde_json::json!({
            "code": code,
            "constraints": constraints_str,
            "knowledge_base": kb_context,
        }))
    }

    /// 获取语义解释 Prompt
    pub fn get_semantics_prompt(
        &self,
        code: &str,
        function_name: &str,
        context: &str,
    ) -> String {
        let template = self.templates.get("semantics").expect("Template not found");

        template.render(serde_json::json!({
            "code": code,
            "function_name": function_name,
            "execution_context": context,
        }))
    }

    /// 获取触发时机 Prompt
    pub fn get_trigger_prompt(
        &self,
        callback_name: &str,
        mechanism: &str,
    ) -> String {
        let template = self.templates.get("trigger").expect("Template not found");

        template.render(serde_json::json!({
            "callback_name": callback_name,
            "mechanism": mechanism,
        }))
    }
}

/// Prompt 模板
struct PromptTemplate {
    /// 模板内容
    template: String,
    /// 变量
    variables: Vec<String>,
}

impl PromptTemplate {
    fn render(&self, data: serde_json::Value) -> String {
        let mut result = self.template.clone();

        for var in &self.variables {
            if let Some(value) = data.get(var) {
                let value_str = value.as_str().unwrap_or_default();
                result = result.replace(&format!("{{{}}}", var), value_str);
            }
        }

        result
    }
}

/// 默认 Prompt 模板
pub fn default_prompts() -> HashMap<String, PromptTemplate> {
    let mut templates = HashMap::new();

    // 条件翻译模板
    templates.insert("translation".to_string(), PromptTemplate {
        template: r###"
### Task: 将代码约束条件翻译为业务语义

### Code:
```
{code}
```

### Constraints:
{constraints}

### Knowledge Base Context:
{knowledge_base}

### Output Format:
请直接输出 JSON 格式的结果，不要有其他内容。

```json
{
  "translations": [
    {
      "original": "原始条件表达式",
      "business_meaning": "这个条件在业务上代表什么",
      "trigger_scenario": "这个条件满足时会发生什么",
      "confidence": 0.95
    }
  ]
}
```
"###.to_string(),
        variables: vec!["code".to_string(), "constraints".to_string(), "knowledge_base".to_string()],
    });

    // 语义解释模板
    templates.insert("semantics".to_string(), PromptTemplate {
        template: r###"
### Task: 解释函数的业务语义

### Function: {function_name}

### Code:
```
{code}
```

### Execution Context: {execution_context}

### Output Format:
请直接输出 JSON 格式的结果，不要有其他内容。

```json
{
  "trigger_condition": "函数在什么条件下被调用",
  "business_meaning": "这个函数在业务上做什么",
  "execution_result": "执行这个函数会产生什么结果",
  "related_functions": ["相关的函数列表"],
  "common_errors": ["常见的错误情况"]
}
```
"###.to_string(),
        variables: vec!["code".to_string(), "function_name".to_string(), "execution_context".to_string()],
    });

    // 触发时机模板
    templates.insert("trigger".to_string(), PromptTemplate {
        template: r###"
### Task: 解释 {mechanism} 回调的触发时机

### Callback: {callback_name}

### Output Format:
请直接输出 JSON 格式的结果，不要有其他内容。

```json
{
  "trigger_event": "什么事件会触发这个回调",
  "execution_context": "在什么上下文中执行",
  "call_chain": ["从什么函数调用过来"],
  "timeline": "执行时序描述"
}
```
"###.to_string(),
        variables: vec!["callback_name".to_string(), "mechanism".to_string()],
    });

    templates
}
```

---

## 6. 自学习模块

### 6.1 反馈收集

```rust
// flowsight-learning/src/feedback.rs

/// 用户反馈收集器
pub struct FeedbackCollector {
    /// 反馈存储路径
    storage_path: PathBuf,
    /// 反馈队列
    queue: Vec<UserFeedback>,
    /// 批处理大小
    batch_size: usize,
}

impl FeedbackCollector {
    /// 收集正确性反馈
    pub fn collect_correctness_feedback(
        &mut self,
        function: &str,
        path_id: &str,
        is_correct: bool,
        correction: Option<String>,
    ) {
        let feedback = UserFeedback::Correctness {
            function: function.to_string(),
            path_id: path_id.to_string(),
            is_correct,
            correction,
        };

        self.queue.push(feedback);

        // 如果队列满了，保存到磁盘
        if self.queue.len() >= self.batch_size {
            self.flush();
        }
    }

    /// 收集知识补充反馈
    pub fn collect_knowledge_feedback(
        &mut self,
        code_pattern: &str,
        explanation: &str,
        trigger_condition: &str,
    ) {
        let feedback = UserFeedback::KnowledgeSupplement {
            code_pattern: code_pattern.to_string(),
            explanation: explanation.to_string(),
            trigger_condition: trigger_condition.to_string(),
        };

        self.queue.push(feedback);
    }

    /// 收集新模式反馈
    pub fn collect_pattern_feedback(
        &mut self,
        code_snippet: &str,
        pattern_type: &str,
        semantics: &str,
    ) {
        let feedback = UserFeedback::NewPattern {
            code_snippet: code_snippet.to_string(),
            pattern_type: pattern_type.to_string(),
            semantics: semantics.to_string(),
        };

        self.queue.push(feedback);
    }

    /// 刷新队列到磁盘
    fn flush(&mut self) {
        if self.queue.is_empty() {
            return;
        }

        // 追加到文件
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.storage_path)
            .expect("Failed to open feedback file");

        for feedback in &self.queue {
            let json = serde_json::to_string(feedback).expect("Failed to serialize");
            writeln!(file, "{}", json).expect("Failed to write");
        }

        self.queue.clear();
    }
}

/// 用户反馈类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserFeedback {
    /// 正确性反馈
    Correctness {
        function: String,
        path_id: String,
        is_correct: bool,
        correction: Option<String>,
        timestamp: i64,
    },
    /// 知识补充反馈
    KnowledgeSupplement {
        code_pattern: String,
        explanation: String,
        trigger_condition: String,
        timestamp: i64,
    },
    /// 新模式反馈
    NewPattern {
        code_snippet: String,
        pattern_type: String,
        semantics: String,
        timestamp: i64,
    },
}

impl UserFeedback {
    /// 转换为训练样本
    pub fn to_training_sample(&self) -> TrainingSample {
        match self {
            UserFeedback::Correctness { function, path_id, is_correct, .. } => {
                TrainingSample {
                    instruction: format!("分析函数 {} 的执行路径是否正确", function),
                    input: path_id.clone(),
                    output: if *is_correct {
                        "正确".to_string()
                    } else {
                        correction.clone().unwrap_or_else(|| "不正确".to_string())
                    },
                    feedback_type: "correctness".to_string(),
                }
            }
            UserFeedback::KnowledgeSupplement { code_pattern, explanation, .. } => {
                TrainingSample {
                    instruction: "解释以下代码模式的业务语义".to_string(),
                    input: code_pattern.clone(),
                    output: explanation.clone(),
                    feedback_type: "knowledge".to_string(),
                }
            }
            UserFeedback::NewPattern { code_snippet, semantics, .. } => {
                TrainingSample {
                    instruction: "识别以下代码中的模式并解释其语义".to_string(),
                    input: code_snippet.clone(),
                    output: semantics.clone(),
                    feedback_type: "pattern".to_string(),
                }
            }
        }
    }
}

/// 训练样本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    pub instruction: String,
    pub input: String,
    pub output: String,
    pub feedback_type: String,
}
```

### 6.2 本地训练

```rust
// flowsight-learning/src/training.rs

/// 本地训练器
pub struct LocalTrainer {
    /// 模型路径
    model_path: PathBuf,
    /// LoRA 路径
    lora_path: PathBuf,
    /// 配置
    config: TrainingConfig,
}

impl LocalTrainer {
    /// 增量训练
    pub fn incremental_train(&self, samples: &[TrainingSample]) -> Result<(), TrainingError> {
        // 检查是否有 GPU
        if !self.has_gpu() {
            // CPU 模式：仅更新知识库
            self.update_knowledge_base(samples)?;
            return Ok(());
        }

        // GPU 模式：执行 QLoRA 训练
        self.run_qlora_training(samples)?;

        Ok(())
    }

    /// 检查是否有 GPU
    fn has_gpu(&self) -> bool {
        // 检查 CUDA 是否可用
        rust_gpu::is_available()
    }

    /// 运行 QLoRA 训练
    fn run_qlora_training(&self, samples: &[TrainingSample]) -> Result<(), TrainingError> {
        // 生成训练数据
        let train_data = self.prepare_training_data(samples)?;

        // 构建训练脚本
        let script = self.build_training_script(&train_data)?;

        // 执行训练
        let output = std::process::Command::new("python")
            .arg("-c")
            .arg(script)
            .output()?;

        if !output.status.success() {
            return Err(TrainingError::TrainingFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // 保存 LoRA 权重
        self.save_lora_adapter()?;

        Ok(())
    }

    /// 准备训练数据
    fn prepare_training_data(&self, samples: &[TrainingSample]) -> Result<String, TrainingError> {
        // 转换为训练格式
        let formatted: Vec<FormattedSample> = samples
            .iter()
            .map(|s| FormattedSample {
                text: format!(
                    "### Instruction:\n{}\n\n### Input:\n{}\n\n### Output:\n{}",
                    s.instruction, s.input, s.output
                ),
            })
            .collect();

        Ok(serde_json::to_string(&formatted)?)
    }

    /// 构建训练脚本
    fn build_training_script(&self, train_data: &str) -> Result<String, TrainingError> {
        Ok(format!(
            r#"
import json
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, TrainingArguments, Trainer
from peft import LoraConfig, get_peft_model, prepare_model_for_kbit_training

# 加载模型
model_path = "{}"
model = AutoModelForCausalLM.from_pretrained(
    model_path,
    torch_dtype=torch.float16,
    device_map="auto"
)
tokenizer = AutoTokenizer.from_pretrained(model_path)

# 准备 QLoRA
model = prepare_model_for_kbit_training(model)
lora_config = LoraConfig(
    r=16,
    lora_alpha=32,
    target_modules=["q_proj", "k_proj", "v_proj"],
    lora_dropout=0.05,
    bias="none",
)
model = get_peft_model(model, lora_config)

# 准备训练数据
train_data = json.loads('''{}''')

# 训练
trainer = Trainer(
    model=model,
    args=TrainingArguments(
        num_train_epochs=1,
        per_device_train_batch_size=4,
        learning_rate=2e-4,
        output_dir="{}",
    ),
    train_dataset=train_data,
)
trainer.train()

# 保存 LoRA
model.save_horizontal_adapter("{}", "user_feedback")
print("Training completed!")
"#,
            self.model_path.display(),
            train_data,
            self.lora_path.display(),
            self.lora_path.display()
        ))
    }

    /// 保存 LoRA 适配器
    fn save_lora_adapter(&self) -> Result<(), TrainingError> {
        // LoRA 权重已在前面的脚本中保存
        Ok(())
    }

    /// 更新知识库（CPU 模式）
    fn update_knowledge_base(&self, samples: &[TrainingSample]) -> Result<(), TrainingError> {
        // 从反馈中提取知识
        for sample in samples {
            if let TrainingSample {
                instruction: _,
                input,
                output,
                feedback_type: _,
            } = sample
            {
                // 更新知识库缓存
                // 实际的更新逻辑在 KnowledgeBase 模块中
            }
        }

        Ok(())
    }
}

/// 训练配置
pub struct TrainingConfig {
    /// 学习率
    pub learning_rate: f32,
    /// 批次大小
    pub batch_size: usize,
    /// 训练轮数
    pub epochs: usize,
    /// 最大序列长度
    pub max_seq_length: usize,
}

/// 格式化后的样本
struct FormattedSample {
    text: String,
}
```

### 6.3 知识上传

```rust
// flowsight-learning/src/upload.rs

/// 知识上传器
pub struct KnowledgeUploader {
    /// 服务器 URL
    server_url: String,
    /// 客户端
    client: reqwest::blocking::Client,
}

impl KnowledgeUploader {
    /// 匿名上传知识
    pub fn upload_anonymized(
        &self,
        knowledge: &AnonymizedKnowledge,
        user_consent: bool,
    ) -> Result<(), UploadError> {
        if !user_consent {
            return Err(UploadError::NoConsent);
        }

        let response = self.client
            .post(&format!("{}/api/knowledge/upload", self.server_url))
            .json(knowledge)
            .send()?;

        if !response.status().is_success() {
            return Err(UploadError::ServerError(response.text()?));
        }

        Ok(())
    }
}

/// 匿名化后的知识
#[derive(Debug, Serialize, Deserialize)]
pub struct AnonymizedKnowledge {
    /// 模式（移除具体代码）
    pub pattern: String,
    /// 语义解释
    pub semantics: String,
    /// 触发条件
    pub trigger_condition: String,
    /// 统计信息
    pub stats: KnowledgeStats,
}

/// 知识统计
#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeStats {
    /// 上报次数
    pub report_count: u32,
    /// 置信度
    pub confidence: f32,
    /// 时间戳
    pub timestamp: i64,
}
```

---

## 7. IDE 展示层

### 7.1 执行流面板

```typescript
// app/src/components/ExecutionFlowPanel.tsx

interface ExecutionFlowPanelProps {
  functionName: string;
  analysis: FunctionAnalysis;
  onFunctionClick: (func: string) => void;
  onPathSelect: (pathId: string) => void;
}

export function ExecutionFlowPanel({
  functionName,
  analysis,
  onFunctionClick,
  onPathSelect,
}: ExecutionFlowPanelProps) {
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'ftrace' | 'graph' | 'tree'>('ftrace');

  return (
    <div className="execution-flow-panel">
      {/* 头部 */}
      <PanelHeader
        functionName={functionName}
        executionContext={analysis.execution_context}
        pathCount={analysis.callGraph.paths.length}
      />

      {/* 视图切换 */}
      <ViewSwitcher value={viewMode} onChange={setViewMode} />

      {/* 执行流视图 */}
      <div className="flow-content">
        {viewMode === 'ftrace' && (
          <FtraceView
            analysis={analysis}
            selectedPath={selectedPath}
            onPathSelect={setSelectedPath}
            onFunctionClick={onFunctionClick}
          />
        )}

        {viewMode === 'graph' && (
          <GraphView
            analysis={analysis}
            onFunctionClick={onFunctionClick}
          />
        )}

        {viewMode === 'tree' && (
          <TreeView
            analysis={analysis}
            onFunctionClick={onFunctionClick}
          />
        )}
      </div>

      {/* 回调列表 */}
      <CallbacksSection callbacks={analysis.callbacks} />

      {/* 异步绑定 */}
      <AsyncBindingsSection bindings={analysis.asyncBindings} />
    </div>
  );
}

/// ftrace 风格视图
function FtraceView({
  analysis,
  selectedPath,
  onPathSelect,
  onFunctionClick,
}: {
  analysis: FunctionAnalysis;
  selectedPath: string | null;
  onPathSelect: (path: string) => void;
  onFunctionClick: (func: string) => void;
}) {
  return (
    <div className="ftrace-view">
      <div className="ftrace-header">
        <span className="cpu-col">CPU</span>
        <span className="depth-col"></span>
        <span className="function-col">Function</span>
      </div>

      <div className="ftrace-body">
        {/* 主函数 */}
        <FunctionRow
          function={analysis.function.name}
          depth={0}
          isUserCode={true}
          onClick={() => onFunctionClick(analysis.function.name)}
        />

        {/* 直接调用 */}
        {analysis.callGraph.directCalls().map((call, i) => (
          <FunctionRow
            key={i}
            function={call.callee}
            depth={1}
            isKernel={call.isKernel}
            onClick={() => onFunctionClick(call.callee)}
          />
        ))}

        {/* 异步回调 */}
        {analysis.asyncBindings.map((binding, i) => (
          <AsyncRow
            key={i}
            binding={binding}
            onClick={() => onFunctionClick(binding.handler)}
          />
        ))}
      </div>
    </div>
  );
}

/// 函数行组件
function FunctionRow({
  function: funcName,
  depth,
  isKernel,
  isUserCode,
  onClick,
}: {
  function: string;
  depth: number;
  isKernel?: boolean;
  isUserCode?: boolean;
  onClick: () => void;
}) {
  return (
    <div className="function-row">
      <span className="cpu-col">0)</span>
      <span className="depth-col">
        {'  '.repeat(depth)}
        {'='.repeat(depth)}
      </span>
      <span
        className={`function-col ${isKernel ? 'kernel-api' : ''} ${isUserCode ? 'user-code' : ''}`}
        onClick={onClick}
      >
        {funcName}()
        {isKernel && <span className="badge kernel">内核</span>}
        {isUserCode && <span className="badge user">用户</span>}
      </span>
    </div>
  );
}

/// 异步行组件
function AsyncRow({ binding, onClick }: { binding: AsyncBinding; onClick: () => void }) {
  return (
    <div className="async-row">
      <span className="cpu-col">0)</span>
      <span className="depth-col">async</span>
      <span className="async-content">
        <span className="async-type">{binding.mechanism}</span>
        <span className="async-arrow">→</span>
        <span className="async-handler" onClick={onClick}>
          {binding.handler}()
        </span>
        <span className="async-context">[{binding.context}]</span>
      </span>
      <div className="async-timeline">
        {binding.timeline.trigger_event}
      </div>
    </div>
  );
}
```

### 7.2 调用图面板

```typescript
// app/src/components/CallGraphPanel.tsx

interface CallGraphPanelProps {
  callGraph: CallGraph;
  onNodeClick: (nodeId: string) => void;
}

export function CallGraphPanel({ callGraph, onNodeClick }: CallGraphPanelProps) {
  const nodes = useMemo(() => {
    return callGraph.nodes.map((node) => ({
      id: node.id,
      position: { x: node.x, y: node.y },
      data: { label: node.name },
      type: node.isUserCode ? 'userNode' : 'kernelNode',
    }));
  }, [callGraph]);

  const edges = useMemo(() => {
    return callGraph.edges.map((edge, i) => ({
      id: `e${i}`,
      source: edge.from,
      target: edge.to,
      type: edge.callType === 'async' ? 'asyncEdge' : 'default',
      animated: edge.callType === 'async',
      label: edge.callType,
    }));
  }, [callGraph]);

  return (
    <div className="call-graph-panel">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodeClick={(_, node) => onNodeClick(node.id)}
        fitView
        fitViewOptions={{ padding: 0.2 }}
      >
        <Background />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  );
}
```

### 7.3 样式

```css
/* app/src/styles/execution-flow.css */

.execution-flow-panel {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  background: #1e1e1e;
  color: #d4d4d4;
  border-radius: 8px;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: #252526;
  border-bottom: 1px solid #3c3c3c;
}

.panel-header h2 {
  margin: 0;
  font-size: 16px;
  color: #dcdcaa;
}

.ftrace-view {
  padding: 8px 0;
}

.ftrace-header {
  display: flex;
  padding: 8px 16px;
  color: #569cd6;
  font-size: 12px;
  font-weight: 600;
  border-bottom: 1px solid #3c3c3c;
}

.ftrace-body {
  max-height: 400px;
  overflow-y: auto;
}

.function-row {
  display: flex;
  align-items: center;
  padding: 4px 16px;
  line-height: 1.6;
  cursor: pointer;
}

.function-row:hover {
  background: #2a2d2e;
}

.cpu-col {
  width: 40px;
  color: #6a9955;
  flex-shrink: 0;
}

.depth-col {
  width: 60px;
  color: #808080;
  flex-shrink: 0;
}

.function-col {
  color: #dcdcaa;
  flex: 1;
}

.function-col.kernel-api {
  color: #4ec9b0;
}

.function-col.user-code {
  color: #dcdcaa;
}

.badge {
  display: inline-block;
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 4px;
  margin-left: 8px;
}

.badge.kernel {
  background: #264f78;
  color: #9cdcfe;
}

.badge.user {
  background: #3c6142;
  color: #9cdcfe;
}

.async-row {
  padding: 8px 16px;
  border-top: 1px solid #3c3c3c;
  background: #1a1a1a;
}

.async-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.async-type {
  background: #c586c0;
  color: #1e1e1e;
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
}

.async-arrow {
  color: #569cd6;
}

.async-handler {
  color: #dcdcaa;
  cursor: pointer;
}

.async-handler:hover {
  text-decoration: underline;
}

.async-context {
  color: #808080;
  font-size: 11px;
}

.async-timeline {
  margin-top: 4px;
  margin-left: 108px;
  color: #808080;
  font-size: 11px;
}

.condition-translation {
  padding: 8px 16px;
  background: #2d2d30;
  border-left: 3px solid #007acc;
  margin: 8px 16px;
}

.condition-translation .label {
  color: #569cd6;
  font-size: 12px;
}

.condition-translation .translation {
  color: #ce9178;
  font-size: 13px;
}
```

---

## 8. 数据格式

### 8.1 分析结果格式

```json
{
  "function": {
    "name": "my_probe",
    "file": "drivers/usb/my_driver.c",
    "line": 42,
    "execution_context": "process"
  },
  "call_graph": {
    "nodes": [
      {"id": "my_probe", "type": "user"},
      {"id": "kzalloc", "type": "kernel"},
      {"id": "usb_set_intfdata", "type": "kernel"}
    ],
    "edges": [
      {"from": "my_probe", "to": "kzalloc", "type": "direct"},
      {"from": "my_probe", "to": "usb_set_intfdata", "type": "direct"}
    ]
  },
  "callbacks": [
    {
      "callback": "my_probe",
      "mechanism": "usb_driver ops",
      "trigger": "USB device insert with matching ID",
      "registration_line": 100
    }
  ],
  "async_bindings": [
    {
      "mechanism": "WorkQueue",
      "variable": "dev->work",
      "handler": "work_handler",
      "context": "process",
      "trigger": "schedule_work()",
      "timeline": {
        "registration": 50,
        "trigger_event": "Data ready interrupt",
        "execution": "kworker thread"
      }
    }
  ],
  "paths": [
    {
      "id": "path_1",
      "constraints": [
        {"variable": "ret", "operator": "==", "value": "0"}
      ],
      "condition_translations": [
        {
          "original": "ret == 0",
          "business_meaning": "设备初始化成功",
          "confidence": 0.95
        }
      ],
      "steps": [
        {"function": "kzalloc", "line": 45},
        {"function": "usb_set_intfdata", "line": 50}
      ],
      "return_value": "0"
    }
  ]
}
```

### 8.2 知识库格式

```yaml
# knowledge/platforms/linux-kernel/core/workqueue.yaml

async_patterns:
  work_struct:
    description: "Linux 工作队列机制"

    bind_patterns:
      - pattern: "INIT_WORK\\(&(\\w+),\\s*(\\w+)\\)"
        groups:
          work: "$1"
          handler: "$2"

    trigger_patterns:
      - pattern: "schedule_work\\(&(\\w+)\\)"
        groups:
          work: "$1"

    kernel_call_chain:
      - function: "schedule_work"
        file: "kernel/workqueue.c"
        line: 2500
      - function: "__queue_work"
        file: "kernel/workqueue.c"
      - function: "insert_work"
        file: "kernel/workqueue.c"
      - function: "wake_up_process"
        file: "kernel/sched/core.c"
      - function: "worker_thread"
        file: "kernel/workqueue.c"
      - function: "process_one_work"
        file: "kernel/workqueue.c"
      - function: "$handler"  # 用户回调
        user_entry: true

    timeline:
      registration: "INIT_WORK() 在 probe 中调用"
      trigger: "schedule_work() 被调用后立即返回"
      execution: "worker 线程在进程上下文执行"

  delayed_work:
    description: "延迟工作队列"

    bind_patterns:
      - pattern: "INIT_DELAYED_WORK\\(&(\\w+),\\s*(\\w+)\\)"
        groups:
          work: "$1"
          handler: "$2"

    trigger_patterns:
      - pattern: "queue_delayed_work\\(&(\\w+),\\s*(\\w+)\\)"
        groups:
          queue: "$1"
          work: "$2"
```

---

## 9. API 设计

### 9.1 Tauri RPC API

```typescript
// app/src-tauri/src/api.rs

#[tauri::module]
pub struct AnalyzerApi {
  // 分析 API
  pub async fn analyze_function(
    file: String,
    function: String,
    options: AnalyzeOptions,
  ) -> Result<FunctionAnalysis, String>;

  // 调用图 API
  pub async fn get_call_graph(
    function: String,
    depth: Option<u32>,
  ) -> Result<CallGraph, String>;

  // 回调 API
  pub async fn get_callbacks(
    function: String,
  ) -> Result<Vec<CallbackBinding>, String>;

  // 异步绑定 API
  pub async fn get_async_bindings(
    function: String,
  ) -> Result<Vec<AsyncBinding>, String>;

  // 符号执行 API
  pub async fn run_symbolic_execution(
    function: String,
    args: Vec<SymbolicArg>,
    options: SymbolicOptions,
  ) -> Result<PathAnalysis, String>;

  // AI 翻译 API
  pub async fn translate_constraints(
    code: String,
    constraints: Vec<Constraint>,
    context: String,
  ) -> Result<Vec<ConditionTranslation>, String>;

  // 反馈 API
  pub async fn submit_feedback(
    feedback: UserFeedback,
  ) -> Result<(), String>;

  // 知识库 API
  pub async fn get_knowledge_context(
    code: String,
  ) -> Result<String, String>;
}

/// 分析选项
struct AnalyzeOptions {
  /// 最大深度
  max_depth: Option<u32>,
  /// 是否包含异步
  include_async: Option<bool>,
  /// 是否运行符号执行
  run_symbolic: Option<bool>,
}

/// 符号执行选项
struct SymbolicOptions {
  /// 最大路径数
  max_paths: Option<usize>,
  /// 最大时间（秒）
  max_time: Option<u64>,
  /// 符号参数
  symbolic_args: Vec<SymbolicArg>,
}
```

### 9.2 内部模块 API

```rust
// flowsight-analysis/src/lib.rs

/// 静态分析器 trait
pub trait Analyzer {
    /// 分析函数
    fn analyze(&self, file: &Path, function: &str) -> Result<FunctionAnalysis, AnalyzerError>;

    /// 构建调用图
    fn build_call_graph(&self, function: &str) -> Result<CallGraph, AnalyzerError>;

    /// 检测回调
    fn detect_callbacks(&self, function: &str) -> Result<Vec<CallbackBinding>, AnalyzerError>;

    /// 追踪异步机制
    fn track_async(&self, function: &str) -> Result<Vec<AsyncBinding>, AnalyzerError>;
}

// flowsight-symbolic/src/lib.rs

/// 符号执行器 trait
pub trait SymbolicExecutor {
    /// 执行符号分析
    fn execute(
        &self,
        llvm_ir: &Path,
        function: &str,
        args: &[SymbolicArg],
    ) -> Result<PathAnalysis, SymbolicError>;

    /// 生成测试用例
    fn generate_test_cases(&self, path: &ExecutionPath) -> Result<Vec<TestCase>, SymbolicError>;
}

// flowsight-ai/src/lib.rs

/// AI 翻译器 trait
pub trait AiTranslator {
    /// 翻译约束条件
    fn translate_constraints(
        &self,
        code: &str,
        constraints: &[Constraint],
        context: &str,
    ) -> Result<Vec<ConditionTranslation>, AiError>;

    /// 解释业务语义
    fn explain_semantics(
        &self,
        code: &str,
        function: &str,
        context: &str,
    ) -> Result<BusinessExplanation, AiError>;
}
```

---

## 10. 性能优化

### 10.1 分析性能

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          性能优化策略                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 增量分析                                                             │
│  ────────────                                                           │
│  • 文件修改后只重新分析受影响的函数                                      │
│  • 缓存符号表和 AST                                                     │
│  • 维护函数依赖图                                                       │
│                                                                          │
│  2. 并行处理                                                             │
│  ────────────                                                           │
│  • Tree-sitter 并行解析                                                 │
│  • 多文件符号表构建并行化                                               │
│  • KLEE 支持并行路径探索                                                │
│                                                                          │
│  3. 缓存策略                                                             │
│  ────────────                                                           │
│  • L1: 内存缓存（热点函数）                                             │
│  • L2: SQLite 缓存（文件级结果）                                        │
│  • L3: 磁盘缓存（项目级结果）                                           │
│                                                                          │
│  4. 懒加载                                                               │
│  ────────────                                                           │
│  • 按需加载符号表                                                       │
│  • 按需展开调用链                                                       │
│  • 按需执行符号执行                                                     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 10.2 AI 推理性能

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          AI 推理优化                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 模型量化                                                             │
│  ────────────                                                           │
│  • 使用 Q4_K_M 量化 (~0.8GB)                                           │
│  • 平衡质量和大小                                                        │
│                                                                          │
│  2. 批处理                                                               │
│  ────────────                                                           │
│  • 合并多个翻译请求                                                      │
│  • 减少推理次数                                                          │
│                                                                          │
│  3. 缓存                                                               │
│  ────────────                                                           │
│  • 缓存常见约束的翻译结果                                                │
│  • LRU 淘汰策略                                                          │
│                                                                          │
│  4. 预加载                                                               │
│  ────────────                                                           │
│  • 后台预加载模型                                                        │
│  • 用户空闲时预热                                                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 10.3 内存优化

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          内存优化策略                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 内存池                                                               │
│  ────────────                                                           │
│  • 频繁分配的对象使用内存池                                              │
│  • 减少内存碎片                                                          │
│                                                                          │
│  2. 引用计数                                                             │
│  ────────────                                                           │
│  • AST 节点使用 Rc/Arc                                                  │
│  • 共享子节点                                                            │
│                                                                          │
│  3. 流式处理                                                             │
│  ────────────                                                           │
│  • 大文件流式解析                                                        │
│  • 避免一次性加载                                                        │
│                                                                          │
│  4. 及时释放                                                             │
│  ────────────                                                           │
│  • 临时对象及时 drop                                                     │
│  • 显式清理缓存                                                          │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 11. 测试策略

### 11.1 测试层次

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          测试金字塔                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│                            ┌─────────┐                                  │
│                            │ 端到端  │  5%                               │
│                            │  测试   │                                   │
│                            └────┬────┘                                  │
│                                 │                                       │
│                     ┌───────────┴───────────┐                           │
│                     │       集成测试         │  25%                      │
│                     │   (模块间交互)        │                            │
│                     └───────────┬───────────┘                           │
│                                 │                                       │
│              ┌──────────────────┴──────────────────┐                    │
│              │           单元测试                  │  70%               │
│              │      (每个模块内部逻辑)            │                     │
│              └────────────────────────────────────┘                    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 11.2 测试用例

```rust
// tests/kernel_analysis_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    use flowsight_analysis::StaticAnalyzer;
    use std::path::Path;

    #[test]
    fn test_simple_driver_analysis() {
        let analyzer = StaticAnalyzer::new();
        let result = analyzer.analyze_function(
            Path::new("tests/fixtures/simple_driver.c"),
            "my_probe",
        ).unwrap();

        // 验证函数基本信息
        assert_eq!(result.function.name, "my_probe");
        assert!(result.function.line > 0);

        // 验证回调检测
        assert!(result.callbacks.iter().any(|b| {
            b.callback == "my_probe" && b.mechanism.contains("usb_driver")
        }));

        // 验证异步绑定
        assert!(result.async_bindings.iter().any(|b| {
            b.handler == "my_work_handler" && b.mechanism == "WorkQueue"
        }));
    }

    #[test]
    fn test_call_graph() {
        let analyzer = StaticAnalyzer::new();
        let call_graph = analyzer.build_call_graph(
            Path::new("tests/fixtures/simple_driver.c"),
            "my_probe",
        ).unwrap();

        // 验证调用关系
        assert!(call_graph.has_edge("my_probe", "kzalloc"));
        assert!(call_graph.has_edge("my_probe", "INIT_WORK"));
    }

    #[test]
    fn test_workqueue_pattern() {
        let analyzer = StaticAnalyzer::new();
        let bindings = analyzer.track_async_mechanisms(
            Path::new("tests/fixtures/simple_driver.c"),
            "my_probe",
        ).unwrap();

        // 验证 workqueue 绑定
        let work = bindings.iter()
            .find(|b| b.mechanism == "WorkQueue")
            .expect("Should find WorkQueue binding");

        assert_eq!(work.handler, "my_work_handler");
        assert_eq!(work.context, ExecutionContext::Process);
    }
}
```

### 11.3 性能测试

```rust
// tests/performance_test.rs

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_analysis_performance() {
        let analyzer = StaticAnalyzer::new();
        let start = Instant::now();

        let result = analyzer.analyze_function(
            Path::new("tests/fixtures/driver_1000.c"),
            "probe",
        ).unwrap();

        let elapsed = start.elapsed();

        // 单函数分析应在 5 秒内完成
        assert!(elapsed < Duration::from_secs(5),
            "Analysis took {} seconds, expected < 5", elapsed.as_secs_f64());

        // 内存使用应在 2GB 以下
        let memory = get_memory_usage();
        assert!(memory < 2 * 1024 * 1024 * 1024,
            "Memory usage {} bytes, expected < 2GB", memory);
    }

    #[test]
    fn test_batch_analysis() {
        let analyzer = StaticAnalyzer::new();
        let files = vec![
            "tests/fixtures/driver1.c",
            "tests/fixtures/driver2.c",
            "tests/fixtures/driver3.c",
        ];

        let start = Instant::now();

        for file in &files {
            let _ = analyzer.analyze_function(Path::new(file), "probe");
        }

        let elapsed = start.elapsed();

        // 批量分析应在 30 秒内完成
        assert!(elapsed < Duration::from_secs(30),
            "Batch analysis took {} seconds, expected < 30", elapsed.as_secs_f64());
    }
}
```

---

## 附录

### A. 参考资源

- [KLEE 官方文档](https://klee.github.io/)
- [llama.cpp](https://github.com/ggerganov/llama.cpp)
- [Tree-sitter](https://tree-sitter.github.io/tree-sitter/)
- [Rust 异步编程](https://rust-lang.github.io/async-book/)

### B. 术语表

| 术语 | 定义 |
|-----|------|
| 符号执行 | 使用符号值执行程序，探索所有可能路径 |
| KLEE | LLVM 上的符号执行引擎 |
| llama.cpp | C++ 实现的 LLM 推理库 |
| LoRA | 低秩适应，用于模型微调 |
| AST | 抽象语法树 |
| CFG | 控制流图 |

### C. 变更日志

| 日期 | 版本 | 变更 |
|-----|------|-----|
| 2025-01-21 | v2.0 | 重新设计，整合符号执行和本地 AI |
| - | v1.0 | 初始设计（存档） |
