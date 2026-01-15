# FlowSight 代码风格指南

本指南定义了 FlowSight 项目的代码风格规范，确保代码的一致性和可维护性。

## 通用规范

### 1. 文件组织

- **文件命名**: 使用 kebab-case (如 `my-component.tsx`)
- **目录结构**: 功能导向，按模块分组
- **单文件职责**: 每个文件只做一件事

### 2. 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 组件 | PascalCase | `FlowView`, `CodeEditor` |
| 函数/变量 | camelCase | `getFunctions`, `currentFile` |
| 常量 | UPPER_SNAKE_CASE | `MAX_DEPTH`, `DEFAULT_TIMEOUT` |
| 类型/接口 | PascalCase | `FunctionDef`, `CallEdge` |
| CSS 类 | kebab-case | `command-palette`, `flow-node` |

### 3. 导入顺序

```typescript
// 1. React 相关
import { useState, useEffect } from 'react'

// 2. 外部库
import { useQuery } from '@tanstack/react-query'

// 3. 内部组件
import { FlowView } from './FlowView'
import { CodeEditor } from './Editor'

// 4. 类型定义
import type { FunctionDef, CallEdge } from '../types'

// 5. 工具函数
import { formatPath } from '../utils/path'
```

---

## Rust 规范 (后端)

### 1. 代码结构

```rust
// 模块声明
pub mod analysis;
pub mod parser;

// 公开结构体放前面
pub struct FunctionDef {
    pub name: String,
    // ...
}

// 私有结构体放后面
struct InternalNode {
    // ...
}
```

### 2. 错误处理

- 使用 `thiserror` 定义自定义错误
- 优先使用 `Result` 而非 `panic`
- 提供有意义的错误信息

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlowSightError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Parse error at {location}: {message}")]
    ParseError {
        location: Location,
        message: String,
    },

    #[error("Index error: {message}")]
    IndexError { message: String },
}
```

### 3. 异步代码

- 使用 `tokio` 异步运行时
- 避免过度嵌套的 `.await`
- 使用 `?` 操作符传播错误

```rust
pub async fn analyze_file(path: PathBuf) -> Result<AnalysisResult, FlowSightError> {
    let source = tokio::fs::read_to_string(&path).await?;
    let ast = parse_source(&source)?;
    let result = perform_analysis(ast).await?;

    Ok(result)
}
```

### 4. 文档注释

```rust
/// 构建函数的调用图
///
/// # 参数
/// * `func` - 要分析的目标函数
/// * `max_depth` - 最大递归深度
///
/// # 返回
/// 返回包含所有调用关系的图结构
///
/// # 示例
/// ```
/// let graph = build_call_graph("main_function", 10);
/// ```
pub fn build_call_graph(func: &str, max_depth: u32) -> Result<CallGraph, Error> {
    // ...
}
```

---

## TypeScript/React 规范 (前端)

### 1. 组件结构

```tsx
// 导入
import { useState, useCallback } from 'react'
import type { FunctionDef } from '../types'
import './MyComponent.css'

// 类型定义
interface MyComponentProps {
  data: FunctionDef[]
  onSelect: (item: FunctionDef) => void
}

// 主组件
export function MyComponent({ data, onSelect }: MyComponentProps) {
  // 状态
  const [selected, setSelected] = useState<string | null>(null)

  // 回调
  const handleClick = useCallback((id: string) => {
    setSelected(id)
    const item = data.find(d => d.id === id)
    if (item) onSelect(item)
  }, [data, onSelect])

  // 渲染
  return (
    <div className="my-component">
      {data.map(item => (
        <div key={item.id} onClick={() => handleClick(item.id)}>
          {item.name}
        </div>
      ))}
    </div>
  )
}

export default MyComponent
```

### 2. 状态管理

- 优先使用局部状态 (`useState`)
- 需要跨组件共享时使用 Zustand store
- 避免不必要的 `useEffect`

```typescript
// Good: 使用局部状态
const [isOpen, setIsOpen] = useState(false)

// Good: 使用 Zustand
import { create } from 'zustand'

interface EditorStore {
  activeFile: string | null
  setActiveFile: (path: string) => void
}

export const useEditorStore = create<EditorStore>((set) => ({
  activeFile: null,
  setActiveFile: (path) => set({ activeFile: path }),
}))
```

### 3. 样式规范

- 使用 CSS 变量实现主题
- 组件样式放在同目录的 `.css` 文件
- 使用 BEM 命名或 CSS 模块

```css
/* ComponentName.css */
.component-name {
  --primary-color: var(--accent);
}

.component-name__header {
  /* ... */
}

.component-name--active {
  /* ... */
}
```

---

## Git 提交规范

### 提交类型

| 类型 | 描述 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档更新 |
| `style` | 代码格式（不影响功能） |
| `refactor` | 重构 |
| `perf` | 性能优化 |
| `test` | 添加测试 |
| `chore` | 构建/工具更新 |

### 提交格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 示例

```
feat(analysis): 添加函数指针解析功能

实现了对 struct file_operations 等 ops 表的解析，
能够识别 .open = my_open 形式的函数指针赋值。

Closes #123
```

---

## 代码审查清单

- [ ] 代码符合风格指南
- [ ] 命名清晰有意义
- [ ] 注释解释"为什么"，而非"是什么"
- [ ] 错误处理完善
- [ ] 没有硬编码的值
- [ ] 适当的测试覆盖
- [ ] 性能考虑
- [ ] 安全考虑

---

## 工具配置

### ESLint

```json
{
  "extends": [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended"
  ],
  "rules": {
    "@typescript-eslint/no-unused-vars": "error",
    "@typescript-eslint/explicit-function-return-type": "warn"
  }
}
```

### rustfmt

```toml
# rustfmt.toml
max_width = 120
tab_spaces = 4
edition = "2021"
group_imports = "StdExternalCrate"
```

---

## 参考资源

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [React 最佳实践](https://react.dev/learn)
- [TypeScript 手册](https://www.typescriptlang.org/docs/)
