# Linux 内核知识库开发 Skill

## 概述

本 skill 定义了 Linux 内核知识库 YAML 文件的开发标准和流程。

## 文件结构标准

每个知识库 YAML 文件必须包含以下结构：

```yaml
# {Subsystem Name} Knowledge Base
# Reference: {kernel documentation URL}

# ============================================================
# {COMPONENT NAME}
# ============================================================

component_name:
  description: "组件描述"
  header: "linux/xxx.h"
  icon: "📦"
  kernel_version: "5.0+"  # 适用内核版本

  context:
    type: "process"  # process/softirq/hardirq/atomic
    can_sleep: true
    can_schedule: true
    preemptible: true

  registration:
    - pattern: 'register_xxx\s*\(\s*&(?P<var>\w+)\s*\)'
      description: "注册描述"
    - pattern: 'unregister_xxx\s*\(\s*&(?P<var>\w+)\s*\)'
      description: "注销描述"
    - pattern: 'module_xxx\s*\(\s*(?P<var>\w+)\s*\)'
      description: "模块宏"

  structure:
    pattern: 'static struct xxx\s+(?P<name>\w+)\s*=\s*\{'
    description: "结构体定义"
    fields:
      - name: "field1"
        type: "type1"
        description: "字段描述"

  callbacks:
    callback_name:
      pattern: '\.callback_name\s*=\s*(?P<handler>\w+)'
      description: "回调描述"
      trigger: "触发条件"
      context: "process"
      can_sleep: true
      signature: "int (*)(struct xxx *, ...)"
      return_value:
        success: "0"
        failure: "-ERRNO"
      typical_actions:
        - "典型操作 1"
        - "典型操作 2"

  call_chains:
    chain_name:
      description: "调用链描述"
      trigger: "触发条件"
      chain:
        - function: "entry_function"
          file: "path/to/file.c"
          description: "函数描述"
        - function: "next_function"
          file: "path/to/file.c"
          context: "context_change"
        - function: "user_callback"
          file: "用户代码"
          is_user_entry: true

  helper_functions:
    - pattern: 'helper_func\s*\([^)]*\)'
      description: "辅助函数描述"
      context: "process"

  examples:
    - description: "示例描述"
      code: |
        // 示例代码
        static int my_probe(struct xxx *dev) {
            // 实现
            return 0;
        }
```

## Pattern 编写规范

### 基本规则

1. **使用命名捕获组**
   ```yaml
   # 好
   pattern: 'request_irq\s*\(\s*(?P<irq>\w+)\s*,'
   
   # 不好
   pattern: 'request_irq\s*\(\s*(\w+)\s*,'
   ```

2. **处理可选空白**
   ```yaml
   # 好 - 处理各种空白情况
   pattern: 'func\s*\(\s*(?P<arg>\w+)\s*\)'
   
   # 不好 - 太严格
   pattern: 'func\((?P<arg>\w+)\)'
   ```

3. **匹配指针和引用**
   ```yaml
   # 好 - 匹配 & 和非 & 两种情况
   pattern: 'func\s*\(\s*&?(?P<var>[\w\.\->]+)\s*\)'
   ```

4. **匹配结构体成员**
   ```yaml
   # 好 - 匹配点号和箭头
   pattern: '(?P<var>[\w\.\->]+)'
   ```

### 常用 Pattern 模板

```yaml
# 函数调用
pattern: 'func_name\s*\(\s*(?P<arg1>[^,]+)\s*,\s*(?P<arg2>[^)]+)\s*\)'

# 结构体定义
pattern: 'static struct type_name\s+(?P<name>\w+)\s*=\s*\{'

# 回调赋值
pattern: '\.callback\s*=\s*(?P<handler>\w+)'

# 宏使用
pattern: 'MACRO_NAME\s*\(\s*(?P<arg>\w+)\s*\)'

# 模块宏
pattern: 'module_xxx\s*\(\s*(?P<driver>\w+)\s*\)'
```

## Context 标注规范

| Context | can_sleep | can_schedule | preemptible | 典型场景 |
|---------|-----------|--------------|-------------|----------|
| process | true | true | true | probe, ioctl |
| softirq | false | false | false | tasklet, timer |
| hardirq | false | false | false | 中断处理函数 |
| atomic | false | false | false | 持有 spinlock |
| irq_disabled | false | false | false | 禁用本地中断 |

## 调用链编写规范

```yaml
call_chains:
  xxx_operation:
    description: "操作描述"
    trigger: "触发条件 (如: 用户调用 read())"
    chain:
      # 1. 入口函数
      - function: "ksys_read"
        file: "fs/read_write.c"
        description: "系统调用入口"
        context: "process"
      
      # 2. 中间函数
      - function: "vfs_read"
        file: "fs/read_write.c"
        description: "VFS 层"
      
      # 3. 上下文转换标注
      - function: "xxx_work_handler"
        context: "process (workqueue)"
        description: "上下文从 softirq 转到 process"
      
      # 4. 用户回调
      - function: "my_read"
        file: "用户代码"
        description: "用户实现的读函数"
        is_user_entry: true
```

## 代码示例规范

```yaml
examples:
  - description: "基本驱动骨架"
    kernel_version: "5.0+"
    code: |
      #include <linux/module.h>
      #include <linux/xxx.h>
      
      static int my_probe(struct xxx_device *dev)
      {
          struct my_data *data;
          int ret;
          
          /* 分配私有数据 */
          data = devm_kzalloc(&dev->dev, sizeof(*data), GFP_KERNEL);
          if (!data)
              return -ENOMEM;
          
          /* 初始化 */
          ret = my_init(data);
          if (ret)
              return ret;
          
          dev_set_drvdata(&dev->dev, data);
          return 0;
      }
      
      static void my_remove(struct xxx_device *dev)
      {
          struct my_data *data = dev_get_drvdata(&dev->dev);
          my_cleanup(data);
      }
      
      static struct xxx_driver my_driver = {
          .driver = {
              .name = "my-driver",
          },
          .probe = my_probe,
          .remove = my_remove,
      };
      module_xxx_driver(my_driver);
      
      MODULE_LICENSE("GPL");
      MODULE_DESCRIPTION("My Driver");
```

## 验证流程

### 1. Pattern 验证

```bash
# 在内核源码中测试 pattern
cd /path/to/linux
grep -rP 'your_pattern' --include='*.c' --include='*.h' | head -20
```

### 2. Context 验证

查看内核文档或源码注释确认：
- 函数是否可以睡眠
- 是否持有锁
- 中断状态

### 3. 调用链验证

使用 ftrace 或 perf 验证：
```bash
# ftrace 函数图
echo function_graph > /sys/kernel/debug/tracing/current_tracer
echo your_function > /sys/kernel/debug/tracing/set_graph_function
cat /sys/kernel/debug/tracing/trace
```

## 常见错误

| 错误 | 示例 | 修复 |
|------|------|------|
| Pattern 太宽泛 | `\w+` | 使用更精确的模式 |
| 缺少空白处理 | `func(` | `func\s*\(` |
| Context 错误 | hardirq + can_sleep | 检查文档 |
| 调用链不完整 | 缺少中间步骤 | 阅读源码 |
| 示例不完整 | 缺少错误处理 | 添加检查 |

## 参考资源

- [Kernel API 文档](https://www.kernel.org/doc/html/latest/)
- [驱动开发指南](https://www.kernel.org/doc/html/latest/driver-api/)
- [LWN.net](https://lwn.net/)
