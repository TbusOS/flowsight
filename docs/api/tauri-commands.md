# Tauri Commands API Reference

> FlowSight v0.2.0 Tauri 命令 API 文档

## 项目管理

### `open_directory`

打开项目目录并扫描文件。

```typescript
invoke('open_directory', { path: string }): Promise<ProjectInfo>
```

**参数:**
| 名称 | 类型 | 描述 |
|------|------|------|
| path | string | 项目目录绝对路径 |

**返回值:**
```typescript
interface ProjectInfo {
  path: string;
  fileCount: number;
  functionCount: number;
  structCount: number;
}
```

### `get_file_tree`

获取项目文件树。

```typescript
invoke('get_file_tree', { path: string }): Promise<FileNode[]>
```

**返回值:**
```typescript
interface FileNode {
  name: string;
  path: string;
  isDir: boolean;
  children?: FileNode[];
}
```

## 代码分析

### `get_functions`

获取文件中的函数列表。

```typescript
invoke('get_functions', { filePath: string }): Promise<FunctionInfo[]>
```

**参数:**
| 名称 | 类型 | 描述 |
|------|------|------|
| filePath | string | 文件绝对路径 |

**返回值:**
```typescript
interface FunctionInfo {
  name: string;
  returnType: string;
  parameters: Parameter[];
  startLine: number;
  endLine: number;
  isAsync: boolean;
  callbackType?: string;
}

interface Parameter {
  name: string;
  type: string;
}
```

### `build_execution_flow`

构建函数执行流。

```typescript
invoke('build_execution_flow', { 
  filePath: string, 
  entryFunction: string,
  maxDepth?: number,
  expandAsync?: boolean 
}): Promise<ExecutionFlow>
```

**参数:**
| 名称 | 类型 | 必须 | 描述 |
|------|------|------|------|
| filePath | string | 是 | 文件路径 |
| entryFunction | string | 是 | 入口函数名 |
| maxDepth | number | 否 | 最大深度 (默认: 10) |
| expandAsync | boolean | 否 | 展开异步调用 (默认: true) |

**返回值:**
```typescript
interface ExecutionFlow {
  entryPoint: FlowNode;
  nodes: FlowNode[];
  edges: FlowEdge[];
  asyncMechanisms: AsyncMechanism[];
}

interface FlowNode {
  id: string;
  name: string;
  file: string;
  line: number;
  nodeType: 'entry' | 'call' | 'callback' | 'async' | 'return';
  confidence: 'certain' | 'possible' | 'unknown';
  asyncType?: string;
  children?: FlowNode[];
}

interface FlowEdge {
  source: string;
  target: string;
  edgeType: 'sync' | 'async' | 'callback';
  label?: string;
}
```

### `get_node_detail`

获取执行流节点详情。

```typescript
invoke('get_node_detail', { 
  filePath: string, 
  functionName: string 
}): Promise<NodeDetail>
```

**返回值:**
```typescript
interface NodeDetail {
  function: FunctionInfo;
  callers: CallSite[];
  callees: CallSite[];
  asyncBindings: AsyncBinding[];
  sourceCode: string;
}

interface CallSite {
  function: string;
  file: string;
  line: number;
}

interface AsyncBinding {
  mechanism: string;
  handler: string;
  triggerSite?: string;
}
```

### `get_entry_points`

获取文件中的入口点函数（回调、模块初始化/退出等）。

```typescript
invoke('get_entry_points', { 
  file_path: string 
}): Promise<EntryPointInfo[]>
```

**参数:**
| 名称 | 类型 | 描述 |
|------|------|------|
| file_path | string | 文件绝对路径 |

**返回值:**
```typescript
interface EntryPointInfo {
  name: string;      // 函数名
  kind: string;      // 类型: 'callback' | 'module_init' | 'module_exit' | 'function'
  line: number;      // 行号
}
```

**示例:**
```typescript
const entryPoints = await invoke('get_entry_points', { 
  file_path: '/path/to/driver.c' 
});
// [
//   { name: 'probe', kind: 'callback', line: 42 },
//   { name: 'module_init', kind: 'module_init', line: 100 }
// ]
```

## 格式化导出

### `format_execution_flow`

格式化执行流为指定格式。

```typescript
invoke('format_execution_flow', {
  filePath: string,
  entryFunction: string,
  options: FormatOptions
}): Promise<FormattedFlow>
```

**参数:**
| 名称 | 类型 | 必须 | 描述 |
|------|------|------|------|
| filePath | string | 是 | 文件路径 |
| entryFunction | string | 是 | 入口函数名 |
| options | FormatOptions | 是 | 格式化选项 |

**格式选项:**
```typescript
interface FormatOptions {
  format: 'mermaid' | 'markdown' | 'ascii' | 'json';
  include_kernel_internal?: boolean;  // 包含内核内部调用
  max_depth?: number;                 // 最大深度
}
```

**返回值:**
```typescript
interface FormattedFlow {
  format: string;        // 使用的格式
  content: string;       // 格式化内容
  entry_function: string; // 入口函数
  summary: string;       // 摘要
}
```

**示例:**
```typescript
const result = await invoke('format_execution_flow', {
  filePath: '/path/to/driver.c',
  entryFunction: 'probe',
  options: { format: 'mermaid' }
});
console.log(result.content);
// flowchart TD
//   probe["probe()"] --> child1["init_device()"]
//   ...
```

### `get_flow_display_data`

获取执行流的展示数据（用于前端 UI 渲染）。

```typescript
invoke('get_flow_display_data', {
  filePath: string,
  entryFunction: string
}): Promise<DisplayFlowData>
```

**参数:**
| 名称 | 类型 | 描述 |
|------|------|------|
| filePath | string | 文件路径 |
| entryFunction | string | 入口函数名 |

**返回值:**
```typescript
interface DisplayFlowData {
  entry_function: string;
  summary: string;
  mermaid_diagram: string;
  nodes: DisplayNode[];
  async_patterns: AsyncPattern[];
  stats: FlowStats;
}

interface DisplayNode {
  id: string;
  name: string;
  display_name: string;
  node_type: 'entry' | 'function' | 'async' | 'callback';
  context: string | null;    // 执行上下文: 'process' | 'softirq' | 'hardirq'
  can_sleep: boolean | null;
  description: string | null;
  depth: number;
  children_count: number;
}

interface AsyncPattern {
  mechanism: string;   // 'WorkQueue' | 'Timer' | 'Tasklet' | 'IRQ'
  trigger: string;     // 触发函数
  handler: string;     // 处理函数
  description: string;
}

interface FlowStats {
  total_nodes: number;
  direct_calls: number;
  indirect_calls: number;
  async_calls: number;
}
```

**示例:**
```typescript
const displayData = await invoke('get_flow_display_data', {
  filePath: '/path/to/driver.c',
  entryFunction: 'probe'
});

console.log(`总节点: ${displayData.stats.total_nodes}`);
console.log(`异步模式: ${displayData.async_patterns.length}`);

// 渲染 Mermaid 图表
renderMermaid(displayData.mermaid_diagram);
```

## LLVM IR

### `generate_llvm_ir`

生成函数的 LLVM IR 表示。

```typescript
invoke('generate_llvm_ir', {
  filePath: string,
  functionName: string,
  optimizationLevel?: 0 | 1 | 2 | 3
}): Promise<LLVMResult>
```

**返回值:**
```typescript
interface LLVMResult {
  ir: string;
  warnings: string[];
  success: boolean;
}
```

## 文件操作

### `read_file`

读取文件内容。

```typescript
invoke('read_file', { path: string }): Promise<string>
```

### `save_file`

保存文件内容。

```typescript
invoke('save_file', { 
  path: string, 
  content: string 
}): Promise<void>
```

## 事件

### `analysis_progress`

分析进度事件。

```typescript
listen('analysis_progress', (event) => {
  const { phase, progress, message } = event.payload;
  // phase: 'scanning' | 'parsing' | 'analyzing' | 'done'
  // progress: 0-100
});
```

### `index_progress`

索引进度事件。

```typescript
listen('index_progress', (event) => {
  const { current, total, file } = event.payload;
});
```

## 错误处理

所有命令在失败时会抛出错误：

```typescript
interface TauriError {
  code: string;
  message: string;
  details?: any;
}
```

**常见错误码:**
| 错误码 | 描述 |
|--------|------|
| `FILE_NOT_FOUND` | 文件不存在 |
| `PARSE_ERROR` | 解析错误 |
| `ANALYSIS_ERROR` | 分析错误 |
| `PERMISSION_DENIED` | 权限不足 |

## 使用示例

### 完整分析流程

```typescript
import { invoke, listen } from '@tauri-apps/api';

// 1. 打开项目
const project = await invoke('open_directory', { 
  path: '/path/to/project' 
});

// 2. 监听进度
const unlisten = await listen('analysis_progress', (e) => {
  console.log(`${e.payload.phase}: ${e.payload.progress}%`);
});

// 3. 获取函数列表
const functions = await invoke('get_functions', { 
  filePath: '/path/to/file.c' 
});

// 4. 构建执行流
const flow = await invoke('build_execution_flow', {
  filePath: '/path/to/file.c',
  entryFunction: 'probe',
  maxDepth: 5,
});

// 5. 格式化导出
const output = await invoke('format_execution_flow', {
  flow,
  format: 'mermaid',
});

console.log(output.content);

// 6. 清理
unlisten();
```
