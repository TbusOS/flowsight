# Agent Browser 集成测试

基于 [agent-browser](https://github.com/TbusOS/agent-browser) 的桌面应用 AI 驱动测试套件。

## 核心优势

1. **CDP 连接**: 直接连接运行中的 Tauri 应用，无需 Mock
2. **AI 友好**: Snapshot + Refs 系统让 AI 可以准确定位元素
3. **视频录制**: 自动录制测试过程用于调试
4. **跨平台**: 支持 macOS, Linux, Windows

## 安装

```bash
# 全局安装 agent-browser
npm install -g agent-browser

# 下载 Chromium
agent-browser install
```

## 使用方式

### 1. 启动 FlowSight 应用（开发模式 + 远程调试）

**macOS:**
```bash
# 在项目根目录
WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222 cargo tauri dev
```

**Linux:**
```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 cargo tauri dev --debug
```

**Windows:**
```powershell
# Tauri WebView2 自动支持 CDP
cargo tauri dev
```

### 2. 连接并测试

```bash
# 连接到 Tauri WebView
agent-browser connect 9222

# 获取当前页面快照（AI 友好格式）
agent-browser snapshot -i

# 示例输出:
# - heading "FlowSight" [ref=e1]
# - button "打开项目" [ref=e2]
# - tree "文件树" [ref=e3]
# - region "执行流" [ref=e4]

# 通过 ref 进行交互
agent-browser click @e2
agent-browser fill @e5 "/path/to/kernel"
```

### 3. 自动化测试脚本

```bash
# 运行完整测试套件
npx tsx tests/desktop/agent-browser/run-tests.ts

# 单独运行特定测试
npx tsx tests/desktop/agent-browser/test-open-project.ts
```

## 测试场景

### 场景 1: 打开项目验证

验证打开内核项目后：
- 文件树正确显示
- 统计信息非零
- 执行流图渲染

### 场景 2: 节点交互

验证点击执行流节点：
- 节点详情面板显示
- 代码高亮正确
- 跳转到定义工作

### 场景 3: 搜索功能

验证搜索：
- 函数搜索返回结果
- 结果点击跳转正确

## 与 Mock 测试的区别

| 维度 | Mock 测试 | Agent Browser 测试 |
|------|----------|-------------------|
| 后端 | 模拟数据 | 真实 Rust 后端 |
| 渲染 | JSDOM | 真实 WebView |
| 交互 | 模拟事件 | 真实用户操作 |
| 可靠性 | 低 | 高 |
| 速度 | 快 | 稍慢 |

## CI/CD 集成

```yaml
# .github/workflows/desktop-e2e.yml
jobs:
  desktop-e2e:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install agent-browser
        run: |
          npm install -g agent-browser
          agent-browser install
      
      - name: Build Tauri app
        run: cargo build --release
      
      - name: Start app with debug port
        run: |
          WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222 \
          ./target/release/flowsight &
          sleep 5
      
      - name: Run E2E tests
        run: npx tsx app/tests/desktop/agent-browser/run-tests.ts
```

## 调试技巧

### 查看实时页面

```bash
# 启用流式传输
AGENT_BROWSER_STREAM_PORT=9223 agent-browser snapshot

# 在浏览器中查看 ws://localhost:9223
```

### 录制视频

```bash
# 开始录制
agent-browser record start ./test-video.webm

# 执行测试操作...

# 停止录制
agent-browser record stop
```

### 调试模式

```bash
# 显示浏览器窗口（headed 模式）
agent-browser --headed open example.com
```
