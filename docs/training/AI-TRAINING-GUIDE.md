# FlowSight AI 模型训练指南

> 本文档提供完整的模型训练步骤，从数据准备到最终部署。

---

## 目录

1. [概述](#1-概述)
2. [环境准备](#2-环境准备)
3. [数据准备](#3-数据准备)
4. [模型微调](#4-模型微调)
5. [知识蒸馏](#5-知识蒸馏)
6. [模型量化](#6-模型量化)
7. [测试验证](#7-测试验证)
8. [部署集成](#8-部署集成)

---

## 1. 概述

### 1.1 训练目标

| 阶段 | 输入 | 输出 |
|------|------|------|
| 微调 | DeepSeek-Coder-6.7B | flowsight-code-6.7b |
| 蒸馏 | flowsight-code-6.7b | flowsight-code-1.3b |
| 量化 | flowsight-code-1.3b | flowsight-code-1.3b.gguf (~1GB) |

### 1.2 硬件要求

| 阶段 | 最低配置 | 推荐配置 |
|------|----------|----------|
| 微调 | A100 40GB (QLoRA) | A100 80GB (全参数) |
| 蒸馏 | A100 40GB | A100 40GB |
| 量化 | 16GB 内存 CPU | 本地 MacBook 即可 |

### 1.3 预估成本和时间

| 阶段 | 时间 | GPU 成本 |
|------|------|----------|
| 数据准备 | 1-2 周 | 免费（本地） |
| 微调 | 15-30 小时 | ¥150-400 |
| 蒸馏 | 5-10 小时 | ¥25-50 |
| 量化 | 1-2 小时 | 免费（本地） |
| **总计** | **2-3 周** | **¥200-500** |

---

## 2. 环境准备

### 2.1 云 GPU 租用（推荐 AutoDL）

```bash
# 1. 注册 AutoDL 账号
#    https://www.autodl.com

# 2. 创建实例
#    - 镜像：PyTorch 2.1 + CUDA 12.1
#    - GPU：A100 80GB（最优）或 A100 40GB
#    - 系统盘：30GB
#    - 数据盘：100GB（存放模型和数据）

# 3. SSH 连接
ssh -p <端口> root@<IP地址>
```

### 2.2 环境配置

```bash
# 创建项目目录
mkdir -p /root/flowsight-training
cd /root/flowsight-training

# 创建 Python 环境
conda create -n flowsight python=3.10 -y
conda activate flowsight

# 安装依赖
pip install torch==2.1.0 transformers==4.36.0 datasets==2.16.0
pip install peft==0.7.0 accelerate==0.25.0 bitsandbytes==0.41.0
pip install wandb tqdm scipy

# 下载基础模型
python -c "
from transformers import AutoModelForCausalLM, AutoTokenizer
model_name = 'deepseek-ai/deepseek-coder-6.7b-base'
tokenizer = AutoTokenizer.from_pretrained(model_name, trust_remote_code=True)
model = AutoModelForCausalLM.from_pretrained(model_name, trust_remote_code=True)
tokenizer.save_pretrained('./models/deepseek-coder-6.7b')
model.save_pretrained('./models/deepseek-coder-6.7b')
print('Model downloaded!')
"
```

---

## 3. 数据准备

### 3.1 数据格式

```json
// data/train.jsonl - 每行一个样本
{
  "instruction": "分析以下代码中函数指针 fp 在调用时指向哪个函数",
  "input": "void handler_a(void) { printf(\"A\"); }\nvoid handler_b(void) { printf(\"B\"); }\n\nvoid (*fp)(void);\nfp = handler_a;\nif (condition) fp = handler_b;\nfp();",
  "output": "fp 可能指向以下函数：\n1. handler_a（在 fp = handler_a 处赋值）\n2. handler_b（在 if 条件为真时赋值）\n\n调用 fp() 时，实际执行的函数取决于 condition 的值。"
}
```

### 3.2 数据收集脚本

```python
# scripts/collect_data.py
"""
从 FlowSight 知识库和开源项目收集训练数据
"""

import json
import os
from pathlib import Path

class DataCollector:
    def __init__(self, output_dir="data"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        self.samples = []
    
    def add_function_pointer_sample(self, code, pointer_name, targets, explanation):
        """添加函数指针分析样本"""
        self.samples.append({
            "instruction": f"分析以下代码中函数指针 {pointer_name} 在调用时指向哪个函数",
            "input": code,
            "output": explanation
        })
    
    def add_callback_timing_sample(self, code, callback_name, trigger_explanation):
        """添加回调触发时机样本"""
        self.samples.append({
            "instruction": f"分析以下代码中 {callback_name} 函数何时会被调用",
            "input": code,
            "output": trigger_explanation
        })
    
    def add_async_pattern_sample(self, code, pattern_type, explanation):
        """添加异步模式识别样本"""
        self.samples.append({
            "instruction": "识别以下代码中的异步模式，并解释执行流程",
            "input": code,
            "output": f"异步模式类型：{pattern_type}\n\n{explanation}"
        })
    
    def save(self, filename="train.jsonl"):
        """保存数据集"""
        output_path = self.output_dir / filename
        with open(output_path, 'w', encoding='utf-8') as f:
            for sample in self.samples:
                f.write(json.dumps(sample, ensure_ascii=False) + '\n')
        print(f"Saved {len(self.samples)} samples to {output_path}")

# 示例：从 Linux 内核知识库生成样本
def generate_linux_kernel_samples(collector):
    """生成 Linux 内核相关的训练样本"""
    
    # USB 驱动 probe 回调
    collector.add_callback_timing_sample(
        code="""
static int my_probe(struct usb_interface *intf, const struct usb_device_id *id)
{
    printk("Device connected!\\n");
    return 0;
}

static struct usb_driver my_driver = {
    .name = "my_usb_driver",
    .probe = my_probe,
    .disconnect = my_disconnect,
    .id_table = my_id_table,
};

module_usb_driver(my_driver);
""",
        callback_name="my_probe",
        trigger_explanation="""my_probe 函数的调用时机：

1. **不是** 在 insmod 加载模块时调用
2. **而是** 在 USB 设备插入且 ID 匹配时调用

完整调用链：
- USB 设备插入
- usb_hub_port_connect()
- usb_new_device()
- device_add()
- bus_probe_device()
- driver_probe_device()
- really_probe()
- usb_probe_interface()
- my_probe()  ← 这里才执行

执行上下文：进程上下文（可以睡眠）"""
    )
    
    # WorkQueue 异步模式
    collector.add_async_pattern_sample(
        code="""
static void my_work_handler(struct work_struct *work)
{
    struct my_device *dev = container_of(work, struct my_device, work);
    // 处理耗时操作
}

static irqreturn_t my_irq_handler(int irq, void *dev_id)
{
    struct my_device *dev = dev_id;
    // 快速处理
    schedule_work(&dev->work);
    return IRQ_HANDLED;
}

static int my_probe(...)
{
    INIT_WORK(&dev->work, my_work_handler);
    request_irq(irq, my_irq_handler, ...);
}
""",
        pattern_type="WorkQueue（工作队列）",
        explanation="""执行流程：

1. **绑定阶段**（probe 中）：
   INIT_WORK(&dev->work, my_work_handler)
   → 将 my_work_handler 绑定到 dev->work

2. **中断阶段**（硬中断上下文，不可睡眠）：
   my_irq_handler() 被触发
   → schedule_work(&dev->work) 提交任务
   → 立即返回，不等待执行

3. **工作执行阶段**（进程上下文，可睡眠）：
   kworker 线程被调度
   → worker_thread()
   → process_one_work()
   → my_work_handler()  ← 这里才真正执行

时间线关系：
中断发生 → 快速返回 → ... → 调度器调度 → work 执行"""
    )

if __name__ == "__main__":
    collector = DataCollector()
    generate_linux_kernel_samples(collector)
    # TODO: 添加更多样本
    collector.save()
```

### 3.3 数据标注工具

```python
# scripts/label_tool.py
"""
交互式数据标注工具
"""

import json
from pathlib import Path

def label_function_pointer(code_file):
    """标注函数指针目标"""
    print("=" * 60)
    print("函数指针标注工具")
    print("=" * 60)
    
    with open(code_file, 'r') as f:
        code = f.read()
    
    print("\n代码内容：")
    print("-" * 40)
    print(code)
    print("-" * 40)
    
    pointer_name = input("\n函数指针名称: ")
    targets = input("可能的目标函数（逗号分隔）: ").split(',')
    explanation = input("解释说明: ")
    
    sample = {
        "instruction": f"分析以下代码中函数指针 {pointer_name} 在调用时指向哪个函数",
        "input": code,
        "output": explanation
    }
    
    return sample

# 批量标注
def batch_label(code_dir, output_file):
    """批量标注代码文件"""
    samples = []
    code_files = list(Path(code_dir).glob("*.c"))
    
    for i, code_file in enumerate(code_files):
        print(f"\n[{i+1}/{len(code_files)}] {code_file.name}")
        sample = label_function_pointer(code_file)
        samples.append(sample)
        
        # 每 10 个保存一次
        if (i + 1) % 10 == 0:
            with open(output_file, 'w') as f:
                for s in samples:
                    f.write(json.dumps(s, ensure_ascii=False) + '\n')
            print(f"已保存 {len(samples)} 个样本")
    
    # 最终保存
    with open(output_file, 'w') as f:
        for s in samples:
            f.write(json.dumps(s, ensure_ascii=False) + '\n')
    print(f"完成！共 {len(samples)} 个样本")
```

### 3.4 数据集规模建议

| 数据类型 | 最低数量 | 推荐数量 | 来源 |
|----------|----------|----------|------|
| 函数指针分析 | 2,000 | 5,000+ | 开源项目 + 人工标注 |
| 回调触发时机 | 1,000 | 3,000+ | 知识库 + 人工标注 |
| 异步模式识别 | 1,000 | 3,000+ | 知识库 + 开源项目 |
| **总计** | **4,000** | **10,000+** | |

---

## 4. 模型微调

### 4.1 全参数微调（推荐，需要 A100 80GB）

```python
# scripts/train_full.py
"""
全参数微调 DeepSeek-Coder-6.7B
需要 A100 80GB GPU
"""

import torch
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    TrainingArguments,
    Trainer,
    DataCollatorForSeq2Seq,
)
from datasets import load_dataset

# 配置
MODEL_NAME = "./models/deepseek-coder-6.7b"
OUTPUT_DIR = "./models/flowsight-code-6.7b"
DATA_FILE = "./data/train.jsonl"

# 加载模型
print("Loading model...")
tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME, trust_remote_code=True)
model = AutoModelForCausalLM.from_pretrained(
    MODEL_NAME,
    torch_dtype=torch.bfloat16,
    trust_remote_code=True,
)

# 加载数据
print("Loading data...")
dataset = load_dataset("json", data_files=DATA_FILE)["train"]

def format_sample(sample):
    """格式化样本为模型输入"""
    prompt = f"""### Instruction:
{sample['instruction']}

### Input:
{sample['input']}

### Response:
{sample['output']}"""
    return {"text": prompt}

dataset = dataset.map(format_sample)

def tokenize(sample):
    return tokenizer(
        sample["text"],
        truncation=True,
        max_length=2048,
        padding="max_length",
    )

dataset = dataset.map(tokenize, batched=True, remove_columns=dataset.column_names)

# 训练配置
training_args = TrainingArguments(
    output_dir=OUTPUT_DIR,
    num_train_epochs=3,
    per_device_train_batch_size=2,
    gradient_accumulation_steps=8,
    learning_rate=2e-5,
    warmup_ratio=0.1,
    logging_steps=10,
    save_steps=500,
    save_total_limit=3,
    bf16=True,
    gradient_checkpointing=True,
    report_to="wandb",  # 可选：使用 wandb 监控
)

# 训练
print("Starting training...")
trainer = Trainer(
    model=model,
    args=training_args,
    train_dataset=dataset,
    data_collator=DataCollatorForSeq2Seq(tokenizer, padding=True),
)

trainer.train()
trainer.save_model(OUTPUT_DIR)
tokenizer.save_pretrained(OUTPUT_DIR)
print(f"Model saved to {OUTPUT_DIR}")
```

### 4.2 QLoRA 微调（备选，A100 40GB 可用）

```python
# scripts/train_qlora.py
"""
QLoRA 微调 DeepSeek-Coder-6.7B
A100 40GB 或更小显存可用
"""

import torch
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    TrainingArguments,
    Trainer,
    BitsAndBytesConfig,
)
from peft import LoraConfig, get_peft_model, prepare_model_for_kbit_training
from datasets import load_dataset

# 配置
MODEL_NAME = "./models/deepseek-coder-6.7b"
OUTPUT_DIR = "./models/flowsight-code-6.7b-qlora"
DATA_FILE = "./data/train.jsonl"

# 4-bit 量化配置
bnb_config = BitsAndBytesConfig(
    load_in_4bit=True,
    bnb_4bit_quant_type="nf4",
    bnb_4bit_compute_dtype=torch.bfloat16,
    bnb_4bit_use_double_quant=True,
)

# 加载模型
print("Loading model with 4-bit quantization...")
tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME, trust_remote_code=True)
model = AutoModelForCausalLM.from_pretrained(
    MODEL_NAME,
    quantization_config=bnb_config,
    trust_remote_code=True,
)

# 准备 LoRA
model = prepare_model_for_kbit_training(model)

lora_config = LoraConfig(
    r=64,
    lora_alpha=128,
    target_modules=["q_proj", "k_proj", "v_proj", "o_proj", "gate_proj", "up_proj", "down_proj"],
    lora_dropout=0.05,
    bias="none",
    task_type="CAUSAL_LM",
)

model = get_peft_model(model, lora_config)
model.print_trainable_parameters()

# ... 数据处理和训练代码与全参数微调相同 ...

# 训练配置（QLoRA 可以用更大 batch size）
training_args = TrainingArguments(
    output_dir=OUTPUT_DIR,
    num_train_epochs=3,
    per_device_train_batch_size=4,
    gradient_accumulation_steps=4,
    learning_rate=2e-4,  # QLoRA 学习率可以更大
    warmup_ratio=0.1,
    logging_steps=10,
    save_steps=500,
    bf16=True,
)

# 训练并保存
trainer = Trainer(...)
trainer.train()

# 合并 LoRA 权重
model = model.merge_and_unload()
model.save_pretrained(OUTPUT_DIR)
```

### 4.3 运行训练

```bash
# SSH 连接到云 GPU
ssh -p <端口> root@<IP地址>

# 激活环境
cd /root/flowsight-training
conda activate flowsight

# 运行全参数微调（A100 80GB）
python scripts/train_full.py

# 或运行 QLoRA（A100 40GB）
python scripts/train_qlora.py

# 监控训练（新终端）
# 如果使用 wandb，可以在网页查看
# 或者查看日志
tail -f ./models/flowsight-code-6.7b/trainer_log.txt
```

---

## 5. 知识蒸馏

### 5.1 蒸馏脚本

```python
# scripts/distill.py
"""
从 6.7B 模型蒸馏到 1.3B 模型
"""

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, Trainer, TrainingArguments
from datasets import load_dataset

# 教师模型（微调后的 6.7B）
TEACHER_MODEL = "./models/flowsight-code-6.7b"
# 学生模型（原版 1.3B）
STUDENT_MODEL = "deepseek-ai/deepseek-coder-1.3b-base"
OUTPUT_DIR = "./models/flowsight-code-1.3b"

print("Loading teacher model...")
teacher = AutoModelForCausalLM.from_pretrained(TEACHER_MODEL, torch_dtype=torch.bfloat16)
teacher.eval()

print("Loading student model...")
student = AutoModelForCausalLM.from_pretrained(STUDENT_MODEL, torch_dtype=torch.bfloat16)
tokenizer = AutoTokenizer.from_pretrained(STUDENT_MODEL)

# 蒸馏训练
class DistillationTrainer(Trainer):
    def __init__(self, teacher, temperature=2.0, alpha=0.5, **kwargs):
        super().__init__(**kwargs)
        self.teacher = teacher
        self.temperature = temperature
        self.alpha = alpha
    
    def compute_loss(self, model, inputs, return_outputs=False):
        # 学生输出
        outputs = model(**inputs)
        student_logits = outputs.logits
        
        # 教师输出
        with torch.no_grad():
            teacher_outputs = self.teacher(**inputs)
            teacher_logits = teacher_outputs.logits
        
        # 软标签损失
        soft_loss = torch.nn.functional.kl_div(
            torch.nn.functional.log_softmax(student_logits / self.temperature, dim=-1),
            torch.nn.functional.softmax(teacher_logits / self.temperature, dim=-1),
            reduction="batchmean",
        ) * (self.temperature ** 2)
        
        # 硬标签损失
        hard_loss = outputs.loss
        
        # 组合损失
        loss = self.alpha * soft_loss + (1 - self.alpha) * hard_loss
        
        return (loss, outputs) if return_outputs else loss

# 训练配置
training_args = TrainingArguments(
    output_dir=OUTPUT_DIR,
    num_train_epochs=2,
    per_device_train_batch_size=4,
    gradient_accumulation_steps=4,
    learning_rate=5e-5,
    warmup_ratio=0.1,
    bf16=True,
    save_steps=500,
)

# 加载数据
dataset = load_dataset("json", data_files="./data/train.jsonl")["train"]
# ... tokenize ...

trainer = DistillationTrainer(
    teacher=teacher,
    model=student,
    args=training_args,
    train_dataset=dataset,
)

trainer.train()
student.save_pretrained(OUTPUT_DIR)
tokenizer.save_pretrained(OUTPUT_DIR)
```

---

## 6. 模型量化

### 6.1 GGUF 量化（可在本地 MacBook 执行）

```bash
# 在本地 MacBook 上执行

# 1. 安装 llama.cpp
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp
make

# 2. 下载蒸馏后的模型（从云服务器）
scp -P <端口> root@<IP>:/root/flowsight-training/models/flowsight-code-1.3b ./

# 3. 转换为 GGUF 格式
python convert.py ./flowsight-code-1.3b --outtype f16 --outfile flowsight-code-1.3b.gguf

# 4. 量化为 INT4
./quantize flowsight-code-1.3b.gguf flowsight-code-1.3b-q4_k_m.gguf q4_k_m

# 5. 测试
./main -m flowsight-code-1.3b-q4_k_m.gguf -p "分析以下代码中函数指针指向哪个函数：" -n 256
```

### 6.2 量化后大小

| 量化方法 | 模型大小 | 质量损失 | 推荐 |
|----------|----------|----------|------|
| F16 | ~2.6GB | 无 | 有足够空间时 |
| Q8_0 | ~1.4GB | 很小 | 追求质量 |
| **Q4_K_M** | **~0.8GB** | **小** | **推荐** |
| Q4_0 | ~0.7GB | 中等 | 极限压缩 |

---

## 7. 测试验证

### 7.1 评估脚本

```python
# scripts/evaluate.py
"""
评估模型在代码执行流分析任务上的表现
"""

import json
from transformers import AutoModelForCausalLM, AutoTokenizer

def evaluate_model(model_path, test_file):
    print(f"Loading model from {model_path}...")
    tokenizer = AutoTokenizer.from_pretrained(model_path)
    model = AutoModelForCausalLM.from_pretrained(model_path)
    model.eval()
    
    # 加载测试数据
    with open(test_file, 'r') as f:
        test_samples = [json.loads(line) for line in f]
    
    correct = 0
    total = len(test_samples)
    
    for sample in test_samples:
        prompt = f"""### Instruction:
{sample['instruction']}

### Input:
{sample['input']}

### Response:
"""
        inputs = tokenizer(prompt, return_tensors="pt")
        outputs = model.generate(**inputs, max_new_tokens=512)
        response = tokenizer.decode(outputs[0], skip_special_tokens=True)
        
        # 简单评估：检查关键词是否在回答中
        expected_keywords = extract_keywords(sample['output'])
        if all(kw in response for kw in expected_keywords):
            correct += 1
    
    accuracy = correct / total * 100
    print(f"Accuracy: {accuracy:.2f}% ({correct}/{total})")
    return accuracy

if __name__ == "__main__":
    evaluate_model("./models/flowsight-code-1.3b", "./data/test.jsonl")
```

---

## 8. 部署集成

### 8.1 集成到 FlowSight

模型训练完成后，将 `.gguf` 文件放到：

```
~/.flowsight/models/
├── flowsight-code-v1.gguf       # 量化后的模型 (~0.8GB)
├── flowsight-code-v1.sha256     # 校验文件
└── version.json                  # 版本信息
```

FlowSight IDE 会自动加载并使用该模型。

---

## 附录：常见问题

### Q: 训练过程中显存不足怎么办？

```bash
# 1. 减小 batch size
per_device_train_batch_size=1

# 2. 增加梯度累积
gradient_accumulation_steps=16

# 3. 启用梯度检查点
gradient_checkpointing=True

# 4. 使用 QLoRA 而不是全参数微调
```

### Q: 如何判断训练是否成功？

```
1. Loss 曲线平稳下降
2. 验证集准确率提升
3. 人工测试回答质量
```

### Q: 训练中断了怎么办？

```bash
# 从最近的 checkpoint 恢复
python scripts/train_full.py --resume_from_checkpoint ./models/flowsight-code-6.7b/checkpoint-1000
```

---

*最后更新: 2025-01-07*

