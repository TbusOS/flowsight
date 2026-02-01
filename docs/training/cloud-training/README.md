# 云端 GPU 训练方案

> 使用 A100 等高性能 GPU 进行全量微调

## ⚠️ 注意

此方案需要 A100 40GB+ GPU，适合：
- 需要训练更大模型 (6.7B+)
- 有云 GPU 资源
- 需要更快的训练速度

**对于大多数用户，推荐使用 [本地 MLX 方案](../local-mlx/README.md)**

## 文档

| 文档 | 说明 |
|------|------|
| [AI-TRAINING-GUIDE.md](AI-TRAINING-GUIDE.md) | 完整训练流程 |
| [TRAINING-DATA-PLAN.md](TRAINING-DATA-PLAN.md) | 数据收集策略 |

## 硬件要求

| 配置 | 最低要求 | 推荐配置 |
|------|----------|----------|
| GPU | A10 24GB | A100 40GB |
| 内存 | 64GB | 128GB |
| 存储 | 100GB SSD | 500GB NVMe |

## 快速开始

```bash
# 1. 准备环境
pip install torch transformers peft accelerate

# 2. 下载基础模型
huggingface-cli download deepseek-ai/deepseek-coder-6.7b-base

# 3. 准备数据
python scripts/prepare_data.py

# 4. 开始训练
accelerate launch train.py
```

详见 [AI-TRAINING-GUIDE.md](AI-TRAINING-GUIDE.md)
