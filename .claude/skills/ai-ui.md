# /sc:ai-ui - AI 驱动自适应 UI 设计

> FlowSight AI 驱动界面设计技能 - 个性化智能界面

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "AI", "人工智能" | AI 集成 |
| "个性化", "personalized" | 个性化界面 |
| "自适应", "adaptive" | 自适应 UI |
| "智能", "smart" | 智能推荐 |

## 2025 前沿设计理念

AI 和机器学习正在推动高度自适应界面的发展：

### 核心概念

1. **超个性化界面 (Hyper-Personalized Interfaces)**
   - 实时分析用户行为和偏好
   - 个性化仪表盘和主题
   - 动态更新内容
   - 预测用户期望

2. **AI 驱动的设计工具**
   - 自动化设计改进建议
   - 智能布局生成
   - 交互模式优化

## 设计模式

### 1. 用户行为追踪与适应

```typescript
interface UserBehavior {
  frequentlyAccessedNodes: string[];
  preferredLayout: 'horizontal' | 'vertical';
  zoomLevel: number;
  theme: 'dark' | 'light';
}

class AdaptiveUI {
  private behavior: UserBehavior;

  adaptToUser(): void {
    // 根据使用模式自动调整布局
    if (this.behavior.preferredLayout === 'horizontal') {
      this.setLayout('horizontal');
    }

    // 智能缩放级别
    this.setZoom(this.behavior.zoomLevel);

    // 自动主题切换
    this.applyTheme(this.behavior.theme);
  }

  // 学习用户偏好
  learnFromInteraction(interaction: InteractionEvent): void {
    this.behavior = updateBehavior(this.behavior, interaction);
  }
}
```

### 2. 智能节点推荐

```tsx
const SmartNodePanel: React.FC = () => {
  const [recommendations, setRecommendations] = useState<NodeType[]>([]);

  useEffect(() => {
    // 基于当前上下文推荐相关节点
    const context = getFlowContext();
    const recommended = aiService.suggestNodes(context);
    setRecommendations(recommended);
  }, []);

  return (
    <div className="smart-panel">
      <h3>智能推荐</h3>
      {recommendations.map(node => (
        <SmartNodeCard
          node={node}
          onClick={() => addNodeToFlow(node)}
          confidence={node.relevanceScore}
        />
      ))}
    </div>
  );
};
```

### 3. 动态内容适应

```typescript
// 根据用户角色动态调整界面
const getAdaptiveContent = (user: User): AdaptiveContent => {
  const roleBasedConfig = {
    developer: {
      showDetailedMetrics: true,
      expandedNodeDetails: true,
      showInternals: true,
    },
    analyst: {
      showDetailedMetrics: false,
      expandedNodeDetails: false,
      showInternals: false,
    },
    manager: {
      showDetailedMetrics: true,
      expandedNodeDetails: false,
      showInternals: false,
    },
  };

  return roleBasedConfig[user.role] || roleBasedConfig.developer;
};
```

## 实施策略

### 数据收集层

```typescript
interface AnalyticsEvent {
  type: 'click' | 'zoom' | 'drag' | 'search';
  target: string;
  timestamp: number;
  duration?: number;
}

class BehaviorTracker {
  private events: AnalyticsEvent[] = [];

  track(event: AnalyticsEvent): void {
    this.events.push(event);
    // 本地聚合分析
    this.analyzePatterns();
  }

  private analyzePatterns(): void {
    // 分析用户模式
    const patterns = aggregatePatterns(this.events);
    this.updateUserProfile(patterns);
  }
}
```

### 智能决策引擎

```typescript
interface UIRecommendation {
  target: string;
  action: 'highlight' | 'suggest' | 'auto_apply';
  reason: string;
  confidence: number;
}

class RecommendationEngine {
  generateRecommendations(profile: UserProfile): UIRecommendation[] {
    return [
      // 基于历史的高频节点推荐
      {
        target: 'frequently_used_node',
        action: 'highlight',
        reason: '您经常使用此节点类型',
        confidence: 0.85,
      },
      // 基于当前上下文的建议
      {
        target: 'related_node',
        action: 'suggest',
        reason: '与当前节点相关',
        confidence: 0.72,
      },
    ];
  }
}
```

## 用户隐私保护

```typescript
// 隐私优先的设计
class PrivacyAwareAnalytics {
  // 本地处理，不上传敏感数据
  private localProcessing = true;

  // 用户可随时禁用追踪
  private userConsent: boolean = false;

  anonymizeData(event: AnalyticsEvent): AnonymizedEvent {
    return {
      type: event.type,
      // 移除个人标识符
      target: hashIdentifier(event.target),
      timestamp: event.timestamp,
    };
  }
}
```

## 与其他 Skills 配合

```
1. /sc:ai-ui "设计智能推荐系统"
2. /sc:immersive-visualization "添加个性化视觉效果"
3. /sc:build "构建验证"
```

---

**快捷命令**:

```
/sc:ai-ui "设计个性化界面"    # AI 驱动 UI
/sc:ai-ui "添加智能推荐"       # 智能推荐
```

---

> FlowSight 专用 - AI 驱动自适应界面设计

**参考资源**:
- [Modern UI Design Trends 2025](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEvXlEUgYJH2o4ouQvZXl0gLF5mWsQLWGvRppsg7p4BbGDX934TpVLfSVNk8eS2PMmWGlW_qrYqCJQvBb4JDD7gBxIthDvohcA-zLUX_KKUtq1EpAWoj3UL2kvcvTn3araumgiT1CU1V7YsSEetaYGvpJSw_UnILMztIqV5hehZ)
