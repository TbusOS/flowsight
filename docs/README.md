# FlowSight 文档中心

## 📁 目录结构

```
docs/
├── README.md                    # 本文件 - 文档导航
├── design/                      # 项目设计与规划
│   └── PROJECT-PLAN.md          # 完整项目计划
├── architecture/                # 技术架构文档
│   ├── KNOWLEDGE-BASE-SCHEMA.md # 知识库 Schema 设计
│   ├── POINTER-ANALYSIS.md      # 指针分析算法
│   └── MULTI-LANGUAGE-ABSTRACTION.md # 多语言抽象层
├── user-guide/                  # 用户使用指南
│   └── (待添加)
└── developer/                   # 开发者文档
    ├── api/                     # API 参考
    └── contributing/            # 贡献指南
```

## 🚀 快速导航

### 入门
- [项目计划](design/PROJECT-PLAN.md) - 了解 FlowSight 的愿景、架构和路线图

### 技术架构
- [知识库 Schema 设计](architecture/KNOWLEDGE-BASE-SCHEMA.md) - 如何设计和扩展知识库
- [指针分析算法](architecture/POINTER-ANALYSIS.md) - 函数指针解析的核心算法
- [多语言抽象层](architecture/MULTI-LANGUAGE-ABSTRACTION.md) - 统一 IR 和跨语言分析

### 用户指南
- (开发中) 安装指南
- (开发中) 快速开始
- (开发中) 功能介绍

### 开发者
- [国际化指南](developer/I18N.md) - i18n 开发与翻译贡献
- (开发中) API 参考
- (开发中) 贡献指南
- (开发中) 构建与测试

## 📋 文档规范

### 文档分类

| 目录 | 用途 | 目标读者 |
|------|------|----------|
| `design/` | 项目规划、愿景、路线图 | 项目管理者、贡献者 |
| `architecture/` | 技术设计、算法原理 | 核心开发者 |
| `user-guide/` | 使用教程、功能说明 | 最终用户 |
| `developer/` | 开发指南、API 文档 | 插件/扩展开发者 |

### 命名约定
- 使用大写字母和连字符命名：`PROJECT-PLAN.md`
- 子目录使用小写：`user-guide/`, `developer/`
- 图片放入对应目录的 `images/` 子目录

## 🔗 相关资源

- [知识库结构](../knowledge/README.md) - 知识库文件组织
- [GitHub 仓库](https://github.com/TbusOS/flowsight) - 源代码

