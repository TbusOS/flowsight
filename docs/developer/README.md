# 开发者文档

本目录包含 FlowSight 的开发者相关文档。

## 📁 子目录

```
developer/
├── api/              # API 参考文档
└── contributing/     # 贡献指南
```

## 📄 文档列表

| 文档 | 描述 |
|------|------|
| [I18N.md](I18N.md) | 国际化开发指南与翻译贡献流程 |

## 📋 计划文档

### API 参考 (`api/`)

| 文档 | 描述 | 状态 |
|------|------|------|
| CORE-API.md | 核心分析引擎 API | 待编写 |
| KNOWLEDGE-API.md | 知识库扩展 API | 待编写 |
| PLUGIN-API.md | 插件开发 API | 待编写 |
| IPC-PROTOCOL.md | 前后端通信协议 | 待编写 |

### 贡献指南 (`contributing/`)

| 文档 | 描述 | 状态 |
|------|------|------|
| CONTRIBUTING.md | 贡献流程与规范 | 待编写 |
| CODE-STYLE.md | 代码风格指南 | 待编写 |
| BUILD.md | 构建与测试指南 | 待编写 |
| RELEASE.md | 发布流程 | 待编写 |

## 🛠️ 开发环境准备

### 前置要求

- **Rust** 1.70+ (rustup 安装)
- **Node.js** 18+ (用于前端构建)
- **pnpm** 8+ (包管理器)
- **LLVM/Clang** 15+ (libclang 依赖)

### 快速开始

```bash
# 克隆仓库
git clone https://github.com/TbusOS/flowsight.git
cd flowsight

# 安装依赖
pnpm install

# 开发模式运行
pnpm tauri dev

# 构建发布版
pnpm tauri build
```

## 🏗️ 项目结构

```
flowsight/
├── src-tauri/           # Rust 后端
│   ├── src/
│   │   ├── main.rs      # 入口
│   │   ├── parser/      # 解析器模块
│   │   ├── analyzer/    # 分析器模块
│   │   ├── knowledge/   # 知识库模块
│   │   └── storage/     # 存储模块
│   └── Cargo.toml
├── src/                 # React 前端
│   ├── components/      # UI 组件
│   ├── views/           # 页面视图
│   ├── stores/          # 状态管理
│   └── App.tsx
├── knowledge/           # 知识库 YAML 文件
└── docs/                # 文档
```

## 🔌 扩展开发

### 添加新语言支持

1. 在 `knowledge/languages/` 添加语言知识
2. 实现 `LanguageAdapter` trait
3. 注册到解析器工厂

### 添加新框架知识

1. 在 `knowledge/platforms/` 添加 YAML 文件
2. 遵循 Schema 规范
3. 添加单元测试验证

详见 [知识库 Schema 设计](../architecture/KNOWLEDGE-BASE-SCHEMA.md)

