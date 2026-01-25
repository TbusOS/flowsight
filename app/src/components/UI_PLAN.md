# FlowSight UI 重写计划

## 目标
打造一个更好用、更简约的现代化 IDE 界面。

## 新目录结构

```
app/src/components/
├── Layout/              # 布局组件
│   ├── AppLayout.tsx    # 主布局容器
│   ├── SplitPane.tsx    # 可分割面板
│   └── Resizable.tsx    # 可调整大小的面板
│
├── Header/              # 顶部导航栏
│   ├── TopBar.tsx       # 顶部栏主组件
│   ├── SearchBar.tsx    # 搜索栏
│   ├── ProjectSwitcher.tsx # 项目切换器
│   ├── ViewModeToggle.tsx  # 视图模式切换
│   └── Actions.tsx      # 快捷操作按钮
│
├── Sidebar/             # 左侧边栏
│   ├── Sidebar.tsx      # 侧边栏容器
│   ├── FileTree.tsx     # 文件树（重构）
│   ├── Breadcrumb.tsx   # 面包屑导航
│   └── NavHistory.tsx   # 导航历史
│
├── Panels/              # 右侧面板
│   ├── AnalysisPanel.tsx    # 分析概览面板
│   ├── OutlinePanel.tsx     # 大纲面板
│   ├── NodeDetail.tsx       # 节点详情
│   ├── LlvmIrPanel.tsx      # LLVM IR 面板
│   └── PanelContainer.tsx   # 面板容器组件
│
├── Dashboard/           # 仪表盘/欢迎页
│   ├── Welcome.tsx      # 欢迎页
│   ├── RecentProjects.tsx   # 最近项目
│   ├── QuickActions.tsx     # 快捷操作
│   └── ProjectStats.tsx     # 项目统计
│
├── Common/              # 通用组件
│   ├── StatusBar.tsx    # 底部状态栏
│   ├── TabBar.tsx       # 标签栏
│   ├── CommandPalette.tsx   # 命令面板
│   ├── Toast.tsx        # 通知提示
│   ├── Dialogs.tsx      # 对话框集合
│   └── Modal.tsx        # 模态框
│
└── ui/                  # shadcn/ui 基础组件（已有）
    ├── button.tsx
    ├── card.tsx
    ├── tabs.tsx
    ├── dialog.tsx
    ├── input.tsx
    └── ...
```

## 新界面布局

```
┌─────────────────────────────────────────────────────────────┐
│  🥧 FlowSight    [📁 kernel-module]  🔍 搜索函数...   [⚙️]  │  ← TopBar
├────┬────────────────────────┬────────────────────────────┤
│ 🗂️ │                        │ 📊 分析                    │
│    │     📝 代码编辑器       │                            │
│ 文件│                        │ ┌─ 概览                   │
│ 树 │     或                  │ ├─ 大纲                   │
│    │     🔀 执行流图         │ └─ 详情                   │
│    │                        │                            │
├────┤                        │                            │
│ 🔙↔🔜│                        │                            │  ← StatusBar
└────┴────────────────────────┴────────────────────────────┘
```

## 实现计划

### Phase 1: 基础架构
1. **AppLayout.tsx** - 主布局容器
   - 三栏布局 (Sidebar | Main | Panel)
   - 响应式设计
   - 面板可折叠

2. **TopBar.tsx** - 顶部导航栏
   - Logo + 项目名称
   - 集成搜索栏
   - 快捷操作按钮

3. **Resizable 组件** - 可调整大小
   - 拖拽调整面板宽度
   - 保存布局状态

### Phase 2: 左侧边栏
1. **Sidebar.tsx** - 侧边栏容器
   - 180px 宽度
   - 可折叠

2. **FileTree.tsx** - 文件树
   - 虚拟滚动（大项目优化）
   - 搜索过滤
   - 选中状态

3. **Breadcrumb.tsx** - 面包屑
   - 路径导航
   - 点击跳转

### Phase 3: 右侧面板
1. **AnalysisPanel.tsx** - 分析概览
   - 统计卡片
   - 入口点列表

2. **OutlinePanel.tsx** - 大纲面板
   - 函数列表
   - 回调高亮

3. **NodeDetail.tsx** - 节点详情
   - Card 组件展示
   - LLVM IR 折叠

### Phase 4: 顶部交互
1. **SearchBar.tsx** - 搜索栏
   - 全局函数搜索
   - 快捷键 Ctrl+P

2. **ViewModeToggle.tsx** - 视图切换
   - Segmented Control 样式
   - Code / Split / Flow

### Phase 5: 欢迎页
1. **Welcome.tsx** - 欢迎页
   - 最近项目卡片
   - 快捷操作按钮
   - 项目统计展示

### Phase 6: 状态栏和工具
1. **StatusBar.tsx** - 精简状态栏
   - 文件路径
   - 行号/列号
   - 分析状态

2. **CommandPalette.tsx** - 命令面板
   - 键盘快捷操作
   - 文件/符号搜索

## 设计原则

### 1. 视觉一致性
- 所有组件使用 shadcn/ui 风格
- 统一的圆角 (rounded-lg)
- 一致的阴影和动画
- 统一的色彩系统

### 2. 交互优化
- 悬停效果
- 点击反馈
- 过渡动画
- 快捷键支持

### 3. 空间利用
- 更窄的侧边栏 (180px)
- 更紧凑的面板
- 留白更合理
- 滚动条美化

### 4. 可访问性
- 键盘导航
- 焦点指示
- ARIA 标签

## 文件清单

| 文件 | 优先级 | 状态 |
|------|--------|------|
| AppLayout.tsx | P0 | 待创建 |
| TopBar.tsx | P0 | 待创建 |
| Sidebar.tsx | P0 | 待创建 |
| FileTree.tsx | P0 | 待创建 |
| AnalysisPanel.tsx | P0 | 待创建 |
| Welcome.tsx | P0 | 待创建 |
| Resizable.tsx | P1 | 待创建 |
| Breadcrumb.tsx | P1 | 待创建 |
| SearchBar.tsx | P1 | 待创建 |
| ViewModeToggle.tsx | P1 | 待创建 |
| OutlinePanel.tsx | P1 | 待创建 |
| NodeDetail.tsx | P1 | 待创建 |
| StatusBar.tsx | P2 | 待创建 |
| CommandPalette.tsx | P2 | 待创建 |

## 样式规范

### 颜色变量 (使用现有 CSS 变量)
```css
--bg-primary    /* 主背景 */
--bg-secondary  /* 次级背景 */
--bg-tertiary   /* 三级背景 */
--text-primary  /* 主文字 */
--text-secondary/* 次级文字 */
--accent        /* 强调色 */
--border-light  /* 边框 */
```

### 间距规范
- xs: 4px
- sm: 8px
- md: 12px
- lg: 16px
- xl: 24px

### 圆角规范
- sm: 6px
- md: 8px
- lg: 12px
- xl: 16px

## 迁移策略

1. **并行开发** - 在 shadcn-ui 分支开发，不影响 main
2. **逐步替换** - 完成一个组件替换一个
3. **功能验证** - 每个阶段都保证功能完整
4. **样式对比** - 随时可以切换新旧界面

## 验收标准

- [ ] 界面更简洁美观
- [ ] 文件树操作流畅
- [ ] 搜索功能便捷
- [ ] 面板可自由调整
- [ ] 暗色模式完整适配
- [ ] 键盘快捷键完整
- [ ] 无功能缺失
