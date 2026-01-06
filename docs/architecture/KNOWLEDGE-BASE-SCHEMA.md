# 📚 FlowSight 知识库 Schema 设计

> 本文档定义了 FlowSight 知识库的 YAML Schema 规范，用于描述各种编程语言和框架中的异步模式、回调机制和执行流语义。

---

## ⭐ 核心补充：内核调用链 (2025-01 更新)

### 什么是真正的"执行流"

**执行流 ≠ 简单的函数调用关系**

**执行流 = 代码真正是怎么运行的，包括完整的内核调用链！**

例如：很多人以为 `insmod` 时就执行了 `probe`，其实并不是！

```
insmod my_driver.ko
  └── sys_init_module()
        └── do_init_module()
              └── mod->init()
                    └── my_init()
                          └── usb_register(&my_driver)
                                └── return 0

═══════════════════════════════════════
↑ insmod 到这里就返回了！probe 还没执行！
═══════════════════════════════════════

                ... 时间流逝 ...

═══════════════════════════════════════
↓ 某个时刻：USB 设备插入
═══════════════════════════════════════

USB 设备插入
  └── usb_hub_port_connect()
        └── usb_new_device()
              └── device_add()
                    └── bus_probe_device()
                          └── driver_probe_device()
                                └── really_probe()
                                      └── usb_probe_interface()
                                            └── drv->probe()
                                                  └── my_probe()  ← 这才执行！
```

### 内核调用链 Schema

```yaml
# 新增 Schema：完整的内核调用链
kernel_call_chains:
  usb_probe:
    name: "USB probe 调用链"
    trigger_source: "USB 设备插入"
    nodes:
      - function: "usb_hub_port_connect"
        file: "drivers/usb/core/hub.c"
        context: "process"    # process / softirq / hardirq
        description: "USB hub 检测到端口连接"
        is_user_entry: false
      
      - function: "usb_new_device"
        file: "drivers/usb/core/hub.c"
        context: "process"
        is_user_entry: false
      
      # ... 更多节点 ...
      
      - function: "drv->probe()"
        file: null            # 用户代码
        context: "process"
        description: "调用驱动的 probe 回调"
        is_user_entry: true   # ⭐ 这是用户代码入口点
```

### 异步时间线 Schema

```yaml
# 展示两条执行流之间的关系
async_timelines:
  irq_to_workqueue:
    name: "中断 + WorkQueue 异步时间线"
    
    phase1:
      name: "中断上半部"
      context: "hardirq"
      call_chain:
        - function: "do_IRQ"
          file: "arch/x86/kernel/irq.c"
        - function: "handle_irq"
        - function: "my_irq_handler"
          is_user_entry: true
    
    separation: "中断返回 → CPU 执行其他任务 → 调度器选择 kworker"
    
    phase2:
      name: "WorkQueue 执行"
      context: "process"
      call_chain:
        - function: "worker_thread"
          file: "kernel/workqueue.c"
        - function: "process_one_work"
        - function: "my_work_handler"
          is_user_entry: true
```

---

## 目录

1. [设计原则](#1-设计原则)
2. [Schema 总览](#2-schema-总览)
3. [异步模式 Schema](#3-异步模式-schema)
4. [框架回调 Schema](#4-框架回调-schema)
5. [类型映射 Schema](#5-类型映射-schema)
6. [跨语言桥接 Schema](#6-跨语言桥接-schema)
7. [验证与工具](#7-验证与工具)
8. [最佳实践](#8-最佳实践)

---

## 1. 设计原则

### 1.1 核心理念

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        知识库设计原则                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 声明式而非命令式                                                     │
│     • 描述"是什么"而非"怎么做"                                          │
│     • 分析引擎负责解释执行                                               │
│                                                                          │
│  2. 可组合性                                                             │
│     • 小粒度的模式可以组合成复杂行为                                     │
│     • 支持继承和引用                                                     │
│                                                                          │
│  3. 渐进式精确                                                           │
│     • 简单模式可以只定义正则                                             │
│     • 复杂模式可以添加类型约束、上下文条件                               │
│                                                                          │
│  4. 语言无关的核心 + 语言特定的扩展                                      │
│     • 核心概念（绑定、触发、回调）是通用的                               │
│     • 具体语法和模式是语言特定的                                         │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 知识库文件组织

```
knowledge/
├── patterns/                    # 异步模式定义
│   ├── _base.yaml              # 基础模式（所有语言通用）
│   ├── c_async.yaml            # C 语言异步模式
│   ├── cpp_async.yaml          # C++ 异步模式
│   ├── java_async.yaml         # Java 异步模式
│   ├── kotlin_async.yaml       # Kotlin 协程模式
│   ├── rust_async.yaml         # Rust async 模式
│   └── go_async.yaml           # Go goroutine 模式
│
├── frameworks/                  # 框架知识
│   ├── linux_kernel/           # Linux 内核
│   │   ├── usb.yaml
│   │   ├── pci.yaml
│   │   ├── netdev.yaml
│   │   └── block.yaml
│   ├── android/                # Android 系统
│   │   ├── activity.yaml
│   │   ├── binder.yaml
│   │   └── hal.yaml
│   ├── spring/                 # Spring 框架
│   │   ├── mvc.yaml
│   │   └── boot.yaml
│   └── node/                   # Node.js
│       ├── express.yaml
│       └── events.yaml
│
├── bridges/                     # 跨语言桥接
│   ├── jni.yaml                # Java ↔ C/C++
│   ├── cgo.yaml                # Go ↔ C
│   ├── pyo3.yaml               # Python ↔ Rust
│   └── napi.yaml               # Node.js ↔ C++
│
└── types/                       # 类型映射
    ├── c_types.yaml
    ├── java_types.yaml
    └── common_types.yaml
```

---

## 2. Schema 总览

### 2.1 根节点结构

```yaml
# 每个知识库文件的顶层结构
$schema: "https://flowsight.dev/schemas/knowledge-v1.json"
version: "1.0"
language: "c"                    # 目标语言
domain: "linux_kernel"           # 领域（可选）

# 元数据
metadata:
  name: "Linux Work Queue Patterns"
  description: "Work queue async patterns for Linux kernel"
  author: "FlowSight Team"
  last_updated: "2025-01-04"
  references:
    - "https://www.kernel.org/doc/html/latest/core-api/workqueue.html"

# 模式定义（以下任选其一或多个）
async_patterns: { ... }
framework_callbacks: { ... }
type_mappings: { ... }
bridge_definitions: { ... }
```

### 2.2 通用字段类型

```yaml
# 位置信息
Location:
  file: string                   # 文件路径
  line: integer                  # 行号
  column: integer                # 列号
  offset: integer                # 字节偏移

# 正则模式（带命名捕获组）
Pattern:
  pattern: string                # 正则表达式
  flags: string                  # 可选：i(忽略大小写), m(多行), s(dotall)
  captures:                      # 捕获组说明
    handler: "回调函数名"
    var: "绑定的变量"

# 类型签名
TypeSignature:
  return_type: string
  parameters:
    - name: string
      type: string
      direction: "in" | "out" | "inout"

# 上下文条件
ContextCondition:
  execution_context: "process" | "interrupt" | "softirq" | "any"
  can_sleep: boolean
  holds_lock: string             # 持有的锁名称（可选）
  requires_rcu: boolean          # 是否在 RCU 读取区
```

---

## 3. 异步模式 Schema

异步模式是知识库的核心，描述"绑定"和"触发"的关系。

### 3.1 完整 Schema 定义

```yaml
async_patterns:
  <pattern_name>:                # 模式唯一标识符
    # === 基本信息 ===
    description: string          # 人类可读描述
    category: string             # 分类：timer, workqueue, irq, thread, ipc
    icon: string                 # 用于 UI 显示的图标
    
    # === 执行上下文 ===
    context:
      type: "process" | "softirq" | "hardirq" | "any"
      can_sleep: boolean
      can_schedule: boolean
      preemptible: boolean
    
    # === 绑定模式 ===
    bind_patterns:
      - pattern: string          # 正则表达式，必须有命名捕获组
        handler_capture: string  # 捕获回调函数的组名，默认 "handler"
        variable_capture: string # 捕获绑定变量的组名，默认 "var"
        scope: "local" | "global" | "struct_field"
        
        # 可选：类型约束
        type_constraints:
          handler_type: string   # 回调函数必须匹配的类型签名
          variable_type: string  # 变量必须匹配的类型
        
        # 可选：额外捕获
        extra_captures:
          - name: string
            description: string
    
    # === 触发模式 ===
    trigger_patterns:
      - pattern: string
        variable_capture: string
        
        # 可选：触发条件
        condition:
          requires_bind: boolean # 是否必须先绑定
          timing: "immediate" | "deferred" | "periodic"
        
        # 可选：传递给 handler 的参数映射
        argument_mapping:
          - from: "captured_var"
            to: "handler_param_0"
    
    # === 取消/解绑模式 ===
    cancel_patterns:
      - pattern: string
        variable_capture: string
        behavior: "sync" | "async" | "trycancel"
    
    # === Handler 签名 ===
    handler_signature:
      return_type: string
      parameters:
        - name: string
          type: string
          source: string         # 参数来源：bind_time, trigger_time, context
    
    # === 生命周期钩子 ===
    lifecycle:
      init_required: boolean     # 使用前是否必须初始化
      cleanup_required: boolean  # 必须显式清理
      ref_counted: boolean       # 是否引用计数
    
    # === 关联模式 ===
    related_patterns:
      - name: string             # 关联的其他模式
        relationship: "extends" | "uses" | "conflicts"
```

### 3.2 示例：完整的 work_struct 定义

```yaml
async_patterns:
  work_struct:
    description: "Linux Kernel Work Queue"
    category: "workqueue"
    icon: "⚙️"
    
    context:
      type: "process"
      can_sleep: true
      can_schedule: true
      preemptible: true
    
    bind_patterns:
      # 标准初始化宏
      - pattern: 'INIT_WORK\s*\(\s*&?(?P<var>[\w\.\->]+)\s*,\s*(?P<handler>\w+)\s*\)'
        handler_capture: "handler"
        variable_capture: "var"
        scope: "struct_field"
        type_constraints:
          handler_type: "void (*)(struct work_struct *)"
          variable_type: "struct work_struct"
      
      # 栈上初始化
      - pattern: 'INIT_WORK_ONSTACK\s*\(\s*&?(?P<var>[\w\.\->]+)\s*,\s*(?P<handler>\w+)\s*\)'
        handler_capture: "handler"
        variable_capture: "var"
        scope: "local"
        type_constraints:
          handler_type: "void (*)(struct work_struct *)"
      
      # 声明时初始化
      - pattern: 'DECLARE_WORK\s*\(\s*(?P<var>\w+)\s*,\s*(?P<handler>\w+)\s*\)'
        handler_capture: "handler"
        variable_capture: "var"
        scope: "global"
    
    trigger_patterns:
      # 默认工作队列
      - pattern: 'schedule_work\s*\(\s*&?(?P<var>[\w\.\->]+)\s*\)'
        variable_capture: "var"
        condition:
          requires_bind: true
          timing: "deferred"
      
      # 指定工作队列
      - pattern: 'queue_work\s*\(\s*(?P<wq>[\w\.\->]+)\s*,\s*&?(?P<var>[\w\.\->]+)\s*\)'
        variable_capture: "var"
        extra_captures:
          - name: "wq"
            description: "目标工作队列"
        condition:
          requires_bind: true
          timing: "deferred"
      
      # 在特定 CPU 上执行
      - pattern: 'queue_work_on\s*\(\s*(?P<cpu>\d+|[\w]+)\s*,\s*(?P<wq>[\w\.\->]+)\s*,\s*&?(?P<var>[\w\.\->]+)\s*\)'
        variable_capture: "var"
        extra_captures:
          - name: "cpu"
            description: "目标 CPU"
          - name: "wq"
            description: "目标工作队列"
    
    cancel_patterns:
      - pattern: 'cancel_work_sync\s*\(\s*&?(?P<var>[\w\.\->]+)\s*\)'
        variable_capture: "var"
        behavior: "sync"
      
      - pattern: 'cancel_work\s*\(\s*&?(?P<var>[\w\.\->]+)\s*\)'
        variable_capture: "var"
        behavior: "trycancel"
    
    handler_signature:
      return_type: "void"
      parameters:
        - name: "work"
          type: "struct work_struct *"
          source: "bind_time"
    
    lifecycle:
      init_required: true
      cleanup_required: true     # 必须 cancel 后才能释放
      ref_counted: false
    
    related_patterns:
      - name: "delayed_work"
        relationship: "extends"
      - name: "rcu_work"
        relationship: "extends"
```

### 3.3 简化版 Schema（快速定义）

对于简单模式，可以使用简化语法：

```yaml
async_patterns:
  # 最小化定义
  simple_timer:
    bind: 'setup_timer\s*\(&?(?P<var>\w+),\s*(?P<handler>\w+)'
    trigger: 'mod_timer\s*\(&?(?P<var>\w+)'
    context: "softirq"
    
  # 系统会自动展开为完整格式
```

---

## 4. 框架回调 Schema

框架回调描述特定框架中的回调接口和生命周期。

### 4.1 完整 Schema 定义

```yaml
framework_callbacks:
  <framework_name>:
    description: string
    header: string               # 头文件路径
    documentation: string        # 文档链接
    
    # 注册方式
    registration:
      functions:                 # 注册函数列表
        - name: string
          pattern: string        # 可选，如果名称不够精确
      macros:                    # 注册宏列表
        - name: string
    
    # 核心数据结构
    core_struct:
      name: string               # 结构体名称
      id_field: string           # 标识字段（如 name）
      ops_field: string          # 操作表字段
    
    # 回调定义
    callbacks:
      <callback_name>:
        # 基本信息
        description: string
        field_path: string       # 在结构体中的路径，如 "ops.probe"
        
        # 触发条件
        trigger:
          event: string          # 触发事件描述
          timing: "sync" | "async" | "deferred"
          initiator: "system" | "user" | "hardware" | "driver"
        
        # 执行上下文
        context:
          type: "process" | "interrupt" | "any"
          can_sleep: boolean
          holds_locks: [string]
        
        # 函数签名
        signature:
          return_type: string
          parameters:
            - name: string
              type: string
              description: string
          
          return_values:
            success: string      # 成功返回值
            failure: string      # 失败返回值
        
        # 典型实现
        typical_actions: [string]
        
        # 生命周期位置
        lifecycle_phase: "init" | "runtime" | "shutdown"
        
        # 调用顺序约束
        ordering:
          must_before: [string]  # 必须在这些回调之前
          must_after: [string]   # 必须在这些回调之后
          may_call: [string]     # 可能调用的其他回调
```

### 4.2 示例：USB 驱动框架

```yaml
framework_callbacks:
  usb_driver:
    description: "USB Device Driver Framework"
    header: "linux/usb.h"
    documentation: "https://www.kernel.org/doc/html/latest/driver-api/usb/"
    
    registration:
      functions:
        - name: "usb_register"
        - name: "usb_register_driver"
      macros:
        - name: "module_usb_driver"
    
    core_struct:
      name: "struct usb_driver"
      id_field: "name"
      ops_field: null            # 回调直接在结构体中
    
    callbacks:
      probe:
        description: "当匹配的 USB 设备插入时调用"
        field_path: "probe"
        
        trigger:
          event: "USB 设备插入且 ID 匹配"
          timing: "sync"
          initiator: "system"
        
        context:
          type: "process"
          can_sleep: true
          holds_locks: []
        
        signature:
          return_type: "int"
          parameters:
            - name: "interface"
              type: "struct usb_interface *"
              description: "USB 接口对象"
            - name: "id"
              type: "const struct usb_device_id *"
              description: "匹配的设备 ID"
          
          return_values:
            success: "0"
            failure: "-ENOMEM, -ENODEV, etc."
        
        typical_actions:
          - "分配设备私有数据"
          - "初始化 USB 端点"
          - "提交初始 URB"
          - "注册字符设备或网络接口"
        
        lifecycle_phase: "init"
        
        ordering:
          must_before: ["disconnect"]
          must_after: []
          may_call: []
      
      disconnect:
        description: "当 USB 设备断开时调用"
        field_path: "disconnect"
        
        trigger:
          event: "USB 设备移除或驱动卸载"
          timing: "sync"
          initiator: "system"
        
        context:
          type: "process"
          can_sleep: true
          holds_locks: []
        
        signature:
          return_type: "void"
          parameters:
            - name: "interface"
              type: "struct usb_interface *"
              description: "USB 接口对象"
        
        typical_actions:
          - "取消待处理的 URB"
          - "释放资源"
          - "注销子设备"
          - "释放私有数据"
        
        lifecycle_phase: "shutdown"
        
        ordering:
          must_before: []
          must_after: ["probe"]
          may_call: []
      
      suspend:
        description: "设备挂起时调用"
        field_path: "suspend"
        
        trigger:
          event: "系统休眠或 USB 自动挂起"
          timing: "sync"
          initiator: "system"
        
        context:
          type: "process"
          can_sleep: true
        
        signature:
          return_type: "int"
          parameters:
            - name: "interface"
              type: "struct usb_interface *"
            - name: "message"
              type: "pm_message_t"
        
        lifecycle_phase: "runtime"
        
        ordering:
          may_call: ["resume"]
      
      resume:
        description: "设备恢复时调用"
        field_path: "resume"
        
        trigger:
          event: "系统唤醒或 USB 自动恢复"
          timing: "sync"
          initiator: "system"
        
        context:
          type: "process"
          can_sleep: true
        
        signature:
          return_type: "int"
          parameters:
            - name: "interface"
              type: "struct usb_interface *"
        
        lifecycle_phase: "runtime"
```

---

## 5. 类型映射 Schema

类型映射用于理解函数指针类型和回调签名。

### 5.1 Schema 定义

```yaml
type_mappings:
  # 函数指针类型
  function_pointers:
    <type_name>:
      pattern: string            # 类型定义的正则模式
      signature:
        return_type: string
        parameters: [string]
      common_uses: [string]      # 常见用途
  
  # 结构体类型
  structs:
    <struct_name>:
      fields:
        <field_name>:
          type: string
          is_callback: boolean
          callback_type: string  # 如果是回调，指向 function_pointers 中的类型
  
  # 类型别名
  typedefs:
    <alias>: <original_type>
```

### 5.2 示例：Linux 内核常用类型

```yaml
type_mappings:
  function_pointers:
    work_func_t:
      pattern: 'typedef\s+void\s+\(\*work_func_t\)\s*\(struct\s+work_struct\s*\*\)'
      signature:
        return_type: "void"
        parameters: ["struct work_struct *"]
      common_uses:
        - "work_struct.func"
        - "INIT_WORK 第二参数"
    
    irq_handler_t:
      pattern: 'typedef\s+irqreturn_t\s+\(\*irq_handler_t\)'
      signature:
        return_type: "irqreturn_t"
        parameters: ["int", "void *"]
      common_uses:
        - "request_irq 第二参数"
        - "devm_request_irq 第三参数"
    
    timer_callback_t:
      pattern: 'void\s+\(\*\)\s*\(struct\s+timer_list\s*\*\)'
      signature:
        return_type: "void"
        parameters: ["struct timer_list *"]
      common_uses:
        - "timer_setup 第二参数"
  
  structs:
    work_struct:
      fields:
        func:
          type: "work_func_t"
          is_callback: true
          callback_type: "work_func_t"
        data:
          type: "atomic_long_t"
          is_callback: false
    
    file_operations:
      fields:
        owner:
          type: "struct module *"
          is_callback: false
        read:
          type: "ssize_t (*)(struct file *, char __user *, size_t, loff_t *)"
          is_callback: true
        write:
          type: "ssize_t (*)(struct file *, const char __user *, size_t, loff_t *)"
          is_callback: true
        open:
          type: "int (*)(struct inode *, struct file *)"
          is_callback: true
        release:
          type: "int (*)(struct inode *, struct file *)"
          is_callback: true
        unlocked_ioctl:
          type: "long (*)(struct file *, unsigned int, unsigned long)"
          is_callback: true
        mmap:
          type: "int (*)(struct file *, struct vm_area_struct *)"
          is_callback: true
  
  typedefs:
    irqreturn_t: "enum irqreturn"
    pm_message_t: "struct pm_message"
    loff_t: "long long"
    size_t: "unsigned long"
    ssize_t: "long"
```

---

## 6. 跨语言桥接 Schema

跨语言桥接描述不同语言间的调用关系。

### 6.1 Schema 定义

```yaml
bridge_definitions:
  <bridge_name>:
    description: string
    source_language: string      # 调用方语言
    target_language: string      # 被调用方语言
    
    # 函数命名约定
    naming_conventions:
      - source_pattern: string   # 源语言中的命名模式
        target_pattern: string   # 目标语言中的命名模式
        transform: string        # 转换规则
    
    # 类型映射
    type_mapping:
      <source_type>: <target_type>
    
    # 调用模式
    call_patterns:
      - description: string
        source_pattern: string   # 源语言调用模式
        target_resolution: string # 如何找到目标函数
    
    # 回调注册模式
    callback_patterns:
      - description: string
        register_pattern: string # 注册回调的模式
        callback_direction: "source_to_target" | "target_to_source"
```

### 6.2 示例：JNI 桥接

```yaml
bridge_definitions:
  jni:
    description: "Java Native Interface"
    source_language: "java"
    target_language: "c"
    
    naming_conventions:
      # Java native 方法 → C 函数
      - source_pattern: 'native\s+(?P<ret>\w+)\s+(?P<name>\w+)\s*\('
        target_pattern: 'Java_(?P<class>[\w_]+)_(?P<name>\w+)'
        transform: |
          将 Java 类名中的 '.' 替换为 '_'
          将方法名保持不变
          示例: com.example.MyClass.doSomething 
                → Java_com_example_MyClass_doSomething
      
      # 简短命名（当方法名唯一时）
      - source_pattern: 'native\s+(?P<ret>\w+)\s+(?P<name>\w+)\s*\('
        target_pattern: '(?P<name>\w+)'
        transform: "动态注册时可使用简短名称"
    
    type_mapping:
      # Java → JNI → C
      "boolean": "jboolean"
      "byte": "jbyte"
      "char": "jchar"
      "short": "jshort"
      "int": "jint"
      "long": "jlong"
      "float": "jfloat"
      "double": "jdouble"
      "void": "void"
      "String": "jstring"
      "Object": "jobject"
      "byte[]": "jbyteArray"
      "int[]": "jintArray"
    
    call_patterns:
      # 静态注册
      - description: "静态 JNI 注册"
        source_pattern: 'native\s+\w+\s+(?P<method>\w+)\s*\([^)]*\)'
        target_resolution: |
          查找函数: Java_{包名}_{类名}_{方法名}
          包名中的 '.' 替换为 '_'
      
      # 动态注册
      - description: "动态 JNI 注册 (RegisterNatives)"
        source_pattern: 'RegisterNatives\s*\([^,]+,\s*(?P<methods>\w+)'
        target_resolution: |
          解析 JNINativeMethod 数组
          数组格式: { "javaName", "signature", function_ptr }
    
    callback_patterns:
      # Java 调用 Native
      - description: "Java → Native 调用"
        register_pattern: 'native\s+\w+\s+(?P<method>\w+)'
        callback_direction: "source_to_target"
      
      # Native 回调 Java
      - description: "Native → Java 回调"
        register_pattern: 'GetMethodID\s*\([^,]+,\s*"(?P<method>\w+)"'
        callback_direction: "target_to_source"
```

### 6.3 示例：CGO 桥接

```yaml
bridge_definitions:
  cgo:
    description: "Go ↔ C Interop"
    source_language: "go"
    target_language: "c"
    
    naming_conventions:
      - source_pattern: '//export\s+(?P<name>\w+)'
        target_pattern: '(?P<name>\w+)'
        transform: "Go 函数导出为同名 C 函数"
    
    type_mapping:
      "C.int": "int"
      "C.char": "char"
      "C.long": "long"
      "*C.char": "char *"
      "unsafe.Pointer": "void *"
    
    call_patterns:
      # Go 调用 C
      - description: "Go 调用 C 函数"
        source_pattern: 'C\.(?P<func>\w+)\s*\('
        target_resolution: "直接查找同名 C 函数"
      
      # C 调用 Go
      - description: "C 调用 Go 导出函数"
        source_pattern: '//export\s+(?P<func>\w+)'
        target_resolution: "查找 //export 注释标记的 Go 函数"
```

---

## 7. 验证与工具

### 7.1 Schema 验证

```yaml
# 使用 JSON Schema 进行验证
$schema: "https://flowsight.dev/schemas/knowledge-v1.json"

# 验证规则：
validation_rules:
  # 1. 每个 pattern 必须是有效正则表达式
  patterns:
    - must_compile: true
    - must_have_capture_groups: ["handler", "var"]  # 至少其一
  
  # 2. 签名必须完整
  signatures:
    - return_type: required
    - parameters: required
  
  # 3. 引用必须存在
  references:
    - related_patterns: must_exist_in_same_file
    - callback_type: must_exist_in_type_mappings
```

### 7.2 验证工具命令

```bash
# 验证知识库文件
flowsight kb validate knowledge/patterns/c_async.yaml

# 验证所有文件
flowsight kb validate-all

# 测试模式匹配
flowsight kb test-pattern \
  --pattern 'INIT_WORK\s*\(&?(?P<var>\w+),\s*(?P<handler>\w+)\)' \
  --input 'INIT_WORK(&dev->work, my_handler);'

# 生成文档
flowsight kb docs --output docs/knowledge/
```

### 7.3 知识库开发工具

```yaml
# .flowsight/kb-dev.yaml - 开发时配置
development:
  # 热重载
  watch: true
  watch_paths:
    - "knowledge/"
  
  # 测试用例
  test_fixtures:
    - path: "tests/fixtures/"
      auto_test: true
  
  # 调试模式
  debug:
    show_match_details: true
    show_capture_groups: true
    trace_resolution: true
```

---

## 8. 最佳实践

### 8.1 编写模式的原则

```yaml
# ✅ 好的模式：精确且有命名捕获组
good_pattern:
  pattern: 'INIT_WORK\s*\(\s*&?(?P<var>[\w\.\->]+)\s*,\s*(?P<handler>\w+)\s*\)'
  # 解释：
  # - \s* 允许空格变化
  # - &? 可选的取地址符
  # - (?P<var>...) 命名捕获组
  # - [\w\.\->]+ 匹配各种变量访问形式

# ❌ 不好的模式：太宽泛
bad_pattern:
  pattern: 'INIT_WORK.*'  # 没有捕获组，匹配太多
```

### 8.2 组织结构建议

```yaml
# 1. 按领域分文件，不要把所有东西放一个文件
# 2. 使用有意义的文件名
# 3. 添加足够的注释和文档链接

# 文件头模板
# ============================================================
# FlowSight Knowledge Base
# 
# Domain: Linux Kernel - Work Queues
# Language: C
# 
# References:
#   - https://www.kernel.org/doc/html/latest/core-api/workqueue.html
#   - include/linux/workqueue.h
# 
# Last updated: 2025-01-04
# ============================================================
```

### 8.3 测试覆盖

```yaml
# 每个模式都应该有测试用例
test_cases:
  work_struct:
    bind_patterns:
      - input: "INIT_WORK(&dev->work, my_handler);"
        expected:
          var: "dev->work"
          handler: "my_handler"
      
      - input: "INIT_WORK(&work, handler);"
        expected:
          var: "work"
          handler: "handler"
      
      - input: "INIT_WORK_ONSTACK(&local_work, local_handler);"
        expected:
          var: "local_work"
          handler: "local_handler"
    
    trigger_patterns:
      - input: "schedule_work(&dev->work);"
        expected:
          var: "dev->work"
```

---

## 附录：完整 JSON Schema

完整的 JSON Schema 定义文件位于：`schemas/knowledge-v1.json`

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://flowsight.dev/schemas/knowledge-v1.json",
  "title": "FlowSight Knowledge Base Schema",
  "type": "object",
  "properties": {
    "version": { "type": "string" },
    "language": { "type": "string" },
    "domain": { "type": "string" },
    "metadata": { "$ref": "#/definitions/Metadata" },
    "async_patterns": { "$ref": "#/definitions/AsyncPatterns" },
    "framework_callbacks": { "$ref": "#/definitions/FrameworkCallbacks" },
    "type_mappings": { "$ref": "#/definitions/TypeMappings" },
    "bridge_definitions": { "$ref": "#/definitions/BridgeDefinitions" }
  }
}
```

---

*文档版本: 1.0*
*最后更新: 2025-01-04*
*作者: FlowSight Team*

