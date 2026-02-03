# 异步机制分析指南

本指南深入介绍如何使用 FlowSight 分析 Linux 内核中的异步执行流。

## 为什么需要异步分析？

Linux 内核大量使用异步机制来处理并发和延迟执行：

```c
// 传统 IDE 在这里就断了 ❌
INIT_WORK(&dev->work, my_handler);    // 绑定
schedule_work(&dev->work);             // 触发 → 谁被调用？

request_irq(irq, irq_handler, ...);    // 注册 → 何时执行？

static struct file_operations fops = {
    .read = my_read,                    // 赋值 → 谁调用 .read？
};
```

**FlowSight 通过理解异步语义来解决这个问题** ✅

---

## WorkQueue 分析

### 基本工作队列

```c
// 绑定阶段
INIT_WORK(&dev->work, my_work_handler);

// 触发阶段（可在中断上下文）
schedule_work(&dev->work);

// 执行阶段（进程上下文）
void my_work_handler(struct work_struct *work)
{
    // 可以睡眠
    msleep(100);
}
```

**FlowSight 执行流**：

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  irq_handler │ ──→ │ schedule_work│ ──→ │my_work_handler│
│  [硬中断]    │     │  [触发]      │     │  [进程]       │
└──────────────┘     └──────────────┘     └──────────────┘
                            ▲
                            │ 异步边界
                            │ (WorkQueue)
```

### 延迟工作队列

```c
// 绑定
INIT_DELAYED_WORK(&dev->dwork, my_delayed_handler);

// 触发（100ms 后执行）
schedule_delayed_work(&dev->dwork, msecs_to_jiffies(100));
```

**FlowSight 识别**：
- 🔗 绑定：`INIT_DELAYED_WORK` → `my_delayed_handler`
- ⏰ 延迟：100ms
- 📍 上下文：进程（可睡眠）

### 自定义工作队列

```c
// 创建队列
my_wq = alloc_workqueue("my_driver_wq", WQ_UNBOUND | WQ_MEM_RECLAIM, 0);

// 调度到自定义队列
queue_work(my_wq, &dev->work);
```

**FlowSight 显示**：
- 队列名称：`my_driver_wq`
- 队列标志：`WQ_UNBOUND`, `WQ_MEM_RECLAIM`
- 最大并发：默认 (256)

---

## Timer 分析

### 标准定时器

```c
// 绑定（新 API，4.15+）
timer_setup(&dev->timer, my_timer_callback, 0);

// 触发
mod_timer(&dev->timer, jiffies + HZ);  // 1 秒后

// 执行（软中断上下文！）
void my_timer_callback(struct timer_list *t)
{
    // 不能睡眠！
    struct my_device *dev = from_timer(dev, t, timer);
}
```

**FlowSight 警告**：
```
⚠️ 定时器回调在软中断上下文执行
   不能调用: msleep, mutex_lock, kmalloc(GFP_KERNEL)
```

### 高精度定时器

```c
// 绑定
hrtimer_init(&dev->hr_timer, CLOCK_MONOTONIC, HRTIMER_MODE_REL);
dev->hr_timer.function = my_hrtimer_callback;

// 触发
hrtimer_start(&dev->hr_timer, ktime_set(0, 1000000), HRTIMER_MODE_REL);  // 1ms

// 执行（硬中断或软中断上下文）
enum hrtimer_restart my_hrtimer_callback(struct hrtimer *timer)
{
    // 绝对不能睡眠！
    return HRTIMER_NORESTART;
}
```

---

## IRQ 分析

### 基本中断处理

```c
// 注册
request_irq(irq, my_irq_handler, IRQF_SHARED, "my_driver", dev);

// 处理（硬中断上下文）
irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    // 快速处理，不能睡眠
    return IRQ_HANDLED;
}
```

### 线程化中断

```c
// 注册（推荐方式）
request_threaded_irq(irq,
                     my_hard_handler,    // 硬中断部分
                     my_thread_handler,  // 线程部分
                     IRQF_SHARED,
                     "my_driver",
                     dev);

// 硬中断处理（快速）
irqreturn_t my_hard_handler(int irq, void *dev_id)
{
    // 只做必要的确认
    return IRQ_WAKE_THREAD;
}

// 线程处理（可睡眠）
irqreturn_t my_thread_handler(int irq, void *dev_id)
{
    // 可以睡眠，处理耗时操作
    msleep(10);
    return IRQ_HANDLED;
}
```

**FlowSight 执行流**：

```
硬件中断
    │
    ▼
┌────────────────┐
│ my_hard_handler│ ← 硬中断上下文
│  [快速确认]    │
└───────┬────────┘
        │ IRQ_WAKE_THREAD
        ▼
┌─────────────────┐
│my_thread_handler│ ← 进程上下文
│  [耗时处理]     │
└─────────────────┘
```

---

## Tasklet 分析

```c
// 绑定
tasklet_init(&dev->tasklet, my_tasklet_handler, (unsigned long)dev);

// 触发（可在硬中断中）
tasklet_schedule(&dev->tasklet);

// 执行（软中断上下文）
void my_tasklet_handler(unsigned long data)
{
    struct my_device *dev = (struct my_device *)data;
    // 不能睡眠！
}
```

**FlowSight 标注**：
- 📍 上下文：软中断
- ⚠️ 约束：不能睡眠、不能被抢占
- 💡 建议：考虑使用 WorkQueue 替代（更灵活）

---

## 函数指针解析

### ops 表模式

```c
static const struct file_operations my_fops = {
    .owner = THIS_MODULE,
    .open = my_open,
    .read = my_read,
    .write = my_write,
    .release = my_release,
};

// 注册
misc_register(&my_miscdev);
```

**FlowSight 解析**：

```
用户空间 open()
    │
    ▼
┌────────────────┐
│ sys_open       │
│    ...         │
│ misc_open      │
└───────┬────────┘
        │ fops->open
        ▼
┌────────────────┐
│ my_open        │ ← FlowSight 解析到这里
└────────────────┘
```

### 驱动 ops 表

```c
static const struct usb_driver my_usb_driver = {
    .name = "my_driver",
    .probe = my_probe,
    .disconnect = my_disconnect,
    .suspend = my_suspend,
    .resume = my_resume,
};
```

FlowSight 自动关联：
- USB 设备匹配 → `my_probe`
- 设备断开 → `my_disconnect`
- 系统挂起 → `my_suspend`
- 系统恢复 → `my_resume`

---

## 分析最佳实践

### 1. 从入口点开始

选择驱动的入口函数开始分析：
- `module_init` 函数
- `probe` 函数
- `open` 函数

### 2. 关注异步边界

FlowSight 用虚线标记异步边界：

```
同步部分 ───────→ 异步边界 - - - → 异步部分
```

### 3. 验证执行上下文

使用知识库面板检查：
- ✅ 上下文是否正确？
- ✅ 是否可以睡眠？
- ✅ 是否需要锁保护？

### 4. 检查资源生命周期

```c
// 常见错误：释放后还会被回调访问
kfree(dev);
// ❌ cancel_work_sync 应该在 kfree 之前！
```

FlowSight 警告：
```
⚠️ 潜在 UAF：dev 在 cancel_work_sync 前被释放
```

---

## 调试技巧

### 使用文本视图比较

导出两个函数的执行流，使用 diff 比较：

```bash
diff flow_a.txt flow_b.txt
```

### 使用 AI 格式化

导出为 AI 格式，让 LLM 分析潜在问题：

```markdown
## 执行流分析

### 异步边界
1. schedule_work → my_handler (WorkQueue)
2. mod_timer → timer_callback (Timer)

### 潜在问题
- timer_callback 在软中断调用 mutex_lock ❌
```

---

## 相关文档

- [快速开始](./quick-start.md)
- [功能说明](./features.md)
- [知识库参考](./knowledge-base.md)

---

**更新时间**: 2026-02-04
