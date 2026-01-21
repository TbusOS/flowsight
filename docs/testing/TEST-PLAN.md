# FlowSight Linux 内核测试方案

> **核心目标**: 验证 FlowSight 是否真正帮助用户理解代码执行流程，而非仅仅显示函数调用关系。

## 目录

1. [测试策略概述](#1-测试策略概述)
2. [核心价值验证测试](#2-核心价值验证测试)
3. [内核子系统测试](#3-内核子系统测试)
4. [IDE 可视化功能测试](#4-ide-可视化功能测试)
5. [知识库覆盖测试](#5-知识库覆盖测试)
6. [真实内核代码测试](#6-真实内核代码测试)
7. [验收标准](#7-验收标准)

---

## 1. 测试策略概述

### 1.1 测试哲学

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        FlowSight 核心价值                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   "执行流 ≠ 函数调用关系"                                                 │
│                                                                          │
│   传统 IDE 显示:                    FlowSight 应该显示:                  │
│                                                                          │
│   probe()                          USB 设备插入                           │
│   ├── INIT_WORK()                    └── usb_hub_port_connect()         │
│   ├── timer_setup()       VS           └── usb_new_device()             │
│   └── usb_set_intfdata()                └── device_add()                │
│                                          └── bus_probe_device()          │
│   用户不知道何时/如何触发               └── driver_probe_device()        │
│                                             └── really_probe()           │
│                                               └── my_probe() ← 用户代码  │
│                                                                          │
│   关键差异:                                                               │
│   • 展示完整内核调用链 (从硬件事件到用户代码)                              │
│   • 标注执行上下文 (process/softirq/hardirq)                             │
│   • 展示异步时间线关系 (中断 → 工作队列)                                  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 测试覆盖现状

| 分类 | 子系统 | 知识库 | 单元测试 | 集成测试 | 优先级 |
|------|--------|--------|----------|----------|--------|
| **核心机制** | WorkQueue | ✅ | ✅ | ✅ | P0 |
| | Timer (timer_list) | ❌ | ❌ | ❌ | P0 |
| | IRQ/异常 | ❌ | ❌ | ❌ | P0 |
| | Kthread | ❌ | ❌ | ❌ | P1 |
| | RCU | ❌ | ❌ | ❌ | P1 |
| **驱动框架** | USB | ✅ | ✅ | ✅ | P0 |
| | 字符设备 | ✅ | ✅ | ✅ | P0 |
| | Platform | ❌ | ✅ | ✅ | P0 |
| | PCI/PCIe | ❌ | ❌ | ❌ | P1 |
| | I2C | ❌ | ✅ | ✅ | P1 |
| | SPI | ❌ | ❌ | ❌ | P2 |
| **子系统** | 网络 (netdev) | ❌ | ✅ | ✅ | P1 |
| | 块设备 | ❌ | ✅ | ✅ | P1 |
| | VFS | ❌ | ❌ | ❌ | P1 |
| **设备驱动** | Input | ❌ | ❌ | ❌ | P2 |
| | Sound/ALSA | ❌ | ❌ | ❌ | P2 |
| | V4L2 | ❌ | ❌ | ❌ | P2 |
| | Bluetooth | ❌ | ❌ | ❌ | P2 |
| | WiFi | ❌ | ❌ | ❌ | P2 |
| | GPU/显示 | ❌ | ❌ | ❌ | P2 |
| | RTC | ❌ | ❌ | ❌ | P2 |
| | Watchdog | ❌ | ❌ | ❌ | P2 |
| | Thermal | ❌ | ❌ | ❌ | P2 |
| | IIO | ❌ | ❌ | ❌ | P2 |
| **同步原语** | Completion | ❌ | ❌ | ❌ | P1 |
| | Mutex/Spinlock | ❌ | ❌ | ❌ | P1 |

---

## 2. 核心价值验证测试

### 2.1 测试目标

验证 FlowSight 是否真正解决了"用户无法理解代码何时、如何被调用"这一核心问题。

### 2.2 测试用例

#### TC-EXEC-001: 内核调用链注入验证

**目的**: 验证对于 USB probe 回调，IDE 是否展示了从"USB 设备插入"到用户代码的完整调用链。

```yaml
测试代码:
  文件: tests/fixtures/usb_probe_driver.c
  模式: USB 设备驱动，包含 probe/disconnect/work handler

预期输出:
  flow_tree:
    - 根节点: "USB 设备插入事件"
      children:
        - "usb_hub_port_connect() [drivers/usb/core/hub.c]"
        - "usb_new_device() [drivers/usb/core/hub.c]"
        - "device_add() [drivers/core/core.c]"
        - "bus_probe_device() [drivers/base/bus.c]"
        - "driver_probe_device() [drivers/base/dd.c]"
        - "really_probe() [drivers/base/dd.c]"
          children:
            - "usb_probe_interface() [drivers/usb/core/driver.c]"
              children:
                - "my_probe() [user code]"  ← 用户代码入口点
                  children:
                    - "INIT_WORK()"
                    - "timer_setup()"

验收标准:
  ✓ 展示完整 8 层内核调用链
  ✓ 标注文件路径
  ✓ 突出显示用户代码入口点 (is_user_entry: true)
  ✓ 上下文标注为 "process"
```

#### TC-EXEC-002: 执行上下文标注验证

**目的**: 验证不同执行上下文的函数是否正确标注。

```yaml
测试代码:
  文件: tests/fixtures/mixed_context_driver.c
  包含:
    - 硬中断处理 (request_irq)
    - 软中断 (tasklet/softirq)
    - 进程上下文 (work queue)
    - 定时器 (timer_list)

预期输出:
  函数上下文标注:
    - my_irq_handler: "hardirq" (cannot_sleep)
    - my_tasklet_fn: "softirq" (cannot_sleep)
    - my_softirq_handler: "softirq" (cannot_sleep)
    - my_work_handler: "process" (can_sleep)
    - my_timer_fn: "softirq" (cannot_sleep)

验收标准:
  ✓ 所有异步回调正确标注执行上下文
  ✓ 正确识别 can_sleep 属性
  ✓ 在 IDE 中用不同颜色/图标区分
```

#### TC-EXEC-003: 异步时间线可视化验证

**目的**: 验证 IDE 是否能展示跨上下文的异步执行流。

```yaml
测试代码:
  文件: tests/fixtures/irq_to_workqueue.c
  场景: 硬件中断 → 上半部 → 下半部工作队列

预期输出:
  timeline:
    - phase: "中断上半部"
      context: "hardirq"
      duration: "< 100μs"
      call_chain:
        - "do_IRQ()"
        - "handle_irq()"
        - "my_irq_handler()"
          actions:
            - "tasklet_schedule(&dev->tasklet)"
            - "return IRQ_WAKE_THREAD"

    - phase: "调度间隔"
      description: "CPU 执行其他任务 → 调度器选择 kworker"

    - phase: "中断下半部 (WorkQueue)"
      context: "process"
      call_chain:
        - "worker_thread() [kernel/workqueue.c]"
        - "process_one_work()"
        - "my_tasklet_fn()"
          actions:
            - "schedule_work(&dev->work)"

    - phase: "工作队列执行"
      context: "process"
      call_chain:
        - "my_work_handler()"

验收标准:
  ✓ 清晰展示中断 → 工作队列的异步关系
  ✓ 标注各阶段执行上下文
  ✓ 展示时间线（即使只是定性的）
```

#### TC-EXEC-004: 回调触发条件展示验证

**目的**: 验证 IDE 是否清晰展示回调的触发条件。

```yaml
测试代码:
  文件: tests/fixtures/callback_triggers.c

  场景:
    - module_init: 模块加载时
    - probe: 设备匹配时
    - suspend: 系统休眠时
    - resume: 系统唤醒时

预期输出:
  entry_points:
    - name: "my_init"
      trigger: "模块加载 (insmod)"
      kernel_chain:
        - "sys_init_module()"
        - "do_init_module()"
        - "my_init()"

    - name: "my_probe"
      trigger: "USB 设备插入 + ID 匹配"
      kernel_chain:
        - "usb_hub_port_connect()"
        - "... (7层调用)"
        - "my_probe()"

    - name: "my_suspend"
      trigger: "系统进入休眠 (echo mem > /sys/power/state)"
      kernel_chain:
        - "pm_suspend()"
        - "suspend_devices_and_enter()"
        - "usb_suspend_both()"
        - "my_suspend()"

验收标准:
  ✓ 每个入口点都有明确的触发条件描述
  ✓ 展示完整的内核调用链
  ✓ 区分"同步触发"和"异步触发"
```

---

## 3. 内核子系统测试

### 3.1 核心机制测试

#### TC-KERN-001: WorkQueue 完整测试

```yaml
知识库文件: knowledge/platforms/linux-kernel/core/workqueue.yaml
状态: ✅ 已完成

测试用例:
  - INIT_WORK / INIT_WORK_ONSTACK / DECLARE_WORK 识别
  - schedule_work / queue_work / queue_work_on 识别
  - cancel_work_sync / cancel_work 识别
  - delayed_work 识别
  - workqueue creation/destruction 识别

测试代码:
  文件: tests/fixtures/workqueue_driver.c
  包含:
    - 标准 work_struct 使用
    - delayed_work 使用
    - 自定义 workqueue 创建
    - 各种 schedule 变体

验收标准:
  ✓ 识别所有 3 种 work 初始化方式
  ✓ 识别所有 4 种调度方式
  ✓ 识别取消操作
  ✓ 正确关联 handler 函数
```

#### TC-KERN-002: Timer 机制测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/core/timer.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - DEFINE_TIMER (旧式宏)
    - timer_setup (新式 API)
    - mod_timer / add_timer / del_timer
    - hrtimer_* 系列

  内核调用链:
    - timer_list: run_timer_softirq() → timer_callback()
    - hrtimer: hrtimer_interrupt() → hrtimer_callback()

测试代码:
  文件: tests/fixtures/timer_driver.c
  场景:
    - 单次定时器
    - 周期定时器
    - 定时器中调度 work
    - 高精度定时器 (hrtimer)

验收标准:
  ✓ 识别 DEFINE_TIMER 和 timer_setup
  ✓ 识别 timer 触发函数
  ✓ 展示 timer 软中断调用链
  ✓ 正确标注 softirq 上下文
```

#### TC-KERN-003: IRQ/异常处理测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/core/irq.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - request_irq / devm_request_irq
    - request_threaded_irq
    - tasklet_init / DECLARE_TASKLET
    - softirq 注册/触发

  内核调用链:
    - 硬件中断入口
    - do_IRQ → handle_irq
    - irq_handler 返回路径

测试代码:
  文件: tests/fixtures/irq_driver.c
  场景:
    - 简单 IRQ 处理
    - 线程化 IRQ (top half + thread fn)
    - Tasklet 使用
    - Softirq 自定义

验收标准:
  ✓ 识别 request_irq 回调
  ✓ 识别线程化 IRQ 的两个函数
  ✓ 识别 tasklet 绑定和调度
  ✓ 展示硬件中断入口调用链
  ✓ 正确标注 hardirq/softirq 上下文
```

#### TC-KERN-004: Kernel Thread 测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/core/kthread.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - kthread_create / kthread_run
    - kthread_should_stop
    - wake_up_process

  内核调用链:
    - kthread_create → kthreadd → 新线程

测试代码:
  文件: tests/fixtures/kthread_driver.c
  场景:
    - 创建内核线程
    - 线程循环模式
    - 线程退出

验收标准:
  ✓ 识别 kthread_create/kthread_run
  ✓ 识别 wake_up_process
  ✓ 展示线程创建调用链
  ✓ 正确标注进程上下文
```

#### TC-KERN-005: RCU 机制测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/core/rcu.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - call_rcu / call_srcu
    - rcu_read_lock / rcu_read_unlock
    - synchronize_rcu

  内核调用链:
    - call_rcu → rcu_do_batch (宽限期后)

测试代码:
  文件: tests/fixtures/rcu_driver.c
  场景:
    - RCU 回调注册
    - RCU 读取侧临界区
    - 同步 RCU

验收标准:
  ✓ 识别 call_rcu 绑定
  ✓ 标注 RCU 读取上下文
  ✓ 展示回调在宽限期后执行
```

### 3.2 驱动框架测试

#### TC-DRV-001: USB 驱动测试

```yaml
知识库文件: knowledge/platforms/linux-kernel/drivers/usb.yaml
状态: ✅ 已完成

测试代码:
  文件: tests/fixtures/usb_driver.c
  文件: tests/fixtures/usb_skeleton.c (真实内核代码)

  场景:
    - probe/disconnect
    - suspend/resume
    - URB 回调
    - class driver (open/release)

验收标准:
  ✓ 识别 usb_driver 结构体回调
  ✓ 识别 module_usb_driver 宏
  ✓ 展示 probe/disconnect 调用链
  ✓ 识别 URB 回调函数
```

#### TC-DRV-002: 字符设备测试

```yaml
知识库文件: knowledge/platforms/linux-kernel/drivers/char_dev.yaml
状态: ✅ 已完成

测试代码:
  文件: tests/fixtures/char_device.c

验收标准:
  ✓ 识别 file_operations 回调
  ✓ 识别 cdev_init/cdev_add
  ✓ 展示用户空间 open 调用链
```

#### TC-DRV-003: Platform 驱动测试

```yaml
知识库文件: knowledge/platforms/linux-kernel/drivers/platform.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - platform_driver_register
    - module_platform_driver
    - probe/remove/suspend/resume 回调

  内核调用链:
    - platform_bus_type.match
    - platform_bus_type.probe

测试代码:
  文件: tests/fixtures/platform_driver.c

验收标准:
  ✓ 识别 platform_driver 结构体
  ✓ 展示 platform 总线匹配和 probe 调用链
```

#### TC-DRV-004: PCI/PCIe 驱动测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/drivers/pci.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - pci_register_driver
    - module_pci_driver
    - probe/remove/suspend/resume
    - MSI/MSI-X 中断

  内核调用链:
    - PCI 总线枚举
    - pci_device_probe
    - pci_match_id

测试代码:
  文件: tests/fixtures/pci_driver.c
  场景:
    - 简单 PCI 驱动
    - MSI 中断
    - PCI 热拔插

验收标准:
  ✓ 识别 pci_driver 回调
  ✓ 展示 PCI 枚举调用链
  ✓ 识别 MSI 中断注册
```

#### TC-DRV-005: I2C 驱动测试 ⭐ 需完善

```yaml
知识库文件: knowledge/platforms/linux-kernel/drivers/i2c.yaml
状态: ❌ 未实现 (虽有测试，无知识库)

需要实现:
  模式:
    - i2c_add_driver / i2c_register_driver
    - module_i2c_driver
    - probe/remove
    - i2c_client 回调

  内核调用链:
    - i2c_register_adapter
    - i2c_device_probe

测试代码:
  文件: tests/fixtures/i2c_driver.c

验收标准:
  ✓ 识别 i2c_driver 结构体
  ✓ 展示 I2C 设备匹配调用链
```

#### TC-DRV-006: SPI 驱动测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/drivers/spi.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - spi_register_driver
    - module_spi_driver
    - probe/remove
    - spi_transfer 回调

测试代码:
  文件: tests/fixtures/spi_driver.c

验收标准:
  ✓ 识别 spi_driver 回调
  ✓ 识别 spi_transfer 完成回调
```

### 3.3 子系统测试

#### TC-SUB-001: 网络设备测试 ⭐ 需完善

```yaml
知识库文件: knowledge/platforms/linux-kernel/net/netdev.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - alloc_etherdev / alloc_netdev
    - net_device_ops 回调
    - ndo_start_xmit / ndo_open / ndo_stop
    - ndo_tx_timeout

  内核调用链:
    - 用户空间 socket 调用
    - 网络栈到驱动

测试代码:
  文件: tests/fixtures/netdev_driver.c
  真实代码: linux_kernel/drivers/net/ethernet/...

验收标准:
  ✓ 识别 net_device_ops 回调
  ✓ 展示数据包发送调用链 (socket → tx_timeout)
  ✓ 识别中断相关的 NAPI 回调
```

#### TC-SUB-002: 块设备测试 ⭐ 需完善

```yaml
知识库文件: knowledge/platforms/linux-kernel/fs/block.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - gendisk 操作
    - block_device_operations
    - request_fn / bio 处理

  内核调用链:
    - 用户空间 I/O → 块设备层 → 驱动

测试代码:
  文件: tests/fixtures/block_driver.c

验收标准:
  ✓ 识别 block_device_operations 回调
  ✓ 展示块 I/O 请求调用链
```

#### TC-SUB-003: VFS 测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/fs/vfs.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - file_system_type
    - super_operations
    - inode_operations
    - dentry_operations

测试代码:
  文件: tests/fixtures/vfs_driver.c

验收标准:
  ✓ 识别文件系统注册
  ✓ 展示 VFS 调用链
```

### 3.4 设备驱动分类测试

#### TC-DEV-001: Input 驱动测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/drivers/input.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - input_register_device
    - input_register_handler
    - input_event 回调

测试代码:
  文件: tests/fixtures/input_driver.c
  场景: 键盘/鼠标/触摸屏驱动

验收标准:
  ✓ 识别 input_dev 回调
  ✓ 展示 input 事件处理调用链
```

#### TC-DEV-002: Sound/ALSA 驱动测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/drivers/sound.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - snd_card_new
    - snd_pcm_ops
    - snd_card_register

测试代码:
  文件: tests/fixtures/sound_driver.c

验收标准:
  ✓ 识别 ALSA PCM 回调
  ✓ 展示音频流处理调用链
```

#### TC-DEV-003: V4L2 驱动测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/drivers/v4l2.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - video_register_device
    - v4l2_file_operations
    - v4l2_ioctl_ops

测试代码:
  文件: tests/fixtures/v4l2_driver.c

验收标准:
  ✓ 识别 V4L2 回调
  ✓ 展示视频采集/输出调用链
```

#### TC-DEV-004: RTC 驱动测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/drivers/rtc.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - rtc_register_device
    - rtc_class_ops

测试代码:
  文件: tests/fixtures/rtc_driver.c

验收标准:
  ✓ 识别 RTC 回调
  ✓ 展示 RTC 闹钟调用链
```

#### TC-DEV-005: 其他设备驱动 ⭐ 未来

| 子系统 | 知识库文件 | 状态 | 测试文件 |
|--------|-----------|------|----------|
| Watchdog | watchdog.yaml | ❌ | watchdog_driver.c |
| Thermal | thermal.yaml | ❌ | thermal_driver.c |
| IIO | iio.yaml | ❌ | iio_driver.c |
| Regulator | regulator.yaml | ❌ | regulator_driver.c |
| PinCtrl | pinctrl.yaml | ❌ | pinctrl_driver.c |
| Bluetooth | bluetooth.yaml | ❌ | bt_driver.c |
| WiFi | wifi.yaml | ❌ | wifi_driver.c |
| GPU | gpu.yaml | ❌ | gpu_driver.c |
| USB Gadget | usb_gadget.yaml | ❌ | gadget_driver.c |
| MD | md.yaml | ❌ | md_driver.c |
| NVMEM | nvmem.yaml | ❌ | nvmem_driver.c |

### 3.5 同步原语测试

#### TC-SYNC-001: Completion 测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/sync/completion.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - init_completion
    - wait_for_completion
    - complete / complete_all

测试代码:
  文件: tests/fixtures/completion.c

验收标准:
  ✓ 识别 completion 初始化
  ✓ 识别等待和完成操作
  ✓ 展示同步等待调用链
```

#### TC-SYNC-002: Mutex/Spinlock 测试 ⭐ 新增

```yaml
知识库文件: knowledge/platforms/linux-kernel/sync/locking.yaml
状态: ❌ 未实现

需要实现:
  模式:
    - mutex_init / mutex_lock / mutex_unlock
    - spin_lock / spin_unlock 系列
    - spin_lock_irqsave / spin_unlock_irqrestore

测试代码:
  文件: tests/fixtures/locking.c

验收标准:
  ✓ 识别锁操作
  ✓ 标注锁类型
  ✓ 展示锁竞争场景
```

---

## 4. IDE 可视化功能测试

### 4.1 执行流展示测试

#### TC-IDE-001: 调用链树状图验证

```yaml
测试场景: 打开一个 USB 驱动文件

预期行为:
  左侧边栏显示函数列表，按调用关系组织成树状图

  usb_driver_probe()
  ├── usb_get_dev()
  ├── usb_find_common_endpoints()
  │   └── usb_endpoint_maxp()
  ├── kmalloc()
  ├── INIT_WORK()
  │   └── skel_work_handler()  ← 可展开
  └── usb_register_dev()

验证:
  ✓ 树状结构正确反映调用关系
  ✓ 用户代码与内核代码用不同颜色区分
  ✓ 可折叠/展开子节点
  ✓ 双击跳转到定义
```

#### TC-IDE-002: 执行上下文颜色标注验证

```yaml
测试场景: 打开包含多种上下文的驱动文件

预期行为:
  函数/调用根据上下文显示不同颜色:

  - hardirq (红色): request_irq 回调
  - softirq (橙色): tasklet, timer
  - process (绿色): work queue, probe
  - atomic (蓝色): spinlock 临界区

验证:
  ✓ 颜色标注正确
  ✓ 悬浮显示上下文详细信息
  ✓ 图标区分不可调度/可睡眠
```

#### TC-IDE-003: 内核调用链注入展示验证

```yaml
测试场景: 选择 probe 回调，查看完整调用链

预期行为:
  显示一个可折叠的调用链面板:

  ▼ USB 设备插入 (触发条件)
    ▼ drivers/usb/core/hub.c
      usb_hub_port_connect()
      usb_new_device()
    ▼ drivers/base/core.c
      device_add()
      bus_probe_device()
    ▼ drivers/base/dd.c
      driver_probe_device()
      really_probe()
    ▼ drivers/usb/core/driver.c
      usb_probe_interface()
      ▼ my_probe() [user code]
        (用户代码...)

验证:
  ✓ 内核代码路径可折叠
  ✓ 用户代码突出显示
  ✓ 显示文件路径和行号
  ✓ 点击跳转到对应代码
```

### 4.2 时间线可视化测试

#### TC-IDE-004: 异步事件时间线验证

```yaml
测试场景: 打开 IRQ + WorkQueue 驱动的代码

预期行为:
  时间线面板显示:

  时间线: 中断处理 → 线程化 IRQ → 工作队列

  ┌─────────────────────────────────────────────────────┐
  │ [硬中断] do_IRQ()     ████░░░░░░░░                   │
  │                          ↑ IRQ_WAKE_THREAD          │
  │ [线程]  my_thread_fn() ░░░░░████░░░░░               │
  │                                ↑ schedule_work()    │
  │ [进程]  my_work()     ░░░░░░░░░░████░               │
  └─────────────────────────────────────────────────────┘

验证:
  ✓ 时间线正确反映异步关系
  ✓ 上下文切换点标注清晰
  ✓ 可缩放查看不同粒度
```

### 4.3 代码导航测试

#### TC-IDE-005: 回调跳转验证

```yaml
测试场景: 点击 schedule_work，查看跳转到 handler

预期行为:
  - 光标定位到 INIT_WORK 绑定
  - 显示浮动卡片:
    ```
    schedule_work(&dev->work)
    └── 绑定到: my_work_handler
       触发: 延迟执行 (process context)
       取消: cancel_work_sync()
    ```

验证:
  ✓ 正确跳转到 handler 定义
  ✓ 显示绑定关系信息
  ✓ 可快速跳转到取消/触发函数
```

#### TC-IDE-006: 函数引用查找验证

```yaml
测试场景: 右键点击 probe 函数，选择"查找所有引用"

预期行为:
  - 引用面板显示:
    ```
    probe() 被以下调用:
    ├── driver_probe_device() [drivers/base/dd.c:1234]
    └── module_usb_driver() [my_driver.c:99]

    probe 触发条件:
    └── USB 设备插入 + ID 匹配
    ```

验证:
  ✓ 找到所有直接引用
  ✓ 找到通过函数指针的引用
  ✓ 显示触发条件说明
```

---

## 5. 知识库覆盖测试

### 5.1 知识库完整性测试

#### TC-KB-001: 模式覆盖率测试

```yaml
目的: 验证知识库覆盖了 Linux 内核中常用的异步模式

测试方法:
  1. 从 linux_kernel/drivers 目录采样 50 个真实驱动
  2. 运行 FlowSight 分析每个驱动
  3. 统计识别的回调函数数量 vs 实际回调函数数量

预期结果:
  - USB 驱动: >95% 识别率
  - 字符设备: >95% 识别率
  - 网络设备: >90% 识别率
  - 块设备: >90% 识别率
  - Platform 设备: >90% 识别率

计算公式:
  识别率 = 识别的回调数 / 实际回调数 × 100%
```

#### TC-KB-002: 知识库一致性测试

```yaml
目的: 验证知识库模式与实际内核 API 一致

测试方法:
  1. 对每个知识库模式，生成对应的正则
  2. 用正则匹配内核源码 (linux/kernel/*.c, drivers/*/*.c)
  3. 验证匹配结果符合预期

验证:
  ✓ 正则不遗漏常见用法
  ✓ 正则不产生误匹配
  ✓ 捕获组正确提取函数名
```

### 5.2 知识库准确性测试

#### TC-KB-003: 执行上下文准确性测试

```yaml
目的: 验证知识库中执行上下文的标注正确

测试方法:
  1. 列出知识库中所有上下文标注
  2. 对照内核文档验证
  3. 运行实际内核测试验证

验证结果:
  work_struct: "process" ✓
  timer_list: "softirq" ✓
  tasklet: "softirq" ✓
  irq_handler: "hardirq" ✓
  kthread: "process" ✓
```

---

## 6. 真实内核代码测试

### 6.1 测试策略

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        真实代码测试策略                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Level 1: 合成测试用例 (已完成)                                           │
│  ════════════════════════                                                │
│  • 人工编写的简单驱动                                                     │
│  • 覆盖基本模式                                                          │
│  • 验证核心功能正确性                                                     │
│                                                                          │
│  Level 2: 内核示例驱动 (需添加)                                           │
│  ════════════════════════                                                │
│  • drivers/usb/usb-skeleton.c                                            │
│  • drivers/char/mem.c                                                    │
│  • drivers/net/ethernet/ 下的简单驱动                                     │
│  • 验证对真实代码的兼容性                                                 │
│                                                                          │
│  Level 3: 主流驱动 (需添加)                                               │
│  ════════════════════════                                                │
│  • usb-storage, ehci-hcd, xhci-hcd                                       │
│  • e1000, iwlwifi, r8169                                                 │
│  • 验证复杂场景处理能力                                                   │
│                                                                          │
│  Level 4: 完整子系统 (未来)                                               │
│  ════════════════════════                                                │
│  • drivers/net/whole                                                      │
│  • drivers/usb/storage/                                                   │
│  • 验证大规模代码库处理能力                                               │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.2 推荐测试文件

#### 核心驱动测试

| 文件路径 | 子系统 | 复杂度 | 优先级 |
|---------|--------|--------|--------|
| drivers/usb/storage/sg.c | USB | 低 | P0 |
| drivers/usb/storage/unusual_devs.h | USB | 低 | P0 |
| drivers/char/mem.c | 字符设备 | 低 | P0 |
| drivers/char/random.c | 字符设备 | 中 | P1 |
| drivers/net/ethernet/intel/e1000/e1000_main.c | 网络 | 高 | P2 |
| drivers/net/ethernet/realtek/8139too.c | 网络 | 中 | P1 |
| drivers/i2c/i2c-core.c | I2C | 高 | P1 |
| drivers/spi/spi.c | SPI | 中 | P2 |
| drivers/pci/pci-driver.c | PCI | 高 | P2 |
| drivers/platform/x86/toshiba_acpi.c | Platform | 高 | P2 |

#### 测试用例模板

```rust
// tests/integration/real_driver_test.rs

#[test]
fn test_usb_storage_driver() {
    // 测试文件: drivers/usb/storage/sg.c
    let test_file = "/home/parallels/linux_kernel/drivers/usb/storage/sg.c";
    let source = std::fs::read_to_string(test_file)
        .expect("Failed to read test file");

    let mut parser = TreeSitterParser::new();
    let mut parse_result = parser.parse_source(&source, test_file)
        .expect("Failed to parse");

    let mut analyzer = Analyzer::new();
    let result = analyzer.analyze(&source, &mut parse_result)
        .expect("Analysis failed");

    // 验证回调识别
    let expected_callbacks = vec![
        "sg_start_req",      // file_operations.read
        "sg_read",           // file_operations.read
        "sg_ioctl",          // file_operations.ioctl
        "sg_new_read",       // usb_stor 回调
    ];

    for callback in expected_callbacks {
        let func = parse_result.functions.get(callback);
        assert!(func.is_some(), "Should find callback: {}", callback);
        if let Some(f) = func {
            assert!(f.is_callback, "{} should be marked as callback", callback);
        }
    }

    // 验证异步机制识别
    let work_handlers: Vec<_> = result.async_bindings.iter()
        .filter(|b| b.mechanism == AsyncMechanism::WorkQueue)
        .map(|b| b.handler.clone())
        .collect();

    assert!(!work_handlers.is_empty(), "Should find work queue handlers");
}
```

---

## 7. 验收标准

### 7.1 核心功能验收标准

| 编号 | 标准 | 验证方法 | 优先级 |
|------|------|----------|--------|
| AC-01 | 正确识别 95%+ 的 USB 驱动回调 | 采样 10 个 USB 驱动 | P0 |
| AC-02 | 正确识别 95%+ 的字符设备回调 | 采样 10 个 char 驱动 | P0 |
| AC-03 | 正确识别 90%+ 的网络设备回调 | 采样 10 个 net 驱动 | P1 |
| AC-04 | 内核调用链展示 100% 准确 | 人工验证 | P0 |
| AC-05 | 执行上下文标注 100% 准确 | 人工验证 | P0 |
| AC-06 | 无内核代码误识别为用户代码 | 人工验证 | P0 |

### 7.2 知识库验收标准

| 编号 | 标准 | 验证方法 | 优先级 |
|------|------|----------|--------|
| KB-01 | 所有模式有对应的正则 | 代码审查 | P0 |
| KB-02 | 正则有足够的测试覆盖 | 每个模式 5+ 测试用例 | P0 |
| KB-03 | 知识库文件格式正确 | schema 验证 | P0 |
| KB-04 | 文档链接有效 | 自动化检查 | P1 |

### 7.3 IDE 验收标准

| 编号 | 标准 | 验证方法 | 优先级 |
|------|------|----------|--------|
| IDE-01 | 执行流树状图正确显示 | 人工测试 | P0 |
| IDE-02 | 内核调用链可折叠/展开 | 人工测试 | P0 |
| IDE-03 | 上下文颜色标注正确 | 人工测试 | P0 |
| IDE-04 | 回调跳转功能正常 | 人工测试 | P0 |
| IDE-05 | 时间线展示清晰 | 人工测试 | P1 |
| IDE-06 | 性能: 10000 行代码 < 3s | 性能测试 | P1 |

### 7.4 性能验收标准

| 指标 | 标准 | 验证方法 |
|------|------|----------|
| 解析速度 | 10000 行/秒 | 基准测试 |
| 内存占用 | < 500MB (20M 行代码) | 内存分析 |
| 知识库加载 | < 100ms | 性能测试 |
| 响应时间 | UI 操作 < 100ms | 性能测试 |

---

## 附录 A: 测试执行指南

### A.1 运行所有测试

```bash
# 运行单元测试
cargo test --package flowsight-analysis

# 运行集成测试
cargo test --package flowsight-analysis --test real_driver_test
cargo test --package flowsight-analysis --test kernel_analysis_test

# 运行 IDE E2E 测试
cd app && pnpm test

# 运行知识库验证
cargo run --bin flowsight-kb validate-all
```

### A.2 添加新测试

1. 在 `tests/fixtures/` 添加测试驱动源码
2. 在 `tests/` 添加对应的测试文件
3. 更新 `knowledge/` 添加对应知识库
4. 运行测试验证

### A.3 评估测试覆盖率

```bash
# 生成覆盖率报告
cargo tarpaulin --package flowsight-analysis

# 检查知识库覆盖率
cargo run --bin flowsight-kb coverage --source /path/to/kernel/drivers
```

---

## 附录 B: 缺失知识库实现优先级

### B.1 P0 (v1.0 必须)

1. `core/timer.yaml` - Timer 机制
2. `core/irq.yaml` - IRQ 处理
3. `drivers/platform.yaml` - Platform 驱动
4. `drivers/pci.yaml` - PCI 驱动 (基础)
5. `drivers/i2c.yaml` - I2C 驱动

### B.2 P1 (v1.x)

1. `core/kthread.yaml` - Kernel thread
2. `core/rcu.yaml` - RCU 机制
3. `net/netdev.yaml` - 网络设备
4. `fs/block.yaml` - 块设备
5. `fs/vfs.yaml` - VFS 回调
6. `drivers/spi.yaml` - SPI 驱动
7. `sync/completion.yaml` - Completion

### B.3 P2 (未来版本)

1. 所有设备驱动 (Input, Sound, V4L2, RTC, etc.)
2. 其他同步原语
3. 其他子系统

---

*文档版本: 1.0*
*最后更新: 2025-01-21*
*作者: FlowSight Team*
