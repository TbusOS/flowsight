# /sc:immersive-visualization - 沉浸式可视化设计

> FlowSight 沉浸式可视化设计技能 - 2025 前沿设计趋势

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "沉浸", "immersive" | 沉浸式体验 |
| "3D", "立体" | 3D 元素 |
| "动态", "dynamic" | 动态效果 |
| "可视化增强" | 可视化升级 |

## 2025 前沿设计理念

现代可视化设计正朝着更动态、更具沉浸感的方向发展：

### 核心趋势

1. **超越扁平设计 (Beyond Flat Design)**
   - 保留极简主义的可用性
   - 添加微妙但有意的细节
   - 战略性色彩点缀
   - 沉浸式 3D 元素

2. **运动即反馈 (Motion as Feedback)**
   - 动画不只是美学，更是功能性引导
   - 平滑过渡和响应手势
   - 即时反馈机制

3. **后新拟态 (Post-Neumorphism)**
   - 使用阴影和斜面创造深度
   - 保持组件清晰度
   - 避免过度使用

## 设计原则

### 1. 深度与层次

```css
/* 深度层次设计 */
.node {
  /* 基础层 */
  background: var(--surface);
  /* 悬浮效果 */
  box-shadow:
    0 4px 6px -1px rgba(0, 0, 0, 0.1),
    0 2px 4px -1px rgba(0, 0, 0, 0.06);
  /* 悬浮时增强 */
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.node:hover {
  transform: translateY(-2px);
  box-shadow:
    0 10px 15px -3px rgba(0, 0, 0, 0.1),
    0 4px 6px -2px rgba(0, 0, 0, 0.05);
}
```

### 2. 动态连接线效果

```typescript
// 连接线流动动画
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
        style={{
          animation: 'flowPulse 2s infinite',
        }}
      />
    </g>
  );
};
```

### 3. 3D 节点展示

```tsx
// 3D 节点效果
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

## 可视化类型

| 类型 | 应用场景 | 视觉效果 |
|------|----------|----------|
| 流程流 | 执行路径展示 | 渐变流动动画 |
| 依赖图 | 模块依赖关系 | 动态连线、高亮 |
| 状态机 | 状态转换 | 脉冲效果 |
| 调用栈 | 函数调用链 | 展开/收起动画 |

## 性能优化

```typescript
// 使用虚拟化处理大量节点
const VirtualizedFlow = ({ nodes, edges }) => {
  const { visibleNodes } = useVirtualization({
    items: nodes,
    containerHeight: 800,
    itemHeight: 100,
  });

  return (
    <ReactFlow
      nodes={visibleNodes}
      edges={edges}
      // 启用只渲染可见区域
      onlyRenderVisibleNodes
    />
  );
};
```

## 与其他 Skills 配合

```
1. /sc:immersive-visualization "设计流程图动画"
2. /sc:micro-interaction "添加微交互动效"
3. /sc:build "构建验证"
```

---

**快捷命令**:

```
/sc:immersive "设计沉浸式流程"    # 沉浸式可视化
/sc:immersive "添加3D节点效果"     # 3D 效果设计
```

---

> FlowSight 专用 - 2025 前沿沉浸式可视化设计

**参考资源**:
- [Modern UI Design Trends 2025](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEvXlEUgYJH2o4ouQvZXl0gLF5mWsQLWGvRppsg7p4BbGDX934TpVLfSVNk8eS2PMmWGlW_qrYqCJQvBb4JDD7gBxIthDvohcA-zLUX_KKUtq1EpAWoj3UL2kvcvTn3araumgiT1CU1V7YsSEetaYGvpJSw_UnILMztIqV5hehZ)
