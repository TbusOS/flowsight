# 🎯 FlowSight 指针分析算法设计

> 本文档详细描述 FlowSight 中函数指针分析的算法原理和实现思路，这是实现执行流追踪的核心技术。

---

## 目录

1. [问题定义](#1-问题定义)
2. [学术背景](#2-学术背景)
3. [FlowSight 的分析策略](#3-flowsight-的分析策略)
4. [核心算法](#4-核心算法)
5. [实现细节](#5-实现细节)
6. [优化策略](#6-优化策略)
7. [局限性与应对](#7-局限性与应对)

---

## 1. 问题定义

### 1.1 什么是函数指针分析？

```c
// 问题：当我们看到这样的代码时
void (*callback)(int);
callback = some_function;
callback(42);  // ← 这里实际调用的是谁？

// 更复杂的情况：
struct ops {
    int (*read)(void *);
    int (*write)(void *, int);
};

struct ops my_ops = {
    .read = my_read_impl,
    .write = my_write_impl,
};

// 在另一个文件
extern struct ops my_ops;
my_ops.read(data);  // ← 需要跨文件追踪
```

### 1.2 挑战等级

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     函数指针分析的难度等级                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Level 1: 局部直接赋值 ✅ 简单                                          │
│  ─────────────────────────────────────                                   │
│  void (*fp)(void) = my_func;                                            │
│  fp();  // → my_func                                                     │
│                                                                          │
│  Level 2: 结构体字段赋值 ✅ 中等                                        │
│  ─────────────────────────────────────                                   │
│  struct ops o = { .callback = my_func };                                │
│  o.callback();  // → my_func                                            │
│                                                                          │
│  Level 3: 跨函数传递 ⚠️ 需要数据流分析                                  │
│  ─────────────────────────────────────                                   │
│  void set_callback(struct dev *d, void (*cb)(void)) {                   │
│      d->callback = cb;                                                   │
│  }                                                                       │
│  set_callback(dev, my_func);                                            │
│  dev->callback();  // → 需要追踪参数传递                                │
│                                                                          │
│  Level 4: 条件分支 ⚠️ 需要路径敏感分析                                  │
│  ─────────────────────────────────────                                   │
│  if (cond) fp = func_a; else fp = func_b;                               │
│  fp();  // → {func_a, func_b}                                           │
│                                                                          │
│  Level 5: 动态数组/表查找 ❌ 静态分析极难                               │
│  ─────────────────────────────────────                                   │
│  void (*table[])(void) = {f1, f2, f3, ...};                             │
│  table[runtime_index]();  // → 需要知识库辅助                           │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.3 我们的目标

| 目标 | 优先级 | 说明 |
|------|--------|------|
| 覆盖 Level 1-2 | P0 | 必须 100% 准确 |
| 覆盖 Level 3 | P0 | 内核中大量使用此模式 |
| 覆盖 Level 4 | P1 | 返回所有可能的目标 |
| 辅助 Level 5 | P2 | 结合知识库处理已知模式 |

---

## 2. 学术背景

### 2.1 经典指针分析算法对比

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      指针分析算法家族                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  算法               精确度        复杂度      适用场景                   │
│  ─────────────────────────────────────────────────────────────────────   │
│  Steensgaard       ★☆☆☆☆        O(n)       超大代码，快速粗略           │
│  Andersen          ★★★☆☆        O(n³)      中等代码，较精确             │
│  Flow-Sensitive    ★★★★☆        O(n⁴)      小代码，高精确               │
│  Context-Sensitive ★★★★★        指数级      极小代码，完美精确          │
│                                                                          │
│  FlowSight 策略：                                                       │
│  ─────────────────                                                       │
│  • 使用改进的 Andersen 作为基础                                         │
│  • 结合知识库降低需要分析的复杂度                                       │
│  • 对已知模式（如 INIT_WORK）直接匹配，跳过复杂分析                     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 关键概念

#### Points-To 集合

```
对于每个指针变量 p，计算 pts(p) = {可能指向的目标}

示例：
    int x, y;
    int *p = &x;
    int *q = &y;
    if (cond) p = q;
    
结果：
    pts(p) = {x, y}  // p 可能指向 x 或 y
    pts(q) = {y}     // q 只指向 y
```

#### 约束求解

```
Andersen 算法将赋值语句转换为约束：

语句                    约束
─────────────────────────────────────
p = &x              → x ∈ pts(p)
p = q               → pts(q) ⊆ pts(p)
p = *q              → ∀o ∈ pts(q): pts(o) ⊆ pts(p)
*p = q              → ∀o ∈ pts(p): pts(q) ⊆ pts(o)

然后迭代求解直到不动点。
```

---

## 3. FlowSight 的分析策略

### 3.1 混合分析架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    FlowSight 指针分析架构                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│                         输入代码                                         │
│                            │                                             │
│                            ▼                                             │
│            ┌───────────────────────────────┐                            │
│            │      Pattern Matcher          │ ← 知识库模式               │
│            │    (快速路径，O(n))           │                            │
│            └───────────────────────────────┘                            │
│                     │              │                                     │
│           匹配成功 ↙                ↘ 匹配失败                          │
│                  ↓                    ↓                                  │
│     ┌─────────────────┐    ┌──────────────────────┐                     │
│     │ 直接返回结果     │    │   Local Flow Analysis │                    │
│     │ (已知模式)       │    │   (函数内数据流)      │                    │
│     └─────────────────┘    └──────────────────────┘                     │
│                                       │                                  │
│                            找到赋值? ↙ ↘ 需要跨函数                      │
│                                  ↓       ↓                               │
│                     ┌─────────────┐ ┌───────────────────┐               │
│                     │ 返回结果     │ │ Inter-Procedural  │               │
│                     └─────────────┘ │  Analysis          │               │
│                                     │ (跨函数分析)       │               │
│                                     └───────────────────┘               │
│                                              │                           │
│                                              ▼                           │
│                                    ┌─────────────────┐                  │
│                                    │   Points-To     │                  │
│                                    │   结果集合       │                  │
│                                    └─────────────────┘                  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 分析层次

| 层次 | 名称 | 精确度 | 成本 | 使用场景 |
|------|------|--------|------|----------|
| L0 | 模式匹配 | 100% | O(1) | 已知异步模式 |
| L1 | 局部流分析 | 95% | O(n) | 函数内赋值 |
| L2 | 过程间分析 | 85% | O(n²) | 参数传递 |
| L3 | 全局分析 | 70% | O(n³) | 全局变量 |

### 3.3 按需分析 (Demand-Driven)

```
传统方式：分析整个程序，构建完整 points-to 图
FlowSight：用户询问时才分析，只计算需要的部分

示例：
  用户问：schedule_work(&dev->work) 调用谁？
  
  FlowSight 执行：
  1. 识别 schedule_work 模式 → 知道要找 work 的 handler
  2. 反向追踪 dev->work 的初始化
  3. 找到 INIT_WORK(&dev->work, my_handler)
  4. 返回 my_handler
  
  不需要分析整个程序！
```

---

## 4. 核心算法

### 4.1 算法 1：模式驱动分析

```rust
/// 模式驱动的函数指针解析
/// 
/// 输入：
///   - call_site: 间接调用位置
///   - knowledge_base: 知识库
///   - code_index: 代码索引
/// 
/// 输出：
///   - Vec<FunctionTarget>: 可能的目标函数列表
/// 
fn resolve_by_pattern(
    call_site: &CallSite,
    kb: &KnowledgeBase,
    index: &CodeIndex,
) -> Vec<FunctionTarget> {
    let mut results = Vec::new();
    
    // 步骤 1: 尝试匹配已知触发模式
    for pattern in kb.trigger_patterns() {
        if let Some(captures) = pattern.match_call(call_site) {
            // 提取变量名
            let var_name = captures.get("var");
            
            // 步骤 2: 查找对应的绑定
            let bindings = find_bindings(var_name, pattern.bind_patterns(), index);
            
            for binding in bindings {
                if let Some(handler) = binding.handler {
                    results.push(FunctionTarget {
                        name: handler,
                        confidence: Confidence::High,
                        source: TargetSource::PatternMatch,
                    });
                }
            }
        }
    }
    
    // 如果模式匹配成功，直接返回
    if !results.is_empty() {
        return results;
    }
    
    // 步骤 3: 回退到通用指针分析
    resolve_by_dataflow(call_site, index)
}
```

### 4.2 算法 2：局部数据流分析

```rust
/// 函数内数据流分析
/// 
/// 使用反向数据流追踪指针赋值
/// 
fn resolve_local_dataflow(
    pointer_var: &Variable,
    current_function: &Function,
    code_index: &CodeIndex,
) -> Vec<FunctionTarget> {
    let mut results = Vec::new();
    let mut worklist: VecDeque<Variable> = VecDeque::new();
    let mut visited: HashSet<Variable> = HashSet::new();
    
    worklist.push_back(pointer_var.clone());
    
    while let Some(var) = worklist.pop_front() {
        if visited.contains(&var) {
            continue;
        }
        visited.insert(var.clone());
        
        // 查找所有对此变量的赋值
        let assignments = find_assignments_to(&var, current_function);
        
        for assign in assignments {
            match &assign.rhs {
                // 直接赋值函数地址
                RValue::FunctionAddress(func_name) => {
                    results.push(FunctionTarget {
                        name: func_name.clone(),
                        confidence: Confidence::High,
                        source: TargetSource::DirectAssignment,
                    });
                }
                
                // 从另一个变量赋值
                RValue::Variable(other_var) => {
                    worklist.push_back(other_var.clone());
                }
                
                // 从结构体字段读取
                RValue::FieldAccess(base, field) => {
                    // 查找结构体初始化
                    let struct_inits = find_struct_initializations(
                        base, 
                        field,
                        code_index
                    );
                    for init in struct_inits {
                        if let Some(handler) = init.value.as_function() {
                            results.push(FunctionTarget {
                                name: handler,
                                confidence: Confidence::Medium,
                                source: TargetSource::StructField,
                            });
                        }
                    }
                }
                
                // 函数参数
                RValue::Parameter(param_idx) => {
                    // 需要过程间分析
                    let callers = find_callers(current_function, code_index);
                    for caller in callers {
                        let arg = get_argument_at_call(caller, param_idx);
                        // 递归分析
                        let sub_results = resolve_local_dataflow(
                            &arg, 
                            caller.function,
                            code_index
                        );
                        results.extend(sub_results);
                    }
                }
                
                _ => {
                    // 复杂表达式，标记为不确定
                    results.push(FunctionTarget {
                        name: "<unknown>".to_string(),
                        confidence: Confidence::Low,
                        source: TargetSource::Unknown,
                    });
                }
            }
        }
    }
    
    results
}
```

### 4.3 算法 3：Andersen 风格的约束求解

```rust
/// Points-To 分析的约束表示
#[derive(Clone, Debug)]
enum Constraint {
    /// x ∈ pts(p): p 指向 x
    AddressOf { pointer: Var, target: Var },
    
    /// pts(q) ⊆ pts(p): p 赋值自 q
    Copy { to: Var, from: Var },
    
    /// ∀o ∈ pts(q): pts(o) ⊆ pts(p): p = *q
    Load { to: Var, from: Var },
    
    /// ∀o ∈ pts(p): pts(q) ⊆ pts(o): *p = q
    Store { to: Var, from: Var },
}

/// Andersen 约束求解器
struct AndersenSolver {
    /// 每个变量的 points-to 集合
    points_to: HashMap<Var, HashSet<Var>>,
    /// 待处理的约束
    constraints: Vec<Constraint>,
    /// 工作列表
    worklist: VecDeque<Var>,
}

impl AndersenSolver {
    fn solve(&mut self) {
        // 初始化：处理所有 AddressOf 约束
        for constraint in &self.constraints {
            if let Constraint::AddressOf { pointer, target } = constraint {
                self.points_to
                    .entry(pointer.clone())
                    .or_default()
                    .insert(target.clone());
                self.worklist.push_back(pointer.clone());
            }
        }
        
        // 迭代直到不动点
        while let Some(var) = self.worklist.pop_front() {
            let pts = self.points_to.get(&var).cloned().unwrap_or_default();
            
            for constraint in &self.constraints.clone() {
                match constraint {
                    // p = q: 如果 q 的 pts 变化，传播到 p
                    Constraint::Copy { to, from } if from == &var => {
                        if self.add_all(to, &pts) {
                            self.worklist.push_back(to.clone());
                        }
                    }
                    
                    // p = *q: 对于 q 指向的每个 o，将 pts(o) 加入 pts(p)
                    Constraint::Load { to, from } if from == &var => {
                        for target in &pts {
                            if let Some(target_pts) = self.points_to.get(target).cloned() {
                                if self.add_all(to, &target_pts) {
                                    self.worklist.push_back(to.clone());
                                }
                            }
                        }
                    }
                    
                    // *p = q: 对于 p 指向的每个 o，将 pts(q) 加入 pts(o)
                    Constraint::Store { to, from } if to == &var => {
                        let from_pts = self.points_to.get(from).cloned().unwrap_or_default();
                        for target in &pts {
                            if self.add_all(target, &from_pts) {
                                self.worklist.push_back(target.clone());
                            }
                        }
                    }
                    
                    _ => {}
                }
            }
        }
    }
    
    fn add_all(&mut self, var: &Var, to_add: &HashSet<Var>) -> bool {
        let pts = self.points_to.entry(var.clone()).or_default();
        let old_size = pts.len();
        pts.extend(to_add.iter().cloned());
        pts.len() > old_size  // 返回是否有变化
    }
    
    /// 查询某个变量可能指向的目标
    fn query(&self, var: &Var) -> HashSet<Var> {
        self.points_to.get(var).cloned().unwrap_or_default()
    }
}
```

### 4.4 算法 4：结构体 ops 表分析

这是 Linux 内核中最常见的模式，需要专门优化：

```rust
/// ops 表分析器
/// 
/// 专门处理如下模式：
/// static struct file_operations fops = {
///     .read = my_read,
///     .write = my_write,
/// };
/// 
struct OpsTableAnalyzer {
    /// 类型名 -> ops 实例列表
    ops_instances: HashMap<String, Vec<OpsInstance>>,
}

#[derive(Debug)]
struct OpsInstance {
    name: String,
    type_name: String,
    location: Location,
    fields: HashMap<String, String>,  // 字段名 -> 函数名
}

impl OpsTableAnalyzer {
    /// 从代码中提取所有 ops 表实例
    fn extract_ops_tables(&mut self, ast: &AST, known_ops_types: &[String]) {
        // 遍历所有全局变量初始化
        for var_decl in ast.global_variables() {
            // 检查类型是否是已知的 ops 类型
            if known_ops_types.contains(&var_decl.type_name) {
                if let Some(init) = &var_decl.initializer {
                    let mut instance = OpsInstance {
                        name: var_decl.name.clone(),
                        type_name: var_decl.type_name.clone(),
                        location: var_decl.location.clone(),
                        fields: HashMap::new(),
                    };
                    
                    // 解析指定初始化器
                    self.parse_designated_init(init, &mut instance.fields);
                    
                    self.ops_instances
                        .entry(var_decl.type_name.clone())
                        .or_default()
                        .push(instance);
                }
            }
        }
    }
    
    /// 解析 { .field = value, ... } 形式的初始化
    fn parse_designated_init(
        &self, 
        init: &Initializer, 
        fields: &mut HashMap<String, String>
    ) {
        for (field_name, value) in init.designated_fields() {
            if let InitValue::Identifier(func_name) = value {
                fields.insert(field_name.clone(), func_name.clone());
            }
        }
    }
    
    /// 解析间接调用：obj->ops->read(...)
    fn resolve_ops_call(
        &self,
        base_expr: &Expr,       // obj->ops
        field_name: &str,       // read
        type_info: &TypeInfo,
    ) -> Vec<FunctionTarget> {
        let mut results = Vec::new();
        
        // 确定 ops 的类型
        let ops_type = type_info.get_field_type(base_expr, "ops")
            .and_then(|t| t.pointee_type());
        
        if let Some(type_name) = ops_type {
            // 查找所有该类型的实例
            if let Some(instances) = self.ops_instances.get(&type_name) {
                for instance in instances {
                    if let Some(handler) = instance.fields.get(field_name) {
                        results.push(FunctionTarget {
                            name: handler.clone(),
                            confidence: Confidence::Medium,
                            source: TargetSource::OpsTable,
                            ops_instance: Some(instance.name.clone()),
                        });
                    }
                }
            }
        }
        
        results
    }
}
```

---

## 5. 实现细节

### 5.1 变量表示

```rust
/// 统一的变量表示
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
enum Var {
    /// 局部变量: function_name::var_name
    Local { function: String, name: String },
    
    /// 全局变量
    Global { name: String },
    
    /// 结构体字段: base.field 或 base->field
    Field { base: Box<Var>, field: String },
    
    /// 数组元素（使用抽象索引）
    ArrayElement { base: Box<Var>, index: ArrayIndex },
    
    /// 函数参数: function_name@param_idx
    Parameter { function: String, index: usize },
    
    /// 函数返回值: function_name@return
    Return { function: String },
    
    /// 堆分配对象: alloc_site_location
    HeapObject { alloc_site: Location },
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
enum ArrayIndex {
    Constant(i64),
    Unknown,  // 抽象为单个位置
}
```

### 5.2 约束提取

```rust
/// 从 AST 节点提取约束
fn extract_constraints(stmt: &Statement, context: &FunctionContext) -> Vec<Constraint> {
    let mut constraints = Vec::new();
    
    match stmt {
        // int *p = &x;
        Statement::VarDecl { name, init: Some(Expr::AddressOf(target)), .. } => {
            constraints.push(Constraint::AddressOf {
                pointer: context.local_var(name),
                target: resolve_var(target, context),
            });
        }
        
        // p = q;
        Statement::Assignment { lhs, rhs: Expr::Var(rhs_name) } => {
            constraints.push(Constraint::Copy {
                to: resolve_var(lhs, context),
                from: context.local_var(rhs_name),
            });
        }
        
        // p = *q;
        Statement::Assignment { lhs, rhs: Expr::Deref(inner) } => {
            constraints.push(Constraint::Load {
                to: resolve_var(lhs, context),
                from: resolve_var(inner, context),
            });
        }
        
        // *p = q;
        Statement::Assignment { 
            lhs: LValue::Deref(ptr), 
            rhs: Expr::Var(rhs_name) 
        } => {
            constraints.push(Constraint::Store {
                to: resolve_var(ptr, context),
                from: context.local_var(rhs_name),
            });
        }
        
        // p->field = func;  (结构体字段赋值)
        Statement::Assignment {
            lhs: LValue::FieldAccess { base, field },
            rhs: Expr::Var(func_name),
        } if is_function_pointer_field(base, field) => {
            constraints.push(Constraint::Store {
                to: Var::Field {
                    base: Box::new(resolve_var(base, context)),
                    field: field.clone(),
                },
                from: Var::Global { name: func_name.clone() },
            });
        }
        
        // 函数调用：传参会产生约束
        Statement::Call { callee, args } => {
            for (i, arg) in args.iter().enumerate() {
                constraints.push(Constraint::Copy {
                    to: Var::Parameter { 
                        function: callee.clone(), 
                        index: i 
                    },
                    from: resolve_var(arg, context),
                });
            }
        }
        
        _ => {}
    }
    
    constraints
}
```

### 5.3 结构体初始化分析

```rust
/// 专门处理结构体指定初始化器
/// 
/// 示例：
/// static struct usb_driver my_driver = {
///     .name = "my_driver",
///     .probe = my_probe,       // ← 提取这个
///     .disconnect = my_disconnect,
/// };
/// 
fn analyze_struct_init(
    init: &StructInitializer,
    type_info: &StructType,
) -> HashMap<String, FunctionBinding> {
    let mut bindings = HashMap::new();
    
    for field_init in &init.fields {
        // 检查字段类型是否是函数指针
        if let Some(field_type) = type_info.get_field(&field_init.name) {
            if field_type.is_function_pointer() {
                match &field_init.value {
                    // .probe = my_probe
                    InitValue::Identifier(name) => {
                        bindings.insert(
                            field_init.name.clone(),
                            FunctionBinding {
                                target: name.clone(),
                                confidence: Confidence::High,
                            }
                        );
                    }
                    
                    // .probe = NULL
                    InitValue::Null => {
                        // 记录为空，不产生绑定
                    }
                    
                    // .probe = (condition ? func_a : func_b)
                    InitValue::Conditional { then_val, else_val, .. } => {
                        // 产生多个可能的绑定
                        if let InitValue::Identifier(name) = then_val.as_ref() {
                            bindings.insert(
                                field_init.name.clone(),
                                FunctionBinding {
                                    target: name.clone(),
                                    confidence: Confidence::Medium,
                                }
                            );
                        }
                        if let InitValue::Identifier(name) = else_val.as_ref() {
                            bindings.insert(
                                format!("{}#alt", field_init.name),
                                FunctionBinding {
                                    target: name.clone(),
                                    confidence: Confidence::Medium,
                                }
                            );
                        }
                    }
                    
                    _ => {}
                }
            }
        }
    }
    
    bindings
}
```

---

## 6. 优化策略

### 6.1 增量分析

```rust
/// 增量分析：只重新分析变化的部分
struct IncrementalAnalyzer {
    /// 上次分析的结果缓存
    cache: AnalysisCache,
    /// 文件依赖图
    dependencies: DependencyGraph,
}

impl IncrementalAnalyzer {
    fn analyze_incremental(&mut self, changed_files: &[PathBuf]) {
        // 1. 找出所有受影响的文件
        let affected = self.dependencies.get_affected_files(changed_files);
        
        // 2. 使缓存中受影响的条目失效
        for file in &affected {
            self.cache.invalidate_file(file);
        }
        
        // 3. 只重新分析受影响的部分
        for file in &affected {
            let constraints = self.extract_constraints_from_file(file);
            self.cache.update_constraints(file, constraints);
        }
        
        // 4. 增量更新 points-to 结果
        self.solver.incremental_solve(&self.cache.changed_constraints);
    }
}
```

### 6.2 分层缓存

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          分层缓存策略                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   L1: 热点缓存 (内存)                                                   │
│   ─────────────────────                                                  │
│   • 最近查询的 100 个函数指针解析结果                                   │
│   • LRU 淘汰策略                                                         │
│   • 命中率预期: 80%+                                                     │
│                                                                          │
│   L2: 文件级缓存 (SQLite)                                               │
│   ─────────────────────                                                  │
│   • 每个文件的约束集                                                     │
│   • 结构体初始化信息                                                     │
│   • 基于文件 hash 失效                                                   │
│                                                                          │
│   L3: 项目级缓存 (磁盘)                                                 │
│   ─────────────────────                                                  │
│   • 完整的 points-to 图                                                  │
│   • 索引版本控制                                                         │
│   • 后台定期重建                                                         │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.3 并行分析

```rust
/// 并行约束提取
fn parallel_extract_constraints(files: &[PathBuf]) -> Vec<Constraint> {
    files.par_iter()  // Rayon 并行迭代
        .flat_map(|file| {
            let ast = parse_file(file);
            extract_file_constraints(&ast)
        })
        .collect()
}

/// 并行查询解析
fn parallel_resolve_calls(
    call_sites: &[CallSite],
    solver: &AndersenSolver,
) -> HashMap<CallSite, Vec<FunctionTarget>> {
    call_sites.par_iter()
        .map(|site| {
            let targets = resolve_indirect_call(site, solver);
            (site.clone(), targets)
        })
        .collect()
}
```

---

## 7. 局限性与应对

### 7.1 已知局限性

| 局限 | 原因 | 应对策略 |
|------|------|----------|
| 动态分发 | 运行时决定目标 | 返回所有可能目标 + 置信度 |
| 间接索引 | `table[i]()` 中的 i 是运行时值 | 分析整个表 + 知识库标注 |
| 外部函数 | 没有源码 | 使用知识库定义行为 |
| 汇编代码 | 无法解析 | 跳过 + 警告 |
| 复杂宏 | 宏展开后可能变化 | 依赖 libclang 完成展开 |

### 7.2 置信度系统

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Confidence {
    /// 确定性结果（模式匹配、直接赋值）
    High = 100,
    
    /// 高可信度（结构体初始化）
    Medium = 75,
    
    /// 中等可信度（跨函数分析）
    Low = 50,
    
    /// 不确定（多路径合并）
    Uncertain = 25,
    
    /// 未知（无法分析）
    Unknown = 0,
}

#[derive(Debug)]
struct FunctionTarget {
    name: String,
    confidence: Confidence,
    source: TargetSource,
    /// 分析路径说明（用于 UI 展示）
    reasoning: Vec<String>,
}
```

### 7.3 回退策略

```rust
/// 当精确分析失败时的回退策略
fn resolve_with_fallback(
    call_site: &CallSite,
    context: &AnalysisContext,
) -> ResolutionResult {
    // 尝试 1: 模式匹配
    if let Some(result) = try_pattern_match(call_site, context) {
        return result;
    }
    
    // 尝试 2: 局部数据流
    if let Some(result) = try_local_dataflow(call_site, context) {
        return result;
    }
    
    // 尝试 3: 类型匹配（所有签名匹配的函数）
    let type_sig = get_function_pointer_type(call_site);
    let candidates = find_functions_with_signature(&type_sig, context);
    
    if !candidates.is_empty() {
        return ResolutionResult {
            targets: candidates.into_iter().map(|name| FunctionTarget {
                name,
                confidence: Confidence::Low,
                source: TargetSource::TypeMatch,
                reasoning: vec!["基于类型签名匹配".to_string()],
            }).collect(),
            is_complete: false,
        };
    }
    
    // 回退 4: 返回未知
    ResolutionResult {
        targets: vec![FunctionTarget {
            name: "<unknown>".to_string(),
            confidence: Confidence::Unknown,
            source: TargetSource::Unknown,
            reasoning: vec!["无法确定目标函数".to_string()],
        }],
        is_complete: false,
    }
}
```

---

## 附录：参考文献

1. Andersen, L. O. (1994). *Program Analysis and Specialization for the C Programming Language*. PhD thesis, DIKU, University of Copenhagen.

2. Steensgaard, B. (1996). *Points-to Analysis in Almost Linear Time*. POPL '96.

3. Hardekopf, B., & Lin, C. (2007). *The Ant and the Grasshopper: Fast and Accurate Pointer Analysis for Millions of Lines of Code*. PLDI '07.

4. Sui, Y., & Xue, J. (2016). *SVF: Interprocedural Static Value-Flow Analysis in LLVM*. CC '16.

---

*文档版本: 1.0*
*最后更新: 2025-01-04*
*作者: FlowSight Team*

