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
        print("\n请先下载模型：")
        print("   mlx_lm.convert --hf-path codellama/CodeLlama-7b-hf \\")
        print(f"       --mlx-path {MODEL} -q")
        return 1
    
    # 检查数据是否存在
    train_file = Path(f"{DATA}/train.jsonl")
    if not train_file.exists():
        print(f"❌ 训练数据不存在: {train_file}")
        print("\n请先运行: python scripts/training/generate_data.py")
        return 1
    
    # 统计数据量
    with open(train_file) as f:
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
    
    try:
        input("按 Enter 开始训练 (Ctrl+C 取消)...")
    except KeyboardInterrupt:
        print("\n已取消")
        return 0
    
    print()
    
    # 创建输出目录
    Path(OUTPUT).mkdir(parents=True, exist_ok=True)
    
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
    result = subprocess.run(cmd)
    
    print("-" * 50)
    print()
    
    # 检查结果
    adapter_file = Path(OUTPUT) / "adapters.safetensors"
    if adapter_file.exists():
        size_mb = adapter_file.stat().st_size / 1024 / 1024
        print(f"✅ 训练完成！")
        print(f"   模型保存在: {OUTPUT}")
        print(f"   适配器大小: {size_mb:.1f} MB")
        print()
        print("下一步: 测试模型")
        print("   python scripts/training/test.py")
        return 0
    else:
        print("⚠️  训练可能未完成，请检查输出")
        return 1


if __name__ == "__main__":
    sys.exit(main() or 0)
