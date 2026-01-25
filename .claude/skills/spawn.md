# /sc:spawn - 任务编排

> FlowSight 任务编排技能

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "编排", "spawn" | 任务编排 |
| "分解", "breakdown" | 任务分解 |
| "分配", "assign" | 任务分配 |

## 使用方式

```
/sc:spawn "分解大型任务"
/sc:spawn "分配子任务"
/sc:spawn "编排开发计划"
```

## 任务分解

### 分解原则

- 每个任务应可在 1-2 小时内完成
- 任务间依赖关系清晰
- 任务产出可验证

### 分解方法

1. **按功能分解** - 按功能模块划分
2. **按层级分解** - 从粗到细逐步分解
3. **按依赖分解** - 从无依赖到有依赖

## 任务编排

### 并行执行

- 无依赖的任务可并行执行
- 利用多核优势
- 提高开发效率

### 串行执行

- 有依赖关系的任务
- 需要顺序执行的任务
- 集成验证任务

### 编排示例

```
任务 A (无依赖) ─────┬──→ 并行执行 ──→ 集成任务 E ──→ 完成
任务 B (依赖 A) ─────┘
任务 C (无依赖) ─────┬──→ 并行执行
任务 D (依赖 B) ─────┘
```

## 任务分配

### 分配策略

| 任务类型 | 分配给 | 说明 |
|----------|--------|------|
| 后端开发 | Core-Agent | Rust 后端开发 |
| 前端开发 | UI-Agent | React 前端开发 |
| 测试任务 | Test-Agent | 测试验证 |

## 输出格式

### 任务清单

```yaml
tasks:
  - id: 1
    name: 任务名称
    description: 详细描述
    assignee: agent-name
    depends_on: []
    estimated_time: 2h

  - id: 2
    name: 任务名称
    description: 详细描述
    assignee: agent-name
    depends_on: [1]
    estimated_time: 1h

  - id: 3
    name: 集成测试
    description: 集成验证
    assignee: test-agent
    depends_on: [1, 2]
    estimated_time: 1h
```

## 与其他 Skills 配合

```
1. /sc:workflow "生成工作流"
2. /sc:spawn "分解任务"
3. /sc:implement "执行实现"
```

---

**快捷命令**:

```
/sc:spawn "分解任务"    # 任务编排
```

---

> FlowSight 专用 - 任务编排
