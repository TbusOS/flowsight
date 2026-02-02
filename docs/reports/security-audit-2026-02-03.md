# FlowSight 安全审计报告

> 日期: 2026-02-03
> 
> 审计人: 🔒 Security-Reviewer Agent

## 摘要

| 类别 | 严重漏洞 | 高危 | 中等 | 低危 | 警告 |
|------|---------|------|------|------|------|
| Rust (cargo) | 0 | 0 | 0 | 0 | 20 |
| Node.js (pnpm) | 0 | 0 | 1 | 0 | 0 |
| **总计** | **0** | **0** | **1** | **0** | **20** |

## Rust 依赖审计 (cargo audit)

### 状态: ✅ 无安全漏洞

发现 20 个维护警告，主要来自 Tauri 依赖的 GTK3 绑定：

| 包名 | 类型 | 说明 | 影响 |
|------|------|------|------|
| `atk` | unmaintained | gtk-rs GTK3 绑定不再维护 | 低（Tauri 依赖） |
| `gdk` | unmaintained | gtk-rs GTK3 绑定不再维护 | 低（Tauri 依赖） |
| `gtk` | unmaintained | gtk-rs GTK3 绑定不再维护 | 低（Tauri 依赖） |
| `fxhash` | unmaintained | 哈希库不再维护 | 低（sled 依赖） |
| `rustls-pemfile` | unmaintained | PEM 解析器不再维护 | 低（reqwest 依赖） |
| `unic-*` | unmaintained | Unicode 库不再维护 | 低（Tauri 依赖） |
| `glib` | unsound | VariantStrIter 迭代器问题 | 低（不直接使用） |

### 建议

1. **无需立即行动** - 所有警告都是第三方依赖的间接依赖
2. **监控 Tauri 更新** - 等待 Tauri 团队更新 GTK 绑定
3. **未来考虑** - 考虑用 `rustls` 替代依赖旧 PEM 库的 crates

## Node.js 依赖审计 (pnpm audit)

### 状态: ⚠️ 1 个中等漏洞

#### lodash-es 原型污染漏洞

| 属性 | 值 |
|------|-----|
| 包名 | `lodash-es` |
| 受影响版本 | `>=4.0.0 <=4.17.22` |
| 修复版本 | `>=4.17.23` |
| 严重程度 | 中等 (Moderate) |
| CVE | GHSA-xxjr-mmjv-4gpg |
| 来源 | `mermaid > @mermaid-js/parser > langium > chevrotain > lodash-es` |

### 影响分析

此漏洞位于深层依赖链中：
```
mermaid → @mermaid-js/parser → langium → chevrotain → lodash-es
```

**风险评估**: 低
- lodash-es 仅在 Mermaid 图表解析时使用
- FlowSight 不直接暴露 lodash 功能给用户输入
- 需要精心构造的输入才能触发漏洞

### 建议

1. **监控 mermaid 更新** - 等待 mermaid 升级 lodash-es 依赖
2. **无需立即行动** - 漏洞不在直接攻击面上
3. **记录风险** - 已记录，等待上游修复

## 构建验证

```
✅ cargo build --release --workspace: 成功 (2m 32s)
✅ 无编译错误
⚠️ 2 个编译警告 (未使用变量)
```

## 结论

FlowSight v0.2.0 安全状态: **可发布**

- 无严重或高危漏洞
- 中等漏洞风险已评估并确认为低风险
- 维护警告来自第三方依赖，不影响安全性

---

*报告生成: 2026-02-03*
*下次审计建议: 2026-03-03 或依赖更新后*
