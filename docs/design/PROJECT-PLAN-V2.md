# FlowSight 项目计划 v2.1：可靠精准的函数执行流分析

> 版本: v2.1
> 更新日期: 2026-01-23
> 状态: 规划中

**核心目标**: 精准识别函数真正的执行过程，做到 100% 可信

**技术路线**:
- LLVM IR 分析（编译产物）→ 准确类型和指针信息
- 知识库 → 补全内核 API 调用链（100% 准确）
- KLEE 符号执行（按需）→ 精确分支条件

---

## 目录

1. [项目愿景](#1-项目愿景)
2. [技术方案总览](#2-技术方案总览)
3. [知识库扩展计划](#3-知识库扩展计划)
4. [KLEE 符号执行集成](#4-klee-符号执行集成)
5. [实施计划](#5-实施计划)
6. [文件变更清单](#6-文件变更清单)
7. [风险与应对](#7-风险与应对)
8. [里程碑](#8-里程碑)

---

## 1. 项目愿景

### 1.1 核心问题

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     精准识别函数真正的执行过程                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  用户需求：                                                                  │
│  • 点击一个函数 → 看到完整、精准的执行流程                                   │
│  • 知道谁调用了这个函数 + 这个函数调用了什么                                 │
│  • 对于回调函数：知道什么时候被触发                                         │
│  • 对于分支：知道什么条件下走哪条路径                                        │
│                                                                              │
│  技术挑战：                                                                  │
│  • 函数指针分析（静态分析无法 100% 精确）                                   │
│  • 外部库调用链（需要知识库补全）                                           │
│  • 分支条件（需要符号执行）                                                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 解决方案

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FlowSight 解决方案                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  愿景：不运行代码，就能精准知道函数执行过程                                  │
│                                                                              │
│  技术手段：                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 1. LLVM IR 分析 (编译产物)                                           │    │
│  │    • 类型信息完整准确                                                │    │
│  │    • 函数指针指向明确                                                │    │
│  │    • 无预处理问题                                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                              │                                               │
│                              ▼                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 2. 知识库 (100% 准确)                                               │    │
│  │    • 预置 Linux 内核 API 调用链                                     │    │
│  │    • 回调注册模式（probe/disconnect/handler）                       │    │
│  │    • 异步机制（workqueue/timer/irq）                                │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                              │                                               │
│                              ▼                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 3. KLEE 按需分析 (100% 路径覆盖)                                    │    │
│  │    • 用户选定函数后才分析（避免路径爆炸）                            │    │
│  │    • 符号执行 → 精确分支条件                                         │    │
│  │    • 生成测试用例                                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.3 差异化定位

| 维度 | 传统 IDE | ftrace/LTTng | FlowSight |
|-----|----------|--------------|-----------|
| 运行方式 | 不需要运行 | 需要真机运行 | 不需要运行 |
| LLVM IR 分析 | ❌ | ❌ | ✅ 准确类型/指针 |
| 知识库调用链 | ❌ | ❌ | ✅ 100% 准确 |
| 符号执行 | ❌ | ❌ | ✅ 按需 KLEE |
| 执行流展示 | ❌ | ✅ | ✅ ftrace 风格 |
| 离线使用 | ✅ | ❌ | ✅ |
| 本地部署 | ✅ | ❌ | ✅ |

---

## 2. 技术方案总览

### 2.1 系统架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FlowSight 分析架构                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                      Frontend (React + TypeScript)                  │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐           │    │
│  │  │ 代码编辑器 │  │ 执行流面板 │  │ 调用图面板 │  │ 设置面板 │           │    │
│  │  │ Monaco   │  │ ftrace   │  │ React    │  │ 配置     │           │    │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘           │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│  ┌─────────────────────���───────────────────────────────────────────────┐    │
│  │                     Backend (Rust + Tauri RPC)                      │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐           │    │
│  │  │ Parser   │  │ Analyzer │  │ LLVM     │  │ KLEE     │           │    │
│  │  │ Tree-    │  │ Static   │  │ IR       │  │ Symbolic │           │    │
│  │  │ sitter   │  │ Analysis │  │ Parser   │  │ Executor │           │    │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘           │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         知识库层                                     │    │
│  │  knowledge/platforms/linux-kernel/                                   │    │
│  │  • core/ (workqueue, timer, irq, kthread, rcu, memory, vfs)        │    │
│  │  • drivers/ (usb, char_dev, platform, pci, i2c, spi, netdev...)    │    │
│  │  • sync/ (completion, mutex, semaphore)                             │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 数据流图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              数据流图                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  1. 编译阶段 (用户执行)                                                      │
│  ────────────────────────                                                    │
│  kernel 源码 ──► clang -emit-llvm ──► LLVM IR (.bc 文件)                   │
│                                                                              │
│  2. 静态分析阶段 (自动化)                                                    │
│  ─────────────────────────                                                    │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │ LLVM IR      │ →  │ 静态调用     │ →  │ 知识库       │                  │
│  │ 解析器       │    │ 分析         │    │ 调用链注入   │                  │
│  │              │    │              │    │              │                  │
│  │ 提取：       │    │ 提取：       │    │ 补全：       │                  │
│  │ • 函数定义   │    │ • 调用关系   │    │ • 内核API链  │                  │
│  │ • 函数调用   │    │ • 指针分析   │    │ • 回调触发   │                  │
│  │ • 类型信息   │    │ • 分支结构   │    │ • 执行上下文 │                  │
│  └──────────────┘    └──────────────┘    └──────────────┘                  │
│                                                                              │
│  3. 符号执行阶段 (按需)                                                       │
│  ─────────────────────────                                                   │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │ 用户选择     │ →  │ KLEE         │ →  │ 分支条件     │                  │
│  │ 函数         │    │ 符号执行     │    │ 提取         │                  │
│  │              │    │              │    │              │                  │
│  │ 避免：       │    │ 分析：       │    │ 输出：       │                  │
│  │ 路径爆炸     │    │ • 探索路径   │    │ • 约束条件   │                  │
│  │              │    │ • 生成约束   │    │ • 测试用例   │                  │
│  └──────────────┘    └──────────────┘    └──────────────┘                  │
│                                                                              │
│  4. 执行流构建                                                               │
│  ─────────────────                                                           │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │ 静态分析结果 │ +  │ 知识库补全   │ +  │ KLEE 条件    │ → 执行流树       │
│  └──────────────┘    └──────────────┘    └──────────────┘                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 技术栈

| 层级 | 技术 | 用途 | 版本要求 |
|-----|------|------|---------|
| 前端框架 | React 18 + TypeScript | UI 开发 | latest |
| 代码编辑器 | Monaco Editor | 代码展示 | latest |
| 图形可视化 | @xyflow/react | 调用图、执行流 | latest |
| 状态管理 | Zustand | 前端状态 | latest |
| 桌面框架 | Tauri 2.0 | 跨平台桌面应用 | 2.0+ |
| 后端语言 | Rust | 分析引擎 | 1.75+ |
| 源码解析 | Tree-sitter | C 代码 AST | latest |
| **IR 解析** | **inkwell** | **LLVM IR 绑定** | **17+** |
| **符号执行** | **KLEE** | **路径分析** | **latest** |
| 数据存储 | SQLite + Sled | 项目索引、缓存 | latest |
| 构建系统 | Cargo + npm | Rust/JS 构建 | latest |

---

## 3. 知识库扩展计划

### 3.1 核心机制 (必须覆盖)

| 机制 | 知识库文件 | 优先级 | 状态 |
|------|-----------|--------|------|
| WorkQueue | core/workqueue.yaml | P0 | ✅ 已完成 |
| Timer (timer_list) | core/timer.yaml | P0 | ✅ 已完成 |
| IRQ/异常处理 | core/irq.yaml | P0 | ✅ 已完成 |
| Kernel Thread | core/kthread.yaml | P0 | ✅ 已完成 |
| RCU | core/rcu.yaml | P0 | ✅ 已完成 |
| Memory (slab/kmalloc) | core/memory.yaml | P1 | ❌ 待创建 |
| VFS | core/vfs.yaml | P1 | ❌ 待创建 |
| SoftIRQ | core/softirq.yaml | P1 | ❌ 待创建 |

### 3.2 驱动框架 (必须覆盖)

| 驱动类型 | 知识库文件 | 优先级 | 状态 |
|---------|-----------|--------|------|
| USB | drivers/usb.yaml | P0 | ✅ 已完成 |
| 字符设备 | drivers/char_dev.yaml | P0 | ✅ 已完成 |
| Platform | drivers/platform.yaml | P0 | ✅ 已完成 |
| PCI/PCIe | drivers/pci.yaml | P0 | ✅ 已完成 |
| I2C | drivers/i2c.yaml | P0 | ✅ 已完成 |
| SPI | drivers/spi.yaml | P0 | ✅ 已完成 |
| 网络设备 (netdev) | drivers/netdev.yaml | P1 | ❌ 待创建 |
| 块设备 (block) | drivers/block.yaml | P1 | ❌ 待创建 |
| GPIO | drivers/gpio.yaml | P1 | ❌ 待创建 |
| Input | drivers/input.yaml | P1 | ❌ 待创建 |
| Clock (clk) | drivers/clk.yaml | P1 | ❌ 待创建 |
| Regulator | drivers/regulator.yaml | P1 | ❌ 待创建 |
| DMA | drivers/dma.yaml | P1 | ❌ 待创建 |
| Thermal | drivers/thermal.yaml | P1 | ❌ 待创建 |
| Watchdog | drivers/watchdog.yaml | P1 | ❌ 待创建 |
| IIO | drivers/iio.yaml | P1 | ❌ 待创建 |
| RTC | drivers/rtc.yaml | P1 | ❌ 待创建 |
| PinCtrl | drivers/pinctrl.yaml | P1 | ❌ 待创建 |
| V4L2 | drivers/v4l2.yaml | P1 | ❌ 待创建 |
| Sound/ALSA | drivers/sound.yaml | P2 | ❌ 待创建 |
| MFD | drivers/mfd.yaml | P2 | ❌ 待创建 |
| PHY | drivers/phy.yaml | P2 | ❌ 待创建 |
| Reset | drivers/reset.yaml | P2 | ❌ 待创建 |

### 3.3 同步原语

| 原语 | 知识库文件 | 优先级 | 状态 |
|------|-----------|--------|------|
| Completion | sync/completion.yaml | P1 | ✅ 已完成 |
| Mutex/Spinlock | sync/locking.yaml | P1 | ❌ 待创建 |
| Semaphore | sync/semaphore.yaml | P2 | ❌ 待创建 |
| RWLock | sync/rwlock.yaml | P2 | ❌ 待创建 |

### 3.4 知识库结构示例

```yaml
# knowledge/platforms/linux-kernel/core/workqueue.yaml

work_struct:
  description: "Work Queue - 延迟到进程上下文执行的工作单元"
  header: "linux/workqueue.h"
  icon: "⚙️"

  context:
    type: "process"
    can_sleep: true
    can_schedule: true
    preemptible: true

  bind_patterns:
    - pattern: 'INIT_WORK\s*\(\s*&?(?P<var>[\w\.\->]+)\s*,\s*(?P<handler>\w+)\s*\)'
      handler_capture: "handler"
      variable_capture: "var"
      scope: "struct_field"

  trigger_patterns:
    - pattern: 'schedule_work\s*\(\s*&?(?P<var>[\w\.\->]+)\s*\)'
      variable_capture: "var"
      workqueue: "system_wq"

  kernel_call_chain:
    trigger: "schedule_work() 被调用"
    chain:
      - function: "try_to_wake_up"
        description: "唤醒工作线程"
      - function: "worker_thread"
        file: "kernel/workqueue.c"
        description: "工作线程主函数"
      - function: "process_one_work"
        description: "处理单个工作"
      - function: "work->func"
        is_user_entry: true
        description: "用户 work handler"

  notes:
    - "使用 container_of 获取包含此 work_struct 的结构体"
```

---

## 4. KLEE 符号执行集成

### 4.1 工作流程

```
用户选择函数
     │
     ▼
┌─────────────┐
│ 提取该函数  │  ← 从 LLVM IR 提取
│ 的 IR 代码  │    (包含类型、调用关系)
└─────────────┘
     │
     ▼
┌─────────────┐
│ 生成 KLEE   │  ← 添加入口点
│ 测试代码    │    (wrap 内核 API 为 stub)
└─────────────┘
     │
     ▼
┌─────────────┐
│ 运行 KLEE   │  ← 符号执行
│              │    (30秒超时, 1000路径限制)
└─────────────┘
     │
     ▼
┌─────────────┐
│ 解析输出    │  ← 路径、条件、测试用例
│              │
└─────────────┘
     │
     ▼
返回分析结果
(用户看到精确的分支条件)
```

### 4.2 关键设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| **触发方式** | 用户选定后分析 | 避免整个内核路径爆炸 |
| **时间限制** | 30 秒/函数 | 平衡分析深度和用户体验 |
| **路径限制** | 1000 条/函数 | 防止特殊情况路径爆炸 |
| **依赖处理** | 内核 API stub | KLEE 无法执行真实内核调用 |

### 4.3 KLEE stub 示例

```c
// klee_stubs/kernel_api.c

// KLEE 无法执行真实的内核 API，需要 stub
// 这些 stub 返回符号值，让 KLEE 探索路径

// 内存分配
void *kzalloc(size_t size, gfp_t flags) {
    void *ptr = malloc(size);
    // 让 KLEE 符号化返回值
    klee_make_symbolic(ptr, size, "kzalloc_result");
    return ptr;
}

// 返回值符号化
int copy_to_user(void *to, const void *from, unsigned long n) {
    int result;
    klee_make_symbolic(&result, sizeof(result), "copy_to_user_result");
    // 约束：正常情况返回 n，错误返回负数
    klee_assume(result >= 0 || result < 0);
    return result;
}
```

### 4.4 输出格式

```json
{
  "function": "my_driver_handler",
  "paths": [
    {
      "id": 0,
      "constraints": ["x > 0", "ret == 0"],
      "branch_conditions": {
        "if (x > 0)": "设备就绪",
        "if (ret == 0)": "操作成功"
      },
      "test_input": {"x": 42, "ret": 0}
    },
    {
      "id": 1,
      "constraints": ["x <= 0", "ret == -ENODEV"],
      "branch_conditions": {
        "if (x > 0)": "设备就绪",
        "else": "设备未就绪"
      },
      "test_input": {"x": 0, "ret": -19}
    }
  ],
  "coverage": {
    "total_branches": 10,
    "covered_branches": 10,
    "coverage_percent": 100.0
  }
}
```

---

## 5. 实施计划

### 5.1 整体时间线

```
v0.1          v0.2           v1.0           v1.5
 │             │              │              │
 ▼             ▼              ▼              ▼
┌─────┐   ┌─────────┐   ┌───────────┐  ┌──────────┐
│LLVM │   │ 知识库  │   │ KLEE      │  │ 完善优化 │
│IR   │   │ 扩展    │   │ 按需分析  │  │          │
│解析 │   │         │   │           │  │          │
└─────┘   └─────────┘   └───────────┘  └──────────┘
 2周        2-4周         2-3周         1-2周
```

### 5.2 Phase 详细计划

#### Phase 1: LLVM IR 解析 (2周)

**目标**: 建立 LLVM IR 解析能力

**任务**:
```
Week 1: 技术选型与 POC
├── 评估 LLVM Rust 绑定 (llvm-sys vs inkwell)
├── inkwell 优：更安全、官方支持
├── 搭建 inkwell 环境
└── 验证 IR 解析可行性

Week 2: IR 解析器实现
├── 提取函数定义 (define void @func)
├── 提取函数调用 (call void @callee)
├── 提取类型信息 (i32, ptr, struct)
├── 提取基本块和分支
└── 编写单元测试
```

**输出**:
- `crates/flowsight-llvm/Cargo.toml`
- `crates/flowsight-llvm/src/lib.rs`
- `crates/flowsight-llvm/src/ir_parser.rs`

**验收标准**:
- ✅ 能够解析 Linux kernel 的 .bc 文件
- ✅ 提取函数定义 100% 准确
- ✅ 提取函数调用 100% 准确
- ✅ 提取类型信息 100% 准确

---

#### Phase 2: 知识库扩展 (2-4周)

**目标**: 覆盖 Linux 内核核心机制

**任务**:
```
Week 1-2: 核心机制 (P0)
├── memory.yaml (slab/kmalloc/vmalloc)
├── vfs.yaml (VFS 操作)
└── softirq.yaml (软中断)

Week 3: 驱动框架补全 (P0)
├── netdev.yaml (网络设备)
├── block.yaml (块设备)
└── gpio.yaml (GPIO)

Week 4: 驱动框架扩展 (P1)
├── input.yaml, clk.yaml, regulator.yaml
├── dma.yaml, thermal.yaml, watchdog.yaml
├── iio.yaml, rtc.yaml, pinctrl.yaml
└── v4l2.yaml, sound.yaml
```

**输出**:
- 20+ 个知识库文件
- 每个知识库包含完整调用链

**验收标准**:
- ✅ 核心机制覆盖率 > 90%
- ✅ 调用链注入 100% 准确
- ✅ 执行上下文标注 100% 准确

---

#### Phase 3: KLEE 集成 (2-3周)

**目标**: 实现按需符号执行

**任务**:
```
Week 1: KLEE 环境搭建
├── 搭建 KLEE 环境 (Docker 镜像)
├── 验证 KLEE 基本功能
└── 创建内核 API stub 库

Week 2: 测试代码生成器
├── 从 LLVM IR 提取目标函数
├── 生成 KLEE 兼容的测试代码
├── 生成内核 API stub 调用
└── 编写集成测试

Week 3: 输出解析器
├── 解析 KLEE 输出格式
├── 提取分支约束条件
├── 生成测试用例
└── 集成到分析流程
```

**输出**:
- `crates/flowsight-klee/Cargo.toml`
- `crates/flowsight-klee/src/lib.rs`
- `crates/flowsight-klee/src/test_gen.rs`
- `crates/flowsight-klee/src/output.rs`

**验收标准**:
- ✅ 用户选定函数后 KLEE 能正确分析
- ✅ 单函数分析时间 < 30 秒
- ✅ 分支覆盖率 > 95% (选定函数)
- ✅ 超时/路径限制正常工作

---

#### Phase 4: 完善与优化 (1-2周)

**目标**: 提升准确度和用户体验

**任务**:
```
Week 1: 测试验证
├── 真实内核代码测试 (e1000, usb-storage)
├── 端到端流程测试
└── 性能测试 (大函数分析)

Week 2: 优化与文档
├── 增量分析支持
├── 缓存优化
├── 错误处理完善
└── 文档完善
```

**输出**:
- v1.0 Beta 版本
- 完整测试用例
- 用户文档

---

## 6. 文件变更清单

### 6.1 新建文件

| 文件路径 | 说明 | Phase |
|---------|------|-------|
| `crates/flowsight-llvm/Cargo.toml` | LLVM 解析 crate | Phase 1 |
| `crates/flowsight-llvm/src/lib.rs` | 主入口 | Phase 1 |
| `crates/flowsight-llvm/src/ir_parser.rs` | IR 解析器 | Phase 1 |
| `crates/flowsight-llvm/src/types.rs` | 类型信息 | Phase 1 |
| `crates/flowsight-klee/Cargo.toml` | KLEE 集成 crate | Phase 3 |
| `crates/flowsight-klee/src/lib.rs` | 主入口 | Phase 3 |
| `crates/flowsight-klee/src/test_gen.rs` | 测试代码生成 | Phase 3 |
| `crates/flowsight-klee/src/output.rs` | 输出解析 | Phase 3 |
| `crates/flowsight-klee/src/stubs.rs` | 内核 API stub | Phase 3 |
| `knowledge/platforms/linux-kernel/core/memory.yaml` | 内存机制 | Phase 2 |
| `knowledge/platforms/linux-kernel/core/vfs.yaml` | VFS 机制 | Phase 2 |
| `knowledge/platforms/linux-kernel/core/softirq.yaml` | 软中断 | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/netdev.yaml` | 网络设备 | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/block.yaml` | 块设备 | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/gpio.yaml` | GPIO | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/input.yaml` | Input | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/clk.yaml` | Clock | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/regulator.yaml` | Regulator | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/dma.yaml` | DMA | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/thermal.yaml` | Thermal | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/watchdog.yaml` | Watchdog | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/iio.yaml` | IIO | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/rtc.yaml` | RTC | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/pinctrl.yaml` | PinCtrl | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/v4l2.yaml` | V4L2 | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/sound.yaml` | Sound | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/mfd.yaml` | MFD | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/phy.yaml` | PHY | Phase 2 |
| `knowledge/platforms/linux-kernel/drivers/reset.yaml` | Reset | Phase 2 |
| `knowledge/platforms/linux-kernel/sync/locking.yaml` | Mutex/Spinlock | Phase 2 |
| `knowledge/platforms/linux-kernel/sync/semaphore.yaml` | Semaphore | Phase 2 |
| `knowledge/platforms/linux-kernel/sync/rwlock.yaml` | RWLock | Phase 2 |

### 6.2 修改文件

| 文件路径 | 修改内容 | Phase |
|---------|---------|-------|
| `crates/flowsight-analysis/src/lib.rs` | 集成 LLVM 解析 | Phase 1 |
| `crates/flowsight-analysis/src/call_graph.rs` | 增强调用图构建 | Phase 1 |
| `app/src/components/FlowView/FlowTextView.tsx` | 增强执行流展示 | Phase 3 |
| `app/src/components/FlowView/FlowGraphView.tsx` | 增强图形展示 | Phase 3 |

---

## 7. 风险与应对

### 7.1 技术风险

| 风险 | 可能性 | 影响 | 应对措施 |
|-----|-------|------|---------|
| LLVM IR 解析复杂 | 中 | 高 | 优先使用 inkwell 库 |
| KLEE 环境搭建困难 | 中 | 高 | 提供 Docker 镜像 |
| 内核 API stub 不完整 | 中 | 中 | 先覆盖常见 API |
| 路径爆炸 | 中 | 中 | 30秒超时 + 1000路径限制 |
| 知识库覆盖不足 | 高 | 中 | 优先覆盖核心机制 |

### 7.2 缓解策略

```
风险缓解总体策略：
1. MVP 优先：先实现 LLVM IR 解析，再扩展知识库
2. 技术验证：每个组件先 POC，再大规模开发
3. 渐进交付：每个 Phase 都有可运行的版本
4. 回退计划：如果 KLEE 不可用，使用简化分支分析
```

---

## 8. 里程碑

| 里程碑 | 日期 | 交付物 | 状态 |
|-------|------|-------|------|
| M0: 项目启动 | 2026-01-23 | 项目计划文档 v2.1 | 进行中 |
| M1: Phase 1 完成 | 2026-02-06 | LLVM IR 解析器 | 待开始 |
| M2: Phase 2 完成 | 2026-03-06 | 知识库扩展 | 待开始 |
| M3: Phase 3 完成 | 2026-03-27 | KLEE 按需分析 | 待开始 |
| M4: Phase 4 完成 | 2026-04-10 | v1.0 Beta | 待开始 |

---

## 附录 A: 参考资源

- LLVM IR 文档: https://llvm.org/docs/LangRef.html
- KLEE 官方文档: https://klee.github.io/
- Linux 内核文档: https://www.kernel.org/doc/html/latest/
- inkwell (Rust LLVM 绑定): https://github.com/TheDan64/inkwell
- KLEE Docker: https://github.com/klee/klee/tree/master/docker

## 附录 B: 术语表

| 术语 | 定义 |
|-----|------|
| LLVM IR | LLVM 中间表示，编译过程中的中间代码 |
| 符号执行 | 使用符号值而非具体值执行程序，探索所有可能路径 |
| KLEE | LLVM 上的符号执行引擎 |
| 路径爆炸 | 程序路径数量指数级增长，无法全部探索 |
| 知识库 | 预置的代码模式和调用链信息 |
| stub | 模拟函数，用于替代无法在分析环境执行的代码 |

---

## 变更日志

| 日期 | 版本 | 变更 |
|-----|------|-----|
| 2026-01-23 | v2.1 | 重写技术方案：LLVM IR + 知识库 + KLEE 按需 |
| 2025-01-21 | v2.0 | 初始规划（已过期） |
