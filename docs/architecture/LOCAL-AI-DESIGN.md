# FlowSight 本地 AI 设计方案

> 精准优先 + 能力强 + 自学习 + 众包共享

## 1. 设计目标

| 目标 | 优先级 | 说明 |
|------|--------|------|
| **准确** | 🔴 最高 | 宁可说"不知道"，也不给错误信息 |
| **能力强** | 🔴 最高 | 能处理复杂场景，理解代码语义 |
| **可扩展** | 🟡 高 | 为 Android、RTOS 等打下基础 |
| **本地运行** | 🟡 高 | 无需网络，下载即用 |
| **自学习** | 🟢 中 | 从用户使用中学习，持续改进 |
| **众包共享** | 🟢 中 | 用户学习成果上传 GitHub，惠及所有人 |

## 2. 技术方案

### 2.1 核心选择：混合 AI 架构

**专用小模型 (必装) + DeepSeek LLM (可选)**

```
┌─────────────────────────────────────────────────────────────────┐
│                    混合 AI 架构                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  专用小模型层 (快速、精准、必装 ~200MB)                          │
│  ├── 函数指针分类器 - 快速筛选候选                               │
│  ├── 异步模式分类器 - 识别已知模式                               │
│  └── 代码嵌入模型 - 语义搜索                                     │
│                              │                                   │
│                              ▼                                   │
│  DeepSeek LLM 层 (强大、可选下载)                                │
│  ├── 复杂推理 - 专用模型无法处理时调用                           │
│  ├── 代码解释 - 生成自然语言解释                                 │
│  └── 未知模式 - 识别知识库未覆盖的新模式                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 模型组成

**专用小模型 (必装)**

| 模型 | 用途 | 大小 |
|------|------|------|
| 函数指针分类器 | 快速筛选候选目标 | ~50MB |
| 异步模式分类器 | 识别已知异步模式 | ~50MB |
| 代码嵌入模型 | 语义搜索、相似代码 | ~100MB |
| **合计** | | **~200MB** |

**DeepSeek LLM (可选下载)**

| 版本 | 基础模型 | 量化后大小 | 能力 | 适用场景 |
|------|----------|-----------|------|----------|
| **轻量版** | DeepSeek-Coder-1.3B | ~0.8GB | 基础 | 低配电脑、快速体验 |
| **标准版** | DeepSeek-Coder-6.7B | ~4GB | 强 | 推荐，大多数用户 |
| **完整版** | DeepSeek-Coder-33B | ~18GB | 最强 | 高配电脑、专业用户 |

### 2.3 混合方案优势

| 场景 | 处理方式 | 速度 | 需要 LLM |
|------|----------|------|----------|
| 已知模式识别 | 专用分类器 | 毫秒级 | ❌ |
| 语义搜索 | 嵌入模型 | 毫秒级 | ❌ |
| 复杂推理 | DeepSeek LLM | 秒级 | ✅ |
| 代码解释 | DeepSeek LLM | 秒级 | ✅ |
| 未知模式 | DeepSeek LLM | 秒级 | ✅ |

**优势**：
- 基础功能不需要下载大模型
- 常见场景毫秒级响应
- 需要更强能力时再调用 LLM
- 两者互补，各取所长

### 2.4 分发方式

```
IDE 安装包 (~100MB)
├── 专用小模型 (必装 ~200MB)
│   ├── funcptr_classifier.onnx
│   ├── async_classifier.onnx
│   └── code_embedder.onnx
│
└── DeepSeek LLM (可选下载)
    ├── flowsight-linux-1.3b.gguf  (~0.8GB) - 轻量版
    ├── flowsight-linux-6.7b.gguf  (~4GB)   - 标准版 ⭐推荐
    └── flowsight-linux-33b.gguf   (~18GB)  - 完整版
```

### 2.5 模型演进路线

```
v1.0 Linux 内核专用
├── 训练数据：70,000 样本
├── 覆盖：完整 Linux 内核子系统
├── 专用小模型：funcptr + async + embedder
└── DeepSeek LLM：1.3B / 6.7B / 33B 三版本
        │
        ▼
v2.0 + Android 框架
├── 增量训练：+30,000 样本
├── 新增：Binder/HAL/JNI/系统服务
        │
        ▼
v3.0 + RTOS/通用 C
├── 增量训练：+20,000 样本
├── 新增：FreeRTOS/Zephyr/通用 C 模式
```

### 2.6 推理框架

| 模型类型 | 框架 | 依赖 |
|----------|------|------|
| 专用小模型 | ONNX Runtime | `ort = "2.0"` |
| DeepSeek LLM | Candle + GGUF | `candle-core = "0.8"` |

### 2.7 用户硬件要求

| 配置 | 仅专用小模型 | + LLM 轻量版 | + LLM 标准版 | + LLM 完整版 |
|------|-------------|-------------|-------------|-------------|
| RAM | 4GB | 8GB | 16GB | 32GB |
| GPU | 不需要 | 可选 | 推荐 | 必需 |

## 3. 精准性保证

### 3.1 三级结果分类

```rust
pub enum Certainty {
    /// 确定 - 静态分析可证明，100% 正确
    Certain { evidence: Evidence },

    /// 可能 - 有依据但不确定，显示所有候选
    Possible {
        candidates: Vec<Candidate>,
        confidence: f32,
    },

    /// 未知 - 无法判断，明确告知用户
    Unknown { reason: String },
}
```

### 3.2 AI 介入规则

```
静态分析结果
    │
    ├─► 唯一确定 ──────────────────► Certain (不调用 AI)
    │
    ├─► 多个候选 ──► AI 排序 ──────► Possible (显示排序后的候选)
    │
    └─► 无法判断 ──► AI 预测 ──┬──► 高置信度 (>0.8) ──► Possible
                              │
                              └──► 低置信度 (<0.8) ──► Unknown
```

### 3.3 绝不猜测原则

- AI 置信度 < 0.8 → 返回 Unknown
- 多个候选置信度接近 → 全部列出，不选择
- 无法分析 → 明确说"无法判断"，提示用户补充

## 4. 自学习机制

### 4.1 学习数据流

```
用户使用 IDE
      │
      ▼
AI 给出预测
      │
      ├─► 用户确认正确 ──► 记录为正样本
      │
      ├─► 用户纠正错误 ──► 记录为负样本 + 正确答案
      │
      └─► 用户补充信息 ──► 记录为新知识
              │
              ▼
      本地学习数据库
      ~/.flowsight/learning/
              │
              ▼
      用户选择上传 ──► GitHub 仓库
```

### 4.2 本地学习数据库

```
~/.flowsight/learning/
├── corrections/           # 用户纠正记录
│   ├── funcptr/          # 函数指针纠正
│   └── async/            # 异步模式纠正
├── confirmations/         # 用户确认记录
├── contributions/         # 待上传的贡献
└── stats.json            # 学习统计
```

### 4.3 学习数据格式

```yaml
# corrections/funcptr/2026-01-09-001.yaml
id: "fp-2026-01-09-001"
type: funcptr_correction
timestamp: "2026-01-09T00:15:00Z"

context:
  file: "drivers/usb/core/hub.c"
  line: 1234
  code: |
    dev->driver->probe(dev);

ai_prediction:
  targets: ["usb_generic_driver_probe"]
  confidence: 0.65
  certainty: Possible

user_correction:
  correct_target: "usb_device_match"
  reason: "这里调用的是 match 回调，不是 probe"

# 用于模型更新
features:
  struct_type: "usb_driver"
  field_name: "probe"
  call_context: "hub_port_connect"
```

### 4.4 本地模型更新

用户纠正后，本地模型可以立即学习：

```rust
impl LocalLearner {
    /// 记录用户纠正并更新本地模型
    pub fn learn_from_correction(&mut self, correction: &Correction) {
        // 1. 保存到本地数据库
        self.save_correction(correction);

        // 2. 更新本地特征权重（轻量级在线学习）
        self.update_feature_weights(correction);

        // 3. 添加到待上传队列
        self.queue_for_upload(correction);
    }
}
```

## 5. 众包学习系统

### 5.1 架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                     FlowSight 众包学习系统                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   用户 A          用户 B          用户 C          ...               │
│   ┌──────┐       ┌──────┐       ┌──────┐                           │
│   │ IDE  │       │ IDE  │       │ IDE  │                           │
│   │ 学习 │       │ 学习 │       │ 学习 │                           │
│   └──┬───┘       └──┬───┘       └──┬───┘                           │
│      │              │              │                                │
│      └──────────────┼──────────────┘                                │
│                     │ 上传学习数据                                   │
│                     ▼                                               │
│           ┌─────────────────────┐                                   │
│           │   GitHub 仓库        │                                   │
│           │   flowsight-ai-data │                                   │
│           │   ├── corrections/  │                                   │
│           │   ├── patterns/     │                                   │
│           │   └── knowledge/    │                                   │
│           └──────────┬──────────┘                                   │
│                      │                                              │
│                      ▼                                              │
│           ┌─────────────────────┐                                   │
│           │   自动化流水线       │                                   │
│           │   (GitHub Actions)  │                                   │
│           │   • 数据验证        │                                   │
│           │   • 去重合并        │                                   │
│           │   • 模型重训练      │                                   │
│           └──────────┬──────────┘                                   │
│                      │                                              │
│                      ▼                                              │
│           ┌─────────────────────┐                                   │
│           │   新版本模型发布     │                                   │
│           │   GitHub Releases   │                                   │
│           └──────────┬──────────┘                                   │
│                      │                                              │
│                      ▼                                              │
│           用户更新模型 (IDE 内一键更新)                              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 上传流程

```rust
impl ContributionUploader {
    /// 上传学习数据到 GitHub
    pub async fn upload(&self) -> Result<()> {
        // 1. 收集待上传数据
        let contributions = self.collect_pending();

        // 2. 匿名化处理（移除敏感路径）
        let anonymized = self.anonymize(&contributions);

        // 3. 创建 Pull Request
        self.create_pr(&anonymized).await?;

        // 4. 标记为已上传
        self.mark_uploaded(&contributions);

        Ok(())
    }
}
```

### 5.3 数据审核

众包数据需要审核才能合并：

1. **自动验证**：格式检查、重复检测
2. **社区审��**：其他用户投票确认
3. **维护者审核**：最终合并决定

### 5.4 隐私保护

- 上传前自动匿名化（移除绝对路径、用户名等）
- 用户可选择不上传
- 只上传代码模式，不上传完整代码

## 6. 实现计划

### Step 1: 基础框架
- [ ] 创建 `flowsight-ai` crate
- [ ] 集成 ONNX Runtime
- [ ] 实现模型加载器

### Step 2: 专用模型
- [ ] 训练函数指针预测器
- [ ] 训练异步模式分类器
- [ ] 集成代码嵌入模型

### Step 3: 自学习
- [ ] 实现本地学习数据库
- [ ] 实现用户纠正 UI
- [ ] 实现在线学习更新

### Step 4: 众包系统
- [ ] 创建 flowsight-ai-data 仓库
- [ ] 实现上传功能
- [ ] 设置 GitHub Actions 流水线

### Step 5: 模型更新
- [ ] 实现模型版本检查
- [ ] 实现一键更新功能
- [ ] 设置自动发布流程

## 7. 文件结构

```
crates/
├── flowsight-ai/              # AI 模块
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── models/            # 模型加载和推理
│       │   ├── mod.rs
│       │   ├── funcptr.rs     # 函数指针预测
│       │   ├── async_pattern.rs # 异步模式分类
│       │   └── embedder.rs    # 代码嵌入
│       ├── learning/          # 自学习
│       │   ├── mod.rs
│       │   ├── database.rs    # 本地数据库
│       │   ├── correction.rs  # 纠正记录
│       │   └── online.rs      # 在线学习
│       ├── crowdsource/       # 众包
│       │   ├── mod.rs
│       │   ├── uploader.rs    # 上传
│       │   └── anonymizer.rs  # 匿名化
│       └── confidence.rs      # 置信度计算
```

## 8. 与现有系统集成

```rust
// 在 flowsight-analysis 中使用 AI
pub struct EnhancedAnalyzer {
    // 现有分析器
    pointer_analyzer: AndersenSolver,
    async_tracker: AsyncTracker,
    funcptr_resolver: FuncPtrResolver,

    // AI 增强
    ai: Option<LocalAI>,
}

impl EnhancedAnalyzer {
    pub fn resolve_funcptr(&self, call_site: &CallSite) -> Certainty {
        // 1. 先用静态分析
        let static_result = self.pointer_analyzer.resolve(call_site);

        // 2. 根据结果决定是否调用 AI
        match static_result {
            StaticResult::Unique(target) => {
                Certainty::Certain { evidence: Evidence::Static }
            }
            StaticResult::Multiple(candidates) => {
                // 用 AI 排序候选
                if let Some(ai) = &self.ai {
                    let ranked = ai.rank_candidates(call_site, &candidates);
                    Certainty::Possible { candidates: ranked, confidence: ranked[0].1 }
                } else {
                    Certainty::Possible { candidates, confidence: 0.5 }
                }
            }
            StaticResult::Unknown => {
                // 用 AI 预测
                if let Some(ai) = &self.ai {
                    let prediction = ai.predict(call_site);
                    if prediction.confidence > 0.8 {
                        Certainty::Possible {
                            candidates: prediction.targets,
                            confidence: prediction.confidence,
                        }
                    } else {
                        Certainty::Unknown { reason: "AI 置信度不足".into() }
                    }
                } else {
                    Certainty::Unknown { reason: "无法静态分析".into() }
                }
            }
        }
    }
}
```

---

*创建日期: 2026-01-09*
