# FlowSight 可视化策略

> 文档版本: 1.0
> 最后更新: 2026-01-08

---

## 1. 问题背景

Linux 内核代码库规模巨大（2000万+ 行），执行流分析可能产生上万节点的调用图。原方案使用 React Flow + D3.js，在大规模图渲染时存在性能瓶颈。

### 1.1 性能瓶颈分析

| 节点规模 | React Flow 表现 | 用户体验 |
|---------|----------------|---------|
| < 500 | 流畅 | 优秀 |
| 500-1000 | 轻微卡顿 | 可接受 |
| 1000-5000 | 明显卡顿 | 较差 |
| > 5000 | 严重卡顿/崩溃 | 不可用 |

**根本原因**：React Flow 基于 SVG/DOM 渲染，每个节点都是 DOM 元素，大量节点导致：
- DOM 操作开销大
- 重绘/回流频繁
- 内存占用高

---

## 2. 解决方案

### 2.1 分层渲染策略（核心）

**核心思想**：不渲染完整图，按需展开

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         分层渲染策略                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Level 0: 入口函数                                                       │
│  ┌─────────┐                                                            │
│  │ my_init │  ← 默认只显示入口                                          │
│  └────┬────┘                                                            │
│       │ 点击展开                                                         │
│       ▼                                                                  │
│  Level 1: 直接调用                                                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                                 │
│  │ func_a  │  │ func_b  │  │ func_c  │  ← 展开第一层                   │
│  └────┬────┘  └─────────┘  └────┬────┘                                 │
│       │ 点击展开                 │ 点击展开                              │
│       ▼                         ▼                                        │
│  Level 2: 深层调用                                                       │
│  ┌─────────┐              ┌─────────┐                                   │
│  │ func_d  │              │ func_e  │  ← 按需展开                       │
│  └─────────┘              └─────────┘                                   │
│                                                                          │
│  优势：                                                                  │
│  • 初始渲染只有 1 个节点                                                │
│  • 用户控制展开深度                                                     │
│  • 永远不会一次性渲染上万节点                                           │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 深度限制参数

用户可配置最大展开深度：

```typescript
interface FlowViewConfig {
  maxDepth: number;        // 默认 5 层
  autoExpandThreshold: number;  // 子节点 < 此值时自动展开，默认 3
  collapseThreshold: number;    // 子节点 > 此值时默认折叠，默认 10
}
```

### 2.3 渲染引擎选型

| 规模 | 推荐方案 | 理由 |
|------|---------|------|
| 小图 (< 1000 节点) | React Flow | 交互体验好，开发效率高 |
| 中图 (1000-10000) | Cytoscape.js | 成熟稳定，支持 WebGL 扩展 |
| 大图 (> 10000) | Sigma.js | 专为大规模图设计，WebGL 渲染 |

**v1.0 决策**：
- 保留 React Flow 作为默认渲染器
- 配合分层渲染策略，实际渲染节点数控制在 1000 以内
- 如果用户���要查看完整图，提供 "导出到 Cytoscape" 功能

### 2.4 Cytoscape.js 集成方案

```typescript
// 大图渲染时切换到 Cytoscape.js
import cytoscape from 'cytoscape';
import cytoscapeWebgl from 'cytoscape-webgl';

cytoscape.use(cytoscapeWebgl);

const cy = cytoscape({
  container: document.getElementById('cy'),
  elements: graphData,
  style: [...],
  layout: {
    name: 'dagre',  // 层次布局
    rankDir: 'TB',
    nodeSep: 50,
    rankSep: 100,
  },
  // WebGL 渲染配置
  renderer: {
    name: 'webgl',
  },
});
```

---

## 3. 交互设计

### 3.1 节点展开/折叠

| 操作 | 行为 |
|------|------|
| 单击节点 | 跳转到代码位置 |
| 双击节点 | 展开/折叠子节点 |
| 右键节点 | 上下文菜单（展开全部、折叠全部、聚焦此节点） |
| Ctrl+点击 | 多选节点 |

### 3.2 智能折叠

自动识别并折叠以下模式��
- 递归调用（显示循环标记）
- 重复调用同一函数（合并显示调用次数）
- 标准库函数（默认折叠，可配置）

### 3.3 关键路径高亮

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         关键路径高亮                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  用户选择：从 my_probe 到 dma_alloc_coherent 的路径                     │
│                                                                          │
│  ┌─────────┐                                                            │
│  │my_probe │ ════════╗                                                  │
│  └────┬────┘         ║ 高亮路径                                         │
│       │              ║                                                   │
│  ┌────┴────┐  ┌─────────┐                                              │
│  │init_dev │  │ log_msg │ (灰显)                                        │
│  └────┬────┘  └─────────┘                                              │
│       │ ═════════════╗                                                  │
│  ┌────┴────┐         ║                                                  │
│  │alloc_buf│ ════════╝                                                  │
│  └────┬────┘                                                            │
│       │                                                                  │
│  ┌────┴────────────┐                                                    │
│  │dma_alloc_coherent│ ← 目标                                            │
│  └──────────────────┘                                                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 4. 导出功能

### 4.1 支持格式

| 格式 | 用途 |
|------|------|
| PNG/SVG | 文档、演示 |
| JSON | 导入其他工具 |
| DOT (Graphviz) | 命令行工具处理 |
| Cytoscape JSON | 在 Cytoscape Desktop 中打开 |

### 4.2 导出配置

```typescript
interface ExportConfig {
  format: 'png' | 'svg' | 'json' | 'dot' | 'cytoscape';
  includeCollapsed: boolean;  // 是否包含折叠的节点
  maxNodes: number;           // 最大导出节点数
  resolution: number;         // 图片分辨率 (DPI)
}
```

---

## 5. 性能优化清单

### 5.1 前端优化

- [x] 使用 `useMemo` 缓存节点和边数据
- [x] 自定义 `nodeTypes` 减少重渲染
- [ ] 虚拟化：只渲染视口内的节点
- [ ] Web Worker：后台计算布局
- [ ] 增量更新：只更新变化的节点

### 5.2 后端优化

- [ ] 分页返回调用图数据
- [ ] 预计算常用入口点的调用图
- [ ] 缓存已计算的子图

---

## 6. 实施计划

### Phase 1: 分层渲染（v1.0 必须）
- 实现节点展开/折叠逻辑
- 添加深度限制配置
- 智能折叠规则

### Phase 2: 性能优化（v1.0 可选）
- 虚拟化渲染
- Web Worker 布局计算

### Phase 3: 大图支持（v1.x）
- Cytoscape.js 集成
- 导出功能完善

---

## 7. 异步时序图（泳道图）

### 7.1 设计目标

展示用户空间、内核空间、硬件之间的异步交互关系，类似 UML 序列图：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      异步执行流时序图                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   用户空间          内核空间              硬件                           │
│      │                 │                   │                            │
│      │   open()        │                   │                            │
│      │────────────────>│                   │                            │
│      │                 │ probe()           │                            │
│      │                 │ request_irq()     │                            │
│      │<────────────────│                   │                            │
│      │                 │                   │                            │
│      │   read()        │                   │                            │
│      │────────────────>│                   │                            │
│      │                 │ wait_for_         │                            │
│      │                 │ completion()      │                            │
│      │     阻塞        │<──────────────────│                            │
│      │ ┄ ┄ ┄ ┄ ┄ ┄ ┄ ┄│                   │                            │
│      │                 │                   │  IRQ!                      │
│      │                 │<══════════════════│                            │
│      │                 │ irq_handler()     │                            │
│      │                 │ schedule_work     │                            │
│      │                 │        │          │                            │
│      │                 │        ▼          │                            │
│      │                 │ work_func()       │                            │
│      │                 │ complete()        │                            │
│      │     唤醒        │                   │                            │
│      │<┄ ┄ ┄ ┄ ┄ ┄ ┄ ┄│                   │                            │
│      │   返回数据      │                   │                            │
│      │<────────────────│                   │                            │
│                                                                          │
│   图例：                                                                 │
│   ────> 同步调用    ════> 硬件中断    ┄ ┄ > 异步唤醒                    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.2 泳道定义

| 泳道 | 执行上下文 | 特点 |
|------|-----------|------|
| 用户空间 | 用户进程 | 可阻塞、可被调度 |
| 内核空间（进程上下文） | 系统调用 | 可睡眠 |
| 内核空间（软中断） | softirq/tasklet | 不可睡眠 |
| 内核空间（硬中断） | IRQ handler | 不可睡眠、最快返回 |
| 硬件 | 外设 | 异步事件源 |

### 7.3 关键点提取（AI 辅助）

默认只显示关键节点，AI 帮助识别：

```yaml
# AI 识别的关键点类型
key_points:
  - type: entry_point
    description: "用户空间入口 (open/read/write/ioctl)"

  - type: blocking_point
    description: "阻塞等待点 (wait_for_completion, mutex_lock)"

  - type: async_trigger
    description: "异步触发点 (schedule_work, queue_work)"

  - type: interrupt_handler
    description: "中断处理入口"

  - type: wakeup_point
    description: "唤醒点 (complete, wake_up)"

  - type: context_switch
    description: "上下文切换 (进程→中断→进程)"
```

### 7.4 交互设计

**点击展开细节**：

```
┌─────────────────────────────────────────────────────────────────────────┐
│  点击 "probe()" 展开详细执行流                                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   probe() [点击前: 单个节点]                                            │
│      │                                                                   │
│      ▼                                                                   │
│   probe() [点击后: 展开子流程]                                          │
│      │                                                                   │
│      ├── devm_kzalloc()                                                 │
│      ├── platform_get_resource()                                        │
│      ├── devm_ioremap()                                                 │
│      ├── devm_request_irq()                                             │
│      │      └── request_threaded_irq()                                  │
│      │             └── __setup_irq()                                    │
│      └── device_create_file()                                           │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**悬停显示信息**：
- 函数签名
- 执行上下文（可睡眠/不可睡眠）
- 调用次数统计
- 源码位置

### 7.5 ftrace 风格文本视图

同时支持 ftrace 风格的缩进文本输出：

```
# FlowSight 执行流 - ftrace 风格
# 入口: my_driver_read()
# 架构: x86_64

 0)               |  my_driver_read() {
 0)               |    mutex_lock() {
 0)   0.123 us    |    }
 0)               |    wait_for_completion_interruptible() {
 0)               |      /* 阻塞等待，切换到其他进程 */
 0)   ==========> |      /* IRQ 触发 */
 0)               |      my_irq_handler() {
 0)               |        schedule_work() {
 0)   0.089 us    |        }
 0)   0.234 us    |      }
 0)   <========== |      /* 返回进程上下文 */
 0)               |      my_work_func() {
 0)               |        process_data() {
 0)   1.234 us    |        }
 0)               |        complete() {
 0)   0.056 us    |        }
 0)   2.345 us    |      }
 0)   5.678 us    |    }
 0)               |    copy_to_user() {
 0)   0.345 us    |    }
 0)               |    mutex_unlock() {
 0)   0.067 us    |    }
 0)   8.901 us    |  }
```

### 7.6 视图切换

| 视图 | 适用场景 | 特点 |
|------|---------|------|
| 时序图（泳道） | 理解异步交互 | 清晰展示上下文切换 |
| 调用图（树形） | 理解调用层次 | 展示函数嵌套关系 |
| ftrace 文本 | 内核开发者习惯 | 类似实际 trace 输出 |
| 时间线 | 性能分析 | 展示时间占比 |

### 7.7 实现方案

**前端组件**：
```typescript
// 使用 Mermaid.js 或自定义 Canvas 渲染
interface SequenceDiagram {
  lanes: Lane[];           // 泳道定义
  messages: Message[];     // 消息/调用
  blocks: Block[];         // 阻塞区域
  annotations: Annotation[]; // 注释
}

interface Lane {
  id: string;
  name: string;            // "用户空间" | "内核空间" | "硬件"
  context: ExecutionContext;
}

interface Message {
  from: string;            // 源泳道
  to: string;              // 目标泳道
  label: string;           // 函数名
  type: 'sync' | 'async' | 'interrupt' | 'wakeup';
  expandable: boolean;     // 是否可展开
  children?: FlowNode[];   // 展开后的子流程
}
```

**AI 关键点识别 Prompt**：
```
分析以下 Linux 内核驱动代码的执行流，识别关键交互点：

代码：[driver code]

请识别：
1. 用户空间入口函数
2. 阻塞等待点
3. 中断处理函数
4. 异步工作提交点
5. 唤醒/完成点

返回 JSON 格式的关键点列表。
```

---

## 附录：技术参考

- [React Flow 性能优化](https://reactflow.dev/docs/guides/performance/)
- [Cytoscape.js 文档](https://js.cytoscape.org/)
- [Sigma.js 大规模图渲染](https://www.sigmajs.org/)
- [dagre 层次布局算法](https://github.com/dagrejs/dagre)
- [Mermaid.js 序列图](https://mermaid.js.org/syntax/sequenceDiagram.html)
