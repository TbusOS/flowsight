#!/usr/bin/env python3
"""
FlowSight AI 训练数据准备 - 真实数据版

核心原则：
1. 使用真实的 Linux 内核代码，不是人工编造的简单示例
2. 多样化的问题形式
3. 包含详细的推理过程
4. 覆盖各种复杂场景

数据类型：
1. 函数指针追踪（多步推理）
2. 异步执行流分析
3. 复杂调用链重建
4. 多条件分支分析
5. 代码模式识别
6. 错误分析与调试
"""

import json
import os
import re
from pathlib import Path
from typing import List, Dict, Optional
from dataclasses import dataclass, field
import random

# ============================================================================
# 真实训练样本结构
# ============================================================================

@dataclass
class RealTrainingSample:
    """真实训练样本 - 包含推理过程"""
    
    # 基本信息
    id: str
    category: str  # pointer_analysis, async_flow, call_chain, etc.
    difficulty: str  # easy, medium, hard, expert
    
    # 输入
    code: str  # 真实代码片段
    question: str  # 多样化的问题
    
    # 输出（关键：包含推理过程）
    thinking: str  # 推理过程（Chain of Thought）
    answer: str  # 最终答案
    
    # 元数据
    source: str = ""  # 代码来源（如 drivers/usb/core/hub.c）
    concepts: List[str] = field(default_factory=list)  # 涉及的概念
    
    def to_training_format(self) -> dict:
        """转换为训练格式"""
        return {
            "instruction": self.question,
            "input": self.code,
            "output": f"<thinking>\n{self.thinking}\n</thinking>\n\n{self.answer}",
            "metadata": {
                "id": self.id,
                "category": self.category,
                "difficulty": self.difficulty,
                "source": self.source,
                "concepts": self.concepts
            }
        }


# ============================================================================
# 类型1：真实的函数指针分析（多步推理）
# ============================================================================

POINTER_ANALYSIS_SAMPLES = [
    # ============ 样本1：USB 驱动 ops 表分析（简单） ============
    RealTrainingSample(
        id="ptr_001",
        category="pointer_analysis",
        difficulty="easy",
        source="drivers/usb/storage/usb.c",
        code='''
/* 来自 Linux 内核 USB 存储驱动 */
static int storage_probe(struct usb_interface *intf,
                         const struct usb_device_id *id)
{
    struct us_data *us;
    int result;
    
    us = kzalloc(sizeof(*us), GFP_KERNEL);
    if (!us)
        return -ENOMEM;
    
    /* 设置传输函数 */
    us->transport = usb_stor_Bulk_transport;
    us->proto_handler = usb_stor_transparent_scsi_command;
    
    result = usb_stor_acquire_resources(us);
    if (result)
        goto BadDevice;
    
    return 0;
    
BadDevice:
    kfree(us);
    return result;
}

static void storage_disconnect(struct usb_interface *intf)
{
    struct us_data *us = usb_get_intfdata(intf);
    us->transport(us->srb, us);  /* 这里调用的是哪个函数？ */
}
''',
        question="在 storage_disconnect 函数中，us->transport(us->srb, us) 调用的是哪个函数？请分析推理过程。",
        thinking='''让我一步步分析这个函数指针调用：

1. **找到调用点**：
   在 storage_disconnect 中有 `us->transport(us->srb, us)`

2. **分析 us 的来源**：
   `struct us_data *us = usb_get_intfdata(intf)`
   这个 us 是在 probe 时通过 usb_set_intfdata 设置的（虽然代码中没显示）

3. **追踪 transport 字段的赋值**：
   在 storage_probe 函数中：
   `us->transport = usb_stor_Bulk_transport;`
   
4. **确认赋值和调用是同一个对象**：
   - probe 时创建 us 并设置 transport
   - disconnect 时获取同一个 us 并调用 transport
   - 中间没有其他地方修改 transport
   
5. **结论**：
   us->transport 在 probe 中被赋值为 usb_stor_Bulk_transport
   disconnect 中调用的就是这个函数''',
        answer='''**分析结果**：

`us->transport(us->srb, us)` 调用的是 **usb_stor_Bulk_transport** 函数。

**推理依据**：
1. 在 `storage_probe` 中，`us->transport = usb_stor_Bulk_transport`
2. 同一个 `us_data` 结构体在 `disconnect` 时被获取
3. 从 probe 到 disconnect 期间没有其他代码修改 `transport` 字段

**确定性**：100%（单一赋值点，无条件分支）''',
        concepts=["function_pointer", "usb_driver", "probe_disconnect"]
    ),
    
    # ============ 样本2：条件赋值的函数指针（中等） ============
    RealTrainingSample(
        id="ptr_002",
        category="pointer_analysis",
        difficulty="medium",
        source="drivers/net/ethernet/intel/e1000e/netdev.c",
        code='''
/* 来自 Intel e1000e 网卡驱动 */
static int e1000_probe(struct pci_dev *pdev, const struct pci_device_id *ent)
{
    struct net_device *netdev;
    struct e1000_adapter *adapter;
    struct e1000_hw *hw;
    
    netdev = alloc_etherdev(sizeof(struct e1000_adapter));
    adapter = netdev_priv(netdev);
    hw = &adapter->hw;
    
    /* 根据硬件类型设置不同的操作函数 */
    switch (hw->mac.type) {
    case e1000_82571:
    case e1000_82572:
        hw->mac.ops.setup_link = e1000_setup_link_82571;
        hw->mac.ops.reset_hw = e1000_reset_hw_82571;
        break;
    case e1000_82573:
        hw->mac.ops.setup_link = e1000_setup_link_82573;
        hw->mac.ops.reset_hw = e1000_reset_hw_82573;
        break;
    case e1000_ich8lan:
    case e1000_ich9lan:
    case e1000_ich10lan:
        hw->mac.ops.setup_link = e1000_setup_link_ich8lan;
        hw->mac.ops.reset_hw = e1000_reset_hw_ich8lan;
        break;
    default:
        hw->mac.ops.setup_link = e1000_setup_link_generic;
        hw->mac.ops.reset_hw = e1000_reset_hw_generic;
        break;
    }
    
    /* 后续调用 */
    hw->mac.ops.reset_hw(hw);
    return 0;
}
''',
        question="当 hw->mac.type 为 e1000_ich9lan 时，hw->mac.ops.reset_hw(hw) 调用的是哪个函数？",
        thinking='''让我分析这个带条件分支的函数指针：

1. **找到调用点**：
   `hw->mac.ops.reset_hw(hw)`

2. **条件是什么**：
   题目说 `hw->mac.type 为 e1000_ich9lan`

3. **分析 switch-case**：
   ```
   case e1000_ich8lan:
   case e1000_ich9lan:    // ← 匹配这个
   case e1000_ich10lan:
       hw->mac.ops.setup_link = e1000_setup_link_ich8lan;
       hw->mac.ops.reset_hw = e1000_reset_hw_ich8lan;  // ← 赋值
       break;
   ```
   
4. **确认匹配**：
   e1000_ich9lan 落入 ich8lan/ich9lan/ich10lan 这个 case 分支

5. **结论**：
   reset_hw 被赋值为 e1000_reset_hw_ich8lan''',
        answer='''**分析结果**：

当 `hw->mac.type` 为 `e1000_ich9lan` 时，`hw->mac.ops.reset_hw(hw)` 调用的是 **e1000_reset_hw_ich8lan** 函数。

**推理依据**：
1. `e1000_ich9lan` 匹配 switch-case 中的 `case e1000_ich8lan: case e1000_ich9lan: case e1000_ich10lan:` 分支
2. 该分支设置 `hw->mac.ops.reset_hw = e1000_reset_hw_ich8lan`

**注意**：
- 函数名是 `ich8lan` 但实际上处理 ich8/ich9/ich10 三种硬件
- 这是常见的代码复用模式

**确定性**：100%（给定条件下，只有一个赋值点）''',
        concepts=["function_pointer", "switch_case", "pci_driver", "hardware_abstraction"]
    ),
    
    # ============ 样本3：多层间接函数指针（困难） ============
    RealTrainingSample(
        id="ptr_003",
        category="pointer_analysis",
        difficulty="hard",
        source="drivers/gpu/drm/drm_crtc.c",
        code='''
/* 来自 DRM (Direct Rendering Manager) 子系统 */

/* 定义操作函数表 */
struct drm_crtc_funcs {
    void (*reset)(struct drm_crtc *crtc);
    int (*set_config)(struct drm_mode_set *set);
    void (*destroy)(struct drm_crtc *crtc);
    int (*page_flip)(struct drm_crtc *crtc, struct drm_framebuffer *fb,
                     struct drm_pending_vblank_event *event, uint32_t flags);
};

/* 驱动注册 CRTC */
int drm_crtc_init_with_planes(struct drm_device *dev,
                              struct drm_crtc *crtc,
                              struct drm_plane *primary,
                              struct drm_plane *cursor,
                              const struct drm_crtc_funcs *funcs)
{
    crtc->dev = dev;
    crtc->funcs = funcs;  /* 存储函数指针表 */
    
    list_add_tail(&crtc->head, &dev->mode_config.crtc_list);
    return 0;
}

/* i915 Intel 显卡驱动的实现 */
static const struct drm_crtc_funcs intel_crtc_funcs = {
    .reset = intel_crtc_reset,
    .set_config = drm_atomic_helper_set_config,
    .destroy = intel_crtc_destroy,
    .page_flip = drm_atomic_helper_page_flip,
};

void intel_crtc_init(struct drm_device *dev, int pipe)
{
    struct intel_crtc *intel_crtc;
    
    intel_crtc = kzalloc(sizeof(*intel_crtc), GFP_KERNEL);
    
    drm_crtc_init_with_planes(dev, &intel_crtc->base,
                              primary, cursor,
                              &intel_crtc_funcs);  /* 传入函数表 */
}

/* DRM 核心调用代码 */
void drm_crtc_reset_all(struct drm_device *dev)
{
    struct drm_crtc *crtc;
    
    list_for_each_entry(crtc, &dev->mode_config.crtc_list, head) {
        if (crtc->funcs->reset)
            crtc->funcs->reset(crtc);  /* 这里调用的是什么？ */
    }
}
''',
        question="在 Intel i915 显卡驱动的场景下，drm_crtc_reset_all 中的 crtc->funcs->reset(crtc) 最终调用的是哪个函数？请详细分析指针的传递链。",
        thinking='''这是一个多层间接调用，让我逐层分析：

1. **调用点**：
   `crtc->funcs->reset(crtc)`
   需要分析：crtc 从哪来，crtc->funcs 指向哪，funcs->reset 是什么

2. **分析 crtc 的来源**：
   `list_for_each_entry(crtc, &dev->mode_config.crtc_list, head)`
   crtc 来自设备的 crtc_list 链表

3. **分析 crtc 如何加入链表**：
   在 `drm_crtc_init_with_planes` 中：
   `list_add_tail(&crtc->head, &dev->mode_config.crtc_list)`
   
4. **分析 funcs 的赋值**：
   在 `drm_crtc_init_with_planes` 中：
   `crtc->funcs = funcs;`  // funcs 是参数传入的

5. **追踪调用者传入的 funcs**：
   在 `intel_crtc_init` 中：
   `drm_crtc_init_with_planes(..., &intel_crtc_funcs);`
   传入的是 `intel_crtc_funcs` 的地址

6. **分析 intel_crtc_funcs.reset**：
   ```
   static const struct drm_crtc_funcs intel_crtc_funcs = {
       .reset = intel_crtc_reset,  // ← 这个！
       ...
   };
   ```

7. **完整调用链**：
   intel_crtc_init 
   → drm_crtc_init_with_planes(&intel_crtc_funcs)
   → crtc->funcs = &intel_crtc_funcs
   → 加入 crtc_list
   → drm_crtc_reset_all 遍历
   → crtc->funcs->reset = intel_crtc_funcs.reset = intel_crtc_reset''',
        answer='''**分析结果**：

在 Intel i915 驱动场景下，`crtc->funcs->reset(crtc)` 调用的是 **intel_crtc_reset** 函数。

**指针传递链**：

```
1. intel_crtc_init()
   └── drm_crtc_init_with_planes(..., &intel_crtc_funcs)
                                        ↓
2. drm_crtc_init_with_planes()
   └── crtc->funcs = funcs (= &intel_crtc_funcs)
   └── list_add_tail → 加入 crtc_list
                                        ↓
3. drm_crtc_reset_all()
   └── 遍历 crtc_list
   └── crtc->funcs->reset(crtc)
       = intel_crtc_funcs.reset(crtc)
       = intel_crtc_reset(crtc)
```

**关键点**：
1. 这是典型的 **ops 表模式**：框架定义接口，驱动实现具体函数
2. 函数指针表 `intel_crtc_funcs` 是 **const static**，全局唯一
3. 通过链表实现了多设备遍历

**确定性**：100%（单一驱动场景下）

**注意**：如果系统有多个不同的显卡驱动，每个驱动有自己的 crtc_funcs，则 reset 会调用不同的函数。''',
        concepts=["function_pointer", "ops_table", "drm", "linked_list", "driver_framework"]
    ),
    
    # ============ 样本4：运行时动态修改的函数指针（专家级） ============
    RealTrainingSample(
        id="ptr_004",
        category="pointer_analysis",
        difficulty="expert",
        source="net/core/dev.c",
        code='''
/* 网络设备 NAPI 机制 */
struct napi_struct {
    int (*poll)(struct napi_struct *napi, int budget);
    struct net_device *dev;
    int weight;
    /* ... */
};

/* 初始化 NAPI */
void netif_napi_add(struct net_device *dev, struct napi_struct *napi,
                    int (*poll)(struct napi_struct *, int), int weight)
{
    napi->poll = poll;
    napi->dev = dev;
    napi->weight = weight;
}

/* Intel e1000e 驱动的 NAPI 初始化 */
static int e1000_probe(struct pci_dev *pdev, ...)
{
    struct e1000_adapter *adapter;
    
    /* 根据硬件特性选择不同的 poll 函数 */
    if (adapter->flags & FLAG_HAS_MSIX) {
        netif_napi_add(netdev, &adapter->napi, e1000_poll_msix, 64);
    } else {
        netif_napi_add(netdev, &adapter->napi, e1000_poll, 64);
    }
    
    return 0;
}

/* NAPI 轮询入口 */
static void napi_poll(struct napi_struct *n)
{
    int work = n->poll(n, n->weight);  /* 这里调用什么？ */
    /* ... */
}

/* 动态切换场景：中断合并优化 */
void e1000_configure_rx(struct e1000_adapter *adapter)
{
    if (adapter->rx_ring_count > 4) {
        /* 多队列优化 */
        adapter->napi.poll = e1000_poll_multiqueue;
    }
}
''',
        question="分析 napi_poll 中 n->poll(n, n->weight) 可能调用哪些函数？需要考虑所有情况。",
        thinking='''这是一个复杂的场景，函数指针可能在多处被修改：

1. **找到所有 poll 的赋值点**：
   - `netif_napi_add` 中：`napi->poll = poll`（参数传入）
   - `e1000_configure_rx` 中：`adapter->napi.poll = e1000_poll_multiqueue`

2. **分析 probe 中的初始赋值**：
   ```
   if (adapter->flags & FLAG_HAS_MSIX) {
       netif_napi_add(..., e1000_poll_msix, ...);
   } else {
       netif_napi_add(..., e1000_poll, ...);
   }
   ```
   条件1：FLAG_HAS_MSIX → e1000_poll_msix
   条件2：无 MSIX → e1000_poll

3. **分析运行时修改**：
   ```
   if (adapter->rx_ring_count > 4) {
       adapter->napi.poll = e1000_poll_multiqueue;
   }
   ```
   条件3：多于4个队列 → e1000_poll_multiqueue
   
4. **时序分析**：
   - probe 时首先设置（msix 或普通）
   - configure_rx 可能在之后修改

5. **所有可能的取值**：
   - e1000_poll_msix（支持 MSIX）
   - e1000_poll（不支持 MSIX）
   - e1000_poll_multiqueue（多队列优化后）''',
        answer='''**分析结果**：

`n->poll(n, n->weight)` 可能调用以下 **3 个函数之一**：

| 函数 | 条件 |
|------|------|
| **e1000_poll_msix** | 硬件支持 MSIX，且 rx_ring_count ≤ 4 |
| **e1000_poll** | 硬件不支持 MSIX，且 rx_ring_count ≤ 4 |
| **e1000_poll_multiqueue** | rx_ring_count > 4（运行时动态修改） |

**赋值时序**：

```
probe 时:
    FLAG_HAS_MSIX? 
    ├── Yes → poll = e1000_poll_msix
    └── No  → poll = e1000_poll

configure_rx 时（可选）:
    rx_ring_count > 4?
    └── Yes → poll = e1000_poll_multiqueue （覆盖之前的值）
```

**确定性**：**不确定** - 取决于运行时硬件配置

**静态分析可做的**：
1. 列出所有可能的目标函数
2. 标注每个的条件
3. 如果有配置文件/设备树可以确定硬件，则可进一步缩小范围

**这就是为什么需要"场景化分析"**：用户指定参数值后才能确定。''',
        concepts=["function_pointer", "runtime_modification", "napi", "conditional_assignment"]
    ),
]

# ============================================================================
# 类型2：异步执行流分析
# ============================================================================

ASYNC_FLOW_SAMPLES = [
    # ============ 样本1：WorkQueue 完整流程 ============
    RealTrainingSample(
        id="async_001",
        category="async_flow",
        difficulty="medium",
        source="drivers/usb/core/hub.c",
        code='''
/* USB Hub 驱动中的 WorkQueue 使用 */
struct usb_hub {
    struct device *dev;
    struct work_struct events;    /* 事件处理工作 */
    unsigned long event_bits[1];  /* 事件位图 */
    /* ... */
};

static void hub_event(struct work_struct *work)
{
    struct usb_hub *hub = container_of(work, struct usb_hub, events);
    
    /* 处理各种 Hub 事件 */
    if (test_and_clear_bit(0, hub->event_bits)) {
        hub_port_connect_change(hub, 1);
    }
}

static int hub_probe(struct usb_interface *intf, ...)
{
    struct usb_hub *hub;
    
    hub = kzalloc(sizeof(*hub), GFP_KERNEL);
    INIT_WORK(&hub->events, hub_event);  /* 绑定处理函数 */
    
    return 0;
}

static irqreturn_t hub_irq(struct urb *urb)
{
    struct usb_hub *hub = urb->context;
    
    set_bit(0, hub->event_bits);
    schedule_work(&hub->events);  /* 触发工作队列 */
    
    return IRQ_HANDLED;
}
''',
        question="当 USB Hub 检测到端口变化时，hub_event 函数是如何被调用的？请分析完整的异步执行流程。",
        thinking='''让我分析从端口变化到 hub_event 执行的完整流程：

1. **触发源**：USB Hub 硬件检测到端口变化
   - 产生中断
   - URB 完成，调用 hub_irq

2. **中断处理（上半部）**：
   ```
   hub_irq:
       set_bit(0, hub->event_bits);   // 标记事件
       schedule_work(&hub->events);   // 提交到工作队列
       return IRQ_HANDLED;            // 立即返回
   ```
   这里在中断上下文，必须快速返回

3. **工作队列机制**：
   schedule_work 做了什么：
   - 将 work 加入系统工作队列
   - 唤醒 kworker 线程
   - 不等待执行，立即返回

4. **工作执行（下半部）**：
   kworker 线程被调度后：
   - 取出 work
   - 调用 work->func，即 hub_event
   - hub_event 通过 container_of 找到 hub
   - 处理事件

5. **绑定关系追踪**：
   probe 中：INIT_WORK(&hub->events, hub_event)
   这建立了 hub->events 和 hub_event 的绑定

6. **时间线**：
   [T0] 硬件中断 → hub_irq → schedule_work → 返回
   [T1] kworker 被调度
   [T2] hub_event 执行''',
        answer='''**异步执行流程分析**：

```
═══════════════════════════════════════════════════════════════════════
                         完整调用链
═══════════════════════════════════════════════════════════════════════

阶段1：硬件中断（上半部）
──────────────────────────────────────────────────────────────────────
[中断上下文，不可睡眠]

USB Hub 硬件检测到端口变化
        ↓
产生 USB 中断
        ↓
hub_irq(urb)
    │
    ├── set_bit(0, hub->event_bits)    // 标记有事件待处理
    │
    └── schedule_work(&hub->events)     // 提交到工作队列
        │
        ├── queue_work_on(cpu, system_wq, work)
        ├── insert_work(pwq, work)
        └── wake_up_worker(pool)        // 唤醒 kworker
                ↓
        返回 IRQ_HANDLED（立即返回，不等待）

═══════════════════════════════════════════════════════════════════════

阶段2：工作队列执行（下半部）
──────────────────────────────────────────────────────────────────────
[进程上下文（kworker），可以睡眠]

kworker 线程被调度
        ↓
worker_thread()
        ↓
process_one_work(worker, work)
        ↓
work->func(work) = hub_event(work)
    │
    ├── container_of(work, struct usb_hub, events) // 找到 hub
    │
    └── hub_port_connect_change(hub, 1)  // 处理端口变化
            │
            └── 可能调用 usb_new_device → probe（可以睡眠）

═══════════════════════════════════════════════════════════════════════
```

**时间线**：

```
[T0 μs]   硬件中断发生
[T0+10μs] hub_irq 执行完毕，返回
          （schedule_work 只是提交，不等待执行）
[T0+??]   kworker 被调度（延迟不确定，通常 <1ms）
[T1]      hub_event 开始执行
          （可能持续数十毫秒，因为可能需要枚举新设备）
```

**关键点**：
1. **INIT_WORK** 建立绑定：`&hub->events` ↔ `hub_event`
2. **schedule_work** 只是触发，不是执行
3. 上半部和下半部之间是 **异步** 的
4. hub_event 在 **进程上下文**，可以调用可能睡眠的函数''',
        concepts=["workqueue", "interrupt", "async", "usb_hub", "container_of"]
    ),
    
    # ============ 样本2：Timer + 重新调度 ============
    RealTrainingSample(
        id="async_002",
        category="async_flow",
        difficulty="medium",
        source="drivers/watchdog/softdog.c",
        code='''
/* 软件看门狗驱动 */
static struct timer_list watchdog_timer;
static unsigned long driver_open;
static int soft_margin = 60;  /* 默认 60 秒超时 */

static void watchdog_fire(struct timer_list *t)
{
    if (time_after(jiffies, watchdog_last_pet + soft_margin * HZ)) {
        pr_crit("Software Watchdog Timer expired!\\n");
        emergency_restart();  /* 系统重启 */
    }
    
    /* 重新调度定时器 */
    mod_timer(&watchdog_timer, jiffies + HZ);
}

static int softdog_open(struct inode *inode, struct file *file)
{
    if (test_and_set_bit(0, &driver_open))
        return -EBUSY;
    
    timer_setup(&watchdog_timer, watchdog_fire, 0);
    mod_timer(&watchdog_timer, jiffies + HZ);  /* 1秒后触发 */
    
    return 0;
}

static ssize_t softdog_write(struct file *file, const char __user *buf,
                             size_t count, loff_t *ppos)
{
    watchdog_last_pet = jiffies;  /* "喂狗" */
    return count;
}
''',
        question="分析软件看门狗的工作机制：定时器是如何周期性执行的？watchdog_fire 函数在什么上下文执行？",
        thinking='''分析看门狗定时器的周期执行机制：

1. **初始化**：
   softdog_open 被调用时：
   - timer_setup 绑定回调函数 watchdog_fire
   - mod_timer 设置 1 秒后触发

2. **定时器触发**：
   1秒后，内核调用 watchdog_fire

3. **周期机制**：
   在 watchdog_fire 中：
   ```
   mod_timer(&watchdog_timer, jiffies + HZ);
   ```
   这会重新设置定时器，又是 1 秒后触发
   
4. **执行上下文**：
   定时器回调在软中断上下文执行
   - TIMER_SOFTIRQ
   - 不可睡眠
   - run_timer_softirq → expire_timers → call_timer_fn

5. **看门狗逻辑**：
   - 用户程序定期写入设备文件 → softdog_write → 更新 watchdog_last_pet
   - 每秒 watchdog_fire 检查：上次"喂狗"是否超过 60 秒
   - 超时则 emergency_restart

6. **时间线**：
   [T0] open → 启动定时器
   [T1] 1秒后 → watchdog_fire → 重新调度
   [T2] 又1秒后 → watchdog_fire → 重新调度
   ... 无限循环 ...''',
        answer='''**软件看门狗工作机制分析**：

**1. 周期执行机制**：

```
softdog_open()
    │
    ├── timer_setup(&watchdog_timer, watchdog_fire, 0)
    │   // 绑定回调函数
    │
    └── mod_timer(&watchdog_timer, jiffies + HZ)
        // 设置 1 秒后触发
                │
                ↓ (1秒后)
        watchdog_fire()
            │
            ├── 检查超时
            │
            └── mod_timer(&watchdog_timer, jiffies + HZ)
                // 重新调度，又是 1 秒后
                        │
                        ↓ (再1秒后)
                watchdog_fire()
                    │
                    └── ... 无限循环 ...
```

**关键**：`mod_timer` 在回调函数里重新调度自己，实现周期执行。

---

**2. 执行上下文**：

`watchdog_fire` 在 **软中断上下文** 执行（TIMER_SOFTIRQ）

**调用链**：
```
[T0] 硬件时钟中断
        ↓
[T1] tick_handle_periodic / hrtimer_interrupt
        ↓
[T2] irq_exit
        ↓
[T3] invoke_softirq → do_softirq
        ↓
[T4] run_timer_softirq (kernel/time/timer.c)
        ↓
[T5] expire_timers
        ↓
[T6] call_timer_fn
        ↓
[T7] watchdog_fire()  ← 你的回调
```

**限制**：
- ❌ 不可睡眠
- ❌ 不可调用可能阻塞的函数
- ✅ 可以访问原子变量
- ✅ 可以调用 spin_lock（不带 _irqsave）

---

**3. 完整时间线**：

```
时间        事件                          上下文
─────────────────────────────────────────────────────
T=0s        open() → 启动定时器           进程
T=1s        watchdog_fire() 第1次         软中断
T=2s        watchdog_fire() 第2次         软中断
...         
T=60s       如果没喂狗 → emergency_restart 软中断
```
''',
        concepts=["timer", "softirq", "periodic", "watchdog", "mod_timer"]
    ),
]

# ============================================================================
# 类型3：复杂调用链分析
# ============================================================================

CALL_CHAIN_SAMPLES = [
    RealTrainingSample(
        id="chain_001",
        category="call_chain",
        difficulty="hard",
        source="drivers/usb/core/",
        code='''
/* 简化的 USB 设备枚举流程 */

/* Hub 检测到设备 */
static void hub_port_connect_change(struct usb_hub *hub, int port)
{
    struct usb_device *udev;
    int status;
    
    udev = usb_alloc_dev(hub->dev, hub->bus, port);
    
    status = hub_port_init(hub, udev, port);
    if (status)
        goto fail;
    
    status = usb_new_device(udev);  /* 注册新设备 */
    if (status)
        goto fail;
    
    return;
fail:
    usb_put_dev(udev);
}

/* 注册 USB 设备 */
int usb_new_device(struct usb_device *udev)
{
    int err;
    
    err = device_add(&udev->dev);  /* 添加到设备模型 */
    if (err)
        return err;
    
    /* 为每个接口创建 usb_interface */
    for (i = 0; i < config->desc.bNumInterfaces; i++) {
        struct usb_interface *intf = config->interface[i];
        device_add(&intf->dev);  /* 这会触发驱动匹配！ */
    }
    
    return 0;
}

/* 设备模型核心 */
int device_add(struct device *dev)
{
    /* ... 各种初始化 ... */
    
    bus_probe_device(dev);  /* 尝试匹配驱动 */
    
    return 0;
}

void bus_probe_device(struct device *dev)
{
    if (dev->bus && dev->bus->probe)
        dev->bus->probe(dev);
    else
        driver_probe_device(drv, dev);
}

int driver_probe_device(struct device_driver *drv, struct device *dev)
{
    return really_probe(dev, drv);
}

int really_probe(struct device *dev, struct device_driver *drv)
{
    /* ... 准备工作 ... */
    
    if (drv->probe)
        ret = drv->probe(dev);  /* 调用驱动的 probe！ */
    
    return ret;
}

/* USB 总线的 probe 封装 */
static int usb_probe_interface(struct device *dev)
{
    struct usb_interface *intf = to_usb_interface(dev);
    struct usb_driver *driver = to_usb_driver(dev->driver);
    
    return driver->probe(intf, id);  /* 调用 USB 驱动的 probe */
}
''',
        question="当用户插入 USB 设备时，USB 驱动的 probe 函数是如何被调用的？请给出从硬件事件到 probe 的完整调用链。",
        thinking='''这是一个长调用链，让我从头到尾追踪：

1. **硬件事件**：USB 设备物理插入

2. **Hub 检测**：
   Hub 中断 → hub_irq → schedule_work → hub_event
   （这部分是异步的）

3. **hub_event 处理**：
   hub_port_connect_change(hub, port)

4. **设备枚举**：
   hub_port_connect_change:
   - usb_alloc_dev → 分配 usb_device
   - hub_port_init → 获取设备描述符
   - usb_new_device → 注册设备

5. **设备注册**：
   usb_new_device:
   - device_add(&udev->dev) → 注册 USB 设备
   - 为每个接口 device_add(&intf->dev) → 注册 USB 接口

6. **驱动匹配**：
   device_add:
   - bus_probe_device(dev)
   - driver_probe_device(drv, dev)
   - really_probe(dev, drv)

7. **USB 封装**：
   really_probe:
   - drv->probe(dev)
   - 对于 USB：usb_probe_interface(dev)

8. **最终调用**：
   usb_probe_interface:
   - driver->probe(intf, id) → 用户驱动的 probe！

完整链条：硬件 → 中断 → 工作队列 → 设备枚举 → 设备模型 → 总线匹配 → probe''',
        answer='''**USB 设备插入到 probe 的完整调用链**：

```
═══════════════════════════════════════════════════════════════════════
                    从 USB 插入到 probe 的完整路径
═══════════════════════════════════════════════════════════════════════

阶段1：硬件检测与中断 [中断上下文]
─────────────────────────────────────────────────────────────────────
USB 设备物理插入
        ↓
Hub 控制器检测到端口变化
        ↓
产生 USB 中断
        ↓
hub_irq()                           [drivers/usb/core/hub.c]
    └── schedule_work(&hub->events)
                ↓
        (异步，转到工作队列)

═══════════════════════════════════════════════════════════════════════

阶段2：Hub 事件处理 [进程上下文 - kworker]
─────────────────────────────────────────────────────────────────────
hub_event()                          [drivers/usb/core/hub.c]
        ↓
hub_port_connect_change(hub, port)   [drivers/usb/core/hub.c]
    │
    ├── usb_alloc_dev()              // 分配 usb_device 结构
    │
    ├── hub_port_init()              // 复位端口，获取描述符
    │
    └── usb_new_device(udev)         // 注册设备
                ↓

═══════════════════════════════════════════════════════════════════════

阶段3：设备注册与驱动匹配
─────────────────────────────────────────────────────────────────────
usb_new_device()                     [drivers/usb/core/hub.c]
    │
    ├── device_add(&udev->dev)       // 注册 USB 设备
    │
    └── device_add(&intf->dev)       // 注册 USB 接口 ← 关键！
                ↓
device_add()                         [drivers/base/core.c]
        ↓
bus_probe_device(dev)                [drivers/base/bus.c]
        ↓
__device_attach(dev)
    └── bus_for_each_drv → __device_attach_driver
                ↓
driver_probe_device(drv, dev)        [drivers/base/dd.c]
        ↓
really_probe(dev, drv)               [drivers/base/dd.c]
        ↓
drv->probe(dev)
    │
    └── (对于 USB 总线) usb_probe_interface()
                ↓

═══════════════════════════════════════════════════════════════════════

阶段4：USB 驱动 probe [进程上下文]
─────────────────────────────────────────────────────────────────────
usb_probe_interface(dev)             [drivers/usb/core/driver.c]
    │
    ├── intf = to_usb_interface(dev)
    │
    ├── driver = to_usb_driver(dev->driver)
    │
    └── driver->probe(intf, id)      ← 用户的 probe 函数！

═══════════════════════════════════════════════════════════════════════
```

**简化调用链**（便于记忆）：

```
USB插入 → hub_irq → hub_event → hub_port_connect_change
       → usb_new_device → device_add → bus_probe_device
       → really_probe → usb_probe_interface → your_probe()
```

**关键点**：
1. **probe 不是 insmod 时调用的**，而是设备插入时
2. 调用者是 **kworker 线程**，不是 insmod 进程
3. probe 在 **进程上下文**，可以睡眠
4. 整个流程是 **异步的**：中断 → 工作队列 → probe
''',
        concepts=["usb", "device_model", "probe", "call_chain", "bus_probe"]
    ),
]

# ============================================================================
# 类型4：代码模式识别
# ============================================================================

PATTERN_RECOGNITION_SAMPLES = [
    RealTrainingSample(
        id="pattern_001",
        category="pattern_recognition",
        difficulty="medium",
        source="多种驱动",
        code='''
/* 模式1：直接赋值 */
static struct file_operations fops = {
    .open = my_open,
    .read = my_read,
};

/* 模式2：宏初始化 */
INIT_WORK(&dev->work, my_work_handler);
timer_setup(&dev->timer, my_timer_fn, 0);

/* 模式3：注册函数 */
request_irq(irq, my_irq_handler, 0, "mydev", dev);
register_netdev(netdev);  /* netdev->netdev_ops 已设置 */

/* 模式4：运行时赋值 */
dev->ops = &default_ops;
if (feature_enabled)
    dev->ops = &enhanced_ops;

/* 模式5：函数返回 */
struct ops *get_ops(int type) {
    switch (type) {
    case TYPE_A: return &ops_a;
    case TYPE_B: return &ops_b;
    }
}
dev->ops = get_ops(config->type);

/* 模式6：回调链 */
struct notifier_block nb = {
    .notifier_call = my_notifier,
};
register_netdevice_notifier(&nb);
''',
        question="识别以上代码中的所有函数指针绑定模式，并说明每种模式的分析难度。",
        thinking='''让我逐一分析这些模式：

1. **直接赋值模式**：
   `.open = my_open`
   静态初始化，编译时确定
   分析难度：简单

2. **宏初始化模式**：
   `INIT_WORK(&dev->work, my_work_handler)`
   宏展开后是赋值
   分析难度：简单（需要知道宏定义）

3. **注册函数模式**：
   `request_irq(irq, my_irq_handler, ...)`
   通过函数参数传递回调
   分析难度：简单（参数位置固定）

4. **运行时条件赋值模式**：
   ```
   dev->ops = &default_ops;
   if (feature_enabled)
       dev->ops = &enhanced_ops;
   ```
   结果取决于运行时条件
   分析难度：中等（需要条件分析）

5. **函数返回模式**：
   `dev->ops = get_ops(config->type)`
   需要分析返回值
   分析难度：困难（需要跨函数分析）

6. **回调链模式**：
   `register_netdevice_notifier(&nb)`
   加入通知链，被动调用
   分析难度：中等（需要知道通知链机制）''',
        answer='''**函数指针绑定模式识别**：

| 模式 | 代码示例 | 分析难度 | 确定性 |
|------|----------|----------|--------|
| **直接赋值** | `.open = my_open` | ⭐ 简单 | 100% |
| **宏初始化** | `INIT_WORK(&work, handler)` | ⭐ 简单 | 100% |
| **注册函数** | `request_irq(irq, handler, ...)` | ⭐ 简单 | 100% |
| **条件赋值** | `if (x) ops = &a; else ops = &b;` | ⭐⭐ 中等 | 多值 |
| **函数返回** | `ops = get_ops(type)` | ⭐⭐⭐ 困难 | 需跨函数 |
| **回调链** | `register_notifier(&nb)` | ⭐⭐ 中等 | 100% |

---

**详细分析**：

**1. 直接赋值（静态初始化）**
```c
static struct file_operations fops = {
    .open = my_open,  // 编译时确定
};
```
- **分析方法**：语法解析即可
- **确定性**：100%

**2. 宏初始化**
```c
INIT_WORK(&dev->work, my_work_handler);
// 展开为：(work)->func = (handler)
```
- **分析方法**：宏展开 + 语法解析
- **确定性**：100%
- **需要**：内置宏知识库

**3. 条件赋值**
```c
if (feature_enabled)
    dev->ops = &enhanced_ops;
else
    dev->ops = &default_ops;
```
- **分析方法**：数据流分析
- **确定性**：列出所有可能值
- **场景化分析**：用户指定条件后可确定

**4. 函数返回**
```c
dev->ops = get_ops(config->type);
```
- **分析方法**：过程间分析（跟踪 get_ops 函数）
- **确定性**：取决于参数
- **难点**：可能有多级调用

**5. 回调链/通知链**
```c
register_netdevice_notifier(&nb);
```
- **分析方法**：识别注册函数 + 关联结构体
- **触发时机**：网络设备事件
- **需要**：内置通知链知识

---

**FlowSight 对各模式的支持**：

| 模式 | 当前支持 | v1.0 目标 |
|------|----------|-----------|
| 直接赋值 | ✅ | ✅ |
| 宏初始化 | ✅ | ✅ |
| 注册函数 | ✅ | ✅ |
| 条件赋值 | ⚠️ 部分 | ✅ 场景化 |
| 函数返回 | ❌ | ⚠️ AI辅助 |
| 回调链 | ⚠️ 部分 | ✅ |
''',
        concepts=["pattern_recognition", "function_pointer", "analysis_difficulty"]
    ),
]

# ============================================================================
# 类型5：多样化问题形式
# ============================================================================

DIVERSE_QUESTION_SAMPLES = [
    # 同一个代码，多种问法
    RealTrainingSample(
        id="diverse_001",
        category="diverse_question",
        difficulty="medium",
        source="drivers/net/",
        code='''
static int my_ndo_open(struct net_device *netdev)
{
    struct my_adapter *adapter = netdev_priv(netdev);
    napi_enable(&adapter->napi);
    netif_start_queue(netdev);
    return 0;
}

static netdev_tx_t my_ndo_start_xmit(struct sk_buff *skb,
                                      struct net_device *netdev)
{
    struct my_adapter *adapter = netdev_priv(netdev);
    /* 发送数据包 */
    return NETDEV_TX_OK;
}

static const struct net_device_ops my_netdev_ops = {
    .ndo_open = my_ndo_open,
    .ndo_stop = my_ndo_stop,
    .ndo_start_xmit = my_ndo_start_xmit,
};

static int my_probe(struct pci_dev *pdev, ...)
{
    struct net_device *netdev = alloc_etherdev(sizeof(*adapter));
    netdev->netdev_ops = &my_netdev_ops;
    register_netdev(netdev);
    return 0;
}
''',
        question="用户在 shell 中执行 `ifconfig eth0 up` 时，内核会调用驱动的哪个函数？",
        thinking='''这是一个从用户命令追踪到内核函数的问题。

1. **用户命令**：ifconfig eth0 up
   - ifconfig 是用户空间工具
   - 设置网络接口为 UP 状态

2. **系统调用**：
   ifconfig 最终调用 ioctl(SIOCSIFFLAGS) 或 netlink

3. **内核处理**：
   - dev_change_flags() 
   - __dev_open()
   - ops->ndo_open()

4. **驱动函数**：
   netdev->netdev_ops = &my_netdev_ops
   my_netdev_ops.ndo_open = my_ndo_open

所以最终调用 my_ndo_open''',
        answer='''当执行 `ifconfig eth0 up` 时，内核会调用 **my_ndo_open** 函数。

**调用链**：
```
用户空间：ifconfig eth0 up
          ↓
系统调用：ioctl(SIOCSIFFLAGS) 或 netlink
          ↓
内核：dev_change_flags()
          ↓
      __dev_open() [net/core/dev.c]
          ↓
      ops->ndo_open(dev)
          ↓
      my_netdev_ops.ndo_open = my_ndo_open
          ↓
驱动：my_ndo_open(netdev)
```

**确定性**：100%''',
        concepts=["network", "ifconfig", "ndo_open"]
    ),
    
    RealTrainingSample(
        id="diverse_002",
        category="diverse_question",
        difficulty="medium",
        source="drivers/net/",
        code='''/* 同上代码 */''',  # 实际使用时复制
        question="当应用程序调用 send() 发送网络数据时，数据包最终通过哪个驱动函数发出？",
        thinking='''追踪 send() 系统调用到驱动函数。

1. send() → sys_sendto()
2. 协议栈处理：TCP/UDP/IP
3. 发送队列：dev_queue_xmit()
4. 驱动调用：ndo_start_xmit()''',
        answer='''应用调用 send() 后，数据包通过 **my_ndo_start_xmit** 发出。

**调用链**：
```
应用：send(sockfd, buf, len, 0)
          ↓
系统调用：sys_sendto() → sock_sendmsg()
          ↓
协议栈：tcp_sendmsg() → tcp_push() → ...
        ip_queue_xmit() → ip_local_out()
          ↓
网络设备：dev_queue_xmit() [net/core/dev.c]
          dev_hard_start_xmit()
          ↓
驱动：ops->ndo_start_xmit(skb, dev)
      = my_ndo_start_xmit(skb, netdev)
```

**注意**：实际路径可能更复杂（有排队、流量控制等）''',
        concepts=["network", "send", "ndo_start_xmit", "protocol_stack"]
    ),
    
    RealTrainingSample(
        id="diverse_003",
        category="diverse_question",
        difficulty="easy",
        source="drivers/net/",
        code='''/* 同上代码 */''',
        question="这个驱动是用什么方式注册 ndo_open 回调的？",
        thinking='''看代码中的绑定方式。

在 my_probe 中：
netdev->netdev_ops = &my_netdev_ops;

my_netdev_ops 是静态初始化的：
.ndo_open = my_ndo_open

这是 "ops 表静态初始化 + 指针赋值" 模式''',
        answer='''这个驱动使用 **ops 表静态初始化** 方式注册回调：

```c
// 1. 静态定义 ops 表
static const struct net_device_ops my_netdev_ops = {
    .ndo_open = my_ndo_open,  // ← 静态初始化
    ...
};

// 2. probe 时赋值给设备
netdev->netdev_ops = &my_netdev_ops;
```

**特点**：
- ops 表是 `const`，不可修改
- 编译时确定所有回调
- 分析难度：简单''',
        concepts=["ops_table", "registration_pattern"]
    ),
]

# ============================================================================
# 数据生成和管理
# ============================================================================

def generate_all_samples() -> List[RealTrainingSample]:
    """收集所有样本"""
    all_samples = []
    all_samples.extend(POINTER_ANALYSIS_SAMPLES)
    all_samples.extend(ASYNC_FLOW_SAMPLES)
    all_samples.extend(CALL_CHAIN_SAMPLES)
    all_samples.extend(PATTERN_RECOGNITION_SAMPLES)
    all_samples.extend(DIVERSE_QUESTION_SAMPLES)
    return all_samples


def save_training_data(samples: List[RealTrainingSample], output_path: str):
    """保存训练数据"""
    output = Path(output_path)
    output.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output, 'w', encoding='utf-8') as f:
        for sample in samples:
            data = sample.to_training_format()
            f.write(json.dumps(data, ensure_ascii=False) + '\n')
    
    print(f"保存 {len(samples)} 个样本到 {output}")


def print_statistics(samples: List[RealTrainingSample]):
    """打印统计信息"""
    print("\n" + "=" * 60)
    print("训练数据统计")
    print("=" * 60)
    
    # 按类别统计
    categories = {}
    difficulties = {}
    
    for s in samples:
        categories[s.category] = categories.get(s.category, 0) + 1
        difficulties[s.difficulty] = difficulties.get(s.difficulty, 0) + 1
    
    print("\n按类别：")
    for cat, count in sorted(categories.items()):
        print(f"  {cat}: {count}")
    
    print("\n按难度：")
    for diff, count in sorted(difficulties.items()):
        print(f"  {diff}: {count}")
    
    print(f"\n总计：{len(samples)} 个样本")
    print("=" * 60)


def main():
    samples = generate_all_samples()
    print_statistics(samples)
    
    # 保存
    save_training_data(samples, "data/real_training_data.jsonl")
    
    print("\n⚠️  警告：这只是示例数据！")
    print("完整训练需要 10000+ 高质量样本")
    print("请：")
    print("  1. 从真实 Linux 内核代码中提取更多样本")
    print("  2. 添加更多多样化的问题形式")
    print("  3. 人工审核和完善推理过程")


if __name__ == "__main__":
    main()

