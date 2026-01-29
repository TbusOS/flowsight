# FlowSight UI 优化计划

> 创建日期: 2026-01-27
> 文档版本: v1.3

## 概述

本文档描述 FlowSight 桌面应用的 UI 优化计划，主要解决当前 UI 的视觉一致性问题，提升用户体验。

## 优化目标

1. **主题系统增强** - 提供多个低饱和度主题选择
2. **间距规范统一** - 修复菜单栏、侧边栏、下拉菜单的间距问题
3. **交互细节优化** - 改善悬停、选中状态的视觉反馈
4. **图标系统统一** - 替换所有 emoji 图标为 Lucide 图标
5. **主题变量统一** - 使用 CSS 变量替代硬编码颜色

---

## 已完成 ✓

### Phase 1: 主题系统增强 ✓

| 任务 | 状态 | 文件 |
|------|------|------|
| 添加主题 CSS 变量 | ✓ 完成 | `app/src/styles/globals.css` |
| 创建 theme-atoms.ts | ✓ 完成 | `app/src/lib/atoms/layout-atoms.ts` |
| 创建 ThemeSelector 组件 | ✓ 完成 | `app/src/components/ThemeSelector/ThemeSelector.tsx` |
| 集成到 App.tsx | ✓ 完成 | `app/src/App.tsx` |

**新增主题**:
- Dark (默认)
- Light (浅色)
- Slate (专业灰蓝)
- Forest (清新绿)
- Sunset (温暖橙)
- Lavender (淡雅紫)

### Phase 2: 间距规范统一 ✓

| 任务 | 状态 | 文件 |
|------|------|------|
| 调整 header.tsx 菜单栏 | ✓ 完成 | `app/src/components/layout/header.tsx` |
| 调整 command.tsx 命令面板 | ✓ 完成 | `app/src/components/ui/command.tsx` |
| 调整 sidebar.tsx 侧边栏 | ✓ 完成 | `app/src/components/layout/sidebar.tsx` |

**间距调整**:
- MenubarTrigger: `px-2 py-1` → `px-2.5 py-1.5`
- MenubarContent padding: `p-1` → `p-1.5`
- MenubarItem: `py-2 rounded-lg` → `py-2.5 gap-2 rounded-md`
- CommandItem: `py-2 rounded-lg px-2` → `py-2.5 gap-2 rounded-md px-3`
- Sidebar: `gap-1 px-1` → `gap-1.5 px-1.5 py-2`
- 按钮圆角: `rounded-sm` → `rounded-md`

### Phase 3: 交互细节优化 ✓

| 任务 | 状态 | 文件 |
|------|------|------|
| 优化 resize handle | ✓ 完成 | `app/src/components/layout/main-layout.tsx` |
| 新增按钮变体 | ✓ 完成 | `app/src/components/ui/button.tsx` |

**新增按钮变体**:
- `success` - 成功状态 (绿色)
- `warning` - 警告状态 (橙色)
- `error` - 错误状态 (红色)

### Phase 4: 图标系统统一 ✓

| 任务 | 状态 | 文件 |
|------|------|------|
| StatusBar 图标替换 | ✓ 完成 | `app/src/components/StatusBar/StatusBar.tsx` |
| StatusBar CSS 变量 | ✓ 完成 | `app/src/components/StatusBar/StatusBar.css` |
| Outline 图标替换 | ✓ 完成 | `app/src/components/Outline/Outline.tsx` |
| Outline CSS 变量 | ✓ 完成 | `app/src/components/Outline/Outline.css` |
| FileTree 图标替换 | ✓ 完成 | `app/src/components/Explorer/FileTree.tsx` |
| Explorer CSS 变量 | ✓ 完成 | `app/src/components/Explorer/Explorer.css` |
| 全局样式更新 | ✓ 完成 | `app/src/styles/globals.css` |

**图标替换详情**:

**StatusBar**:
- `⏳` → `<Loader2 className="animate-spin" />`
- `✅` → `<CheckCircle2 />`
- `❌` → `<XCircle />`
- `💤` → `<Circle />`
- `●` → `<Dot fill="currentColor" />`
- `ƒ` → `<FileCode />`

**Outline**:
- `📦` → `<Box />` (Functions)
- `🏗️` → `<Building2 />` (Structures)
- `📌` → `<Pin />` (Variables)
- `🔧` → `<Wrench />` (Macros)
- `📋` → `<ListOrdered />` (Enums)
- `📝` → `<FileType />` (Typedefs)
- `⚡` → `<Zap />` (Callback)
- `📋` (empty) → `<FileType />`
- `🔍` (search) → `<Search />`
- `✕` (clear) → `<X />`
- `▶` (arrow) → `<ChevronRight />`
- `→` (action) → `<ArrowRight />`

**FileTree**:
- `📂` → `<FolderOpen />` (expanded)
- `📁` → `<Folder />` (collapsed)
- `🔷` → `<FileCode className="text-blue-400" />` (.c)
- `📘` → `<FileCode className="text-blue-300" />` (.h)
- `🔶` → `<FileCode className="text-orange-400" />` (.cpp/.cc/.cxx)
- `📙` → `<FileCode className="text-orange-300" />` (.hpp/.hxx)
- `🦀` → `<FileCode className="text-orange-500" />` (.rs)
- `🐍` → `<FileCode className="text-yellow-400" />` (.py)
- `💛` → `<FileCode className="text-yellow-300" />` (.js/.ts/.tsx)
- `📋` → `<FileJson />` (.json)
- `📝` → `<FileText />` (.md)
- `⚙️` → `<Settings />` (.yaml/.yml)
- `📄` → `<File />` (default)
- `◌` (loading) → `<Loader2 className="animate-spin" />`
- `▶` (chevron) → `<ChevronRight />`
- Context menu icons

**新增 CSS 变量**:
- `--accent-pink: #ec4899` (Enum 类型颜色)

### Phase 5: 沉浸式可视化优化 ✓

| 任务 | 状态 | 文件 |
|------|------|------|
| FlowNode 图标替换 | ✓ 完成 | `app/src/components/FlowView/FlowNode.tsx` |
| FlowNode 深度效果 | ✓ 完成 | `app/src/components/FlowView/FlowNode.css` |
| FlowView 边动画增强 | ✓ 完成 | `app/src/components/FlowView/FlowView.css` |
| ASYNC_COLORS 变量化 | ✓ 完成 | `app/src/components/FlowView/FlowView.tsx` |
| 全局异步颜色变量 | ✓ 完成 | `app/src/styles/globals.css` |

**FlowNode 改进**:
- 置信度图标: `✓` → `<Check />`, `?` → `<HelpCircle />`, `!` → `<AlertTriangle />`
- 异步机制图标: 🔄 → `<RefreshCw />`, ⏱️ → `<Clock />`, ⚡ → `<Zap />` 等
- 节点类型图标: 👤 → `<User />`, 🔧 → `<Settings />`, 📦 → `<ArrowRight />` 等
- 展开/收起: SVG inline → `<ChevronDown />`, `<ChevronRight />`

**FlowNode CSS 增强**:
- 节点悬停阴影: `0 1px 2px` → `0 4px 8px` + glow 效果
- 悬停位移: `transform: translateY(-1px)`
- 选中状态: 添加 `0 0 16px rgba(59, 130, 246, 0.2)` 发光效果
- 置信度徽章: 14px → 18px, 添加缩放动画
- 异步标签: 添加图标和更好的渐变阴影

**FlowView CSS 增强**:
- 异步边动画: `stroke-dasharray: 6 4` → `8 4`, 动画时间 `1.2s` → `1.5s`
- 异步边颜色: 根据类型显示不同颜色 (WorkQueue/黄色, Timer/绿色, IRQ/红色 等)
- 图例优化: 添加悬停效果和 data-async 属性选择器

**新增异步机制 CSS 变量**:
- `--async-workqueue: #f59e0b`
- `--async-timer: #22c55e`
- `--async-irq: #ef4444`
- `--async-tasklet: #8b5cf6`
- `--async-kthread: #3b82f6`
- `--async-softirq: #06b6d4`
- `--async-completion: #14b8a6`
- `--async-rcu: #f97316`

---

## 待完成

无

---

## 文件变更清单

### 新增文件

| 文件 | 描述 |
|------|------|
| `docs/ui-optimization-plan.md` | 本计划文档 |
| `app/src/components/ThemeSelector/ThemeSelector.tsx` | 主题选择组件 |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| `app/src/styles/globals.css` | 新增 5 个主题变量 (slate, forest, sunset, lavender, light 增强)、新增 --accent-pink |
| `app/src/lib/atoms/layout-atoms.ts` | 新增 Theme Atoms (themeAtom, themeInfoAtom, themesAtom, setThemeWithPersistenceAtom, initThemeAtom) |
| `app/src/components/layout/header.tsx` | 菜单栏间距调整、添加 ThemeSelector |
| `app/src/components/layout/sidebar.tsx` | 侧边栏间距调整 |
| `app/src/components/ui/command.tsx` | 命令面板间距调整 |
| `app/src/components/layout/main-layout.tsx` | resize handle 优化 |
| `app/src/components/ui/button.tsx` | 新增功能性按钮变体 (success/warning/error) |
| `app/src/App.tsx` | 集成 ThemeInitializer |
| `app/src/components/StatusBar/StatusBar.tsx` | Emoji → Lucide 图标 |
| `app/src/components/StatusBar/StatusBar.css` | 硬编码颜色 → CSS 变量 |
| `app/src/components/Outline/Outline.tsx` | Emoji → Lucide 图标 |
| `app/src/components/Outline/Outline.css` | 硬编码颜色 → CSS 变量 |
| `app/src/components/Explorer/FileTree.tsx` | Emoji → Lucide 图标 |
| `app/src/components/Explorer/Explorer.css` | 硬编码颜色 → CSS 变量 |
| `app/src/components/FlowView/FlowNode.tsx` | Emoji → Lucide 图标，置信度/异步机制/节点类型 |
| `app/src/components/FlowView/FlowNode.css` | 深度效果、悬停动画、选中发光 |
| `app/src/components/FlowView/FlowView.tsx` | ASYNC_COLORS 变量化，添加 data-async-type |
| `app/src/components/FlowView/FlowView.css` | 异步边动画增强，特定类型颜色 |

---

## 验收标准

- [x] 6 个低饱和度主题可切换
- [x] 菜单栏字体、间距统一
- [x] 下拉菜单项有足够行距 (py-2.5)
- [x] 侧边栏图标间距适中
- [x] 底部面板 resize handle 悬停效果明显
- [x] 功能性按钮（success/warning/error）可用
- [x] 状态栏图标统一为 Lucide
- [x] 大纲面板图标统一为 Lucide
- [x] 文件树图标统一为 Lucide
- [x] 所有硬编码颜色替换为 CSS 主题变量
- [x] 整体视觉层次清晰
- [x] 构建验证通过
- [x] **FlowNode** 置信度/异步机制/节点类型图标替换为 Lucide
- [x] **FlowNode** 悬停时有深度提升效果 (+ shadow + glow)
- [x] **FlowNode** 选中状态有明显发光效果
- [x] **FlowView** 异步边根据类型显示不同颜色
- [x] **FlowView** 异步边流动动画平滑流畅

---

## 注意事项

1. **向后兼容**: 保持默认主题与现有深色主题一致
2. **性能**: 主题切换即时生效，localStorage 持久化
3. **可访问性**: 对比度符合现代设计标准
4. **一致性**: 所有下拉菜单、弹出层的间距保持统一
5. **图标一致性**: 使用 Lucide React 图标库替代 emoji
