# FlowSight UI 优化计划

> 记录 FlowSight 桌面应用的 UI 优化历程

## 概述

本计划记录 FlowSight 代码执行流可视化 IDE 的 UI/UX 优化工作，从初始状态逐步改进为现代化、沉浸式的设计。

---

## Phase 1: 主题系统完善 ✓

**完成日期**: 2025-01-27

### 完成内容

- [x] 6 个低饱和度主题系统 (cyan, green, orange, pink, purple, yellow)
- [x] CSS 变量完整覆盖所有颜色
- [x] 主题切换平滑过渡动画
- [x] 系统自动检测深色/浅色偏好

### 文件修改

- `src/index.css` - 6 主题系统 CSS 变量
- `src/components/ThemeToggle/ThemeToggle.tsx` - 主题切换组件

---

## Phase 2: 间距规范统一 ✓

**完成日期**: 2025-01-27

### 完成内容

- [x] 菜单栏间距规范化 (16px padding, 8px gap)
- [x] 命令面板间距优化
- [x] 侧边栏项目间距调整
- [x] 底部状态栏统一间距

### 文件修改

- `src/components/MenuBar/MenuBar.css`
- `src/components/CommandPalette/CommandPalette.css`
- `src/components/Sidebar/Sidebar.css`
- `src/components/StatusBar/StatusBar.css`

---

## Phase 3: 交互细节优化 ✓

**完成日期**: 2025-01-27

### 完成内容

- [x] Resize handle 视觉优化
- [x] 按钮 hover/active 状态过渡
- [x] Focus visible 聚焦指示器
- [x] Success/Warning/Error 按钮变体

### 文件修改

- `src/components/Button/Button.css`
- `src/components/SplitPane/SplitPane.css`
- `src/components/Sidebar/Sidebar.css`

---

## Phase 4: 图标系统统一 ✓

**完成日期**: 2025-01-27

### 完成内容

- [x] Emoji 图标替换为 Lucide React 图标
- [x] 图标尺寸统一规范 (16px/20px/24px)
- [x] 图标颜色变量化 (`var(--text-secondary)`)
- [x] SVG stroke-width 统一 (1/1.5/2/2.5)

### 组件更新

- `FlowNode.tsx` - 节点图标 (confidence, async, type)
- `CommandPalette.tsx` - 命令图标
- `Toast.tsx` - 通知图标
- `EmptyState.tsx` - 空状态图标
- `LoadingSpinner.tsx` - 加载图标
- `ContextMenu.tsx` - 菜单图标

---

## Phase 5: 沉浸式可视化优化 ✓

**完成日期**: 2025-01-27

### 完成内容

- [x] 执行流图节点深度效果 (post-neumorphism)
- [x] 节点 hover 动画 (scale + shadow)
- [x] 节点选中发光效果 (glow)
- [x] 异步边动画 (脉动渐变流动)
- [x] 异步机制颜色标识 (8 种颜色)

### 文件修改

- `src/components/Flow/FlowNode.tsx` - 节点图标 Lucide 化
- `src/components/Flow/FlowNode.css` - 深度效果和动画
- `src/components/Flow/FlowView.css` - 异步边动画
- `src/components/Flow/FlowView.tsx` - CSS 变量化
- `src/index.css` - 8 个异步机制颜色变量

### 新增 CSS 变量

```css
--async-workqueue: #f59e0b;
--async-timer: #22c55e;
--async-irq: #ef4444;
--async-tasklet: #8b5cf6;
--async-kthread: #3b82f6;
--async-softirq: #06b6d4;
--async-completion: #14b8a6;
--async-rcu: #f97316;
```

---

## Phase 6: 交互设计完善 ✓

**完成日期**: 2025-01-27

### 完成内容

- [x] 命令面板图标系统完善
- [x] Toast 通知组件优化
- [x] 空状态组件图标化
- [x] 加载动画组件重构
- [x] 上下文菜单样式统一
- [x] 所有组件 CSS 变量化

### 文件修改

#### 组件文件

- `src/components/CommandPalette/CommandPalette.tsx` - Lucide 图标
- `src/components/Toast/Toast.tsx` - Lucide 图标
- `src/components/EmptyState/EmptyState.tsx` - Lucide 图标
- `src/components/LoadingSpinner/LoadingSpinner.tsx` - Lucide + Cancel
- `src/components/ContextMenu/ContextMenu.tsx` - Lucide 图标

#### 样式文件

- `src/components/Toast/Toast.css` - CSS 变量化
- `src/components/EmptyState/EmptyState.css` - CSS 变量化
- `src/components/ContextMenu/ContextMenu.css` - CSS 变量化
- `src/components/LoadingSpinner/LoadingSpinner.css` - 简化动画

### 技术亮点

1. **图标类型统一**: `string` → `React.ReactNode`
2. **主题无缝切换**: 所有硬编码颜色替换为 CSS 变量
3. **加载动画优化**: 简化多环动画，使用 Lucide 图标 + CSS 动画
4. **取消功能**: LoadingSpinner 新增 `onCancel` prop

---

## Phase 7: 待实现 (规划中)

### 键盘快捷键支持

- [ ] 全局键盘事件监听器
- [ ] 快捷键提示 UI
- [ ] 快捷键冲突检测
- [ ] 自定义快捷键设置

### 拖拽交互优化

- [ ] 节点拖拽预览效果
- [ ] 放置区域高亮
- [ ] 拖拽吸附对齐
- [ ] 批量选择拖拽

### 错误状态增强

- [ ] 错误边界组件
- [ ] 错误恢复建议
- [ ] 网络错误重试 UI
- [ ] 错误历史记录

### 空状态模板

- [ ] 搜索无结果
- [ ] 筛选无匹配
- [ ] 首次使用引导
- [ ] 数据导入完成

---

## 性能指标

| 指标 | 优化前 | 优化后 |
|------|--------|--------|
| 构建时间 | - | ~1.2s |
| CSS 文件大小 | - | 59.6 kB (gzip: 11.3 kB) |
| JS 包大小 | - | 440.8 kB (gzip: 139.1 kB) |
| 动画帧率 | - | 60fps (CSS transform) |

---

## 设计原则

### 视觉层次

1. **背景层**: `var(--bg-primary)` - 最底层
2. **表面层**: `var(--bg-secondary)` - 卡片、面板
3. **交互层**: `var(--bg-hover)` - hover 状态
4. **强调层**: `var(--accent)` - 主要操作

### 动画曲线

- **快速交互**: `cubic-bezier(0.4, 0, 0.2, 1)` - 150ms
- **平滑过渡**: `cubic-bezier(0.4, 0, 0.2, 1)` - 300ms
- **强调效果**: `cubic-bezier(0.34, 1.56, 0.64, 1)` - 500ms

### 间距系统

| 用途 | 间距值 |
|------|--------|
| 紧凑 | 4px / 8px |
| 标准 | 12px / 16px |
| 宽松 | 24px / 32px |
| 间距递增 | 1.25x 比例 |

---

## 后续计划

1. **Phase 7**: 键盘快捷键 + 拖拽交互
2. **Phase 8**: 错误处理 + 空状态模板
3. **Phase 9**: 响应式适配 (移动端)
4. **Phase 10**: 动效一致性审查

---

## 更新日志

### v1.0 (2025-01-27)

- 初始版本
- 完成 Phase 1-6 全部优化
- 构建验证通过

### v1.1 (规划中)

- Phase 7 功能实现
