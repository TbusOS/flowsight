---
name: xyflow-advanced
description: "FlowSight XyFlow Advanced Skill - Professional flow visualization with @xyflow/react"
category: frontend
complexity: advanced
mcp-servers: [magic]
personas: []
---

# /sc:xyflow-advanced - XyFlow Advanced

## Triggers
- XyFlow/React Flow component requests
- Custom node and edge design
- Flow visualization enhancement
- Performance optimization for large graphs

## Usage
```
/sc:xyflow-advanced "<task description>"
```

## FlowSight XyFlow Configuration

```typescript
// XyFlow config for FlowSight
const flowSightFlowConfig = {
  nodesDraggable: true,
  nodesConnectable: true,
  elementsSelectable: true,
  zoomOnScroll: true,
  zoomOnDoubleClick: true,
  panOnScroll: false,
  panOnDrag: true,
  preventScrolling: true,
};
```

## Custom Node Types

### Code Execution Node
```tsx
const CodeNode: Node<CodeNodeData> = ({ data, id, selected }) => {
  return (
    <div className={`code-node ${selected ? 'selected' : ''}`}>
      <NodeHeader icon={getLanguageIcon(data.language)} label={data.label} />
      <div className="code-content">
        <MonacoEditor value={data.code} language={data.language} readOnly />
      </div>
      <NodeHandles type="target" position={Position.Left} />
      <NodeHandles type="source" position={Position.Right} />
    </div>
  );
};
```

### Flowing Edge with Animation
```tsx
const FlowingEdge = ({ id, sourceX, sourceY, targetX, targetY }) => {
  const path = calculateBezierPath({ sourceX, sourceY, targetX, targetY, curvature: 0.5 });

  return (
    <>
      <path id={id} d={path} className="flowing-edge-path" markerEnd="url(#arrowhead)" />
      <circle r={4} className="flow-particle">
        <animateMotion dur="2s" repeatCount="indefinite" path={path} />
      </circle>
    </>
  );
};
```

## MCP Integration
- **magic (21st.dev)**: Search UI components for flow visualization
- **playwright**: Visual testing of flow components

## Tool Coordination
- **mcp__plugin_superclaude-framework_magic__21st_magic_component_builder**: Build UI components
- **Bash**: Run frontend build and test

## Examples

### Create Custom Node
```
/sc:xyflow-advanced "设计代码执行节点"
/ui graph node code editor --dark-mode
```

### Optimize Large Graph
```
/sc:xyflow-advanced "优化大图性能"
```

### Add Edge Animation
```
/sc:xyflow-advanced "添加连线流动动画"
```

## Performance Optimization

| Strategy | Implementation |
|----------|----------------|
| Node memo | `React.memo()` on custom nodes |
| Selective updates | `useShallow` for state |
| Virtualization | `react-window` for 1000+ nodes |
| Edge caching | `useMemo` for path calculation |

## Boundaries

**Will:**
- Create custom nodes and edges for flow visualization
- Optimize performance for large graphs
- Implement smooth animations and transitions

**Will Not:**
- Create components without performance consideration
- Use inconsistent node types across the graph

## See Also
- `/sc:ui-design` - General UI component design
- `/sc:micro-interaction` - Animation and feedback
- `/sc:immersive-visualization` - Enhanced visualization
