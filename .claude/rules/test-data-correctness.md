# 测试数据正确性规则

> **重要**: 此规则强制所有测试 Agent 验证数据的实际显示，而不仅仅是 UI 元素的存在。

## 背景

2024-02 发现的问题：
- FlowExportPanel 弹窗打开了，但内容为空
- E2E 测试只验证了"弹窗打开"，没有验证"数据显示"
- Mock 测试全部通过，但实际功能不工作

## 强制规则

### 1. 每个 UI 测试必须包含数据正确性断言

```typescript
// ❌ 错误示例 - 只验证 UI 存在
test('弹窗打开', async ({ page }) => {
  await button.click();
  const panel = page.locator('.panel');
  expect(await panel.isVisible()).toBe(true); // 只验证可见性
});

// ✅ 正确示例 - 验证数据实际显示
test('弹窗打开并显示数据', async ({ page }) => {
  await button.click();
  const panel = page.locator('.panel');
  expect(await panel.isVisible()).toBe(true);
  
  // 关键：验证内容不为空
  const content = await panel.locator('.content').textContent();
  expect(content?.length).toBeGreaterThan(0);
  
  // 关键：验证关键数据元素存在
  const dataItems = page.locator('.data-item');
  expect(await dataItems.count()).toBeGreaterThan(0);
});
```

### 2. Mock 数据必须覆盖所有字段

```typescript
// ❌ 错误 - 部分 Mock
const mockData = { name: 'test' };

// ✅ 正确 - 完整 Mock 包含所有预期字段
const mockData = {
  name: 'test',
  content: 'actual content here',
  items: [{ id: 1, label: 'item' }],
  stats: { count: 5 }
};
```

### 3. 空状态必须有明确提示

```typescript
// ❌ 错误 - 空状态返回空白
if (!data) return null;

// ✅ 正确 - 空状态显示提示信息
if (!data) {
  return (
    <div className="empty-state">
      <Icon />
      <p>暂无数据，请先选择文件</p>
    </div>
  );
}
```

### 4. 测试必须验证错误处理

```typescript
test('API 失败时显示错误提示', async ({ page }) => {
  // 模拟 API 失败
  await page.route('**/api/**', route => route.abort());
  
  await page.goto('/');
  await button.click();
  
  // 验证错误提示显示
  const errorMsg = page.locator('.error-message, [role="alert"]');
  expect(await errorMsg.isVisible()).toBe(true);
  expect(await errorMsg.textContent()).toContain('失败');
});
```

## Agent 协作规则

### E2E-Tester 必须

1. **数据验证**：每个测试用例必须验证数据实际显示
2. **空状态测试**：测试无数据时的 UI 状态
3. **错误处理测试**：测试 API 失败的处理
4. **截图比对**：关键功能截图存档

### Unit-Tester 必须

1. **边界测试**：测试 null/undefined 输入
2. **Mock 完整性**：确保 Mock 数据结构完整
3. **返回值验证**：验证函数返回值的结构和内容

### UI-Dev 必须

1. **空状态设计**：所有组件必须设计空状态 UI
2. **错误状态设计**：所有 API 调用必须有错误处理 UI
3. **加载状态设计**：异步操作必须有加载指示

### Debug-Dev 必须

1. **日志添加**：关键数据流添加 console.log
2. **断点调试**：复杂逻辑支持 debugger
3. **错误边界**：React 组件使用 ErrorBoundary

## 测试检查清单

每个新功能的测试必须覆盖：

- [ ] UI 元素可见性
- [ ] **数据实际显示**（内容不为空）
- [ ] 加载状态显示
- [ ] 错误状态处理
- [ ] 空状态提示
- [ ] 交互响应（点击、输入）
- [ ] 键盘快捷键
- [ ] 截图存档

## 自动化检测

在 CI 中添加以下检查：

```yaml
- name: Test Data Correctness
  run: |
    # 检查测试文件是否包含数据正确性断言
    grep -r "textContent\|toBeGreaterThan\|not.toBeEmpty" tests/ || echo "⚠️ 警告：缺少数据正确性断言"
```
