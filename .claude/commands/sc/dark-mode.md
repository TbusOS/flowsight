---
name: dark-mode
description: "FlowSight Dark Mode Theme Implementation"
category: frontend
complexity: standard
mcp-servers: []
personas: []
---

# /sc:dark-mode - Dark Mode Implementation

## Triggers
- Dark theme styling requests
- Theme toggle implementation
- Color scheme application
- Dark mode compatibility fixes

## FlowSight Dark Theme Colors

```typescript
const DarkTheme = {
  colors: {
    // Background layers
    background: '#0c1222',      // Main background
    surface: '#151e32',         // Card/surface background
    surfaceLight: '#1e293b',    // Elevated surface
    surfaceLighter: '#243044',  // Hover state

    // Text
    text: '#f1f5f9',            // Primary text
    textSecondary: '#94a3b8',   // Secondary text
    textMuted: '#64748b',       // Muted text
    textDisabled: '#475569',    // Disabled text

    // Primary
    primary: '#3b82f6',         // Primary blue
    primaryHover: '#2563eb',
    primaryLight: '#60a5fa',

    // Secondary
    secondary: '#8b5cf6',       // Purple accent
    secondaryHover: '#7c3aed',

    // Status
    success: '#22c55e',
    warning: '#f59e0b',
    error: '#ef4444',
    info: '#3b82f6',

    // Async mechanisms
    asyncWorkQueue: '#f59e0b',
    asyncTimer: '#22c55e',
    asyncIrq: '#ef4444',
    asyncTasklet: '#a855f7',
    asyncKThread: '#3b82f6',
  },

  // Transparency variants
  alpha: {
    primary10: 'rgba(59, 130, 246, 0.1)',
    primary20: 'rgba(59, 130, 246, 0.2)',
    surface80: 'rgba(21, 30, 50, 0.8)',
  },

  // Borders
  border: {
    subtle: 'rgba(255, 255, 255, 0.05)',
    light: 'rgba(255, 255, 255, 0.1)',
    medium: 'rgba(255, 255, 255, 0.2)',
  },
}
```

## Implementation Examples

### Apply Dark Theme to Component
```tsx
// ✅ Correct
<div className="bg-surface text-text border border-border-light">
  Content
</div>

// ❌ Avoid
<div className="bg-gray-900 text-gray-100">
  Content
</div>
```

### Glassmorphism Effect
```tsx
<div className="
  bg-surface/80 backdrop-blur-xl
  border border-white/10
  shadow-lg
">
  Glass effect panel
</div>
```

### Glow Effect
```tsx
<div className="
  bg-surface
  border border-primary/50
  shadow-lg shadow-primary/20
">
  Glowing element
</div>
```

## Boundaries

**Will:**
- Use FlowSight color tokens consistently
- Apply dark theme by default
- Use glassmorphism appropriately

**Will Not:**
- Use hard-coded gray colors
- Create light theme without explicit request
- Mix different color systems

## See Also
- `/sc:ui-design` - UI styling
- `/sc:design-tokens` - Design tokens
