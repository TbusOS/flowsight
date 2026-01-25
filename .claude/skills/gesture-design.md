# /sc:gesture-design - 手势交互设计

> FlowSight 手势交互技能 - 自然直观的多点触控

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "手势", "gesture" | 手势交互 |
| "触控", "touch" | 触摸操作 |
| "多点触控", "multi-touch" | 多点触控 |
| "滑动", "swipe" | 滑动操作 |

## 2025 前沿设计理念

手势交互提供更自然直观的操作方式：

### 核心概念

1. **交互对象**
   - 复杂 3D 模型交互
   - 富文本手势操作
   - 自然的手势反馈

2. **微交互**
   - 悬停效果
   - 点击反馈
   - 拖拽确认

## 设计模式

### 1. 节点拖拽交互

```typescript
// 高级节点拖拽系统
const useNodeDrag = (nodeId: string) => {
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);

  const handlePointerDown = useCallback((e: React.PointerEvent) => {
    if (e.button !== 0) return; // 只响应左键

    setIsDragging(true);
    const startPos = { x: e.clientX, y: e.clientY };
    const nodeStartPos = { ...position };

    const handlePointerMove = (moveEvent: PointerEvent) => {
      const delta = {
        x: moveEvent.clientX - startPos.x,
        y: moveEvent.clientY - startPos.y,
      };

      setPosition({
        x: nodeStartPos.x + delta.x,
        y: nodeStartPos.y + delta.y,
      });
    };

    const handlePointerUp = () => {
      setIsDragging(false);
      document.removeEventListener('pointermove', handlePointerMove);
      document.removeEventListener('pointerup', handlePointerUp);
    };

    document.addEventListener('pointermove', handlePointerMove);
    document.addEventListener('pointerup', handlePointerUp);
  }, [position, nodeId]);

  return {
    position,
    isDragging,
    handlePointerDown,
  };
};
```

### 2. 缩放与平移手势

```typescript
// 视口缩放和平移
const useViewportGestures = (viewportRef: RefObject<HTMLElement>) => {
  const [viewport, setViewport] = useState({
    x: 0,
    y: 0,
    zoom: 1,
  });

  const handleWheel = useCallback((e: WheelEvent) => {
    e.preventDefault();

    if (e.ctrlKey || e.metaKey) {
      // 缩放手势
      const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
      const newZoom = Math.min(Math.max(viewport.zoom * zoomFactor, 0.1), 5);

      setViewport(prev => ({
        ...prev,
        zoom: newZoom,
      }));
    } else {
      // 平移手势
      setViewport(prev => ({
        ...prev,
        x: prev.x - e.deltaX,
        y: prev.y - e.deltaY,
      }));
    }
  }, [viewport.zoom]);

  useEffect(() => {
    const element = viewportRef.current;
    if (element) {
      element.addEventListener('wheel', handleWheel, { passive: false });
      return () => element.removeEventListener('wheel', handleWheel);
    }
  }, [viewportRef, handleWheel]);

  return viewport;
};
```

### 3. 双指捏合缩放

```typescript
// 触摸设备双指缩放
const usePinchZoom = (viewportRef: RefObject<HTMLElement>) => {
  const [zoom, setZoom] = useState(1);
  const initialDistance = useRef<number>(0);
  const initialZoom = useRef<number>(1);

  const handleTouchStart = (e: TouchEvent) => {
    if (e.touches.length === 2) {
      initialDistance = getTouchDistance(e.touches);
      initialZoom = zoom;
    }
  };

  const handleTouchMove = (e: TouchEvent) => {
    if (e.touches.length === 2) {
      e.preventDefault();

      const currentDistance = getTouchDistance(e.touches);
      const scale = currentDistance / initialDistance;
      const newZoom = Math.min(Math.max(initialZoom * scale, 0.1), 5);

      setZoom(newZoom);
    }
  };

  return zoom;
};

const getTouchDistance = (touches: TouchList): number => {
  const dx = touches[0].clientX - touches[1].clientX;
  const dy = touches[0].clientY - touches[1].clientY;
  return Math.sqrt(dx * dx + dy * dy);
};
```

### 4. 连接手势

```typescript
// 拖拽创建连接
const useConnectionGesture = (sourceNodeId: string) => {
  const [isConnecting, setIsConnecting] = useState(false);
  const [tempLine, setTempLine] = useState<Line | null>(null);
  const [hoveredHandle, setHoveredHandle] = useState<string | null>(null);

  const handleHandlePointerDown = (e: React.PointerEvent, handleId: string) => {
    e.stopPropagation();
    setIsConnecting(true);

    const startPoint = { x: e.clientX, y: e.clientY };

    const handlePointerMove = (moveEvent: PointerEvent) => {
      setTempLine({
        startX: startPoint.x,
        startY: startPoint.y,
        endX: moveEvent.clientX,
        endY: moveEvent.clientY,
      });
    };

    const handlePointerUp = (upEvent: PointerEvent) => {
      if (hoveredHandle) {
        // 创建连接
        createEdge(sourceNodeId, hoveredHandle);
      }

      setIsConnecting(false);
      setTempLine(null);
      document.removeEventListener('pointermove', handlePointerMove);
      document.removeEventListener('pointerup', handlePointerUp);
    };

    document.addEventListener('pointermove', handlePointerMove);
    document.addEventListener('pointerup', handlePointerUp);
  };

  return {
    isConnecting,
    tempLine,
    handleHandlePointerDown,
    setHoveredHandle,
  };
};
```

### 5. 多选手势

```typescript
// 套索选择和框选
const useSelectionGesture = () => {
  const [selection, setSelection] = useState<Rect | null>(null);
  const [selectedNodes, setSelectedNodes] = useState<string[]>([]);
  const [isSelecting, setIsSelecting] = useState(false);

  const handleSelectionStart = (e: React.PointerEvent) => {
    // 只有点击空白区域才触发
    if (!isNodeElement(e.target)) {
      setIsSelecting(true);
      setSelection({
        x: e.clientX,
        y: e.clientY,
        width: 0,
        height: 0,
      });
    }
  };

  const handleSelectionMove = (e: React.PointerEvent) => {
    if (isSelecting && selection) {
      setSelection(prev => ({
        ...prev!,
        width: e.clientX - prev!.x,
        height: e.clientY - prev!.y,
      }));

      // 实时计算选中的节点
      const nodesInRect = findNodesInRect(selection);
      setSelectedNodes(nodesInRect);
    }
  };

  const handleSelectionEnd = () => {
    setIsSelecting(false);
    if (!selection || (selection.width === 0 && selection.height === 0)) {
      setSelection(null);
      setSelectedNodes([]);
    }
  };

  return {
    selection,
    selectedNodes,
    isSelecting,
    handleSelectionStart,
    handleSelectionMove,
    handleSelectionEnd,
  };
};
```

### 6. 手势反馈动画

```typescript
// 拖拽过程中的视觉反馈
const useDragFeedback = () => {
  const [feedback, setFeedback] = useState({
    opacity: 1,
    scale: 1,
    shadow: 'medium',
  });

  const handleDragStart = () => {
    setFeedback({
      opacity: 0.8,
      scale: 1.05,
      shadow: 'large',
    });
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    // 高亮放置区域
    highlightDropZone(e.dataTransfer.dropEffect);
  };

  const handleDragEnd = () => {
    setFeedback({
      opacity: 1,
      scale: 1,
      shadow: 'medium',
    });
    clearDropZoneHighlights();
  };

  return {
    feedbackStyles: {
      opacity: feedback.opacity,
      transform: `scale(${feedback.scale})`,
      boxShadow: getShadowValue(feedback.shadow),
    },
    handlers: {
      handleDragStart,
      handleDragOver,
      handleDragEnd,
    },
  };
};
```

## 手势交互最佳实践

| 手势 | 操作 | 视觉反馈 |
|------|------|----------|
| 单指拖拽 | 移动节点 | 半透明 + 投影 |
| 双指捏合 | 缩放视图 | 缩放进度指示 |
| 双击 | 展开/收起 | 缩放动画 |
| 长按 | 上下文菜单 | 波纹效果 |
| 滑动选择 | 多选 | 框选高亮 |
| 右键拖拽 | 创建连接 | 连线预览 |

## 与其他 Skills 配合

```
1. /sc:gesture-design "设计节点拖拽"
2. /sc:micro-interaction "添加拖拽反馈"
3. /sc:xyflow-advanced "实现连接手势"
```

---

**快捷命令**:

```
/sc:gesture "设计手势交互"    # 手势设计
/sc:touch "实现触控操作"       # 触控设计
```

---

> FlowSight 专用 - 自然手势交互设计

**参考资源**:
- [Modern UI Design Trends 2025](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEvXlEUgYJH2o4ouQvZXl0gLF5mWsQLWGvRppsg7p4BbGDX934TpVLfSVNk8eS2PMmWGlW_qrYqCJQvBb4JDD7gBxIthDvohcA-zLUX_KKUtq1EpAWoj3UL2kvcvTn3araumgiT1CU1V7YsSEetaYGvpJSw_UnILMztIqV5hehZ)
