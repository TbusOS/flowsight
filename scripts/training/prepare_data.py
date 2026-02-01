#!/usr/bin/env python3
"""
FlowSight AI 训练数据准备工具

数据来源：
1. FlowSight 知识库自动转换
2. Linux 内核源码分析
3. 开源项目分析
4. LLM 辅助生成
5. 人工标注

使用方法：
    python prepare_data.py --source knowledge --output data/
    python prepare_data.py --source kernel --kernel-path /path/to/linux
    python prepare_data.py --source llm --count 1000
    python prepare_data.py --merge --output data/train.jsonl
"""

import argparse
import json
import os
import re
from pathlib import Path
from typing import List, Dict, Optional
from dataclasses import dataclass, asdict
import subprocess

# ============================================================
# 数据结构定义
# ============================================================

@dataclass
class TrainingSample:
    """训练样本"""
    task: str  # function_pointer_target, callback_timing, async_pattern, call_chain
    instruction: str
    input: str  # 代码
    output: str  # 答案
    metadata: Optional[Dict] = None
    
    def to_dict(self):
        d = {
            "task": self.task,
            "instruction": self.instruction,
            "input": self.input,
            "output": self.output,
        }
        if self.metadata:
            d["metadata"] = self.metadata
        return d

# ============================================================
# 来源1：从 FlowSight 知识库生成
# ============================================================

class KnowledgeBaseConverter:
    """从 FlowSight 知识库转换训练数据"""
    
    # ==========================================================================
    # Linux 内核框架知识（完整版）
    # ==========================================================================
    FRAMEWORKS = {
        # ======================================================================
        # 1. 驱动框架
        # ======================================================================
        "usb_driver": {
            "callbacks": {
                "probe": {
                    "trigger": "USB 设备插入且 ID 匹配",
                    "context": "进程上下文（可睡眠）",
                    "call_chain": [
                        "usb_hub_port_connect (drivers/usb/core/hub.c)",
                        "usb_new_device",
                        "device_add (drivers/base/core.c)",
                        "bus_probe_device",
                        "driver_probe_device (drivers/base/dd.c)",
                        "really_probe",
                        "usb_probe_interface (drivers/usb/core/driver.c)",
                        "drv->probe()"
                    ],
                    "note": "注意：不是 insmod 时调用，而是设备插入后异步调用"
                },
                "disconnect": {
                    "trigger": "USB 设备拔出",
                    "context": "进程上下文（可睡眠）",
                    "call_chain": [
                        "usb_disconnect (drivers/usb/core/hub.c)",
                        "device_del",
                        "bus_remove_device",
                        "drv->disconnect()"
                    ]
                },
                "suspend": {
                    "trigger": "系统休眠或 USB 自动挂起",
                    "context": "进程上下文",
                    "call_chain": ["pm_suspend", "dpm_suspend", "usb_suspend_interface", "drv->suspend()"]
                },
                "resume": {
                    "trigger": "系统唤醒或 USB 自动恢复",
                    "context": "进程上下文",
                    "call_chain": ["pm_resume", "dpm_resume", "usb_resume_interface", "drv->resume()"]
                }
            }
        },
        "platform_driver": {
            "callbacks": {
                "probe": {
                    "trigger": "设备树匹配 / platform_device_register / ACPI",
                    "context": "进程上下文",
                    "call_chain": [
                        "platform_device_add (drivers/base/platform.c)",
                        "device_add",
                        "bus_probe_device",
                        "really_probe (drivers/base/dd.c)",
                        "platform_drv_probe",
                        "drv->probe()"
                    ]
                },
                "remove": {
                    "trigger": "设备移除或模块卸载",
                    "context": "进程上下文",
                    "call_chain": ["platform_device_del", "device_del", "drv->remove()"]
                },
                "suspend": {
                    "trigger": "系统休眠",
                    "context": "进程上下文",
                    "call_chain": ["pm_suspend", "dpm_suspend", "platform_pm_suspend", "drv->suspend()"]
                },
                "resume": {
                    "trigger": "系统唤醒",
                    "context": "进程上下文",
                    "call_chain": ["pm_resume", "dpm_resume", "platform_pm_resume", "drv->resume()"]
                }
            }
        },
        "pci_driver": {
            "callbacks": {
                "probe": {
                    "trigger": "PCI 设备发现且 ID 匹配",
                    "context": "进程上下文",
                    "call_chain": [
                        "pci_device_add (drivers/pci/probe.c)",
                        "device_add",
                        "bus_probe_device",
                        "really_probe",
                        "pci_device_probe (drivers/pci/pci-driver.c)",
                        "drv->probe()"
                    ]
                },
                "remove": {
                    "trigger": "PCI 设备移除或模块卸载",
                    "context": "进程上下文",
                    "call_chain": ["pci_stop_and_remove_bus_device", "device_del", "drv->remove()"]
                }
            }
        },
        "i2c_driver": {
            "callbacks": {
                "probe": {
                    "trigger": "I2C 设备匹配（设备树/ACPI/手动注册）",
                    "context": "进程上下文",
                    "call_chain": [
                        "i2c_device_register (drivers/i2c/i2c-core-base.c)",
                        "device_add",
                        "bus_probe_device",
                        "really_probe",
                        "i2c_device_probe",
                        "drv->probe()"
                    ]
                }
            }
        },
        "spi_driver": {
            "callbacks": {
                "probe": {
                    "trigger": "SPI 设备匹配",
                    "context": "进程上下文",
                    "call_chain": ["spi_add_device", "device_add", "bus_probe_device", "drv->probe()"]
                }
            }
        },
        
        # ======================================================================
        # 2. 文件操作
        # ======================================================================
        "file_operations": {
            "callbacks": {
                "open": {
                    "trigger": "用户调用 open() 系统调用",
                    "context": "进程上下文",
                    "call_chain": [
                        "sys_open / sys_openat (fs/open.c)",
                        "do_sys_open",
                        "do_filp_open",
                        "path_openat",
                        "vfs_open",
                        "do_dentry_open (fs/open.c)",
                        "f_op->open()"
                    ]
                },
                "read": {
                    "trigger": "用户调用 read() 系统调用",
                    "context": "进程上下文",
                    "call_chain": [
                        "sys_read (fs/read_write.c)",
                        "ksys_read",
                        "vfs_read",
                        "f_op->read() / f_op->read_iter()"
                    ]
                },
                "write": {
                    "trigger": "用户调用 write() 系统调用",
                    "context": "进程上下文",
                    "call_chain": [
                        "sys_write (fs/read_write.c)",
                        "ksys_write",
                        "vfs_write",
                        "f_op->write() / f_op->write_iter()"
                    ]
                },
                "unlocked_ioctl": {
                    "trigger": "用户调用 ioctl() 系统调用",
                    "context": "进程上下文",
                    "call_chain": [
                        "sys_ioctl (fs/ioctl.c)",
                        "do_vfs_ioctl",
                        "vfs_ioctl",
                        "f_op->unlocked_ioctl()"
                    ]
                },
                "mmap": {
                    "trigger": "用户调用 mmap() 系统调用",
                    "context": "进程上下文",
                    "call_chain": [
                        "sys_mmap (mm/mmap.c)",
                        "ksys_mmap_pgoff",
                        "vm_mmap_pgoff",
                        "do_mmap",
                        "mmap_region",
                        "call_mmap",
                        "f_op->mmap()"
                    ],
                    "note": "mmap 后首次访问会触发缺页中断"
                },
                "poll": {
                    "trigger": "用户调用 poll()/select()/epoll_wait()",
                    "context": "进程上下文",
                    "call_chain": [
                        "sys_poll / sys_epoll_wait",
                        "do_poll / ep_poll",
                        "vfs_poll",
                        "f_op->poll()"
                    ]
                },
                "release": {
                    "trigger": "文件描述符关闭（最后一个引用）",
                    "context": "进程上下文",
                    "call_chain": [
                        "sys_close (fs/open.c)",
                        "__close_fd",
                        "filp_close",
                        "__fput",
                        "f_op->release()"
                    ]
                },
                "fsync": {
                    "trigger": "用户调用 fsync()/fdatasync()",
                    "context": "进程上下文",
                    "call_chain": ["sys_fsync", "vfs_fsync", "f_op->fsync()"]
                }
            }
        },
        
        # ======================================================================
        # 3. 网络设备操作
        # ======================================================================
        "net_device_ops": {
            "callbacks": {
                "ndo_open": {
                    "trigger": "ifconfig up / ip link set up",
                    "context": "进程上下文",
                    "call_chain": [
                        "dev_open (net/core/dev.c)",
                        "__dev_open",
                        "ops->ndo_open()"
                    ]
                },
                "ndo_stop": {
                    "trigger": "ifconfig down / ip link set down",
                    "context": "进程上下文",
                    "call_chain": ["dev_close", "__dev_close", "ops->ndo_stop()"]
                },
                "ndo_start_xmit": {
                    "trigger": "数据包发送",
                    "context": "软中断上下文或进程上下文",
                    "call_chain": [
                        "send() / sendto() / sendmsg()",
                        "协议栈处理 (TCP/UDP/IP)",
                        "dev_queue_xmit (net/core/dev.c)",
                        "__dev_queue_xmit",
                        "dev_hard_start_xmit",
                        "ops->ndo_start_xmit()"
                    ],
                    "note": "高性能场景可能在软中断中调用"
                },
                "ndo_set_rx_mode": {
                    "trigger": "设置多播/混杂模式",
                    "context": "进程上下文",
                    "call_chain": ["dev_set_rx_mode", "ops->ndo_set_rx_mode()"]
                }
            }
        },
        
        # ======================================================================
        # 4. 块设备操作
        # ======================================================================
        "block_device_operations": {
            "callbacks": {
                "open": {
                    "trigger": "打开块设备",
                    "context": "进程上下文",
                    "call_chain": ["blkdev_open", "bdev_open_by_dev", "ops->open()"]
                },
                "release": {
                    "trigger": "关闭块设备",
                    "context": "进程上下文",
                    "call_chain": ["blkdev_close", "ops->release()"]
                },
                "ioctl": {
                    "trigger": "块设备 ioctl",
                    "context": "进程上下文",
                    "call_chain": ["blkdev_ioctl", "ops->ioctl()"]
                }
            }
        },
        "blk_mq_ops": {
            "callbacks": {
                "queue_rq": {
                    "trigger": "I/O 请求入队",
                    "context": "进程上下文或软中断",
                    "call_chain": [
                        "submit_bio (block/blk-core.c)",
                        "blk_mq_submit_bio",
                        "blk_mq_try_issue_directly / blk_mq_sched_insert_request",
                        "ops->queue_rq()"
                    ]
                },
                "complete": {
                    "trigger": "I/O 请求完成（通常由中断触发）",
                    "context": "中断上下文或软中断",
                    "call_chain": [
                        "硬件中断",
                        "blk_mq_complete_request (block/blk-mq.c)",
                        "ops->complete()"
                    ]
                }
            }
        }
    }
    
    # ==========================================================================
    # 模块加载/卸载调用链
    # ==========================================================================
    MODULE_LIFECYCLE = {
        "insmod": {
            "description": "模块加载",
            "call_chain": [
                "sys_init_module / sys_finit_module (kernel/module.c)",
                "load_module",
                "do_init_module",
                "mod->init() = module_init 指定的函数"
            ],
            "note": "insmod 返回时，module_init 已执行完毕，但 probe 可能还没调用"
        },
        "rmmod": {
            "description": "模块卸载",
            "call_chain": [
                "sys_delete_module (kernel/module.c)",
                "mod->exit() = module_exit 指定的函数",
                "free_module"
            ],
            "note": "rmmod 会先调用 disconnect/remove（如果设备存在），再调用 exit"
        }
    }
    
    # ==========================================================================
    # 内存管理调用链
    # ==========================================================================
    MEMORY_OPERATIONS = {
        "kmalloc": {
            "description": "内核小内存分配",
            "call_chain": [
                "kmalloc (include/linux/slab.h)",
                "__kmalloc",
                "slab_alloc / slub_alloc",
                "从 slab 缓存分配"
            ],
            "context": "进程上下文（GFP_KERNEL）或中断上下文（GFP_ATOMIC）"
        },
        "vmalloc": {
            "description": "虚拟连续内存分配",
            "call_chain": [
                "vmalloc (mm/vmalloc.c)",
                "__vmalloc_node",
                "分配多个物理页",
                "映射到虚拟地址空间"
            ],
            "context": "只能在进程上下文，可能睡眠"
        },
        "page_fault": {
            "description": "缺页中断",
            "call_chain": [
                "CPU 缺页异常",
                "do_page_fault (arch/x86/mm/fault.c)",
                "handle_mm_fault (mm/memory.c)",
                "handle_pte_fault",
                "do_anonymous_page / do_fault",
                "分配物理页并映射"
            ],
            "context": "中断上下文转进程上下文"
        },
        "dma_alloc": {
            "description": "DMA 内存分配",
            "call_chain": [
                "dma_alloc_coherent (kernel/dma/mapping.c)",
                "dma_direct_alloc / iommu_dma_alloc",
                "分配物理连续内存",
                "返回物理地址和虚拟地址"
            ],
            "context": "进程上下文"
        }
    }
    
    # ==========================================================================
    # 进程调度调用链
    # ==========================================================================
    SCHEDULER_OPERATIONS = {
        "timer_interrupt": {
            "description": "时钟中断触发调度检查",
            "call_chain": [
                "时钟中断 (arch/x86/kernel/time.c)",
                "tick_handle_periodic / tick_nohz_handler",
                "update_process_times",
                "scheduler_tick (kernel/sched/core.c)",
                "curr->sched_class->task_tick()",
                "设置 TIF_NEED_RESCHED 标志"
            ]
        },
        "voluntary_schedule": {
            "description": "主动调度（睡眠等待）",
            "call_chain": [
                "schedule() (kernel/sched/core.c)",
                "__schedule",
                "pick_next_task",
                "context_switch",
                "switch_to (arch/x86/kernel/process.c)"
            ]
        },
        "wait_event": {
            "description": "等待事件",
            "call_chain": [
                "wait_event / wait_event_interruptible",
                "prepare_to_wait",
                "schedule()",
                "被唤醒后 finish_wait"
            ]
        },
        "wake_up": {
            "description": "唤醒进程",
            "call_chain": [
                "wake_up / wake_up_interruptible",
                "__wake_up_common",
                "try_to_wake_up (kernel/sched/core.c)",
                "ttwu_queue",
                "设置进程为 TASK_RUNNING"
            ]
        }
    }
    
    # ==========================================================================
    # 网络收包流程
    # ==========================================================================
    NETWORK_RX_FLOW = {
        "traditional_irq": {
            "description": "传统中断收包",
            "call_chain": [
                "网卡中断",
                "do_IRQ (arch/x86/kernel/irq.c)",
                "handle_irq",
                "驱动中断处理函数",
                "netif_rx (net/core/dev.c)",
                "enqueue_to_backlog",
                "NET_RX_SOFTIRQ 软中断",
                "net_rx_action",
                "协议栈处理"
            ]
        },
        "napi_poll": {
            "description": "NAPI 轮询收包（高性能）",
            "call_chain": [
                "网卡中断",
                "驱动中断处理函数",
                "napi_schedule (include/linux/netdevice.h)",
                "禁用中断",
                "NET_RX_SOFTIRQ 软中断",
                "net_rx_action (net/core/dev.c)",
                "napi_poll",
                "驱动 poll 函数",
                "napi_gro_receive",
                "netif_receive_skb",
                "协议栈处理"
            ]
        }
    }
    
    # ==========================================================================
    # 异步机制（完整版）
    # ==========================================================================
    ASYNC_PATTERNS = {
        "workqueue": {
            "bind_funcs": ["INIT_WORK", "INIT_DELAYED_WORK", "DECLARE_WORK"],
            "trigger_funcs": ["schedule_work", "queue_work", "schedule_delayed_work", "queue_delayed_work"],
            "context": "进程上下文（可睡眠）",
            "description": "工作队列，用于延迟执行耗时操作",
            "call_chain": [
                "schedule_work (kernel/workqueue.c)",
                "queue_work_on",
                "insert_work",
                "唤醒 kworker 线程",
                "worker_thread",
                "process_one_work",
                "worker->current_func = work->func",
                "work->func(work)"
            ],
            "typical_use": "中断下半部、延迟初始化、耗时 I/O"
        },
        "timer": {
            "bind_funcs": ["timer_setup", "DEFINE_TIMER", "setup_timer"],
            "trigger_funcs": ["mod_timer", "add_timer", "timer_reduce"],
            "context": "软中断上下文（不可睡眠）",
            "description": "定时器，在指定时间后执行",
            "call_chain": [
                "mod_timer (kernel/time/timer.c)",
                "internal_add_timer",
                "时钟中断",
                "run_timer_softirq",
                "expire_timers",
                "call_timer_fn",
                "timer->function(timer)"
            ],
            "typical_use": "超时处理、周期性任务、看门狗"
        },
        "hrtimer": {
            "bind_funcs": ["hrtimer_init"],
            "trigger_funcs": ["hrtimer_start", "hrtimer_start_range_ns"],
            "context": "硬中断上下文（不可睡眠）",
            "description": "高精度定时器",
            "call_chain": [
                "hrtimer_start (kernel/time/hrtimer.c)",
                "enqueue_hrtimer",
                "高精度时钟中断",
                "hrtimer_interrupt",
                "__hrtimer_run_queues",
                "hrtimer_run_softirq (如果配置)",
                "timer->function(timer)"
            ],
            "typical_use": "纳秒级精度定时、POSIX 定时器"
        },
        "tasklet": {
            "bind_funcs": ["tasklet_init", "tasklet_setup", "DECLARE_TASKLET"],
            "trigger_funcs": ["tasklet_schedule", "tasklet_hi_schedule"],
            "context": "软中断上下文（不可睡眠）",
            "description": "软中断，优先级高于工作队列",
            "call_chain": [
                "tasklet_schedule (include/linux/interrupt.h)",
                "raise_softirq_irqoff(TASKLET_SOFTIRQ)",
                "软中断处理",
                "tasklet_action",
                "tasklet->func(tasklet)"
            ],
            "typical_use": "中断下半部快速处理"
        },
        "irq": {
            "bind_funcs": ["request_irq", "devm_request_irq", "request_threaded_irq"],
            "trigger_funcs": [],  # 硬件触发
            "context": "硬中断上下文（不可睡眠，快速执行）",
            "description": "硬件中断处理",
            "call_chain": [
                "硬件产生中断信号",
                "CPU 响应中断",
                "do_IRQ (arch/x86/kernel/irq.c)",
                "handle_irq",
                "generic_handle_irq",
                "handle_fasteoi_irq / handle_edge_irq",
                "handle_irq_event",
                "action->handler(irq, dev_id)"
            ],
            "typical_use": "硬件事件响应"
        },
        "threaded_irq": {
            "bind_funcs": ["request_threaded_irq", "devm_request_threaded_irq"],
            "trigger_funcs": [],
            "context": "进程上下文（可睡眠）",
            "description": "线程化中断处理",
            "call_chain": [
                "硬件中断 → hardirq handler (快速)",
                "返回 IRQ_WAKE_THREAD",
                "唤醒 irq_thread",
                "irq_thread_fn",
                "action->thread_fn(irq, dev_id)"
            ],
            "typical_use": "需要睡眠的中断处理（如 I2C 通信）"
        },
        "softirq": {
            "bind_funcs": ["open_softirq"],
            "trigger_funcs": ["raise_softirq", "raise_softirq_irqoff"],
            "context": "软中断上下文（不可睡眠）",
            "description": "软中断（最底层机制）",
            "call_chain": [
                "raise_softirq (kernel/softirq.c)",
                "中断返回时检查",
                "irq_exit → invoke_softirq",
                "do_softirq",
                "__do_softirq",
                "softirq_vec[nr].action()"
            ],
            "typical_use": "网络收发、块设备完成"
        },
        "completion": {
            "bind_funcs": ["init_completion", "DECLARE_COMPLETION"],
            "trigger_funcs": ["complete", "complete_all"],
            "wait_funcs": ["wait_for_completion", "wait_for_completion_timeout"],
            "context": "wait 在进程上下文，complete 可在任何上下文",
            "description": "同步等待机制",
            "call_chain": [
                "wait_for_completion (kernel/sched/completion.c)",
                "wait_for_common",
                "schedule()",
                "--- 另一方 ---",
                "complete()",
                "swake_up_locked",
                "唤醒等待者"
            ],
            "typical_use": "等待异步操作完成"
        },
        "waitqueue": {
            "bind_funcs": ["init_waitqueue_head", "DECLARE_WAIT_QUEUE_HEAD"],
            "trigger_funcs": ["wake_up", "wake_up_interruptible", "wake_up_all"],
            "wait_funcs": ["wait_event", "wait_event_interruptible", "wait_event_timeout"],
            "context": "wait 在进程上下文，wake_up 可在任何上下文",
            "description": "等待队列",
            "call_chain": [
                "wait_event (include/linux/wait.h)",
                "prepare_to_wait",
                "设置进程状态为 TASK_INTERRUPTIBLE",
                "schedule()",
                "--- 另一方 ---",
                "wake_up()",
                "__wake_up_common",
                "唤醒等待进程"
            ],
            "typical_use": "等待条件满足"
        },
        "kthread": {
            "bind_funcs": ["kthread_create", "kthread_run"],
            "trigger_funcs": ["wake_up_process"],
            "context": "进程上下文（可睡眠）",
            "description": "内核线程",
            "call_chain": [
                "kthread_create (kernel/kthread.c)",
                "kthread_create_on_node",
                "创建 kthread 结构",
                "唤醒 kthreadd",
                "kthreadd → create_kthread",
                "kernel_thread → kthread",
                "threadfn(data)"
            ],
            "typical_use": "后台服务、周期性任务"
        },
        "rcu": {
            "bind_funcs": ["call_rcu", "synchronize_rcu"],
            "trigger_funcs": [],
            "context": "call_rcu 回调在软中断，synchronize_rcu 在进程上下文",
            "description": "Read-Copy-Update 同步机制",
            "call_chain": [
                "call_rcu (kernel/rcu/tree.c)",
                "注册回调",
                "等待宽限期",
                "rcu_process_callbacks",
                "rcu_do_batch",
                "callback()"
            ],
            "typical_use": "无锁数据结构更新"
        },
        "notifier": {
            "bind_funcs": ["blocking_notifier_chain_register", "atomic_notifier_chain_register"],
            "trigger_funcs": ["blocking_notifier_call_chain", "atomic_notifier_call_chain"],
            "context": "blocking 在进程上下文，atomic 在任何上下文",
            "description": "通知链机制",
            "call_chain": [
                "blocking_notifier_call_chain (kernel/notifier.c)",
                "down_read(&nh->rwsem)",
                "notifier_call_chain",
                "遍历链表调用每个 notifier_block->notifier_call"
            ],
            "typical_use": "内核子系统事件通知"
        }
    }
    
    # ==========================================================================
    # 电源管理调用链
    # ==========================================================================
    POWER_MANAGEMENT = {
        "system_suspend": {
            "description": "系统休眠",
            "call_chain": [
                "echo mem > /sys/power/state",
                "pm_suspend (kernel/power/suspend.c)",
                "enter_state",
                "suspend_prepare",
                "suspend_devices_and_enter",
                "dpm_suspend_start",
                "dpm_suspend",
                "遍历设备调用 dev->driver->pm->suspend()"
            ]
        },
        "system_resume": {
            "description": "系统唤醒",
            "call_chain": [
                "唤醒事件",
                "dpm_resume",
                "遍历设备调用 dev->driver->pm->resume()",
                "resume_finish"
            ]
        },
        "runtime_suspend": {
            "description": "运行时挂起（单个设备）",
            "call_chain": [
                "pm_runtime_put (drivers/base/power/runtime.c)",
                "rpm_idle",
                "rpm_suspend",
                "dev->driver->pm->runtime_suspend()"
            ]
        },
        "runtime_resume": {
            "description": "运行时恢复（单个设备）",
            "call_chain": [
                "pm_runtime_get (drivers/base/power/runtime.c)",
                "rpm_resume",
                "dev->driver->pm->runtime_resume()"
            ]
        }
    }
    
    # ==========================================================================
    # 同步原语
    # ==========================================================================
    SYNCHRONIZATION = {
        "mutex": {
            "description": "互斥锁（可睡眠）",
            "lock_funcs": ["mutex_lock", "mutex_lock_interruptible"],
            "unlock_funcs": ["mutex_unlock"],
            "context": "只能在进程上下文",
            "call_chain_contended": [
                "mutex_lock (kernel/locking/mutex.c)",
                "__mutex_lock",
                "mutex_optimistic_spin (尝试自旋)",
                "失败则 schedule_preempt_disabled",
                "进程睡眠",
                "持有者 unlock 后被唤醒"
            ]
        },
        "spinlock": {
            "description": "自旋锁（不可睡眠）",
            "lock_funcs": ["spin_lock", "spin_lock_irqsave", "spin_lock_bh"],
            "unlock_funcs": ["spin_unlock", "spin_unlock_irqrestore", "spin_unlock_bh"],
            "context": "任何上下文",
            "note": "spin_lock_irqsave 用于中断上下文"
        },
        "rwlock": {
            "description": "读写锁",
            "lock_funcs": ["read_lock", "write_lock"],
            "unlock_funcs": ["read_unlock", "write_unlock"],
            "context": "任何上下文（如用 _irqsave 变体）"
        },
        "semaphore": {
            "description": "信号量（可睡眠）",
            "lock_funcs": ["down", "down_interruptible"],
            "unlock_funcs": ["up"],
            "context": "进程上下文"
        }
    }
    
    def generate_samples(self) -> List[TrainingSample]:
        samples = []
        
        # 1. 生成框架回调样本
        print("  生成驱动框架样本...")
        for framework_name, framework in self.FRAMEWORKS.items():
            for callback_name, callback_info in framework["callbacks"].items():
                samples.extend(self._generate_callback_samples(
                    framework_name, callback_name, callback_info
                ))
        
        # 2. 生成异步模式样本
        print("  生成异步模式样本...")
        for pattern_name, pattern_info in self.ASYNC_PATTERNS.items():
            samples.extend(self._generate_async_samples(pattern_name, pattern_info))
        
        # 3. 生成模块加载/卸载样本
        print("  生成模块生命周期样本...")
        samples.extend(self._generate_module_lifecycle_samples())
        
        # 4. 生成内存管理样本
        print("  生成内存管理样本...")
        samples.extend(self._generate_memory_samples())
        
        # 5. 生成调度器样本
        print("  生成进程调度样本...")
        samples.extend(self._generate_scheduler_samples())
        
        # 6. 生成网络收包样本
        print("  生成网络协议栈样本...")
        samples.extend(self._generate_network_samples())
        
        # 7. 生成电源管理样本
        print("  生成电源管理样本...")
        samples.extend(self._generate_power_samples())
        
        # 8. 生成同步机制样本
        print("  生成同步机制样本...")
        samples.extend(self._generate_sync_samples())
        
        # 9. 生成复杂调用链组合样本
        print("  生成复杂调用链组合样本...")
        samples.extend(self._generate_complex_chain_samples())
        
        return samples
    
    def _generate_module_lifecycle_samples(self) -> List[TrainingSample]:
        """生成模块加载/卸载样本"""
        samples = []
        
        for op_name, op_info in self.MODULE_LIFECYCLE.items():
            code = self._generate_module_code()
            samples.append(TrainingSample(
                task="call_chain",
                instruction=f"分析 {op_name} 命令执行时的内核调用链",
                input=code,
                output=self._format_chain_answer(op_name, op_info),
                metadata={"operation": op_name}
            ))
        
        return samples
    
    def _generate_memory_samples(self) -> List[TrainingSample]:
        """生成内存管理样本"""
        samples = []
        
        for op_name, op_info in self.MEMORY_OPERATIONS.items():
            samples.append(TrainingSample(
                task="call_chain",
                instruction=f"分析 {op_name} 操作的内核调用链",
                input=self._generate_memory_code(op_name),
                output=self._format_memory_answer(op_name, op_info),
                metadata={"operation": op_name}
            ))
        
        return samples
    
    def _generate_scheduler_samples(self) -> List[TrainingSample]:
        """生成调度器样本"""
        samples = []
        
        for op_name, op_info in self.SCHEDULER_OPERATIONS.items():
            samples.append(TrainingSample(
                task="call_chain",
                instruction=f"分析 {op_name} 场景下的调度流程",
                input="// 进程调度相关代码",
                output=self._format_scheduler_answer(op_name, op_info),
                metadata={"operation": op_name}
            ))
        
        return samples
    
    def _generate_network_samples(self) -> List[TrainingSample]:
        """生成网络收包样本"""
        samples = []
        
        for flow_name, flow_info in self.NETWORK_RX_FLOW.items():
            samples.append(TrainingSample(
                task="call_chain",
                instruction=f"分析 {flow_name} 网络收包流程",
                input=self._generate_network_code(flow_name),
                output=self._format_network_answer(flow_name, flow_info),
                metadata={"flow": flow_name}
            ))
        
        return samples
    
    def _generate_power_samples(self) -> List[TrainingSample]:
        """生成电源管理样本"""
        samples = []
        
        for op_name, op_info in self.POWER_MANAGEMENT.items():
            samples.append(TrainingSample(
                task="call_chain",
                instruction=f"分析 {op_name} 电源管理流程",
                input=self._generate_pm_code(op_name),
                output=self._format_power_answer(op_name, op_info),
                metadata={"operation": op_name}
            ))
        
        return samples
    
    def _generate_sync_samples(self) -> List[TrainingSample]:
        """生成同步机制样本"""
        samples = []
        
        for sync_name, sync_info in self.SYNCHRONIZATION.items():
            samples.append(TrainingSample(
                task="sync_mechanism",
                instruction=f"分析 {sync_name} 同步机制的使用方式和注意事项",
                input=self._generate_sync_code(sync_name),
                output=self._format_sync_answer(sync_name, sync_info),
                metadata={"mechanism": sync_name}
            ))
        
        return samples
    
    def _generate_complex_chain_samples(self) -> List[TrainingSample]:
        """生成复杂调用链组合样本"""
        samples = []
        
        # 场景1：完整的驱动生命周期
        samples.append(TrainingSample(
            task="call_chain",
            instruction="分析一个 USB 驱动从 insmod 到设备可用的完整执行流程",
            input=self._generate_full_usb_driver_code(),
            output=self._format_full_usb_lifecycle(),
            metadata={"scenario": "usb_full_lifecycle"}
        ))
        
        # 场景2：中断下半部完整流程
        samples.append(TrainingSample(
            task="call_chain",
            instruction="分析中断触发后，通过 workqueue 处理的完整执行流程",
            input=self._generate_irq_workqueue_code(),
            output=self._format_irq_workqueue_flow(),
            metadata={"scenario": "irq_workqueue"}
        ))
        
        # 场景3：insmod 和 probe 的关系
        samples.append(TrainingSample(
            task="call_chain",
            instruction="解释 insmod 加载模块和 probe 函数调用的关系，它们是同步还是异步？",
            input=self._generate_module_probe_code(),
            output=self._format_insmod_probe_relation(),
            metadata={"scenario": "insmod_probe_relation"}
        ))
        
        # 场景4：缺页中断完整流程
        samples.append(TrainingSample(
            task="call_chain",
            instruction="分析 mmap 后首次访问触发缺页中断的完整流程",
            input=self._generate_mmap_fault_code(),
            output=self._format_mmap_fault_flow(),
            metadata={"scenario": "mmap_page_fault"}
        ))
        
        return samples
    
    # ========== 辅助生成方法 ==========
    
    def _generate_module_code(self) -> str:
        return '''static int __init my_init(void)
{
    printk("Module loaded\\n");
    return usb_register(&my_driver);
}

static void __exit my_exit(void)
{
    usb_deregister(&my_driver);
    printk("Module unloaded\\n");
}

module_init(my_init);
module_exit(my_exit);'''
    
    def _generate_memory_code(self, op_name: str) -> str:
        if op_name == "kmalloc":
            return '''struct my_device *dev = kmalloc(sizeof(*dev), GFP_KERNEL);
if (!dev)
    return -ENOMEM;'''
        elif op_name == "page_fault":
            return '''// 用户空间 mmap 后首次访问
char *buf = mmap(NULL, 4096, PROT_READ|PROT_WRITE, MAP_ANONYMOUS|MAP_PRIVATE, -1, 0);
buf[0] = 'A';  // 触发缺页中断'''
        return ""
    
    def _generate_network_code(self, flow_name: str) -> str:
        if flow_name == "napi_poll":
            return '''static int my_poll(struct napi_struct *napi, int budget)
{
    struct my_device *dev = container_of(napi, struct my_device, napi);
    int done = 0;
    
    while (done < budget) {
        struct sk_buff *skb = my_receive_packet(dev);
        if (!skb)
            break;
        napi_gro_receive(napi, skb);
        done++;
    }
    
    if (done < budget) {
        napi_complete_done(napi, done);
        my_enable_irq(dev);  // 重新使能中断
    }
    return done;
}

static irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    struct my_device *dev = dev_id;
    my_disable_irq(dev);
    napi_schedule(&dev->napi);
    return IRQ_HANDLED;
}'''
        return ""
    
    def _generate_pm_code(self, op_name: str) -> str:
        if "suspend" in op_name:
            return '''static int my_suspend(struct device *dev)
{
    struct my_device *mydev = dev_get_drvdata(dev);
    // 保存状态，停止设备
    my_save_state(mydev);
    my_stop_device(mydev);
    return 0;
}

static SIMPLE_DEV_PM_OPS(my_pm_ops, my_suspend, my_resume);'''
        return ""
    
    def _generate_sync_code(self, sync_name: str) -> str:
        if sync_name == "mutex":
            return '''static DEFINE_MUTEX(my_mutex);

void my_function(void)
{
    mutex_lock(&my_mutex);
    // 临界区，可以睡眠
    msleep(10);
    mutex_unlock(&my_mutex);
}'''
        elif sync_name == "spinlock":
            return '''static DEFINE_SPINLOCK(my_lock);

irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    unsigned long flags;
    spin_lock_irqsave(&my_lock, flags);
    // 临界区，不能睡眠！
    spin_unlock_irqrestore(&my_lock, flags);
    return IRQ_HANDLED;
}'''
        return ""
    
    def _generate_full_usb_driver_code(self) -> str:
        return '''// 完整的 USB 驱动代码
static int my_probe(struct usb_interface *intf, const struct usb_device_id *id)
{
    struct my_device *dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    usb_set_intfdata(intf, dev);
    return 0;
}

static void my_disconnect(struct usb_interface *intf)
{
    struct my_device *dev = usb_get_intfdata(intf);
    kfree(dev);
}

static struct usb_driver my_driver = {
    .name = "my_usb_driver",
    .probe = my_probe,
    .disconnect = my_disconnect,
    .id_table = my_id_table,
};

static int __init my_init(void) { return usb_register(&my_driver); }
static void __exit my_exit(void) { usb_deregister(&my_driver); }

module_init(my_init);
module_exit(my_exit);'''
    
    def _generate_irq_workqueue_code(self) -> str:
        return '''static void my_work_handler(struct work_struct *work)
{
    struct my_device *dev = container_of(work, struct my_device, work);
    // 耗时操作，可以睡眠
    my_process_data(dev);
}

static irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    struct my_device *dev = dev_id;
    // 快速处理
    u32 status = readl(dev->regs + STATUS_REG);
    writel(status, dev->regs + STATUS_REG);  // 清除中断
    schedule_work(&dev->work);  // 延迟处理
    return IRQ_HANDLED;
}'''
    
    def _generate_module_probe_code(self) -> str:
        return '''// insmod 执行时会调用 my_init
static int __init my_init(void)
{
    printk("my_init called\\n");
    return platform_driver_register(&my_driver);  // 注册驱动
}

// probe 函数不是 insmod 时调用的！
static int my_probe(struct platform_device *pdev)
{
    printk("my_probe called\\n");
    return 0;
}'''
    
    def _generate_mmap_fault_code(self) -> str:
        return '''// 驱动实现 mmap
static int my_mmap(struct file *filp, struct vm_area_struct *vma)
{
    // 只是建立映射，不分配物理页
    vma->vm_ops = &my_vm_ops;
    return 0;
}

static vm_fault_t my_fault(struct vm_fault *vmf)
{
    // 缺页时分配物理页
    struct page *page = alloc_page(GFP_KERNEL);
    vmf->page = page;
    return 0;
}

static struct vm_operations_struct my_vm_ops = {
    .fault = my_fault,
};'''
    
    # ========== 格式化答案 ==========
    
    def _format_chain_answer(self, op_name: str, op_info: dict) -> str:
        answer = f"**{op_info['description']}**\n\n"
        answer += "**调用链**：\n"
        for i, func in enumerate(op_info['call_chain']):
            answer += f"  {i+1}. {func}\n"
        if 'note' in op_info:
            answer += f"\n**注意**：{op_info['note']}\n"
        return answer
    
    def _format_memory_answer(self, op_name: str, op_info: dict) -> str:
        answer = f"**{op_info['description']}**\n\n"
        answer += f"**执行上下文**：{op_info['context']}\n\n"
        answer += "**调用链**：\n"
        for func in op_info['call_chain']:
            answer += f"  • {func}\n"
        return answer
    
    def _format_scheduler_answer(self, op_name: str, op_info: dict) -> str:
        answer = f"**{op_info['description']}**\n\n"
        answer += "**调用链**：\n"
        for func in op_info['call_chain']:
            answer += f"  → {func}\n"
        return answer
    
    def _format_network_answer(self, flow_name: str, flow_info: dict) -> str:
        answer = f"**{flow_info['description']}**\n\n"
        answer += "**完整流程**：\n"
        for i, step in enumerate(flow_info['call_chain']):
            answer += f"  {i+1}. {step}\n"
        return answer
    
    def _format_power_answer(self, op_name: str, op_info: dict) -> str:
        answer = f"**{op_info['description']}**\n\n"
        answer += "**调用链**：\n"
        for func in op_info['call_chain']:
            answer += f"  → {func}\n"
        return answer
    
    def _format_sync_answer(self, sync_name: str, sync_info: dict) -> str:
        answer = f"**{sync_info['description']}**\n\n"
        answer += f"**使用上下文**：{sync_info['context']}\n\n"
        if 'note' in sync_info:
            answer += f"**注意**：{sync_info['note']}\n\n"
        if 'call_chain_contended' in sync_info:
            answer += "**竞争时调用链**：\n"
            for func in sync_info['call_chain_contended']:
                answer += f"  → {func}\n"
        return answer
    
    def _format_full_usb_lifecycle(self) -> str:
        return '''**USB 驱动完整生命周期**

═══════════════════════════════════════════════════════════════════════════

**阶段1：模块加载（insmod 命令）**

  执行时机：用户运行 insmod my_driver.ko
  
  调用链：
  1. sys_init_module (kernel/module.c)
  2. load_module
  3. do_init_module
  4. my_init()                    ← 你的初始化函数
  5. usb_register(&my_driver)     ← 注册驱动到 USB 子系统
  6. 返回 0
  7. insmod 命令返回成功
  
  ⚠️ 关键：此时 probe 函数还没有被调用！
           驱动只是注册了，但还没有匹配的设备

═══════════════════════════════════════════════════════════════════════════

**阶段2：设备插入（异步事件）**

  执行时机：USB 设备物理插入
  
  调用链：
  1. USB Hub 检测到端口变化
  2. hub_event (khubd 内核线程)
  3. hub_port_connect_change
  4. usb_hub_port_connect (drivers/usb/core/hub.c)
  5. usb_new_device
  6. device_add (drivers/base/core.c)
  7. bus_probe_device
  8. driver_probe_device
  9. really_probe (drivers/base/dd.c)
  10. usb_probe_interface (drivers/usb/core/driver.c)
  11. my_probe()                   ← 你的 probe 函数
  
  ⚡ 关键：probe 是由内核 khubd 线程异步调用的，
          不是 insmod 同步调用的！

═══════════════════════════════════════════════════════════════════════════

**阶段3：设备使用**

  用户打开设备：
    open("/dev/my_device") → sys_open → vfs_open → f_op->open()
  
  用户读写设备：
    read(fd, ...) → sys_read → vfs_read → f_op->read()
    write(fd, ...) → sys_write → vfs_write → f_op->write()

═══════════════════════════════════════════════════════════════════════════

**阶段4：设备拔出（异步事件）**

  执行时机：USB 设备物理拔出
  
  调用链：
  1. hub_event (khubd 内核线程)
  2. usb_disconnect (drivers/usb/core/hub.c)
  3. device_del
  4. bus_remove_device
  5. my_disconnect()              ← 你的 disconnect 函数

═══════════════════════════════════════════════════════════════════════════

**阶段5：模块卸载（rmmod 命令）**

  调用链：
  1. sys_delete_module (kernel/module.c)
  2. my_exit()                    ← 你的退出函数
  3. usb_deregister(&my_driver)
  4. 如果还有设备连接，先调用 disconnect
  5. free_module

═══════════════════════════════════════════════════════════════════════════

**时间线总结**：

  [T0] insmod           → my_init → usb_register → 返回
       （立即完成，probe 未调用）

  [T1] 设备插入         → (异步) hub 检测 → probe
       （可能是几秒后）

  [T2] 用户使用设备     → open/read/write
       
  [T3] 设备拔出         → (异步) disconnect
  
  [T4] rmmod            → my_exit → usb_deregister → 返回
'''
    
    def _format_irq_workqueue_flow(self) -> str:
        return '''**中断 + WorkQueue 完整执行流程**

═══════════════════════════════════════════════════════════════════════════

**阶段1：硬件中断（上半部）**

  执行上下文：硬中断上下文（不可睡眠，快速执行！）
  
  调用链：
  1. 硬件产生中断信号
  2. CPU 响应中断，保存上下文
  3. do_IRQ (arch/x86/kernel/irq.c)
  4. handle_irq
  5. generic_handle_irq
  6. handle_edge_irq / handle_fasteoi_irq
  7. handle_irq_event
  8. my_irq_handler()             ← 你的中断处理函数
  9. 快速处理（读状态、清中断）
  10. schedule_work(&dev->work)    ← 提交工作到队列
  11. 返回 IRQ_HANDLED
  
  ⚠️ 关键：上半部必须快速完成，不能睡眠！
          schedule_work 只是"提交"，不会执行 handler

═══════════════════════════════════════════════════════════════════════════

**阶段2：软中断 / 工作队列调度（下半部前奏）**

  执行时机：中断返回前或稍后
  
  调用链：
  1. irq_exit (kernel/softirq.c)
  2. invoke_softirq
  3. do_softirq
  4. __do_softirq

═══════════════════════════════════════════════════════════════════════════

**阶段3：工作队列执行（下半部）**

  执行上下文：进程上下文（kworker 线程，可以睡眠）
  
  调用链：
  1. kworker/xxx 线程被调度
  2. worker_thread (kernel/workqueue.c)
  3. process_one_work
  4. worker->current_func = work->func
  5. my_work_handler()            ← 你的工作处理函数
  6. 可以执行耗时操作、睡眠
  
  ✅ 关键：下半部可以睡眠，可以调用可能阻塞的函数

═══════════════════════════════════════════════════════════════════════════

**时间线**：

  [T0] 硬件中断 ─────┬──► my_irq_handler (快速, <100us)
                     │    │
                     │    └──► schedule_work (提交任务)
                     │
  [T1] 中断返回 ─────┘
  
  [T2] kworker 调度 ─────► my_work_handler (可以慢, 可睡眠)
       (异步，可能是 ms 级延迟)

═══════════════════════════════════════════════════════════════════════════

**为什么要分上下半部？**

  1. 中断处理必须快速，否则影响系统响应
  2. 某些操作（I2C通信、申请内存）可能需要睡眠
  3. 上半部：快速响应，清除中断
  4. 下半部：耗时处理，可以睡眠
'''
    
    def _format_insmod_probe_relation(self) -> str:
        return '''**insmod 和 probe 的关系：异步！**

═══════════════════════════════════════════════════════════════════════════

**常见误解** ❌

  很多人以为 insmod 会直接调用 probe，这是错误的！

  错误理解：insmod → my_init → probe → 返回
  
═══════════════════════════════════════════════════════════════════════════

**正确理解** ✅

  insmod 只是注册驱动，probe 是设备匹配时才调用的。
  
  insmod 执行流程：
  1. sys_init_module
  2. load_module
  3. do_init_module
  4. my_init()
  5. platform_driver_register(&my_driver)  // 只是注册
  6. 返回成功
  7. insmod 命令退出
  
  probe 执行流程（异步）：
  - 如果设备已存在（设备树）：insmod 过程中可能触发 probe
  - 如果设备后插入（USB）：设备插入时才触发 probe
  
═══════════════════════════════════════════════════════════════════════════

**不同情况分析**

  **情况1：Platform 驱动 + 设备树**
  
    如果设备树中已有匹配的节点，probe 可能在 insmod 过程中被调用。
    因为 platform_driver_register 会检查已有设备。
    
    但这仍然是"设备匹配触发"，不是"insmod 直接调用"。
  
  **情况2：USB 驱动**
  
    insmod 只注册驱动，probe 不会被调用。
    直到 USB 设备插入且 ID 匹配，probe 才被调用。
    调用者是 khubd 内核线程，不是 insmod 进程。
  
  **情况3：手动创建设备**
  
    如果在 my_init 中调用 platform_device_register，
    可能立即触发 probe（如果驱动已注册）。

═══════════════════════════════════════════════════════════════════════════

**关键结论**

  1. insmod 返回 ≠ probe 已执行
  2. probe 是"设备-驱动匹配"触发的
  3. 调用 probe 的可能是其他内核线程
  4. 对于热插拔设备，probe 和 insmod 是完全异步的
'''
    
    def _format_mmap_fault_flow(self) -> str:
        return '''**mmap + 缺页中断完整流程**

═══════════════════════════════════════════════════════════════════════════

**阶段1：mmap 系统调用**

  执行上下文：进程上下文
  
  用户空间：
    char *buf = mmap(NULL, 4096, PROT_READ|PROT_WRITE, ...);
  
  内核调用链：
  1. sys_mmap (mm/mmap.c)
  2. ksys_mmap_pgoff
  3. vm_mmap_pgoff
  4. do_mmap
  5. mmap_region
  6. call_mmap
  7. my_mmap()                    ← 你的 mmap 实现
  8. 设置 vma->vm_ops = &my_vm_ops
  9. 返回
  
  ⚠️ 关键：此时只是建立了虚拟地址映射，
          没有分配物理页！buf 指向的内存还不存在。

═══════════════════════════════════════════════════════════════════════════

**阶段2：首次访问触发缺页**

  用户空间：
    buf[0] = 'A';  // 写入第一个字节
  
  硬件行为：
  1. CPU 尝试访问 buf 地址
  2. MMU 查页表，发现没有映射
  3. CPU 产生缺页异常 (Page Fault)

═══════════════════════════════════════════════════════════════════════════

**阶段3：缺页中断处理**

  执行上下文：中断上下文 → 进程上下文
  
  调用链：
  1. CPU 缺页异常
  2. do_page_fault (arch/x86/mm/fault.c)
  3. handle_mm_fault (mm/memory.c)
  4. handle_pte_fault
  5. do_fault (因为是文件映射或匿名映射)
  6. vma->vm_ops->fault = my_fault  ← 你的缺页处理
  7. 分配物理页
  8. 建立页表映射
  9. 返回用户空间，重新执行访问指令
  
  ✅ 这次 buf[0] = 'A' 成功了！

═══════════════════════════════════════════════════════════════════════════

**时间线**：

  [T0] mmap()         → 只建立 VMA，不分配物理页
                        返回虚拟地址 buf
  
  [T1] buf[0] = 'A'   → CPU 访问 buf
                      → MMU 缺页
                      → 缺页中断
                      → my_fault()
                      → 分配物理页
                      → 建立映射
                      → 返回用户空间
                      → 重新执行 buf[0] = 'A'
                      → 成功！
  
  [T2] buf[1] = 'B'   → 同一页，已映射，直接成功

  [T3] buf[4096] = 'X' → 新的一页，再次缺页
                       → 再次 my_fault()
                       → ...

═══════════════════════════════════════════════════════════════════════════

**这就是"按需分页"（Demand Paging）**

  - 不预先分配所有物理页
  - 访问时才分配（缺页中断）
  - 节省内存，提高效率
'''
    
    def _generate_callback_samples(self, framework: str, callback: str, info: dict) -> List[TrainingSample]:
        samples = []
        
        # 生成触发时机样本
        code = self._generate_framework_code(framework, callback)
        samples.append(TrainingSample(
            task="callback_timing",
            instruction=f"分析以下代码中 my_{callback} 函数何时被调用",
            input=code,
            output=self._format_callback_answer(framework, callback, info),
            metadata={"framework": framework, "callback": callback}
        ))
        
        # 生成函数指针目标样本
        samples.append(TrainingSample(
            task="function_pointer_target",
            instruction=f"分析 {framework} 结构体中 .{callback} 字段指向哪个函数",
            input=code,
            output=f".{callback} 字段指向 my_{callback} 函数。\n\n这是在结构体初始化时通过 .{callback} = my_{callback} 赋值的。",
            metadata={"framework": framework, "callback": callback}
        ))
        
        return samples
    
    def _generate_framework_code(self, framework: str, callback: str) -> str:
        """生成框架示例代码"""
        if framework == "usb_driver":
            return f'''static int my_{callback}(struct usb_interface *intf, const struct usb_device_id *id)
{{
    printk("my_{callback} called\\n");
    return 0;
}}

static struct usb_driver my_driver = {{
    .name = "my_usb_driver",
    .{callback} = my_{callback},
    .id_table = my_id_table,
}};

module_usb_driver(my_driver);'''
        
        elif framework == "platform_driver":
            return f'''static int my_{callback}(struct platform_device *pdev)
{{
    printk("my_{callback} called\\n");
    return 0;
}}

static struct platform_driver my_driver = {{
    .driver = {{
        .name = "my_platform_driver",
    }},
    .{callback} = my_{callback},
}};

module_platform_driver(my_driver);'''
        
        elif framework == "file_operations":
            return f'''static int my_{callback}(struct inode *inode, struct file *filp)
{{
    printk("my_{callback} called\\n");
    return 0;
}}

static struct file_operations my_fops = {{
    .owner = THIS_MODULE,
    .{callback} = my_{callback},
}};'''
        
        return ""
    
    def _format_callback_answer(self, framework: str, callback: str, info: dict) -> str:
        """格式化回调触发时机答案"""
        answer = f"my_{callback} 函数的触发时机：\n\n"
        answer += f"**触发条件**：{info['trigger']}\n\n"
        answer += f"**执行上下文**：{info['context']}\n\n"
        
        if info['call_chain']:
            answer += "**调用链**：\n"
            for i, func in enumerate(info['call_chain']):
                prefix = "└── " if i == len(info['call_chain']) - 1 else "├── "
                answer += f"{'    ' * i}{prefix}{func}\n"
        
        return answer
    
    def _generate_async_samples(self, pattern: str, info: dict) -> List[TrainingSample]:
        samples = []
        
        code = self._generate_async_code(pattern, info)
        
        samples.append(TrainingSample(
            task="async_pattern",
            instruction="识别以下代码中的异步模式，并解释执行流程",
            input=code,
            output=self._format_async_answer(pattern, info),
            metadata={"pattern": pattern}
        ))
        
        return samples
    
    def _generate_async_code(self, pattern: str, info: dict) -> str:
        """生成异步模式示例代码"""
        if pattern == "workqueue":
            return '''static void my_work_handler(struct work_struct *work)
{
    struct my_device *dev = container_of(work, struct my_device, work);
    // 耗时操作
    printk("Work executed\\n");
}

static int my_probe(struct platform_device *pdev)
{
    struct my_device *dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
    INIT_WORK(&dev->work, my_work_handler);
    return 0;
}

static irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    struct my_device *dev = dev_id;
    schedule_work(&dev->work);
    return IRQ_HANDLED;
}'''
        
        elif pattern == "timer":
            return '''static void my_timer_callback(struct timer_list *t)
{
    struct my_device *dev = from_timer(dev, t, timer);
    printk("Timer expired\\n");
    mod_timer(&dev->timer, jiffies + HZ);  // 重新启动
}

static int my_probe(struct platform_device *pdev)
{
    struct my_device *dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
    timer_setup(&dev->timer, my_timer_callback, 0);
    mod_timer(&dev->timer, jiffies + HZ);  // 1秒后触发
    return 0;
}'''
        
        elif pattern == "tasklet":
            return '''static void my_tasklet_handler(struct tasklet_struct *t)
{
    struct my_device *dev = from_tasklet(dev, t, tasklet);
    printk("Tasklet executed\\n");
}

static int my_probe(struct platform_device *pdev)
{
    struct my_device *dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
    tasklet_setup(&dev->tasklet, my_tasklet_handler);
    return 0;
}

static irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    struct my_device *dev = dev_id;
    tasklet_schedule(&dev->tasklet);
    return IRQ_HANDLED;
}'''
        
        elif pattern == "irq":
            return '''static irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    struct my_device *dev = dev_id;
    // 快速处理，不能睡眠！
    u32 status = readl(dev->regs + STATUS_REG);
    writel(status, dev->regs + STATUS_REG);  // 清除中断
    return IRQ_HANDLED;
}

static int my_probe(struct platform_device *pdev)
{
    struct my_device *dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
    int irq = platform_get_irq(pdev, 0);
    devm_request_irq(&pdev->dev, irq, my_irq_handler, 0, "my_device", dev);
    return 0;
}'''
        
        return ""
    
    def _format_async_answer(self, pattern: str, info: dict) -> str:
        """格式化异步模式答案"""
        answer = f"**异步模式**：{pattern.upper()} ({info['description']})\n\n"
        
        answer += f"**执行上下文**：{info['context']}\n\n"
        
        answer += "**执行流程**：\n"
        if pattern == "workqueue":
            answer += "1. probe 中调用 INIT_WORK 绑定处理函数\n"
            answer += "2. 中断中调用 schedule_work 提交任务\n"
            answer += "3. schedule_work 立即返回，不等待执行\n"
            answer += "4. 内核 kworker 线程稍后调度执行处理函数\n"
            answer += "\n**时间线**：\n"
            answer += "中断发生 → schedule_work → 立即返回 → ... → kworker 调度 → handler 执行\n"
        elif pattern == "timer":
            answer += "1. probe 中调用 timer_setup 绑定回调函数\n"
            answer += "2. mod_timer 设置定时器到期时间\n"
            answer += "3. 定时器到期后，内核调用回调函数\n"
            answer += "\n**注意**：回调在软中断上下文，不能睡眠！\n"
        elif pattern == "tasklet":
            answer += "1. probe 中调用 tasklet_setup 绑定处理函数\n"
            answer += "2. 中断中调用 tasklet_schedule 调度执行\n"
            answer += "3. 中断返回后，软中断上下文执行 tasklet\n"
            answer += "\n**优先级**：高于 workqueue，低于硬中断\n"
        elif pattern == "irq":
            answer += "1. probe 中调用 request_irq 注册中断处理函数\n"
            answer += "2. 硬件触发中断时，CPU 调用处理函数\n"
            answer += "3. 必须快速执行，不能睡眠！\n"
        
        return answer


# ============================================================
# 来源2：Linux 内核源码分析
# ============================================================

class KernelCodeAnalyzer:
    """分析 Linux 内核源码生成训练数据"""
    
    def __init__(self, kernel_path: str):
        self.kernel_path = Path(kernel_path)
    
    def analyze_drivers(self, max_files: int = 100) -> List[TrainingSample]:
        """分析 drivers/ 目录下的代码"""
        samples = []
        
        drivers_path = self.kernel_path / "drivers"
        if not drivers_path.exists():
            print(f"Warning: {drivers_path} not found")
            return samples
        
        # 查找 C 文件
        c_files = list(drivers_path.rglob("*.c"))[:max_files]
        
        for c_file in c_files:
            try:
                samples.extend(self._analyze_file(c_file))
            except Exception as e:
                print(f"Error analyzing {c_file}: {e}")
        
        return samples
    
    def _analyze_file(self, file_path: Path) -> List[TrainingSample]:
        """分析单个文件"""
        samples = []
        
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
        
        # 查找 ops 结构体初始化
        ops_pattern = r'static\s+(?:const\s+)?struct\s+(\w+_operations?|\w+_ops)\s+(\w+)\s*=\s*\{([^}]+)\}'
        
        for match in re.finditer(ops_pattern, content, re.DOTALL):
            struct_type = match.group(1)
            var_name = match.group(2)
            body = match.group(3)
            
            # 提取字段赋值
            field_pattern = r'\.(\w+)\s*=\s*(\w+)'
            fields = re.findall(field_pattern, body)
            
            if fields:
                code_snippet = match.group(0)
                for field_name, func_name in fields:
                    if not func_name.startswith('__'):  # 跳过内部函数
                        samples.append(TrainingSample(
                            task="function_pointer_target",
                            instruction=f"分析 {var_name}.{field_name} 指向哪个函数",
                            input=code_snippet,
                            output=f"{var_name}.{field_name} 指向 {func_name} 函数。\n\n这是通过结构体初始化 .{field_name} = {func_name} 赋值的。",
                            metadata={"file": str(file_path), "struct": struct_type}
                        ))
        
        return samples


# ============================================================
# 来源3：LLM 辅助生成
# ============================================================

class LLMDataGenerator:
    """使用 LLM 辅助生成训练数据"""
    
    PROMPTS = {
        "function_pointer": '''生成一个 C 语言代码片段，包含函数指针的使用，以及对应的分析问答。

要求：
1. 代码要真实、有意义
2. 包含函数指针的定义、赋值、调用
3. 答案要详细解释函数指针指向谁

输出 JSON 格式：
{
  "code": "...",
  "question": "...",
  "answer": "..."
}''',

        "async_pattern": '''生成一个 Linux 内核异步编程的代码片段，以及对应的分析问答。

异步模式可以是：workqueue, timer, tasklet, completion, waitqueue

要求：
1. 代码要符合内核编程规范
2. 展示完整的绑定和触发过程
3. 答案要解释执行流程和时间线

输出 JSON 格式：
{
  "code": "...",
  "pattern": "workqueue|timer|tasklet|...",
  "question": "...",
  "answer": "..."
}'''
    }
    
    def generate_samples(self, count: int = 100) -> List[TrainingSample]:
        """生成训练样本（需要配置 API）"""
        print("LLM 生成需要配置 OpenAI/Claude API")
        print("请手动运行生成脚本或使用其他数据源")
        return []


# ============================================================
# 数据合并和导出
# ============================================================

class DataManager:
    """管理训练数据"""
    
    def __init__(self, output_dir: str = "data"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True, parents=True)
        self.samples: List[TrainingSample] = []
    
    def add_samples(self, samples: List[TrainingSample]):
        """添加样本"""
        self.samples.extend(samples)
        print(f"Added {len(samples)} samples, total: {len(self.samples)}")
    
    def save(self, filename: str = "train.jsonl"):
        """保存为 JSONL 格式"""
        output_path = self.output_dir / filename
        
        with open(output_path, 'w', encoding='utf-8') as f:
            for sample in self.samples:
                # 转换为训练格式
                train_item = {
                    "instruction": sample.instruction,
                    "input": sample.input,
                    "output": sample.output,
                }
                f.write(json.dumps(train_item, ensure_ascii=False) + '\n')
        
        print(f"Saved {len(self.samples)} samples to {output_path}")
    
    def save_by_task(self):
        """按任务类型分别保存"""
        task_samples = {}
        for sample in self.samples:
            task = sample.task
            if task not in task_samples:
                task_samples[task] = []
            task_samples[task].append(sample)
        
        for task, samples in task_samples.items():
            filename = f"{task}.jsonl"
            output_path = self.output_dir / filename
            
            with open(output_path, 'w', encoding='utf-8') as f:
                for sample in samples:
                    f.write(json.dumps(sample.to_dict(), ensure_ascii=False) + '\n')
            
            print(f"Saved {len(samples)} {task} samples to {output_path}")
    
    def statistics(self):
        """打印统计信息"""
        print("\n" + "=" * 50)
        print("数据集统计")
        print("=" * 50)
        
        task_counts = {}
        for sample in self.samples:
            task = sample.task
            task_counts[task] = task_counts.get(task, 0) + 1
        
        for task, count in sorted(task_counts.items()):
            print(f"  {task}: {count}")
        
        print(f"\n  总计: {len(self.samples)} 样本")
        print("=" * 50)


# ============================================================
# 主程序
# ============================================================

def main():
    parser = argparse.ArgumentParser(description="FlowSight 训练数据准备工具")
    parser.add_argument("--source", choices=["knowledge", "kernel", "llm", "all"],
                        default="knowledge", help="数据来源")
    parser.add_argument("--kernel-path", type=str, help="Linux 内核源码路径")
    parser.add_argument("--output", type=str, default="data", help="输出目录")
    parser.add_argument("--merge", action="store_true", help="合并所有数据")
    
    args = parser.parse_args()
    
    manager = DataManager(args.output)
    
    # 来源1：知识库
    if args.source in ["knowledge", "all"]:
        print("\n[1/3] 从知识库生成数据...")
        converter = KnowledgeBaseConverter()
        samples = converter.generate_samples()
        manager.add_samples(samples)
    
    # 来源2：内核源码
    if args.source in ["kernel", "all"] and args.kernel_path:
        print("\n[2/3] 分析内核源码...")
        analyzer = KernelCodeAnalyzer(args.kernel_path)
        samples = analyzer.analyze_drivers()
        manager.add_samples(samples)
    
    # 来源3：LLM 生成
    if args.source in ["llm", "all"]:
        print("\n[3/3] LLM 辅助生成...")
        generator = LLMDataGenerator()
        samples = generator.generate_samples()
        manager.add_samples(samples)
    
    # 保存
    manager.statistics()
    manager.save()
    manager.save_by_task()
    
    print("\n完成！")
    print(f"数据保存在: {args.output}/")
    print("\n下一步:")
    print("  1. 检查生成的数据质量")
    print("  2. 人工补充更多样本")
    print("  3. 运行训练脚本")


if __name__ == "__main__":
    main()

