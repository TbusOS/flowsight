# /sc:xyflow-advanced - XyFlow 高级交互模式

> FlowSight XyFlow 高级模式技能 - 专业流程可视化

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "XyFlow", "React Flow" | XyFlow/React Flow |
| "自定义节点", "custom node" | 自定义节点 |
| "高级连线", "advanced edge" | 高级连线 |
| "性能优化" | 性能优化 |

## XyFlow (React Flow) 核心特性

XyFlow 是构建基于节点的用户界面的强大 React 库：

### 核心功能

1. **可定制节点和边**
   - 完全自定义的 React 组件
   - 兼容 Tailwind CSS
   - 灵活的样式系统

2. **内置交互能力**
   - 拖拽节点
   - 缩放和平移
   - 多选
   - 添加/删除元素

3. **性能优化**
   - 内部记忆化
   - 选择性更新
   - 虚拟化

## 高级模式

### 1. 自定义节点模式

```tsx
// 自定义代码节点
const CodeNode: Node<CodeNodeData> = ({ data, id, selected }) => {
  const { code, language, onUpdate } = data;

  return (
    <div
      className={`code-node ${selected ? 'selected' : ''}`}
      style={{ minWidth: 200 }}
    >
      <NodeHeader
        icon={getLanguageIcon(language)}
        label={data.label}
        onDelete={() => removeNodes([id])}
      />
      <div className="code-content">
        <Editor
          value={code}
          language={language}
          onChange={(value) => onUpdate?.(id, value)}
          readOnly={!selected}
        />
      </div>
      <NodeHandles
        type="target"
        position={Position.Left}
        isConnectable={true}
      />
      <NodeHandles
        type="source"
        position={Position.Right}
        isConnectable={true}
      />
    </div>
  );
};

// 注册自定义节点
registerNodeType('code', CodeNode);
```

### 2. 自定义边模式

```tsx
// 流动边 - 带动画效果
const FlowingEdge = ({
  id,
  sourceX, sourceY,
  targetX, targetY,
  data,
}) => {
  const path = calculateBezierPath({
    sourceX, sourceY,
    targetX, targetY,
    curvature: 0.5,
  });

  return (
    <>
      <path
        id={id}
        d={path}
        className="flowing-edge-path"
        markerEnd="url(#arrowhead)"
      />
      {/* 流动动画粒子 */}
      <circle r={4} className="flow-particle">
        <animateMotion
          dur="2s"
          repeatCount="indefinite"
          path={path}
        />
      </circle>
    </>
  );
};

// 浮动边 - 自动寻找连接点
const FloatingEdge = ({
  id,
  sourceHandle,
  targetHandle,
}) => {
  const { findFloatingPosition } = useFloatingEdge();

  return (
    <ConnectionLine
      source={sourceHandle}
      target={targetHandle}
      style={{
        strokeDasharray: '5,5',
        animation: 'flow 1s linear infinite',
      }}
    />
  );
};
```

### 3. 虚拟化大图模式

```tsx
// 虚拟化大量节点
const VirtualizedFlow = ({ nodes, edges }) => {
  const { viewportRef, visibleNodes } = useViewportVirtualization({
    nodes,
    containerHeight: 800,
    overscan: 5,
  });

  return (
    <div ref={viewportRef} className="viewport">
      <ReactFlow
        nodes={visibleNodes}
        edges={edges}
        onlyRenderVisibleNodes
        // 禁用默认渲染
        nodesDraggable={false}
        nodesConnectable={false}
        // 自定义渲染
        nodeTypes={virtualizedNodeTypes}
      />
    </div>
  );
};
```

### 4. 动态布局模式

```tsx
// 自动解决节点重叠
const AutoLayoutFlow = ({ nodes, edges }) => {
  const handleLayout = useCallback(() => {
    const layoutedNodes = dagreLayout(nodes, edges, {
      rankdir: 'LR',
      nodesep: 50,
      ranksep: 100,
    });

    applyNodeChanges(layoutedNodes.map(n => ({
      type: 'position',
      id: n.id,
      position: n.position,
    })));
  }, [nodes, edges]);

  return (
    <>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onLayout={handleLayout}
      />
      <LayoutToolbar onLayout={handleLayout} />
    </>
  );
};

// 使用 dagre 进行自动布局
const dagreLayout = (nodes, edges, options) => {
  const graph = new dagre.graphlib.Graph();
  graph.setGraph(options);
  nodes.forEach(node => graph.setNode(node.id, { width: 150, height: 50 }));
  edges.forEach(edge => graph.setEdge(edge.source, edge.target));
  dagre.layout(graph);
  return nodes.map(node => ({
    ...node,
    position: {
      x: graph.node(node.id).x,
      y: graph.node(node.id).y,
    },
  }));
};
```

### 5. 交互扩展模式

```tsx
// 套索选择
const LassoSelection = () => {
  const [lassoPath, setLassoPath] = useState('');
  const [selectedNodes, setSelectedNodes] = useState([]);

  const handleMouseMove = (e) => {
    if (isDrawing) {
      setLassoPath(prev => `${prev} L ${e.x} ${e.y}`);
      const nodesInPath = findNodesInPolygon(lassoPath);
      setSelectedNodes(nodesInPath);
    }
  };

  return (
    <svg className="lasso-layer">
      <path d={lassoPath} className="lasso-selection" />
      {selectedNodes.map(node => (
        <NodeOverlay key={node.id} node={node} />
      ))}
    </svg>
  );
};

// 小地图
const MiniMap = () => {
  return (
    <ReactFlowMiniMap
      nodeColor={(node) => getNodeTypeColor(node.type)}
      maskColor="rgba(0, 0, 0, 0.1)"
      className="mini-map"
    />
  );
};
```

## 性能优化最佳实践

| 优化项 | 策略 | 实现方式 |
|--------|------|----------|
| 节点渲染 | 记忆化 | `React.memo()` |
| 状态更新 | 选择性更新 | `useShallow` |
| 大量节点 | 虚拟化 | `react-window` |
| 边计算 | 缓存 | `useMemo` |

## 与其他 Skills 配合

```
1. /sc:xyflow-advanced "设计自定义节点"
2. /sc:micro-interaction "添加连接动画"
3. /sc:immersive-visualization "增强可视化效果"
4. /sc:build "构建验证"
```

---

**快捷命令**:

```
/sc:xyflow "设计自定义节点"    # XyFlow 高级模式
/sc:xyflow "优化流程渲染"      # 性能优化
```

---

> FlowSight 专用 - XyFlow 专业流程可视化

**参考资源**:
- [React Flow / XyFlow Official](https://reactflow.dev/)
- [Best Practices for Interactive Design](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHjhtTnQgusCeBSBoyLTmkmYK_tJf4TjpSpjTQrtfCd1TAHwt6PAZJ84tEu4kz1025gaqgKexyeWOOBpG5sVk4LipRXuO02BJQq3yX1qOwUngs=)
