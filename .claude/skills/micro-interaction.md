# /sc:micro-interaction - 微交互设计

> FlowSight 微交互设计技能 - 细致入微的用户体验

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "微交互", "micro-interaction" | 微交互设计 |
| "动画", "animation" | 动画效果 |
| "动效", "motion" | 动效设计 |
| "反馈", "feedback" | 用户反馈 |

## 2025 前沿设计理念

微交互是提升用户体验的关键细节：

### 核心原则

1. **交互对象设计 (Interactive Objects)**
   - 从静态按钮到复杂 3D 模型
   - 富有吸引力的交互元素
   - 自然的交互感受

2. **深思熟虑的微交互**
   - 动画进度指示器
   - 悬停效果
   - 状态切换过渡

3. **目的性动画**
   - 引导用户
   - 提升可用性
   - 提供即时反馈

## 设计模式

### 1. 节点悬停效果

```typescript
const useNodeMicrointeraction = (nodeId: string) => {
  const [isHovered, setIsHovered] = useState(false);
  const [ripples, setRipples] = useState<Ripple[]>([]);

  const handleMouseEnter = (e: React.MouseEvent) => {
    setIsHovered(true);
    // 添加涟漪效果
    const ripple = createRipple(e, nodeId);
    setRipples(prev => [...prev, ripple]);
  };

  const handleMouseLeave = () => {
    setIsHovered(false);
    // 延迟清除涟漪
    setTimeout(() => setRipples([]), 300);
  };

  return {
    isHovered,
    ripples,
    handleMouseEnter,
    handleMouseLeave,
  };
};
```

### 2. 连接建立动画

```css
/* 连接线动画 */
@keyframes connection-pulse {
  0% {
    stroke-dashoffset: 1000;
    opacity: 0;
  }
  50% {
    opacity: 1;
  }
  100% {
    stroke-dashoffset: 0;
    opacity: 0;
  }
}

.edge-connecting {
  stroke-dasharray: 1000;
  animation: connection-pulse 0.6s ease-out forwards;
}
```

### 3. 状态切换过渡

```tsx
const NodeStateTransition: React.FC<{ active: boolean }> = ({ active }) => {
  return (
    <motion.div
      className={`node ${active ? 'active' : 'inactive'}`}
      initial={false}
      animate={{
        scale: active ? 1.05 : 1,
        boxShadow: active
          ? '0 0 20px rgba(59, 130, 246, 0.5)'
          : '0 2px 4px rgba(0, 0, 0, 0.1)',
      }}
      transition={{
        type: 'spring',
        stiffness: 300,
        damping: 20,
      }}
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

### 4. 手势反馈

```typescript
// 拖拽过程中的实时反馈
const useDragFeedback = () => {
  const [feedback, setFeedback] = useState<DragFeedback>({
    opacity: 1,
    scale: 1,
    shadow: 'medium',
  });

  const handleDragStart = () => {
    setFeedback({
      opacity: 0.8,
      scale: 1.1,
      shadow: 'large',
    });
  };

  const handleDragEnd = () => {
    setFeedback({
      opacity: 1,
      scale: 1,
      shadow: 'medium',
    });
  };

  return { feedback, handleDragStart, handleDragEnd };
};
```

## 动画原则

### 动画时序

| 交互类型 | 持续时间 | 缓动函数 |
|----------|----------|----------|
| 悬停效果 | 150-200ms | ease-out |
| 点击反馈 | 100-150ms | ease-in |
| 状态切换 | 200-300ms | spring |
| 页面过渡 | 300-500ms | cubic-bezier |

### 性能优化

```typescript
// 使用 transform 和 opacity 进行动画
const OptimizedAnimation: React.FC = () => {
  return (
    <motion.div
      className="animated-element"
      animate={{
        // 仅使用 transform 和 opacity
        x: 100,
        opacity: 0.8,
      }}
      // 启用 GPU 加速
      style={{ willChange: 'transform, opacity' }}
    />
  );
};
```

## 微交互类型

| 场景 | 微交互效果 | 目的 |
|------|------------|------|
| 节点悬停 | 轻微放大 + 阴影增强 | 提示可交互 |
| 连接成功 | 绿色脉冲 + 勾号动画 | 确认反馈 |
| 拖拽中 | 半透明 + 投影 | 状态指示 |
| 加载中 | 骨架屏 + 脉冲 | 进度感知 |
| 错误状态 | 红色边框 + 震动 | 警示提醒 |

## 与其他 Skills 配合

```
1. /sc:micro-interaction "设计节点微交互"
2. /sc:immersive-visualization "添加动态效果"
3. /sc:build "构建验证"
```

---

**快捷命令**:

```
/sc:micro "设计微交互"       # 微交互设计
/sc:micro "添加交互动效"      # 动效设计
```

---

> FlowSight 专用 - 细致入微的用户体验设计

**参考资源**:
- [Modern UI Design Trends 2025](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEvXlEUgYJH2o4ouQvZXl0gLF5mWsQLWGvRppsg7p4BbGDX934TpVLfSVNk8eS2PMmWGlW_qrYqCJQvBb4JDD7gBxIthDvohcA-zLUX_KKUtq1EpAWoj3UL2kvcvTn3araumgiT1CU1V7YsSEetaYGvpJSw_UnILMztIqV5hehZ)
