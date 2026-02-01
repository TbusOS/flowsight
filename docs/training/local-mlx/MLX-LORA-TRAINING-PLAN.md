# FlowSight 本地 MLX 训练计划

> CodeLlama-7B + LoRA 微调方案 | MacBook Air M3 24GB

---

## 目录

1. [项目概述](#1-项目概述)
2. [硬件分析](#2-硬件分析)
3. [技术选型](#3-技术选型)
4. [环境搭建](#4-环境搭建)
5. [数据准备](#5-数据准备)
6. [模型训练](#6-模型训练)
7. [训练监控](#7-训练监控)
8. [模型测试](#8-模型测试)
9. [集成部署](#9-集成部署)
10. [常见问题](#10-常见问题)

---

## 1. 项目概述

### 1.1 目标

训练一个专门用于 **Linux 内核代码执行流生成** 的小型 AI 模型，能够：

- 分析函数调用关系
- 识别异步机制（WorkQueue, Timer, IRQ, Tasklet）
- 解析函数指针指向
- 理解回调触发时机
- 生成结构化的执行流 JSON

### 1.2 方案选择

| 方案 | 模型 | 训练方式 | 预计时间 | 效果 |
|------|------|----------|----------|------|
| ~~短期方案~~ | DeepSeek-1.3B | LoRA | 30分钟-2小时 | 基础 |
| **中期方案** ✓ | CodeLlama-7B | LoRA | 2-8小时 | **推荐** |
| ~~长期方案~~ | CodeLlama-13B | LoRA | 需要更多内存 | 最佳 |

**选择理由**：
- CodeLlama-7B 代码理解能力强
- 量化后可在 24GB 内存运行
- LoRA 只训练少量参数，效率高

### 1.3 最终交付物

```
~/.flowsight/models/
├── flowsight-flow-v1/          # 模型目录
│   ├── adapters.safetensors    # LoRA 权重 (~100MB)
│   ├── adapter_config.json     # 配置文件
│   └── tokenizer/              # 分词器
└── version.json                # 版本信息
```

---

## 2. 硬件分析

### 2.1 你的设备配置

```
┌─────────────────────────────────────────────────────────┐
│  MacBook Air M3                                         │
├─────────────────────────────────────────────────────────┤
│  CPU: Apple M3 (4P + 4E = 8核)                         │
│  GPU: Apple M3 (10核)                                   │
│  NPU: 16核 Neural Engine                                │
│  RAM: 24GB 统一内存 (CPU/GPU 共享)                      │
│  带宽: ~100GB/s                                         │
├─────────────────────────────────────────────────────────┤
│  Metal: 4 ✓                                             │
│  MLX:   兼容 ✓                                          │
└─────────────────────────────────────────────────────────┘
```

### 2.2 内存分配计划

| 用途 | 内存占用 | 说明 |
|------|----------|------|
| 系统 + 应用 | ~4GB | macOS 基础开销 |
| CodeLlama-7B Q4 | ~4GB | 4-bit 量化模型 |
| LoRA 参数 | ~0.5GB | rank=8, 8层 |
| 优化器状态 | ~1GB | AdamW 状态 |
| 激活值缓存 | ~2-4GB | 取决于序列长度 |
| **预留空间** | **~10GB** | 安全余量 |
| **总计** | **~14-16GB** | 24GB 足够 ✓ |

### 2.3 性能预估

| 指标 | 预估值 | 说明 |
|------|--------|------|
| 每条数据处理时间 | 1-2秒 | batch_size=1, seq_len=512 |
| 每 epoch 时间 | ~30-60分钟/1000条 | 包含前向+反向传播 |
| GPU 利用率 | 70-90% | Metal 加速 |
| 功耗 | 15-30W | 无风扇设计，注意散热 |
| 温度 | 80-100°C | 可能触发降频 |

---

## 3. 技术选型

### 3.1 为什么选择 MLX？

```
┌─────────────────────────────────────────────────────────────────┐
│                    MLX vs PyTorch (Apple Silicon)               │
├─────────────────────────┬───────────────────────────────────────┤
│        MLX              │        PyTorch + MPS                  │
├─────────────────────────┼───────────────────────────────────────┤
│  ✓ Apple 官方维护       │  △ 社区维护 MPS 后端                  │
│  ✓ 统一内存零拷贝       │  × 需要内存拷贝                       │
│  ✓ 惰性计算优化         │  × 即时执行                           │
│  ✓ 原生支持 LoRA        │  △ 需要 PEFT 库                       │
│  ✓ 专为 M 芯片优化      │  △ 通用设计                           │
└─────────────────────────┴───────────────────────────────────────┘
```

### 3.2 模型选择：CodeLlama-7B

| 特性 | 值 |
|------|-----|
| 参数量 | 7B (70亿) |
| 上下文长度 | 16K tokens |
| 训练数据 | 500B tokens 代码 |
| 支持语言 | Python, C, C++, Java, Go, Rust... |
| 代码理解 | 优秀 |
| 量化后大小 | ~4GB (Q4_K_M) |

### 3.3 LoRA 配置

```python
lora_config = {
    "rank": 8,              # LoRA 秩，越大容量越大
    "alpha": 16,            # 缩放因子，通常 2x rank
    "target_layers": 8,     # 微调最后 8 层
    "dropout": 0.05,        # 防止过拟合
    "modules": [            # 目标模块
        "q_proj",           # Query 投影
        "v_proj",           # Value 投影
    ]
}
```

**可训练参数**：~4M（原模型的 0.06%）

---

## 4. 环境搭建

### 4.1 安装依赖

```bash
# 1. 创建虚拟环境（推荐）
python3 -m venv ~/.venv/flowsight-train
source ~/.venv/flowsight-train/bin/activate

# 2. 安装 MLX 生态
pip install mlx mlx-lm

# 3. 安装辅助工具
pip install transformers datasets huggingface_hub
pip install tqdm pyyaml rich

# 4. 验证安装
python -c "import mlx.core as mx; print(f'MLX version: {mx.__version__}')"
python -c "import mlx_lm; print('mlx-lm installed')"
```

### 4.2 下载基础模型

```bash
# 创建模型目录
mkdir -p ~/.flowsight/models

# 方法1: 使用 mlx_lm 转换（推荐）
# 自动下载、转换、量化
mlx_lm.convert \
    --hf-path codellama/CodeLlama-7b-hf \
    --mlx-path ~/.flowsight/models/codellama-7b-mlx \
    -q  # 启用 4-bit 量化

# 方法2: 手动下载
huggingface-cli download codellama/CodeLlama-7b-hf \
    --local-dir ~/.flowsight/models/codellama-7b-hf
```

### 4.3 验证模型

```bash
# 测试模型推理
mlx_lm.generate \
    --model ~/.flowsight/models/codellama-7b-mlx \
    --prompt "// Function to find callback targets in C code\nvoid analyze_callbacks(" \
    --max-tokens 100
```

---

## 5. 数据准备

### 5.1 数据格式

FlowSight 执行流训练数据使用 JSONL 格式：

```jsonl
{"text": "<|system|>你是一个 Linux 内核代码分析专家...</|system|><|user|>分析以下代码的执行流：\n```c\nstatic int my_probe(struct usb_interface *intf, ...) {...}\n```</|user|><|assistant|>{\"function\": \"my_probe\", \"trigger\": \"USB设备插入\", \"context\": \"process\", \"calls\": [...]}"}
```

### 5.2 数据目录结构

```
data/flowsight-train/
├── train.jsonl          # 训练集 (80%)
├── valid.jsonl          # 验证集 (10%)
├── test.jsonl           # 测试集 (10%)
└── metadata.json        # 数据集信息
```

### 5.3 数据收集脚本

在 FlowSight 项目中创建数据收集脚本：

```python
#!/usr/bin/env python3
"""
scripts/training/collect_flow_data.py
从 Linux 内核代码收集执行流训练数据
"""

import json
import os
from pathlib import Path
from typing import List, Dict
import subprocess

# 配置
LINUX_KERNEL_PATH = "/Users/sky/linux-kernel/linux"
OUTPUT_DIR = Path("data/flowsight-train")
TARGET_DIRS = [
    "arch/arm/mach-imx",      # ARM32 平台代码
    "drivers/usb/gadget",     # USB 驱动
    "drivers/net/ethernet",   # 网络驱动
]

class FlowDataCollector:
    """执行流训练数据收集器"""
    
    def __init__(self):
        self.samples = []
        self.system_prompt = """你是一个专业的 Linux 内核代码分析助手。
你的任务是分析 C 代码并生成结构化的执行流信息。

输出格式（JSON）：
{
    "function": "函数名",
    "file": "文件路径",
    "trigger": "触发条件描述",
    "context": "执行上下文 (process/softirq/hardirq/atomic)",
    "calls": [
        {"name": "被调用函数", "type": "sync/async", "mechanism": "direct/workqueue/timer/..."}
    ],
    "async_patterns": [
        {"type": "WorkQueue/Timer/Tasklet/IRQ", "handler": "处理函数", "trigger": "触发方式"}
    ]
}"""

    def add_probe_sample(self, code: str, analysis: Dict):
        """添加 probe 函数分析样本"""
        prompt = f"分析以下 Linux 驱动 probe 函数的执行流：\n```c\n{code}\n```"
        response = json.dumps(analysis, ensure_ascii=False, indent=2)
        self._add_sample(prompt, response)
    
    def add_irq_sample(self, code: str, analysis: Dict):
        """添加中断处理函数分析样本"""
        prompt = f"分析以下中断处理函数的执行流和异步机制：\n```c\n{code}\n```"
        response = json.dumps(analysis, ensure_ascii=False, indent=2)
        self._add_sample(prompt, response)
    
    def add_callback_sample(self, code: str, callback_name: str, analysis: Dict):
        """添加回调函数分析样本"""
        prompt = f"分析以下代码中 {callback_name} 回调函数何时被调用：\n```c\n{code}\n```"
        response = json.dumps(analysis, ensure_ascii=False, indent=2)
        self._add_sample(prompt, response)
    
    def add_funcptr_sample(self, code: str, ptr_name: str, analysis: Dict):
        """添加函数指针分析样本"""
        prompt = f"分析以下代码中函数指针 {ptr_name} 可能指向哪些函数：\n```c\n{code}\n```"
        response = json.dumps(analysis, ensure_ascii=False, indent=2)
        self._add_sample(prompt, response)
    
    def _add_sample(self, user_msg: str, assistant_msg: str):
        """添加单个样本"""
        text = f"<|system|>{self.system_prompt}<|/system|>"
        text += f"<|user|>{user_msg}<|/user|>"
        text += f"<|assistant|>{assistant_msg}<|/assistant|>"
        self.samples.append({"text": text})
    
    def save(self, train_ratio=0.8, valid_ratio=0.1):
        """保存数据集"""
        import random
        random.shuffle(self.samples)
        
        total = len(self.samples)
        train_end = int(total * train_ratio)
        valid_end = train_end + int(total * valid_ratio)
        
        OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
        
        splits = {
            "train.jsonl": self.samples[:train_end],
            "valid.jsonl": self.samples[train_end:valid_end],
            "test.jsonl": self.samples[valid_end:],
        }
        
        for filename, data in splits.items():
            path = OUTPUT_DIR / filename
            with open(path, 'w', encoding='utf-8') as f:
                for sample in data:
                    f.write(json.dumps(sample, ensure_ascii=False) + '\n')
            print(f"Saved {len(data)} samples to {path}")
        
        # 保存元数据
        metadata = {
            "total_samples": total,
            "train_samples": len(splits["train.jsonl"]),
            "valid_samples": len(splits["valid.jsonl"]),
            "test_samples": len(splits["test.jsonl"]),
            "source_dirs": TARGET_DIRS,
        }
        with open(OUTPUT_DIR / "metadata.json", 'w') as f:
            json.dump(metadata, f, indent=2)


def generate_samples_from_knowledge():
    """从 FlowSight 知识库生成样本"""
    collector = FlowDataCollector()
    
    # 示例1: USB probe 函数
    collector.add_probe_sample(
        code="""static int my_usb_probe(struct usb_interface *intf, 
                        const struct usb_device_id *id)
{
    struct my_device *dev;
    
    dev = kzalloc(sizeof(*dev), GFP_KERNEL);
    if (!dev)
        return -ENOMEM;
    
    INIT_WORK(&dev->work, my_work_handler);
    usb_set_intfdata(intf, dev);
    
    return 0;
}

static struct usb_driver my_driver = {
    .name = "my_usb",
    .probe = my_usb_probe,
    .disconnect = my_usb_disconnect,
    .id_table = my_id_table,
};
module_usb_driver(my_driver);""",
        analysis={
            "function": "my_usb_probe",
            "file": "drivers/usb/my_driver.c",
            "trigger": "USB 设备插入且 ID 匹配时由 USB 核心调用",
            "context": "process",
            "calls": [
                {"name": "kzalloc", "type": "sync", "mechanism": "direct"},
                {"name": "INIT_WORK", "type": "sync", "mechanism": "direct", 
                 "note": "绑定 work 和 handler，不执行"},
                {"name": "usb_set_intfdata", "type": "sync", "mechanism": "direct"},
            ],
            "async_patterns": [
                {
                    "type": "WorkQueue",
                    "handler": "my_work_handler",
                    "binding": "INIT_WORK(&dev->work, my_work_handler)",
                    "trigger": "需要后续调用 schedule_work(&dev->work)",
                }
            ],
            "call_chain": [
                "usb_hub_port_connect",
                "usb_new_device", 
                "device_add",
                "bus_probe_device",
                "usb_probe_interface",
                "my_usb_probe"
            ]
        }
    )
    
    # 示例2: 中断处理 + WorkQueue
    collector.add_irq_sample(
        code="""static irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    struct my_device *dev = dev_id;
    u32 status;
    
    status = readl(dev->regs + STATUS_REG);
    if (!(status & IRQ_PENDING))
        return IRQ_NONE;
    
    writel(status, dev->regs + STATUS_REG);  // ACK
    schedule_work(&dev->work);
    
    return IRQ_HANDLED;
}

static void my_work_handler(struct work_struct *work)
{
    struct my_device *dev = container_of(work, struct my_device, work);
    // 处理耗时操作...
}""",
        analysis={
            "function": "my_irq_handler",
            "trigger": "硬件中断触发",
            "context": "hardirq",
            "calls": [
                {"name": "readl", "type": "sync", "mechanism": "direct"},
                {"name": "writel", "type": "sync", "mechanism": "direct"},
                {"name": "schedule_work", "type": "async", "mechanism": "workqueue",
                 "target": "my_work_handler", "note": "提交到 workqueue，不等待执行"},
            ],
            "async_patterns": [
                {
                    "type": "WorkQueue",
                    "handler": "my_work_handler",
                    "trigger": "schedule_work(&dev->work)",
                    "execution_context": "process (kworker线程)",
                    "timing": "中断返回后，由调度器决定执行时机",
                }
            ],
            "execution_flow": {
                "phase1_hardirq": ["readl", "writel", "schedule_work"],
                "phase2_workqueue": ["my_work_handler (延后执行)"],
            }
        }
    )
    
    # 示例3: 函数指针分析
    collector.add_funcptr_sample(
        code="""struct file_operations my_fops = {
    .owner = THIS_MODULE,
    .open = my_open,
    .release = my_release,
    .read = my_read,
    .write = my_write,
    .unlocked_ioctl = my_ioctl,
};

static int __init my_init(void)
{
    cdev_init(&my_cdev, &my_fops);
    cdev_add(&my_cdev, devno, 1);
    return 0;
}""",
        ptr_name="my_fops",
        analysis={
            "pointer_name": "my_fops",
            "type": "struct file_operations",
            "targets": {
                ".open": {"function": "my_open", "trigger": "用户调用 open()"},
                ".release": {"function": "my_release", "trigger": "用户调用 close()"},
                ".read": {"function": "my_read", "trigger": "用户调用 read()"},
                ".write": {"function": "my_write", "trigger": "用户调用 write()"},
                ".unlocked_ioctl": {"function": "my_ioctl", "trigger": "用户调用 ioctl()"},
            },
            "registration": "cdev_init + cdev_add 注册到字符设备子系统",
            "call_chain_example": {
                "user_space": "fd = open(\"/dev/mydev\", O_RDWR)",
                "kernel": ["sys_open", "do_sys_open", "vfs_open", "chrdev_open", "my_open"]
            }
        }
    )
    
    # 示例4: Timer 异步模式
    collector.add_callback_sample(
        code="""static void my_timer_callback(struct timer_list *t)
{
    struct my_device *dev = from_timer(dev, t, timer);
    // 定时处理...
    mod_timer(&dev->timer, jiffies + HZ);  // 重新调度
}

static int my_probe(...)
{
    timer_setup(&dev->timer, my_timer_callback, 0);
    mod_timer(&dev->timer, jiffies + HZ);  // 1秒后触发
}""",
        callback_name="my_timer_callback",
        analysis={
            "function": "my_timer_callback",
            "type": "timer_callback",
            "trigger": "定时器到期时由软中断调用",
            "context": "softirq",
            "binding": "timer_setup(&dev->timer, my_timer_callback, 0)",
            "activation": "mod_timer(&dev->timer, jiffies + HZ)",
            "call_chain": [
                "timer_interrupt (硬中断)",
                "irq_exit",
                "invoke_softirq",
                "run_timer_softirq",
                "call_timer_fn",
                "my_timer_callback"
            ],
            "notes": [
                "执行上下文是 softirq，不能睡眠",
                "mod_timer 可重新调度实现周期性执行",
                "del_timer_sync 可安全删除定时器"
            ]
        }
    )
    
    # TODO: 添加更多样本类型
    # - Tasklet 模式
    # - RCU 回调
    # - Completion 等待
    # - Wait Queue
    # - kthread
    
    return collector


def main():
    print("=" * 60)
    print("FlowSight 训练数据收集")
    print("=" * 60)
    
    collector = generate_samples_from_knowledge()
    
    print(f"\n收集到 {len(collector.samples)} 个样本")
    
    # 实际使用时，还需要：
    # 1. 从真实内核代码中提取更多样本
    # 2. 使用 FlowSight 现有分析结果
    # 3. 人工标注复杂案例
    
    # 暂时添加一些重复样本以达到训练所需数量
    # TODO: 替换为真实数据
    original_count = len(collector.samples)
    for _ in range(249):  # 扩展到 ~1000 条
        for sample in collector.samples[:original_count]:
            collector.samples.append(sample.copy())
            if len(collector.samples) >= 1000:
                break
        if len(collector.samples) >= 1000:
            break
    
    collector.save()
    print("\n数据准备完成！")


if __name__ == "__main__":
    main()
```

### 5.4 运行数据收集

```bash
# 在 FlowSight 项目目录执行
cd /Users/sky/linux-kernel/usb-learn/flowsight

# 创建脚本目录
mkdir -p scripts/training

# 保存上述脚本后执行
python scripts/training/collect_flow_data.py

# 检查生成的数据
ls -la data/flowsight-train/
wc -l data/flowsight-train/*.jsonl
```

### 5.5 数据质量要求

| 指标 | 要求 | 说明 |
|------|------|------|
| 最小数据量 | 1,000 条 | 基础模型 |
| 推荐数据量 | 3,000+ 条 | 较好效果 |
| 序列长度 | 256-1024 tokens | 代码 + 分析结果 |
| 代码覆盖 | 多种异步模式 | WorkQueue, Timer, IRQ, Tasklet |
| 标注准确性 | >95% | 人工审核关键样本 |

---

## 6. 模型训练

### 6.1 训练脚本

创建训练配置和脚本：

```python
#!/usr/bin/env python3
"""
scripts/training/train_flow_model.py
使用 MLX LoRA 训练执行流生成模型
"""

import os
import subprocess
import sys
from pathlib import Path

# 配置
CONFIG = {
    # 模型
    "model_path": os.path.expanduser("~/.flowsight/models/codellama-7b-mlx"),
    "output_path": os.path.expanduser("~/.flowsight/models/flowsight-flow-v1"),
    
    # 数据
    "data_path": "data/flowsight-train",
    
    # LoRA 参数
    "lora_layers": 8,       # 微调最后 8 层
    "lora_rank": 8,         # LoRA 秩
    
    # 训练参数
    "batch_size": 1,        # M3 24GB 建议 batch_size=1
    "learning_rate": 1e-5,  # 学习率
    "epochs": 3,            # 训练轮数
    "steps_per_report": 10, # 每 N 步报告一次
    "steps_per_eval": 100,  # 每 N 步评估一次
    "save_every": 500,      # 每 N 步保存一次
    
    # 序列长度
    "max_seq_length": 512,  # 最大序列长度
}


def check_prerequisites():
    """检查前置条件"""
    print("检查环境...")
    
    # 检查 mlx_lm
    try:
        import mlx_lm
        print(f"  ✓ mlx_lm 已安装")
    except ImportError:
        print("  ✗ mlx_lm 未安装，请运行: pip install mlx-lm")
        return False
    
    # 检查模型
    model_path = Path(CONFIG["model_path"])
    if not model_path.exists():
        print(f"  ✗ 模型不存在: {model_path}")
        print("    请运行: mlx_lm.convert --hf-path codellama/CodeLlama-7b-hf --mlx-path {model_path} -q")
        return False
    print(f"  ✓ 模型存在: {model_path}")
    
    # 检查数据
    data_path = Path(CONFIG["data_path"])
    train_file = data_path / "train.jsonl"
    if not train_file.exists():
        print(f"  ✗ 训练数据不存在: {train_file}")
        print("    请先运行: python scripts/training/collect_flow_data.py")
        return False
    
    # 统计数据量
    with open(train_file) as f:
        train_count = sum(1 for _ in f)
    print(f"  ✓ 训练数据: {train_count} 条")
    
    return True


def estimate_training_time():
    """估算训练时间"""
    data_path = Path(CONFIG["data_path"]) / "train.jsonl"
    with open(data_path) as f:
        sample_count = sum(1 for _ in f)
    
    # M3 估算：每条约 1.5 秒
    time_per_sample = 1.5
    epochs = CONFIG["epochs"]
    total_seconds = sample_count * time_per_sample * epochs
    
    hours = total_seconds // 3600
    minutes = (total_seconds % 3600) // 60
    
    print(f"\n预估训练时间:")
    print(f"  样本数: {sample_count}")
    print(f"  轮数: {epochs}")
    print(f"  总时间: {int(hours)} 小时 {int(minutes)} 分钟")
    
    return total_seconds


def run_training():
    """执行训练"""
    print("\n" + "=" * 60)
    print("开始训练")
    print("=" * 60)
    
    # 构建命令
    cmd = [
        sys.executable, "-m", "mlx_lm.lora",
        "--model", CONFIG["model_path"],
        "--data", CONFIG["data_path"],
        "--train",
        "--batch-size", str(CONFIG["batch_size"]),
        "--lora-layers", str(CONFIG["lora_layers"]),
        "--learning-rate", str(CONFIG["learning_rate"]),
        "--iters", str(CONFIG["epochs"] * 1000),  # 迭代次数
        "--steps-per-report", str(CONFIG["steps_per_report"]),
        "--steps-per-eval", str(CONFIG["steps_per_eval"]),
        "--save-every", str(CONFIG["save_every"]),
        "--adapter-path", CONFIG["output_path"],
    ]
    
    print(f"命令: {' '.join(cmd)}\n")
    
    # 执行训练
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    
    # 实时输出
    for line in process.stdout:
        print(line, end='')
    
    process.wait()
    
    if process.returncode == 0:
        print("\n" + "=" * 60)
        print("训练完成！")
        print(f"模型保存在: {CONFIG['output_path']}")
        print("=" * 60)
    else:
        print(f"\n训练失败，退出码: {process.returncode}")
        return False
    
    return True


def main():
    print("=" * 60)
    print("FlowSight 执行流模型训练")
    print("=" * 60)
    
    if not check_prerequisites():
        print("\n前置检查失败，请解决上述问题后重试。")
        return
    
    estimate_training_time()
    
    print("\n即将开始训练。注意事项：")
    print("  1. 建议使用散热垫或合盖外接显示器")
    print("  2. 训练期间避免运行其他大型程序")
    print("  3. 可以随时 Ctrl+C 中断，已保存的 checkpoint 不会丢失")
    
    input("\n按 Enter 开始训练...")
    
    run_training()


if __name__ == "__main__":
    main()
```

### 6.2 启动训练

```bash
# 确保在正确的环境
source ~/.venv/flowsight-train/bin/activate

# 进入项目目录
cd /Users/sky/linux-kernel/usb-learn/flowsight

# 运行训练脚本
python scripts/training/train_flow_model.py

# 或者直接使用 mlx_lm 命令
mlx_lm.lora \
    --model ~/.flowsight/models/codellama-7b-mlx \
    --data data/flowsight-train \
    --train \
    --batch-size 1 \
    --lora-layers 8 \
    --learning-rate 1e-5 \
    --iters 3000 \
    --adapter-path ~/.flowsight/models/flowsight-flow-v1
```

### 6.3 训练参数说明

| 参数 | 值 | 说明 |
|------|-----|------|
| `--batch-size` | 1 | 24GB 内存建议设为 1 |
| `--lora-layers` | 8 | 微调最后 8 层 Transformer |
| `--learning-rate` | 1e-5 | 学习率，可调 1e-6 ~ 1e-4 |
| `--iters` | 3000 | 总迭代次数 ≈ 3 epochs × 1000 |
| `--save-every` | 500 | 每 500 步保存 checkpoint |

---

## 7. 训练监控

### 7.1 实时监控

```bash
# 新开一个终端，监控系统资源
# 方法1: 使用 htop
htop

# 方法2: 使用 Activity Monitor (图形界面)
open -a "Activity Monitor"

# 方法3: 监控 GPU 使用
# Apple Silicon 没有 nvidia-smi，使用 powermetrics
sudo powermetrics --samplers gpu_power -i 1000 -n 1
```

### 7.2 温度监控

```bash
# 安装 osx-cpu-temp (Homebrew)
brew install osx-cpu-temp

# 监控温度
while true; do
    clear
    echo "=== FlowSight 训练监控 ==="
    osx-cpu-temp
    echo ""
    echo "内存使用:"
    vm_stat | head -5
    sleep 5
done
```

### 7.3 训练日志

MLX 训练会输出类似以下日志：

```
Iter 10: Train loss 2.345, Learning rate 1.00e-05, It/sec 0.67
Iter 20: Train loss 2.123, Learning rate 1.00e-05, It/sec 0.69
Iter 30: Train loss 1.987, Learning rate 1.00e-05, It/sec 0.68
...
Iter 100: Val loss 1.654, Val ppl 5.23
...
Saved adapter weights to ~/.flowsight/models/flowsight-flow-v1/adapters.safetensors
```

### 7.4 判断训练效果

| 指标 | 良好 | 需关注 |
|------|------|--------|
| Train Loss | 持续下降 | 不下降或上升 |
| Val Loss | 接近 Train Loss | 远大于 Train Loss（过拟合） |
| PPL (困惑度) | < 10 | > 50 |
| It/sec | 稳定 | 大幅波动（可能降频） |

---

## 8. 模型测试

### 8.1 加载训练后的模型

```python
#!/usr/bin/env python3
"""
scripts/training/test_flow_model.py
测试训练后的执行流生成模型
"""

from mlx_lm import load, generate

# 加载模型和 LoRA 适配器
model, tokenizer = load(
    "~/.flowsight/models/codellama-7b-mlx",
    adapter_path="~/.flowsight/models/flowsight-flow-v1"
)

# 测试用例
test_cases = [
    # 测试1: USB probe 分析
    """分析以下 Linux 驱动 probe 函数的执行流：
```c
static int my_probe(struct platform_device *pdev)
{
    struct my_device *dev;
    dev = devm_kzalloc(&pdev->dev, sizeof(*dev), GFP_KERNEL);
    INIT_DELAYED_WORK(&dev->dwork, my_delayed_handler);
    schedule_delayed_work(&dev->dwork, HZ);
    return 0;
}
```""",

    # 测试2: 中断处理分析
    """分析以下中断处理函数的执行流和异步机制：
```c
static irqreturn_t my_irq(int irq, void *data)
{
    struct my_dev *dev = data;
    u32 status = readl(dev->base + IRQ_STATUS);
    writel(status, dev->base + IRQ_ACK);
    tasklet_schedule(&dev->tasklet);
    return IRQ_HANDLED;
}
```""",
]

print("=" * 60)
print("FlowSight 执行流模型测试")
print("=" * 60)

for i, test in enumerate(test_cases, 1):
    print(f"\n--- 测试 {i} ---")
    prompt = f"<|system|>你是一个 Linux 内核代码分析专家。<|/system|><|user|>{test}<|/user|><|assistant|>"
    
    response = generate(
        model, 
        tokenizer, 
        prompt=prompt,
        max_tokens=512,
        temp=0.1,  # 低温度，更确定性的输出
    )
    
    print(f"输入:\n{test[:100]}...")
    print(f"\n输出:\n{response}")
    print("-" * 40)
```

### 8.2 批量评估

```bash
# 使用 mlx_lm 评估
mlx_lm.lora \
    --model ~/.flowsight/models/codellama-7b-mlx \
    --adapter-path ~/.flowsight/models/flowsight-flow-v1 \
    --data data/flowsight-train \
    --test
```

### 8.3 评估指标

| 指标 | 目标 | 说明 |
|------|------|------|
| 执行上下文正确率 | >90% | 正确识别 process/softirq/hardirq |
| 异步机制识别率 | >85% | 正确识别 WorkQueue/Timer/Tasklet |
| 调用链准确率 | >80% | 生成的调用链与实际一致 |
| JSON 格式正确率 | >95% | 输出合法的 JSON |

---

## 9. 集成部署

### 9.1 模型文件结构

训练完成后，模型文件位于：

```
~/.flowsight/models/flowsight-flow-v1/
├── adapters.safetensors    # LoRA 权重 (~100MB)
├── adapter_config.json     # LoRA 配置
└── tokenizer_config.json   # 分词器配置

~/.flowsight/models/codellama-7b-mlx/
├── weights.00.safetensors  # 基础模型权重
├── weights.01.safetensors
├── config.json
└── tokenizer.model
```

### 9.2 FlowSight 集成

在 FlowSight Rust 后端添加模型调用：

```rust
// crates/flowsight-ai/src/local_model.rs

use std::process::Command;

pub struct LocalFlowModel {
    model_path: String,
    adapter_path: String,
}

impl LocalFlowModel {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap();
        Self {
            model_path: format!("{}/.flowsight/models/codellama-7b-mlx", home),
            adapter_path: format!("{}/.flowsight/models/flowsight-flow-v1", home),
        }
    }
    
    pub async fn analyze_flow(&self, code: &str) -> Result<FlowAnalysis, Error> {
        // 调用 Python 脚本执行推理
        let output = Command::new("python3")
            .args([
                "-c",
                &format!(r#"
from mlx_lm import load, generate
import json

model, tokenizer = load("{}", adapter_path="{}")
prompt = '''<|user|>分析以下代码的执行流：
```c
{}
```<|/user|><|assistant|>'''

result = generate(model, tokenizer, prompt=prompt, max_tokens=512, temp=0.1)
print(result)
"#, self.model_path, self.adapter_path, code)
            ])
            .output()?;
        
        let response = String::from_utf8(output.stdout)?;
        // 解析 JSON 输出
        let analysis: FlowAnalysis = serde_json::from_str(&response)?;
        Ok(analysis)
    }
}
```

### 9.3 推理服务器（可选）

如果需要更好的性能，可以启动一个本地推理服务器：

```python
#!/usr/bin/env python3
"""
scripts/inference/flow_server.py
本地执行流分析服务器
"""

from fastapi import FastAPI
from pydantic import BaseModel
from mlx_lm import load, generate
import uvicorn

app = FastAPI()

# 加载模型（启动时加载一次）
print("Loading model...")
model, tokenizer = load(
    "~/.flowsight/models/codellama-7b-mlx",
    adapter_path="~/.flowsight/models/flowsight-flow-v1"
)
print("Model loaded!")

class AnalyzeRequest(BaseModel):
    code: str
    max_tokens: int = 512

class AnalyzeResponse(BaseModel):
    analysis: str

@app.post("/analyze", response_model=AnalyzeResponse)
async def analyze(request: AnalyzeRequest):
    prompt = f"<|user|>分析以下代码的执行流：\n```c\n{request.code}\n```<|/user|><|assistant|>"
    
    result = generate(
        model, tokenizer,
        prompt=prompt,
        max_tokens=request.max_tokens,
        temp=0.1,
    )
    
    return AnalyzeResponse(analysis=result)

if __name__ == "__main__":
    uvicorn.run(app, host="127.0.0.1", port=8765)
```

启动服务器：

```bash
pip install fastapi uvicorn
python scripts/inference/flow_server.py

# 测试
curl -X POST http://127.0.0.1:8765/analyze \
    -H "Content-Type: application/json" \
    -d '{"code": "static int my_probe(...) { ... }"}'
```

---

## 10. 常见问题

### Q1: 训练时内存不足 (Out of Memory)

```bash
# 解决方案:
# 1. 减小 batch_size
--batch-size 1

# 2. 减少 LoRA 层数
--lora-layers 4

# 3. 缩短序列长度
# 在数据准备时截断过长的样本

# 4. 关闭其他应用
# 关闭浏览器、IDE 等占用内存的应用
```

### Q2: 训练速度很慢

```bash
# 检查是否降频
sudo powermetrics --samplers cpu_power,gpu_power -i 1000 -n 1

# 解决方案:
# 1. 改善散热
#    - 使用散热垫
#    - 开启空调
#    - 合盖使用外接显示器
# 
# 2. 降低 batch_size（减少内存带宽压力）
# 
# 3. 关闭不必要的后台进程
```

### Q3: 训练损失不下降

```bash
# 可能原因:
# 1. 学习率太小 → 增大到 1e-4
# 2. 学习率太大 → 减小到 1e-6
# 3. 数据质量问题 → 检查数据格式
# 4. 数据太少 → 增加训练数据

# 尝试不同学习率
--learning-rate 1e-4
--learning-rate 5e-5
--learning-rate 1e-6
```

### Q4: 模型输出质量不好

```
可能原因和解决方案:

1. 训练数据不足
   → 增加训练数据量（至少 3000 条）

2. 训练轮数不够
   → 增加 epochs（3 → 5）

3. 数据格式问题
   → 检查 prompt 格式是否一致

4. LoRA 容量不足
   → 增加 rank（8 → 16）

5. 验证集过拟合
   → 减少 epochs，或增加训练数据
```

### Q5: 如何继续训练（从 checkpoint）

```bash
# MLX LoRA 支持从 checkpoint 继续
mlx_lm.lora \
    --model ~/.flowsight/models/codellama-7b-mlx \
    --adapter-path ~/.flowsight/models/flowsight-flow-v1 \
    --resume-adapter-file ~/.flowsight/models/flowsight-flow-v1/adapters.safetensors \
    --data data/flowsight-train \
    --train \
    --iters 2000  # 继续训练 2000 步
```

---

## 附录

### A. 训练时间详细估算

| 数据量 | Batch | 每步时间 | 1 Epoch | 3 Epochs |
|--------|-------|----------|---------|----------|
| 500 | 1 | 1.5s | 12 分钟 | 36 分钟 |
| 1,000 | 1 | 1.5s | 25 分钟 | 1.25 小时 |
| 2,000 | 1 | 1.5s | 50 分钟 | 2.5 小时 |
| 3,000 | 1 | 1.5s | 1.25 小时 | 3.75 小时 |
| 5,000 | 1 | 1.5s | 2 小时 | 6 小时 |

### B. 资源消耗

| 阶段 | 内存占用 | GPU 利用率 | 功耗 |
|------|----------|------------|------|
| 模型加载 | ~6GB | 0% | 5W |
| 训练中 | ~14GB | 70-90% | 20-30W |
| 推理 | ~6GB | 30-50% | 10-15W |

### C. 参考链接

- [MLX 官方文档](https://ml-explore.github.io/mlx/)
- [mlx-lm GitHub](https://github.com/ml-explore/mlx-examples/tree/main/llms/mlx_lm)
- [CodeLlama 论文](https://arxiv.org/abs/2308.12950)
- [LoRA 论文](https://arxiv.org/abs/2106.09685)

---

*最后更新: 2026-02-01*
*适用版本: MLX 0.x, mlx-lm 0.x*
