# FlowSight 本地 AI 训练 - MLX 方案

> 在 MacBook Air M3 (24GB) 上使用 Apple MLX 框架训练执行流生成模型

## 概述

本目录包含在 Apple Silicon Mac 上本地训练 AI 模型的完整指南，无需云 GPU。

| 项目 | 详情 |
|------|------|
| **目标硬件** | MacBook Air M3, 24GB 统一内存 |
| **框架** | Apple MLX |
| **基础模型** | CodeLlama-7B (量化版) |
| **训练方式** | LoRA 微调 |
| **用途** | 辅助 FlowSight 生成函数执行流 |

## 文档列表

| 文档 | 说明 |
|------|------|
| [MLX-LORA-TRAINING-PLAN.md](MLX-LORA-TRAINING-PLAN.md) | **完整训练计划**（推荐先读） |
| [QUICK-START.md](QUICK-START.md) | 快速开始指南 |

## 为什么选择 MLX？

1. **Apple 官方优化** - 专为 M 系列芯片设计
2. **统一内存优势** - CPU/GPU 共享 24GB，无需数据传输
3. **原生 Metal 支持** - 充分利用 GPU 算力
4. **简单易用** - API 类似 PyTorch，学习成本低

## 快速开始

```bash
# 1. 安装依赖
pip install mlx mlx-lm

# 2. 下载模型
mlx_lm.convert --hf-path codellama/CodeLlama-7b-hf -q

# 3. 准备数据
python scripts/prepare_training_data.py

# 4. 开始训练
mlx_lm.lora --model ./models/codellama-7b-mlx \
            --data ./data/flowsight-train \
            --batch-size 1 \
            --lora-layers 8
```

## 预计时间

| 数据量 | 训练时间 | 说明 |
|--------|----------|------|
| 1,000 条 | 1.5-2 小时 | 初版模型 |
| 3,000 条 | 5-6 小时 | 正式版本 |
| 5,000 条 | 8-10 小时 | 完整训练 |

## 相关文档

- [云端训练指南](../AI-TRAINING-GUIDE.md) - A100 GPU 训练方案
- [训练数据计划](../TRAINING-DATA-PLAN.md) - 数据收集策略
- [本地 AI 架构](../../architecture/LOCAL-AI-DESIGN.md) - 系统设计
