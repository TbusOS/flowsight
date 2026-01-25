---
name: a11y-design
description: "FlowSight Accessibility Skill - WCAG 2.2 compliant design"
category: frontend
complexity: standard
mcp-servers: []
personas: []
---

# /sc:a11y-design - Accessibility Design

## Triggers
- Accessibility and a11y requests
- Keyboard navigation implementation
- WCAG compliance requirements
- Screen reader support

## Usage
```
/sc:a11y-design "<task description>"
/sc:a11y "<task description>"
```

## 2025 Design Principles

### Core Principles

1. **Designing for Everyone**
   - Respectful and responsible design
   - WCAG 2.2 compliance
   - Inclusive design thinking

2. **Tool Support**
   - React Aria components
   - Headless UI components
   - Automated compliance checks

## Design Patterns

### 1. Keyboard Navigation
```tsx
const KeyboardNavigableFlow: React.FC = () => {
  const [focusedNode, setFocusedNode] = useState<string | null>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowUp': moveFocus('up'); break;
        case 'ArrowDown': moveFocus('down'); break;
        case 'ArrowLeft': moveFocus('left'); break;
        case 'ArrowRight': moveFocus('right'); break;
        case 'Enter': activateNode(focusedNode); break;
        case 'Escape': clearFocus(); break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [focusedNode]);

  return <FlowGraph onFocusChange={setFocusedNode} />;
};
```

### 2. Screen Reader Support
```tsx
const AccessibleNode: React.FC<NodeProps> = ({ id, label, status }) => {
  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={`Flow node: ${label}`}
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

// Live status announcement
const LiveRegion: React.FC<{ message: string }> = ({ message }) => {
  return (
    <div role="status" aria-live="polite" aria-atomic="true" className="sr-only">
      {message}
    </div>
  );
};
```

### 3. High Contrast Mode
```tsx
const useHighContrastMode = () => {
  const [isHighContrast, setIsHighContrast] = useState(false);

  useEffect(() => {
    const mediaQuery = window.matchMedia('(forced-colors: active)');
    setIsHighContrast(mediaQuery.matches);

    const handler = (e: MediaQueryListEvent) => setIsHighContrast(e.matches);
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  return isHighContrast;
};

const highContrastTheme = {
  colors: {
    nodeDefault: 'Canvas',
    nodeHover: 'Highlight',
    border: 'CanvasText',
    text: 'CanvasText',
    connection: 'LinkText',
  },
  borderWidth: '2px',
  borderStyle: 'solid',
};
```

### 4. Focus Management
```tsx
const useFocusManagement = (containerRef: RefObject<HTMLElement>) => {
  const focusHistory = useRef<HTMLElement[]>([]);

  const trapFocus = () => {
    const focusableElements = containerRef.current?.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    if (focusableElements?.length) {
      const first = focusableElements[0] as HTMLElement;
      first.focus();
      focusHistory.current.push(first);
    }
  };

  const restoreFocus = () => {
    const last = focusHistory.current.pop();
    last?.focus();
  };

  return { trapFocus, restoreFocus };
};
```

## WCAG 2.2 Checklist

| Principle | Requirement | Check |
|-----------|-------------|-------|
| Perceivable | Alternatives | Images have alt text |
| Operable | Keyboard | All features keyboard accessible |
| Understandable | Predictability | Consistent behavior |
| Robust | Compatibility | Works with assistive tech |

## Testing Tools

```bash
# Automated accessibility testing
npm install -D @axe-core/react
npx axe .

# Lighthouse audit
# Chrome DevTools > Lighthouse > Accessibility
```

## Tool Coordination
- **Bash**: Run tests and audits

## Examples

### Design Accessible Interface
```
/sc:a11y-design "设计无障碍界面"
```

### Add Keyboard Navigation
```
/sc:a11y "添加键盘导航"
```

### Implement ARIA Labels
```
/sc:a11y-design "实现ARIA标签"
```

## Boundaries

**Will:**
- Implement full keyboard navigation
- Add proper ARIA labels and roles
- Support high contrast and reduced motion

**Will Not:**
- Compromise functionality for accessibility
- Ignore assistive technology requirements

## See Also
- `/sc:ui-design` - General UI design
- `/sc:micro-interaction` - Focus animations
- `/sc:build` - Build verification
