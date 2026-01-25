---
name: ui-design
description: "FlowSight UI Design Skill - Create modern, beautiful UI components"
category: frontend
complexity: standard
mcp-servers: [magic]
personas: []
---

# /sc:ui-design - FlowSight UI Design

## Triggers
- UI component creation and styling requests
- Frontend component design and beautification
- FlowSight-specific UI development tasks

## Usage
```
/sc:ui-design "<task description>"
/ui <component description> --style modern --dark-mode
```

## Design Principles

### FlowSight Theme
```typescript
const FlowSightTheme = {
  colors: {
    primary: '#3b82f6',
    primaryHover: '#2563eb',
    secondary: '#8b5cf6',
    background: '#0c1222',
    surface: '#151e32',
    surfaceLight: '#1e293b',
    text: '#f1f5f9',
    textSecondary: '#94a3b8',
    textMuted: '#64748b',
    success: '#22c55e',
    warning: '#f59e0b',
    error: '#ef4444',
    info: '#3b82f6',
  },
  gradients: {
    primary: 'linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%)',
    surface: 'linear-gradient(180deg, #1e293b 0%, #0c1222 100%)',
    glow: '0 0 20px rgba(59, 130, 246, 0.3)',
  },
}
```

## MCP Integration
- **magic (21st.dev)**: Search and generate UI components
- **playwright**: Visual testing of UI components

## Tool Coordination
- **mcp__plugin_superclaude-framework_magic__21st_magic_component_builder**: Build UI components
- **mcp__plugin_superclaude-framework_magic__21st_magic_component_inspiration**: Get design inspiration
- **Bash**: Run frontend build and test

## Examples

### Create a Panel Component
```
/sc:ui-design "创建节点详情面板组件"
/ui modern sidebar navigation panel --dark-mode
```

### Beautify Toolbar
```
/sc:ui-design "美化工具栏样式"
```

### Design Flow Visualization Node
```
/sc:ui-design "改进执行流图节点样式"
/ui graph node visualization --style modern --dark-mode
```

## Implementation Steps

1. **Search Components**: Use `/ui` to find matching components
2. **Customize**: Apply FlowSight theme and design tokens
3. **Add Animation**: Use Framer Motion for smooth transitions
4. **Test**: Verify with playwright snapshot testing
5. **Integrate**: Add to FlowSight component library

## Design Tokens
- Border radius: `4px`, `8px`, `12px`, `16px`
- Shadows: `sm`, `md`, `lg`, `glow`
- Transitions: `fast` (0.15s), `normal` (0.3s), `slow` (0.5s)

## Boundaries

**Will:**
- Create modern, dark-themed UI components
- Apply FlowSight design system consistently
- Use glassmorphism and gradient effects

**Will Not:**
- Create components without considering performance
- Use inconsistent styling across components
- Ignore accessibility requirements

## See Also
- `/sc:interaction-design` - User interaction design
- `/sc:implement` - General implementation
- `/sc:build` - Frontend build
