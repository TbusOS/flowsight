---
name: collaborative-dev
description: "Multi-agent collaborative development for large features"
category: collaboration
complexity: advanced
mcp-servers: []
personas: []
---

# /sc:collaborative-dev - Multi-Agent Collaborative Development

## Triggers
- Large feature development requiring multiple agents
- Parallel backend and frontend development
- Complex tasks requiring task decomposition
- Coordinated development across multiple domains

## Usage
```
/sc:collaborative-dev "实现节点详情面板"
/sc:collaborative-dev "添加场景对比功能" --backend --frontend --tests
```

## Coordination Pattern

```typescript
const CollaborativeDev = {
  agents: {
    backend: {
      focus: 'Rust/Tauri backend logic',
      files: ['crates/**/*', 'app/src-tauri/**/*'],
    },
    frontend: {
      focus: 'React/TypeScript UI components',
      files: ['app/src/**/*'],
    },
    tests: {
      focus: 'Unit and integration tests',
      files: ['**/*.test.ts', '**/tests/**/*'],
    },
  },
  sync: {
    api: true,
    types: true,
    schema: true,
  },
}
```

## Workflow

### Phase 1: Task Decomposition
1. Analyze the feature requirements
2. Identify independent work packages
3. Define interfaces and contracts
4. Create sync checkpoints

### Phase 2: Parallel Execution
- Agent 1: Backend implementation
- Agent 2: Frontend implementation
- Agent 3: Tests and validation

### Phase 3: Integration
1. Merge API changes
2. Connect frontend to backend
3. Run full test suite
4. Verify build

## Examples

### Implement Node Detail Panel (3 agents)
```
/sc:collaborative-dev "实现节点详情面板"
  --backend: "实现 Rust API 和数据处理"
  --frontend: "创建 React 组件和状态管理"
  --tests: "编写单元测试和集成测试"
```

### Add Scenario Comparison Feature
```
/sc:collaborative-dev "添加场景对比功能"
  --backend: "实现场景数据聚合 API"
  --frontend: "创建对比视图 UI"
  --tests: "测试对比逻辑"
```

## Tool Coordination
- **Task**: Create subtasks for each agent
- **Read**: Understand existing code structure
- **Write**: Implement features in parallel
- **Bash**: Build and test verification
- **Git**: Coordinate changes

## Sync Points

### Must Sync
- API contract changes
- Type definitions
- Data schemas
- Configuration

### Can Parallelize
- UI component implementation
- Backend logic
- Test cases
- Documentation

## Boundaries

**Will:**
- Decompose complex tasks effectively
- Coordinate parallel development
- Ensure interface consistency
- Validate integration

**Will Not:**
- Skip interface definition
- Create conflicting changes
- Ignore test coverage
- Skip integration validation

## See Also
- `/sc:spawn` - Task orchestration
- `/sc:design` - Architecture design
- `/sc:implement` - Single-agent implementation
- `/sc:test` - Test execution
