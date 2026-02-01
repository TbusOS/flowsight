# 快速开始：本地 MLX 训练

> 10 分钟上手指南

## 1. 安装环境（5 分钟）

```bash
# 创建虚拟环境
python3 -m venv ~/.venv/flowsight-train
source ~/.venv/flowsight-train/bin/activate

# 安装依赖
pip install mlx mlx-lm transformers datasets huggingface_hub tqdm pyyaml

# 验证
python -c "import mlx_lm; print('Ready!')"
```

## 2. 下载模型（10-30 分钟，取决于网速）

```bash
# 创建目录
mkdir -p ~/.flowsight/models

# 下载并转换 CodeLlama-7B（自动量化）
mlx_lm.convert \
    --hf-path codellama/CodeLlama-7b-hf \
    --mlx-path ~/.flowsight/models/codellama-7b-mlx \
    -q
```

## 3. 准备数据（5 分钟）

```bash
cd /Users/sky/linux-kernel/usb-learn/flowsight

# 创建数据目录
mkdir -p data/flowsight-train

# 创建简单的训练数据（示例）
cat > data/flowsight-train/train.jsonl << 'EOF'
{"text": "<|user|>分析 probe 函数执行流<|/user|><|assistant|>{\"trigger\": \"设备匹配\", \"context\": \"process\"}"}
EOF

# 复制为验证集
cp data/flowsight-train/train.jsonl data/flowsight-train/valid.jsonl
```

## 4. 开始训练

```bash
# 基础训练命令
mlx_lm.lora \
    --model ~/.flowsight/models/codellama-7b-mlx \
    --data data/flowsight-train \
    --train \
    --batch-size 1 \
    --lora-layers 8 \
    --learning-rate 1e-5 \
    --iters 100 \
    --adapter-path ~/.flowsight/models/flowsight-flow-v1
```

## 5. 测试模型

```bash
# 使用训练后的模型生成
mlx_lm.generate \
    --model ~/.flowsight/models/codellama-7b-mlx \
    --adapter-path ~/.flowsight/models/flowsight-flow-v1 \
    --prompt "分析以下 USB 驱动的执行流：static int my_probe(...)" \
    --max-tokens 200
```

---

## 常用命令速查

| 操作 | 命令 |
|------|------|
| 激活环境 | `source ~/.venv/flowsight-train/bin/activate` |
| 训练 | `mlx_lm.lora --model ... --train` |
| 测试 | `mlx_lm.lora --model ... --test` |
| 生成 | `mlx_lm.generate --model ... --prompt ...` |
| 继续训练 | 添加 `--resume-adapter-file ...` |

## 预计时间

- 100 条数据，100 步：**3-5 分钟**（测试用）
- 1000 条数据，3 epochs：**1.5-2 小时**（初版）
- 3000 条数据，3 epochs：**5-6 小时**（正式版）

## 注意事项

1. **散热**：长时间训练请使用散热垫
2. **内存**：关闭其他大型应用
3. **中断**：Ctrl+C 中断不会丢失 checkpoint
4. **恢复**：使用 `--resume-adapter-file` 继续训练

---

详细文档请参考 [MLX-LORA-TRAINING-PLAN.md](MLX-LORA-TRAINING-PLAN.md)
