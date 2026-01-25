---
name: micro-interaction
description: "FlowSight Micro-interaction Skill - Delightful animations and user feedback"
category: frontend
complexity: standard
mcp-servers: []
personas: []
---

# /sc:micro-interaction - Micro-interaction Design

## Triggers
- Micro-interaction and animation requests
- User feedback and state transition design
- Motion design and effects
- Interactive feedback implementation

## Usage
```
/sc:micro-interaction "<task description>"
/sc:micro "<task description>"
```

## Animation Principles

### Timing Guidelines

| Interaction Type | Duration | Easing |
|-----------------|----------|--------|
| Hover effect | 150-200ms | ease-out |
| Click feedback | 100-150ms | ease-in |
| State transition | 200-300ms | spring |
| Page transition | 300-500ms | cubic-bezier |

## Design Patterns

### 1. Node Hover Effect
```tsx
const useNodeMicrointeraction = (nodeId: string) => {
  const [isHovered, setIsHovered] = useState(false);
  const [ripples, setRipples] = useState<Ripple[]>([]);

  const handleMouseEnter = (e: React.MouseEvent) => {
    setIsHovered(true);
    const ripple = createRipple(e, nodeId);
    setRipples(prev => [...prev, ripple]);
  };

  const handleMouseLeave = () => {
    setIsHovered(false);
    setTimeout(() => setRipples([]), 300);
  };

  return { isHovered, ripples, handleMouseEnter, handleMouseLeave };
};
```

### 2. Connection Animation
```css
@keyframes connection-pulse {
  0% { stroke-dashoffset: 1000; opacity: 0; }
  50% { opacity: 1; }
  100% { stroke-dashoffset: 0; opacity: 0; }
}

.edge-connecting {
  stroke-dasharray: 1000;
  animation: connection-pulse 0.6s ease-out forwards;
}
```

### 3. State Transition
```tsx
const NodeStateTransition: React.FC<{ active: boolean }> = ({ active }) => {
  return (
    <motion.div
      className={`node ${active ? 'active' : 'inactive'}`}
      animate={{
        scale: active ? 1.05 : 1,
        boxShadow: active
          ? '0 0 20px rgba(59, 130, 246, 0.5)'
          : '0 2px 4px rgba(0, 0, 0, 0.1)',
      }}
      transition={{ type: 'spring', stiffness: 300, damping: 20 }}
    >
      <motion.div
        initial={{ width: 0 }}
        animate={{ width: active ? '100%' : '0%' }}
        transition={{ duration: 0.3 }}
        className="progress-bar"
      />
    </motion.div>
  );
};
```

### 4. Drag Feedback
```tsx
const useDragFeedback = () => {
  const [feedback, setFeedback] = useState({ opacity: 1, scale: 1, shadow: 'medium' });

  const handleDragStart = () => {
    setFeedback({ opacity: 0.8, scale: 1.1, shadow: 'large' });
  };

  const handleDragEnd = () => {
    setFeedback({ opacity: 1, scale: 1, shadow: 'medium' });
  };

  return { feedback, handleDragStart, handleDragEnd };
};
```

## Micro-interaction Types

| Scenario | Effect | Purpose |
|----------|--------|---------|
| Node hover | Slight scale + shadow | Hint interactivity |
| Connection success | Green pulse + check | Confirmation |
| Dragging | Semi-transparent + shadow | State indicator |
| Loading | Skeleton + pulse | Progress awareness |
| Error | Red border + shake | Warning |

## Performance Optimization
```tsx
// Use transform and opacity only for GPU acceleration
<motion.div
  animate={{ x: 100, opacity: 0.8 }}
  style={{ willChange: 'transform, opacity' }}
/>
```

## MCP Integration
- **playwright**: Visual testing of animations

## Tool Coordination
- **Bash**: Run frontend build and test

## Examples

### Design Node Hover
```
/sc:micro-interaction "设计节点悬停效果"
```

### Add Connection Animation
```
/sc:micro "添加连线动画"
```

### Create Loading Animation
```
/sc:micro-interaction "创建加载动画"
```

## Boundaries

**Will:**
- Create performant animations using transform/opacity
- Apply consistent animation patterns
- Consider accessibility (prefers-reduced-motion)

**Will Not:**
- Create excessive animations that slow down the app
- Ignore user preference for reduced motion

## See Also
- `/sc:ui-design` - General UI design
- `/sc:xyflow-advanced` - Flow visualization
- `/sc:a11y-design` - Accessibility considerations
