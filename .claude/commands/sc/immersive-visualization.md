---
name: immersive-visualization
description: "FlowSight Immersive Visualization Skill - 2025 cutting-edge visualization design"
category: frontend
complexity: advanced
mcp-servers: [magic]
personas: []
---

# /sc:immersive-visualization - Immersive Visualization

## Triggers
- Immersive experience requests
- 3D elements and dynamic effects
- Visualization enhancement
- Dynamic flow animation design

## Usage
```
/sc:immersive-visualization "<task description>"
/sc:immersive "<task description>"
```

## 2025 Design Trends

### Core Principles

1. **Beyond Flat Design**
   - Minimalism with intentional details
   - Strategic color accents
   - Immersive 3D elements

2. **Motion as Feedback**
   - Animations for functional guidance
   - Smooth transitions and responsive gestures
   - Instant feedback mechanisms

3. **Post-Neumorphism**
   - Shadows and bevels for depth
   - Component clarity
   - Restrained usage

## Design Patterns

### 1. Depth and Layers
```css
.node {
  background: var(--surface);
  box-shadow:
    0 4px 6px -1px rgba(0, 0, 0, 0.1),
    0 2px 4px -1px rgba(0, 0, 0, 0.06);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.node:hover {
  transform: translateY(-2px);
  box-shadow:
    0 10px 15px -3px rgba(0, 0, 0, 0.1),
    0 4px 6px -2px rgba(0, 0, 0, 0.05);
}
```

### 2. Dynamic Edge Animation
```tsx
const AnimatedEdge = ({ id, data }) => {
  return (
    <g>
      <defs>
        <linearGradient id={`gradient-${id}`}>
          <stop offset="0%" stopColor="var(--primary)" />
          <stop offset="100%" stopColor="var(--secondary)" />
        </linearGradient>
      </defs>
      <path
        className="animated-flow"
        d={calculatePath(data)}
        stroke={`url(#gradient-${id})`}
        style={{ animation: 'flowPulse 2s infinite' }}
      />
    </g>
  );
};
```

### 3. 3D Node Effect
```tsx
const Node3D: React.FC<{ data: NodeData }> = ({ data }) => {
  return (
    <div className="node-3d" style={{ perspective: '1000px' }}>
      <div
        className="node-content"
        style={{
          transform: 'rotateX(5deg)',
          transformStyle: 'preserve-3d',
        }}
      >
        <div className="node-front">{data.label}</div>
        <div className="node-side" />
      </div>
    </div>
  );
};
```

## Visualization Types

| Type | Use Case | Visual Effect |
|------|----------|---------------|
| Flow | Execution path | Gradient flow animation |
| Dependency | Module relationships | Dynamic edges, highlights |
| State machine | State transitions | Pulse effects |
| Call stack | Function calls | Expand/collapse animation |

## MCP Integration
- **magic (21st.dev)**: Search visualization components
- **playwright**: Visual testing

## Tool Coordination
- **mcp__plugin_superclaude-framework_magic__21st_magic_component_builder**: Build components
- **Bash**: Run frontend build and test

## Examples

### Design Flow Animation
```
/sc:immersive-visualization "设计流程图动画"
```

### Add 3D Node Effect
```
/sc:immersive "添加3D节点效果"
```

### Enhance Visualization
```
/sc:immersive-visualization "增强执行流可视化"
```

## Performance Optimization
```tsx
// Virtualization for large graphs
const VirtualizedFlow = ({ nodes, edges }) => {
  const { visibleNodes } = useVirtualization({
    items: nodes,
    containerHeight: 800,
    itemHeight: 100,
  });

  return <ReactFlow nodes={visibleNodes} edges={edges} onlyRenderVisibleNodes />;
};
```

## Boundaries

**Will:**
- Create visually stunning yet performant visualizations
- Apply depth, motion, and 3D effects strategically
- Optimize for large graphs using virtualization

**Will Not:**
- Create visual effects that degrade performance
- Overuse 3D effects that reduce clarity

## See Also
- `/sc:ui-design` - General UI design
- `/sc:xyflow-advanced` - Flow visualization
- `/sc:micro-interaction` - Animation details
