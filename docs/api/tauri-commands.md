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

## 格式化导出

### `format_execution_flow`

格式化执行流为指定格式。

```typescript
invoke('format_execution_flow', {
  flow: ExecutionFlow,
  format: 'mermaid' | 'table' | 'text' | 'ai',
  options?: FormatOptions
}): Promise<FormattedOutput>
```

**参数:**
| 名称 | 类型 | 描述 |
|------|------|------|
| flow | ExecutionFlow | 执行流数据 |
| format | string | 输出格式 |
| options | FormatOptions | 格式化选项 |

**格式选项:**
```typescript
interface FormatOptions {
  includeLineNumbers?: boolean;
  includeConfidence?: boolean;
  maxDepth?: number;
  language?: 'zh' | 'en';
}
```

**返回值:**
```typescript
interface FormattedOutput {
  content: string;
  format: string;
  metadata: {
    nodeCount: number;
    edgeCount: number;
    asyncCount: number;
  };
}
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
