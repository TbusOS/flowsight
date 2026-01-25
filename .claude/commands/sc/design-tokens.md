---
name: design-tokens
description: "FlowSight Design Tokens Skill - Theming and design system"
category: frontend
complexity: standard
mcp-servers: []
personas: []
---

# /sc:design-tokens - Design Tokens

## Triggers
- Design token and theme requests
- Design system implementation
- Theming and styling consistency
- Token-based styling

## Usage
```
/sc:design-tokens "<task description>"
/sc:tokens "<task description>"
```

## FlowSight Theme Tokens

### Color Tokens
```typescript
const FlowSightColors = {
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
  accent: {
    success: '#22c55e',
    warning: '#f59e0b',
    error: '#ef4444',
    info: '#3b82f6',
  },
};
```

### Semantic Tokens
```css
:root {
  /* Colors */
  --color-primary: var(--color-primary-500);
  --color-primary-hover: var(--color-primary-600);
  --color-background: var(--color-surface-900);
  --color-surface: var(--color-surface-800);
  --color-surface-light: var(--color-surface-700);
  --color-text: var(--color-surface-50);
  --color-text-secondary: var(--color-surface-200);
  --color-text-muted: var(--color-surface-400);

  /* Shadows */
  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1);
  --shadow-glow: 0 0 20px rgba(59, 130, 246, 0.3);

  /* Transitions */
  --transition-fast: 150ms ease-out;
  --transition-normal: 200ms ease-out;
  --transition-slow: 300ms ease-in-out;

  /* Border Radius */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;
}
```

## Implementation Patterns

### 1. CSS Variables
```css
:root {
  --color-primary-500: #3b82f6;
  --color-primary-600: #2563eb;
  --color-surface-800: #1e293b;
  --color-surface-900: #0f172a;
}
```

### 2. Tailwind CSS Config
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
          800: '#1e293b',
          900: '#0f172a',
        },
      },
      boxShadow: {
        'node': '0 4px 6px -1px rgb(0 0 0 / 0.1)',
        'node-hover': '0 10px 15px -3px rgb(0 0 0 / 0.1)',
        'glow': '0 0 20px rgba(59, 130, 246, 0.3)',
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

### 3. TypeScript Types
```typescript
// design-tokens.ts
export const tokens = {
  colors: {
    primary: { 50: '#eff6ff', 500: '#3b82f6', 600: '#2563eb' },
    surface: { 50: '#f8fafc', 800: '#1e293b', 900: '#0f172a' },
  },
  shadows: {
    node: '0 4px 6px -1px rgb(0 0 0 / 0.1)',
    'node-hover': '0 10px 15px -3px rgb(0 0 0 / 0.1)',
  },
} as const;

export type ColorToken = typeof tokens.colors.primary[keyof typeof tokens.colors.primary];
export type ShadowToken = keyof typeof tokens.shadows;
```

### 4. Dark Mode Support
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

## Best Practices

| Practice | Description |
|----------|-------------|
| Naming | Use semantic names over descriptive |
| Hierarchy | Primitives → Semantic → Component |
| Type Safety | TypeScript type checking |
| Automation | Style Dictionary for generation |
| Documentation | Document each token usage |

## Tool Coordination
- **Bash**: Run frontend build and test

## Examples

### Create Design Token System
```
/sc:design-tokens "设计令牌系统"
```

### Implement Dark Theme
```
/sc:tokens "实现深色主题"
```

### Define Color Tokens
```
/sc:design-tokens "定义颜色令牌"
```

## Boundaries

**Will:**
- Create consistent design tokens
- Support dark mode and theming
- Provide type-safe token access

**Will Not:**
- Create inconsistent token naming
- Duplicate token values

## See Also
- `/sc:dark-mode` - Dark mode implementation
- `/sc:ui-design` - UI component design
- `/sc:build` - Build verification
