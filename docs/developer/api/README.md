# API 参考文档

本目录包含 FlowSight 的 API 参考文档。

> 🚧 文档编写中，将在核心代码实现后完善

## 📋 计划文档

| 文档 | 描述 |
|------|------|
| CORE-API.md | 核心分析引擎 Rust API |
| KNOWLEDGE-API.md | 知识库加载与查询 API |
| PLUGIN-API.md | 插件扩展接口 |
| IPC-PROTOCOL.md | Tauri 前后端通信协议 |
| CLI-REFERENCE.md | 命令行工具参考 |

## 🔑 核心 API 预览

### 分析引擎

```rust
// 项目分析入口
pub trait Analyzer {
    fn analyze_file(&self, path: &Path) -> Result<AnalysisResult>;
    fn get_call_graph(&self, function: &str) -> Result<CallGraph>;
    fn get_execution_flow(&self, entry: &str) -> Result<FlowGraph>;
}
```

### 知识库

```rust
// 知识库查询
pub trait KnowledgeBase {
    fn match_async_pattern(&self, code: &str) -> Vec<AsyncPattern>;
    fn get_framework_callbacks(&self, framework: &str) -> Vec<Callback>;
    fn resolve_ops_table(&self, type_name: &str) -> Option<OpsTable>;
}
```

### 前端通信

```typescript
// Tauri IPC 命令
invoke('analyze_function', { path: string, name: string }): Promise<FlowGraph>
invoke('get_call_hierarchy', { path: string, position: Position }): Promise<CallHierarchy>
invoke('search_symbols', { query: string }): Promise<Symbol[]>
```

