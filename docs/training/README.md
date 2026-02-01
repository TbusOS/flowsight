# FlowSight AI 模型训练文档

> 本目录包含 AI 模型训练相关的完整文档。

## 文档列表

### 🆕 本地训练方案（推荐）

#### [local-mlx/](local-mlx/) - MacBook M 系列本地训练
在 Apple Silicon Mac 上使用 MLX 框架训练，无需云 GPU：
- [快速开始](local-mlx/QUICK-START.md) - 10 分钟上手
- [完整计划](local-mlx/MLX-LORA-TRAINING-PLAN.md) - 详细训练方案

| 方案 | 硬件要求 | 训练时间 | 成本 |
|------|----------|----------|------|
| **本地 MLX** | MacBook M3 24GB | 2-8 小时 | 免费 |
| 云端 GPU | A100 40-80GB | 15-30 小时 | ¥200-500 |

---

### 云端训练方案

### 1. [AI-TRAINING-GUIDE.md](AI-TRAINING-GUIDE.md)
AI 模型训练指南，提供完整的模型训练步骤：
- 环境准备（云 GPU 租用）
- 数据准备格式和收集脚本
- 模型微调（全参数 + QLoRA）
- 知识蒸馏（6.7B → 1.3B）
- 模型量化（GGUF）
- 测试验证
- 部署集成

### 2. [TRAINING-DATA-PLAN.md](TRAINING-DATA-PLAN.md)
训练数据完整计划，包含：
- 总体目标（70,000 样本）
- 训练策略与迭代计划
- 覆盖范围（5 个 Phase）
- 数据规模估算
- 分阶段实施计划
- 成本预算

## 快速开始

### Step 1: 准备数据
```bash
# 生成训练数据
python scripts/collect_data.py --output data/train.jsonl
```

### Step 2: 训练模型
```bash
# 全参数微调（需要 A100 80GB）
python scripts/train_full.py

# 或 QLoRA 微调（A100 40GB 可用）
python scripts/train_qlora.py
```

### Step 3: 知识蒸馏
```bash
python scripts/distill.py
```

### Step 4: 模型量化
```bash
# 转换为 GGUF 格式
python convert.py ./flowsight-code-1.3b --outtype f16 --outfile flowsight-code-1.3b.gguf

# 量化为 INT4
./quantize flowsight-code-1.3b.gguf flowsight-code-1.3b-q4_k_m.gguf q4_k_m
```

## 训练流程概览

```
基础模型: DeepSeek-Coder-6.7B
    │
    ├── 微调 (A100 80GB, ~20h)
    │       │
    │       └──→ FlowSight-Linux-6.7B
    │               │
    │               ├── 知识蒸馏 (A100 40GB, ~10h)
    │               │       │
    │               │       └──→ FlowSight-Linux-1.3B
    │               │               │
    │               │               ├── GGUF 量化 (本地)
    │               │               │       │
    │               │               │       └──→ FlowSight-Linux-1.3B.gguf (~0.8GB)
    │               │               │
    │               │               └── 嵌入 IDE
    │               │
    │               └── 自学习 (用户反馈)
```

## 相关文档

- [docs/design/TECHNICAL-DESIGN.md](../design/TECHNICAL-DESIGN.md) - 技术架构设计
- [docs/design/PROJECT-PLAN-V2.md](../design/PROJECT-PLAN-V2.md) - 项目实施计划
- [docs/architecture/LOCAL-AI-DESIGN.md](../architecture/LOCAL-AI-DESIGN.md) - 本地 AI 设计
- [docs/architecture/AI-MODEL-SELECTION.md](../architecture/AI-MODEL-SELECTION.md) - AI 模型选择
