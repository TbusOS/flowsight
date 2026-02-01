# FlowSight 桌面应用 E2E 测试指南

## 概述

本文档描述了 FlowSight 桌面应用的端到端 (E2E) 测试方案，基于 [agent-browser](https://github.com/TbusOS/agent-browser) 实现对真实 Tauri 应用的自动化测试。

## 为什么选择 Agent Browser

### 传统 Mock 测试的问题

在之前的测试实践中，我们遇到了一个严重的问题：

- **现象**: UI 显示 "发现 0 个文件，0 个函数，0 个结构体"，执行流只有一个 "probe" 节点
- **原因**: 所有测试都使用 Mock 数据，从未测试真实的后端

### Agent Browser 的优势

| 特性 | 传统 Mock 测试 | Agent Browser 测试 |
|------|---------------|-------------------|
| 后端 | 模拟数据 | 真实 Rust 后端 |
| 渲染 | JSDOM/happy-dom | 真实 WebView |
| 交互 | 模拟事件 | 真实用户操作 |
| 元素定位 | CSS/XPath | AI 友好的 Refs |
| 视频录制 | 无 | 原生支持 |
| 跨平台 | 部分 | 完全支持 |

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│                    测试运行器 (TypeScript)                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────┐    ┌─────────────────┐                 │
│  │   测试用例       │    │   断言库         │                 │
│  │  (test-*.ts)    │───▶│ (TestAssertions) │                │
│  └─────────────────┘    └─────────────────┘                 │
│           │                                                 │
│           ▼                                                 │
│  ┌─────────────────────────────────────────┐                │
│  │        AgentBrowserClient               │                │
│  │  - snapshot()   - click()   - fill()    │                │
│  │  - waitFor()    - getText() - evaluate()│                │
│  └─────────────────────────────────────────┘                │
│                       │                                      │
└───────────────────────│──────────────────────────────────────┘
                        │ CLI 调用
                        ▼
┌───────────────────────────────────────────────────────────────┐
│                    agent-browser CLI                          │
│  Rust native binary with Node.js fallback                    │
└───────────────────────────────────────────────────────────────┘
                        │ CDP (Chrome DevTools Protocol)
                        ▼
┌───────────────────────────────────────────────────────────────┐
│                    Tauri 应用                                  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                    WebView (WRY)                        │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │  │
│  │  │  React UI   │  │  执行流视图  │  │   文件树     │     │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘     │  │
│  └─────────────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                Rust Backend (Tauri)                     │  │
│  │  flowsight-parser  │  flowsight-analysis  │  flowsight-cli │
│  └─────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────┘
```

## 快速开始

### 1. 安装依赖

```bash
# 项目依赖
cd app && pnpm install

# 全局安装 agent-browser (推荐)
npm install -g agent-browser
agent-browser install  # 下载 Chromium
```

### 2. 启动 Tauri 应用（启用远程调试）

**macOS:**
```bash
WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222 cargo tauri dev
```

**Linux:**
```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 cargo tauri dev --debug
```

**Windows:**
```powershell
# WebView2 默认支持 CDP
cargo tauri dev
```

### 3. 运行测试

```bash
# 完整测试套件
pnpm test:desktop

# 数据正确性测试（关键）
pnpm test:desktop:data

# 后端集成测试
pnpm test:integration
```

## 测试文件结构

```
app/tests/
├── desktop/
│   └── agent-browser/
│       ├── README.md              # 使用指南
│       ├── utils.ts               # 测试工具库
│       ├── run-tests.ts           # 主测试套件
│       └── test-data-correctness.ts  # 数据正确性测试
├── integration/
│   └── real-backend-test.ts       # 真实后端测试
└── fixtures/
    └── sample-project/            # 测试用 C 代码
```

## 核心测试场景

### 1. 数据正确性测试

防止 "0 个文件, 0 个函数" 问题：

```typescript
// test-data-correctness.ts

// 检查统计信息不全是零
const statsPatterns = [
  /(\d+)\s*个文件/,
  /(\d+)\s*个函数/,
  /(\d+)\s*个结构体/,
];

const allZero = foundStats.every(s => s.value === 0);
assert.assertTrue(!allZero, '统计信息不应该全是零');

// 检查执行流不只有一个 probe 节点
const hasOnlyProbeNode = probeMatches.length === 1 && !hasOtherNodes;
assert.assertTrue(!hasOnlyProbeNode, '执行流应该有多个节点');
```

### 2. UI 元素验证

使用 Snapshot + Refs 系统：

```typescript
// 获取可访问性树快照
const snapshot = await client.snapshot({ interactive: true });

// 输出示例:
// - heading "FlowSight" [ref=e1]
// - button "打开项目" [ref=e2]
// - tree "文件树" [ref=e3]

// 通过 ref 交互
await client.click('@e2');
await client.fill('@e5', '/path/to/kernel');
```

### 3. 真实后端验证

直接调用 Rust CLI：

```typescript
// real-backend-test.ts
import { execSync } from 'child_process';

const result = execSync(`${CLI_PATH} analyze ${kernelPath}`, {
  encoding: 'utf-8',
});

const data = JSON.parse(result);
assert(data.functions > 0, '应该解析出函数');
assert(data.structs > 0, '应该解析出结构体');
```

## CI/CD 集成

### GitHub Actions 配置

```yaml
# .github/workflows/desktop-e2e.yml
name: Desktop E2E Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
      
      - name: Install Dependencies
        run: |
          npm install -g agent-browser
          agent-browser install
          cd app && pnpm install
      
      - name: Build Rust Backend
        run: cargo build --release
      
      - name: Start Tauri App
        run: |
          WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222 \
          ./target/release/flowsight &
          sleep 10  # 等待应用启动
      
      - name: Run E2E Tests
        run: cd app && pnpm test:desktop
      
      - name: Upload Test Artifacts
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: test-results
          path: app/tests/desktop/agent-browser/results/
```

## 最佳实践

### 1. 使用 Refs 而非 CSS 选择器

```typescript
// 推荐: 使用 snapshot 获取的 ref
const snapshot = await client.snapshot({ interactive: true });
const buttonRef = await client.findRef({ name: '打开项目' });
await client.click(`@${buttonRef}`);

// 不推荐: 硬编码 CSS 选择器（容易过时）
await client.click('#open-project-btn');
```

### 2. 验证数据正确性，而非仅存在性

```typescript
// 推荐: 验证具体数值
assert.assertGreaterThan(
  data.functions, 10,
  'GPIO 驱动应该有超过 10 个函数'
);

// 不推荐: 仅检查存在
assert.assertTrue(data.functions !== undefined, '应该有函数');
```

### 3. 等待状态稳定后断言

```typescript
// 推荐: 显式等待
await client.click('@e2');
await client.waitMs(2000);  // 等待加载完成
const snapshot = await client.snapshot();

// 不推荐: 立即断言
await client.click('@e2');
const snapshot = await client.snapshot();  // 可能还在加载中
```

### 4. 录制视频用于调试

```typescript
// 在测试开始时录制
await client.startRecording('./test-video.webm');

// 执行测试...

// 测试结束时停止
await client.stopRecording();
```

## 常见问题

### Q: 连接失败 "Failed to connect via CDP"

**A:** 确保 Tauri 应用已启动并启用了远程调试：
```bash
WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222 cargo tauri dev
```

### Q: 找不到元素

**A:** 使用 snapshot 命令检查当前页面状态：
```bash
agent-browser snapshot -i
```

### Q: 测试在 CI 中失败但本地通过

**A:** 可能是时序问题，增加等待时间：
```typescript
await client.waitMs(5000);  // 增加等待
await client.waitFor('selector', 30000);  // 增加超时
```

## 扩展阅读

- [Agent Browser 官方文档](https://github.com/TbusOS/agent-browser)
- [Tauri WebView 调试指南](https://tauri.app/v1/guides/debugging/)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
