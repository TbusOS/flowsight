# FlowSight 设计原理

> 函数执行流分析引擎 + 内核知识库驱动的调用链重建

## 要解决的问题

内核代码最大的阅读障碍不是"这个函数做了什么"，而是：

1. **谁调了它？** — 回调函数在 ops table 里赋值，看代码找不到调用者
2. **内核怎么到这儿的？** — 从硬件事件到你的 `probe()` 经过了 6 层内核函数
3. **异步去了哪？** — `schedule_work()` 之后，handler 在完全不同的上下文执行
4. **能不能睡眠？** — 中断上下文调了 `kmalloc(GFP_KERNEL)` 就死锁了

传统工具 (ctags, cscope, clangd) 能找到函数定义，但无法重建完整的内核执行路径。

FlowSight 的核心创新：**用知识库补全静态分析的盲区**。

---

## 整体架构

```
            源代码 (.c)
                │
    ┌───────────┼───────────┐
    │     ① Tree-sitter     │   语法解析 → 函数 + 调用关系
    │     flowsight-parser  │
    └───────────┼───────────┘
                │ ParseResult
    ┌───────────┼───────────┐
    │   ② Analysis Engine   │   异步检测 + 指针解析 + 调用图
    │  flowsight-analysis   │
    │         +             │
    │   ③ Knowledge Base    │   内核调用链注入
    │  flowsight-knowledge  │
    └───────────┼───────────┘
                │ FlowNode 树
    ┌───────────┼───────────┐
    │      ④ CLI 输出       │   树形 / ftrace / 序列图 / JSON
    │     flowsight-cli     │
    └───────────────────────┘
```

### Crate 职责

| Crate | 行数 | 职责 |
|-------|------|------|
| `flowsight-core` | ~800 | 核心类型定义 (FlowNode, ExecutionContext 等) |
| `flowsight-parser` | ~1,500 | tree-sitter C 语言解析 |
| `flowsight-analysis` | ~12,000 | 分析引擎 (流构建、异步检测、指针解析) |
| `flowsight-knowledge` | ~2,000 | 知识库加载与匹配 (137 YAML) |
| `flowsight-cli` | ~1,500 | CLI + REPL 交互 |

---

## ① 解析层：Tree-sitter

### 为什么用 tree-sitter 而不是 clang

| | tree-sitter | clang/libclang |
|--|---|---|
| 速度 | 毫秒级增量解析 | 秒级完整编译 |
| 依赖 | 零依赖，纯 Rust | 需要完整工具链 + 头文件 |
| 容错 | 语法错误不影响其他部分 | 一个 #include 找不到就全挂 |
| 内核适配 | 不需要内核构建环境 | 需要 `make menuconfig` 等 |

内核代码充满宏 (`module_init`, `DECLARE_WORK`, `__init`)，tree-sitter 不展开宏但能提取关键结构，配合知识库正则弥补宏展开的缺失。

### 解析产物

```rust
ParseResult {
    functions: HashMap<String, FunctionDef>,  // 函数名 → 定义
    structs: HashMap<String, StructDef>,      // 结构体
    errors: Vec<String>,                      // 解析错误
}

FunctionDef {
    name: String,              // "my_probe"
    return_type: String,       // "int"
    params: Vec<Parameter>,    // [(struct usb_interface *, intf), ...]
    calls: Vec<String>,        // ["usb_alloc_urb", "kmalloc", ...]
    called_by: Vec<String>,    // 反向引用
    is_callback: bool,         // 是否被识别为回调
    attributes: Vec<String>,   // ["static", "__init"]
}
```

### 解析算法

1. tree-sitter 生成 AST
2. 递归遍历 `function_definition` 节点，提取签名和参数
3. 在函数体内遍历 `call_expression` 节点，收集调用列表
4. 遍历 `struct_specifier` 提取结构体定义
5. 构建双向调用关系 (`calls` / `called_by`)

---

## ② 分析层：四大引擎

### 2a. 异步模式检测 (AsyncTracker)

内核中大量异步机制，代码上没有直接调用关系：

```c
// 注册阶段 — 绑定 handler
INIT_WORK(&dev->work, my_work_handler);

// 触发阶段 — 可能在完全不同的函数里
schedule_work(&dev->work);

// 执行阶段 — 在另一个上下文
void my_work_handler(struct work_struct *work) { ... }
```

**检测算法：**

```
对每种异步模式 (work_struct, timer_list, ...):
    1. 用正则扫描源码，匹配 bind_pattern:
       INIT_WORK(&(?P<var>...), (?P<handler>...))
       → 提取: var = "dev->work", handler = "my_work_handler"

    2. 用正则扫描源码，匹配 trigger_pattern:
       schedule_work(&(?P<var>...))
       → 提取: var = "dev->work"

    3. 通过 var 名关联 bind 和 trigger
       → AsyncBinding { var, handler, mechanism, trigger_locations }
```

**支持的异步模式：**

| 模式 | 绑定宏 | 触发函数 | 执行上下文 |
|------|--------|---------|-----------|
| work_struct | `INIT_WORK()` | `schedule_work()` | 进程 (可睡眠) |
| delayed_work | `INIT_DELAYED_WORK()` | `schedule_delayed_work()` | 进程 |
| timer_list | `timer_setup()` | `mod_timer()` / `add_timer()` | 软中断 (不可睡眠) |
| hrtimer | `hrtimer_init()` | `hrtimer_start()` | 硬中断 |
| IRQ | `request_irq()` | 硬件触发 | 硬中断 |
| threaded IRQ | `request_threaded_irq()` | 硬件触发 | 进程 |

### 2b. 函数指针解析 (FuncPtrResolver)

内核大量使用 ops table 做多态：

```c
static const struct file_operations my_fops = {
    .open    = my_open,      // 函数指针赋值
    .read    = my_read,
    .release = my_release,
};
```

静态分析看到 `vfs_open()` 调用 `f->f_op->open()`，无法知道 `open` 指向谁。

**解析算法：**

```
1. 从知识库加载回调模式:
   usb_driver.probe → pattern: '\.probe\s*=\s*(?P<handler>\w+)'

2. 扫描源码匹配 ops table 初始化:
   struct usb_driver { .probe = my_probe }

3. 建立映射:
   (framework=usb_driver, field=probe) → handler=my_probe
   → 标记 my_probe.is_callback = true
   → 标记 my_probe.callback_context = "usb_driver.probe"

4. 返回 FuncPtrBinding { framework, field, handler, confidence }
   confidence: High (精确匹配) / Medium (类型推断) / Low (启发式)
```

### 2c. 调用图构建 (CallGraph)

三种边：

```
Direct:   foo() { bar(); }           →  foo ──→ bar
Async:    foo() { schedule_work(); } →  foo ~~→ work_handler  (虚线)
Callback: .probe = my_probe         →  usb_probe_interface ──→ my_probe
```

### 2d. 执行流构建 (FlowBuilder) — 核心

**两级构建：**

**第一级 — 用户代码树：**

```
build_flow_tree(entry_func, parse_result, depth=0):
    if depth > MAX_DEPTH or entry_func in visited:
        return
    visited.add(entry_func)

    node = FlowNode { name: entry_func }
    for callee in parse_result[entry_func].calls:
        if callee in parse_result:
            // 用户定义的函数 → 递归展开
            child = build_flow_tree(callee, depth+1)
        else:
            // 外部函数 → 标记为 KernelApi 叶节点
            child = FlowNode { name: callee, type: KernelApi }
        node.children.push(child)
    return node
```

**第二级 — 内核调用链注入 (核心创新)：**

```
build_full_flow_tree(entry_func, parse_result, kb):
    user_tree = build_flow_tree(entry_func)

    if entry_func.is_callback:
        // 查知识库: "my_probe" 是 usb_driver.probe 回调
        chain = kb.get_call_chain("usb_driver", "probe")

        if chain:
            // 构建内核链路节点
            kernel_nodes = chain.nodes.map(|n| FlowNode {
                name: n.function,
                type: KernelInternal,
                source_file: n.file,          // "drivers/usb/core/hub.c"
                context: n.context,           // Process/SoftIrq/HardIrq
                is_kernel_internal: true,
            })

            // 在用户入口标记 "<<< YOU"
            // 将用户树挂在内核链末端
            root = FlowNode { name: chain.trigger }
            root.children = kernel_nodes + [user_tree]
            return root

    return user_tree
```

**效果对比：**

没有知识库注入 (传统工具):
```
my_probe()
├── usb_alloc_urb()
├── kmalloc()
└── usb_submit_urb()
```

有知识库注入 (FlowSight):
```
🎯 USB 设备插入
└── usb_hub_port_connect()              ← drivers/usb/core/hub.c
    └── usb_new_device()
        └── device_add()                ← drivers/base/core.c
            └── bus_probe_device()
                └── driver_probe_device()
                    └── really_probe()
                        └── usb_probe_interface()
                            └── 🔌 my_probe()  <<< YOUR CODE
                                ├── usb_alloc_urb()
                                ├── kmalloc()
                                └── usb_submit_urb()
```

---

## ③ 知识库设计

### 设计哲学

> **静态分析能看到代码写了什么，知识库告诉你内核会怎么调它。**

知识库不是文档，是 **可执行的内核行为模型**。每条记录包含：
- 正则模式 (检测代码中的使用)
- 完整调用链 (从触发源到用户代码)
- 执行上下文 (决定能否睡眠)

### 目录结构

```
knowledge/platforms/linux-kernel/
├── core/                          # 核心子系统
│   ├── workqueue.yaml             # 工作队列
│   ├── timer.yaml                 # 定时器
│   ├── irq.yaml                   # 中断
│   ├── memory.yaml                # 内存管理
│   ├── sched.yaml                 # 调度器
│   └── ...
├── drivers/                       # 驱动框架
│   ├── usb.yaml                   # USB
│   ├── platform.yaml              # Platform Driver
│   ├── pci.yaml                   # PCI
│   ├── i2c.yaml                   # I2C
│   ├── spi.yaml                   # SPI
│   ├── net.yaml                   # 网络驱动
│   └── ...
├── fs/                            # 文件系统
│   ├── vfs.yaml                   # VFS 层
│   └── ...
├── net/                           # 网络协议栈
│   ├── socket.yaml
│   └── ...
└── sync/                          # 同步原语
    ├── spinlock.yaml
    ├── mutex.yaml
    └── ...
```

### YAML Schema

#### 框架回调定义

```yaml
usb_driver:
  description: "USB 设备驱动框架"
  header: "linux/usb.h"

  callbacks:
    probe:
      description: "设备探测回调"
      trigger: "USB 设备插入且 ID 匹配"
      context: process                     # 执行上下文
      can_sleep: true
      signature: "int (*probe)(struct usb_interface *, const struct usb_device_id *)"

      # 检测模式: 在源码中找到 .probe = xxx
      pattern: '\.probe\s*=\s*(?P<handler>\w+)'

      # 完整内核调用链: 从硬件事件到用户回调
      call_chain:
        name: "USB probe 调用链"
        trigger_source: "USB 设备插入"
        nodes:
          - function: usb_hub_port_connect
            file: "drivers/usb/core/hub.c"
            context: process
            description: "Hub 检测到端口变化"
          - function: usb_new_device
            file: "drivers/usb/core/hub.c"
            context: process
          - function: device_add
            file: "drivers/base/core.c"
            context: process
            description: "注册到设备模型"
          - function: bus_probe_device
            file: "drivers/base/dd.c"
            context: process
          - function: driver_probe_device
          - function: really_probe
            description: "实际执行 probe"
          - function: usb_probe_interface
            file: "drivers/usb/core/driver.c"
            description: "USB 核心调用驱动 probe"
          - function: handler()
            is_user_entry: true            # 标记: 这是用户代码入口
            description: "你的 probe 回调"

    disconnect:
      description: "设备断开回调"
      trigger: "USB 设备拔出或驱动卸载"
      context: process
      pattern: '\.disconnect\s*=\s*(?P<handler>\w+)'
      call_chain:
        # ...
```

#### 异步模式定义

```yaml
work_struct:
  description: "工作队列 — 延迟到进程上下文执行"
  context: process
  can_sleep: true

  bind_patterns:
    - pattern: 'INIT_WORK\s*\(\s*&?(?P<var>[\w\.\->]+)\s*,\s*(?P<handler>\w+)\s*\)'
      description: "初始化工作项"

  trigger_patterns:
    - pattern: 'schedule_work\s*\(\s*&?(?P<var>[\w\.\->]+)\s*\)'
      description: "调度工作执行"
    - pattern: 'queue_work\s*\(\s*\w+\s*,\s*&?(?P<var>[\w\.\->]+)\s*\)'
      description: "指定工作队列调度"

  handler_signature: "void (*work_func_t)(struct work_struct *work)"

  # 时间线: 注册阶段 → [异步分界] → 执行阶段
  timeline:
    phase1:
      name: "注册/调度阶段"
      context: any                         # 任何上下文都能调度
    separation: "异步 — 工作队列线程调度"
    phase2:
      name: "执行阶段"
      context: process                     # handler 在进程上下文

  # handler 被调用时的内核路径
  handler_call_chain:
    trigger_source: "schedule_work() 调度"
    nodes:
      - function: queue_work
        file: "kernel/workqueue.c"
      - function: insert_work
      - function: worker_thread
        description: "工作线程循环"
      - function: process_one_work
        description: "取出并执行一个工作项"
      - function: "work->func()"
        is_user_entry: true
```

### 数据结构

```rust
KnowledgeBase {
    frameworks: HashMap<String, Framework>,        // 865 个框架
    async_patterns: HashMap<String, AsyncPattern>, // 4 种异步模式
    kernel_apis: HashMap<String, KernelApi>,       // 25 个核心 API
}

Framework {
    description: String,
    header: Option<String>,
    callbacks: HashMap<String, FrameworkCallback>,  // 共 696 个回调
}

FrameworkCallback {
    description: String,
    trigger: String,                    // 触发条件 (人类可读)
    context: ExecutionContext,           // Process / SoftIrq / HardIrq
    can_sleep: Option<bool>,
    signature: Option<String>,
    pattern: Option<String>,            // 正则检测模式
    call_chain: Option<CallChain>,      // 内核调用链
}

CallChain {
    name: String,
    trigger_source: String,             // "USB 设备插入"
    nodes: Vec<CallChainNode>,
}

CallChainNode {
    function: String,                   // "usb_probe_interface"
    file: Option<String>,               // "drivers/usb/core/driver.c"
    context: ExecutionContext,
    description: Option<String>,
    is_user_entry: bool,                // true = 用户代码入口
}
```

### 知识库规模

| 类别 | 数量 |
|------|------|
| YAML 文件 | 137 |
| 框架 (Framework) | 865 |
| 回调 (Callback) | 696 |
| 调用链 (CallChain) | 11 |
| 异步模式 (AsyncPattern) | 4 |
| 内核 API | 25 |

---

## ④ 执行上下文追踪

### 三种上下文

```
┌─────────────────────────────────────────────────┐
│  进程上下文 (Process Context)                     │
│  ─────────────────────────────                   │
│  可睡眠 ✓  可调度 ✓  可被抢占 ✓                   │
│  例: probe(), work_struct handler, ioctl()       │
│  可用: mutex_lock, kmalloc(GFP_KERNEL), msleep   │
├─────────────────────────────────────────────────┤
│  软中断上下文 (SoftIRQ Context)                   │
│  ─────────────────────────────                   │
│  可睡眠 ✗  可调度 ✗  可被硬中断 ✓                 │
│  例: timer_list handler, NAPI poll, tasklet      │
│  可用: spin_lock, kmalloc(GFP_ATOMIC)            │
│  禁止: mutex_lock, msleep, copy_to_user          │
├─────────────────────────────────────────────────┤
│  硬中断上下文 (HardIRQ Context)                   │
│  ─────────────────────────────                   │
│  可睡眠 ✗  可调度 ✗  可被抢占 ✗                   │
│  例: request_irq handler, hrtimer               │
│  可用: spin_lock_irqsave                         │
│  禁止: 几乎所有可能阻塞的操作                      │
└─────────────────────────────────────────────────┘
```

### 上下文传播

知识库中每个回调和调用链节点都标注了上下文，FlowSight 据此：

1. **传播上下文到子调用** — 如果 `probe()` 是进程上下文，它调用的 `my_init_hw()` 也是
2. **检测上下文违规** — timer handler (软中断) 里调 `mutex_lock()` → 警告
3. **序列图分栏** — 按上下文分到 User Space / Kernel / Hardware 三栏

---

## 完整分析流水线

以分析 `drivers/usb/gadget/udc/fsl_udc_core.c` 为例：

```
输入: fsl_udc_core.c (3000+ 行)
          │
          ▼
[1] tree-sitter 解析
    → 67 个函数定义
    → 每个函数的调用列表
    → 3 个结构体定义
          │
          ▼
[2] 异步模式检测 (AsyncTracker)
    → 检测到 INIT_WORK → 1 个 work_struct 绑定
    → 检测到 request_irq → 1 个 IRQ handler
          │
          ▼
[3] 函数指针解析 (FuncPtrResolver)
    → 匹配 usb_gadget_ops: .pullup = fsl_pullup, ...
    → 匹配 usb_ep_ops: .queue = fsl_ep_queue, ...
    → 标记 8 个回调函数
          │
          ▼
[4] 入口点识别
    → module_init(fsl_udc_init)
    → module_exit(fsl_udc_exit)
    → 回调: fsl_udc_irq, fsl_pullup, fsl_ep_queue, ...
          │
          ▼
[5] 调用图构建
    → 直接调用边: 200+ 条
    → 异步调用边: 2 条
    → 回调边: 8 条
          │
          ▼
[6] 执行流树构建 (FlowBuilder)
    对每个入口点:
      a) 构建用户代码调用树 (递归, depth ≤ 20)
      b) 查知识库获取内核调用链
      c) 注入内核链 → 完整执行流
          │
          ▼
输出: FlowNode 树
    fsl_udc_irq: 120 个节点, 深度 5
    fsl_pullup:   15 个节点, 深度 3
    ...
```

---

## 与同类工具对比

| 能力 | ctags/cscope | clangd/LSP | cflow | FlowSight |
|------|:-----------:|:----------:|:-----:|:---------:|
| 函数定义跳转 | o | o | o | o |
| 调用关系 | o | o | o | o |
| 函数指针解析 | x | 部分 | x | o |
| 内核调用链 | x | x | x | o |
| 异步流追踪 | x | x | x | o |
| 执行上下文 | x | x | x | o |
| 不需要编译环境 | o | x | x | o |
| 知识库驱动 | x | x | x | o |

---

## 设计权衡

### 为什么不用编译器前端

用 tree-sitter 而不是 clang 意味着放弃了类型信息和宏展开，但换来了：
- **零配置**：不需要内核构建环境就能分析
- **速度**：毫秒级解析，适合交互式使用
- **容错**：单个文件语法错误不影响其他文件

缺失的信息通过知识库正则模式补偿 — 用 `INIT_WORK\(&(?P<var>...), (?P<handler>...)\)` 直接匹配宏调用文本。

### 为什么用 YAML 知识库而不是自动推断

全自动推断内核调用链需要分析整个内核源码 (2800 万行)，且需要跨模块指针分析。
知识库方案的优势：
- **准确**：人工验证的调用链比推断更可靠
- **丰富**：包含语义信息 (触发条件、上下文、能否睡眠)
- **可扩展**：添加新框架只需写 YAML，不需要改代码
- **可维护**：跟随内核版本更新只需修改 YAML

### 置信度分级

不是所有分析结果都同样可靠：

| 级别 | 来源 | 例子 |
|------|------|------|
| Certain | 直接调用 + KB 精确匹配 | `foo()` 直接调用 `bar()` |
| Possible | 类型匹配 + 启发式 | ops table 赋值推断 |
| Unknown | 外部函数 | `kmalloc()` — 只知道名字 |

---

## 源码索引

| 模块 | 文件 | 核心内容 |
|------|------|---------|
| 核心类型 | `crates/flowsight-core/src/types.rs` | FlowNode, ExecutionContext, ExecutionFlow |
| 解析器 | `crates/flowsight-parser/src/treesitter.rs` | tree-sitter AST 遍历 |
| 分析入口 | `crates/flowsight-analysis/src/lib.rs` | Analyzer 主流程 |
| 执行流构建 | `crates/flowsight-analysis/src/flow_builder.rs` | 两级流构建算法 |
| 异步检测 | `crates/flowsight-analysis/src/async_tracker.rs` | 正则匹配异步模式 |
| 指针解析 | `crates/flowsight-analysis/src/funcptr.rs` | ops table 解析 |
| 调用图 | `crates/flowsight-analysis/src/callgraph.rs` | 三种边 + 链注入 |
| 知识库 | `crates/flowsight-knowledge/src/lib.rs` | YAML 加载与数据结构 |
| 知识数据 | `knowledge/platforms/linux-kernel/` | 137 个 YAML 文件 |
