# /sc:dark-mode - 深色模式设计

> FlowSight 深色模式技能 - Lightning Dark 沉浸设计

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "深色模式", "dark mode" | 深色模式 |
| "暗色", "dark theme" | 暗色主题 |
| "夜间模式" | 夜间模式 |
| "OLED" | OLED 优化 |

## 2025 前沿设计理念

深色模式已从可选功能变为默认要求：

### 核心概念

1. **Lightning Dark**
   - 深色模式与动态光效结合
   - 沉浸式体验
   - OLED 友好节能设计

2. **默认深色**
   - 用户期望深色模式
   - 开发者工具默认支持
   - 主题自动切换

## 设计模式

### 1. 现代化深色主题

```css
/* FlowSight 深色主题 */
:root[data-theme='dark'] {
  /* 基础颜色 - 使用深灰色而非纯黑 */
  --bg-primary: #0f1117;
  --bg-secondary: #1a1d27;
  --bg-tertiary: #242836;

  /* 表面层级 */
  --surface-elevated: #1e222d;
  --surface-overlay: #2a2f3d;

  /* 语义颜色 */
  --text-primary: #f0f2f5;
  --text-secondary: #a0a8b8;
  --text-muted: #6b7280;

  /* 强调色 - 降低饱和度 */
  --accent-primary: #60a5fa;
  --accent-secondary: #818cf8;
  --accent-success: #34d399;
  --accent-warning: #fbbf24;
  --accent-error: #f87171;

  /* 边框 */
  --border-default: rgba(255, 255, 255, 0.08);
  --border-hover: rgba(255, 255, 255, 0.12);

  /* 阴影 - 调整为深色阴影 */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
  --shadow-glow: 0 0 20px rgba(96, 165, 250, 0.15);
}
```

### 2. 节点深色样式

```css
/* 深色模式下的节点样式 */
.node {
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  box-shadow: var(--shadow-md);
  color: var(--text-primary);
  transition: all var(--transition-normal);
}

.node:hover {
  background: var(--bg-tertiary);
  border-color: var(--border-hover);
  box-shadow: var(--shadow-glow);
}

.node.selected {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 2px rgba(96, 165, 250, 0.2),
              var(--shadow-glow);
}

.node-running {
  animation: pulse-glow 2s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% {
    box-shadow: var(--shadow-glow);
  }
  50% {
    box-shadow: 0 0 30px rgba(96, 165, 250, 0.3);
  }
}
```

### 3. 代码编辑器深色主题

```json
{
  "editorTheme": {
    "name": "FlowSight Dark",
    "colors": {
      "editor.background": "#0f1117",
      "editor.foreground": "#f0f2f5",
      "editorCursor.foreground": "#60a5fa",
      "editorLineNumber.foreground": "#4b5563",
      "editorLineNumber.activeForeground": "#9ca3af",
      "editor.selectionBackground": "rgba(96, 165, 250, 0.2)",
      "editor.inactiveSelectionBackground": "rgba(96, 165, 250, 0.1)",
      "editorLineHighlightBackground": "rgba(255, 255, 255, 0.03)"
    },
    "tokenColors": [
      {
        "scope": "keyword",
        "settings": { "foreground": "#c084fc" }
      },
      {
        "scope": "function",
        "settings": { "foreground": "#60a5fa" }
      },
      {
        "scope": "string",
        "settings": { "foreground": "#34d399" }
      },
      {
        "scope": "comment",
        "settings": { "foreground": "#6b7280", "fontStyle": "italic" }
      }
    ]
  }
}
```

### 4. 动态主题切换

```typescript
type ThemeMode = 'light' | 'dark' | 'system';

class ThemeManager {
  private currentTheme: ThemeMode = 'system';

  initialize(): void {
    // 读取保存的主题偏好
    const saved = localStorage.getItem('theme');
    if (saved) {
      this.currentTheme = saved as ThemeMode;
    }

    // 监听系统主题变化
    this.watchSystemTheme();
    this.applyTheme();
  }

  private watchSystemTheme(): void {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', () => {
      if (this.currentTheme === 'system') {
        this.applyTheme();
      }
    });
  }

  applyTheme(): void {
    const isDark = this.shouldUseDarkMode();
    const theme = isDark ? 'dark' : 'light';

    document.documentElement.setAttribute('data-theme', theme);
    document.documentElement.classList.remove('light', 'dark');
    document.documentElement.classList.add(theme);
  }

  private shouldUseDarkMode(): boolean {
    if (this.currentTheme === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    return this.currentTheme === 'dark';
  }

  setTheme(mode: ThemeMode): void {
    this.currentTheme = mode;
    localStorage.setItem('theme', mode);
    this.applyTheme();
  }
}
```

### 5. OLED 节能优化

```css
/* OLED 优化 - 使用纯黑背景 */
@media (prefers-color-scheme: dark) {
  /* 对于 OLED 屏幕，使用纯黑 */
  @media (oled: true) {
    :root[data-theme='dark'] {
      --bg-primary: #000000;
      --bg-overlay: #0a0a0a;
    }
  }

  /* 避免使用大块亮色区域 */
  .large-surface {
    background: linear-gradient(
      180deg,
      #0f1117 0%,
      #000000 100%
    );
  }

  /* 减少白色文字占比 */
  .heavy-text-area {
    color: #d1d5db; /* 略微调暗 */
  }
}
```

### 6. 平滑过渡动画

```css
/* 主题切换过渡 */
:root {
  /* 基础过渡设置 */
  --theme-transition-duration: 300ms;
  --theme-transition-easing: cubic-bezier(0.4, 0, 0.2, 1);
}

/* 颜色属性过渡 */
* {
  transition:
    background-color var(--theme-transition-duration) var(--theme-transition-easing),
    border-color var(--theme-transition-duration) var(--theme-transition-easing),
    color var(--theme-transition-duration) var(--theme-transition-easing),
    box-shadow var(--theme-transition-duration) var(--theme-transition-easing);
}

/* 避免过渡的属性 */
.no-theme-transition,
.no-theme-transition * {
  transition: none !important;
}
```

## 深色模式最佳实践

| 实践 | 说明 |
|------|------|
| 使用深灰色 | 避免纯黑，使用 #0f1117 等 |
| 降低饱和度 | 强调色饱和度降低 20% |
| 增加对比度 | 确保文字可读性 |
| OLED 优化 | 纯黑背景节省电量 |
| 平滑过渡 | 300ms 主题切换动画 |

## 与其他 Skills 配合

```
1. /sc:dark-mode "实现深色主题"
2. /sc:design-tokens "定义主题令牌"
3. /sc:sustainable-design "优化 OLED 节能"
```

---

**快捷命令**:

```
/sc:dark "设计深色模式"    # 深色模式
/sc:theme "实现主题切换"   # 主题系统
```

---

> FlowSight 专用 - 深色模式与主题设计

**参考资源**:
- [Modern UI Design Trends 2025](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEvXlEUgYJH2o4ouQvZXl0gLF5mWsQLWGvRppsg7p4BbGDX934TpVLfSVNk8eS2PMmWGlW_qrYqCJQvBb4JDD7gBxIthDvohcA-zLUX_KKUtq1EpAWoj3UL2kvcvTn3araumgiT1CU1V7YsSEetaYGvpJSw_UnILMztIqV5hehZ)
