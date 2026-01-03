# 🌐 FlowSight 多语言统一抽象层设计

> 本文档描述 FlowSight 如何设计统一的抽象层来支持多种编程语言的代码分析，实现"一次设计，多语言适用"的目标。

---

## 目录

1. [设计目标](#1-设计目标)
2. [整体架构](#2-整体架构)
3. [核心抽象模型](#3-核心抽象模型)
4. [语言适配器](#4-语言适配器)
5. [统一 IR 设计](#5-统一-ir-设计)
6. [跨语言分析](#6-跨语言分析)
7. [扩展新语言](#7-扩展新语言)
8. [案例分析](#8-案例分析)

---

## 1. 设计目标

### 1.1 核心理念

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        设计原则                                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 抽象共性，保留特性                                                   │
│     ├── 所有语言都有：函数、类型、调用、控制流                           │
│     ├── 语言特性：协程(Kotlin)、泛型(C++)、反射(Java)                   │
│     └── 统一模型表达共性，扩展点处理特性                                 │
│                                                                          │
│  2. 渐进式解析精度                                                       │
│     ├── L1: 语法树级别（所有语言）                                       │
│     ├── L2: 类型信息（需要语义分析）                                     │
│     └── L3: 完整语义（需要编译器支持）                                   │
│                                                                          │
│  3. 插件化语言支持                                                       │
│     ├── 核心引擎与语言解析分离                                           │
│     ├── 新语言只需实现 LanguageAdapter trait                            │
│     └── 可以热加载语言支持模块                                           │
│                                                                          │
│  4. 性能优先                                                             │
│     ├── 使用 tree-sitter 做快速解析                                     │
│     ├── 按需进行深度分析                                                 │
│     └── 增量更新                                                         │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 支持语言优先级

| 优先级 | 语言 | 复杂度 | 主要场景 |
|--------|------|--------|----------|
| P0 | C | ★★★☆☆ | Linux内核、嵌入式 |
| P0 | C++ | ★★★★★ | Android Native、系统软件 |
| P1 | Java | ★★★☆☆ | Android Framework、后端 |
| P1 | Kotlin | ★★★★☆ | Android App |
| P2 | Rust | ★★★★☆ | 系统软件 |
| P2 | Go | ★★☆☆☆ | 云原生 |
| P3 | Python | ★★☆☆☆ | 脚本、AI |
| P3 | JavaScript/TypeScript | ★★★☆☆ | 前端、Node.js |

---

## 2. 整体架构

### 2.1 分层架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        FlowSight 多语言架构                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                         应用层 (Application)                       │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐   │  │
│  │  │ 执行流可视化 │  │ 调用图生成  │  │    异步追踪              │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘   │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                   │                                      │
│                                   ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                     分析层 (Analysis Engine)                       │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐   │  │
│  │  │ 指针分析     │  │ 数据流分析  │  │    模式匹配              │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘   │  │
│  │                        │                                          │  │
│  │                        ▼ 操作统一 IR                              │  │
│  │  ┌───────────────────────────────────────────────────────────┐   │  │
│  │  │              Unified IR (统一中间表示)                      │   │  │
│  │  │  • FlowNode, FlowEdge, Symbol, Type, CallSite             │   │  │
│  │  └───────────────────────────────────────────────────────────┘   │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                   │                                      │
│                                   ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                     适配层 (Language Adapters)                     │  │
│  │                                                                    │  │
│  │   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │  │
│  │   │ C/C++   │ │  Java   │ │ Kotlin  │ │  Rust   │ │   Go    │   │  │
│  │   │ Adapter │ │ Adapter │ │ Adapter │ │ Adapter │ │ Adapter │   │  │
│  │   └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘   │  │
│  │        │           │           │           │           │         │  │
│  └────────┼───────────┼───────────┼───────────┼───────────┼─────────┘  │
│           │           │           │           │           │            │
│           ▼           ▼           ▼           ▼           ▼            │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                      解析层 (Parsers)                              │  │
│  │                                                                    │  │
│  │   ┌──────────────────┐         ┌───────────────────────────────┐  │  │
│  │   │   Tree-sitter    │         │         libclang / LSP         │  │  │
│  │   │ (快速语法解析)    │         │    (精确语义分析，可选)        │  │  │
│  │   │ • c, cpp, java   │         │    • 类型推导                   │  │  │
│  │   │ • kotlin, rust   │         │    • 宏展开                     │  │  │
│  │   │ • go, python     │         │    • 完整符号表                 │  │  │
│  │   └──────────────────┘         └───────────────────────────────┘  │  │
│  │                                                                    │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                   │                                      │
│                                   ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                      源代码 (Source Code)                          │  │
│  │   .c  .h  .cpp  .java  .kt  .rs  .go  .py  .ts  .js               │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 核心模块职责

| 模块 | 职责 | 语言相关性 |
|------|------|-----------|
| **Parser** | 源码 → AST | 语言特定 |
| **Adapter** | AST → Unified IR | 语言特定 |
| **Unified IR** | 统一数据模型 | 语言无关 |
| **Analysis Engine** | IR 上的分析算法 | 语言无关 |
| **Knowledge Base** | 语义规则 | 部分语言特定 |
| **Application** | 用户功能 | 语言无关 |

---

## 3. 核心抽象模型

### 3.1 统一符号模型

```rust
/// 统一符号表示
/// 
/// 所有语言的"命名实体"都可以用这个模型表示
#[derive(Debug, Clone)]
pub struct Symbol {
    /// 唯一标识符 (全局唯一)
    pub id: SymbolId,
    
    /// 符号名称
    pub name: String,
    
    /// 完全限定名 (如 java.lang.String 或 linux::kernel::work_struct)
    pub qualified_name: String,
    
    /// 符号类型
    pub kind: SymbolKind,
    
    /// 所属作用域
    pub scope: ScopeId,
    
    /// 可见性
    pub visibility: Visibility,
    
    /// 源码位置
    pub location: Location,
    
    /// 语言特定属性 (扩展点)
    pub language_attrs: LanguageAttrs,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    // === 通用符号类型 ===
    Function(FunctionSymbol),
    Variable(VariableSymbol),
    Type(TypeSymbol),
    Constant(ConstantSymbol),
    
    // === 面向对象特有 ===
    Class(ClassSymbol),
    Interface(InterfaceSymbol),
    Method(MethodSymbol),
    Field(FieldSymbol),
    
    // === 模块化 ===
    Module(ModuleSymbol),
    Package(PackageSymbol),
    Namespace(NamespaceSymbol),
}

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Protected,
    Private,
    Internal,      // Kotlin internal
    PackageLocal,  // Java default
    FileLocal,     // Rust pub(crate)
}

/// 语言特定属性
/// 
/// 使用 enum 而非 trait object，便于序列化和模式匹配
#[derive(Debug, Clone)]
pub enum LanguageAttrs {
    C(CAttrs),
    Cpp(CppAttrs),
    Java(JavaAttrs),
    Kotlin(KotlinAttrs),
    Rust(RustAttrs),
    Go(GoAttrs),
    None,
}

// 语言特定属性示例
#[derive(Debug, Clone)]
pub struct CAttrs {
    pub is_static: bool,
    pub is_inline: bool,
    pub linkage: CLinkage,
}

#[derive(Debug, Clone)]
pub struct JavaAttrs {
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_synchronized: bool,
    pub annotations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct KotlinAttrs {
    pub is_suspend: bool,     // 协程函数
    pub is_inline: bool,
    pub is_data_class: bool,
    pub annotations: Vec<String>,
}
```

### 3.2 统一类型模型

```rust
/// 统一类型表示
#[derive(Debug, Clone)]
pub enum UnifiedType {
    // === 基础类型 ===
    Primitive(PrimitiveType),
    
    // === 复合类型 ===
    Struct {
        name: String,
        fields: Vec<TypedField>,
        language_variant: StructVariant,
    },
    
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
    },
    
    Union {
        name: String,
        fields: Vec<TypedField>,
    },
    
    // === 函数类型 ===
    Function {
        params: Vec<UnifiedType>,
        return_type: Box<UnifiedType>,
        is_variadic: bool,
    },
    
    // === 修饰类型 ===
    Pointer(Box<UnifiedType>),
    Reference(Box<UnifiedType>, RefKind),
    Array(Box<UnifiedType>, Option<usize>),
    
    // === 泛型 ===
    Generic {
        base: Box<UnifiedType>,
        type_args: Vec<UnifiedType>,
    },
    TypeParameter {
        name: String,
        bounds: Vec<UnifiedType>,
    },
    
    // === 特殊类型 ===
    Void,
    Unknown,
    Error,
}

#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Bool,
    Char,
    
    // 整数类型（统一为位宽表示）
    Int { bits: u8, signed: bool },
    
    // 浮点类型
    Float { bits: u8 },
    
    // 字符串（语言差异大，特殊处理）
    String,
}

#[derive(Debug, Clone)]
pub enum StructVariant {
    CStruct,
    CppClass { has_vtable: bool },
    JavaClass { is_interface: bool },
    KotlinDataClass,
    RustStruct,
    GoStruct,
}

#[derive(Debug, Clone)]
pub enum RefKind {
    LValueRef,  // C++ &
    RValueRef,  // C++ &&
    Shared,     // Rust &
    Mutable,    // Rust &mut
}
```

### 3.3 统一调用模型

```rust
/// 统一调用表示
/// 
/// 涵盖各种调用方式：直接调用、方法调用、间接调用等
#[derive(Debug, Clone)]
pub struct UnifiedCall {
    /// 调用位置
    pub location: Location,
    
    /// 调用类型
    pub kind: CallKind,
    
    /// 调用者
    pub caller: SymbolId,
    
    /// 被调用者（可能是表达式）
    pub callee: CalleeExpr,
    
    /// 参数
    pub arguments: Vec<Argument>,
    
    /// 是否可能抛出异常/返回错误
    pub may_throw: bool,
    
    /// 执行上下文
    pub context: ExecutionContext,
}

#[derive(Debug, Clone)]
pub enum CallKind {
    /// 直接函数调用: func(args)
    Direct,
    
    /// 方法调用: obj.method(args)
    Method {
        receiver: Box<Expr>,
        is_virtual: bool,
    },
    
    /// 静态方法调用: Class.method(args)
    StaticMethod,
    
    /// 函数指针调用: (*fp)(args)
    FunctionPointer,
    
    /// 回调调用: obj->ops->callback(args)
    Callback {
        ops_field: String,
        callback_field: String,
    },
    
    /// 构造函数调用: new Class(args)
    Constructor,
    
    /// 闭包/Lambda调用
    Closure,
    
    /// 异步调用
    Async(AsyncCallKind),
}

#[derive(Debug, Clone)]
pub enum AsyncCallKind {
    /// Go: go func()
    Goroutine,
    
    /// Kotlin: launch { }
    CoroutineLaunch,
    
    /// Java: executor.submit(() -> {})
    ExecutorSubmit,
    
    /// Rust: tokio::spawn(async {})
    TokioSpawn,
    
    /// JavaScript: Promise, async/await
    Promise,
    
    /// 内核: schedule_work(&work)
    KernelWorkQueue,
    
    /// 通用异步
    Generic { mechanism: String },
}

#[derive(Debug, Clone)]
pub enum CalleeExpr {
    /// 符号引用
    Symbol(SymbolId),
    
    /// 字段访问表达式
    FieldAccess {
        base: Box<CalleeExpr>,
        field: String,
    },
    
    /// 间接引用
    Deref(Box<CalleeExpr>),
    
    /// 动态表达式（需要运行时解析）
    Dynamic(String),
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 执行线程/协程
    pub thread_context: ThreadContext,
    
    /// 是否可以阻塞
    pub can_block: bool,
    
    /// 持有的锁
    pub held_locks: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ThreadContext {
    MainThread,
    WorkerThread,
    Coroutine,
    Interrupt,
    Unknown,
}
```

### 3.4 统一控制流模型

```rust
/// 统一控制流图 (CFG)
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// 入口节点
    pub entry: NodeId,
    
    /// 出口节点（可能多个）
    pub exits: Vec<NodeId>,
    
    /// 所有节点
    pub nodes: HashMap<NodeId, CfgNode>,
    
    /// 边
    pub edges: Vec<CfgEdge>,
}

#[derive(Debug, Clone)]
pub struct CfgNode {
    pub id: NodeId,
    pub kind: CfgNodeKind,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub enum CfgNodeKind {
    /// 基本块
    BasicBlock {
        statements: Vec<Statement>,
    },
    
    /// 条件分支
    Branch {
        condition: Expr,
    },
    
    /// 循环头
    LoopHeader {
        loop_kind: LoopKind,
    },
    
    /// 函数调用
    Call(UnifiedCall),
    
    /// 返回
    Return {
        value: Option<Expr>,
    },
    
    /// 异常处理
    ExceptionHandler {
        exception_type: UnifiedType,
    },
    
    /// 异步边界
    AsyncBoundary {
        kind: AsyncBoundaryKind,
    },
}

#[derive(Debug, Clone)]
pub enum AsyncBoundaryKind {
    /// 协程挂起点: await, suspend
    SuspendPoint,
    
    /// 异步任务边界
    TaskSpawn,
    
    /// 回调注册
    CallbackRegistration {
        callback: SymbolId,
    },
}

#[derive(Debug, Clone)]
pub struct CfgEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone)]
pub enum EdgeKind {
    /// 顺序执行
    Sequential,
    
    /// 条件为真
    ConditionTrue,
    
    /// 条件为假
    ConditionFalse,
    
    /// 循环回边
    LoopBack,
    
    /// 异常路径
    Exception,
    
    /// 异步触发
    AsyncTrigger {
        mechanism: String,
    },
}
```

---

## 4. 语言适配器

### 4.1 适配器 Trait 定义

```rust
/// 语言适配器接口
/// 
/// 每种语言需要实现此 trait 来接入 FlowSight
pub trait LanguageAdapter: Send + Sync {
    /// 语言标识符
    fn language_id(&self) -> &'static str;
    
    /// 支持的文件扩展名
    fn file_extensions(&self) -> &[&'static str];
    
    /// 解析单个文件
    fn parse_file(&self, path: &Path, content: &str) -> Result<ParsedFile>;
    
    /// 提取符号
    fn extract_symbols(&self, parsed: &ParsedFile) -> Result<Vec<Symbol>>;
    
    /// 提取调用
    fn extract_calls(&self, parsed: &ParsedFile) -> Result<Vec<UnifiedCall>>;
    
    /// 提取类型信息
    fn extract_types(&self, parsed: &ParsedFile) -> Result<Vec<UnifiedType>>;
    
    /// 构建控制流图
    fn build_cfg(&self, function: &FunctionSymbol) -> Result<ControlFlowGraph>;
    
    /// 解析函数指针目标
    fn resolve_indirect_call(
        &self,
        call: &UnifiedCall,
        context: &AnalysisContext,
    ) -> Result<Vec<SymbolId>>;
    
    /// 加载异步模式
    fn async_patterns(&self) -> &[AsyncPattern];
    
    /// 语言特定的知识库
    fn knowledge_base(&self) -> Option<&KnowledgeBase>;
    
    // === 可选能力 ===
    
    /// 是否支持精确类型分析
    fn supports_semantic_analysis(&self) -> bool {
        false
    }
    
    /// 执行精确语义分析（如果支持）
    fn semantic_analysis(&self, _project: &Project) -> Result<SemanticInfo> {
        Err(Error::NotSupported)
    }
}
```

### 4.2 C 语言适配器实现

```rust
pub struct CAdapter {
    ts_parser: tree_sitter::Parser,
    knowledge: KnowledgeBase,
    async_patterns: Vec<AsyncPattern>,
}

impl CAdapter {
    pub fn new() -> Result<Self> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_c::language())?;
        
        Ok(Self {
            ts_parser: parser,
            knowledge: KnowledgeBase::load("c")?,
            async_patterns: load_c_async_patterns()?,
        })
    }
}

impl LanguageAdapter for CAdapter {
    fn language_id(&self) -> &'static str {
        "c"
    }
    
    fn file_extensions(&self) -> &[&'static str] {
        &["c", "h"]
    }
    
    fn parse_file(&self, path: &Path, content: &str) -> Result<ParsedFile> {
        let tree = self.ts_parser.parse(content, None)
            .ok_or(Error::ParseFailed)?;
        
        Ok(ParsedFile {
            path: path.to_path_buf(),
            language: "c".to_string(),
            tree,
            content: content.to_string(),
        })
    }
    
    fn extract_symbols(&self, parsed: &ParsedFile) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();
        let root = parsed.tree.root_node();
        
        // 遍历 AST，提取函数定义
        for node in root.children(&mut root.walk()) {
            match node.kind() {
                "function_definition" => {
                    let func = self.extract_function(&node, &parsed.content)?;
                    symbols.push(func);
                }
                "struct_specifier" => {
                    let s = self.extract_struct(&node, &parsed.content)?;
                    symbols.push(s);
                }
                "declaration" => {
                    // 可能是函数声明、变量声明、typedef 等
                    let decls = self.extract_declarations(&node, &parsed.content)?;
                    symbols.extend(decls);
                }
                _ => {}
            }
        }
        
        Ok(symbols)
    }
    
    fn extract_calls(&self, parsed: &ParsedFile) -> Result<Vec<UnifiedCall>> {
        let mut calls = Vec::new();
        
        // 使用 tree-sitter 查询语法
        let query = tree_sitter::Query::new(
            tree_sitter_c::language(),
            "(call_expression
                function: (_) @callee
                arguments: (argument_list) @args
            ) @call"
        )?;
        
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches = cursor.matches(&query, parsed.tree.root_node(), parsed.content.as_bytes());
        
        for m in matches {
            let call_node = m.captures[0].node;
            let callee_node = m.captures[1].node;
            
            let call = self.node_to_unified_call(&call_node, &callee_node, parsed)?;
            calls.push(call);
        }
        
        Ok(calls)
    }
    
    fn resolve_indirect_call(
        &self,
        call: &UnifiedCall,
        context: &AnalysisContext,
    ) -> Result<Vec<SymbolId>> {
        // 首先尝试知识库模式匹配
        for pattern in &self.async_patterns {
            if pattern.matches_trigger(call) {
                if let Some(targets) = pattern.resolve_targets(call, context) {
                    return Ok(targets);
                }
            }
        }
        
        // 回退到通用指针分析
        match &call.callee {
            CalleeExpr::FieldAccess { base, field } => {
                // 分析 ops 表
                self.resolve_ops_callback(base, field, context)
            }
            CalleeExpr::Deref(inner) => {
                // 函数指针解引用
                self.resolve_function_pointer(inner, context)
            }
            _ => Ok(vec![]),
        }
    }
    
    fn async_patterns(&self) -> &[AsyncPattern] {
        &self.async_patterns
    }
    
    fn knowledge_base(&self) -> Option<&KnowledgeBase> {
        Some(&self.knowledge)
    }
}
```

### 4.3 Java 适配器实现

```rust
pub struct JavaAdapter {
    ts_parser: tree_sitter::Parser,
    knowledge: KnowledgeBase,
    async_patterns: Vec<AsyncPattern>,
}

impl LanguageAdapter for JavaAdapter {
    fn language_id(&self) -> &'static str {
        "java"
    }
    
    fn file_extensions(&self) -> &[&'static str] {
        &["java"]
    }
    
    fn extract_symbols(&self, parsed: &ParsedFile) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();
        let root = parsed.tree.root_node();
        
        // Java 特有：类声明
        for node in self.find_nodes(&root, "class_declaration") {
            let class = self.extract_class(&node, &parsed.content)?;
            symbols.push(class);
            
            // 提取类中的方法
            for method_node in self.find_nodes(&node, "method_declaration") {
                let method = self.extract_method(&method_node, &class.id, &parsed.content)?;
                symbols.push(method);
            }
        }
        
        // 接口声明
        for node in self.find_nodes(&root, "interface_declaration") {
            let interface = self.extract_interface(&node, &parsed.content)?;
            symbols.push(interface);
        }
        
        Ok(symbols)
    }
    
    fn extract_calls(&self, parsed: &ParsedFile) -> Result<Vec<UnifiedCall>> {
        let mut calls = Vec::new();
        
        // 方法调用
        for node in self.find_nodes(&parsed.tree.root_node(), "method_invocation") {
            let call = self.extract_method_call(&node, parsed)?;
            calls.push(call);
        }
        
        // new 表达式
        for node in self.find_nodes(&parsed.tree.root_node(), "object_creation_expression") {
            let call = self.extract_constructor_call(&node, parsed)?;
            calls.push(call);
        }
        
        Ok(calls)
    }
    
    fn resolve_indirect_call(
        &self,
        call: &UnifiedCall,
        context: &AnalysisContext,
    ) -> Result<Vec<SymbolId>> {
        match &call.kind {
            CallKind::Method { receiver, is_virtual: true } => {
                // 虚方法调用：查找所有实现类
                let method_name = call.callee.as_symbol_name()?;
                let receiver_type = context.type_of(receiver)?;
                
                self.find_virtual_targets(&receiver_type, &method_name, context)
            }
            
            CallKind::Async(AsyncCallKind::ExecutorSubmit) => {
                // executor.submit(() -> { ... })
                // 提取 lambda 体
                self.extract_lambda_target(call, context)
            }
            
            _ => Ok(vec![]),
        }
    }
    
    fn async_patterns(&self) -> &[AsyncPattern] {
        &self.async_patterns
    }
}
```

### 4.4 Kotlin 适配器（协程支持）

```rust
pub struct KotlinAdapter {
    ts_parser: tree_sitter::Parser,
    knowledge: KnowledgeBase,
}

impl LanguageAdapter for KotlinAdapter {
    fn language_id(&self) -> &'static str {
        "kotlin"
    }
    
    fn extract_calls(&self, parsed: &ParsedFile) -> Result<Vec<UnifiedCall>> {
        let mut calls = Vec::new();
        
        // 普通函数调用
        for node in self.find_nodes(&parsed.tree.root_node(), "call_expression") {
            let call = self.extract_call(&node, parsed)?;
            calls.push(call);
        }
        
        // 协程相关
        for node in self.find_nodes(&parsed.tree.root_node(), "call_expression") {
            let name = self.get_callee_name(&node, parsed)?;
            
            match name.as_str() {
                "launch" | "async" => {
                    // 协程启动
                    let async_call = self.extract_coroutine_launch(&node, parsed)?;
                    calls.push(async_call);
                }
                "withContext" => {
                    // 上下文切换
                    let ctx_call = self.extract_context_switch(&node, parsed)?;
                    calls.push(ctx_call);
                }
                _ => {}
            }
        }
        
        Ok(calls)
    }
    
    /// 处理 suspend 函数
    fn build_cfg(&self, function: &FunctionSymbol) -> Result<ControlFlowGraph> {
        let mut cfg = self.build_basic_cfg(function)?;
        
        // 检查是否是 suspend 函数
        if let LanguageAttrs::Kotlin(attrs) = &function.language_attrs {
            if attrs.is_suspend {
                // 在每个 suspend 调用点添加挂起边界
                self.add_suspend_boundaries(&mut cfg, function)?;
            }
        }
        
        Ok(cfg)
    }
    
    fn resolve_indirect_call(
        &self,
        call: &UnifiedCall,
        context: &AnalysisContext,
    ) -> Result<Vec<SymbolId>> {
        match &call.kind {
            CallKind::Async(AsyncCallKind::CoroutineLaunch) => {
                // launch { block } 中的 block
                self.extract_lambda_body(call, context)
            }
            
            CallKind::Closure => {
                // lambda 调用
                self.resolve_lambda(call, context)
            }
            
            _ => Ok(vec![]),
        }
    }
}
```

---

## 5. 统一 IR 设计

### 5.1 IR 概述

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      Unified IR 设计理念                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  源代码 (多语言)                                                         │
│       │                                                                  │
│       ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    Language-Specific AST                         │    │
│  │  • Tree-sitter 生成的具体语法树                                  │    │
│  │  • 保留语言特有结构                                               │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│       │                                                                  │
│       │  LanguageAdapter.extract_*()                                    │
│       ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                       Unified IR                                  │    │
│  │                                                                   │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │    │
│  │  │ Symbol Table│  │ Type Graph  │  │    Control Flow Graph   │  │    │
│  │  │ 统一符号表   │  │ 类型关系图  │  │    统一控制流图         │  │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │    │
│  │                                                                   │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │    │
│  │  │ Call Graph  │  │ Data Flow   │  │    Async Flow Graph     │  │    │
│  │  │ 调用关系图   │  │ 数据流图    │  │    异步流图             │  │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │    │
│  │                                                                   │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│       │                                                                  │
│       │  Analysis Engine                                                │
│       ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    Analysis Results                               │    │
│  │  • 执行流路径                                                     │    │
│  │  • 函数指针解析结果                                               │    │
│  │  • 异步边界标注                                                   │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.2 项目级 IR 结构

```rust
/// 整个项目的 IR 表示
pub struct ProjectIR {
    /// 项目元数据
    pub metadata: ProjectMetadata,
    
    /// 所有文件
    pub files: HashMap<PathBuf, FileIR>,
    
    /// 全局符号表
    pub symbol_table: SymbolTable,
    
    /// 类型图
    pub type_graph: TypeGraph,
    
    /// 调用图
    pub call_graph: CallGraph,
    
    /// 异步流图
    pub async_flow_graph: AsyncFlowGraph,
    
    /// 跨语言边界
    pub language_bridges: Vec<LanguageBridge>,
}

/// 单文件 IR
pub struct FileIR {
    pub path: PathBuf,
    pub language: String,
    
    /// 文件级符号
    pub symbols: Vec<SymbolId>,
    
    /// 导入/包含
    pub imports: Vec<Import>,
    
    /// 导出
    pub exports: Vec<Export>,
}

/// 调用图
pub struct CallGraph {
    /// 节点：函数/方法
    pub nodes: HashMap<SymbolId, CallGraphNode>,
    
    /// 边：调用关系
    pub edges: Vec<CallGraphEdge>,
}

pub struct CallGraphEdge {
    pub caller: SymbolId,
    pub callee: SymbolId,
    pub call_site: Location,
    pub call_kind: CallKind,
    pub is_direct: bool,  // true = 直接调用, false = 间接/虚调用
}

/// 异步流图
pub struct AsyncFlowGraph {
    /// 异步边界节点
    pub boundaries: Vec<AsyncBoundary>,
    
    /// 异步触发边
    pub trigger_edges: Vec<AsyncTriggerEdge>,
}

pub struct AsyncBoundary {
    pub id: AsyncBoundaryId,
    pub kind: AsyncBoundaryKind,
    pub location: Location,
    
    /// 绑定的处理器
    pub handler: Option<SymbolId>,
    
    /// 触发条件
    pub trigger_condition: String,
}

pub struct AsyncTriggerEdge {
    /// 触发点
    pub trigger_site: Location,
    
    /// 目标异步边界
    pub target_boundary: AsyncBoundaryId,
    
    /// 触发机制
    pub mechanism: String,
}
```

### 5.3 IR 构建流程

```rust
/// IR 构建器
pub struct IRBuilder {
    adapters: HashMap<String, Box<dyn LanguageAdapter>>,
    symbol_table: SymbolTable,
    type_graph: TypeGraph,
}

impl IRBuilder {
    pub fn build_project(&mut self, project: &Project) -> Result<ProjectIR> {
        let mut file_irs = HashMap::new();
        
        // Phase 1: 解析所有文件，收集符号
        for file in project.source_files() {
            let language = detect_language(&file);
            let adapter = self.get_adapter(&language)?;
            
            let content = fs::read_to_string(&file)?;
            let parsed = adapter.parse_file(&file, &content)?;
            
            // 提取符号
            let symbols = adapter.extract_symbols(&parsed)?;
            for symbol in symbols {
                self.symbol_table.insert(symbol);
            }
            
            file_irs.insert(file.clone(), FileIR::from_parsed(&parsed));
        }
        
        // Phase 2: 类型解析
        for (_, file_ir) in &file_irs {
            self.resolve_types(file_ir)?;
        }
        
        // Phase 3: 构建调用图
        let call_graph = self.build_call_graph(&file_irs)?;
        
        // Phase 4: 识别异步边界
        let async_flow_graph = self.build_async_flow_graph(&file_irs, &call_graph)?;
        
        // Phase 5: 检测跨语言边界
        let bridges = self.detect_language_bridges(&file_irs)?;
        
        Ok(ProjectIR {
            metadata: project.metadata.clone(),
            files: file_irs,
            symbol_table: self.symbol_table.clone(),
            type_graph: self.type_graph.clone(),
            call_graph,
            async_flow_graph,
            language_bridges: bridges,
        })
    }
    
    fn build_call_graph(&self, files: &HashMap<PathBuf, FileIR>) -> Result<CallGraph> {
        let mut graph = CallGraph::new();
        
        for (path, file_ir) in files {
            let adapter = self.get_adapter(&file_ir.language)?;
            let parsed = /* 获取解析后的文件 */;
            
            let calls = adapter.extract_calls(&parsed)?;
            
            for call in calls {
                // 解析调用目标
                let targets = if call.is_direct() {
                    vec![call.direct_target()]
                } else {
                    adapter.resolve_indirect_call(&call, &self.context())?
                };
                
                for target in targets {
                    graph.add_edge(CallGraphEdge {
                        caller: call.caller,
                        callee: target,
                        call_site: call.location.clone(),
                        call_kind: call.kind.clone(),
                        is_direct: call.is_direct(),
                    });
                }
            }
        }
        
        Ok(graph)
    }
}
```

---

## 6. 跨语言分析

### 6.1 语言边界检测

```rust
/// 跨语言边界
pub struct LanguageBridge {
    /// 边界类型
    pub kind: BridgeKind,
    
    /// 源语言符号
    pub source: BridgeEndpoint,
    
    /// 目标语言符号
    pub target: BridgeEndpoint,
    
    /// 参数/返回值映射
    pub type_mappings: Vec<TypeMapping>,
}

#[derive(Debug, Clone)]
pub enum BridgeKind {
    /// Java ↔ C/C++ (JNI)
    JNI {
        java_method: String,
        native_function: String,
        is_static: bool,
    },
    
    /// Go ↔ C (CGO)
    CGO {
        go_function: String,
        c_function: String,
        direction: CallDirection,
    },
    
    /// Python ↔ C (Python C API)
    PythonC {
        python_function: String,
        c_function: String,
    },
    
    /// Node.js ↔ C++ (N-API)
    NAPI {
        js_function: String,
        cpp_function: String,
    },
    
    /// Android AIDL/HIDL
    AndroidIPC {
        service_interface: String,
        implementation: String,
        transport: String,  // "binder", "hwbinder"
    },
}

pub struct BridgeEndpoint {
    pub language: String,
    pub symbol: SymbolId,
    pub location: Location,
}

/// 边界检测器
pub struct BridgeDetector {
    jni_patterns: Vec<JNIPattern>,
    cgo_patterns: Vec<CGOPattern>,
    // ...
}

impl BridgeDetector {
    pub fn detect(&self, ir: &ProjectIR) -> Vec<LanguageBridge> {
        let mut bridges = Vec::new();
        
        // 检测 JNI 边界
        bridges.extend(self.detect_jni_bridges(ir));
        
        // 检测 CGO 边界
        bridges.extend(self.detect_cgo_bridges(ir));
        
        // 检测 Android IPC
        bridges.extend(self.detect_android_ipc(ir));
        
        bridges
    }
    
    fn detect_jni_bridges(&self, ir: &ProjectIR) -> Vec<LanguageBridge> {
        let mut bridges = Vec::new();
        
        // 查找所有 native 方法声明
        for symbol in ir.symbol_table.iter() {
            if let SymbolKind::Method(method) = &symbol.kind {
                if let LanguageAttrs::Java(attrs) = &symbol.language_attrs {
                    if attrs.is_native {
                        // 查找对应的 C 函数
                        let jni_name = self.generate_jni_name(&symbol);
                        if let Some(c_symbol) = ir.symbol_table.find_by_name(&jni_name) {
                            bridges.push(LanguageBridge {
                                kind: BridgeKind::JNI {
                                    java_method: symbol.qualified_name.clone(),
                                    native_function: jni_name,
                                    is_static: method.is_static,
                                },
                                source: BridgeEndpoint {
                                    language: "java".to_string(),
                                    symbol: symbol.id,
                                    location: symbol.location.clone(),
                                },
                                target: BridgeEndpoint {
                                    language: "c".to_string(),
                                    symbol: c_symbol.id,
                                    location: c_symbol.location.clone(),
                                },
                                type_mappings: self.compute_jni_type_mappings(method),
                            });
                        }
                    }
                }
            }
        }
        
        bridges
    }
    
    fn generate_jni_name(&self, java_symbol: &Symbol) -> String {
        // Java_com_example_MyClass_methodName
        let package = java_symbol.qualified_name
            .replace('.', "_")
            .replace('$', "_00024");  // 内部类
        format!("Java_{}", package)
    }
}
```

### 6.2 跨语言调用图构建

```rust
/// 跨语言调用图
pub struct CrossLanguageCallGraph {
    /// 语言内调用图
    inner_graphs: HashMap<String, CallGraph>,
    
    /// 跨语言边
    bridge_edges: Vec<BridgeEdge>,
}

pub struct BridgeEdge {
    pub caller: SymbolId,
    pub caller_language: String,
    
    pub callee: SymbolId,
    pub callee_language: String,
    
    pub bridge: LanguageBridge,
    pub call_site: Location,
}

impl CrossLanguageCallGraph {
    /// 查找从 A 到 B 的调用路径（可能跨语言）
    pub fn find_call_path(
        &self,
        from: SymbolId,
        to: SymbolId,
    ) -> Option<CallPath> {
        // 使用 BFS/DFS 搜索，考虑跨语言边界
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent = HashMap::new();
        
        queue.push_back(from);
        
        while let Some(current) = queue.pop_front() {
            if current == to {
                // 回溯构建路径
                return Some(self.reconstruct_path(&parent, from, to));
            }
            
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            
            // 获取当前符号的语言
            let language = self.get_language(current);
            
            // 语言内调用
            if let Some(inner) = self.inner_graphs.get(&language) {
                for edge in inner.edges_from(current) {
                    if !visited.contains(&edge.callee) {
                        parent.insert(edge.callee, (current, PathStep::IntraLanguage(edge.clone())));
                        queue.push_back(edge.callee);
                    }
                }
            }
            
            // 跨语言调用
            for bridge_edge in &self.bridge_edges {
                if bridge_edge.caller == current && !visited.contains(&bridge_edge.callee) {
                    parent.insert(
                        bridge_edge.callee,
                        (current, PathStep::CrossLanguage(bridge_edge.clone()))
                    );
                    queue.push_back(bridge_edge.callee);
                }
            }
        }
        
        None
    }
}
```

---

## 7. 扩展新语言

### 7.1 添加新语言的步骤

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      添加新语言支持的流程                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Step 1: 确保 tree-sitter 语法可用                                       │
│  ────────────────────────────────────                                    │
│  • 检查 https://github.com/tree-sitter 是否有对应语法                   │
│  • 或自己编写 grammar.js                                                 │
│                                                                          │
│  Step 2: 创建 Adapter 骨架                                               │
│  ────────────────────────────────────                                    │
│  • 实现 LanguageAdapter trait                                           │
│  • 先实现基本的 parse_file 和 extract_symbols                           │
│                                                                          │
│  Step 3: 编写语言映射规则                                                │
│  ────────────────────────────────────                                    │
│  • AST 节点类型 → Symbol 类型                                            │
│  • 语言类型 → UnifiedType                                                │
│  • 调用语法 → UnifiedCall                                                │
│                                                                          │
│  Step 4: 添加异步模式                                                    │
│  ────────────────────────────────────                                    │
│  • 识别语言特有的异步机制                                                │
│  • 编写 YAML 模式定义                                                    │
│                                                                          │
│  Step 5: 测试和优化                                                      │
│  ────────────────────────────────────                                    │
│  • 使用真实项目测试                                                      │
│  • 性能优化                                                              │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.2 示例：添加 Swift 支持

```rust
// crates/flowsight-lang-swift/src/adapter.rs

pub struct SwiftAdapter {
    ts_parser: tree_sitter::Parser,
    knowledge: KnowledgeBase,
}

impl SwiftAdapter {
    pub fn new() -> Result<Self> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_swift::language())?;
        
        Ok(Self {
            ts_parser: parser,
            knowledge: KnowledgeBase::load("swift")?,
        })
    }
    
    /// Swift 特有：提取闭包
    fn extract_closure(&self, node: &Node, content: &str) -> Result<ClosureSymbol> {
        // Swift 闭包语法: { (params) -> ReturnType in body }
        let params = self.extract_closure_params(node, content)?;
        let return_type = self.extract_closure_return(node, content)?;
        let body = self.extract_closure_body(node, content)?;
        
        Ok(ClosureSymbol {
            params,
            return_type,
            body,
            is_escaping: self.is_escaping_closure(node)?,
        })
    }
}

impl LanguageAdapter for SwiftAdapter {
    fn language_id(&self) -> &'static str {
        "swift"
    }
    
    fn file_extensions(&self) -> &[&'static str] {
        &["swift"]
    }
    
    fn extract_symbols(&self, parsed: &ParsedFile) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();
        let root = parsed.tree.root_node();
        
        // 类声明
        for node in self.find_nodes(&root, "class_declaration") {
            let class = self.extract_class(&node, &parsed.content)?;
            symbols.push(class);
        }
        
        // 结构体
        for node in self.find_nodes(&root, "struct_declaration") {
            let s = self.extract_struct(&node, &parsed.content)?;
            symbols.push(s);
        }
        
        // 协议 (Protocol)
        for node in self.find_nodes(&root, "protocol_declaration") {
            let proto = self.extract_protocol(&node, &parsed.content)?;
            symbols.push(proto);
        }
        
        // 函数
        for node in self.find_nodes(&root, "function_declaration") {
            let func = self.extract_function(&node, &parsed.content)?;
            symbols.push(func);
        }
        
        Ok(symbols)
    }
    
    fn extract_calls(&self, parsed: &ParsedFile) -> Result<Vec<UnifiedCall>> {
        let mut calls = Vec::new();
        
        // 函数调用
        for node in self.find_nodes(&parsed.tree.root_node(), "call_expression") {
            let call = self.extract_call(&node, parsed)?;
            calls.push(call);
        }
        
        // GCD 异步调用
        for node in self.find_nodes(&parsed.tree.root_node(), "call_expression") {
            let name = self.get_callee_name(&node, parsed)?;
            if name.contains("DispatchQueue") && name.contains("async") {
                let async_call = UnifiedCall {
                    location: self.node_location(&node),
                    kind: CallKind::Async(AsyncCallKind::Generic {
                        mechanism: "GCD".to_string(),
                    }),
                    // ...
                };
                calls.push(async_call);
            }
        }
        
        Ok(calls)
    }
    
    fn async_patterns(&self) -> &[AsyncPattern] {
        // Swift 异步模式
        static PATTERNS: &[AsyncPattern] = &[
            // GCD
            AsyncPattern::new(
                "gcd_async",
                "Grand Central Dispatch",
                r#"DispatchQueue\.\w+\.async\s*\{"#,
                AsyncCallKind::Generic { mechanism: "GCD".to_string() },
            ),
            // Swift async/await
            AsyncPattern::new(
                "swift_async",
                "Swift Concurrency",
                r#"Task\s*\{"#,
                AsyncCallKind::Generic { mechanism: "Swift Task".to_string() },
            ),
        ];
        PATTERNS
    }
}
```

### 7.3 语言适配器注册

```rust
// crates/flowsight-core/src/registry.rs

/// 语言适配器注册表
pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn LanguageAdapter>>,
    extension_map: HashMap<String, String>,  // 扩展名 → 语言
}

impl AdapterRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            adapters: HashMap::new(),
            extension_map: HashMap::new(),
        };
        
        // 注册内置适配器
        registry.register(Box::new(CAdapter::new().unwrap()));
        registry.register(Box::new(CppAdapter::new().unwrap()));
        registry.register(Box::new(JavaAdapter::new().unwrap()));
        registry.register(Box::new(KotlinAdapter::new().unwrap()));
        registry.register(Box::new(RustAdapter::new().unwrap()));
        registry.register(Box::new(GoAdapter::new().unwrap()));
        
        registry
    }
    
    pub fn register(&mut self, adapter: Box<dyn LanguageAdapter>) {
        let lang_id = adapter.language_id().to_string();
        
        for ext in adapter.file_extensions() {
            self.extension_map.insert(ext.to_string(), lang_id.clone());
        }
        
        self.adapters.insert(lang_id, adapter);
    }
    
    pub fn get_adapter(&self, language: &str) -> Option<&dyn LanguageAdapter> {
        self.adapters.get(language).map(|a| a.as_ref())
    }
    
    pub fn detect_language(&self, path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.extension_map.get(ext))
            .cloned()
    }
    
    /// 动态加载语言插件
    pub fn load_plugin(&mut self, plugin_path: &Path) -> Result<()> {
        // 使用 libloading 动态加载
        unsafe {
            let lib = libloading::Library::new(plugin_path)?;
            let create_adapter: libloading::Symbol<fn() -> Box<dyn LanguageAdapter>> =
                lib.get(b"create_adapter")?;
            
            let adapter = create_adapter();
            self.register(adapter);
        }
        Ok(())
    }
}
```

---

## 8. 案例分析

### 8.1 Android App 分析示例

```
项目结构：
android-app/
├── app/src/main/java/com/example/     # Java/Kotlin 代码
├── app/src/main/cpp/                   # Native 代码
└── app/src/main/aidl/                  # AIDL 接口
```

```rust
/// 分析 Android 项目
fn analyze_android_project(project: &Project) -> Result<ProjectIR> {
    let mut builder = IRBuilder::new();
    
    // 注册需要的适配器
    builder.register_adapter(JavaAdapter::new()?);
    builder.register_adapter(KotlinAdapter::new()?);
    builder.register_adapter(CppAdapter::new()?);
    
    // 构建 IR
    let mut ir = builder.build_project(project)?;
    
    // 特殊处理：解析 AIDL
    let aidl_interfaces = parse_aidl_files(&project.aidl_files())?;
    
    // 建立 Binder 调用关系
    for interface in aidl_interfaces {
        // 查找 Stub 实现
        let stub_impl = ir.symbol_table.find_class_extending(&format!(
            "{}.Stub", interface.qualified_name
        ));
        
        if let Some(impl_class) = stub_impl {
            ir.add_bridge(LanguageBridge {
                kind: BridgeKind::AndroidIPC {
                    service_interface: interface.qualified_name.clone(),
                    implementation: impl_class.qualified_name.clone(),
                    transport: "binder".to_string(),
                },
                source: /* client stub */,
                target: /* server implementation */,
                type_mappings: vec![],
            });
        }
    }
    
    // 检测 JNI 边界
    let jni_bridges = BridgeDetector::new().detect_jni_bridges(&ir);
    ir.language_bridges.extend(jni_bridges);
    
    Ok(ir)
}
```

### 8.2 执行流追踪示例

```
场景：用户点击按钮 → JNI 调用 → Native 处理 → 回调到 Java

执行流：
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                          │
│  [Java] MainActivity.onClick()                                          │
│       │                                                                  │
│       └──► [Java] MyService.processData(data)                           │
│                 │                                                        │
│                 └──► [JNI] Java_com_example_MyService_nativeProcess()   │
│                           │                                              │
│                           └──► [C++] NativeProcessor::process()         │
│                                     │                                    │
│                                     └──► [C++] processInBackground()    │
│                                               │                          │
│                                               │ (ThreadPool)             │
│                                               ▼                          │
│                                           [C++] workerThread()          │
│                                               │                          │
│                                               └──► [JNI Callback]       │
│                                                     │                    │
│                                                     ▼                    │
│                                               [Java] Callback.onComplete()
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

```rust
/// 生成执行流报告
fn generate_execution_flow(
    ir: &ProjectIR,
    entry_point: SymbolId,
) -> ExecutionFlow {
    let cross_lang_graph = CrossLanguageCallGraph::from_ir(ir);
    
    let mut flow = ExecutionFlow::new(entry_point);
    let mut visited = HashSet::new();
    
    fn traverse(
        graph: &CrossLanguageCallGraph,
        current: SymbolId,
        flow: &mut ExecutionFlow,
        visited: &mut HashSet<SymbolId>,
    ) {
        if visited.contains(&current) {
            return;
        }
        visited.insert(current);
        
        let symbol = graph.get_symbol(current);
        
        // 添加流节点
        let node = FlowNode {
            symbol: current,
            language: symbol.language.clone(),
            kind: classify_flow_node(&symbol),
        };
        flow.add_node(node);
        
        // 遍历所有调用
        for edge in graph.edges_from(current) {
            // 标记边类型
            let edge_kind = if edge.is_bridge() {
                FlowEdgeKind::CrossLanguage {
                    from_lang: edge.caller_language.clone(),
                    to_lang: edge.callee_language.clone(),
                    bridge_type: edge.bridge.kind.clone(),
                }
            } else if edge.is_async() {
                FlowEdgeKind::Async {
                    mechanism: edge.async_mechanism.clone(),
                }
            } else {
                FlowEdgeKind::Direct
            };
            
            flow.add_edge(current, edge.callee, edge_kind);
            
            // 递归
            traverse(graph, edge.callee, flow, visited);
        }
    }
    
    traverse(&cross_lang_graph, entry_point, &mut flow, &mut visited);
    flow
}
```

---

## 附录：语言特性对照表

| 特性 | C | C++ | Java | Kotlin | Rust | Go |
|------|---|-----|------|--------|------|-----|
| 函数指针 | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ |
| 虚函数 | ❌ | ✅ | ✅ | ✅ | ✅(trait) | ✅(interface) |
| 闭包/Lambda | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 泛型 | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 协程 | ❌ | ✅(C++20) | ❌ | ✅ | ✅ | ✅(goroutine) |
| 异常 | ❌ | ✅ | ✅ | ✅ | ❌(Result) | ❌(error) |
| 反射 | ❌ | ⚠️(RTTI) | ✅ | ✅ | ❌ | ✅ |
| 宏系统 | ✅(预处理器) | ✅ | ❌ | ❌ | ✅(过程宏) | ❌ |

---

*文档版本: 1.0*
*最后更新: 2025-01-04*
*作者: FlowSight Team*

