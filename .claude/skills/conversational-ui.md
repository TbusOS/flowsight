# /sc:conversational-ui - 对话式界面设计

> FlowSight 对话式界面技能 - 语音与聊天交互

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "对话", "conversational" | 对话式 UI |
| "语音", "voice" | 语音界面 |
| "聊天", "chat" | 聊天界面 |
| "AI 助手", "AI assistant" | AI 助手 |

## 2025 前沿设计理念

对话式界面正在改变用户交互方式：

### 核心趋势

1. **语音界面普及**
   - 智能音箱和虚拟助手推动
   - 解放双手的操作方式
   - 适合复杂流程引导

2. **对话式 UI**
   - 自然语言交互
   - 渐进式信息展示
   - 上下文感知

## 设计模式

### 1. AI 助手面板

```tsx
const AIAssistantPanel: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isListening, setIsListening] = useState(false);

  const handleVoiceInput = async () => {
    setIsListening(true);
    const transcript = await voiceService.listen();

    const response = await aiService.query(transcript, {
      context: currentFlow,
      history: messages,
    });

    setMessages(prev => [
      ...prev,
      { role: 'user', content: transcript },
      { role: 'assistant', content: response },
    ]);

    setIsListening(false);
  };

  return (
    <div className="ai-assistant-panel">
      <ConversationHistory messages={messages} />

      <div className="input-area">
        {isListening && <VoiceWaveAnimation />}
        <TextInput
          onSubmit={sendMessage}
          placeholder="询问关于流程图的问题..."
        />
        <VoiceButton
          active={isListening}
          onClick={handleVoiceInput}
        />
      </div>
    </div>
  );
};
```

### 2. 自然语言流程控制

```typescript
// 自然语言命令解析
interface FlowCommand {
  action: 'create' | 'modify' | 'navigate' | 'analyze';
  target?: string;
  parameters?: Record<string, any>;
  confidence: number;
}

class CommandParser {
  parse(input: string): FlowCommand {
    const patterns = [
      {
        regex: /创建.*节点/i,
        action: 'create',
        extractTarget: (match) => match[0],
      },
      {
        regex: /导航到.*(函数|模块)/i,
        action: 'navigate',
        extractTarget: (match) => match[1],
      },
      {
        regex: /分析.*(性能|依赖)/i,
        action: 'analyze',
        extractTarget: (match) => match[1],
      },
    ];

    for (const pattern of patterns) {
      const match = input.match(pattern.regex);
      if (match) {
        return {
          action: pattern.action,
          target: pattern.extractTarget(match),
          confidence: 0.9,
        };
      }
    }

    return { action: 'analyze', confidence: 0.3 };
  }
}
```

### 3. 上下文感知建议

```typescript
// 基于当前上下文提供智能建议
const useContextualSuggestions = () => {
  const [suggestions, setSuggestions] = useState<string[]>([]);

  useEffect(() => {
    const context = {
      selectedNode: getSelectedNode(),
      currentView: getCurrentView(),
      recentActions: getRecentActions(),
      flowStructure: getFlowStructure(),
    };

    const generated = generateSuggestions(context);
    setSuggestions(generated);
  }, [selectedNodeId, currentView]);

  return suggestions;
};

const generateSuggestions = (context: FlowContext): string[] => {
  const suggestions = [];

  if (context.selectedNode) {
    suggestions.push(
      `查看 "${context.selectedNode.label}" 的详细分析`,
      `添加 "${context.selectedNode.label}" 的依赖节点`,
    );
  }

  if (context.flowStructure.complexity > 10) {
    suggestions.push(
      '流程复杂度较高，建议优化布局',
      '生成流程摘要报告',
    );
  }

  return suggestions;
};
```

### 4. 渐进式信息展示

```tsx
const ProgressiveInfoCard: React.FC<{ data: FlowNode }> = ({ data }) => {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className={`info-card ${expanded ? 'expanded' : ''}`}>
      {/* 基础信息 - 始终显示 */}
      <div className="basic-info">
        <NodeIcon type={data.type} />
        <span className="label">{data.label}</span>
        {data.status && <StatusBadge status={data.status} />}
      </div>

      {/* 展开后显示详情 */}
      {expanded && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: 'auto' }}
          className="detailed-info"
        >
          <CodePreview code={data.code} />
          <DependenciesList deps={data.dependencies} />
          <PerformanceMetrics metrics={data.metrics} />
        </motion.div>
      )}

      {/* 展开/收起控制 */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="expand-toggle"
      >
        {expanded ? '收起' : '详情'}
      </button>
    </div>
  );
};
```

### 5. 语音反馈系统

```typescript
// 提供语音反馈的辅助功能
class VoiceFeedbackSystem {
  private synth: SpeechSynthesis;

  announce(message: string, priority: 'low' | 'normal' | 'high' = 'normal') {
    // 低优先级消息在用户空闲时播放
    if (priority === 'low') {
      this.queueForIdle(message);
      return;
    }

    // 高优先级立即播放
    this.speak(message);
  }

  private speak(text: string) {
    const utterance = new SpeechSynthesisUtterance(text);
    utterance.rate = 1.2; // 稍快的语速
    utterance.pitch = 1;
    this.synth.speak(utterance);
  }

  // 操作确认反馈
  confirmAction(action: string, success: boolean) {
    const message = success
      ? `${action} 完成`
      : `${action} 失败，请重试`;

    this.announce(message, 'high');
  }
}
```

## 对话式 UI 最佳实践

| 实践 | 说明 |
|------|------|
| 渐进式披露 | 先展示概要，按需展开详情 |
| 语音反馈 | 重要操作提供语音确认 |
| 上下文记忆 | 记住对话历史和用户偏好 |
| 多模态输入 | 支持语音和文字切换 |
| 错误恢复 | 自然语言理解失败时的优雅降级 |

## 与其他 Skills 配合

```
1. /sc:conversational-ui "设计 AI 助手"
2. /sc:ai-ui "添加智能推荐"
3. /sc:a11y-design "增强无障碍支持"
```

---

**快捷命令**:

```
/sc:conversational "设计对话界面"    # 对话式 UI
/sc:voice "添加语音控制"              # 语音界面
```

---

> FlowSight 专用 - 对话式界面设计

**参考资源**:
- [Modern UI Design Trends 2025](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEvXlEUgYJH2o4ouQvZXl0gLF5mWsQLWGvRppsg7p4BbGDX934TpVLfSVNk8eS2PMmWGlW_qrYqCJQvBb4JDD7gBxIthDvohcA-zLUX_KKUtq1EpAWoj3UL2kvcvTn3araumgiT1CU1V7YsSEetaYGvpJSw_UnILMztIqV5hehZ)
