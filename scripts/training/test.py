#!/usr/bin/env python3
"""
测试训练后的模型
运行: python scripts/training/test.py
"""

import os
import sys
from pathlib import Path

def main():
    MODEL = os.path.expanduser("~/.flowsight/models/codellama-7b-mlx")
    ADAPTER = os.path.expanduser("~/.flowsight/models/flowsight-flow-v1")
    
    # 检查模型和适配器是否存在
    if not Path(MODEL).exists():
        print(f"❌ 基础模型不存在: {MODEL}")
        return 1
    
    if not Path(ADAPTER).exists():
        print(f"❌ LoRA 适配器不存在: {ADAPTER}")
        print("\n请先运行训练: python scripts/training/train.py")
        return 1
    
    # 延迟导入（加载时间较长）
    print("加载模型...")
    try:
        from mlx_lm import load, generate
    except ImportError:
        print("❌ mlx_lm 未安装")
        print("\n请运行: pip install mlx mlx-lm")
        return 1
    
    model, tokenizer = load(MODEL, adapter_path=ADAPTER)
    print("模型加载完成！\n")
    
    # 测试用例
    tests = [
        # 测试1: 中断处理
        """分析以下代码的执行流：
```c
static irqreturn_t my_irq(int irq, void *data)
{
    struct my_dev *dev = data;
    schedule_work(&dev->work);
    return IRQ_HANDLED;
}
```""",
        # 测试2: 定时器
        """分析以下定时器代码：
```c
static void my_timer(struct timer_list *t)
{
    struct my_dev *dev = from_timer(dev, t, timer);
    process_data(dev);
    mod_timer(t, jiffies + HZ);
}
```""",
        # 测试3: probe 函数
        """分析以下 probe 函数：
```c
static int my_probe(struct platform_device *pdev)
{
    INIT_WORK(&dev->work, handler);
    request_irq(irq, my_irq, 0, "mydev", dev);
    return 0;
}
```""",
    ]
    
    print("=" * 50)
    print("开始测试")
    print("=" * 50)
    
    for i, test in enumerate(tests, 1):
        print(f"\n--- 测试 {i} ---")
        # 显示问题的前几行
        lines = test.strip().split('\n')
        preview = '\n'.join(lines[:3])
        print(f"问题:\n{preview}...")
        
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
        
        # 移除可能的结束标记
        if "<|" in answer:
            answer = answer.split("<|")[0]
            
        print(f"\n回答:\n{answer.strip()}")
        print("-" * 50)
    
    print("\n✅ 测试完成！")
    print("\n你可以使用以下代码在 Python 中调用模型:")
    print("""
from mlx_lm import load, generate

model, tokenizer = load(
    "~/.flowsight/models/codellama-7b-mlx",
    adapter_path="~/.flowsight/models/flowsight-flow-v1"
)

prompt = "<|user|>你的问题<|/user|><|assistant|>"
response = generate(model, tokenizer, prompt=prompt, max_tokens=300)
print(response)
""")
    
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)
