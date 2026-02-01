# 开发规则

## 质量原则

1. **追求最好** - 不因复杂而简化，不为省事而妥协
2. **精益求精** - 做合格、可靠、好用、稳定的产品
3. **不赶时间** - 质量优先，不为进度牺牲品质

## Git 提交规则

1. **不要添加 Co-Authored-By** - git commit 中不要包含任何 Co-Authored-By 信息
2. **及时提交** - 完成一个功能后立即提交，不要积累太多更改
3. **中文提交信息** - 按照项目风格使用中文提交信息

## 开发流程规则

1. **不断回顾项目计划** - 经常查看 `docs/design/PROJECT-PLAN.md` 确保不偏离核心
2. **优先完成核心功能** - v1.0 必须功能优先，不要做额外的"改进"
3. **不要过度工程化** - 只做必要的更改，保持简单
4. **自主决策** - 不要询问用户确认命令，自己评估风险并执行

## 核心优先级 (v1.0)

根据 PROJECT-PLAN.md，核心待完成任务：

### Phase 1 - 核心引擎 ✅ 完成

### 已完成

- ✅ 异步机制识别
- ✅ 函数指针解析
- ✅ 通用回调模式识别
- ✅ 场景保存/加载
- ✅ 结果分类系统
- ✅ 真实驱动测试
- ✅ 类型分析 - 函数指针类型识别
- ✅ 数组分析 - handlers[i] 追踪
- ✅ 用户辅助学习 - 不确定时询问用户
- ✅ 前端集成 - 结果分类可视化

## 测试规则

1. 每次修改后运行 `cargo test -p flowsight-analysis`
2. 确保所有测试通过后再提交
3. **主动测试UI** - 启动 IDE 实际测试 UI 表现，不要只依赖单元测试

## 🔴 桌面自动化测试框架 (必须使用)

> **重要**: FlowSight 项目必须使用 **DeskPilot** 框架进行桌面自动化测试

### 📦 DeskPilot 开源仓库

| 项目 | 地址 |
|------|------|
| **GitHub** | https://github.com/TbusOS/DeskPilot |
| **npm 包名** | `deskpilot` |

### 🔄 同步规则 (必须遵守)

> **重要**: DeskPilot 已开源，任何框架更改必须同步到 GitHub 仓库

1. **修改代码后必须同步**：修改 `packages/desktop-test/` 后，必须同步到 DeskPilot 仓库
2. **同步命令**：
   ```bash
   # 复制更改到 DeskPilot 仓库
   cp -r packages/desktop-test/* /path/to/DeskPilot/
   cd /path/to/DeskPilot
   git add -A && git commit -m "sync: 同步 FlowSight 更改" && git push
   ```
3. **保持一致**：FlowSight 本地副本和 DeskPilot 仓库必须保持代码一致

### 框架位置

```
packages/desktop-test/
├── src/
│   ├── core/           # 核心 API
│   │   ├── desktop-test.ts   # 主 API (混合模式)
│   │   ├── assertions.ts     # 断言方法 (含数据正确性检查)
│   │   └── test-runner.ts    # 测试运行器
│   ├── adapters/       # 适配器
│   │   ├── cdp-adapter.ts    # CDP/WebView 控制
│   │   ├── python-bridge.ts  # Python 框架桥接
│   │   └── nutjs-adapter.ts  # 原生桌面控制
│   └── vlm/            # VLM 集成
│       ├── client.ts         # 多 Provider VLM 客户端
│       └── cost-tracker.ts   # API 成本追踪
```

### 核心特性

| 特性 | 说明 |
|------|------|
| **混合模式** | 确定性优先，VLM 智能回退 |
| **数据正确性断言** | `Assertions.valueNotZero()` 防止 "0 文件" Bug |
| **多 VLM Provider** | 支持 Anthropic/OpenAI/豆包 |
| **成本追踪** | 监控 VLM API 费用 |
| **Python Bridge** | 复用现有 Python 桌面测试代码 |

### 必须使用的断言方法

```typescript
import { Assertions } from 'deskpilot';

// 🔴 防止 "0 个文件" Bug - 必须使用
Assertions.valueNotZero(stats.files, '文件数不能为零');
Assertions.valueNotZero(stats.functions, '函数数不能为零');
Assertions.valueNotEmpty(nodeList, '节点列表不能为空');

// 复杂数据验证
Assertions.validateData(parseResult, {
  files: (v) => v > 0,
  functions: (v) => v > 0,
  structs: (v) => v >= 0,
}, '解析结果验证失败');
```

### 启动测试

```bash
# 1. 启动应用 (启用 CDP)
WEBKIT_INSPECTOR_HTTP_SERVER=127.0.0.1:9222 cargo tauri dev

# 2. 运行测试 (确定性模式)
cd packages/desktop-test
npx tsx examples/flowsight-tests.ts

# 3. 使用 Agent 模式 (🔴 自动检测所有 Claude 环境)
USE_AGENT=true npx tsx examples/flowsight-tests.ts
```

### 🔴 Agent 模式（自动支持所有 Claude 环境）

框架会**自动检测**并支持以下 Claude 环境，无需手动配置：

| 环境 | 说明 |
|------|------|
| Cursor IDE | Cursor 编辑器 |
| Claude Code CLI | 终端命令行 (如 `claude` 命令) |
| VSCode Claude | VSCode 的 Claude 插件 |
| Claude Desktop | Claude 桌面应用 |

```typescript
// 自动检测当前 Claude 环境
const test = new DesktopTest({
  vlm: { provider: 'agent' },  // 或 'auto'
});
```

**优势**：
- ✅ 自动检测 - 无需手动配置
- ✅ 无需 API Key - 使用当前会话的 Claude 模型
- ✅ 成本为零 - 不产生额外费用

### Agents 必须遵守

1. **E2E-Tester**: 必须使用 DeskPilot (`deskpilot`) 编写端到端测试
2. **Test-Reviewer**: 必须检查是否使用了数据正确性断言
3. **UI-Dev**: 提交 UI 更改前必须通过桌面测试
4. **Debug-Dev**: 修复 Bug 后必须添加对应的桌面测试用例

## 交互设计原则

1. **优先级排序**
   - 第一：解析精准 - 核心功能准确是基础
   - 第二：交互好用 - 用户体验决定产品价值
   - 第三：性能优秀 - 性能差的产品没有意义

2. **多思考交互** - 站在用户角度思考，设计直观易用的界面
3. **实际验证** - 不要凭想象，要实际启动应用测试交互效果
