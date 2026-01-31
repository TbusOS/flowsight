# 测试失效复盘报告

> **日期**: 2024-01
> **问题**: 多个严重 Bug 未被测试发现
> **影响**: 执行流功能完全不可用

## 问题清单

| Bug | 严重程度 | 测试员 | 评审员 | 桌面工具 |
|-----|---------|-------|-------|---------|
| 执行流只有1个节点 | 🔴 严重 | ❌ | ❌ | ❌ |
| 参数命名不匹配 | 🔴 严重 | ❌ | ❌ | ❌ |
| 索引完成事件不处理 | 🔴 严重 | ❌ | ❌ | ❌ |
| 缺少ftrace格式 | 🟡 功能 | ❌ | ❌ | ❌ |

## 根本原因分析

### 1. 测试设计缺陷

**问题**: 测试只验证"存在性"，不验证"正确性"

```typescript
// ❌ 当前测试 - 只检查元素存在
test('执行流显示', async () => {
  await expect(page.locator('.react-flow')).toBeVisible();
  // 通过了！但节点可能只有1个
});

// ✅ 应该的测试 - 验证业务正确性
test('执行流分析结果正确', async () => {
  // 打开真实内核文件
  await openRealFile('/linux/drivers/gpio/gpio-dwapb.c');
  await clickAnalyze('dwapb_gpio_probe');
  
  // 验证节点数量
  const nodeCount = await page.locator('.react-flow__node').count();
  expect(nodeCount).toBeGreaterThan(5);  // 不只是1个！
  
  // 验证包含预期的子函数
  await expect(page.locator('text=dwapb_gpio_add_port')).toBeVisible();
});
```

### 2. Mock 数据设计缺陷

**问题**: Mock 返回的永远是"完美"的假数据

```typescript
// ❌ 当前 Mock - 永远返回完美数据
case 'build_execution_flow': {
  return {
    nodes: [
      { id: '1', label: 'main', ... },
      { id: '2', label: 'init', ... },  // 永远有2个节点
    ],
    edges: [{ source: '1', target: '2' }]
  };
}

// 真实情况: 参数错误 → 返回只有入口函数的1个节点
```

### 3. 缺少需求追踪

**问题**: 没有"需求清单"来验证功能完整性

| 需求 | 测试用例 | 状态 |
|------|---------|------|
| 执行流显示多个节点 | ❓ 无 | 未测试 |
| ftrace 格式显示 | ❓ 无 | 未测试 |
| 执行上下文标签 | ❓ 无 | 未测试 |

### 4. 测试评审标准不完善

**问题**: 评审只看代码质量，不看业务覆盖

```markdown
## 当前评审标准
- ✅ 代码风格
- ✅ 断言数量
- ❌ 业务覆盖度 (缺失!)
- ❌ Mock 真实性 (缺失!)
- ❌ 边界条件 (缺失!)
```

## 改进方案

### 1. 业务正确性测试 (必须)

每个功能必须有验证业务正确性的测试：

```typescript
// 新测试规范：必须验证实际结果
test('执行流分析必须包含调用关系', async () => {
  // 使用真实文件，不用 Mock
  const result = await invokeReal('build_execution_flow', {
    filePath: '/linux/drivers/gpio/gpio-dwapb.c',
    entryFunction: 'dwapb_gpio_probe'
  });
  
  // 验证节点数量 > 1
  expect(result.nodes.length).toBeGreaterThan(1);
  
  // 验证有调用边
  expect(result.edges.length).toBeGreaterThan(0);
  
  // 验证包含预期的被调函数
  const labels = result.nodes.map(n => n.label);
  expect(labels).toContain('dwapb_gpio_add_port');
});
```

### 2. 真实后端测试层 (必须)

```bash
# 测试层级（全部必须通过）
层级1: 真实后端 CLI 测试  → 验证 Rust 分析正确
层级2: Tauri 集成测试    → 验证前后端通信
层级3: UI 功能测试       → 验证界面交互
层级4: 视觉回归测试      → 验证显示正确
```

### 3. 需求-测试追踪矩阵

| 需求ID | 需求描述 | 测试用例 | 状态 |
|--------|---------|---------|------|
| F-001 | 执行流显示 > 1 个节点 | `test_flow_multiple_nodes` | ✅ |
| F-002 | ftrace 格式显示 | `test_ftrace_format` | ✅ |
| F-003 | 树形视图显示 | `test_tree_view` | ✅ |
| F-004 | 执行上下文标签 | `test_context_labels` | ✅ |

### 4. 改进的测试评审标准

```markdown
## 新评审标准

### 必须检查项
- [ ] **业务正确性**: 测试是否验证了实际业务结果？
- [ ] **Mock 真实性**: Mock 数据是否反映真实场景？
- [ ] **边界条件**: 是否测试了空数据、单节点、大量节点？
- [ ] **错误处理**: 是否测试了后端错误情况？
- [ ] **真实后端**: 是否有不使用 Mock 的测试？

### 禁止的模式
- ❌ 只检查 `.toBeVisible()` 不检查内容
- ❌ Mock 返回完美数据不模拟错误
- ❌ 没有数量/内容断言
```

## 行动计划

### 立即执行
1. ✅ 添加真实后端测试 (`real-backend-test.ts`)
2. ✅ 添加契约验证 (`contract-validator.ts`)
3. ⬜ 添加业务正确性测试
4. ⬜ 添加视觉回归测试

### 短期改进
1. ⬜ 创建需求-测试追踪矩阵
2. ⬜ 更新测试评审标准
3. ⬜ 添加 CI 强制运行真实后端测试

### 长期改进
1. ⬜ 自动化需求覆盖检查
2. ⬜ Mock 与真实后端对比测试
3. ⬜ 性能基准测试

## 教训总结

1. **Mock 测试 ≠ 真实测试**: Mock 通过不代表功能正常
2. **存在性测试 ≠ 正确性测试**: 元素存在不代表内容正确
3. **代码质量 ≠ 业务覆盖**: 代码好不代表功能全
4. **测试通过 ≠ 功能可用**: 必须有真实后端验证
