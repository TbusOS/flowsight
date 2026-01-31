# 测试质量评审报告

> **评审日期**: 2026-01-29
> **评审范围**: `app/tests/desktop/*.spec.ts`
> **评审标准**: `.claude/agents/test-reviewer.md`

## 总体评估

### 当前状态

| 指标 | 当前值 | 目标值 | 评价 |
|------|--------|--------|------|
| 测试数量 | 174 | - | ✅ 数量充足 |
| 通过率 | 100% | 100% | ⚠️ 可能是假阳性 |
| 功能覆盖 | ~30% | 80% | 🔴 严重不足 |
| 断言质量 | 2/5 | 4/5 | 🟡 需要加强 |
| 真实性 | 1/5 | 3/5 | 🔴 大量假测试 |

### 核心问题

> **测试通过 ≠ 功能正常**
> 
> 当前 174 个测试中，约 80% 是"存在性测试"，只验证 UI 元素是否存在，
> 而不验证功能是否真正工作。这导致测试全部通过，但实际功能可能完全不可用。

---

## 🔴 严重问题

### 1. 存在性测试占比过高

**现状**: 大部分测试只检查 `isVisible()`

```typescript
// 典型的"假测试"
test('节点详情面板显示', async ({ page }) => {
  const panel = page.locator('[data-testid="detail-panel"]');
  await expect(panel).toBeVisible();  // 只证明 DOM 存在
  // 没有验证: 数据是否正确、交互是否生效
});
```

**问题**:
- ✅ 测试通过
- ❌ 面板可能显示错误数据
- ❌ 面板可能没有响应点击
- ❌ 面板可能显示空白

### 2. Mock 过度导致测试失真

**现状**: 几乎所有 Tauri API 都被 Mock，返回固定假数据

```typescript
// Mock 返回固定数据
window.__TAURI__.core.invoke = async (cmd) => {
  if (cmd === 'build_execution_flow') {
    return { nodes: [固定数据], edges: [] };  // 永远成功
  }
};
```

**问题**:
- 测试的是 Mock，不是真正的功能
- 后端可能有 Bug，前端测试永远不会发现
- 数据格式变更不会被检测到

### 3. 关键用户流程未验证

**缺失的核心功能测试**:

| 功能 | 当前测试 | 真正需要测试的 |
|------|----------|----------------|
| 执行流分析 | 按钮存在 ✅ | 分析结果正确 ❌ |
| 节点点击 | 面板可见 ✅ | 跳转到代码行 ❌ |
| 文件保存 | 无 ❌ | 保存后内容正确 ❌ |
| 搜索功能 | 输入框存在 ✅ | 搜索结果正确 ❌ |

---

## 🟡 中等问题

### 4. 缺少状态变化验证

```typescript
// 当前: 只点击不验证
test('点击分析按钮', async ({ page }) => {
  await page.locator('button:has-text("分析")').click();
  // 然后呢？没有然后了
});

// 应该: 验证状态变化
test('点击分析按钮后显示执行流', async ({ page }) => {
  // 验证初始状态
  await expect(page.locator('.empty-state')).toBeVisible();
  
  // 执行操作
  await page.locator('button:has-text("分析")').click();
  
  // 验证加载中
  await expect(page.locator('.loading')).toBeVisible();
  
  // 验证最终状态
  await expect(page.locator('.empty-state')).not.toBeVisible();
  await expect(page.locator('.react-flow__node')).toHaveCount({ min: 1 });
});
```

### 5. 缺少数据正确性验证

```typescript
// 当前: 不验证数据
test('显示函数列表', async ({ page }) => {
  const list = page.locator('.function-list');
  await expect(list).toBeVisible();
});

// 应该: 验证数据正确
test('大纲显示当前文件的函数', async ({ page }) => {
  await selectFile('driver.c');
  
  const outline = page.locator('[data-testid="outline-panel"]');
  // 验证显示的函数来自正确的文件
  await expect(outline).toContainText('init_driver');
  await expect(outline).toContainText('probe');
  // 验证不显示其他文件的函数
  await expect(outline).not.toContainText('unrelated_function');
});
```

### 6. 缺少错误处理测试

**完全没有测试的场景**:
- 文件读取失败
- 分析超时
- 网络错误
- 无效数据格式
- 权限不足

---

## 具体文件评审

### `node-detail.spec.ts` (12 测试)

| 测试名称 | 类型 | 问题 |
|----------|------|------|
| 空状态显示正确提示 | 存在性 | ✅ OK |
| detail-panel 元素存在 | 存在性 | ⚠️ 没有验证内容 |
| 函数名称正确显示 | 存在性 | ⚠️ 没有验证具体值 |
| 调用列表元素存在 | 存在性 | ⚠️ 没有验证数据 |
| ... | ... | ... |

**评分**: 2/5

**缺失**:
- [ ] 点击调用函数跳转到该函数
- [ ] 点击文件位置跳转到代码行
- [ ] 查看调用链按钮真正生成调用链
- [ ] LLVM IR 按钮显示正确的 IR

### `flow-interaction.spec.ts` (9 测试)

| 测试名称 | 类型 | 问题 |
|----------|------|------|
| 切换到执行流视图 | 存在性 | ✅ OK |
| 执行流视图显示工具栏 | 存在性 | ⚠️ 没有验证工具栏功能 |
| 点击分析按钮触发流程构建 | 半功能 | ⚠️ 没有验证结果 |
| ReactFlow 画布存在 | 存在性 | ⚠️ 没有验证节点内容 |

**评分**: 2/5

**缺失**:
- [ ] 分析后节点数量正确
- [ ] 节点名称与源代码匹配
- [ ] 边连接正确的节点
- [ ] 缩放/平移功能正常
- [ ] 节点点击选中并显示详情

### `workflow.spec.ts` (10 测试)

**评分**: 2/5

**缺失**:
- [ ] 完整的打开项目 → 分析 → 查看详情流程
- [ ] 文件修改 → 保存 → 重新分析流程
- [ ] 搜索 → 跳转 → 返回流程

---

## 改进方案

### 方案 1: 功能性测试改造

将现有存在性测试升级为功能性测试：

```typescript
// 改造前
test('节点详情面板显示', async ({ page }) => {
  await expect(page.locator('[data-testid="detail-panel"]')).toBeVisible();
});

// 改造后
test('点击执行流节点后详情面板显示该节点信息', async ({ page }) => {
  // 1. 打开项目并分析
  await openProject('fixtures/simple_driver.c');
  await clickAnalyze();
  await waitForFlowRender();
  
  // 2. 点击一个节点
  const node = page.locator('.react-flow__node:has-text("init_driver")');
  await node.click();
  
  // 3. 验证详情面板显示正确数据
  const panel = page.locator('[data-testid="detail-panel"]');
  await expect(panel.locator('.function-name')).toHaveText('init_driver');
  await expect(panel.locator('.return-type')).toHaveText('int');
  await expect(panel.locator('.file-path')).toContainText('simple_driver.c');
  await expect(panel.locator('.line-number')).toHaveText('42');
  
  // 4. 验证调用列表
  const calls = panel.locator('.calls-list .call-item');
  await expect(calls).toHaveCount(3);
  await expect(calls.nth(0)).toContainText('register_device');
});
```

### 方案 2: 真实 Fixture 测试

使用真实的测试文件而不是 Mock 数据：

```typescript
// 使用真实的 C 代码文件进行测试
const FIXTURE_PATH = 'tests/fixtures/simple_driver.c';

test('分析真实 C 代码', async ({ page }) => {
  // 使用真实文件
  await openProject(FIXTURE_PATH);
  await clickAnalyze();
  
  // 验证分析结果与源代码匹配
  // (需要预先分析 simple_driver.c 知道预期结果)
  await expect(page.locator('.react-flow__node')).toHaveCount(5);
  await expect(page.locator('text=init_driver')).toBeVisible();
  await expect(page.locator('text=probe')).toBeVisible();
});
```

### 方案 3: 端到端数据验证

```typescript
test('执行流数据正确性', async ({ page }) => {
  await openProject('fixtures/simple_driver.c');
  await clickAnalyze();
  
  // 获取执行流数据
  const flowData = await page.evaluate(() => {
    return window.__flowData;  // 需要在组件中暴露
  });
  
  // 验证数据结构
  expect(flowData.nodes).toHaveLength(5);
  expect(flowData.edges).toHaveLength(4);
  
  // 验证特定节点
  const initNode = flowData.nodes.find(n => n.data.name === 'init_driver');
  expect(initNode).toBeDefined();
  expect(initNode.data.calls).toContain('register_device');
});
```

---

## 优先级行动计划

### 本周 (高优先级)

1. **创建功能性测试框架**
   - 添加真实 fixture 文件
   - 创建测试工具函数
   - 定义预期结果

2. **改造核心测试**
   - `flow-interaction.spec.ts` → 验证分析结果
   - `node-detail.spec.ts` → 验证数据正确性
   - `workflow.spec.ts` → 完整流程验证

### 下周 (中优先级)

3. **添加错误处理测试**
   - 文件不存在
   - 分析失败
   - 网络超时

4. **添加性能测试**
   - 大文件加载时间
   - 渲染性能

### 持续 (低优先级)

5. **添加视觉回归测试**
6. **添加可访问性测试**

---

## 总结

**当前测试的本质问题**: 测试的是"UI 组件是否渲染"，而不是"功能是否正常工作"。

**解决方案核心**: 从"存在性验证"转向"功能性验证"，测试真实的用户操作和数据流。

**建议**: 将测试数量从 174 减少到 50 个高质量测试，比 174 个假测试更有价值。
