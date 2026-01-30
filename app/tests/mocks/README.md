# Tauri API Mock

用于在 Playwright 测试中模拟 Tauri 原生 API，使测试可以在纯浏览器环境中运行。

## 功能特性

- 模拟 `@tauri-apps/api/core` 的 `invoke()` 命令
- 模拟 `@tauri-apps/plugin-dialog` 的文件对话框
- 支持自定义返回值和响应
- 提供 Mock 数据工厂创建测试数据
- TypeScript 类型完整支持

## 快速开始

### 基本用法

```typescript
import { test, expect } from '@playwright/test';
import { tauriMockScript } from '../mocks';

test('opens command palette', async ({ page }) => {
  // 注入 Mock 脚本（必须在 goto 之前）
  await page.addInitScript(tauriMockScript);
  
  await page.goto('http://localhost:5173/');
  
  // 打开命令面板
  await page.keyboard.press('Meta+K');
  
  // 点击"打开项目"
  await page.click('text=打开项目');
  
  // Mock 会自动返回 /mock/project 路径
  // 验证项目打开成功...
});
```

### 自定义配置

```typescript
import { setupTauriMock } from '../mocks';

test('custom dialog response', async ({ page }) => {
  await setupTauriMock(page, {
    verbose: true, // 启用详细日志
    dialogResponses: {
      openProject: '/custom/project/path',
      openFile: '/custom/file.c',
      saveFile: '/custom/output.txt',
    },
  });
  
  await page.goto('http://localhost:5173/');
  // ...
});
```

### 自定义文件内容

```typescript
import { createMockWithFiles } from '../mocks';

test('test with custom files', async ({ page }) => {
  const script = createMockWithFiles({
    '/my/project/main.c': `
      #include <stdio.h>
      int main() { return 0; }
    `,
    '/my/project/utils.h': `
      #ifndef UTILS_H
      #define UTILS_H
      void helper(void);
      #endif
    `,
  });
  
  await page.addInitScript(script);
  await page.goto('http://localhost:5173/');
  // ...
});
```

## Mock 的 API 列表

### Dialog API

| 函数 | 默认返回值 |
|------|-----------|
| `open({ directory: true })` | `/mock/project` |
| `open({ ... })` | `/mock/project/src/main.c` |
| `save({ ... })` | `/mock/output/result.txt` |
| `message()` | `void` |
| `ask()` | `true` |
| `confirm()` | `true` |

### Core API (invoke 命令)

| 命令 | 描述 |
|------|------|
| `open_project` | 打开项目，返回 ProjectInfo |
| `get_index_stats` | 获取索引统计 |
| `list_directory` | 列出目录内容 |
| `read_file` | 读取文件内容 |
| `write_file` | 写入文件内容 |
| `get_functions` | 获取函数列表 |
| `analyze_file` | 分析文件 |
| `search_symbols` | 搜索符号 |
| `build_execution_flow` | 构建执行流 |
| `get_entry_points` | 获取入口点 |
| `get_async_bindings` | 获取异步绑定 |
| `delete_file_or_dir` | 删除文件/目录 |
| `rename_file` | 重命名文件 |
| `create_file` | 创建文件 |
| `create_directory` | 创建目录 |
| `export_flow_text` | 导出执行流文本 |

## 高级用法

### 使用 Mock 数据工厂

```typescript
import { createMockDataFactory } from '../mocks';

const factory = createMockDataFactory();

// 创建自定义文件树
const fileTree = factory.createFileTree('/my/project');

// 创建函数列表
const functions = factory.createFunctions();

// 创建分析结果
const result = factory.createAnalysisResult('/my/project/main.c');

// 创建执行流
const flow = factory.createExecutionFlow('main', '/my/project/main.c');
```

### 在 Node.js 测试中使用

```typescript
import { createTauriMock } from '../mocks';

describe('Analysis Store', () => {
  const mock = createTauriMock({ verbose: true });
  
  beforeEach(() => {
    // 设置全局 mock
    (global as any).__TAURI__ = {
      core: mock.core,
      dialog: mock.dialog,
    };
  });
  
  it('opens project', async () => {
    const result = await mock.core.invoke('open_project', { path: '/test' });
    expect(result.indexed).toBe(true);
  });
});
```

### 自定义 invoke 处理器

```typescript
import { generateTauriMockScript } from '../mocks';

const script = generateTauriMockScript({
  customInvokeHandlers: {
    my_custom_command: async (args) => {
      return { custom: 'result' };
    },
  },
});
```

## 文件结构

```
app/tests/mocks/
├── index.ts          # 入口文件
├── tauri-api.ts      # 主 Mock 实现
└── README.md         # 本文档
```

## 注意事项

1. **注入时机**: `page.addInitScript()` 必须在 `page.goto()` 之前调用
2. **页面刷新**: 页面刷新后需要重新注入脚本
3. **真实 API**: Mock 仅在测试环境使用，真实应用仍使用 Tauri API
4. **状态隔离**: 每个测试应该使用独立的 Mock 实例

## 相关链接

- [Playwright 文档](https://playwright.dev/)
- [Tauri API 文档](https://tauri.app/v1/api/js/)
- [FlowSight 测试指南](../desktop/README.md)
