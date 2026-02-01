#!/usr/bin/env python3
"""
生成 FlowSight 执行流训练数据
运行: python scripts/training/generate_data.py
"""

import json
from pathlib import Path

# 输出目录
OUTPUT_DIR = Path("data/flowsight-train")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

# 系统提示词
SYSTEM = "你是 Linux 内核代码分析专家，擅长分析函数执行流和异步机制。"

def make_sample(question: str, answer: str) -> dict:
    """创建一个训练样本"""
    text = f"<|system|>{SYSTEM}<|/system|>"
    text += f"<|user|>{question}<|/user|>"
    text += f"<|assistant|>{answer}<|/assistant|>"
    return {"text": text}


def generate_samples():
    """生成训练样本"""
    samples = []
    
    # ===== 样本1: USB probe 函数 =====
    samples.append(make_sample(
        question="""分析以下 USB 驱动 probe 函数的执行流：
```c
static int my_probe(struct usb_interface *intf, const struct usb_device_id *id)
{
    struct my_device *dev;
    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    INIT_WORK(&dev->work, my_work_handler);
    return 0;
}

static struct usb_driver my_driver = {
    .probe = my_probe,
};
module_usb_driver(my_driver);
```""",
        answer=json.dumps({
            "function": "my_probe",
            "trigger": "USB 设备插入且 VID/PID 匹配时调用",
            "context": "process",
            "can_sleep": True,
            "calls": [
                {"name": "kzalloc", "type": "sync"},
                {"name": "INIT_WORK", "type": "sync", "note": "绑定 work 和 handler，不执行"}
            ],
            "async_patterns": [
                {
                    "type": "WorkQueue",
                    "handler": "my_work_handler",
                    "binding": "INIT_WORK(&dev->work, my_work_handler)",
                    "trigger_by": "需要调用 schedule_work(&dev->work) 才会执行"
                }
            ]
        }, ensure_ascii=False, indent=2)
    ))
    
    # ===== 样本2: 中断处理 + WorkQueue =====
    samples.append(make_sample(
        question="""分析以下中断处理函数的执行流：
```c
static irqreturn_t my_irq(int irq, void *dev_id)
{
    struct my_dev *dev = dev_id;
    u32 status = readl(dev->regs + STATUS);
    if (!(status & IRQ_PENDING))
        return IRQ_NONE;
    writel(status, dev->regs + STATUS);  // ACK
    schedule_work(&dev->work);
    return IRQ_HANDLED;
}
```""",
        answer=json.dumps({
            "function": "my_irq",
            "trigger": "硬件中断信号",
            "context": "hardirq",
            "can_sleep": False,
            "calls": [
                {"name": "readl", "type": "sync"},
                {"name": "writel", "type": "sync"},
                {"name": "schedule_work", "type": "async", "note": "提交到 workqueue，立即返回"}
            ],
            "execution_flow": "中断触发 → 快速处理 → 提交 work → 返回",
            "deferred_work": {
                "mechanism": "WorkQueue",
                "timing": "中断返回后，由 kworker 线程执行"
            }
        }, ensure_ascii=False, indent=2)
    ))
    
    # ===== 样本3: 定时器回调 =====
    samples.append(make_sample(
        question="""分析以下定时器相关代码的执行流：
```c
static void my_timer_fn(struct timer_list *t)
{
    struct my_dev *dev = from_timer(dev, t, timer);
    process_data(dev);
    mod_timer(&dev->timer, jiffies + HZ);
}

static int my_init(void)
{
    timer_setup(&dev->timer, my_timer_fn, 0);
    mod_timer(&dev->timer, jiffies + HZ);
}
```""",
        answer=json.dumps({
            "function": "my_timer_fn",
            "trigger": "定时器到期",
            "context": "softirq",
            "can_sleep": False,
            "setup": {
                "function": "timer_setup",
                "binds": "my_timer_fn 绑定到 dev->timer"
            },
            "activation": {
                "function": "mod_timer",
                "effect": "设置定时器在 1 秒后 (jiffies + HZ) 触发"
            },
            "call_chain": [
                "时钟中断",
                "run_timer_softirq",
                "call_timer_fn",
                "my_timer_fn"
            ],
            "periodic": "mod_timer 重新调度实现周期执行"
        }, ensure_ascii=False, indent=2)
    ))
    
    # ===== 样本4: file_operations 函数指针 =====
    samples.append(make_sample(
        question="""分析以下代码中 file_operations 的回调函数：
```c
static int my_open(struct inode *inode, struct file *file) { return 0; }
static ssize_t my_read(struct file *f, char __user *buf, size_t len, loff_t *off) { return 0; }
static ssize_t my_write(struct file *f, const char __user *buf, size_t len, loff_t *off) { return len; }

static const struct file_operations my_fops = {
    .owner = THIS_MODULE,
    .open = my_open,
    .read = my_read,
    .write = my_write,
};
```""",
        answer=json.dumps({
            "type": "file_operations",
            "callbacks": [
                {"field": ".open", "function": "my_open", "trigger": "用户调用 open(\"/dev/xxx\")"},
                {"field": ".read", "function": "my_read", "trigger": "用户调用 read(fd, ...)"},
                {"field": ".write", "function": "my_write", "trigger": "用户调用 write(fd, ...)"}
            ],
            "context": "process",
            "can_sleep": True,
            "user_kernel_boundary": "这些函数是用户空间和内核空间的接口"
        }, ensure_ascii=False, indent=2)
    ))
    
    # ===== 样本5: Tasklet =====
    samples.append(make_sample(
        question="""分析以下 tasklet 代码的执行流：
```c
static void my_tasklet_fn(unsigned long data)
{
    struct my_dev *dev = (struct my_dev *)data;
    process_rx_packets(dev);
}

DECLARE_TASKLET(my_tasklet, my_tasklet_fn, 0);

static irqreturn_t my_irq(int irq, void *dev_id)
{
    ack_interrupt();
    tasklet_schedule(&my_tasklet);
    return IRQ_HANDLED;
}
```""",
        answer=json.dumps({
            "functions": {
                "my_irq": {
                    "context": "hardirq",
                    "can_sleep": False
                },
                "my_tasklet_fn": {
                    "context": "softirq",
                    "can_sleep": False
                }
            },
            "async_pattern": {
                "type": "Tasklet",
                "declaration": "DECLARE_TASKLET",
                "handler": "my_tasklet_fn",
                "trigger": "tasklet_schedule(&my_tasklet)"
            },
            "execution_flow": [
                "硬件中断 → my_irq (hardirq)",
                "tasklet_schedule 标记 tasklet 待执行",
                "中断返回时触发 softirq",
                "my_tasklet_fn 在 softirq 上下文执行"
            ],
            "vs_workqueue": "Tasklet 比 WorkQueue 优先级更高，但不能睡眠"
        }, ensure_ascii=False, indent=2)
    ))
    
    # ===== 样本6: platform_driver =====
    samples.append(make_sample(
        question="""分析以下 platform 驱动的执行流：
```c
static int my_probe(struct platform_device *pdev)
{
    struct resource *res;
    res = platform_get_resource(pdev, IORESOURCE_MEM, 0);
    dev->base = devm_ioremap_resource(&pdev->dev, res);
    return 0;
}

static int my_remove(struct platform_device *pdev)
{
    return 0;
}

static struct platform_driver my_driver = {
    .probe = my_probe,
    .remove = my_remove,
    .driver = { .name = "my-device" },
};
module_platform_driver(my_driver);
```""",
        answer=json.dumps({
            "driver_type": "platform_driver",
            "callbacks": {
                "probe": {
                    "function": "my_probe",
                    "trigger": "设备树匹配或 platform_device 注册时",
                    "context": "process",
                    "typical_work": "获取资源、映射寄存器、初始化硬件"
                },
                "remove": {
                    "function": "my_remove", 
                    "trigger": "设备移除或驱动卸载时",
                    "context": "process",
                    "typical_work": "释放资源、停止硬件"
                }
            },
            "registration": "module_platform_driver 宏注册驱动",
            "matching": "通过 driver.name 或设备树 compatible 匹配"
        }, ensure_ascii=False, indent=2)
    ))
    
    # ===== 样本7: completion 同步 =====
    samples.append(make_sample(
        question="""分析以下使用 completion 的代码：
```c
static DECLARE_COMPLETION(my_completion);

static void worker_thread(void)
{
    do_long_work();
    complete(&my_completion);
}

static int my_ioctl(...)
{
    start_worker();
    wait_for_completion(&my_completion);
    return 0;
}
```""",
        answer=json.dumps({
            "mechanism": "completion",
            "purpose": "线程间同步，等待异步操作完成",
            "functions": {
                "wait_for_completion": {
                    "effect": "阻塞当前线程直到 complete() 被调用",
                    "can_sleep": True,
                    "context": "只能在 process context 使用"
                },
                "complete": {
                    "effect": "唤醒等待的线程",
                    "can_sleep": False,
                    "context": "可在任何上下文调用"
                }
            },
            "flow": [
                "my_ioctl 启动 worker",
                "my_ioctl 调用 wait_for_completion 阻塞",
                "worker_thread 完成工作",
                "worker_thread 调用 complete",
                "my_ioctl 被唤醒继续执行"
            ]
        }, ensure_ascii=False, indent=2)
    ))
    
    # ===== 样本8: 延迟工作 =====
    samples.append(make_sample(
        question="""分析以下延迟工作队列代码：
```c
static void delayed_work_fn(struct work_struct *work)
{
    struct my_dev *dev = container_of(work, struct my_dev, dwork.work);
    check_hardware_status(dev);
    schedule_delayed_work(&dev->dwork, msecs_to_jiffies(1000));
}

static int my_probe(...)
{
    INIT_DELAYED_WORK(&dev->dwork, delayed_work_fn);
    schedule_delayed_work(&dev->dwork, 0);  // 立即开始
}
```""",
        answer=json.dumps({
            "mechanism": "delayed_work",
            "handler": "delayed_work_fn",
            "context": "process (kworker 线程)",
            "can_sleep": True,
            "setup": "INIT_DELAYED_WORK 绑定 handler",
            "schedule": {
                "function": "schedule_delayed_work",
                "delay_param": "第二个参数指定延迟 jiffies",
                "msecs_to_jiffies": "将毫秒转换为 jiffies"
            },
            "pattern": "周期性执行：handler 结尾重新调度自己",
            "vs_timer": "delayed_work 在进程上下文执行，可以睡眠"
        }, ensure_ascii=False, indent=2)
    ))
    
    # ===== 样本9: kthread =====
    samples.append(make_sample(
        question="""分析以下内核线程代码：
```c
static int my_thread_fn(void *data)
{
    struct my_dev *dev = data;
    
    while (!kthread_should_stop()) {
        process_pending_requests(dev);
        schedule_timeout_interruptible(HZ);
    }
    return 0;
}

static int my_probe(...)
{
    dev->thread = kthread_run(my_thread_fn, dev, "my-kthread");
}

static void my_remove(...)
{
    kthread_stop(dev->thread);
}
```""",
        answer=json.dumps({
            "mechanism": "kthread",
            "thread_function": "my_thread_fn",
            "context": "process",
            "can_sleep": True,
            "lifecycle": {
                "create": "kthread_run 创建并启动线程",
                "stop": "kthread_stop 请求线程退出",
                "check": "kthread_should_stop 检查是否应退出"
            },
            "pattern": {
                "type": "polling loop",
                "sleep": "schedule_timeout_interruptible 让出 CPU",
                "period": "HZ = 1秒"
            },
            "vs_workqueue": "kthread 是独立线程，workqueue 复用 kworker"
        }, ensure_ascii=False, indent=2)
    ))
    
    # ===== 样本10: request_threaded_irq =====
    samples.append(make_sample(
        question="""分析以下 threaded IRQ 代码：
```c
static irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    struct my_dev *dev = dev_id;
    if (!is_my_interrupt(dev))
        return IRQ_NONE;
    disable_hw_interrupt(dev);
    return IRQ_WAKE_THREAD;
}

static irqreturn_t my_irq_thread(int irq, void *dev_id)
{
    struct my_dev *dev = dev_id;
    process_interrupt_data(dev);
    enable_hw_interrupt(dev);
    return IRQ_HANDLED;
}

request_threaded_irq(irq, my_irq_handler, my_irq_thread, IRQF_SHARED, "my-dev", dev);
```""",
        answer=json.dumps({
            "mechanism": "threaded_irq",
            "handlers": {
                "hardirq": {
                    "function": "my_irq_handler",
                    "context": "hardirq",
                    "can_sleep": False,
                    "purpose": "快速确认中断，禁用硬件中断"
                },
                "thread": {
                    "function": "my_irq_thread",
                    "context": "process (irq/xxx 内核线程)",
                    "can_sleep": True,
                    "purpose": "耗时的中断处理"
                }
            },
            "flow": [
                "硬件中断触发",
                "my_irq_handler 在 hardirq 上下文执行",
                "返回 IRQ_WAKE_THREAD 唤醒线程",
                "my_irq_thread 在进程上下文执行",
                "重新启用硬件中断"
            ],
            "vs_workqueue": "threaded_irq 专用线程，延迟更低"
        }, ensure_ascii=False, indent=2)
    ))
    
    return samples


def save_samples(samples, train_ratio=0.8):
    """保存训练数据"""
    import random
    random.shuffle(samples)
    
    # 分割数据
    n = len(samples)
    train_end = int(n * train_ratio)
    
    train_samples = samples[:train_end]
    valid_samples = samples[train_end:]
    
    # 如果样本太少，复制扩充（实际使用时应该用真实数据）
    min_train = 100
    min_valid = 20
    
    while len(train_samples) < min_train:
        train_samples = train_samples + train_samples
    train_samples = train_samples[:min_train]
    
    while len(valid_samples) < min_valid:
        valid_samples = valid_samples + valid_samples
    valid_samples = valid_samples[:min_valid]
    
    # 保存
    for name, data in [("train.jsonl", train_samples), ("valid.jsonl", valid_samples)]:
        path = OUTPUT_DIR / name
        with open(path, 'w', encoding='utf-8') as f:
            for sample in data:
                f.write(json.dumps(sample, ensure_ascii=False) + '\n')
        print(f"✓ 保存 {len(data)} 条到 {path}")


def main():
    print("=" * 50)
    print("FlowSight 训练数据生成器")
    print("=" * 50)
    
    samples = generate_samples()
    print(f"\n生成了 {len(samples)} 个原始样本")
    
    save_samples(samples)
    
    print("\n✅ 数据准备完成！")
    print(f"   位置: {OUTPUT_DIR}")
    print("\n下一步: 运行训练")
    print("   python scripts/training/train.py")


if __name__ == "__main__":
    main()
