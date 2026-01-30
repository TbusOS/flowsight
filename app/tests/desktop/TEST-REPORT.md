# FlowSight 桌面自动化测试报告

**测试日期**: 2026-01-30  
**测试人员**: E2E-Tester Agent  
**测试范围**: 侧边栏视图切换、文件浏览器、大纲面板

---

## 1. 测试结果摘要

| 测试类别 | 通过 | 失败 | 总计 | 通过率 |
|---------|------|------|------|--------|
| 侧边栏视图切换 | 2 | 1 | 3 | 66.7% |
| 文件浏览器面板 | 2 | 1 | 3 | 66.7% |
| 大纲面板 | 1 | 2 | 3 | 33.3% |
| 键盘快捷键 | 3 | 0 | 3 | 100% |
| UI 可访问性 | 2 | 0 | 2 | 100% |
| **总计** | **10** | **4** | **14** | **71.4%** |

---

## 2. 发现的问题列表

### 2.1 高优先级问题

#### P1: 侧边栏按钮缺少 Accessible Name
- **严重程度**: 高
- **影响范围**: 所有侧边栏按钮 (7 个)
- **问题描述**: 所有侧边栏导航按钮都没有 `aria-label` 属性，屏幕阅读器无法正确读取按钮功能
- **建议修复**:
  ```tsx
  // 在 sidebar.tsx 中为按钮添加 aria-label
  <Button
    aria-label={item.label}  // 添加这一行
    variant="ghost"
    ...
  >
  ```

#### P2: 按钮没有 data-testid
- **严重程度**: 高
- **影响范围**: 所有侧边栏按钮
- **问题描述**: 测试只能通过按钮索引定位，不稳定且容易出错
- **建议修复**:
  ```tsx
  // 添加 data-testid 属性
  <Button
    data-testid={`sidebar-${item.id}`}
    ...
  >
  ```

### 2.2 中优先级问题

#### P3: 按钮索引映射不一致
- **严重程度**: 中
- **影响范围**: 测试稳定性
- **问题描述**: MCP 浏览器和 Playwright 看到的按钮索引不一致
- **根本原因**: 存在隐藏的 generic 元素 (tooltips) 影响了按钮计数
- **当前状态**: 已通过调整索引映射临时修复

#### P4: 视图切换状态难以验证
- **严重程度**: 中
- **影响范围**: 视图切换测试
- **问题描述**: 代码视图和执行流视图没有明显的可测试标识
- **建议修复**: 添加 `data-view-mode` 属性到主容器

### 2.3 低优先级问题

#### P5: 模态对话框阻止点击
- **严重程度**: 低
- **影响范围**: 连续测试场景
- **问题描述**: 前一个测试打开的模态框可能阻止后续测试的点击操作
- **当前状态**: 已通过 `beforeEach` 添加 Escape 键处理修复

---

## 3. 测试工具覆盖情况

| 功能 | MCP Browser | Playwright | 覆盖状态 |
|-----|-------------|------------|---------|
| 侧边栏按钮点击 | ✓ | ✓ | 完全覆盖 |
| 视图模式切换 | ✓ | ✓ | 完全覆盖 |
| 右侧面板打开/关闭 | ✓ | ✓ | 完全覆盖 |
| 面板标签切换 | ✓ | ✓ | 完全覆盖 |
| 键盘快捷键 | ✓ | ✓ | 完全覆盖 |
| 文件对话框 (原生) | ✗ | ✗ | **需要桌面自动化** |
| 项目打开/分析 | ✗ | ✗ | **需要 Tauri API Mock** |

---

## 4. 需要新增的测试功能建议

### 4.1 UI 组件改进 (开发任务)

1. **添加 data-testid 属性**
   - 位置: `app/src/components/layout/sidebar.tsx`
   - 内容: 为所有侧边栏按钮添加 `data-testid`

2. **添加 aria-label 属性**
   - 位置: `app/src/components/layout/sidebar.tsx`
   - 内容: 为所有按钮添加 `aria-label` 以提高可访问性

3. **添加视图模式标识**
   - 位置: `app/src/components/layout/main-layout.tsx`
   - 内容: 在主容器添加 `data-view-mode="code"` 或 `data-view-mode="flow"`

### 4.2 测试工具改进

1. **Tauri API Mock**
   - 用途: 模拟文件对话框返回值
   - 实现: 创建 `app/tests/mocks/tauri-api.ts`

2. **测试数据准备脚本**
   - 用途: 创建测试项目目录和文件
   - 实现: 创建 `app/tests/fixtures/` 目录

3. **视觉回归测试**
   - 用途: 检测 UI 变化
   - 工具: Playwright 内置的 `toHaveScreenshot()`

---

## 5. 测试用例清单

### 已实现的测试用例

| 文件 | 测试名称 | 状态 |
|-----|---------|------|
| `sidebar-panels.spec.ts` | 默认显示代码视图 | ✓ 通过 |
| `sidebar-panels.spec.ts` | 点击执行流按钮切换视图 | ✓ 通过 |
| `sidebar-panels.spec.ts` | 点击项目按钮切换回代码视图 | ✗ 失败 |
| `sidebar-panels.spec.ts` | 点击文件按钮打开文件浏览器 | ✗ 失败 |
| `sidebar-panels.spec.ts` | 文件按钮点击切换面板 | ✓ 通过 |
| `sidebar-panels.spec.ts` | 右侧面板标签切换 | ✓ 通过 |
| `sidebar-panels.spec.ts` | 点击大纲按钮打开大纲面板 | ✗ 失败 |
| `sidebar-panels.spec.ts` | 大纲面板显示空状态提示 | ✓ 通过 |
| `sidebar-panels.spec.ts` | 大纲面板有搜索输入框 | ✗ 失败 |
| `sidebar-panels.spec.ts` | Cmd/Ctrl + K 打开命令面板 | ✓ 通过 |
| `sidebar-panels.spec.ts` | Cmd/Ctrl + B 切换侧边栏 | ✓ 通过 |
| `sidebar-panels.spec.ts` | Cmd/Ctrl + J 切换底部面板 | ✓ 通过 |
| `sidebar-panels.spec.ts` | 侧边栏按钮应该有 tooltip | ✓ 通过 |
| `sidebar-panels.spec.ts` | 检查侧边栏按钮缺少 accessible name | ✓ 通过 |

### 待实现的测试用例

| 优先级 | 测试名称 | 依赖 |
|-------|---------|------|
| 高 | 打开项目并显示文件树 | Tauri API Mock |
| 高 | 选择文件并显示大纲 | 测试数据 |
| 高 | 分析函数并显示执行流 | 后端 API |
| 中 | 节点点击显示详情 | 执行流数据 |
| 中 | 搜索符号功能 | 项目数据 |
| 低 | 主题切换 | 无 |
| 低 | 面板拖拽调整大小 | 无 |

---

## 6. 运行测试命令

```bash
# 确保开发服务器运行
cd app && pnpm tauri dev

# 运行侧边栏和面板测试
cd app && npx playwright test tests/desktop/sidebar-panels.spec.ts \
  --config=tests/desktop/playwright.config.ts \
  --reporter=list

# 运行所有桌面测试
cd app && npx playwright test tests/desktop/ \
  --config=tests/desktop/playwright.config.ts

# 生成 HTML 报告
cd app && npx playwright test tests/desktop/ \
  --config=tests/desktop/playwright.config.ts \
  --reporter=html
```

---

## 7. 后续行动项

### 开发团队 (UI-Dev)

- [ ] 为侧边栏按钮添加 `data-testid` 属性
- [ ] 为侧边栏按钮添加 `aria-label` 属性
- [ ] 添加视图模式标识到主容器

### 测试团队 (E2E-Tester)

- [ ] 创建 Tauri API Mock
- [ ] 准备测试数据 fixtures
- [ ] 实现打开项目测试用例
- [ ] 添加视觉回归测试

---

*报告生成时间: 2026-01-30 21:33 CST*
