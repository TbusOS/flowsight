# /sc:design-tokens - 设计令牌系统

> FlowSight 设计令牌技能 - 主题化与设计系统

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "设计令牌", "design token" | 设计令牌 |
| "主题", "theme" | 主题系统 |
| "设计系统", "design system" | 设计系统 |
| "令牌化" | 令牌化设计 |

## 2025 前沿设计理念

设计令牌是现代设计系统的核心：

### 核心工具

1. **Style Dictionary**
   - 跨平台设计令牌管理
   - 自动生成多平台代码
   - 保持设计与代码同步

2. **Tailwind CSS 集成**
   - 使用 Tailwind 进行令牌化
   - 快速主题定制
   - 响应式设计令牌

## 设计令牌结构

### 基础令牌 (Primitive)

```json
{
  "color": {
    "primary": {
      "50": { "value": "#eff6ff" },
      "100": { "value": "#dbeafe" },
      "500": { "value": "#3b82f6" },
      "600": { "value": "#2563eb" },
      "700": { "value": "#1d4ed8" }
    },
    "surface": {
      "50": { "value": "#f8fafc" },
      "100": { "value": "#f1f5f9" },
      "200": { "value": "#e2e8f0" },
      "800": { "value": "#1e293b" },
      "900": { "value": "#0f172a" }
    }
  },
  "spacing": {
    "1": { "value": "0.25rem" },
    "2": { "value": "0.5rem" },
    "4": { "value": "1rem" },
    "8": { "value": "2rem" }
  },
  "shadow": {
    "sm": { "value": "0 1px 2px 0 rgb(0 0 0 / 0.05)" },
    "md": { "value": "0 4px 6px -1px rgb(0 0 0 / 0.1)" },
    "lg": { "value": "0 10px 15px -3px rgb(0 0 0 / 0.1)" }
  }
}
```

### 语义令牌 (Semantic)

```json
{
  "semantic": {
    "background": {
      "default": { "value": "{color.surface.50}" },
      "muted": { "value": "{color.surface.100}" },
      "elevated": { "value": "{color.surface.50}" }
    },
    "text": {
      "primary": { "value": "{color.surface.900}" },
      "secondary": { "value": "{color.surface.600}" },
      "muted": { "value": "{color.surface.400}" }
    },
    "border": {
      "default": { "value": "{color.surface.200}" },
      "focus": { "value": "{color.primary.500}" }
    }
  }
}
```

## 实现模式

### 1. CSS 变量令牌

```css
:root {
  /* 颜色令牌 */
  --color-primary-500: #3b82f6;
  --color-primary-600: #2563eb;
  --color-surface-50: #f8fafc;
  --color-surface-800: #1e293b;
  --color-surface-900: #0f172a;

  /* 语义令牌 */
  --bg-default: var(--color-surface-50);
  --bg-elevated: #ffffff;
  --text-primary: var(--color-surface-900);
  --border-default: var(--color-surface-200);

  /* 阴影令牌 */
  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1);

  /* 过渡令牌 */
  --transition-fast: 150ms ease-out;
  --transition-normal: 200ms ease-out;
  --transition-slow: 300ms ease-in-out;
}
```

### 2. Tailwind CSS 集成

```javascript
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#eff6ff',
          100: '#dbeafe',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
        },
        surface: {
          50: '#f8fafc',
          100: '#f1f5f9',
          200: '#e2e8f0',
          800: '#1e293b',
          900: '#0f172a',
        },
      },
      boxShadow: {
        'node': '0 4px 6px -1px rgb(0 0 0 / 0.1)',
        'node-hover': '0 10px 15px -3px rgb(0 0 0 / 0.1)',
      },
      transitionDuration: {
        'micro': '150ms',
        'normal': '200ms',
        'complex': '300ms',
      },
    },
  },
};
```

### 3. TypeScript 类型安全

```typescript
// design-tokens.ts
export const tokens = {
  colors: {
    primary: {
      50: '#eff6ff',
      100: '#dbeafe',
      500: '#3b82f6',
      600: '#2563eb',
      700: '#1d4ed8',
    },
    surface: {
      50: '#f8fafc',
      100: '#f1f5f9',
      200: '#e2e8f0',
      800: '#1e293b',
      900: '#0f172a',
    },
  },
  shadows: {
    node: '0 4px 6px -1px rgb(0 0 0 / 0.1)',
    'node-hover': '0 10px 15px -3px rgb(0 0 0 / 0.1)',
  },
} as const;

export type ColorToken = typeof tokens.colors.primary[keyof typeof tokens.colors.primary];
export type ShadowToken = keyof typeof tokens.shadows;
```

### 4. 动态主题切换

```typescript
type Theme = 'light' | 'dark' | 'system';

const themeTokens: Record<Theme, DesignTokens> = {
  light: lightTokens,
  dark: darkTokens,
  system: getSystemTheme(),
};

const applyTheme = (theme: Theme) => {
  const tokens = themeTokens[theme];
  Object.entries(tokens.colors).forEach(([key, value]) => {
    document.documentElement.style.setProperty(`--color-${key}`, value);
  });
};
```

## 最佳实践

| 实践 | 说明 |
|------|------|
| 命名规范 | 使用语义化命名而非描述性命名 |
| 层级结构 | 基础令牌 → 语义令牌 → 组件令牌 |
| 类型安全 | TypeScript 类型检查 |
| 自动化 | 使用 Style Dictionary 自动生成 |
| 文档化 | 每个令牌有使用说明 |

## 与其他 Skills 配合

```
1. /sc:design-tokens "设计令牌系统"
2. /sc:dark-mode "实现深色主题"
3. /sc:build "构建验证"
```

---

**快捷命令**:

```
/sc:tokens "设计令牌系统"    # 设计令牌
/sc:theme "设计主题系统"     # 主题设计
```

---

> FlowSight 专用 - 设计令牌与主题系统

**参考资源**:
- [Modern UI Design Trends 2025](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEvXlEUgYJH2o4ouQvZXl0gLF5mWsQLWGvRppsg7p4BbGDX934TpVLfSVNk8eS2PMmWGlW_qrYqCJQvBb4JDD7gBxIthDvohcA-zLUX_KKUtq1EpAWoj3UL2kvcvTn3araumgiT1CU1V7YsSEetaYGvpJSw_UnILMztIqV5hehZ)
