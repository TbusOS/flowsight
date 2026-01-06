//! FlowSight Knowledge Base
//!
//! Contains framework patterns, API information, and **complete kernel call chains**.
//!
//! ## 核心理念
//! 
//! **执行流 = 代码真正怎么运行的完整调用链**
//! 
//! 不是简单的"触发描述"，而是完整的内核函数调用路径！
//! 
//! 例如 USB probe 的调用链：
//! ```text
//! [USB 设备插入]
//!   └── usb_hub_port_connect()
//!         └── usb_new_device()
//!               └── device_add()
//!                     └── bus_probe_device()
//!                           └── __device_attach()
//!                                 └── driver_probe_device()
//!                                       └── really_probe()
//!                                             └── usb_probe_interface()
//!                                                   └── drv->probe()
//!                                                         └── [用户的 probe 函数]
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 调用链中的一个节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallChainNode {
    /// 函数名
    pub function: String,
    /// 所属文件 (内核源码路径)
    pub file: Option<String>,
    /// 执行上下文
    pub context: ExecutionContext,
    /// 节点说明
    pub description: Option<String>,
    /// 是否是用户代码入口点 (如 drv->probe 就是用户代码入口)
    pub is_user_entry: bool,
}

/// 完整的调用链
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallChain {
    /// 调用链名称 (如 "USB probe 调用链")
    pub name: String,
    /// 触发源头 (如 "USB 设备插入")
    pub trigger_source: String,
    /// 调用链节点 (从触发源到用户代码)
    pub nodes: Vec<CallChainNode>,
}

/// 异步时间线关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncTimeline {
    /// 时间线名称 (如 "中断 + WorkQueue 异步关系")
    pub name: String,
    /// 第一阶段 (如 "中断上半部")
    pub phase1: TimelinePhase,
    /// 分割说明 (如 "中断返回后，由调度器决定何时执行")
    pub separation: String,
    /// 第二阶段 (如 "WorkQueue 执行")
    pub phase2: TimelinePhase,
}

/// 时间线阶段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePhase {
    /// 阶段名称
    pub name: String,
    /// 执行上下文
    pub context: ExecutionContext,
    /// 该阶段的调用链
    pub call_chain: CallChain,
}

/// 执行上下文
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionContext {
    /// 进程上下文 (可睡眠)
    Process,
    /// 软中断上下文 (不可睡眠)
    SoftIrq,
    /// 硬中断上下文 (不可睡眠，最严格)
    HardIrq,
    /// 用户空间
    User,
    /// 未知
    Unknown,
}

impl ExecutionContext {
    pub fn can_sleep(&self) -> bool {
        matches!(self, ExecutionContext::Process | ExecutionContext::User)
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            ExecutionContext::Process => "进程上下文 (可睡眠)",
            ExecutionContext::SoftIrq => "软中断上下文 (不可睡眠)",
            ExecutionContext::HardIrq => "硬中断上下文 (不可睡眠)",
            ExecutionContext::User => "用户空间",
            ExecutionContext::Unknown => "未知上下文",
        }
    }
}

/// Framework callback information - 增强版
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkCallback {
    /// Callback description
    pub description: String,
    /// When the callback is triggered (文字描述)
    pub trigger: String,
    /// Execution context
    pub context: ExecutionContext,
    /// Function signature
    pub signature: Option<String>,
    /// ⭐ 完整的内核调用链！
    pub call_chain: Option<CallChain>,
}

/// Framework definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Framework {
    /// Framework description
    pub description: String,
    /// Header file
    pub header: Option<String>,
    /// Callbacks defined by this framework
    pub callbacks: HashMap<String, FrameworkCallback>,
}

/// Async pattern definition - 增强版
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncPattern {
    /// Description
    pub description: String,
    /// Execution context
    pub context: ExecutionContext,
    /// Bind patterns (regex)
    pub bind_patterns: Vec<String>,
    /// Trigger patterns (regex)
    pub trigger_patterns: Vec<String>,
    /// Handler signature
    pub handler_signature: Option<String>,
    /// ⭐ 异步时间线关系
    pub timeline: Option<AsyncTimeline>,
    /// ⭐ Handler 被调用时的完整内核调用链
    pub handler_call_chain: Option<CallChain>,
}

/// Kernel API information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelApi {
    /// Description
    pub description: String,
    /// Whether it can sleep
    pub can_sleep: bool,
    /// Whether it can fail
    pub can_fail: bool,
    /// Parameter descriptions
    pub params: Option<Vec<String>>,
}

/// Knowledge base
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// Framework definitions
    pub frameworks: HashMap<String, Framework>,
    /// Async patterns
    pub async_patterns: HashMap<String, AsyncPattern>,
    /// Kernel API info
    pub kernel_apis: HashMap<String, KernelApi>,
}

impl KnowledgeBase {
    /// Create an empty knowledge base
    pub fn new() -> Self {
        Self::default()
    }

    /// Load knowledge base from YAML file
    pub fn load_yaml(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let kb: KnowledgeBase = serde_yaml::from_str(&content)?;
        Ok(kb)
    }

    /// Load knowledge base from JSON file
    pub fn load_json(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let kb: KnowledgeBase = serde_json::from_str(&content)?;
        Ok(kb)
    }

    /// Load built-in knowledge base
    pub fn builtin() -> Self {
        let mut kb = Self::new();
        kb.load_builtin_frameworks();
        kb.load_builtin_apis();
        kb
    }

    fn load_builtin_frameworks(&mut self) {
        // USB driver framework - 带完整调用链
        let mut usb_callbacks = HashMap::new();
        
        // USB probe 完整调用链
        let usb_probe_chain = CallChain {
            name: "USB probe 调用链".into(),
            trigger_source: "USB 设备插入".into(),
            nodes: vec![
                CallChainNode {
                    function: "usb_hub_port_connect".into(),
                    file: Some("drivers/usb/core/hub.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("USB hub 检测到端口连接".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "usb_new_device".into(),
                    file: Some("drivers/usb/core/hub.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("创建新 USB 设备".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "device_add".into(),
                    file: Some("drivers/base/core.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("添加设备到设备模型".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "bus_probe_device".into(),
                    file: Some("drivers/base/bus.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("总线层探测设备".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "__device_attach".into(),
                    file: Some("drivers/base/dd.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("尝试将设备与驱动匹配".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "driver_probe_device".into(),
                    file: Some("drivers/base/dd.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("驱动探测设备".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "really_probe".into(),
                    file: Some("drivers/base/dd.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("实际执行探测".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "usb_probe_interface".into(),
                    file: Some("drivers/usb/core/driver.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("USB 接口探测".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "drv->probe()".into(),
                    file: None,
                    context: ExecutionContext::Process,
                    description: Some("调用驱动的 probe 回调".into()),
                    is_user_entry: true, // ⭐ 这是用户代码入口
                },
            ],
        };
        
        usb_callbacks.insert(
            "probe".into(),
            FrameworkCallback {
                description: "Called when a USB device matching the ID table is connected".into(),
                trigger: "USB 设备插入并且 ID 匹配".into(),
                context: ExecutionContext::Process,
                signature: Some(
                    "int (*)(struct usb_interface *, const struct usb_device_id *)".into(),
                ),
                call_chain: Some(usb_probe_chain),
            },
        );
        
        // USB disconnect 调用链
        let usb_disconnect_chain = CallChain {
            name: "USB disconnect 调用链".into(),
            trigger_source: "USB 设备拔出或驱动卸载".into(),
            nodes: vec![
                CallChainNode {
                    function: "usb_hub_port_disconnect".into(),
                    file: Some("drivers/usb/core/hub.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("USB hub 检测到端口断开".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "usb_disconnect".into(),
                    file: Some("drivers/usb/core/hub.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("USB 断开处理".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "device_del".into(),
                    file: Some("drivers/base/core.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("从设备模型删除".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "__device_release_driver".into(),
                    file: Some("drivers/base/dd.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("释放驱动".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "usb_unbind_interface".into(),
                    file: Some("drivers/usb/core/driver.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("解绑 USB 接口".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "drv->disconnect()".into(),
                    file: None,
                    context: ExecutionContext::Process,
                    description: Some("调用驱动的 disconnect 回调".into()),
                    is_user_entry: true,
                },
            ],
        };
        
        usb_callbacks.insert(
            "disconnect".into(),
            FrameworkCallback {
                description: "Called when the USB device is disconnected".into(),
                trigger: "USB 设备拔出或驱动卸载".into(),
                context: ExecutionContext::Process,
                signature: Some("void (*)(struct usb_interface *)".into()),
                call_chain: Some(usb_disconnect_chain),
            },
        );

        self.frameworks.insert(
            "usb_driver".into(),
            Framework {
                description: "USB device driver framework".into(),
                header: Some("linux/usb.h".into()),
                callbacks: usb_callbacks,
            },
        );

        // file_operations - 带调用链
        let mut fops_callbacks = HashMap::new();
        
        let fops_open_chain = CallChain {
            name: "file open 调用链".into(),
            trigger_source: "用户空间 open() 系统调用".into(),
            nodes: vec![
                CallChainNode {
                    function: "sys_open / sys_openat".into(),
                    file: Some("fs/open.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("系统调用入口".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "do_sys_open".into(),
                    file: Some("fs/open.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("处理 open 系统调用".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "do_filp_open".into(),
                    file: Some("fs/namei.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("打开文件路径".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "vfs_open".into(),
                    file: Some("fs/open.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("VFS 层打开".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "do_dentry_open".into(),
                    file: Some("fs/open.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("打开目录项".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "f->f_op->open()".into(),
                    file: None,
                    context: ExecutionContext::Process,
                    description: Some("调用文件操作的 open".into()),
                    is_user_entry: true,
                },
            ],
        };
        
        fops_callbacks.insert(
            "open".into(),
            FrameworkCallback {
                description: "Called when the device is opened".into(),
                trigger: "用户空间 open() 系统调用".into(),
                context: ExecutionContext::Process,
                signature: Some("int (*)(struct inode *, struct file *)".into()),
                call_chain: Some(fops_open_chain),
            },
        );
        fops_callbacks.insert(
            "read".into(),
            FrameworkCallback {
                description: "Called to read from the device".into(),
                trigger: "用户空间 read() 系统调用".into(),
                context: ExecutionContext::Process,
                signature: Some(
                    "ssize_t (*)(struct file *, char __user *, size_t, loff_t *)".into(),
                ),
                call_chain: None, // 可以后续添加
            },
        );
        fops_callbacks.insert(
            "write".into(),
            FrameworkCallback {
                description: "Called to write to the device".into(),
                trigger: "用户空间 write() 系统调用".into(),
                context: ExecutionContext::Process,
                signature: Some(
                    "ssize_t (*)(struct file *, const char __user *, size_t, loff_t *)".into(),
                ),
                call_chain: None,
            },
        );

        self.frameworks.insert(
            "file_operations".into(),
            Framework {
                description: "Character device file operations".into(),
                header: Some("linux/fs.h".into()),
                callbacks: fops_callbacks,
            },
        );
        
        // ⭐ 添加 WorkQueue 异步调用链
        self.load_builtin_async_patterns();
    }
    
    /// 加载内置的异步模式调用链
    fn load_builtin_async_patterns(&mut self) {
        // WorkQueue 调用链
        let workqueue_handler_chain = CallChain {
            name: "WorkQueue handler 调用链".into(),
            trigger_source: "内核 kworker 线程被调度".into(),
            nodes: vec![
                CallChainNode {
                    function: "kworker/xxx (内核线程)".into(),
                    file: Some("kernel/workqueue.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("工作队列内核线程".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "worker_thread".into(),
                    file: Some("kernel/workqueue.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("工作线程主循环".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "process_one_work".into(),
                    file: Some("kernel/workqueue.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("处理单个工作项".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "worker->current_func".into(),
                    file: Some("kernel/workqueue.c".into()),
                    context: ExecutionContext::Process,
                    description: Some("调用工作函数".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "work->func()".into(),
                    file: None,
                    context: ExecutionContext::Process,
                    description: Some("用户的工作处理函数".into()),
                    is_user_entry: true,
                },
            ],
        };
        
        // WorkQueue 时间线
        let workqueue_timeline = AsyncTimeline {
            name: "中断 + WorkQueue 异步时间线".into(),
            phase1: TimelinePhase {
                name: "中断上半部".into(),
                context: ExecutionContext::HardIrq,
                call_chain: CallChain {
                    name: "中断处理".into(),
                    trigger_source: "硬件中断".into(),
                    nodes: vec![
                        CallChainNode {
                            function: "do_IRQ".into(),
                            file: Some("arch/x86/kernel/irq.c".into()),
                            context: ExecutionContext::HardIrq,
                            description: Some("中断入口".into()),
                            is_user_entry: false,
                        },
                        CallChainNode {
                            function: "handle_irq".into(),
                            file: Some("kernel/irq/handle.c".into()),
                            context: ExecutionContext::HardIrq,
                            description: Some("处理 IRQ".into()),
                            is_user_entry: false,
                        },
                        CallChainNode {
                            function: "irq_handler()".into(),
                            file: None,
                            context: ExecutionContext::HardIrq,
                            description: Some("用户的中断处理函数".into()),
                            is_user_entry: true,
                        },
                    ],
                },
            },
            separation: "中断返回 → CPU 可能执行其他任务 → 调度器选择 kworker 执行".into(),
            phase2: TimelinePhase {
                name: "WorkQueue 执行".into(),
                context: ExecutionContext::Process,
                call_chain: workqueue_handler_chain.clone(),
            },
        };
        
        self.async_patterns.insert(
            "work_struct".into(),
            AsyncPattern {
                description: "工作队列异步执行机制".into(),
                context: ExecutionContext::Process,
                bind_patterns: vec![
                    r"INIT_WORK\s*\(\s*&?\s*(\w+(?:->\w+)*)\s*,\s*(\w+)\s*\)".into(),
                    r"INIT_DELAYED_WORK\s*\(\s*&?\s*(\w+(?:->\w+)*)\s*,\s*(\w+)\s*\)".into(),
                ],
                trigger_patterns: vec![
                    r"queue_work\s*\(".into(),
                    r"schedule_work\s*\(".into(),
                    r"queue_delayed_work\s*\(".into(),
                ],
                handler_signature: Some("void (*)(struct work_struct *)".into()),
                timeline: Some(workqueue_timeline),
                handler_call_chain: Some(workqueue_handler_chain),
            },
        );
        
        // Timer 调用链
        let timer_handler_chain = CallChain {
            name: "Timer handler 调用链".into(),
            trigger_source: "定时器到期".into(),
            nodes: vec![
                CallChainNode {
                    function: "timer interrupt (时钟中断)".into(),
                    file: Some("kernel/time/timer.c".into()),
                    context: ExecutionContext::SoftIrq,
                    description: Some("时钟中断触发".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "run_timer_softirq".into(),
                    file: Some("kernel/time/timer.c".into()),
                    context: ExecutionContext::SoftIrq,
                    description: Some("定时器软中断".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "__run_timers".into(),
                    file: Some("kernel/time/timer.c".into()),
                    context: ExecutionContext::SoftIrq,
                    description: Some("运行到期的定时器".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "call_timer_fn".into(),
                    file: Some("kernel/time/timer.c".into()),
                    context: ExecutionContext::SoftIrq,
                    description: Some("调用定时器函数".into()),
                    is_user_entry: false,
                },
                CallChainNode {
                    function: "timer->function()".into(),
                    file: None,
                    context: ExecutionContext::SoftIrq,
                    description: Some("用户的定时器回调 (软中断上下文，不可睡眠!)".into()),
                    is_user_entry: true,
                },
            ],
        };
        
        self.async_patterns.insert(
            "timer_list".into(),
            AsyncPattern {
                description: "内核定时器机制".into(),
                context: ExecutionContext::SoftIrq, // 注意：软中断上下文！
                bind_patterns: vec![
                    r"timer_setup\s*\(\s*&?\s*(\w+(?:->\w+)*)\s*,\s*(\w+)\s*,".into(),
                    r"setup_timer\s*\(\s*&?\s*(\w+(?:->\w+)*)\s*,\s*(\w+)\s*,".into(),
                ],
                trigger_patterns: vec![
                    r"mod_timer\s*\(".into(),
                    r"add_timer\s*\(".into(),
                ],
                handler_signature: Some("void (*)(struct timer_list *)".into()),
                timeline: None,
                handler_call_chain: Some(timer_handler_chain),
            },
        );
    }

    fn load_builtin_apis(&mut self) {
        self.kernel_apis.insert(
            "kzalloc".into(),
            KernelApi {
                description: "Allocate zeroed kernel memory".into(),
                can_sleep: true,
                can_fail: true,
                params: Some(vec!["size".into(), "gfp_flags".into()]),
            },
        );

        self.kernel_apis.insert(
            "kfree".into(),
            KernelApi {
                description: "Free kernel memory".into(),
                can_sleep: false,
                can_fail: false,
                params: Some(vec!["ptr".into()]),
            },
        );

        self.kernel_apis.insert(
            "mutex_lock".into(),
            KernelApi {
                description: "Acquire a mutex".into(),
                can_sleep: true,
                can_fail: false,
                params: Some(vec!["mutex".into()]),
            },
        );

        self.kernel_apis.insert(
            "spin_lock".into(),
            KernelApi {
                description: "Acquire a spinlock".into(),
                can_sleep: false,
                can_fail: false,
                params: Some(vec!["lock".into()]),
            },
        );

        self.kernel_apis.insert(
            "printk".into(),
            KernelApi {
                description: "Print a kernel message".into(),
                can_sleep: false,
                can_fail: false,
                params: Some(vec!["fmt".into(), "...".into()]),
            },
        );
    }

    /// Get framework info
    pub fn get_framework(&self, name: &str) -> Option<&Framework> {
        self.frameworks.get(name)
    }

    /// Get callback info
    pub fn get_callback(&self, framework: &str, callback: &str) -> Option<&FrameworkCallback> {
        self.frameworks.get(framework)?.callbacks.get(callback)
    }

    /// Get API info
    pub fn get_api(&self, name: &str) -> Option<&KernelApi> {
        self.kernel_apis.get(name)
    }
    
    /// ⭐ 获取框架回调的完整内核调用链
    pub fn get_callback_call_chain(&self, framework: &str, callback: &str) -> Option<&CallChain> {
        self.get_callback(framework, callback)?.call_chain.as_ref()
    }
    
    /// ⭐ 获取异步模式的 handler 调用链
    pub fn get_async_handler_chain(&self, pattern_name: &str) -> Option<&CallChain> {
        self.async_patterns.get(pattern_name)?.handler_call_chain.as_ref()
    }
    
    /// ⭐ 获取异步模式的时间线关系
    pub fn get_async_timeline(&self, pattern_name: &str) -> Option<&AsyncTimeline> {
        self.async_patterns.get(pattern_name)?.timeline.as_ref()
    }
    
    /// ⭐ 根据函数名查找其所属的框架和回调类型
    /// 例如：my_probe 函数可能被识别为 usb_driver 的 probe 回调
    pub fn identify_callback(&self, function_name: &str, code_context: &str) -> Option<(&str, &str, &FrameworkCallback)> {
        // 检查代码上下文中是否有框架注册
        for (fw_name, framework) in &self.frameworks {
            for (cb_name, callback) in &framework.callbacks {
                // 简单匹配：函数名包含回调名，或代码中有赋值
                if function_name.contains(cb_name) {
                    return Some((fw_name, cb_name, callback));
                }
                // 检查是否是 ops 表赋值
                let pattern = format!(r"\.{}\s*=\s*{}", cb_name, function_name);
                if regex::Regex::new(&pattern).ok()?.is_match(code_context) {
                    return Some((fw_name, cb_name, callback));
                }
            }
        }
        None
    }
    
    /// ⭐ 获取异步模式信息
    pub fn get_async_pattern(&self, name: &str) -> Option<&AsyncPattern> {
        self.async_patterns.get(name)
    }
}
