# FlowSight UI Design Skill

> 为 FlowSight 项目创建现代化、美观的 UI 界面组件

## 使用场景

当需要为 FlowSight 项目创建或美化 UI 组件时使用此 Skill。

## 使用方式

```
/sc:ui-design "创建节点详情面板组件"
/sc:ui-design "美化工具栏样式"
/sc:ui-design "设计场景对比视图"
/sc:ui-design "改进执行流图节点样式"
```

## 设计原则

### 1. FlowSight 设计规范

```typescript
// 配色方案
const FlowSightTheme = {
  colors: {
    // 主色调 - 深色科技感
    primary: '#3b82f6',      // 蓝色 - 主色
    primaryHover: '#2563eb',
    secondary: '#8b5cf6',    // 紫色 - 次要

    // 背景色
    background: '#0c1222',   // 深蓝黑背景
    surface: '#151e32',      // 表面色
    surfaceLight: '#1e293b', // 浅表面色

    // 文字色
    text: '#f1f5f9',         // 主文字
    textSecondary: '#94a3b8',// 次要文字
    textMuted: '#64748b',    // 弱文字

    // 状态色
    success: '#22c55e',      // 成功
    warning: '#f59e0b',      // 警告
    error: '#ef4444',        // 错误
    info: '#3b82f6',         // 信息

    // 异步机制颜色
    asyncWorkQueue: '#f59e0b',
    asyncTimer: '#22c55e',
    asyncIrq: '#ef4444',
    asyncTasklet: '#a855f7',
    asyncKThread: '#3b82f6',
  },

  // 渐变效果
  gradients: {
    primary: 'linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%)',
    surface: 'linear-gradient(180deg, #1e293b 0%, #0c1222 100%)',
    glow: '0 0 20px rgba(59, 130, 246, 0.3)',
  },

  // 圆角
  radius: {
    sm: '4px',
    md: '8px',
    lg: '12px',
    xl: '16px',
    full: '9999px',
  },

  // 阴影
  shadows: {
    sm: '0 1px 2px rgba(0, 0, 0, 0.3)',
    md: '0 4px 6px rgba(0, 0, 0, 0.4)',
    lg: '0 10px 15px rgba(0, 0, 0, 0.5)',
    glow: '0 0 20px rgba(59, 130, 246, 0.3)',
  },

  // 过渡
  transitions: {
    fast: '0.15s ease',
    normal: '0.3s ease',
    slow: '0.5s ease',
  }
}
```

### 2. 推荐的 UI 组件风格

| 组件 | 推荐风格 | 来源 |
|------|----------|------|
| 卡片 | 玻璃态 + 渐变边框 | 21st.dev modern card |
| 按钮 | 渐变 + 微光效果 | 21st.dev button |
| 侧边栏 | 深色 + 玻璃态 | 21st.dev sidebar |
| 弹窗 | 居中 + 模糊背景 | 21st.dev modal |
| 工具提示 | 暗色主题 + 箭头 | 21st.dev tooltip |
| 图表节点 | 发光边框 + 渐变 | custom |
| 表格 | 条纹 + 悬停效果 | 21st.dev table |
| 输入框 | 底部边框 + 聚焦发光 | 21st.dev input |

### 3. 交互模式

```typescript
// 推荐的交互模式
const Interactions = {
  // 悬停效果
  hover: {
    scale: 1.02,
    brightness: 1.1,
    shadow: '0 0 12px rgba(59, 130, 246, 0.4)',
  },

  // 点击效果
  click: {
    scale: 0.98,
    brightness: 0.95,
  },

  // 选中状态
  selected: {
    borderColor: 'var(--primary)',
    backgroundColor: 'rgba(59, 130, 246, 0.1)',
    glow: '0 0 16px rgba(59, 130, 246, 0.3)',
  },

  // 动画
  animations: {
    fadeIn: 'opacity 0.3s ease',
    slideUp: 'transform 0.3s ease, opacity 0.3s ease',
    scaleIn: 'transform 0.2s ease, opacity 0.2s ease',
  }
}
```

## 实现步骤

### Step 1: 获取组件模板

使用 21st.dev MCP 工具获取高质量组件：

```
# 搜索命令格式
/ui <组件描述> --style modern --dark-mode

# 示例
/ui modern sidebar navigation panel --dark-mode
/ui command palette search --style modern
/ui graph node visualization --style modern --dark-mode
/ui modal dialog with form --style modern
/ui tooltip hover card --style modern
```

### Step 2: 自定义组件

根据 FlowSight 设计规范调整组件：

```typescript
// 示例：自定义卡片组件
import { Card } from './Card'

const FlowSightCard = ({ children, variant = 'default' }) => {
  const variants = {
    default: 'bg-surface border border-white/10',
    elevated: 'bg-surface border border-white/10 shadow-lg',
    glow: 'bg-surface border border-primary/50 shadow-lg shadow-primary/20',
  }

  return (
    <Card className={`${variants[variant]} rounded-xl backdrop-blur-sm`}>
      {children}
    </Card>
  )
}
```

### Step 3: 添加动画

使用 Framer Motion 实现平滑动画：

```typescript
import { motion } from 'framer-motion'

// 卡片悬停动画
const Card = ({ children }) => (
  <motion.div
    whileHover={{ scale: 1.02 }}
    whileTap={{ scale: 0.98 }}
    transition={{ type: 'spring', stiffness: 300 }}
    className="..."
  >
    {children}
  </motion.div>
)

// 面板滑入动画
const Panel = ({ isOpen }) => (
  <motion.div
    initial={{ opacity: 0, x: 20 }}
    animate={{ opacity: isOpen ? 1 : 0, x: isOpen ? 0 : 20 }}
    exit={{ opacity: 0, x: 20 }}
    transition={{ duration: 0.3 }}
  >
    {children}
  </motion.div>
)
```

### Step 4: 响应式设计

```typescript
// 使用 Tailwind 响应式类
const ResponsiveLayout = {
  // 移动端：单栏
  mobile: 'grid grid-cols-1 gap-4',

  // 平板：双栏
  tablet: 'grid grid-cols-2 gap-6',

  // 桌面：三栏
  desktop: 'grid grid-cols-3 gap-6 lg:grid-cols-4',

  // 大屏：四栏
  wide: 'grid grid-cols-4 gap-6 xl:grid-cols-5',
}
```

## 常用组件模板

### 1. 执行流节点 (FlowNode)

```tsx
// 高性能、美观的执行流节点
export function FlowNode({ node, isSelected, isExpanded }) {
  return (
    <motion.div
      className={`
        relative px-4 py-3 rounded-lg
        bg-surface/80 backdrop-blur
        border ${isSelected ? 'border-primary' : 'border-white/10'}
        ${isSelected ? 'shadow-lg shadow-primary/20' : 'shadow-md'}
        transition-all duration-200
        hover:border-white/20 hover:shadow-lg
        cursor-pointer
      `}
      whileHover={{ scale: 1.02 }}
      whileTap={{ scale: 0.98 }}
    >
      {/* 发光效果 */}
      {isSelected && (
        <div className="absolute inset-0 rounded-lg bg-primary/20 blur-xl -z-10" />
      )}

      {/* 异步机制标签 */}
      {node.asyncLabel && (
        <span className={`
          absolute -top-2 -right-2 px-2 py-0.5 text-xs rounded-full
          bg-${node.asyncColor || 'primary'}/20 text-${node.asyncColor || 'primary'}
          border border-${node.asyncColor || 'primary'}/30
        `}>
          {node.asyncLabel}
        </span>
      )}

      {/* 内容 */}
      <div className="flex items-center gap-2">
        <span className="text-lg">{node.icon}</span>
        <span className="font-medium text-white">{node.name}</span>
        {node.children?.length > 0 && (
          <span className="text-xs text-muted">
            {node.children.length}
          </span>
        )}
      </div>
    </motion.div>
  )
}
```

### 2. 侧边面板 (SidePanel)

```tsx
// 现代化侧边面板
export function SidePanel({ title, children, isOpen, onClose }) {
  return (
    <motion.aside
      initial={{ x: 320, opacity: 0 }}
      animate={{ x: isOpen ? 0 : 320, opacity: isOpen ? 1 : 0 }}
      transition={{ type: 'spring', damping: 25 }}
      className="fixed right-0 top-0 h-full w-80"
    >
      <div className="h-full ml-4 bg-surface/95 backdrop-blur-xl border-l border-white/10 shadow-2xl rounded-l-2xl overflow-hidden">
        {/* 头部 */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-white/10">
          <h3 className="font-semibold text-white">{title}</h3>
          <button
            onClick={onClose}
            className="p-1 rounded-lg hover:bg-white/10 transition-colors"
          >
            <X className="w-5 h-5 text-muted" />
          </button>
        </div>

        {/* 内容 */}
        <div className="p-4 overflow-y-auto h-[calc(100%-52px)]">
          {children}
        </div>
      </div>
    </motion.aside>
  )
}
```

### 3. 工具栏 (Toolbar)

```tsx
// 现代化工具栏
export function Toolbar({ actions }) {
  return (
    <div className="flex items-center gap-1 px-3 py-2 bg-surface/80 backdrop-blur-md border-b border-white/10">
      {actions.map((action, index) => (
        <Tooltip key={index} content={action.label}>
          <button
            onClick={action.onClick}
            className={`
              p-2 rounded-lg transition-all duration-200
              ${action.active
                ? 'bg-primary/20 text-primary'
                : 'text-muted hover:text-white hover:bg-white/10'
              }
            `}
          >
            <span className="text-lg">{action.icon}</span>
          </button>
        </Tooltip>
      ))}

      {/* 分隔线 */}
      <div className="w-px h-6 mx-2 bg-white/10" />

      {/* 深度选择器 */}
      <DepthSelector />
    </div>
  )
}
```

### 4. 详情面板 (DetailPanel)

```tsx
// 多级详情展示面板
export function DetailPanel({ data, activeTab }) {
  const tabs = [
    { id: 'function', label: '函数', icon: '📦' },
    { id: 'branch', label: '分支', icon: '🔀' },
    { id: 'statement', label: '语句', icon: '📝' },
  ]

  return (
    <div className="bg-surface/95 backdrop-blur-xl border border-white/10 rounded-xl overflow-hidden">
      {/* 标签页头部 */}
      <div className="flex items-center gap-1 px-2 py-1 border-b border-white/10 bg-surface/50">
        {tabs.map(tab => (
          <button
            key={tab.id}
            className={`
              px-3 py-1.5 rounded-lg text-sm transition-all duration-200
              ${activeTab === tab.id
                ? 'bg-primary/20 text-primary'
                : 'text-muted hover:text-white hover:bg-white/5'
              }
            `}
          >
            <span className="mr-1">{tab.icon}</span>
            {tab.label}
          </button>
        ))}
      </div>

      {/* 内容区域 */}
      <div className="p-4">
        <AnimatePresence mode="wait">
          <motion.div
            key={activeTab}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2 }}
          >
            {activeTab === 'function' && <FunctionView data={data} />}
            {activeTab === 'branch' && <BranchView data={data} />}
            {activeTab === 'statement' && <StatementView data={data} />}
          </motion.div>
        </AnimatePresence>
      </div>
    </div>
  )
}
```

## 样式最佳实践

### 1. 使用 CSS 变量

```css
/* app/src/styles/variables.css */
:root {
  /* 基础颜色 */
  --color-primary: #3b82f6;
  --color-primary-hover: #2563eb;
  --color-background: #0c1222;
  --color-surface: #151e32;

  /* 透明度变体 */
  --color-primary/10: rgba(59, 130, 246, 0.1);
  --color-primary/20: rgba(59, 130, 246, 0.2);

  /* 边框 */
  --border-light: rgba(255, 255, 255, 0.1);
  --border-medium: rgba(255, 255, 255, 0.2);

  /* 阴影 */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.4);
  --shadow-glow: 0 0 20px rgba(59, 130, 246, 0.3);
}
```

### 2. 使用 Tailwind 插件

```javascript
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: '#151e32',
          light: '#1e293b',
          dark: '#0c1222',
        },
      },
      backdropBlur: {
        xs: '2px',
      },
      animation: {
        'fade-in': 'fadeIn 0.3s ease',
        'slide-up': 'slideUp 0.3s ease',
        'glow': 'glow 2s ease-in-out infinite',
      },
    },
  },
}
```

### 3. 使用 clsx 或 classcat 管理类名

```typescript
import { clsx } from 'clsx'

const cardClass = clsx(
  'rounded-xl p-4 transition-all duration-200',
  'bg-surface/80 backdrop-blur',
  'border border-white/10',
  {
    'hover:border-white/20 hover:shadow-lg': !disabled,
    'opacity-50 cursor-not-allowed': disabled,
    'ring-2 ring-primary': isSelected,
  }
)
```

## 常见问题

### Q: 如何保持 UI 一致性？

A: 遵循以下原则：
1. 使用统一的设计 token（颜色、间距、圆角）
2. 复用基础组件（Button、Card、Input）
3. 使用相同的动画曲线和时长
4. 保持相似组件的视觉层次一致

### Q: 如何优化性能？

A:
1. 使用 `React.memo` 包装静态组件
2. 使用 `useMemo` 缓存计算结果
3. 对大型列表使用虚拟滚动
4. 延迟加载非关键组件
5. 使用 CSS 动画代替 JS 动画

### Q: 如何支持深色/浅色主题？

A:
```typescript
// 使用 CSS 变量
className={theme === 'dark' ? 'bg-dark text-light' : 'bg-light text-dark'}

// 或使用 Tailwind dark: 前缀
className="bg-white dark:bg-surface-light text-gray-900 dark:text-white"
```

## 参考资源

- [21st.dev](https://21st.dev/) - 高质量 React 组件市场
- [Tailwind CSS](https://tailwindcss.com/) - 实用优先的 CSS 框架
- [Framer Motion](https://www.framer.com/motion/) - 动画库
- [Radix UI](https://www.radix-ui.com/) - 无样式组件库
- [Lucide Icons](https://lucide.dev/) - 图标库

---

*FlowSight UI Design Skill - 2026*
