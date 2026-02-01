# FlowSight 测试质量门禁

> **目标**: 确保 UI 显示的数据与后端解析结果一致，防止出现 "0 个文件, 0 个函数" 的问题

## 问题背景

2024-01 发现的问题：
- 打开真实内核目录后，UI 显示 "发现 0 个文件，0 个函数，0 个结构体"
- 执行流视图只显示一个孤立的 "probe" 节点

**根因分析**：
1. 所有测试都使用 Mock 数据，没有测试真实后端
2. 测试只验证 "存在性"，不验证 "正确性"
3. 依赖人工触发评审，没有自动化检查

## 解决方案：多层防护

```
┌─────────────────────────────────────────────────────────────┐
│                    测试质量门禁体系                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Layer 1: Rust 单元测试 (数据正确性)                          │
│  ════════════════════════════════════                       │
│  文件: crates/flowsight-analysis/tests/data_correctness_test.rs│
│  内容: 验证解析器返回正确的函数/结构体数量                      │
│  触发: cargo test, pre-commit, CI                           │
│                                                             │
│  Layer 2: 真实后端集成测试                                    │
│  ════════════════════════════════════                       │
│  文件: app/tests/integration/real-backend-test.ts           │
│  内容: 使用真实内核代码测试 CLI                               │
│  触发: pre-push, CI                                         │
│                                                             │
│  Layer 3: 测试质量检查                                       │
│  ════════════════════════════════════                       │
│  文件: app/tests/scripts/quality-check.ts                   │
│  内容: 检查测试文件的断言质量                                 │
│  触发: pre-commit, CI                                       │
│                                                             │
│  Layer 4: E2E 业务正确性测试                                 │
│  ════════════════════════════════════                       │
│  文件: app/tests/desktop/business-correctness.spec.ts       │
│  内容: 验证 Mock 返回的数据符合业务规则                       │
│  触发: CI                                                   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 质量门禁检查项

### 1. Rust 数据正确性 (必须通过)

```rust
// 🔴 关键测试示例
#[test]
fn test_parser_returns_nonzero_functions() {
    let result = parser.parse_source(GPIO_DRIVER, "gpio.c").unwrap();
    assert!(
        !result.functions.is_empty(),
        "🔴 解析器返回 0 个函数！这会导致 UI 显示问题"
    );
}
```

**检查项**:
- [ ] 解析器返回的函数数量 > 0
- [ ] 解析器返回的结构体数量 > 0
- [ ] 调用关系在源代码中真实存在
- [ ] 执行流树包含多个节点

### 2. 真实后端集成 (必须通过)

**检查项**:
- [ ] CLI 能正确解析真实内核代码
- [ ] 索引结果与实际文件数量一致
- [ ] 不能返回 "0 个文件, 0 个函数"
- [ ] 执行流深度 >= 2 层

### 3. 测试质量 (必须通过)

**阈值**:
- 弱断言比例 < 50%
- 每个测试至少 1 个强断言
- Mock 必须验证参数

**弱断言示例** (避免):
```typescript
expect(element).toBeVisible();      // 只检查存在
expect(data).toBeDefined();         // 不检查内容
expect(count).toBeGreaterThan(0);   // 阈值太低
```

**强断言示例** (推荐):
```typescript
expect(nodes.length).toBeGreaterThan(1);      // 具体数量
expect(text).toContain('dwapb_gpio_probe');   // 具体内容
expect(edges).toEqual(expectedEdges);          // 完整验证
```

### 4. Mock 参数验证 (必须通过)

```typescript
// Mock 必须验证参数名
const CONTRACTS = {
  'build_execution_flow': {
    required: ['filePath', 'entryFunction'],  // camelCase
  }
};

function validateContract(cmd, args) {
  const contract = CONTRACTS[cmd];
  for (const param of contract.required) {
    if (!(param in args)) {
      throw new Error(`[契约违规] 缺少参数 ${param}`);
    }
  }
}
```

## 运行测试

### 本地开发

```bash
# 安装 Git hooks (一次性)
./scripts/install-hooks.sh

# 手动运行所有检查
cargo test --package flowsight-analysis --test data_correctness_test
cd app && npx tsx tests/integration/real-backend-test.ts
cd app && npx tsx tests/scripts/quality-check.ts
```

### CI/CD

所有检查在以下情况自动运行:
- Push to main
- Pull Request to main

**CI 配置**: `.github/workflows/test.yml`

## 失败处理

### 如果 Rust 数据正确性测试失败

1. 检查 `crates/flowsight-parser/src/treesitter.rs`
2. 确认 tree-sitter 语法解析正确
3. 添加更多测试用例覆盖边界情况

### 如果真实后端集成测试失败

1. 检查 CLI 是否正确编译
2. 验证测试用的内核代码目录存在
3. 查看具体哪个断言失败

### 如果测试质量检查失败

1. 增加强断言
2. 减少弱断言
3. 添加 Mock 参数验证

## 相关文件

| 文件 | 用途 |
|------|------|
| `crates/flowsight-analysis/tests/data_correctness_test.rs` | Rust 数据正确性测试 |
| `app/tests/integration/real-backend-test.ts` | 真实后端集成测试 |
| `app/tests/scripts/quality-check.ts` | 测试质量检查脚本 |
| `app/tests/desktop/business-correctness.spec.ts` | 业务正确性测试 |
| `scripts/pre-commit` | Git pre-commit hook |
| `scripts/install-hooks.sh` | Hook 安装脚本 |
| `.github/workflows/test.yml` | CI 配置 |
| `.claude/agents/test-reviewer.md` | 测试评审规范 |

---

*最后更新: 2025-01*
