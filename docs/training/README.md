# FlowSight AI 训练文档

> 训练专用 AI 模型，辅助 FlowSight 生成函数执行流

## 训练方案对比

| 方案 | 硬件要求 | 模型大小 | 训练时间 | 推荐度 |
|------|----------|----------|----------|--------|
| **本地 MLX** | MacBook M3 24GB | 7B (量化) | 2-6 小时 | ⭐⭐⭐ 推荐 |
| 云端 GPU | A100 40GB | 6.7B | 30-60 分钟 | 可选 |

## 推荐方案：本地 MLX 训练

在 Apple Silicon Mac 上使用 MLX 框架训练，无需云 GPU。

📂 **[local-mlx/](local-mlx/README.md)**

| 文档 | 说明 |
|------|------|
| [README.md](local-mlx/README.md) | 方案概述 |
| [TUTORIAL.md](local-mlx/TUTORIAL.md) | **🎓 手把手教程**（新手必看） |
| [MLX-LORA-TRAINING-PLAN.md](local-mlx/MLX-LORA-TRAINING-PLAN.md) | 完整训练计划 |
| [QUICK-START.md](local-mlx/QUICK-START.md) | 命令速查 |
| [KERNEL-COVERAGE-PLAN.md](local-mlx/KERNEL-COVERAGE-PLAN.md) | Linux 内核覆盖计划 |

### 快速开始

```bash
# 1. 安装依赖
pip install mlx mlx-lm

# 2. 下载模型
mlx_lm.convert --hf-path codellama/CodeLlama-7b-hf -q

# 3. 准备数据
python scripts/training/generate_data.py

# 4. 开始训练
python scripts/training/train.py
```

## 可选方案：云端 GPU 训练

使用 A100 等高性能 GPU 进行全量训练。

📂 **[cloud-training/](cloud-training/)**

| 文档 | 说明 |
|------|------|
| [AI-TRAINING-GUIDE.md](cloud-training/AI-TRAINING-GUIDE.md) | 云端训练指南 |
| [TRAINING-DATA-PLAN.md](cloud-training/TRAINING-DATA-PLAN.md) | 训练数据规划 |

## 训练数据

训练数据生成脚本位于 `scripts/training/`:

```
scripts/training/
├── generate_data.py   # 生成训练数据
├── train.py           # 训练脚本
└── test.py            # 测试脚本
```

## 模型用途

训练后的模型用于：

1. **格式化输出** - 将分析结果转为 Mermaid 图、表格
2. **自然语言解释** - 用中文解释代码功能
3. **模式建议** - 帮助识别未知的内核模式

**注意**：核心分析不依赖 AI，基于知识库的模式匹配保证 100% 准确。
