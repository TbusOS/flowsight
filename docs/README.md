# FlowSight 文档中心

## 📁 目录结构

```
docs/
├── README.md                        # 本文件 - 文档导航
├── design/                          # 项目设计与规划
│   ├── PROJECT-PLAN-V3.md           # ⭐ 当前项目计划 (v3.0)
│   ├── VISUALIZATION-STRATEGY.md    # 可视化策略
│   ├── TARGET-ARCHITECTURE.md       # 目标架构选择
│   └── ...
├── architecture/                    # 技术架构文档
│   ├── KNOWLEDGE-BASE-SCHEMA.md     # 知识库 Schema 设计
│   ├── POINTER-ANALYSIS.md          # 指针分析算法
│   ├── LOCAL-AI-DESIGN.md           # 本地 AI 设计
│   └── ...
├── training/                        # AI 训练文档
│   ├── local-mlx/                   # ⭐ 本地 MLX 训练 (推荐)
│   │   ├── README.md
│   │   ├── TUTORIAL.md              # 手把手教程
│   │   ├── MLX-LORA-TRAINING-PLAN.md
│   │   └── KERNEL-COVERAGE-PLAN.md  # 内核覆盖计划
│   └── cloud-training/              # 云端 GPU 训练 (可选)
│       ├── AI-TRAINING-GUIDE.md
│       └── TRAINING-DATA-PLAN.md
├── plans/                           # 开发计划
│   ├── ui-optimization-plan.md      # UI 优化计划
│   └── QUALITY-IMPROVEMENT-PLAN.md
├── testing/                         # 测试文档
│   ├── DESKTOP-E2E-TESTING.md
│   └── TEST-PLAN.md
├── user-guide/                      # 用户指南
├── developer/                       # 开发者文档
└── archive/                         # 📦 归档 (旧版本)
    ├── PROJECT-PLAN-V1.md
    ├── PROJECT-PLAN-V2.md
    └── UI_PLAN-V1.md
```

## 🚀 快速导航

### 入门必读

| 文档 | 说明 |
|------|------|
| [项目计划 v3](design/PROJECT-PLAN-V3.md) | 了解 FlowSight 的愿景、架构和路线图 |
| [本地 AI 训练](training/local-mlx/README.md) | 在 MacBook M3 上训练模型 |

### 技术架构

| 文档 | 说明 |
|------|------|
| [知识库 Schema](architecture/KNOWLEDGE-BASE-SCHEMA.md) | 如何设计和扩展知识库 |
| [指针分析算法](architecture/POINTER-ANALYSIS.md) | 函数指针解析的核心算法 |
| [本地 AI 设计](architecture/LOCAL-AI-DESIGN.md) | 本地 AI 推理架构 |
| [多语言抽象层](architecture/MULTI-LANGUAGE-ABSTRACTION.md) | 统一 IR 和跨语言分析 |

### 设计文档

| 文档 | 说明 |
|------|------|
| [可视化策略](design/VISUALIZATION-STRATEGY.md) | 分层渲染解决大规模图性能问题 |
| [目标架构选择](design/TARGET-ARCHITECTURE.md) | 条件编译处理，确保执行流唯一 |
| [执行流设计](design/EXECUTION-FLOW-DESIGN.md) | ExecutionFlow API 设计 |

### AI 训练

| 方案 | 硬件要求 | 推荐度 |
|------|----------|--------|
| [本地 MLX 训练](training/local-mlx/README.md) | MacBook M3 24GB | ⭐⭐⭐ 推荐 |
| [云端 GPU 训练](training/cloud-training/AI-TRAINING-GUIDE.md) | A100 40GB | 可选 |

### 测试文档

| 文档 | 说明 |
|------|------|
| [桌面 E2E 测试](testing/DESKTOP-E2E-TESTING.md) | Playwright 桌面测试指南 |
| [测试计划](testing/TEST-PLAN.md) | 整体测试策略 |

### 开发者

| 文档 | 说明 |
|------|------|
| [国际化指南](developer/I18N.md) | i18n 开发与翻译贡献 |
| [API 参考](developer/api/README.md) | Tauri 命令 API |

## 📋 文档规范

### 文档分类

| 目录 | 用途 | 目标读者 |
|------|------|----------|
| `design/` | 项目规划、愿景、路线图 | 项目管理者、贡献者 |
| `architecture/` | 技术设计、算法原理 | 核心开发者 |
| `training/` | AI 模型训练指南 | AI 开发者 |
| `plans/` | 当前开发计划 | 开发团队 |
| `testing/` | 测试策略和指南 | QA 工程师 |
| `user-guide/` | 使用教程、功能说明 | 最终用户 |
| `developer/` | 开发指南、API 文档 | 插件/扩展开发者 |
| `archive/` | 归档的旧版本文档 | 历史参考 |

### 命名约定

- 使用大写字母和连字符命名：`PROJECT-PLAN.md`
- 子目录使用小写：`user-guide/`, `developer/`
- 版本号后缀：`-V2.md`, `-V3.md`
- 图片放入 `images/` 子目录

## 🔗 相关资源

- [知识库结构](../knowledge/README.md) - 知识库文件组织
- [GitHub 仓库](https://github.com/TbusOS/flowsight) - 源代码
- [CLAUDE.md](../CLAUDE.md) - Agent 配置
