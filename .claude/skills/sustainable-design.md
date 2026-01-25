# /sc:sustainable-design - 可持续设计

> FlowSight 可持续设计技能 - 节能环保 UI

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "可持续", "sustainable" | 可持续设计 |
| "节能", "energy efficient" | 节能设计 |
| "环保", "eco-friendly" | 环保界面 |
| "性能优化" | 性能优化 |

## 2025 前沿设计理念

可持续设计正在成为行业标准：

### 核心原则

1. **能源效率**
   - 减少设备电池消耗
   - 最小化数据传输
   - 优化渲染性能

2. **Lightning Dark**
   - 深色模式与动态光效结合
   - 沉浸且节能
   - OLED 友好设计

3. **精简设计**
   - 减少不必要的视觉元素
   - 优化加载流程
   - 延迟加载非关键资源

## 设计模式

### 1. 深色模式优化

```css
/* 深色模式 - 节能优先 */
@media (prefers-color-scheme: dark) {
  :root {
    /* 使用纯黑或深灰，减少 OLED 功耗 */
    --bg-primary: #000000;
    --bg-secondary: #121212;

    /* 避免高亮色，减少发光区域 */
    --accent-dim: rgba(59, 130, 246, 0.3);

    /* 减少动画以节省 CPU */
    --transition-reduced: 100ms ease-out;
  }

  /* 纯黑背景 - OLED 最省电 */
  .surface-elevated {
    background: #000000;
  }
}
```

### 2. 动态亮度调节

```typescript
// 根据环境光调节亮度
const useAdaptiveBrightness = () => {
  const [brightness, setBrightness] = useState(1);

  useEffect(() => {
    const handleLightChange = (e: AmbientLightEvent) => {
      // 环境光亮时降低亮度以减少功耗
      const normalizedBrightness = Math.min(1, 500 / e.value);
      setBrightness(normalizedBrightness);
    };

    if ('onambientlight' in window) {
      window.addEventListener('ambientlight', handleLightChange);
    }

    return () => window.removeEventListener('ambientlight', handleLightChange);
  }, []);

  return brightness;
};
```

### 3. 智能动画节流

```typescript
// 基于用户偏好和性能的动画控制
type AnimationPreference = 'full' | 'reduced' | 'none';

const useAnimationControl = () => {
  const preference = useReducedMotion()
    ? 'reduced'
    : 'full';

  const shouldAnimate = preference !== 'none';

  const animationConfig = {
    duration: preference === 'reduced' ? 100 : 200,
    reducedMotion: preference === 'reduced',
    enabled: shouldAnimate,
  };

  return animationConfig;
};
```

### 4. 按需渲染优化

```typescript
// 只渲染视口内的节点
const useLazyRender = <T extends { id: string }>(
  items: T[],
  containerRef: RefObject<HTMLElement>
) => {
  const [visibleIds, setVisibleIds] = useState<string[]>([]);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          const id = entry.target.getAttribute('data-id');
          if (entry.isIntersecting) {
            setVisibleIds(prev => [...new Set([...prev, id])]);
          }
        });
      },
      { root: containerRef.current, rootMargin: '200px' }
    );

    items.forEach(item => {
      const el = document.querySelector(`[data-id="${item.id}"]`);
      if (el) observer.observe(el);
    });

    return () => observer.disconnect();
  }, [items, containerRef]);

  return visibleIds;
};
```

### 5. 数据传输优化

```typescript
// 智能数据加载策略
class SmartDataLoader {
  private cache = new Map<string, DataPayload>();

  async loadData(key: string): Promise<DataPayload> {
    // 优先使用缓存
    if (this.cache.has(key)) {
      return this.cache.get(key)!;
    }

    // 延迟加载非关键数据
    if (!this.isCriticalData(key)) {
      this.loadInBackground(key);
      return this.getPlaceholder(key);
    }

    // 关键数据同步加载
    const data = await fetchData(key);
    this.cache.set(key, data);
    return data;
  }

  private loadInBackground(key: string): void {
    requestIdleCallback(() => {
      fetchData(key).then(data => {
        this.cache.set(key, data);
        this.notifyUpdate(key);
      });
    });
  }
}
```

## 可持续设计检查清单

| 领域 | 优化项 | 影响 |
|------|--------|------|
| 渲染 | 使用 CSS 替代 JS 动画 | 减少 CPU 使用 |
| 网络 | 压缩图片和资源 | 减少数据传输 |
| 缓存 | 智能缓存策略 | 减少重复请求 |
| 深色 | OLED 优化黑色 | 减少屏幕功耗 |
| 动画 | 基于偏好的动画 | 节省电池 |

## 与其他 Skills 配合

```
1. /sc:sustainable-design "优化能源效率"
2. /sc:dark-mode "实现深色模式"
3. /sc:xyflow-advanced "优化渲染性能"
```

---

**快捷命令**:

```
/sc:sustainable "设计节能界面"    # 可持续设计
/sc:eco "优化能源效率"            # 节能优化
```

---

> FlowSight 专用 - 可持续环保设计

**参考资源**:
- [Modern UI Design Trends 2025](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEvXlEUgYJH2o4ouQvZXl0gLF5mWsQLWGvRppsg7p4BbGDX934TpVLfSVNk8eS2PMmWGlW_qrYqCJQvBb4JDD7gBxIthDvohcA-zLUX_KKUtq1EpAWoj3UL2kvcvTn3araumgiT1CU1V7YsSEetaYGvpJSw_UnILMztIqV5hehZ)
