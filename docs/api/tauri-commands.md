# Tauri Commands API Reference

> FlowSight v0.2.0 Tauri 命令 API 文档
> 
> 最后更新: 2026-02-03

## 命令概览

| 分类 | 命令数 | 描述 |
|------|--------|------|
| 项目管理 | 5 | 打开、索引、搜索 |
| 代码分析 | 10 | 解析、执行流、入口点 |
| 文件操作 | 6 | 读写、创建、删除 |
| 格式化导出 | 3 | Mermaid、Markdown、JSON |
| AI/知识库 | 3 | 语义解释、上下文注解 |
| LLVM IR | 2 | 生成、解析 |

## 项目管理

### `open_project`

打开项目目录，立即返回，后台异步索引。

```typescript
invoke('open_project', { path: string }): Promise<ProjectInfo>
```

**参数:**
| 名称 | 类型 | 描述 |
|------|------|------|
| path | string | 项目目录绝对路径 |

**返回值:**
```typescript
interface ProjectInfo {
  path: string;
  name: string;
  file_count: number;
  function_count: number;
  struct_count: number;
  index_status: 'ready' | 'indexing' | 'error';
}
```

**事件:** 会触发 `index_progress` 事件通知索引进度。

### `list_directory`

列出目录内容，支持递归。

```typescript
invoke('list_directory', { path: string, recursive: boolean }): Promise<FileNode[]>
```

**参数:**
| 名称 | 类型 | 描述 |
|------|------|------|
| path | string | 目录路径 |
| recursive | boolean | 是否递归 |

**返回值:**
```typescript
interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  is_c_file: boolean;
  children?: FileNode[];
  function_count?: number;
}
```

### `expand_directory`

展开单个目录（懒加载）。

```typescript
invoke('expand_directory', { path: string }): Promise<FileNode[]>
```

### `search_symbols`

全局符号搜索。

```typescript
invoke('search_symbols', { 
  query: string, 
  options?: SearchOptions 
}): Promise<SearchResult[]>
```

**参数:**
| 名称 | 类型 | 必须 | 描述 |
|------|------|------|------|
| query | string | 是 | 搜索关键词 |
| options | SearchOptions | 否 | 搜索选项 |

**搜索选项:**
```typescript
interface SearchOptions {
  kind_filter?: 'function' | 'struct' | 'macro' | 'all';
  file_filter?: string;   // 文件路径过滤
  max_results?: number;   // 最大结果数 (默认: 50)
}
```

**返回值:**
```typescript
interface SearchResult {
  name: string;
  kind: 'function' | 'struct' | 'macro';
  file: string;
  line: number;
  score: number;  // 匹配分数
  preview?: string;
}
```

### `get_index_stats`

获取索引统计信息。

```typescript
invoke('get_index_stats'): Promise<IndexStats>
```

**返回值:**
```typescript
interface IndexStats {
  total_files: number;
  indexed_files: number;
  total_functions: number;
  total_structs: number;
  index_size_bytes: number;
  last_update: string;  // ISO 时间戳
}

## 代码分析

### `analyze_file`

分析单个源文件。

```typescript
invoke('analyze_file', { path: string }): Promise<AnalysisResult>
```

**返回值:**
```typescript
interface AnalysisResult {
  functions: FunctionInfo[];
  structs: StructInfo[];
  entry_points: string[];
  async_bindings: AsyncBinding[];
}
```

### `get_functions`

获取文件中的函数列表。

```typescript
invoke('get_functions', { path: string }): Promise<FunctionInfo[]>
```

**参数:**
| 名称 | 类型 | 描述 |
|------|------|------|
| path | string | 文件绝对路径 |

**返回值:**
```typescript
interface FunctionInfo {
  name: string;
  return_type: string;
  parameters: Parameter[];
  start_line: number;
  end_line: number;
  is_callback: boolean;
  callback_context?: string;
}

interface Parameter {
  name: string;
  type_name: string;
}
```

### `get_function_detail`

从索引获取函数详情。

```typescript
invoke('get_function_detail', { name: string }): Promise<FunctionDetail | null>
```

**返回值:**
```typescript
interface FunctionDetail {
  name: string;
  file: string;
  line: number;
  signature: string;
  callers: string[];
  callees: string[];
  is_entry_point: boolean;
}
```

### `get_function_detail_from_file`

从文件直接解析函数详情（含局部变量、复杂度）。

```typescript
invoke('get_function_detail_from_file', { 
  file_path: string, 
  function_name: string 
}): Promise<ExtendedFunctionDetail>
```

**返回值:**
```typescript
interface ExtendedFunctionDetail {
  name: string;
  return_type: string;
  parameters: Parameter[];
  local_variables: LocalVariable[];
  complexity: number;        // 圈复杂度
  doc_comment?: string;      // 文档注释
  source_code: string;       // 函数源码
}
```

### `get_function_locations`

获取文件中所有函数位置（用于代码导航）。

```typescript
invoke('get_function_locations', { path: string }): Promise<FunctionLocation[]>
```

**返回值:**
```typescript
interface FunctionLocation {
  name: string;
  line: number;
  end_line: number;
  kind: 'function' | 'callback' | 'async_handler';
}
```

### `get_function_callers`

获取函数的调用者。

```typescript
invoke('get_function_callers', { 
  function_name: string, 
  project_path?: string 
}): Promise<CallerInfo[]>
```

**返回值:**
```typescript
interface CallerInfo {
  function: string;
  file: string;
  line: number;
  call_type: 'direct' | 'indirect' | 'callback';
}

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

### `get_async_bindings`

获取文件中检测到的异步机制绑定。

```typescript
invoke('get_async_bindings', { 
  file_path: string 
}): Promise<AsyncBindingInfo[]>
```

**返回值:**
```typescript
interface AsyncBindingInfo {
  mechanism: 'WorkQueue' | 'Timer' | 'Tasklet' | 'IRQ' | 'SoftIRQ' | 'Kthread';
  handler: string;       // 处理函数名
  trigger?: string;      // 触发函数
  bind_location: {       // 绑定位置
    line: number;
    column: number;
  };
  confidence: 'Certain' | 'Possible' | 'Unknown';
}
```

### `build_execution_flow_tree`

获取原始树形结构的执行流（高级用例）。

```typescript
invoke('build_execution_flow_tree', { 
  file_path: string, 
  entry_function: string,
  max_depth?: number 
}): Promise<FlowNode>
```

**返回值:**
```typescript
interface FlowNode {
  id: string;
  name: string;
  file?: string;
  line?: number;
  node_type: 'entry' | 'call' | 'callback' | 'async_trigger' | 'async_handler';
  confidence: 'Certain' | 'Possible' | 'Unknown';
  async_mechanism?: string;
  context?: {
    type: 'process' | 'softirq' | 'hardirq' | 'atomic';
    can_sleep: boolean;
  };
  children: FlowNode[];
}
```

### `execute_scenario`

执行场景化符号分析。

```typescript
invoke('execute_scenario', { 
  file_path: string, 
  scenario: ScenarioRequest 
}): Promise<ScenarioResult>
```

**场景请求:**
```typescript
interface ScenarioRequest {
  entry_function: string;
  constraints: Constraint[];   // 输入约束
  trace_variables?: string[];  // 追踪的变量
}

interface Constraint {
  variable: string;
  condition: 'eq' | 'ne' | 'lt' | 'gt' | 'range' | 'null' | 'not_null';
  value?: any;
}
```

**返回值:**
```typescript
interface ScenarioResult {
  paths: ExecutionPath[];
  variable_states: Map<string, VariableState[]>;
  warnings: string[];
}

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

使用 clang 编译 C 文件为 LLVM IR。

```typescript
invoke('generate_llvm_ir', { file_path: string }): Promise<LlvmIrResult>
```

**要求:** 系统需安装 clang。

**返回值:**
```typescript
interface LlvmIrResult {
  success: boolean;
  ir_content?: string;           // LLVM IR 内容
  functions: LlvmFunction[];     // 解析的函数列表
  globals: LlvmGlobal[];         // 全局变量
  error?: string;                // 错误信息
}

interface LlvmFunction {
  name: string;
  return_type: string;
  parameters: LlvmParam[];
  attributes: string[];
  basic_blocks: number;          // 基本块数量
  instruction_count: number;     // 指令数量
}
```

### `parse_llvm_ir_file`

解析已有的 LLVM IR 文件（.ll 或 .bc）。

```typescript
invoke('parse_llvm_ir_file', { file_path: string }): Promise<LlvmIrResult>
```

**支持格式:**
- `.ll` - LLVM 文本格式
- `.bc` - LLVM 字节码格式

## 文件操作

### `read_file`

读取文件内容。

```typescript
invoke('read_file', { path: string }): Promise<string>
```

### `write_file`

写入文件内容。

```typescript
invoke('write_file', { 
  path: string, 
  content: string 
}): Promise<void>
```

### `create_file`

创建新文件。

```typescript
invoke('create_file', { path: string }): Promise<void>
```

### `create_directory`

创建新目录。

```typescript
invoke('create_directory', { path: string }): Promise<void>
```

### `rename_file`

重命名文件或目录。

```typescript
invoke('rename_file', { 
  old_path: string, 
  new_path: string 
}): Promise<void>
```

### `delete_file_or_dir`

删除文件或目录。

```typescript
invoke('delete_file_or_dir', { path: string }): Promise<void>
```

### `export_flow_text`

导出执行流分析文本到文件。

```typescript
invoke('export_flow_text', { 
  path: string, 
  content: string 
}): Promise<void>
```

## AI 与知识库

### `explain_function`

使用 AI 或知识库解释函数语义。

```typescript
invoke('explain_function', { 
  file_path: string, 
  function_name: string 
}): Promise<FunctionExplanation>
```

**返回值:**
```typescript
interface FunctionExplanation {
  summary: string;           // 功能摘要
  context: string;           // 执行上下文
  parameters_explained: ParameterExplanation[];
  side_effects: string[];    // 副作用
  related_functions: string[];
  knowledge_source: 'knowledge_base' | 'ai' | 'heuristic';
}
```

### `get_context_annotation`

获取异步处理函数的上下文注解。

```typescript
invoke('get_context_annotation', { 
  mechanism: string, 
  handler_code: string 
}): Promise<ContextAnnotation>
```

**返回值:**
```typescript
interface ContextAnnotation {
  execution_context: 'process' | 'softirq' | 'hardirq' | 'atomic';
  can_sleep: boolean;
  can_schedule: boolean;
  preemptible: boolean;
  warnings: string[];
}
```

### `translate_condition`

将约束条件翻译为业务语义。

```typescript
invoke('translate_condition', { 
  code: string, 
  constraint: string 
}): Promise<ConditionTranslation>
```

**返回值:**
```typescript
interface ConditionTranslation {
  original: string;      // 原始条件
  translated: string;    // 翻译后的语义
  confidence: number;    // 置信度 0-1
}

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
