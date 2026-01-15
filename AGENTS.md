# FlowSight Agent 系统

FlowSight 内置三种 AI Agent，用于辅助代码分析和理解。

## Agent 类型

| Agent | 用途 | 使用方式 |
|-------|------|----------|
| **analyze** | 执行流分析 | 命令面板输入 `? analyze` |
| **explain** | 代码解释 | 命令面板输入 `? explain` |
| **search** | 代码搜索 | 命令面板输入 `? search` |

## 使用方法

### 1. 打开命令面板

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+P` / `Cmd+P` | 打开命令面板 |

### 2. 使用 Agent

在命令面板中：
- 输入 `?` 激活 AI Agent
- 或直接输入 Agent 名称搜索

```
? analyze        # 启动分析 Agent
? explain        # 启动解释 Agent
? search         # 启动搜索 Agent
```

## Agent 详细说明

### Analyze Agent

**用途**: 分析代码的执行流

**能力**:
- 分析函数的调用链
- 识别异步机制（WorkQueue, Timer, IRQ 等）
- 解析函数指针
- 生成执行流报告

**使用示例**:
```
> 分析 usb_probe 函数的执行流
> 分析当前文件的异步调用
> trace schedule_work 的调用路径
```

### Explain Agent

**用途**: 用自然语言解释代码

**能力**:
- 解释函数的功能
- 描述调用关系
- 说明执行上下文
- 提供代码示例

**使用示例**:
```
> 解释 INIT_WORK 宏的作用
> 解释这个回调函数什么时候被调用
> 解释 USB probe 的流程
```

### Search Agent

**用途**: 搜索代码模式

**能力**:
- 搜索函数定义
- 查找调用点
- 发现模式
- 列出相关符号

**使用示例**:
```
> 查找所有 probe 函数
> 搜索 file_operations 初始化
> 找 my_handler 的调用者
```

## 配置文件

Agent 配置位于 `.claude/opencode.jsonc`:

```jsonc
{
  "agent": {
    "analyze": {
      "description": "Code execution flow analysis",
      "model": "claude-sonnet-4-5"
    },
    "explain": {
      "description": "Natural language code explanation",
      "model": "claude-sonnet-4-5"
    },
    "search": {
      "description": "Code pattern search",
      "model": "claude-haiku-4-5"
    }
  }
}
```

## Agent 配置文件

每个 Agent 的详细配置位于 `.claude/agents/` 目录：

| 文件 | 作用 |
|------|------|
| `analyze.md` | 分析 Agent 的行为规范 |
| `explain.md` | 解释 Agent 的行为规范 |
| `search.md` | 搜索 Agent 的行为规范 |

## 工具权限

Agent 只拥有必要的只读权限：

| 工具 | 权限 | 用途 |
|------|------|------|
| `read_file` | ✅ | 读取源代码 |
| `search_symbols` | ✅ | 搜索符号 |
| `analyze_code` | ✅ | 分析代码 |
| `get_functions` | ✅ | 获取函数列表 |
| `write_file` | ❌ | 禁止写入 |
| `run_commands` | ❌ | 禁止执行命令 |

## 输出格式

### Analyze 输出

```markdown
## 分析结果

### 函数信息
- 名称: `function_name`
- 位置: `file:line`
- 类型: sync/async

### 执行流

#### 同步调用链
```
caller1
  └── caller2
        └── target_function
```

#### 异步机制
- 类型: WorkQueue/Timer/IRQ/Tasklet
- 绑定: `INIT_WORK(&work, handler)`
- 触发: `schedule_work(&work)`
```

### Explain 输出

```markdown
## 代码解释

### 概述
[一句话总结代码功能]

### 何时执行
- **触发条件**: [执行条件]
- **执行上下文**: Process/SoftIrq/HardIrq/User

### 代码详解
[详细解释]
```

### Search 输出

```markdown
## 搜索结果: [query]

### 概览
- 找到: N 个结果
- 范围: [scope]
- 类型: [type]

### 详细结果
[带文件路径和行号的结果列表]
```

## 最佳实践

1. **明确提问**: 提供具体的函数名或文件路径
2. **指定范围**: 说明要分析的范围（整个文件还是单个函数）
3. **查看置信度**: 注意分析结果的置信度（Certain/Possible/Unknown）
4. **追问细节**: 可以继续提问获取更详细的解释

## 常见问题

### Q: Agent 和内置分析有什么区别？

A: 内置分析提供精确的静态分析结果，Agent 则可以：
- 用自然语言解释分析结果
- 发现模式和处理边缘情况
- 提供代码示例和最佳实践

### Q: Agent 会修改我的代码吗？

A: 不会。Agent 只有只读权限，无法修改任何文件。

### Q: Agent 支持离线使用吗？

A: 当前版本需要网络连接。未来计划支持 Ollama 本地模型。

## 相关文档

- [安装指南](docs/install.md)
- [使用手册](docs/user-guide/README.md)
- [开发者指南](docs/developer/README.md)
