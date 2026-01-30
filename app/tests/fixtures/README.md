# Test Fixtures

测试用的示例项目和数据文件。

## 目录结构

```
fixtures/
├── sample-project/        # 简单的 C 项目示例
│   ├── main.c            # 主程序入口
│   ├── utils.c           # 工具函数实现
│   ├── utils.h           # 头文件
│   └── Makefile          # 构建文件
└── README.md             # 本文档
```

## sample-project

一个简单的 C 项目，用于测试：
- 项目打开功能
- 文件浏览器显示
- 大纲面板函数列表
- 代码分析功能

### 文件说明

| 文件 | 内容 | 测试用途 |
|------|------|----------|
| `main.c` | 包含 `main`、`init_system`、`process_data`、`cleanup` 函数 | 测试函数列表、入口点检测 |
| `utils.c` | 包含 `calculate`、`print_result` 函数 | 测试多文件分析 |
| `utils.h` | 函数声明 | 测试头文件解析 |
| `Makefile` | 构建配置 | 可选，用于验证项目结构 |

### 使用方法

#### 在 Playwright 测试中

```typescript
import { test } from '@playwright/test';
import { setupTauriMock } from '../mocks';
import * as path from 'path';

// 获取 fixtures 绝对路径
const fixturesPath = path.resolve(__dirname, '../fixtures/sample-project');

test('打开示例项目', async ({ page }) => {
  await setupTauriMock(page, {
    dialogResponses: {
      openProject: fixturesPath,
    },
  });
  
  await page.goto('http://localhost:5173/');
  // ... 测试逻辑
});
```

#### 在单元测试中

```typescript
import * as fs from 'fs';
import * as path from 'path';

const sampleProjectPath = path.resolve(__dirname, '../fixtures/sample-project');
const mainCContent = fs.readFileSync(path.join(sampleProjectPath, 'main.c'), 'utf-8');
```

## 添加新的 Fixtures

1. 在 `fixtures/` 目录下创建新子目录
2. 添加所需的测试文件
3. 更新本 README 文档
4. 在相关测试中使用新 fixtures

## 注意事项

- Fixtures 应该保持简单，只包含测试必需的内容
- 避免添加大型文件或二进制文件
- 文件内容应该能触发预期的解析和分析行为
