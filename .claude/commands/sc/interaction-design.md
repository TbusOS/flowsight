---
name: interaction-design
description: "FlowSight Interaction Design Skill - User flows, layouts, and panel systems"
category: frontend
complexity: advanced
mcp-servers: [magic]
personas: []
---

# /sc:interaction-design - FlowSight Interaction Design

## Triggers
- User flow design and optimization
- Layout and information architecture
- Panel system and navigation design
- UX improvements and usability enhancements

## Usage
```
/sc:interaction-design "优化执行流程图的用户交互"
/sc:interaction-design "设计场景对比视图的布局"
```

## Key Patterns

### Panel System Design
```typescript
const PanelSystem = {
  detail: {
    width: 400,
    position: 'right',
    animation: 'slide-in',
    backdrop: true,
  },
  sidebar: {
    width: 280,
    position: 'left',
    collapsible: true,
  },
  floating: {
    width: 360,
    position: 'center',
    modal: false,
  },
}
```

### User Flow Design
```typescript
const UserFlows = {
  // 节点查看流程
  nodeView: {
    trigger: 'click',
    panels: ['detail', 'code'],
    animation: 'sequential',
  },
  // 场景对比流程
  comparison: {
    trigger: 'select',
    layout: 'split-view',
    sync: true,
  },
  // 搜索流程
  search: {
    trigger: 'type',
    debounce: 300,
    highlight: true,
  },
}
```

## MCP Integration
- **magic**: Component layout and structure
- **playwright**: Interaction testing

## Tool Coordination
- **mcp__plugin_superclaude-framework_magic__21st_magic_component_builder**: Build interactive components
- **mcp__plugin_superclaude-framework_playwright__browser_snapshot**: Test interactions
- **Bash**: Verify build and tests

## Examples

### Optimize Execution Flow Interaction
```
/sc:interaction-design "优化执行流程图的交互体验"
```

### Design Scenario Comparison View
```
/sc:interaction-design "设计场景对比视图的布局"
/ui split view comparison layout --style modern --dark-mode
```

### Improve Navigation
```
/sc:interaction-design "改进侧边栏导航的交互逻辑"
```

## Interaction Guidelines

### Hover Effects
- Scale: 1.02
- Brightness: 1.1
- Shadow: 0 0 12px rgba(59, 130, 246, 0.4)

### Click Effects
- Scale: 0.98
- Brightness: 0.95

### Selected State
- Border color: var(--primary)
- Background: rgba(59, 130, 246, 0.1)
- Glow: 0 0 16px rgba(59, 130, 246, 0.3)

### Animations
- Fade in: 0.3s ease
- Slide up: 0.3s ease
- Scale in: 0.2s ease

## Boundaries

**Will:**
- Design intuitive user flows
- Create consistent interaction patterns
- Optimize for usability and accessibility

**Will Not:**
- Overcomplicate simple interactions
- Create inconsistent UX across features
- Ignore performance implications

## See Also
- `/sc:ui-design` - Component styling
- `/sc:implement` - Component implementation
- `/sc:a11y-design` - Accessibility
