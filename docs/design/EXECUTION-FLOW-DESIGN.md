# FlowSight 执行流可视化设计文档

> **核心问题**：当前实现只显示简单的函数调用关系，完全没有展示真正的执行流！
>
> **本文档目的**：明确什么是"真正的执行流"，以及如何正确实现它。

---

## 1. 当前实现的问题

### 1.1 当前显示效果（错误示例）

```
do_thaw_all()
├── iterate_supers()
├── kfree()
└── printk()
```

**问题**：
- 这只是简单的函数调用关系，不是执行流
- 没有展示这个函数是如何被调用的
- 没有展示执行上下文（进程/中断/软中断）
- 没有展示时间线关系
- 没有任何有价值的注释说明
- 用户看了完全不知道这个函数什么时候执行、在什么场景下执行

### 1.2 用户真正想知道的

当用户点击一个函数时，他们想知道：

1. **这个函数什么时候会被执行？** - 触发条件是什么？
2. **谁调用了这个函数？** - 完整的调用链，从触发源到这个函数
3. **执行上下文是什么？** - 进程上下文？中断上下文？可以睡眠吗？
4. **这个函数调用了什么？** - 它的执行流程是什么
5. **有什么需要注意的？** - 关键的注释和说明

---

## 2. 正确的执行流可视化

### 2.1 示例1：USB probe 函数

当用户点击 `my_probe()` 函数时，应该显示：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    my_probe() 执行流分析                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  📍 触发条件: USB 设备插入且 ID 匹配                                     │
│  🔄 执行上下文: 进程上下文 (可以睡眠)                                    │
│                                                                          │
│  ════════════════════════════════════════════════════════════════════   │
│  完整调用链 (从触发源到用户代码)                                         │
│  ════════════════════════════════════════════════════════════════════   │
│                                                                          │
│  硬件事件: USB 设备插入                                                  │
│    │                                                                     │
│    └── usb_hub_port_connect()     [drivers/usb/core/hub.c]              │
│          └── usb_new_device()                                           │
│                └── device_add()                                         │
│                      └── bus_probe_device()                             │
│                            └── __device_attach()                        │
│                                  └── driver_probe_device()              │
│                                        └── really_probe()               │
│                                              └── usb_probe_interface()  │
│                                                    └── drv->probe()     │
│                                                          │              │
│                                                          ▼              │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  📦 my_probe()  ← 你的代码从这里开始执行                          │   │
│  │                                                                   │   │
│  │  参数:                                                            │   │
│  │    • interface: USB 接口描述符                                    │   │
│  │    • id: 匹配的设备 ID                                            │   │
│  │                                                                   │   │
│  │  执行流程:                                                        │   │
│  │    ├── kzalloc()           // 分配设备私有数据                    │   │
│  │    ├── usb_set_intfdata()  // 保存私有数据                        │   │
│  │    ├── INIT_WORK()         // 初始化工作队列 → work_handler       │   │
│  │    ├── usb_register_dev()  // 注册字符设备                        │   │
│  │    └── return 0            // 成功                                │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 示例2：WorkQueue handler

当用户点击 `my_work_handler()` 函数时，应该显示：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    my_work_handler() 执行流分析                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  📍 触发条件: schedule_work() 或 queue_work() 被调用后                   │
│  🔄 执行上下文: 进程上下文 (可以睡眠)                                    │
│  ⏰ 执行时机: 由内核调度器决定，不是立即执行！                           │
│                                                                          │
│  ════════════════════════════════════════════════════════════════════   │
│  绑定信息                                                                │
│  ════════════════════════════════════════════════════════════════════   │
│                                                                          │
│  绑定位置: my_probe() 第 45 行                                           │
│  绑定代码: INIT_WORK(&dev->work, my_work_handler)                       │
│  触发位置: my_irq_handler() 第 78 行                                     │
│  触发代码: schedule_work(&dev->work)                                    │
│                                                                          │
│  ════════════════════════════════════════════════════════════════════   │
│  时间线关系 (重要！)                                                     │
│  ════════════════════════════════════════════════════════════════════   │
│                                                                          │
│  时间 ════════════════════════════════════════════════════════════▶     │
│                                                                          │
│  ┌─────────────────────┐                                                │
│  │ 硬中断上下文        │                                                │
│  │ (不可睡眠!)         │                                                │
│  └─────────────────────┘                                                │
│         │                                                                │
│         └── my_irq_handler()                                            │
│               ├── 读取硬件状态 (快速操作)                                │
│               ├── schedule_work(&dev->work)  ← 提交任务，立即返回！     │
│               │         │                                                │
│               │         └── 任务被放入队列，还没执行                    │
│               └── return IRQ_HANDLED                                    │
│                                                                          │
│  ══════════════════════════════════════════════════════════════════════ │
│  ↑ 中断处理到这里就结束了！work handler 还没执行！                      │
│  ══════════════════════════════════════════════════════════════════════ │
│                                                                          │
│                    ... CPU 可能去做其他事情 ...                          │
│                                                                          │
│  ══════════════════════════════════════════════════════════════════════ │
│  ↓ 稍后：内核调度器调度 kworker 线程                                    │
│  ══════════════════════════════════════════════════════════════════════ │
│                                                                          │
│  ┌─────────────────────┐                                                │
│  │ 进程上下文          │                                                │
│  │ (可以睡眠)          │                                                │
│  └─────────────────────┘                                                │
│         │                                                                │
│         └── kworker/xxx 被调度                                          │
│               └── worker_thread()                                       │
│                     └── process_one_work()                              │
│                           └── work->func()                              │
│                                 │                                        │
│                                 ▼                                        │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  📦 my_work_handler()  ← 这才真正执行                             │   │
│  │                                                                   │   │
│  │  执行流程:                                                        │   │
│  │    ├── kmalloc(GFP_KERNEL)  // 可以睡眠分配                       │   │
│  │    ├── copy_to_user()       // 可以睡眠                           │   │
│  │    └── 处理数据...                                                │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.3 示例3：msleep() 的完整执行流

当用户想了解 `msleep(1000)` 是如何工作的：

```
msleep(1000)
│
├── msleep() [kernel/time/timer.c]
│     └── msecs_to_jiffies(1000)        // 毫秒转换为 jiffies
│     └── schedule_timeout_uninterruptible()
│           ├── set_current_state(TASK_UNINTERRUPTIBLE)
│           └── schedule_timeout()
│                 ├── add_timer() / mod_timer()    // 设置超时定时器
│                 │     └── process_timeout()      // 超时回调函数
│                 └── schedule()                   // 让出 CPU
│                       └── __schedule()
│                             ├── deactivate_task()    // 从运行队列移除
│                             ├── pick_next_task()     // CFS 调度器选择下一个任务
│                             └── context_switch()     // 上下文切换
│                                   ├── switch_mm()    // 切换地址空间
│                                   └── switch_to()    // 切换寄存器/栈
│
│  ... 1000ms 期间 CPU 执行其他任务 ...
│
├── [定时器中断触发]
│     └── timer_interrupt() / hrtimer_interrupt()
│           └── run_timer_softirq()
│                 └── process_timeout()            // 执行回调
│                       └── wake_up_process(task)
│                             └── try_to_wake_up()
│                                   ├── set task->state = TASK_RUNNING
│                                   └── enqueue_task()  // 加入运行队列
│
├── [进程被重新调度后，从 schedule() 返回]
│     └── msleep() 返回
│           └── 继续执行 func_b()
```

---

## 3. 实现方案

### 3.1 数据结构设计

```rust
/// 完整的执行流信息
pub struct ExecutionFlowInfo {
    /// 函数名
    pub function_name: String,

    /// 触发条件描述
    pub trigger_condition: String,

    /// 执行上下文
    pub execution_context: ExecutionContext,

    /// 完整的调用链（从触发源到用户代码）
    pub call_chain: Vec<CallChainNode>,

    /// 函数内部的执行流程
    pub internal_flow: Vec<FlowStep>,

    /// 异步关系（如果有）
    pub async_relation: Option<AsyncRelation>,

    /// 绑定信息（如果是回调函数）
    pub binding_info: Option<BindingInfo>,
}

/// 调用链节点
pub struct CallChainNode {
    /// 函数名
    pub function: String,
    /// 源文件
    pub file: Option<String>,
    /// 执行上下文
    pub context: ExecutionContext,
    /// 说明
    pub description: Option<String>,
    /// 是否是用户代码入口
    pub is_user_entry: bool,
}

/// 执行上下文
pub enum ExecutionContext {
    /// 进程上下文（可以睡眠）
    Process { can_sleep: true },
    /// 软中断上下文（不可睡眠）
    SoftIrq,
    /// 硬中断上下文（不可睡眠）
    HardIrq,
    /// 用户空间
    User,
}

/// 异步关系
pub struct AsyncRelation {
    /// 绑定函数
    pub bind_function: String,
    /// 绑定位置
    pub bind_location: Location,
    /// 触发函数
    pub trigger_function: String,
    /// 触发位置
    pub trigger_location: Location,
    /// 时间线说明
    pub timeline_description: String,
}
```

### 3.2 知识库增强

需要在知识库中添加完整的内核调用链：

```yaml
# knowledge/platforms/linux-kernel/call-chains/usb_probe.yaml
usb_probe_call_chain:
  name: "USB probe 调用链"
  trigger_source: "USB 设备插入"
  context: "process"
  can_sleep: true

  nodes:
    - function: "usb_hub_port_connect"
      file: "drivers/usb/core/hub.c"
      description: "Hub 检测到新设备"
    - function: "usb_new_device"
      description: "创建新的 USB 设备"
    - function: "device_add"
      description: "添加设备到设备模型"
    - function: "bus_probe_device"
      description: "总线探测设备"
    - function: "__device_attach"
      description: "尝试匹配驱动"
    - function: "driver_probe_device"
      description: "调用驱动的 probe"
    - function: "really_probe"
      description: "真正执行 probe"
    - function: "usb_probe_interface"
      description: "USB 接口 probe"
    - function: "drv->probe"
      is_user_entry: true
      description: "调用用户的 probe 函数"
```

### 3.3 UI 设计要点

1. **点击函数时**：
   - 显示完整的执行流信息面板
   - 包含触发条件、执行上下文、调用链、内部流程

2. **时间线可视化**：
   - 对于异步操作，清晰展示时间分离
   - 用不同颜色区分不同的执行上下文

3. **交互功能**：
   - 点击调用链中的函数可以跳转
   - 支持前进/后退导航
   - 支持展开/折叠详细信息

---

## 4. 当前需要修复的问题

### 4.1 高优先级

1. **入口点识别** - ✅ 已修复
   - 当没有 module_init 时，使用所有函数作为入口点

2. **调用链注入** - 🔴 待实现
   - 检测到回调函数时，自动注入内核调用链
   - 使用知识库中的调用链数据

3. **执行上下文标注** - 🔴 待实现
   - 每个节点标注执行上下文
   - 显示是否可以睡眠

4. **时间线可视化** - 🔴 待实现
   - 对于异步操作，展示时间分离
   - 清晰标注"这里返回"和"稍后执行"

### 4.2 中优先级

5. **调用者分析增强** - 🟡 部分实现
   - 不仅显示直接调用者
   - 还要显示完整的调用链

6. **点击跳转** - 🟡 部分实现
   - 点击函数跳转到定义
   - 支持跨文件跳转

7. **前进/后退导航** - 🟡 部分实现
   - 记录导航历史
   - 支持快捷键

---

## 5. 总结

**FlowSight 的核心价值不是显示函数调用关系，而是让用户真正理解代码是如何执行的！**

这包括：
- 完整的调用链（从触发源到用户代码）
- 执行上下文（进程/中断/软中断）
- 时间线关系（异步操作的时间分离）
- 有价值的注释说明

只有做到这些，FlowSight 才能真正帮助用户理解复杂的内核代码。
