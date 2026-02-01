# 桌面自动化测试技能

> 使用 **DeskPilot** (`deskpilot`) 框架进行桌面应用自动化测试

## 概述

**DeskPilot** 是 AI 驱动的桌面自动化测试框架，已开源在 [GitHub](https://github.com/TbusOS/DeskPilot)。

| 项目 | 信息 |
|------|------|
| GitHub | https://github.com/TbusOS/DeskPilot |
| npm 包名 | `deskpilot` |
| 本地路径 | `packages/desktop-test/` |

框架采用混合模式：
- **确定性优先**: 使用 CDP/DOM 进行快速、可靠的测试
- **VLM 智能回退**: 当确定性方法失败时，使用视觉 AI 自动恢复

## 框架位置

```
packages/desktop-test/
├── src/
│   ├── core/
│   │   ├── desktop-test.ts   # 主 API
│   │   ├── assertions.ts     # 断言方法
│   │   └── test-runner.ts    # 测试运行器
│   ├── adapters/
│   │   ├── cdp-adapter.ts    # Chrome DevTools Protocol
│   │   ├── python-bridge.ts  # Python 桥接
│   │   └── nutjs-adapter.ts  # 原生桌面控制
│   └── vlm/
│       ├── client.ts         # VLM 客户端
│       └── cost-tracker.ts   # 成本追踪
├── examples/
│   └── flowsight-tests.ts    # FlowSight 测试示例
└── docs/
    ├── API.md                # API 参考
    └── DESIGN.md             # 设计文档
```

## 核心 API

### 1. DesktopTest - 主 API

```typescript
import { DesktopTest, TestMode } from 'deskpilot';

const test = new DesktopTest({
  mode: TestMode.Hybrid,        // 混合模式（推荐）
  cdp: {
    port: 9222,                 // CDP 端口
  },
  vlm: {
    provider: 'anthropic',      // VLM 提供商
    model: 'claude-sonnet-4-20250514',
  },
});

await test.connect();
await test.click('[data-testid="button"]');
await test.type('input', 'hello world');
await test.disconnect();
```

### 2. TestRunner - 测试运行器

```typescript
import { TestRunner, DesktopTest } from 'deskpilot';

const runner = new TestRunner({
  name: 'FlowSight Tests',
  retries: 2,
});

runner.test('测试名称', async (test: DesktopTest) => {
  await test.connect();
  // 测试逻辑
});

runner.run();
```

### 3. Assertions - 断言方法

```typescript
import { Assertions } from 'deskpilot';

// 🔴 数据正确性断言（必须使用）
Assertions.valueNotZero(count, '数量不能为零');
Assertions.valueNotEmpty(list, '列表不能为空');
Assertions.validateData(data, {
  files: (v) => v > 0,
  functions: (v) => v > 0,
}, '数据验证失败');

// 基本断言
test.assert.ok(condition, '条件检查');
test.assert.equal(actual, expected, '相等检查');

// 元素断言
await test.assert.visible('[data-testid="panel"]');
await test.assert.hasText('[data-testid="stats"]', '42 个文件');
```

## 测试模式

| 模式 | 说明 | 使用场景 |
|------|------|---------|
| `TestMode.Deterministic` | 仅 CDP/DOM | 开发调试，快速迭代 |
| `TestMode.VLM` | 仅 VLM | 视觉验证，复杂交互 |
| `TestMode.Hybrid` | 混合（推荐） | 生产环境，自动恢复 |

## VLM Provider

| Provider | 说明 | API Key |
|----------|------|---------|
| `agent` | **🔴 推荐：自动检测 Claude 环境** | 无需配置 |
| `cursor` | Cursor IDE | 无需配置 |
| `anthropic` | Anthropic Claude API | `ANTHROPIC_API_KEY` |
| `openai` | OpenAI GPT-4V | `OPENAI_API_KEY` |
| `volcengine` | 火山引擎豆包 | `VOLCENGINE_API_KEY` |

### 🔴 Agent 模式（自动检测所有 Claude 环境）

框架会**自动检测**以下 Claude Agent 环境，无需手动配置：

| 环境 | 检测方式 | 说明 |
|------|---------|------|
| **Cursor IDE** | `CURSOR_SESSION` 等 | Cursor 编辑器 |
| **Claude Code CLI** | `CLAUDE_CODE` 等 | 终端命令行 |
| **VSCode Claude** | `VSCODE_CLAUDE` 等 | VSCode 插件 |
| **Claude Desktop** | `CLAUDE_DESKTOP` 等 | 桌面应用 |
| **MCP 环境** | `MCP_SERVER` 等 | 任何 MCP 服务 |

```typescript
// 方式 1: 显式使用 agent provider
const test = new DesktopTest({
  vlm: { provider: 'agent' },  // 自动检测环境
});

// 方式 2: 自动检测（无 API Key 时自动启用）
const test = new DesktopTest({
  mode: TestMode.Hybrid,
  // 如果没有 ANTHROPIC_API_KEY，会自动检测 Agent 环境
});
```

**优势**：
- ✅ **自动检测** - 支持所有 Claude 环境（Cursor/CLI/VSCode/Desktop）
- ✅ **无需 API Key** - 使用当前会话的 Claude 模型
- ✅ **成本为零** - 不产生额外 API 费用
- ✅ **最新模型** - 使用当前环境的 Claude 模型（如 Opus 4.5）

## 启动测试

### 1. 启动应用（启用 CDP）

```bash
WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222 cargo tauri dev
```

### 2. 运行测试

```bash
# 安装依赖
cd packages/desktop-test
npm install
npm run build

# 运行示例测试
npx tsx examples/flowsight-tests.ts

# 使用 VLM 模式
ANTHROPIC_API_KEY=xxx npx tsx examples/flowsight-tests.ts
```

## 典型测试场景

### 场景 1: 打开目录并验证统计

```typescript
runner.test('打开目录显示正确统计', async (test) => {
  // 打开目录
  await test.clickText('打开');
  await test.type('[data-testid="path-input"]', '/path/to/drivers/gpio');
  await test.click('[data-testid="confirm-btn"]');
  
  // 等待扫描完成
  await test.waitFor('[data-testid="stats-panel"]');
  
  // 获取并验证统计
  const statsText = await test.getText('[data-testid="stats-panel"]');
  const match = statsText.match(/(\d+) 个文件.*?(\d+) 个函数/);
  
  if (match) {
    const files = parseInt(match[1]);
    const functions = parseInt(match[2]);
    
    // 🔴 必须使用数据正确性断言
    Assertions.valueNotZero(files, '文件数不能为零');
    Assertions.valueNotZero(functions, '函数数不能为零');
  }
});
```

### 场景 2: 分析执行流

```typescript
runner.test('分析函数执行流', async (test) => {
  // 选择函数
  await test.click('[data-testid="function-list"] >> text=probe');
  
  // 验证执行流面板显示
  await test.assert.visible('[data-testid="flow-panel"]');
  
  // 验证节点数量
  const nodeCount = await test.evaluate(() => 
    document.querySelectorAll('.react-flow__node').length
  );
  
  // 必须有多个节点（不能只有入口）
  Assertions.valueNotZero(nodeCount - 1, '执行流应有调用节点');
  
  // 验证边数量
  const edgeCount = await test.evaluate(() => 
    document.querySelectorAll('.react-flow__edge').length
  );
  Assertions.valueNotZero(edgeCount, '执行流应有调用关系');
});
```

### 场景 3: VLM 智能交互

```typescript
runner.test('使用 AI 完成复杂操作', async (test) => {
  // 使用自然语言指令
  await test.ai('打开文件树中的 gpio-dwapb.c 文件');
  
  // 使用文本点击（VLM 智能识别）
  await test.clickText('分析执行流');
  
  // VLM 视觉验证
  const result = await test.vlm?.assertVisual({
    screenshot: await test.getScreenshotBase64(),
    assertion: '执行流图中应该显示多个连接的节点',
  });
  
  test.assert.ok(result?.passed, 'VLM 视觉验证');
});
```

## 与其他 Agents 协作

### E2E-Tester Agent

```
E2E-Tester 必须使用 DeskPilot (`deskpilot`) 框架编写测试
- 使用 TestRunner 组织测试用例
- 使用 Assertions.valueNotZero 验证数据
- 提交前确保所有测试通过
```

### Test-Reviewer Agent

```
Test-Reviewer 必须检查:
- 是否使用了正确的测试框架
- 是否使用了数据正确性断言
- 是否避免了只检查存在性的弱断言
```

### UI-Dev Agent

```
UI-Dev 在提交 UI 更改前:
1. 添加对应的桌面测试用例
2. 确保测试覆盖核心交互
3. 运行测试验证无回归
```

### Debug-Dev Agent

```
Debug-Dev 修复 Bug 后:
1. 添加回归测试用例
2. 使用数据正确性断言
3. 确保 Bug 不会再次出现
```

## 最佳实践

### ✅ 推荐

```typescript
// 使用数据正确性断言
Assertions.valueNotZero(count, 'message');

// 验证实际内容
await test.assert.hasText('.panel', '期望的文本');

// 使用测试 ID
await test.click('[data-testid="button"]');

// 设置合理超时
await test.waitFor('.loading', { timeout: 10000 });
```

### ❌ 避免

```typescript
// 只检查存在性
await test.assert.visible('.panel'); // ❌ 可能显示错误内容

// 使用不稳定选择器
await test.click('.btn-primary'); // ❌ 可能匹配多个元素

// 硬编码等待
await test.wait(5000); // ❌ 使用 waitFor 代替
```

## 相关文档

- [API 参考](../../packages/desktop-test/docs/API.md)
- [设计文档](../../packages/desktop-test/docs/DESIGN.md)
- [贡献指南](../../packages/desktop-test/CONTRIBUTING.md)
