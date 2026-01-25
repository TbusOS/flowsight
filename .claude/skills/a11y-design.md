# /sc:a11y-design - 无障碍设计优先

> FlowSight 无障碍设计技能 - WCAG 2.2 合规设计

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "无障碍", "a11y" | 无障碍设计 |
| " accessibility" | Accessibility |
| "WCAG" | WCAG 合规 |
| "键盘导航" | 键盘操作 |

## 2025 前沿设计理念

无障碍设计正在成为行业标准：

### 核心原则

1. **为所有人设计 (Designing for Everyone)**
   - 负责任地尊重地设计界面
   - WCAG 2.2 合规
   - 包容性设计思维

2. **工具支持**
   - React Aria 组件库
   - Headless UI 组件
   - 自动化合规检查

## 设计模式

### 1. 键盘导航支持

```typescript
// 全键盘导航支持
const KeyboardNavigableFlow: React.FC = () => {
  const [focusedNode, setFocusedNode] = useState<string | null>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowUp':
          moveFocus('up');
          break;
        case 'ArrowDown':
          moveFocus('down');
          break;
        case 'ArrowLeft':
          moveFocus('left');
          break;
        case 'ArrowRight':
          moveFocus('right');
          break;
        case 'Enter':
          activateNode(focusedNode);
          break;
        case 'Escape':
          clearFocus();
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [focusedNode]);

  return <FlowGraph onFocusChange={setFocusedNode} />;
};
```

### 2. 屏幕阅读器支持

```tsx
// ARIA 标签和描述
const AccessibleNode: React.FC<NodeProps> = ({ id, label, status }) => {
  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={`流程节点: ${label}`}
      aria-describedby={`node-${id}-description`}
      aria-expanded={status === 'expanded'}
      aria-busy={status === 'loading'}
    >
      <span id={`node-${id}-description`} className="sr-only">
        {getNodeDescription(status)}
      </span>
      <Visuals label={label} status={status} />
    </div>
  );
};

// 实时状态播报
const LiveRegion: React.FC<{ message: string }> = ({ message }) => {
  return (
    <div role="status" aria-live="polite" aria-atomic="true" className="sr-only">
      {message}
    </div>
  );
};
```

### 3. 高对比度模式

```typescript
// 自动检测并适配高对比度
const useHighContrastMode = () => {
  const [isHighContrast, setIsHighContrast] = useState(false);

  useEffect(() => {
    // 检测系统高对比度设置
    const mediaQuery = window.matchMedia('(forced-colors: active)');
    setIsHighContrast(mediaQuery.matches);

    const handler = (e: MediaQueryListEvent) => {
      setIsHighContrast(e.matches);
    };

    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  return isHighContrast;
};

// 高对比度主题
const highContrastTheme = {
  colors: {
    nodeDefault: 'Canvas',
    nodeHover: 'Highlight',
    nodeSelected: 'Highlight',
    border: 'CanvasText',
    text: 'CanvasText',
    connection: 'LinkText',
  },
  borderWidth: '2px',
  borderStyle: 'solid',
};
```

### 4. 焦点管理

```typescript
// 焦点陷阱和恢复
const useFocusManagement = (containerRef: RefObject<HTMLElement>) => {
  const focusHistory = useRef<HTMLElement[]>([]);

  const trapFocus = () => {
    const focusableElements = containerRef.current?.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );

    if (focusableElements && focusableElements.length > 0) {
      const firstElement = focusableElements[0] as HTMLElement;
      const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;

      firstElement.focus();
      focusHistory.current.push(firstElement);
    }
  };

  const restoreFocus = () => {
    const lastFocused = focusHistory.current.pop();
    lastFocused?.focus();
  };

  return { trapFocus, restoreFocus };
};
```

## WCAG 2.2 检查清单

| 原则 | 要求 | 检查项 |
|------|------|--------|
| 可感知 | 替代文本 | 图片有 alt 描述 |
| 可操作 | 键盘导航 | 所有功能可键盘访问 |
| 可理解 | 可预测性 | 界面行为一致 |
| 健壮性 | 兼容性 | 与辅助技术兼容 |

## 测试工具

```bash
# 自动化无障碍测试
npm install -D @axe-core/react

# 运行测试
npx axe . --all

# Lighthouse 审计
# Chrome DevTools > Lighthouse > Accessibility
```

## 与其他 Skills 配合

```
1. /sc:a11y-design "添加键盘导航"
2. /sc:micro-interaction "优化焦点动画"
3. /sc:test "运行无障碍测试"
```

---

**快捷命令**:

```
/sc:a11y "设计无障碍界面"    # 无障碍设计
/sc:a11y "添加键盘导航"       # 键盘支持
```

---

> FlowSight 专用 - WCAG 2.2 无障碍设计

**参考资源**:
- [Modern UI Design Trends 2025](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEvXlEUgYJH2o4ouQvZXl0gLF5mWsQLWGvRppsg7p4BbGDX934TpVLfSVNk8eS2PMmWGlW_qrYqCJQvBb4JDD7gBxIthDvohcA-zLUX_KKUtq1EpAWoj3UL2kvcvTn3araumgiT1CU1V7YsSEetaYGvpJSw_UnILMztIqV5hehZ)
