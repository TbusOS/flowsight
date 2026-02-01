# FlowSight 本地 AI 训练教程

> 🎯 **目标**：手把手教你在 MacBook M3 上训练一个专门分析 Linux 内核执行流的 AI 模型
>
> ⏱️ **总时长**：环境搭建 30 分钟 + 训练 2-6 小时（可后台运行）
>
> 📋 **前置要求**：MacBook Air/Pro M3 (24GB 内存)，会用终端

---

## 目录

- [第一课：理解我们要做什么](#第一课理解我们要做什么)
- [第二课：安装 Python 环境](#第二课安装-python-环境)
- [第三课：安装 MLX 框架](#第三课安装-mlx-框架)
- [第四课：下载基础模型](#第四课下载基础模型)
- [第五课：准备训练数据](#第五课准备训练数据)
- [第六课：开始训练](#第六课开始训练)
- [第七课：测试你的模型](#第七课测试你的模型)
- [第八课：集成到 FlowSight](#第八课集成到-flowsight)
- [常见问题排查](#常见问题排查)

---

## 第一课：理解我们要做什么

### 1.1 目标

我们要训练一个 AI 模型，它能够：
- 看懂 Linux 内核的 C 代码
- 分析函数之间的调用关系
- 识别异步执行模式（中断、定时器、工作队列等）
- 输出结构化的执行流信息

### 1.2 训练方法：LoRA 微调

我们不从零训练模型（那需要几百张 GPU），而是：

```
已有的代码大模型 (CodeLlama-7B)
        │
        │ + 你的训练数据
        │ + LoRA 微调（只训练 0.1% 的参数）
        ▼
你的专属模型 (flowsight-flow-v1)
```

**LoRA 的好处**：
- 只需要 24GB 内存（你的 MacBook 刚好够）
- 训练只需几小时（不是几周）
- 不会破坏原模型的能力

### 1.3 检查你的电脑

打开终端，运行以下命令确认你的配置：

```bash
# 检查芯片
sysctl -n machdep.cpu.brand_string
# 期望输出: Apple M3

# 检查内存
sysctl hw.memsize | awk '{print $2/1024/1024/1024 " GB"}'
# 期望输出: 24 GB
```

如果你的内存小于 24GB，训练可能会失败。16GB 可以尝试，但需要调小参数。

---

## 第二课：安装 Python 环境

### 2.1 检查 Python 版本

```bash
python3 --version
```

**期望输出**：`Python 3.10.x` 或更高版本

如果没有安装 Python 或版本太低：
```bash
# 使用 Homebrew 安装
brew install python@3.11
```

### 2.2 创建虚拟环境

虚拟环境是一个独立的 Python 空间，不会影响系统的 Python。

```bash
# 创建虚拟环境
python3 -m venv ~/.venv/flowsight-train

# 激活虚拟环境
source ~/.venv/flowsight-train/bin/activate
```

**怎么知道激活成功了？**

看终端提示符，会多出 `(flowsight-train)` 前缀：

```
(flowsight-train) sky@MacBook ~ %
```

### 2.3 升级 pip

```bash
pip install --upgrade pip
```

### ✅ 检查点

运行以下命令，确认环境正常：

```bash
which python
# 期望输出包含: .venv/flowsight-train

python --version
# 期望输出: Python 3.10+ 
```

---

## 第三课：安装 MLX 框架

### 3.1 什么是 MLX？

MLX 是 Apple 官方为 M 系列芯片开发的机器学习框架，特点：
- 专为 Apple Silicon 优化
- CPU 和 GPU 共享内存，效率高
- 支持 LoRA 训练

### 3.2 安装 MLX 和相关库

确保虚拟环境已激活，然后运行：

```bash
# 安装核心库
pip install mlx mlx-lm

# 安装辅助库
pip install transformers datasets huggingface_hub
pip install tqdm pyyaml rich
```

安装过程大约需要 2-3 分钟。

### 3.3 验证安装

```bash
python -c "import mlx.core as mx; print(f'MLX 版本: {mx.__version__}')"
```

**期望输出**：`MLX 版本: 0.x.x`

```bash
python -c "import mlx_lm; print('mlx-lm 安装成功')"
```

**期望输出**：`mlx-lm 安装成功`

### ✅ 检查点

如果上面两个命令都成功，恭喜你！环境搭建完成。

**常见问题**：

如果报错 `No module named 'mlx'`：
```bash
# 确认虚拟环境已激活
source ~/.venv/flowsight-train/bin/activate
# 重新安装
pip install mlx mlx-lm
```

---

## 第四课：下载基础模型

### 4.1 创建模型目录

```bash
mkdir -p ~/.flowsight/models
```

### 4.2 下载 CodeLlama-7B

这一步会下载约 4GB 的模型文件，需要稳定的网络。

```bash
mlx_lm.convert \
    --hf-path codellama/CodeLlama-7b-hf \
    --mlx-path ~/.flowsight/models/codellama-7b-mlx \
    -q
```

**参数说明**：
- `--hf-path`：从 HuggingFace 下载的模型名
- `--mlx-path`：保存到本地的路径
- `-q`：启用 4-bit 量化（减小模型体积）

**预计时间**：10-30 分钟（取决于网速）

**下载过程中你会看到**：

```
Fetching 16 files: 100%|████████████████| 16/16 [02:30<00:00]
Converting model...
Quantizing...
Done!
```

### 4.3 验证下载

```bash
ls -la ~/.flowsight/models/codellama-7b-mlx/
```

**期望看到这些文件**：

```
config.json
model.safetensors.index.json
tokenizer.json
tokenizer.model
weights.00.safetensors
weights.01.safetensors
...
```

### 4.4 测试模型能否运行

```bash
mlx_lm.generate \
    --model ~/.flowsight/models/codellama-7b-mlx \
    --prompt "// C function to calculate factorial\nint factorial(int n) {" \
    --max-tokens 50
```

**期望输出**（模型会补全代码）：

```
// C function to calculate factorial
int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
```

### ✅ 检查点

如果模型能生成代码，说明下载和转换都成功了！

**常见问题**：

1. **下载太慢**：考虑使用代理或镜像
2. **磁盘空间不足**：模型需要约 5GB 空间，清理磁盘后重试

---

## 第五课：准备训练数据

### 5.1 理解数据格式

训练数据是一系列"问答对"，格式如下：

```json
{"text": "<|user|>你的问题<|/user|><|assistant|>期望的回答<|/assistant|>"}
```

对于执行流分析，例如：

```json
{"text": "<|user|>分析这段代码的执行流：\n```c\nstatic int my_probe(...) { INIT_WORK(&dev->work, handler); }\n```<|/user|><|assistant|>{\"function\": \"my_probe\", \"async\": [{\"type\": \"WorkQueue\", \"handler\": \"handler\"}]}<|/assistant|>"}
```

### 5.2 创建数据目录

```bash
cd /Users/sky/linux-kernel/usb-learn/flowsight
mkdir -p data/flowsight-train
```

### 5.3 创建数据收集脚本

我们先创建一个脚本来生成训练数据：

```bash
mkdir -p scripts/training
```

创建文件 `scripts/training/generate_data.py`：

```python
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
    min_samples = 100
    while len(train_samples) < min_samples:
        train_samples = train_samples + train_samples
    train_samples = train_samples[:min_samples]
    
    while len(valid_samples) < 20:
        valid_samples = valid_samples + valid_samples
    valid_samples = valid_samples[:20]
    
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
```

### 5.4 运行数据生成脚本

```bash
# 确保在项目目录
cd /Users/sky/linux-kernel/usb-learn/flowsight

# 确保虚拟环境已激活
source ~/.venv/flowsight-train/bin/activate

# 运行脚本
python scripts/training/generate_data.py
```

**期望输出**：

```
==================================================
FlowSight 训练数据生成器
==================================================

生成了 8 个原始样本
✓ 保存 100 条到 data/flowsight-train/train.jsonl
✓ 保存 20 条到 data/flowsight-train/valid.jsonl

✅ 数据准备完成！
   位置: data/flowsight-train

下一步: 运行训练
   python scripts/training/train.py
```

### 5.5 检查生成的数据

```bash
# 查看文件
ls -la data/flowsight-train/

# 查看数据条数
wc -l data/flowsight-train/*.jsonl

# 查看第一条数据（格式化显示）
head -1 data/flowsight-train/train.jsonl | python -m json.tool
```

### ✅ 检查点

如果 `data/flowsight-train/` 目录下有 `train.jsonl` 和 `valid.jsonl`，数据准备完成！

---

## 第六课：开始训练

### 6.1 创建训练脚本

创建文件 `scripts/training/train.py`：

```python
#!/usr/bin/env python3
"""
FlowSight 模型训练脚本
运行: python scripts/training/train.py
"""

import os
import subprocess
import sys
from pathlib import Path

def main():
    # 配置
    MODEL = os.path.expanduser("~/.flowsight/models/codellama-7b-mlx")
    DATA = "data/flowsight-train"
    OUTPUT = os.path.expanduser("~/.flowsight/models/flowsight-flow-v1")
    
    # 检查模型是否存在
    if not Path(MODEL).exists():
        print(f"❌ 模型不存在: {MODEL}")
        print("请先运行第四课的下载命令")
        return
    
    # 检查数据是否存在
    if not Path(f"{DATA}/train.jsonl").exists():
        print(f"❌ 训练数据不存在: {DATA}/train.jsonl")
        print("请先运行: python scripts/training/generate_data.py")
        return
    
    # 统计数据量
    with open(f"{DATA}/train.jsonl") as f:
        train_count = sum(1 for _ in f)
    
    print("=" * 50)
    print("FlowSight 模型训练")
    print("=" * 50)
    print(f"基础模型: {MODEL}")
    print(f"训练数据: {train_count} 条")
    print(f"输出路径: {OUTPUT}")
    print()
    
    # 计算迭代次数 (3 epochs)
    iters = train_count * 3
    
    print(f"训练配置:")
    print(f"  - Batch Size: 1")
    print(f"  - LoRA Layers: 8")
    print(f"  - Learning Rate: 1e-5")
    print(f"  - 迭代次数: {iters} (约 3 轮)")
    print()
    
    # 预估时间
    time_minutes = iters * 1.5 / 60
    print(f"⏱️  预估训练时间: {time_minutes:.0f} 分钟")
    print()
    
    # 提示
    print("💡 提示:")
    print("   - 建议使用散热垫")
    print("   - 可以 Ctrl+C 中断，进度会保存")
    print("   - 训练期间可以做其他事")
    print()
    
    input("按 Enter 开始训练...")
    print()
    
    # 构建命令
    cmd = [
        sys.executable, "-m", "mlx_lm.lora",
        "--model", MODEL,
        "--data", DATA,
        "--train",
        "--batch-size", "1",
        "--lora-layers", "8",
        "--learning-rate", "1e-5",
        "--iters", str(iters),
        "--steps-per-report", "10",
        "--steps-per-eval", "50",
        "--save-every", "100",
        "--adapter-path", OUTPUT,
    ]
    
    print("执行命令:")
    print(" ".join(cmd))
    print()
    print("-" * 50)
    
    # 执行
    subprocess.run(cmd)
    
    print("-" * 50)
    print()
    
    # 检查结果
    adapter_file = Path(OUTPUT) / "adapters.safetensors"
    if adapter_file.exists():
        size_mb = adapter_file.stat().st_size / 1024 / 1024
        print(f"✅ 训练完成！")
        print(f"   模型保存在: {OUTPUT}")
        print(f"   适配器大小: {size_mb:.1f} MB")
    else:
        print("⚠️  训练可能未完成，请检查输出")


if __name__ == "__main__":
    main()
```

### 6.2 开始训练

```bash
# 确保虚拟环境已激活
source ~/.venv/flowsight-train/bin/activate

# 进入项目目录
cd /Users/sky/linux-kernel/usb-learn/flowsight

# 运行训练
python scripts/training/train.py
```

### 6.3 训练过程中你会看到

```
==================================================
FlowSight 模型训练
==================================================
基础模型: /Users/sky/.flowsight/models/codellama-7b-mlx
训练数据: 100 条
输出路径: /Users/sky/.flowsight/models/flowsight-flow-v1

训练配置:
  - Batch Size: 1
  - LoRA Layers: 8
  - Learning Rate: 1e-5
  - 迭代次数: 300 (约 3 轮)

⏱️  预估训练时间: 8 分钟

💡 提示:
   - 建议使用散热垫
   - 可以 Ctrl+C 中断，进度会保存
   - 训练期间可以做其他事

按 Enter 开始训练...
```

然后会看到训练进度：

```
Loading model...
Starting training...
Iter 10: Train loss 2.543, It/sec 0.68
Iter 20: Train loss 2.234, It/sec 0.71
Iter 30: Train loss 1.987, It/sec 0.69
...
Iter 50: Val loss 1.876
Saved adapter weights to /Users/sky/.flowsight/models/flowsight-flow-v1/adapters.safetensors
...
```

### 6.4 如何判断训练正常

| 指标 | 正常情况 | 需要关注 |
|------|----------|----------|
| Train loss | 逐渐下降 | 不下降或上升 |
| It/sec | 0.5-1.0 | 低于 0.3（可能过热降频） |
| Val loss | 接近 train loss | 远大于 train loss |

### 6.5 训练完成

训练完成后，你会看到：

```
✅ 训练完成！
   模型保存在: /Users/sky/.flowsight/models/flowsight-flow-v1
   适配器大小: 83.2 MB
```

### ✅ 检查点

```bash
ls ~/.flowsight/models/flowsight-flow-v1/
```

**期望看到**：

```
adapter_config.json
adapters.safetensors
```

---

## 第七课：测试你的模型

### 7.1 创建测试脚本

创建文件 `scripts/training/test.py`：

```python
#!/usr/bin/env python3
"""
测试训练后的模型
运行: python scripts/training/test.py
"""

import os
from mlx_lm import load, generate

def main():
    MODEL = os.path.expanduser("~/.flowsight/models/codellama-7b-mlx")
    ADAPTER = os.path.expanduser("~/.flowsight/models/flowsight-flow-v1")
    
    print("加载模型...")
    model, tokenizer = load(MODEL, adapter_path=ADAPTER)
    print("模型加载完成！\n")
    
    # 测试用例
    tests = [
        """分析以下代码的执行流：
```c
static irqreturn_t my_irq(int irq, void *data)
{
    struct my_dev *dev = data;
    schedule_work(&dev->work);
    return IRQ_HANDLED;
}
```""",
        """分析以下定时器代码：
```c
static void my_timer(struct timer_list *t)
{
    mod_timer(t, jiffies + HZ);
}
```""",
    ]
    
    print("=" * 50)
    print("开始测试")
    print("=" * 50)
    
    for i, test in enumerate(tests, 1):
        print(f"\n--- 测试 {i} ---")
        print(f"问题: {test[:80]}...")
        
        prompt = f"<|system|>你是 Linux 内核代码分析专家。<|/system|><|user|>{test}<|/user|><|assistant|>"
        
        response = generate(
            model, tokenizer,
            prompt=prompt,
            max_tokens=300,
            temp=0.1,
        )
        
        # 提取助手的回答
        if "<|assistant|>" in response:
            answer = response.split("<|assistant|>")[-1]
        else:
            answer = response
            
        print(f"\n回答:\n{answer}")
        print("-" * 50)


if __name__ == "__main__":
    main()
```

### 7.2 运行测试

```bash
python scripts/training/test.py
```

### 7.3 期望看到的输出

模型应该能输出类似这样的 JSON 格式分析：

```json
{
  "function": "my_irq",
  "context": "hardirq",
  "can_sleep": false,
  "calls": [
    {"name": "schedule_work", "type": "async"}
  ],
  "async_patterns": [
    {"type": "WorkQueue", "trigger": "schedule_work"}
  ]
}
```

### 7.4 交互式测试

你也可以直接用命令行测试：

```bash
mlx_lm.generate \
    --model ~/.flowsight/models/codellama-7b-mlx \
    --adapter-path ~/.flowsight/models/flowsight-flow-v1 \
    --prompt "<|user|>分析 INIT_WORK 宏的作用<|/user|><|assistant|>" \
    --max-tokens 200
```

### ✅ 检查点

如果模型能输出有意义的执行流分析，恭喜你！训练成功！

---

## 第八课：集成到 FlowSight

### 8.1 模型位置

训练好的模型位于：

```
~/.flowsight/models/
├── codellama-7b-mlx/           # 基础模型 (~4GB)
│   ├── weights.*.safetensors
│   ├── config.json
│   └── tokenizer.model
└── flowsight-flow-v1/          # 你的 LoRA 适配器 (~100MB)
    ├── adapters.safetensors
    └── adapter_config.json
```

### 8.2 使用方式

在 Python 中使用你的模型：

```python
from mlx_lm import load, generate

# 加载模型（只需要一次）
model, tokenizer = load(
    "~/.flowsight/models/codellama-7b-mlx",
    adapter_path="~/.flowsight/models/flowsight-flow-v1"
)

# 分析代码
def analyze_code(code: str) -> str:
    prompt = f"<|user|>分析以下代码的执行流：\n```c\n{code}\n```<|/user|><|assistant|>"
    response = generate(model, tokenizer, prompt=prompt, max_tokens=500, temp=0.1)
    return response

# 使用
result = analyze_code("static int my_probe(...) { ... }")
print(result)
```

### 8.3 下一步

后续我们会：
1. 添加更多训练数据（目标 3000+ 条）
2. 在 FlowSight Rust 后端集成调用
3. 优化模型输出格式

---

## 常见问题排查

### Q1: `pip install` 报错

```bash
# 确保虚拟环境已激活
source ~/.venv/flowsight-train/bin/activate

# 升级 pip
pip install --upgrade pip

# 重试安装
pip install mlx mlx-lm
```

### Q2: 下载模型时网络超时

```bash
# 设置 HuggingFace 镜像（中国大陆）
export HF_ENDPOINT=https://hf-mirror.com

# 重新下载
mlx_lm.convert --hf-path codellama/CodeLlama-7b-hf ...
```

### Q3: 训练时内存不足 (killed)

```bash
# 关闭其他应用（Chrome, VSCode 等）

# 减小 LoRA 层数
--lora-layers 4  # 从 8 改为 4

# 减小序列长度（需要修改数据）
```

### Q4: 训练速度很慢

```bash
# 检查是否过热降频
# 打开 Activity Monitor > CPU > 查看频率

# 解决方案：
# 1. 使用散热垫
# 2. 打开空调
# 3. 晚上睡觉时训练
```

### Q5: 模型输出乱码

可能是 prompt 格式不对，确保使用：
```
<|user|>你的问题<|/user|><|assistant|>
```

### Q6: 忘记激活虚拟环境

每次打开新终端都需要激活：

```bash
source ~/.venv/flowsight-train/bin/activate
```

可以添加 alias 到 `~/.zshrc`：

```bash
echo 'alias fstrain="source ~/.venv/flowsight-train/bin/activate"' >> ~/.zshrc
source ~/.zshrc

# 以后只需输入
fstrain
```

---

## 恭喜！

你已经完成了本地 AI 模型训练的全部课程：

1. ✅ 理解训练原理
2. ✅ 搭建 Python 环境
3. ✅ 安装 MLX 框架
4. ✅ 下载基础模型
5. ✅ 准备训练数据
6. ✅ 运行训练
7. ✅ 测试模型
8. ✅ 了解集成方式

**下一步提升**：
- 收集更多真实的内核代码样本
- 增加训练数据到 3000+ 条
- 调整训练参数优化效果

有问题随时问我！
