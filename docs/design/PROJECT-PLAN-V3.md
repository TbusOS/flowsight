# FlowSight 项目计划 v3.0：Linux 内核执行流可视化 IDE

> 版本: v3.0
> 创建日期: 2026-01-29
> 状态: 规划中
> 前置版本: v2.1 (技术方案参考)

---

## 目录

1. [项目愿景](#1-项目愿景)
2. [核心需求](#2-核心需求)
3. [技术方案](#3-技术方案)
4. [Linux 内核覆盖计划](#4-linux-内核覆盖计划)
5. [AI 辅助集成](#5-ai-辅助集成)
6. [实施计划](#6-实施计划)
7. [技术架构](#7-技术架构)
8. [风险与应对](#8-风险与应对)
9. [里程碑](#9-里程碑)
10. [附录](#附录)

---

## 1. 项目愿景

### 1.1 一句话描述

> **FlowSight：不运行代码，就能看懂 Linux 内核函数的完整执行流程**

### 1.2 核心价值

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FlowSight 解决什么问题？                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Linux 内核代码阅读的四大痛点：                                              │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 1. 函数指针 → 不知道 drv->probe 实际指向哪个函数                    │    │
│  │                                                                      │    │
│  │    static struct usb_driver my_driver = {                           │    │
│  │        .probe = my_probe,  ← 静态分析可以找到                       │    │
│  │    };                                                                │    │
│  │    usb_register(&my_driver);                                        │    │
│  │    // 后来某处: drv->probe(intf, id) ← 这里调用的是谁？             │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 2. 异步队列 → schedule_work() 后代码跑哪去了？                      │    │
│  │                                                                      │    │
│  │    INIT_WORK(&dev->work, my_work_handler);                          │    │
│  │    schedule_work(&dev->work);  ← 提交后立即返回                     │    │
│  │    // my_work_handler 什么时候执行？被谁调用？                       │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 3. 中断处理 → request_irq 注册的 handler 什么时候被调用？           │    │
│  │                                                                      │    │
│  │    request_irq(irq, my_irq_handler, ...);                           │    │
│  │    // 硬件中断来了 → ??? → my_irq_handler                           │    │
│  │    // 中间经过了什么？在什么上下文执行？                             │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 4. 代码量大 → 内核几千万行，人工追踪太难                            │    │
│  │                                                                      │    │
│  │    想知道 msleep(1000) 内部做了什么？                                │    │
│  │    → 定时器设置 → 调度器 → 进程休眠 → 定时器触发 → 进程唤醒        │    │
│  │    人工追踪需要在几十个文件间跳转                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  FlowSight 的解决方案：                                                      │
│  ════════════════════════                                                    │
│  • 静态分析 + 知识库 + AI 辅助                                              │
│  • 自动追踪函数指针绑定                                                      │
│  • 自动补全内核机制调用链                                                    │
│  • 可视化展示完整执行流                                                      │
│  • 不需要真机运行，纯静态分析                                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.3 目标定位

| 维度 | FlowSight 的定位 |
|-----|-----------------|
| **目标** | 可视化代码执行流结构，帮助开发者理解代码 |
| **不是** | 预测运行时实际走哪条路径 |
| **输入** | 源码 + LLVM IR（编译产物） |
| **输出** | 执行流图、时序图、调用链 |
| **使用场景** | 阅读代码、理解机制、调试定位 |

### 1.4 与现有工具对比

| 维度 | 传统 IDE | ftrace/perf | cscope/ctags | **FlowSight** |
|-----|----------|-------------|--------------|---------------|
| 运行方式 | 不需运行 | 需要真机 | 不需运行 | 不需运行 |
| 函数指针分析 | ❌ | 运行时可见 | ❌ | ✅ 静态分析 |
| 异步机制追踪 | ❌ | ✅ | ❌ | ✅ 知识库 |
| 内核调用链 | ❌ | ✅ | ❌ | ✅ 知识库 |
| 执行流可视化 | ❌ | 文本 | ❌ | ✅ 图形化 |
| 离线使用 | ✅ | ❌ | ✅ | ✅ |
| AI 辅助 | ❌ | ❌ | ❌ | ✅ |

---

## 2. 核心需求

### 2.1 用户故事

```
作为一个 Linux 内核/驱动开发者，我希望：

US-1: 点击一个函数，看到它的完整执行流
      - 谁调用了它（向上追溯）
      - 它调用了什么（向下展开）
      - 包括异步调用链（workqueue/timer/irq）

US-2: 对于函数指针，能看到它可能指向哪些函数
      - 静态初始化的指针 → 100% 确定
      - 动态赋值的指针 → 列出可能的目标

US-3: 对于异步机制，能看到完整的触发链
      - schedule_work → worker_thread → process_one_work → my_handler
      - request_irq → do_IRQ → handle_irq → my_handler

US-4: 对于分支，能看到所有可能的路径
      - 不需要预测走哪条，只需列出所有分支
      - 可以选择展开/折叠特定分支

US-5: 能像 ftrace 一样展示执行流
      - 树状结构，带缩进
      - 显示函数名、文件位置、执行上下文
      - 区分同步调用和异步调用

US-6: 支持时序图展示
      - 用户空间 / 内核空间 / 硬件 三列
      - 显示调用顺序和等待关系
```

### 2.2 功能需求

#### FR-1: 执行流分析

| 需求 ID | 描述 | 优先级 |
|--------|------|--------|
| FR-1.1 | 分析函数的直接调用关系 | P0 |
| FR-1.2 | 分析函数指针绑定（静态初始化） | P0 |
| FR-1.3 | 分析函数指针绑定（动态赋值） | P1 |
| FR-1.4 | 分析异步机制调用链（workqueue/timer/irq） | P0 |
| FR-1.5 | 分析分支结构，列出所有路径 | P1 |
| FR-1.6 | 标注执行上下文（进程/软中断/硬中断） | P0 |

#### FR-2: 可视化展示

| 需求 ID | 描述 | 优先级 |
|--------|------|--------|
| FR-2.1 | ftrace 风格的树状执行流 | P0 |
| FR-2.2 | 时序图（用户空间/内核/硬件） | P1 |
| FR-2.3 | 调用图（节点和边） | P1 |
| FR-2.4 | 代码高亮和跳转 | P0 |
| FR-2.5 | 执行流导出（文本/图片/Markdown） | P2 |

#### FR-3: 知识库

| 需求 ID | 描述 | 优先级 |
|--------|------|--------|
| FR-3.1 | 预置 Linux 内核核心机制知识库 | P0 |
| FR-3.2 | 预置主要驱动框架知识库 | P0 |
| FR-3.3 | 用户可扩展知识库 | P1 |
| FR-3.4 | AI 自动生成知识库建议 | P2 |

#### FR-4: AI 辅助

| 需求 ID | 描述 | 优先级 |
|--------|------|--------|
| FR-4.1 | 函数指针目标推断 | P1 |
| FR-4.2 | 未知 API 语义识别 | P1 |
| FR-4.3 | 知识库自动生成 | P2 |
| FR-4.4 | 分析置信度评估 | P1 |

### 2.3 非功能需求

| 需求 ID | 描述 | 目标 |
|--------|------|------|
| NFR-1 | 单函数分析响应时间 | < 3 秒 |
| NFR-2 | 大型模块分析时间 | < 30 秒 |
| NFR-3 | 内存占用 | < 2GB |
| NFR-4 | 离线可用 | 完全支持 |
| NFR-5 | 跨平台 | macOS / Linux / Windows |

### 2.4 效果示例

#### 示例 1: msleep(1000) 执行流

```
msleep(1000)
│
├── msleep() [kernel/time/timer.c]
│   ├── msecs_to_jiffies(1000)           // 毫秒转换为jiffies
│   └── schedule_timeout_uninterruptible()
│       ├── set_current_state(TASK_UNINTERRUPTIBLE)
│       ├── schedule_timeout()
│       │   ├── add_timer() / mod_timer()    // 设置超时定时器
│       │   │   └── process_timeout()        // 超时回调函数
│       │   └── schedule()                   // 让出CPU
│       │       └── __schedule()
│       │           ├── deactivate_task()    // 从运行队列移除
│       │           ├── pick_next_task()     // CFS调度器选择下一个任务
│       │           └── context_switch()     // 上下文切换
│       │               ├── switch_mm()      // 切换地址空间
│       │               └── switch_to()      // 切换寄存器/栈
│
│   ... 1000ms 期间 CPU 执行其他任务 ...
│
├── [定时器中断触发]
│   └── timer_interrupt() / hrtimer_interrupt()
│       └── run_timer_softirq()
│           └── process_timeout()            // 执行回调
│               └── wake_up_process(task)
│                   └── try_to_wake_up()
│                       ├── set task->state = TASK_RUNNING
│                       └── enqueue_task()   // 加入运行队列
│
├── [进程被重新调度后, 从 schedule() 返回]
│   └── msleep() 返回
│       └── 继续执行 func_b()
```

#### 示例 2: insmod 和 probe 的时间线关系

```
insmod my_driver.ko 执行流
│
├── 用户空间: insmod my_driver.ko
│   └── 内核: sys_init_module()
│       └── load_module()
│           └── do_init_module()
│               └── mod->init()
│                   └── my_init()              ← 用户的 init 函数
│                       └── usb_register(&my_driver)
│                           └── 注册到 USB 子系统
│                               └── return 0
│
│   ↑ insmod 到这里就返回了！probe 还没执行！
│
│   ... 时间流逝 ...
│
├── ↓ 某个时刻: USB 设备插入
│   └── 硬件中断: USB 设备插入
│       └── usb_hub_port_connect()
│           └── usb_new_device()
│               └── device_add()
│                   └── bus_probe_device()
│                       └── __device_attach()
│                           └── driver_probe_device()
│                               └── really_probe()
│                                   └── usb_probe_interface()
│                                       └── drv->probe()
│                                           ▼
│                                           my_probe()  ← 这才执行!
```

#### 示例 3: 中断上半部和 WorkQueue 时序

```
时间线 ══════════════════════════════════════════════════════════════►

┌─────────────────────────────────────────────────────────────────────┐
│ 硬中断上下文 (不可睡眠!)                                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  my_irq_handler()                                                   │
│  ├── 读取硬件状态 (快速操作)                                         │
│  ├── queue_work(wq, &dev->work)  ← 提交任务，立即返回!              │
│  │   └── 任务被放入队列，还没执行                                    │
│  └── return IRQ_HANDLED                                             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘

    ... CPU 可能去做其他事情 ...

┌─────────────────────────────────────────────────────────────────────┐
│ ↓ 稍后: 内核调度器调度 kworker 线程                                  │
├─────────────────────────────────────────────────────────────────────┤
│ 进程上下文 (可以睡眠)                                                │
│                                                                      │
│  kworker/xxx 被调度                                                  │
│  └── worker_thread()                                                │
│      └── process_one_work()                                         │
│          └── work->func()                                           │
│              ▼                                                       │
│              my_work_handler()  ← 这才真正执行耗时操作               │
│              ├── 可以调用 kmalloc(GFP_KERNEL)                        │
│              ├── 可以睡眠等待                                        │
│              └── 处理数据...                                         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. 技术方案

### 3.1 技术路线

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           技术路线总览                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  输入层                                                                      │
│  ══════                                                                      │
│  ┌──────────────┐    ┌──────────────┐                                       │
│  │ C 源码       │    │ LLVM IR      │  ← 编译产物 (clang -emit-llvm)        │
│  │ Tree-sitter  │    │ inkwell      │                                       │
│  └──────────────┘    └──────────────┘                                       │
│         │                   │                                                │
│         └─────────┬─────────┘                                                │
│                   ▼                                                          │
│  分析层                                                                      │
│  ══════                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                       静态分析引擎                                   │    │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐    │    │
│  │  │ 直接调用   │  │ 函数指针   │  │ 分支结构   │  │ 类型信息   │    │    │
│  │  │ 分析       │  │ 绑定分析   │  │ 分析       │  │ 分析       │    │    │
│  │  └────────────┘  └────────────┘  └────────────┘  └────────────┘    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                   │                                                          │
│                   ▼                                                          │
│  增强层                                                                      │
│  ══════                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  ┌────────────────────┐    ┌────────────────────┐                   │    │
│  │  │ 知识库             │    │ AI 辅助            │                   │    │
│  │  │                    │    │                    │                   │    │
│  │  │ • 内核机制调用链   │    │ • 函数指针推断     │                   │    │
│  │  │ • 回调触发模式     │    │ • 未知API识别      │                   │    │
│  │  │ • 执行上下文       │    │ • 置信度评估       │                   │    │
│  │  └────────────────────┘    └────────────────────┘                   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                   │                                                          │
│                   ▼                                                          │
│  输出层                                                                      │
│  ══════                                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                       │
│  │ 执行流树     │  │ 时序图       │  │ 调用图       │                       │
│  │ (ftrace风格) │  │ (UML风格)    │  │ (节点/边)    │                       │
│  └──────────────┘  └──────────────┘  └──────────────┘                       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 分析能力与准确度

| 分析类型 | 技术手段 | 准确度 | 说明 |
|---------|---------|--------|------|
| **直接函数调用** | LLVM IR call 指令 | 100% | 编译产物，完全准确 |
| **静态初始化函数指针** | LLVM IR 全局变量分析 | 100% | `.probe = my_probe` |
| **宏绑定回调** | 知识库模式匹配 | 100% | `INIT_WORK`, `request_irq` |
| **动态赋值函数指针** | 数据流分析 + AI | 80-95% | 需要追踪赋值点 |
| **内核机制调用链** | 知识库 | 100% | 预置内核行为 |
| **分支路径** | 控制流分析 | 100% | 列出所有分支 |
| **未知 API** | AI 推断 | 70-90% | 需要标注置信度 |

### 3.3 技术栈

| 层级 | 技术 | 用途 | 备注 |
|-----|------|------|------|
| **前端框架** | React 18 + TypeScript | UI 开发 | |
| **代码编辑器** | Monaco Editor | 代码展示/跳转 | |
| **图形可视化** | @xyflow/react | 调用图、执行流 | |
| **状态管理** | Zustand | 前端状态 | |
| **桌面框架** | Tauri 2.0 | 跨平台桌面 | Rust 后端 |
| **后端语言** | Rust | 分析引擎 | |
| **源码解析** | Tree-sitter | C 代码 AST | 快速预览 |
| **IR 解析** | inkwell | LLVM IR 绑定 | 精确分析 |
| **AI 推断** | Ollama (本地) | 函数指针推断等 | 可选云端 |
| **数据存储** | SQLite + Sled | 项目索引 | |

---

## 4. Linux 内核覆盖计划

### 4.1 覆盖范围总览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Linux 内核子系统覆盖计划                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                        核心子系统                                    │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │    │
│  │  │ 调度器  │ │ 内存    │ │  VFS   │ │ 网络    │ │ 块层    │       │    │
│  │  │ sched/  │ │  mm/    │ │  fs/   │ │  net/   │ │ block/  │       │    │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘       │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                        异步机制                                      │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │    │
│  │  │workqueue │ │  timer   │ │   irq    │ │ softirq  │ │ tasklet  │  │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐                            │    │
│  │  │ kthread  │ │  RCU     │ │ irq_work │                            │    │
│  │  └──────────┘ └──────────┘ └──────────┘                            │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                        同步原语                                      │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │    │
│  │  │ spinlock │ │  mutex   │ │semaphore │ │ rwlock   │ │completion│  │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐                            │    │
│  │  │wait_queue│ │ seqlock  │ │  rwsem   │                            │    │
│  │  └──────────┘ └──────────┘ └──────────┘                            │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                        驱动框架 (30+)                                │    │
│  │                                                                      │    │
│  │  总线驱动:                                                           │    │
│  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐          │    │
│  │  │ USB │ │ PCI │ │Platf│ │ I2C │ │ SPI │ │MDIO │ │AMBA │          │    │
│  │  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘          │    │
│  │                                                                      │    │
│  │  设备类驱动:                                                         │    │
│  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                   │    │
│  │  │Char │ │Block│ │ Net │ │ TTY │ │SCSI │ │NVMe │                   │    │
│  │  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘                   │    │
│  │                                                                      │    │
│  │  子系统驱动:                                                         │    │
│  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐  │    │
│  │  │Input│ │GPIO │ │Clock│ │Regul│ │Pinct│ │Therm│ │ DMA │ │ PWM │  │    │
│  │  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘  │    │
│  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐  │    │
│  │  │ IIO │ │ RTC │ │Wdog │ │Power│ │ PHY │ │Reset│ │ MFD │ │Mbox │  │    │
│  │  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘  │    │
│  │                                                                      │    │
│  │  多媒体驱动:                                                         │    │
│  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                                   │    │
│  │  │ DRM │ │V4L2 │ │ALSA │ │ MTD │                                   │    │
│  │  └─────┘ └─────┘ └─────┘ └─────┘                                   │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 知识库详细列表

#### 4.2.1 核心机制 (P0 - 必须)

| 机制 | 知识库文件 | 状态 | 关键调用链 |
|------|-----------|------|-----------|
| WorkQueue | `core/workqueue.yaml` | ✅ | schedule_work → worker_thread → work->func |
| Timer | `core/timer.yaml` | ✅ | add_timer → run_timer_softirq → timer->func |
| HRTimer | `core/hrtimer.yaml` | ⬜ | hrtimer_start → hrtimer_interrupt → timer->func |
| IRQ | `core/irq.yaml` | ✅ | request_irq → do_IRQ → handle_irq → handler |
| Threaded IRQ | `core/threaded_irq.yaml` | ⬜ | request_threaded_irq → irq_thread → handler |
| SoftIRQ | `core/softirq.yaml` | ✅ | raise_softirq → do_softirq → action |
| Tasklet | `core/tasklet.yaml` | ⬜ | tasklet_schedule → tasklet_action → func |
| KThread | `core/kthread.yaml` | ✅ | kthread_create → kthread → threadfn |
| RCU | `core/rcu.yaml` | ✅ | call_rcu → rcu_process_callbacks → func |

#### 4.2.2 同步原语 (P1)

| 原语 | 知识库文件 | 状态 | 说明 |
|------|-----------|------|------|
| Spinlock | `sync/spinlock.yaml` | ⬜ | spin_lock/spin_unlock |
| Mutex | `sync/mutex.yaml` | ⬜ | mutex_lock/mutex_unlock |
| Semaphore | `sync/semaphore.yaml` | ✅ | down/up |
| RWLock | `sync/rwlock.yaml` | ✅ | read_lock/write_lock |
| Completion | `sync/completion.yaml` | ✅ | wait_for_completion/complete |
| Wait Queue | `sync/wait_queue.yaml` | ⬜ | wait_event/wake_up |
| Seqlock | `sync/seqlock.yaml` | ⬜ | read_seqbegin/write_seqlock |
| RW Semaphore | `sync/rwsem.yaml` | ⬜ | down_read/down_write |

#### 4.2.3 核心子系统 (P1)

| 子系统 | 知识库文件 | 状态 | 关键回调 |
|-------|-----------|------|---------|
| 调度器 | `subsys/sched.yaml` | ⬜ | sched_class ops |
| 内存管理 | `subsys/mm.yaml` | ⬜ | vm_operations, address_space_ops |
| VFS | `core/vfs.yaml` | ✅ | file_operations, inode_operations |
| 网络协议栈 | `subsys/net.yaml` | ⬜ | proto_ops, net_device_ops |
| 块层 | `subsys/block.yaml` | ⬜ | block_device_operations, request_queue |

#### 4.2.4 总线驱动框架 (P0)

| 框架 | 知识库文件 | 状态 | 关键回调 |
|-----|-----------|------|---------|
| USB | `drivers/usb.yaml` | ✅ | probe/disconnect/suspend/resume |
| PCI | `drivers/pci.yaml` | ✅ | probe/remove/suspend/resume |
| Platform | `drivers/platform.yaml` | ✅ | probe/remove |
| I2C | `drivers/i2c.yaml` | ✅ | probe/remove |
| SPI | `drivers/spi.yaml` | ✅ | probe/remove |
| MDIO | `drivers/mdio.yaml` | ⬜ | probe/remove |
| AMBA | `drivers/amba.yaml` | ⬜ | probe/remove |
| ACPI | `drivers/acpi.yaml` | ⬜ | add/remove |
| OF (DeviceTree) | `drivers/of.yaml` | ⬜ | of_device_id 匹配 |

#### 4.2.5 设备类驱动框架 (P1)

| 框架 | 知识库文件 | 状态 | 关键回调 |
|-----|-----------|------|---------|
| 字符设备 | `drivers/char_dev.yaml` | ✅ | file_operations |
| 块设备 | `drivers/block.yaml` | ⬜ | block_device_operations |
| 网络设备 | `drivers/netdev.yaml` | ⬜ | net_device_ops, ethtool_ops |
| TTY | `drivers/tty.yaml` | ⬜ | tty_operations |
| SCSI | `drivers/scsi.yaml` | ⬜ | scsi_host_template |
| NVMe | `drivers/nvme.yaml` | ⬜ | nvme_ctrl_ops |

#### 4.2.6 子系统驱动框架 (P1-P2)

| 框架 | 知识库文件 | 优先级 | 状态 | 关键回调 |
|-----|-----------|--------|------|---------|
| Input | `drivers/input.yaml` | P1 | ⬜ | input_handler ops |
| GPIO | `drivers/gpio.yaml` | P1 | ⬜ | gpio_chip ops |
| Clock | `drivers/clk.yaml` | P1 | ⬜ | clk_ops |
| Regulator | `drivers/regulator.yaml` | P1 | ⬜ | regulator_ops |
| Pinctrl | `drivers/pinctrl.yaml` | P1 | ⬜ | pinctrl_ops, pinmux_ops |
| Thermal | `drivers/thermal.yaml` | P1 | ⬜ | thermal_zone_device_ops |
| DMA Engine | `drivers/dma.yaml` | P1 | ⬜ | dma_device ops |
| PWM | `drivers/pwm.yaml` | P2 | ⬜ | pwm_ops |
| IIO | `drivers/iio.yaml` | P2 | ⬜ | iio_info |
| RTC | `drivers/rtc.yaml` | P2 | ⬜ | rtc_class_ops |
| Watchdog | `drivers/watchdog.yaml` | P2 | ⬜ | watchdog_ops |
| Power/PM | `drivers/power.yaml` | P1 | ⬜ | dev_pm_ops |
| PHY | `drivers/phy.yaml` | P2 | ⬜ | phy_ops |
| Reset | `drivers/reset.yaml` | P2 | ⬜ | reset_control_ops |
| MFD | `drivers/mfd.yaml` | P2 | ⬜ | mfd_cell |
| Mailbox | `drivers/mailbox.yaml` | P2 | ⬜ | mbox_chan_ops |
| IOMMU | `drivers/iommu.yaml` | P2 | ⬜ | iommu_ops |
| Remoteproc | `drivers/remoteproc.yaml` | P2 | ⬜ | rproc_ops |
| RPMsg | `drivers/rpmsg.yaml` | P2 | ⬜ | rpmsg_driver ops |

#### 4.2.7 多媒体驱动框架 (P2)

| 框架 | 知识库文件 | 状态 | 关键回调 |
|-----|-----------|------|---------|
| DRM/GPU | `drivers/drm.yaml` | ⬜ | drm_driver, drm_crtc_funcs |
| V4L2 | `drivers/v4l2.yaml` | ⬜ | v4l2_file_operations |
| ALSA | `drivers/sound.yaml` | ⬜ | snd_pcm_ops |
| MTD | `drivers/mtd.yaml` | ⬜ | mtd_info ops |

### 4.3 知识库结构示例

```yaml
# knowledge/platforms/linux-kernel/core/workqueue.yaml

name: workqueue
description: "Work Queue - 将工作延迟到进程上下文执行"
header: "linux/workqueue.h"
icon: "⚙️"

# 执行上下文
context:
  type: "process"
  can_sleep: true
  can_schedule: true
  preemptible: true

# 回调绑定模式
bind_patterns:
  - name: "INIT_WORK"
    pattern: 'INIT_WORK\s*\(\s*&?(?P<work>[\w\.\->]+)\s*,\s*(?P<handler>\w+)\s*\)'
    handler_field: "handler"
    work_field: "work"
    description: "初始化 work_struct 并绑定处理函数"

  - name: "INIT_DELAYED_WORK"
    pattern: 'INIT_DELAYED_WORK\s*\(\s*&?(?P<work>[\w\.\->]+)\s*,\s*(?P<handler>\w+)\s*\)'
    handler_field: "handler"
    work_field: "work"
    description: "初始化 delayed_work 并绑定处理函数"

# 触发模式
trigger_patterns:
  - name: "schedule_work"
    pattern: 'schedule_work\s*\(\s*&?(?P<work>[\w\.\->]+)\s*\)'
    work_field: "work"
    workqueue: "system_wq"
    description: "将 work 提交到系统默认工作队列"

  - name: "queue_work"
    pattern: 'queue_work\s*\(\s*(?P<wq>\w+)\s*,\s*&?(?P<work>[\w\.\->]+)\s*\)'
    work_field: "work"
    workqueue_field: "wq"
    description: "将 work 提交到指定工作队列"

  - name: "schedule_delayed_work"
    pattern: 'schedule_delayed_work\s*\(\s*&?(?P<work>[\w\.\->]+)\s*,\s*(?P<delay>.+)\s*\)'
    work_field: "work"
    delay_field: "delay"
    description: "延迟指定时间后执行"

# 内核调用链 (从触发到用户回调)
kernel_call_chain:
  description: "schedule_work() 被调用后的执行链"
  trigger: "schedule_work(&work)"
  chain:
    - function: "__queue_work"
      file: "kernel/workqueue.c"
      description: "将 work 加入工作队列"
    - function: "insert_work"
      description: "插入到工作链表"
    - function: "wake_up_worker"
      description: "唤醒工作线程 (如果需要)"
    - separator: "... 稍后，kworker 线程被调度 ..."
      type: "async_boundary"
    - function: "worker_thread"
      file: "kernel/workqueue.c"
      description: "工作线程主循环"
    - function: "process_one_work"
      description: "处理单个 work"
    - function: "work->func(work)"
      is_user_callback: true
      description: "调用用户注册的处理函数"

# 使用注意事项
notes:
  - "work handler 在进程上下文执行，可以睡眠"
  - "使用 container_of(work, struct xxx, work_field) 获取包含结构体"
  - "同一个 work 不能同时被多次 schedule"
  - "cancel_work_sync() 会等待正在执行的 work 完成"

# 相关 API
related_apis:
  - "create_workqueue"
  - "destroy_workqueue"
  - "flush_work"
  - "cancel_work_sync"
  - "flush_workqueue"
```

---

## 5. AI 辅助集成

### 5.1 AI 的角色定位

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    AI 辅助模式 (非对话式)                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  AI 不是对话助手，而是静默的分析增强器                                       │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                                                                      │    │
│  │  静态分析 ───► 分析结果 ───► AI 增强 ───► 增强后结果              │    │
│  │                                 │                                    │    │
│  │                                 │                                    │    │
│  │                   ┌─────────────┴─────────────┐                      │    │
│  │                   │                           │                      │    │
│  │                   ▼                           ▼                      │    │
│  │         ┌─────────────────┐         ┌─────────────────┐             │    │
│  │         │ 能确定的        │         │ 不确定的         │             │    │
│  │         │                 │         │                 │             │    │
│  │         │ • 直接调用      │         │ • 动态函数指针  │             │    │
│  │         │ • 静态绑定      │         │ • 未知 API      │             │    │
│  │         │ • 知识库匹配    │         │ • 复杂数据流    │             │    │
│  │         │                 │         │                 │             │    │
│  │         │ AI: 不参与      │         │ AI: 推断 + 标注│             │    │
│  │         │     (100%准确)  │         │     置信度      │             │    │
│  │         └─────────────────┘         └─────────────────┘             │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 AI 辅助场景

#### 场景 1: 函数指针目标推断

```
输入代码:
─────────
    dev->ops->custom_handler(dev, data);

静态分析结果:
─────────────
    目标: Unknown (动态函数指针)

AI 分析过程:
─────────────
    1. 查找 dev->ops 的类型定义
    2. 扫描代码中对 dev->ops 的赋值
    3. 分析 custom_handler 字段的赋值点
    4. 找到: dev->ops = &my_ops;
             static struct xxx_ops my_ops = {
                 .custom_handler = my_custom_handler,
             };

AI 输出:
────────
    目标: my_custom_handler
    置信度: 90%
    依据: 静态初始化 my_ops.custom_handler
```

#### 场景 2: 未知驱动框架识别

```
输入代码:
─────────
    some_framework_register(&my_driver);

知识库:
────────
    没有 some_framework 的条目

AI 分析过程:
─────────────
    1. 分析 some_framework_register 函数签名
    2. 识别参数类型 struct xxx_driver
    3. 分析结构体中的回调字段
    4. 推断这是一个驱动注册函数

AI 输出:
────────
    类型: 驱动框架注册函数
    回调字段: probe, remove, suspend, resume
    触发时机: 设备匹配时调用 probe
    置信度: 85%
    建议: 生成知识库条目供人工确认
```

#### 场景 3: 分支重要性判断

```
输入代码:
─────────
    if (unlikely(!dev)) {
        dev_err(&pdev->dev, "device not found\n");
        return -ENODEV;
    }
    // ... 主要逻辑 ...

AI 分析:
────────
    分支类型: 错误处理 (unlikely 提示)
    主路径: dev != NULL 的情况
    建议: 折叠显示错误处理分支
```

### 5.3 AI 集成架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         AI 集成架构                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                       分析引擎 (Rust)                                │    │
│  │                                                                      │    │
│  │  静态分析完成后，对不确定的部分调用 AI:                             │    │
│  │                                                                      │    │
│  │  ┌──────────────────────────────────────────────────────────────┐   │    │
│  │  │                    AI 推断引擎                                │   │    │
│  │  │                                                               │   │    │
│  │  │  优先级顺序:                                                  │   │    │
│  │  │                                                               │   │    │
│  │  │  1. 本地模型 (优先，离线可用)                                 │   │    │
│  │  │     ┌──────────────────────────────────────────────────┐     │   │    │
│  │  │     │ Ollama                                            │     │   │    │
│  │  │     │ • 推荐模型: deepseek-coder:6.7b / codellama:7b   │     │   │    │
│  │  │     │ • 内存需求: ~8GB                                  │     │   │    │
│  │  │     │ • 特点: 完全离线，响应快                          │     │   │    │
│  │  │     └──────────────────────────────────────────────────┘     │   │    │
│  │  │                                                               │   │    │
│  │  │  2. 云端模型 (可选，需要联网)                                 │   │    │
│  │  │     ┌──────────────────────────────────────────────────┐     │   │    │
│  │  │     │ Claude API / OpenAI API                           │     │   │    │
│  │  │     │ • 更高准确度                                      │     │   │    │
│  │  │     │ • 需要 API Key                                    │     │   │    │
│  │  │     │ • 用于复杂场景                                    │     │   │    │
│  │  │     └──────────────────────────────────────────────────┘     │   │    │
│  │  │                                                               │   │    │
│  │  │  3. 无 AI (fallback)                                          │   │    │
│  │  │     标注为 "Unknown"，等待用户补充                            │   │    │
│  │  │                                                               │   │    │
│  │  └──────────────────────────────────────────────────────────────┘   │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.4 AI 知识库自动生成

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    AI 自动生成知识库流程                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  1. 触发条件                                                                 │
│     ─────────                                                                │
│     • 遇到未知的驱动框架注册函数                                            │
│     • 用户请求分析新框架                                                    │
│                                                                              │
│  2. AI 分析步骤                                                              │
│     ────────────                                                             │
│     a. 扫描框架相关头文件                                                    │
│     b. 识别核心数据结构 (xxx_driver, xxx_device, xxx_ops)                   │
│     c. 分析回调函数字段                                                      │
│     d. 查找注册/注销函数                                                     │
│     e. 分析回调触发时机                                                      │
│                                                                              │
│  3. 生成知识库草稿                                                           │
│     ────────────────                                                         │
│     knowledge/platforms/linux-kernel/drivers/xxx.yaml.draft                 │
│                                                                              │
│  4. 人工审核                                                                 │
│     ──────────                                                               │
│     • 检查准确性                                                             │
│     • 补充细节                                                               │
│     • 合入主知识库                                                           │
│                                                                              │
│  示例: 分析 hwmon 框架                                                       │
│  ────────────────────                                                        │
│  输入: drivers/hwmon/ 目录                                                   │
│  输出:                                                                       │
│                                                                              │
│  name: hwmon                                                                 │
│  description: "Hardware Monitoring 驱动框架"                                 │
│  header: "linux/hwmon.h"                                                    │
│                                                                              │
│  callbacks:                                                                  │
│    - name: read                                                             │
│      trigger: "用户读取 /sys/class/hwmon/hwmonX/xxx"                        │
│      context: process                                                       │
│    - name: write                                                            │
│      trigger: "用户写入 sysfs 属性"                                         │
│      context: process                                                       │
│    - name: is_visible                                                       │
│      trigger: "设备注册时检查属性可见性"                                    │
│      context: process                                                       │
│                                                                              │
│  kernel_call_chain:                                                          │
│    - sysfs_kf_seq_show                                                      │
│    - dev_attr_show                                                          │
│    - hwmon_attr_show                                                        │
│    - chip->ops->read  # 用户回调                                            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.5 置信度标注

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         置信度等级                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  🟢 Certain (100%)                                                          │
│     ─────────────────                                                        │
│     • 直接函数调用 (LLVM IR call 指令)                                      │
│     • 静态初始化的函数指针                                                   │
│     • 知识库精确匹配                                                         │
│                                                                              │
│  🟡 High (90%+)                                                              │
│     ─────────────                                                            │
│     • 简单数据流分析确定的函数指针                                          │
│     • AI 高置信度推断                                                        │
│                                                                              │
│  🟠 Medium (70-90%)                                                          │
│     ─────────────────                                                        │
│     • 复杂数据流分析                                                         │
│     • AI 中等置信度推断                                                      │
│     • 有多个可能目标，但某个更可能                                          │
│                                                                              │
│  🔴 Low (< 70%)                                                              │
│     ─────────────                                                            │
│     • AI 低置信度推断                                                        │
│     • 多个等可能目标                                                         │
│                                                                              │
│  ⚪ Unknown                                                                  │
│     ──────────                                                               │
│     • 无法分析                                                               │
│     • 需要运行时信息                                                         │
│     • 等待用户补充                                                           │
│                                                                              │
│  UI 展示:                                                                    │
│  ─────────                                                                   │
│  • 🟢/🟡: 正常显示                                                          │
│  • 🟠: 显示但带警告图标                                                     │
│  • 🔴: 显示为"可能"，列出所有候选                                          │
│  • ⚪: 显示为"未知"，提供手动标注入口                                      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. 实施计划

### 6.1 阶段总览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           实施阶段                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Phase 1: 核心引擎                                                           │
│  ═══════════════════                                                         │
│  • LLVM IR 解析器                                                            │
│  • 直接调用分析                                                              │
│  • 静态函数指针绑定                                                          │
│  • 基础知识库 (workqueue/timer/irq)                                         │
│  • 执行流数据结构                                                            │
│                                                                              │
│  Phase 2: 可视化 UI                                                          │
│  ═══════════════════                                                         │
│  • ftrace 风格执行流视图                                                     │
│  • 代码编辑器集成                                                            │
│  • 点击函数 → 展示执行流                                                    │
│  • 基本交互 (展开/折叠/跳转)                                                │
│                                                                              │
│  Phase 3: 知识库扩展                                                         │
│  ════════════════════                                                        │
│  • 全部核心机制 (softirq/tasklet/hrtimer...)                                │
│  • 全部 P0 驱动框架                                                          │
│  • 同步原语知识库                                                            │
│  • 核心子系统知识库                                                          │
│                                                                              │
│  Phase 4: AI 辅助集成                                                        │
│  ═══════════════════════                                                     │
│  • Ollama 本地模型集成                                                       │
│  • 函数指针推断                                                              │
│  • 未知 API 识别                                                             │
│  • 置信度标注                                                                │
│                                                                              │
│  Phase 5: 完善与扩展                                                         │
│  ═══════════════════════                                                     │
│  • P1/P2 驱动框架知识库                                                      │
│  • 时序图视图                                                                │
│  • 用户自定义知识库                                                          │
│  • 知识库自动生成                                                            │
│  • 分析结果学习/反馈                                                         │
│                                                                              │
│  Phase 6: 架构支持 (可选)                                                    │
│  ═════════════════════════                                                   │
│  • ARM 架构特定支持                                                          │
│  • x86 架构特定支持                                                          │
│  • 寄存器级分析                                                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Phase 详细计划

#### Phase 1: 核心引擎

**目标**: 建立分析能力基础

**模块**:

| 模块 | 文件 | 说明 |
|-----|------|------|
| LLVM IR 解析 | `crates/flowsight-llvm/` | ✅ 已有基础 |
| 调用分析 | `crates/flowsight-analysis/call_graph.rs` | 直接调用 + 函数指针 |
| 知识库引擎 | `crates/flowsight-knowledge/` | 知识库加载和匹配 |
| 执行流构建 | `crates/flowsight-analysis/execution_flow.rs` | 构建执行流树 |

**知识库**:
- `core/workqueue.yaml` ✅
- `core/timer.yaml` ✅
- `core/irq.yaml` ✅
- `core/kthread.yaml` ✅
- `core/rcu.yaml` ✅

**验收标准**:
- [ ] 能解析 Linux kernel 的 .bc 文件
- [ ] 能提取直接函数调用
- [ ] 能识别静态初始化的函数指针
- [ ] 能匹配 INIT_WORK/schedule_work 等模式
- [ ] 能输出执行流数据结构

---

#### Phase 2: 可视化 UI

**目标**: 基本可用的 UI

**组件**:

| 组件 | 文件 | 说明 |
|-----|------|------|
| 执行流视图 | `app/src/components/FlowView/FlowTextView.tsx` | ftrace 风格 |
| 代码编辑器 | `app/src/components/Editor/` | Monaco 集成 |
| 主布局 | `app/src/components/layout/` | 三栏布局 |
| 状态管理 | `app/src/store/` | Zustand |

**功能**:
- 点击函数名 → 显示执行流
- 执行流树展开/折叠
- 点击执行流节点 → 跳转到代码
- 区分同步/异步调用 (不同颜色/图标)
- 显示执行上下文标签

**验收标准**:
- [ ] 能展示 workqueue 完整调用链
- [ ] 能展示 timer 完整调用链
- [ ] UI 响应流畅

---

#### Phase 3: 知识库扩展

**目标**: 覆盖 Linux 内核主要机制

**知识库清单**:

| 类别 | 数量 | 状态 |
|-----|------|------|
| 核心机制 | 9 | 5 ✅ / 4 ⬜ |
| 同步原语 | 8 | 3 ✅ / 5 ⬜ |
| 核心子系统 | 5 | 1 ✅ / 4 ⬜ |
| P0 驱动框架 | 9 | 6 ✅ / 3 ⬜ |
| **Phase 3 总计** | **31** | **15 ✅ / 16 ⬜** |

**验收标准**:
- [ ] 核心机制 100% 覆盖
- [ ] 同步原语 100% 覆盖
- [ ] P0 驱动框架 100% 覆盖

---

#### Phase 4: AI 辅助集成

**目标**: 提升分析准确度

**模块**:

| 模块 | 说明 |
|-----|------|
| AI 接口抽象 | 统一本地/云端模型接口 |
| Ollama 集成 | 本地模型支持 |
| 函数指针推断 | 动态赋值分析 |
| 置信度系统 | 标注分析结果可信度 |

**验收标准**:
- [ ] Ollama 集成完成
- [ ] 函数指针推断准确率 > 80%
- [ ] 置信度标注显示正常

---

#### Phase 5: 完善与扩展

**目标**: 完整覆盖 + 高级功能

**内容**:
- P1/P2 驱动框架 (20+)
- 时序图视图
- 用户自定义知识库
- 知识库自动生成建议
- 分析结果反馈学习

---

#### Phase 6: 架构支持 (可选)

**目标**: 架构特定分析

**内容**:
- ARM 架构寄存器
- x86 架构寄存器
- 中断向量表
- 架构特定调用约定

---

## 7. 技术架构

### 7.1 整体架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FlowSight 架构                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    Frontend (React + TypeScript)                    │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐            │    │
│  │  │ Editor   │  │ FlowView │  │ GraphView│  │ Settings │            │    │
│  │  │ Monaco   │  │ ftrace   │  │ xyflow   │  │          │            │    │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘            │    │
│  └────────────────────────────────┬────────────────────────────────────┘    │
│                                   │ Tauri IPC                               │
│  ┌────────────────────────────────┴────────────────────────────────────┐    │
│  │                     Backend (Rust)                                  │    │
│  │                                                                      │    │
│  │  ┌──────────────────────────────────────────────────────────────┐   │    │
│  │  │                    flowsight-tauri                            │   │    │
│  │  │                    (Tauri Commands)                           │   │    │
│  │  └──────────────────────────────────────────────────────────────┘   │    │
│  │                                │                                     │    │
│  │  ┌─────────────────────────────┼─────────────────────────────────┐  │    │
│  │  │                             │                                  │  │    │
│  │  │  ┌──────────────┐  ┌───────┴──────┐  ┌──────────────┐        │  │    │
│  │  │  │ flowsight-   │  │ flowsight-   │  │ flowsight-   │        │  │    │
│  │  │  │ parser       │  │ analysis     │  │ llvm         │        │  │    │
│  │  │  │ (Tree-sitter)│  │ (分析引擎)   │  │ (IR 解析)    │        │  │    │
│  │  │  └──────────────┘  └──────────────┘  └──────────────┘        │  │    │
│  │  │                             │                                  │  │    │
│  │  │  ┌──────────────┐  ┌───────┴──────┐  ┌──────────────┐        │  │    │
│  │  │  │ flowsight-   │  │ flowsight-   │  │ flowsight-   │        │  │    │
│  │  │  │ knowledge    │  │ ai           │  │ index        │        │  │    │
│  │  │  │ (知识库)     │  │ (AI 推断)    │  │ (索引)       │        │  │    │
│  │  │  └──────────────┘  └──────────────┘  └──────────────┘        │  │    │
│  │  │                                                                │  │    │
│  │  └────────────────────────────────────────────────────────────────┘  │    │
│  │                                                                      │    │
│  └──────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                         数据层                                        │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                │   │
│  │  │ knowledge/   │  │ SQLite       │  │ Sled         │                │   │
│  │  │ (YAML 知识库)│  │ (项目索引)   │  │ (缓存)       │                │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘                │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Crate 结构

```
crates/
├── flowsight-core/          # 核心类型定义
│   ├── src/
│   │   ├── lib.rs
│   │   ├── types.rs         # ExecutionFlow, CallNode, etc.
│   │   ├── context.rs       # 执行上下文定义
│   │   └── confidence.rs    # 置信度定义
│   └── Cargo.toml
│
├── flowsight-parser/        # Tree-sitter 解析
│   ├── src/
│   │   ├── lib.rs
│   │   └── c_parser.rs
│   └── Cargo.toml
│
├── flowsight-llvm/          # LLVM IR 解析
│   ├── src/
│   │   ├── lib.rs
│   │   ├── ir_parser.rs     # IR 解析
│   │   ├── call_extractor.rs # 调用提取
│   │   └── ptr_analyzer.rs  # 函数指针分析
│   └── Cargo.toml
│
├── flowsight-knowledge/     # 知识库
│   ├── src/
│   │   ├── lib.rs
│   │   ├── loader.rs        # YAML 加载
│   │   ├── matcher.rs       # 模式匹配
│   │   └── chain.rs         # 调用链注入
│   └── Cargo.toml
│
├── flowsight-analysis/      # 分析引擎
│   ├── src/
│   │   ├── lib.rs
│   │   ├── call_graph.rs    # 调用图构建
│   │   ├── execution_flow.rs # 执行流构建
│   │   ├── funcptr.rs       # 函数指针分析
│   │   └── branch.rs        # 分支分析
│   └── Cargo.toml
│
├── flowsight-ai/            # AI 辅助 (Phase 4)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── inference.rs     # 推断接口
│   │   ├── ollama.rs        # Ollama 集成
│   │   └── confidence.rs    # 置信度评估
│   └── Cargo.toml
│
├── flowsight-index/         # 索引
│   └── ...
│
└── flowsight-query/         # 查询
    └── ...
```

### 7.3 执行流数据结构

```rust
// flowsight-core/src/types.rs

/// 执行流树
pub struct ExecutionFlow {
    /// 根节点 (入口函数)
    pub root: FlowNode,
    /// 分析的函数名
    pub entry_function: String,
    /// 分析时间
    pub analyzed_at: DateTime<Utc>,
}

/// 执行流节点
pub struct FlowNode {
    /// 唯一 ID
    pub id: NodeId,
    /// 函数名
    pub function_name: String,
    /// 文件位置
    pub location: Option<Location>,
    /// 执行上下文
    pub context: ExecutionContext,
    /// 节点类型
    pub kind: NodeKind,
    /// 子节点 (调用的函数)
    pub children: Vec<FlowNode>,
    /// 置信度
    pub confidence: Confidence,
    /// 注释/说明
    pub annotation: Option<String>,
}

/// 节点类型
pub enum NodeKind {
    /// 直接调用
    DirectCall,
    /// 函数指针调用
    IndirectCall {
        /// 可能的目标函数
        targets: Vec<FunctionTarget>,
    },
    /// 知识库注入的调用链
    KnowledgeChain {
        /// 来源知识库
        source: String,
    },
    /// 异步边界
    AsyncBoundary {
        /// 异步机制类型
        mechanism: AsyncMechanism,
    },
    /// 分支
    Branch {
        /// 分支条件
        condition: String,
        /// 分支类型
        branch_type: BranchType,
    },
    /// 分隔符 (时间流逝等)
    Separator {
        /// 说明文字
        text: String,
    },
}

/// 执行上下文
pub enum ExecutionContext {
    /// 进程上下文 (可睡眠)
    Process,
    /// 软中断上下文
    SoftIrq,
    /// 硬中断上下文 (不可睡眠)
    HardIrq,
    /// 用户空间
    UserSpace,
    /// 未知
    Unknown,
}

/// 异步机制类型
pub enum AsyncMechanism {
    WorkQueue,
    Timer,
    HRTimer,
    Irq,
    SoftIrq,
    Tasklet,
    KThread,
    Rcu,
}

/// 置信度
pub enum Confidence {
    /// 100% 确定
    Certain,
    /// 高置信度 (90%+)
    High(f32),
    /// 中等置信度 (70-90%)
    Medium(f32),
    /// 低置信度 (< 70%)
    Low(f32),
    /// 未知
    Unknown,
}
```

---

## 8. 风险与应对

### 8.1 技术风险

| 风险 | 可能性 | 影响 | 应对措施 |
|-----|-------|------|---------|
| LLVM 版本兼容 | 中 | 高 | 固定 LLVM 17+，提供编译指南 |
| 知识库维护成本 | 高 | 中 | AI 辅助生成，社区贡献 |
| AI 推断准确度 | 中 | 中 | 置信度标注，用户可修正 |
| 大型项目性能 | 中 | 中 | 增量分析，缓存优化 |
| 内核版本差异 | 中 | 中 | 标注版本兼容性 |

### 8.2 缓解策略

```
1. 渐进式交付
   ──────────────
   每个 Phase 都有可运行版本，及时获取反馈

2. 知识库优先
   ──────────────
   静态知识库优先，AI 作为补充

3. 用户可参与
   ──────────────
   用户可以补充/修正分析结果，贡献知识库

4. 本地优先
   ──────────────
   优先本地模型，云端可选

5. 标注不确定性
   ──────────────
   明确标注分析的置信度，不隐藏不确定性
```

---

## 9. 里程碑

| 里程碑 | 目标 | 交付物 | 状态 |
|-------|------|-------|------|
| **M1** | Phase 1 完成 | 核心分析引擎 | 🔄 进行中 |
| **M2** | Phase 2 完成 | 基本可用 UI | ⬜ 待开始 |
| **M3** | Phase 3 完成 | 知识库覆盖 90% | ⬜ 待开始 |
| **M4** | Phase 4 完成 | AI 辅助集成 | ⬜ 待开始 |
| **M5** | Phase 5 完成 | 完整功能 | ⬜ 待开始 |
| **M6** | Phase 6 完成 | 架构支持 | ⬜ 可选 |

---

## 附录

### 附录 A: 参考资源

- LLVM IR 文档: https://llvm.org/docs/LangRef.html
- inkwell (Rust LLVM 绑定): https://github.com/TheDan64/inkwell
- Linux 内核文档: https://www.kernel.org/doc/html/latest/
- Ollama: https://ollama.ai/

### 附录 B: 术语表

| 术语 | 定义 |
|-----|------|
| 执行流 | 函数执行过程中的调用序列，包括同步和异步调用 |
| LLVM IR | LLVM 中间表示，编译过程中的中间代码 |
| 知识库 | 预置的代码模式和调用链信息 |
| 函数指针 | 指向函数的指针，运行时确定调用目标 |
| 回调 | 通过函数指针调用的函数 |
| 异步机制 | workqueue/timer/irq 等延迟执行的机制 |
| 执行上下文 | 代码运行的环境 (进程/软中断/硬中断) |

### 附录 C: 与 V2 的主要变更

| 变更项 | V2 | V3 |
|-------|----|----|
| 目标定位 | "100% 精准" | "结构可视化 + 置信度标注" |
| 子系统覆盖 | 部分 | 完整计划 (30+ 驱动框架) |
| AI 角色 | KLEE 符号执行 | 静默辅助推断 |
| KLEE | 核心组件 | 移除 (可选) |
| 用户需求 | 隐含 | 明确定义 |
| 效果示例 | 简单 | 详细图示 |

---

## 变更日志

| 日期 | 版本 | 变更 |
|-----|------|-----|
| 2026-01-29 | v3.0 | 创建 V3 计划，重新定位目标，完整覆盖计划，AI 辅助方案 |
