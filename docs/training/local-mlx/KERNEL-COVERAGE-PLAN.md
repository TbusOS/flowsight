# Linux 内核全覆盖计划

> 目标：让 FlowSight 能够识别和分析 Linux 内核的所有主要子系统

## 设计原则

FlowSight 的分析能力来自 **三层架构**：

```
┌─────────────────────────────────────────────────────────────────┐
│ 第一层：Tree-sitter 解析器（精确）                               │
│ - 100% 准确的语法解析                                           │
│ - 函数定义、调用、结构体识别                                     │
│ - 不依赖 AI，不会出错                                           │
├─────────────────────────────────────────────────────────────────┤
│ 第二层：知识库模式匹配（可扩展）                                 │
│ - YAML 定义的模式规则                                           │
│ - 每个子系统的绑定/触发/取消模式                                │
│ - 执行上下文、调用链信息                                        │
├─────────────────────────────────────────────────────────────────┤
│ 第三层：AI 辅助（可选）                                         │
│ - 本地 7B 模型：快速生成结构化输出                              │
│ - 大模型 API：复杂问题的解释                                    │
│ - AI 只是辅助，不是核心                                         │
└─────────────────────────────────────────────────────────────────┘
```

**关键认识**：覆盖全部子系统是 **工程量问题**，不是技术问题。

---

## Linux 内核子系统清单

### 1. 核心子系统 (kernel/)

| 子系统 | 知识库文件 | 状态 | 优先级 |
|--------|-----------|------|--------|
| 进程调度 | `core/sched.yaml` | ❌ 缺失 | P0 |
| 中断处理 | `core/irq.yaml` | ✅ 已有 | - |
| 软中断 | `core/softirq.yaml` | ✅ 已有 | - |
| 工作队列 | `core/workqueue.yaml` | ✅ 已有 | - |
| 定时器 | `core/timer.yaml` | ✅ 已有 | - |
| 高精度定时器 | `core/hrtimer.yaml` | ❌ 缺失 | P1 |
| RCU | `core/rcu.yaml` | ✅ 已有 | - |
| 内核线程 | `core/kthread.yaml` | ✅ 已有 | - |
| 信号处理 | `core/signal.yaml` | ❌ 缺失 | P1 |
| 系统调用 | `core/syscall.yaml` | ❌ 缺失 | P1 |
| Futex | `core/futex.yaml` | ❌ 缺失 | P2 |
| Trace/Perf | `core/trace.yaml` | ❌ 缺失 | P2 |
| BPF | `core/bpf.yaml` | ❌ 缺失 | P2 |
| Cgroup | `core/cgroup.yaml` | ❌ 缺失 | P2 |
| Namespace | `core/namespace.yaml` | ❌ 缺失 | P2 |

### 2. 同步原语 (kernel/locking/)

| 机制 | 知识库文件 | 状态 | 优先级 |
|------|-----------|------|--------|
| Spinlock | `sync/spinlock.yaml` | ❌ 缺失 | P0 |
| Mutex | `sync/mutex.yaml` | ❌ 缺失 | P0 |
| Semaphore | `sync/semaphore.yaml` | ✅ 已有 | - |
| RW Lock | `sync/rwlock.yaml` | ✅ 已有 | - |
| Completion | `sync/completion.yaml` | ✅ 已有 | - |
| Wait Queue | `sync/waitqueue.yaml` | ❌ 缺失 | P0 |
| Per-CPU | `sync/percpu.yaml` | ❌ 缺失 | P1 |
| Atomic | `sync/atomic.yaml` | ❌ 缺失 | P1 |
| Seqlock | `sync/seqlock.yaml` | ❌ 缺失 | P2 |

### 3. 内存管理 (mm/)

| 子系统 | 知识库文件 | 状态 | 优先级 |
|--------|-----------|------|--------|
| 内存分配 | `mm/memory.yaml` | ✅ 已有 | - |
| SLAB/SLUB | `mm/slab.yaml` | ❌ 缺失 | P1 |
| 页面管理 | `mm/page.yaml` | ❌ 缺失 | P1 |
| VMA | `mm/vma.yaml` | ❌ 缺失 | P1 |
| 页表 | `mm/pagetable.yaml` | ❌ 缺失 | P2 |
| CMA | `mm/cma.yaml` | ❌ 缺失 | P2 |
| DMA 映射 | `mm/dma-mapping.yaml` | ❌ 缺失 | P1 |
| IOMMU | `mm/iommu.yaml` | ❌ 缺失 | P2 |

### 4. 文件系统 (fs/)

| 子系统 | 知识库文件 | 状态 | 优先级 |
|--------|-----------|------|--------|
| VFS | `fs/vfs.yaml` | ✅ 已有 | - |
| 文件操作 | `fs/file_ops.yaml` | ❌ 缺失 | P0 |
| 目录操作 | `fs/dir_ops.yaml` | ❌ 缺失 | P1 |
| 超级块 | `fs/superblock.yaml` | ❌ 缺失 | P1 |
| Inode | `fs/inode.yaml` | ❌ 缺失 | P1 |
| Dentry | `fs/dentry.yaml` | ❌ 缺失 | P1 |
| 页缓存 | `fs/pagecache.yaml` | ❌ 缺失 | P2 |
| 块 I/O | `fs/bio.yaml` | ❌ 缺失 | P1 |
| sysfs | `fs/sysfs.yaml` | ❌ 缺失 | P0 |
| procfs | `fs/procfs.yaml` | ❌ 缺失 | P1 |
| debugfs | `fs/debugfs.yaml` | ❌ 缺失 | P1 |

### 5. 网络 (net/)

| 子系统 | 知识库文件 | 状态 | 优先级 |
|--------|-----------|------|--------|
| 网络设备 | `net/netdev.yaml` | ✅ 已有 | - |
| Socket | `net/socket.yaml` | ❌ 缺失 | P0 |
| SKB | `net/skb.yaml` | ❌ 缺失 | P0 |
| TCP | `net/tcp.yaml` | ❌ 缺失 | P1 |
| IP | `net/ip.yaml` | ❌ 缺失 | P1 |
| Netfilter | `net/netfilter.yaml` | ❌ 缺失 | P2 |
| NAPI | `net/napi.yaml` | ❌ 缺失 | P1 |

### 6. 设备驱动 (drivers/)

| 框架 | 知识库文件 | 状态 | 优先级 |
|------|-----------|------|--------|
| Platform | `drivers/platform.yaml` | ✅ 已有 | - |
| USB | `drivers/usb.yaml` | ✅ 已有 | - |
| PCI | `drivers/pci.yaml` | ✅ 已有 | - |
| I2C | `drivers/i2c.yaml` | ✅ 已有 | - |
| SPI | `drivers/spi.yaml` | ✅ 已有 | - |
| GPIO | `drivers/gpio.yaml` | ✅ 已有 | - |
| Clock | `drivers/clk.yaml` | ✅ 已有 | - |
| Pinctrl | `drivers/pinctrl.yaml` | ✅ 已有 | - |
| Regulator | `drivers/regulator.yaml` | ✅ 已有 | - |
| DMA | `drivers/dma.yaml` | ✅ 已有 | - |
| Char 设备 | `drivers/char_dev.yaml` | ✅ 已有 | - |
| Block 设备 | `drivers/block.yaml` | ✅ 已有 | - |
| Input | `drivers/input.yaml` | ✅ 已有 | - |
| IIO | `drivers/iio.yaml` | ✅ 已有 | - |
| V4L2 | `drivers/v4l2.yaml` | ✅ 已有 | - |
| Thermal | `drivers/thermal.yaml` | ✅ 已有 | - |
| Watchdog | `drivers/watchdog.yaml` | ✅ 已有 | - |
| MFD | `drivers/mfd.yaml` | ✅ 已有 | - |
| NVMEM | `drivers/nvmem.yaml` | ✅ 已有 | - |
| PHY | `drivers/phy.yaml` | ✅ 已有 | - |
| Reset | `drivers/reset.yaml` | ✅ 已有 | - |
| PM Runtime | `drivers/pm_runtime.yaml` | ❌ 缺失 | P0 |
| Device Model | `drivers/device_model.yaml` | ❌ 缺失 | P0 |
| OF/DT | `drivers/of.yaml` | ❌ 缺失 | P0 |
| ACPI | `drivers/acpi.yaml` | ❌ 缺失 | P2 |
| TTY | `drivers/tty.yaml` | ❌ 缺失 | P1 |
| UART | `drivers/uart.yaml` | ❌ 缺失 | P1 |
| MTD | `drivers/mtd.yaml` | ❌ 缺失 | P1 |
| MMC | `drivers/mmc.yaml` | ❌ 缺失 | P1 |
| Crypto | `drivers/crypto.yaml` | ❌ 缺失 | P2 |
| LED | `drivers/led.yaml` | ❌ 缺失 | P2 |
| PWM | `drivers/pwm.yaml` | ❌ 缺失 | P2 |
| RTC | `drivers/rtc.yaml` | ❌ 缺失 | P2 |

### 7. 音频 (sound/)

| 框架 | 知识库文件 | 状态 | 优先级 |
|------|-----------|------|--------|
| ASoC | `sound/soc.yaml` | ✅ 已有 | - |
| ALSA Core | `sound/alsa.yaml` | ❌ 缺失 | P1 |

### 8. 架构相关 (arch/)

| 架构 | 知识库文件 | 状态 | 优先级 |
|------|-----------|------|--------|
| ARM32 | `arch/arm32/` | ✅ 部分 | - |
| ARM64 | `arch/arm64/` | ❌ 缺失 | P1 |
| x86 | `arch/x86/` | ❌ 缺失 | P2 |
| RISC-V | `arch/riscv/` | ❌ 缺失 | P2 |

---

## 统计

### 当前覆盖率

| 类别 | 已覆盖 | 总计 | 覆盖率 |
|------|--------|------|--------|
| 核心机制 | 7 | 15 | 47% |
| 同步原语 | 4 | 9 | 44% |
| 内存管理 | 1 | 8 | 12% |
| 文件系统 | 1 | 11 | 9% |
| 网络 | 1 | 7 | 14% |
| 驱动框架 | 21 | 32 | 66% |
| 音频 | 1 | 2 | 50% |
| 架构 | 1 | 4 | 25% |
| **总计** | **37** | **88** | **42%** |

### 工作量估算

| 优先级 | 数量 | 工作量/个 | 总时间 |
|--------|------|-----------|--------|
| P0 (必须) | 12 | 2-4 小时 | 24-48 小时 |
| P1 (重要) | 20 | 2-4 小时 | 40-80 小时 |
| P2 (可选) | 19 | 2-4 小时 | 38-76 小时 |
| **总计** | **51** | - | **~100-200 小时** |

---

## 知识库 YAML 格式

每个知识库文件遵循统一格式：

```yaml
# 机制名称
mechanism_name:
  description: "简要描述"
  header: "linux/xxx.h"
  icon: "🔧"
  
  context:
    type: "process | softirq | hardirq | atomic"
    can_sleep: true | false
  
  # 绑定模式 - 何时建立关联
  bind_patterns:
    - pattern: 'MACRO\s*\(\s*(?P<var>[\w\.\->]+)\s*,\s*(?P<handler>\w+)\s*\)'
      handler_capture: "handler"
      variable_capture: "var"
  
  # 触发模式 - 何时执行
  trigger_patterns:
    - pattern: 'trigger_func\s*\(\s*&?(?P<var>[\w\.\->]+)\s*\)'
      variable_capture: "var"
  
  # 取消/清理模式
  cancel_patterns:
    - pattern: 'cancel_func\s*\(\s*&?(?P<var>[\w\.\->]+)\s*\)'
  
  # Handler 签名
  handler_signature:
    return_type: "void | int | ..."
    parameters:
      - name: "param1"
        type: "type1"
  
  # 生命周期
  lifecycle:
    init_required: true
    cleanup_required: true
  
  # 注意事项
  notes:
    - "重要提示1"
    - "重要提示2"
```

---

## 实施计划

### Phase 1: 核心机制完善 (P0)

目标：让常见驱动分析工作正常

```bash
# 新增知识库
knowledge/platforms/linux-kernel/
├── core/
│   ├── sched.yaml          # NEW: 调度器
│   └── hrtimer.yaml        # NEW: 高精度定时器
├── sync/
│   ├── spinlock.yaml       # NEW
│   ├── mutex.yaml          # NEW
│   └── waitqueue.yaml      # NEW
├── drivers/
│   ├── device_model.yaml   # NEW: 设备模型
│   ├── of.yaml             # NEW: 设备树
│   └── pm_runtime.yaml     # NEW: 运行时电源管理
└── fs/
    ├── file_ops.yaml       # NEW
    └── sysfs.yaml          # NEW
```

### Phase 2: 常用框架 (P1)

目标：覆盖 80% 的内核代码

### Phase 3: 完整覆盖 (P2)

目标：覆盖 95%+ 的内核代码

---

## 本地 7B 模型的角色

在这个架构中，7B 模型的作用是：

| 任务 | 是否使用 7B 模型 | 说明 |
|------|-----------------|------|
| 模式匹配 | ❌ | 知识库 + 正则，100% 准确 |
| 执行上下文判断 | ❌ | 知识库定义，100% 准确 |
| 调用链追踪 | ❌ | Tree-sitter + 符号索引 |
| 结构化输出生成 | ✅ | 将分析结果转为自然语言或 JSON |
| 代码解释 | ✅ | 辅助解释分析结果 |
| 模式建议 | ✅ | 帮助识别新的未知模式 |

**核心分析不依赖 AI**，AI 只是锦上添花。

---

## 结论

1. **可以做到**：覆盖全部 Linux 子系统是可行的
2. **工程量**：约 100-200 小时的知识库编写工作
3. **渐进式**：按优先级分阶段完成
4. **准确性高**：基于模式匹配，不依赖 AI 理解
5. **可维护**：YAML 格式易于扩展和修改

---

*最后更新: 2026-02-01*
