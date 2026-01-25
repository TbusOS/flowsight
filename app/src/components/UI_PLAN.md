# FlowSight UI 重写计划

## 核心设计理念

**"简约极美，超越 Cursor"**

- 参考 21st.dev 的极简美学
- macOS 设计语言
- 内容优先，面板悬浮/可折叠
- 键盘优先操作

---

## 设计效果预览

```
┌─────────────────────────────────────────────────────────────────┐
│  🔭 FlowSight                                      [⌘K]  ⚙️    │
├─────────┬───────────────────────────────────────┬───────────────┤
│         │                                       │               │
│    ○    │         代码 / 执行流区域              │   📋 大纲    │
│    📁   │                                       │               │
│    📜   │    42  int main() {                   │   函数列表    │
│    🔍   │    43      demo_init();     ← 光标    │   快速跳转    │
│    ⚡   │    44      return 0;                  │               │
│    ⌘K   │    45  }                              │               │
│         │                                       │               │
├─────────┴───────────────────────────────────────┴───────────────┤
│  [终端]                                       Ln 45, Col 3     │
└─────────────────────────────────────────────────────────────────┘

  ╔═══════════════════════════════════════════════════════════════╗
  ║  按 ⌘K 弹出命令面板 (悬浮，21st.dev 风格)                      ║
  ╠═══════════════════════════════════════════════════════════════╣
  ║  🔍 搜索文件/函数...                                           ║
  ║  ────────────────────────────                                  ║
  ║  > main.c                                                     ║
  ║  > main()                                                     ║
  ║  > init_module()                                              ║
  ╚═══════════════════════════════════════════════════════════════╝
```

---

## 极简设计原则

### 1. 极致简约

- **去除所有不必要的边框和分隔线**
- **面板默认折叠，按需展开**
- **留白充足，视觉呼吸感**
- **单色系为主，亮点色点缀**

### 2. 悬浮面板 (21st.dev 风格)

```
┌─────────────────────────────────────────────────┐
│                                                 │
│         代码编辑区域 (纯内容，无边框)            │
│                                                 │
├─────────────────────────────────────────────────┤
│  当需要大纲时，右边缘滑出悬浮面板 ────────────→ │
│  ╭──────────────────────────────────────╮       │
│  │ 📋 大纲                          ○ │◀─ 关闭  │
│  ├──────────────────────────────────────┤        │
│  │ 📦 main()                          │        │
│  │ ⚡ irq_handler()                   │        │
│  │ 📦 cleanup()                       │        │
│  ╰──────────────────────────────────────╯       │
└─────────────────────────────────────────────────┘
```

### 3. 键盘优先

| 快捷键 | 功能 |
|--------|------|
| `⌘K` | 弹出命令面板 (全局) |
| `⌘B` | 切换侧边栏 |
| `⌘J` | 切换底部面板 |
| `⌘P` | 快速打开文件 |
| `⌘⇧P` | 高级命令 |

---

## 完整布局设计

```
┌─────────────────────────────────────────────────────────────────┐
│  🔭 FlowSight                           [⌘K]  [⚙️]  [⊞]       │
├─────────┬───────────────────────────────────────┬───────────────┤
│         │                                       │               │
│    ○    │                                       │   (悬浮面板)  │
│  ┌───┐  │         代码 / 执行流区域              │   ╭───────╮   │
│  │ ○ │  │                                       │   │ 📋    │   │
│  │ 📁│  │    42  int main() {                   │   ├───────╤   │
│  │ 📜│  │    43      demo_init();               │   │ 📦 main│   │
│  │ 📋│  │    44      return 0;                  │   │ ⚡ irq  │   │
│  │ ⚡│  │    45  }                              │   │ 📦 exit│   │
│  │ 🔍│  │                                       │   ╰───────╯   │
│  │ ⌘K│  │                                       │               │
│  └───┘  │                                       │               │
│         │                                       │               │
├─────────┴───────────────────────────────────────┴───────────────┤
│  [Terminal]  [Output]  [Problems]               Ln 45, Col 3   │
│        ◀── 可折叠底部面板 (悬浮边缘)                              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 与 Cursor 对比

| 维度 | Cursor | FlowSight (新设计) | 优势 |
|------|--------|-------------------|------|
| **整体风格** | 深灰沉闷 | 极简通透 | 视觉更舒适 |
| **边框** | 粗重边框 | 无边框/细线 | 更现代 |
| **面板** | 固定占据 | 悬浮/折叠 | 内容更大 |
| **动画** | 生硬 | Motion 弹簧 | 更流畅 |
| **颜色** | 单调深灰 | 柔和灰+蓝点缀 | 更有层次 |
| **留白** | 紧凑 | 充足留白 | 不压抑 |
| **命令面板** | 复杂 | ⌘K 悬浮 | 更高效 |

### 视觉对比示意

```
Cursor (当前)                    FlowSight (目标)
┌──────────────────────┐        ┌──────────────────────┐
│ ● ● ●                │        │ 🔭                   │
├──────────────────────┤        ├──────────────────────┤
│                      │        │                      │
│  深灰背景 ████████   │        │  柔和背景  ░░░░░░░░  │
│  粗边框   ████████   │        │  无边框   ═════════  │
│  强阴影   ████████   │        │  轻阴影   ═════════  │
│                      │        │                      │
└──────────────────────┘        └──────────────────────┘
```

---

## 颜色系统

```css
:root {
  /* 背景 - 柔和深色 */
  --background: 240 10% 5%;      /* #0d0d0d - 极深灰 */
  --surface: 240 10% 8%;         /* #141414 - 悬浮层 */
  --surface-hover: 240 10% 12%;  /* #1f1f1f - 悬浮态 */

  /* 前景 */
  --foreground: 0 0% 95%;        /* 主文字 */
  --foreground-muted: 0 0% 60%;  /* 次要文字 */

  /* 品牌色 - 蓝色系 */
  --primary: 228 100% 50%;       /* #0034FF - 品牌蓝 */
  --primary-hover: 228 100% 55%;
  --primary-light: 228 100% 50% / 0.15;

  /* 语义色 */
  --success: 142 76% 36%;
  --warning: 38 92% 50%;
  --error: 0 84% 60%;

  /* 边框 - 极细 */
  --border: 240 4% 10%;
  --border-light: 240 4% 8%;

  /* 圆角 - macOS 风格 */
  --radius-sm: 6px;
  --radius-md: 10px;
  --radius-lg: 14px;

  /* 阴影 - 轻盈 */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 16px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.5);
  --shadow-float: 0 12px 48px rgba(0, 0, 0, 0.6);
}
```

---

## 技术架构

### 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 构建工具 | Vite 6 | 极速 HMR |
| 框架 | React 19 | 最新特性 |
| 桌面端 | Tauri 2.0 | Rust 后端 |
| 状态管理 | Jotai | 原子化 |
| 样式 | Tailwind CSS | 原子化 |
| 组件 | shadcn/ui + Radix UI | 无头组件 |
| 动画 | Motion | 声明式 |
| 图形 | @xyflow/react | 流程图 |

### 项目结构

```
app/src/
├── components/
│   ├── ui/                    # shadcn/ui 基础组件
│   │   ├── button.tsx         # 渐变边框按钮
│   │   ├── input.tsx
│   │   ├── dialog.tsx         # 悬浮对话框
│   │   ├── popover.tsx        # 悬浮弹出
│   │   ├── dropdown-menu.tsx
│   │   ├── command.tsx        # ⌘K 命令面板
│   │   ├── tooltip.tsx
│   │   ├── resizable.tsx      # 可调整面板
│   │   └── tabs.tsx
│   │
│   ├── layout/
│   │   ├── main-layout.tsx    # 主布局
│   │   ├── sidebar.tsx        # 左侧图标栏 (macOS 风格)
│   │   ├── header.tsx         # 极简顶部栏
│   │   └── status-bar.tsx     # 底部状态栏
│   │
│   ├── panels/                # 功能面板 (悬浮式)
│   │   ├── outline-panel.tsx      # 大纲面板
│   │   ├── node-detail-panel.tsx  # 节点详情
│   │   ├── llvm-ir-panel.tsx      # LLVM IR
│   │   └── explorer-panel.tsx     # 文件浏览器
│   │
│   ├── flow/
│   │   ├── flow-view.tsx      # 执行流视图
│   │   ├── flow-node.tsx      # 节点组件
│   │   └── flow-canvas.tsx    # 画布组件
│   │
│   └── editor/
│       └── code-editor.tsx    # Monaco 编辑器
│
├── lib/
│   ├── atoms/                 # Jotai 状态
│   │   ├── layout-atoms.ts    # 布局状态
│   │   ├── panel-atoms.ts     # 面板状态
│   │   └── project-atoms.ts   # 项目状态
│   ├── stores/                # Zustand
│   ├── hooks/
│   │   ├── use-panel.ts       # 面板控制
│   │   └── use-command.ts     # ⌘K 命令
│   └── utils/
│
├── styles/
│   └── globals.css            # 全局样式
│
└── App.tsx
```

---

## 核心组件设计

### 1. 悬浮命令面板 (Command Palette)

基于 1code 的 `Command` 组件，21st.dev 风格：

```tsx
function CommandPalette() {
  return (
    <CommandDialog open={open} onOpenChange={setOpen}>
      <CommandInput placeholder="搜索文件、函数、命令..." />
      <CommandList>
        <CommandEmpty>无结果</CommandEmpty>
        <CommandGroup heading="文件">
          <CommandItem>main.c</CommandItem>
          <CommandItem>utils.c</CommandItem>
        </CommandGroup>
        <CommandGroup heading="函数">
          <CommandItem>main()</CommandItem>
          <CommandItem>init()</CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  )
}
```

**设计特点：**
- 居中悬浮，不遮挡主内容
- 模糊背景遮罩
- 键盘导航支持
- 分组 + 搜索高亮

### 2. 悬浮侧边栏面板

```tsx
function FloatingPanel({
  side,      // 'right' | 'left' | 'bottom'
  isOpen,
  onClose,
  children
}) {
  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* 模糊遮罩 */}
          <motion.div
            className="fixed inset-0 bg-black/20 backdrop-blur-sm z-40"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
          />

          {/* 悬浮面板 */}
          <motion.div
            className={`fixed z-50 bg-surface rounded-lg shadow-float ${
              side === 'right' ? 'right-4 top-20 bottom-20 w-72' :
              side === 'left' ? 'left-20 top-20 bottom-20 w-64' :
              'bottom-20 left-20 right-20 h-64'
            }`}
            initial={{ opacity: 0, x: side === 'right' ? 20 : 0 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: side === 'right' ? 20 : 0 }}
          >
            {children}
            <Button className="absolute top-2 right-2" variant="ghost" size="icon">
              <XIcon />
            </Button>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  )
}
```

### 3. 左侧图标导航栏 (macOS 风格)

```
┌─────────┐
│    ○    │  ◀ 当前激活
│  ─────  │
│    📁   │  项目文件
│    📜   │  大纲
│    ⚡   │  执行流
│    🔍   │  搜索
│    ⌘K   │  命令
│  ─────  │
│    ⚙️   │  设置
│         │
└─────────┘
```

**设计特点：**
- 极简图标，无文字
- macOS 风格的 traffic lights
- 悬浮提示 (tooltip)
- 激活态有底部指示条

### 4. 可折叠底部面板

```
┌─────────────────────────────────────────────────┐
│                                                 │
│              代码编辑区域                        │
│                                                 │
├─────────────────────────────────────────────────┤
│  [Terminal]  [Output]  [Problems]     ────────  │ ◀ 展开/折叠
│  ─────────────────────────────────────────────  │
│  $ cargo build                                  │
│  Compiling flow v0.1.0...                       │
│  Finished `dev` in 2.3s                         │
└─────────────────────────────────────────────────┘
```

---

## 性能优化

### 1. 渲染性能

- **Jotai 原子化**：只重渲染变化的组件
- **React.memo**：纯展示组件
- **虚拟滚动**：长列表（大纲、搜索结果）
- **代码分割**：懒加载大型组件

### 2. 动画性能

```css
/* GPU 加速 */
.optimize-animation {
  will-change: transform, opacity;
  transform: translateZ(0);
  backface-visibility: hidden;
}
```

### 3. 内存优化

- **WeakMap 缓存**：计算结果
- **事件委托**：减少监听器
- **清理副作用**：组件卸载时

### 4. 跨平台兼容

```css
/* 字体回退链 */
font-family: 'SF Pro', 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;

/* 颜色空间 */
color: hsl(var(--foreground));
```

---

## 实施计划

### Phase 1: 基础架构 (5 天)

- [ ] 初始化 Vite + React 19 项目
- [ ] 配置 Tailwind CSS + shadcn/ui
- [ ] 设置 Jotai 状态管理
- [ ] 实现基础 UI 组件 (Button, Input, Dialog, Popover, Command)

### Phase 2: 布局组件 (5 天)

- [ ] 实现 MainLayout 主布局
- [ ] 实现 Sidebar 左侧图标栏
- [ ] 实现 Header 顶部栏
- [ ] 实现 StatusBar 状态栏
- [ ] 实现悬浮面板组件 (FloatingPanel)

### Phase 3: 命令面板 (3 天)

- [ ] 实现 ⌘K 命令面板
- [ ] 集成文件搜索
- [ ] 集成符号搜索
- [ ] 键盘导航支持

### Phase 4: 功能面板 (7 天)

- [ ] 实现 Outline 大纲面板
- [ ] 实现 NodeDetail 节点详情面板
- [ ] 实现 LlvmIrPanel LLVM IR 面板
- [ ] 实现 Explorer 文件浏览器面板
- [ ] 实现 Terminal 面板

### Phase 5: 执行流视图 (5 天)

- [ ] 集成 @xyflow/react
- [ ] 实现 FlowView 组件
- [ ] 实现节点样式
- [ ] 集成代码-图联动

### Phase 6: 代码编辑器 (3 天)

- [ ] 集成 Monaco Editor
- [ ] 实现语法高亮
- [ ] 集成代码导航

### Phase 7: 优化与完善 (5 天)

- [ ] 性能测试与优化
- [ ] 动画调优 (Motion)
- [ ] 跨平台测试 (macOS/Linux/Windows)
- [ ] 可访问性检查
- [ ] 文档完善

**总计：约 5-6 周**

---

## 设计规范

### 圆角规范

| 组件 | 圆角值 |
|------|--------|
| 按钮 | 8px |
| 输入框 | 8px |
| 卡片 | 12px |
| 对话框 | 16px |
| 悬浮面板 | 16px |
| 标签页 | 10px |

### 阴影层次

| 场景 | 阴影 |
|------|------|
| 按钮悬浮 | `0 2px 8px rgba(0,0,0,0.2)` |
| 面板悬浮 | `0 12px 48px rgba(0,0,0,0.6)` |
| 对话框 | `0 24px 64px rgba(0,0,0,0.8)` |

### 动画曲线

```css
/* 面板滑入 */
transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
transition-duration: 200ms;

/* 悬浮效果 */
transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
transition-duration: 150ms;
```

---

## 验收标准

| 维度 | 标准 |
|------|------|
| **视觉** | 极简美观，超越 Cursor |
| **动画** | 60fps 流畅，无卡顿 |
| **交互** | 键盘操作完整，⌘K 响应 |
| **性能** | 内存占用低，冷启动 < 3s |
| **跨平台** | 三平台表现一致 |
| **可访问性** | 键盘导航完整，屏幕阅读器支持 |

---

## 风险与对策

| 风险 | 应对措施 |
|------|----------|
| 设计效果不达预期 | 参考 21st.dev，持续迭代 |
| 性能不达标 | 早期性能测试，持续优化 |
| 悬浮面板交互复杂 | 简化逻辑，提供多种打开方式 |
| 跨平台差异 | 自动化测试，定期验证 |

---

## 参考资源

### 设计参考

- [21st.dev](https://21st.dev) - 极简设计参考
- [shadcn/ui](https://ui.shadcn.com) - 组件库
- [Radix UI](https://radix-ui.com) - 无头组件
- [Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines/) - macOS 设计

### 代码参考

- `/tmp/1code/src/renderer/components/ui/` - 1code 的 shadcn/ui 实现
- `/tmp/1code/src/renderer/features/layout/agents-layout.tsx` - 布局模式

---

## 附录

### CSS 变量速查

```css
:root {
  /* 背景 */
  --background: #0d0d0d;
  --surface: #141414;
  --surface-hover: #1f1f1f;

  /* 文字 */
  --foreground: #f2f2f2;
  --foreground-muted: #999;

  /* 品牌 */
  --primary: #0034FF;
  --primary-hover: #264DFF;

  /* 边框 */
  --border: #1a1a1a;
  --border-light: #262626;

  /* 圆角 */
  --radius-sm: 6px;
  --radius-md: 10px;
  --radius-lg: 14px;
  --radius-xl: 16px;

  /* 阴影 */
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.3);
  --shadow-md: 0 4px 16px rgba(0,0,0,0.4);
  --shadow-lg: 0 8px 32px rgba(0,0,0,0.5);
  --shadow-float: 0 12px 48px rgba(0,0,0,0.6);
}
```

### 快捷键速查

| 快捷键 | 功能 |
|--------|------|
| `⌘K` | 打开命令面板 |
| `⌘B` | 切换侧边栏 |
| `⌘J` | 切换底部面板 |
| `⌘P` | 快速打开文件 |
| `⌘⇧P` | 高级命令 |
| `⌘W` | 关闭标签 |
| `⌘Tab` | 切换标签 |
| `⌘[` / `⌘]` | 后退/前进 |
