# FlowSight 知识库说明

FlowSight 内置了完整的 Linux 内核知识库，用于理解 API 语义和执行上下文。

## 知识库概览

### 统计数据

| 指标 | 数量 |
|------|------|
| **YAML 文件** | 116 个 |
| **覆盖子系统** | 12 个 |
| **API 模式** | 6,200+ 个 |
| **内核版本** | 4.0+ (优化 5.x/6.x) |

### 目录结构

```
knowledge/platforms/linux-kernel/
├── arch/           # 架构相关
│   ├── arm32/      # ARM32 (imx, irq, pm)
│   ├── arm64/      # ARM64
│   ├── riscv/      # RISC-V
│   └── x86/        # x86
├── block/          # 块设备层
│   ├── bio.yaml    # Block I/O
│   ├── blk_mq.yaml # Multi-queue
│   └── elevator.yaml
├── core/           # 核心子系统
│   ├── memory.yaml # 内存管理
│   ├── sched.yaml  # 调度器
│   ├── irq.yaml    # 中断
│   ├── softirq.yaml
│   ├── timer.yaml  # 定时器
│   ├── workqueue.yaml # 工作队列
│   ├── rcu.yaml    # RCU
│   └── ...
├── drivers/        # 驱动框架
│   ├── platform.yaml
│   ├── usb.yaml
│   ├── i2c.yaml
│   ├── spi.yaml
│   ├── pci.yaml
│   ├── gpio.yaml
│   ├── clk.yaml
│   ├── regmap.yaml
│   └── ...
├── fs/             # 文件系统
│   ├── vfs_ops.yaml
│   ├── procfs.yaml
│   ├── sysfs.yaml
│   └── debugfs.yaml
├── lib/            # 库函数
│   ├── list.yaml
│   ├── rbtree.yaml
│   ├── xarray.yaml
│   └── idr.yaml
├── mm/             # 内存管理
│   ├── page_alloc.yaml
│   ├── slub.yaml
│   ├── vmalloc.yaml
│   └── dma.yaml
├── net/            # 网络子系统
│   ├── netdev.yaml
│   ├── socket.yaml
│   ├── tcp.yaml
│   └── udp.yaml
└── sync/           # 同步原语
    ├── locking.yaml
    ├── rwlock.yaml
    ├── semaphore.yaml
    └── completion.yaml
```

## 子系统覆盖详情

### 核心子系统 (core/)

| 文件 | 覆盖 API | 说明 |
|------|----------|------|
| `workqueue.yaml` | INIT_WORK, schedule_work, ... | 工作队列完整支持 |
| `timer.yaml` | timer_setup, mod_timer, hrtimer_* | 标准/高精度定时器 |
| `irq.yaml` | request_irq, request_threaded_irq | 中断注册和处理 |
| `softirq.yaml` | tasklet_*, softirq_action | 软中断和 Tasklet |
| `sched.yaml` | schedule, wait_event_*, wake_up_* | 调度和等待 |
| `memory.yaml` | kmalloc, kfree, __get_free_pages | 内存分配 |
| `rcu.yaml` | rcu_read_lock, synchronize_rcu | RCU 同步 |
| `kthread.yaml` | kthread_create, kthread_run | 内核线程 |

### 驱动框架 (drivers/)

| 文件 | 覆盖 API | 说明 |
|------|----------|------|
| `platform.yaml` | platform_driver_register, ... | 平台驱动 |
| `usb.yaml` | usb_register, usb_submit_urb | USB 驱动 |
| `i2c.yaml` | i2c_add_driver, i2c_transfer | I2C 总线 |
| `spi.yaml` | spi_register_driver, spi_sync | SPI 总线 |
| `pci.yaml` | pci_register_driver, pci_iomap | PCI 总线 |
| `gpio.yaml` | gpiod_get, gpiod_set_value | GPIO 子系统 |
| `clk.yaml` | clk_get, clk_prepare_enable | 时钟框架 |
| `regmap.yaml` | regmap_read, regmap_write | 寄存器映射 |
| `dma.yaml` | dma_alloc_coherent, dmaengine_* | DMA 操作 |

### 文件系统 (fs/)

| 文件 | 覆盖 API | 说明 |
|------|----------|------|
| `vfs_ops.yaml` | file_operations, inode_operations | VFS 操作表 |
| `procfs.yaml` | proc_create, proc_mkdir | procfs 接口 |
| `sysfs.yaml` | sysfs_create_file, device_attribute | sysfs 属性 |
| `debugfs.yaml` | debugfs_create_file, debugfs_remove | debugfs 调试 |

### 同步原语 (sync/)

| 文件 | 覆盖 API | 说明 |
|------|----------|------|
| `locking.yaml` | spin_lock, mutex_lock, ... | 自旋锁/互斥锁 |
| `rwlock.yaml` | read_lock, write_lock | 读写锁 |
| `semaphore.yaml` | down, up, down_interruptible | 信号量 |
| `completion.yaml` | init_completion, wait_for_completion | 完成量 |

## 知识库格式

每个 YAML 文件包含以下结构：

```yaml
# 异步机制定义
work_struct:
  description: "Work Queue - 延迟到进程上下文执行"
  header: "linux/workqueue.h"
  kernel_version: "2.6+"
  
  context:
    type: "process"
    can_sleep: true
    can_schedule: true
  
  bind_patterns:
    - pattern: 'INIT_WORK\s*\(\s*&?(?P<var>[\w\.\->]+)\s*,\s*(?P<handler>\w+)\s*\)'
      handler_capture: "handler"
      variable_capture: "var"
      description: "初始化 work_struct"
  
  trigger_patterns:
    - pattern: 'schedule_work\s*\(\s*&?(?P<var>[\w\.\->]+)\s*\)'
      variable_capture: "var"
      description: "调度工作执行"
  
  call_chains:
    work_execution:
      description: "工作项执行调用链"
      chain:
        - function: "schedule_work"
          context: "any"
        - function: "__queue_work"
          file: "kernel/workqueue.c"
        - function: "worker_thread"
          context: "process"
        - function: "process_one_work"
        - function: "work->func(work)"
          is_user_entry: true
```

## 知识库信息面板

在 FlowSight 中选中函数时，右侧面板显示知识库信息：

```
┌─────────────────────────────────────┐
│ 📚 知识库信息                        │
├─────────────────────────────────────┤
│ 函数: schedule_work                 │
│ 头文件: linux/workqueue.h           │
│ 内核版本: 2.6+                      │
├─────────────────────────────────────┤
│ 📍 执行上下文                        │
│ • 调用上下文: 任意（进程/中断）      │
│ • Handler 上下文: 进程              │
│ • 可睡眠: Handler 可以睡眠          │
├─────────────────────────────────────┤
│ 📖 说明                              │
│ 将工作项加入 system_wq 队列，        │
│ 由 worker 线程在进程上下文中执行。   │
├─────────────────────────────────────┤
│ ⚠️ 注意事项                          │
│ • 同一 work 不能同时在多个队列中     │
│ • 返回 false 表示已在队列中          │
│ • 使用 cancel_work_sync 同步取消    │
├─────────────────────────────────────┤
│ 🔗 相关 API                          │
│ • INIT_WORK() - 初始化              │
│ • cancel_work_sync() - 同步取消     │
│ • flush_work() - 等待完成           │
└─────────────────────────────────────┘
```

## 扩展知识库

### 添加自定义知识库

1. 在 `knowledge/platforms/` 下创建目录
2. 添加 YAML 文件，遵循上述格式
3. 重启 FlowSight 加载新知识库

### 知识库验证

```bash
# 验证 YAML 格式
cargo test --package flowsight-knowledge

# 检查覆盖率
./scripts/kb-coverage.sh
```

## 相关文档

- [功能说明](./features.md)
- [异步机制分析](./async-analysis.md)
- [知识库架构](../architecture/KNOWLEDGE-BASE-SCHEMA.md)

---

**知识库版本**: 2026.02  
**最后更新**: 2026-02-04
