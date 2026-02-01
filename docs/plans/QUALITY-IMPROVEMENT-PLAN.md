# FlowSight 质量改进计划

> **目标**: 打造完美的 IDE 体验
> **日期**: 2024-01
> **状态**: 进行中

## 问题总览

### 评分概览

| 维度 | 当前评分 | 目标评分 | 差距 |
|------|---------|---------|------|
| UI 交互 | 6/10 | 9/10 | 🔴 |
| 渲染性能 | 5/10 | 9/10 | 🔴 |
| 加载速度 | 5/10 | 9/10 | 🔴 |
| 显示效果 | 6/10 | 9/10 | 🟡 |
| 界面布局 | 7/10 | 9/10 | 🟡 |
| 测试方案 | 7/10 | 9/10 | 🟡 |
| 测试框架 | 6/10 | 9/10 | 🔴 |
| 评审机制 | 8/10 | 9/10 | 🟢 |

---

## 一、UI 交互改进

### 1.1 错误处理 (P0)

**问题**: 部分错误只 console.error，用户看不到

```typescript
// ❌ 当前 - 用户看不到错误
} catch (error) {
  console.error("加载失败:", error)
}

// ✅ 改进 - 显示错误UI + 重试按钮
} catch (error) {
  setError(error.message)
  toast.error("加载失败", { action: { label: "重试", onClick: retry } })
}
```

**需要修改的文件**:
- [ ] `app/src/components/panels/file-explorer.tsx:186`
- [ ] `app/src/components/panels/flow-view.tsx:200`
- [ ] `app/src/components/panels/code-editor.tsx:150`

**新增组件**:
- [ ] `app/src/components/ErrorBoundary/ErrorBoundary.tsx` - 全局错误边界

### 1.2 Loading 状态 (P1)

**问题**: 只有 spinner，缺少骨架屏

**需要新增**:
- [ ] `app/src/components/Skeleton/Skeleton.tsx` - 通用骨架屏组件
- [ ] `app/src/components/Skeleton/FileTreeSkeleton.tsx`
- [ ] `app/src/components/Skeleton/FlowViewSkeleton.tsx`
- [ ] `app/src/components/Skeleton/EditorSkeleton.tsx`

### 1.3 反馈动画 (P1)

**问题**: 缺少操作成功反馈和过渡动画

**需要添加**:
- [ ] 保存成功 Toast 提示
- [ ] 面板展开/收起动画
- [ ] 节点高亮过渡效果
- [ ] 加载进度条（文件索引）

### 1.4 键盘快捷键 (P1)

**问题**: 文档与实现不一致，部分快捷键缺失

| 快捷键 | 文档 | 实现 | 状态 |
|--------|------|------|------|
| `Ctrl+W` | 关闭标签 | ❌ | 需实现 |
| `Ctrl+B` | 切换侧边栏 | ⚠️ | 部分实现 |
| `Ctrl+\`` | 切换终端 | ❌ | 需实现 |
| `Ctrl+Shift+P` | 命令面板 | ✅ | 已实现 |

---

## 二、渲染性能改进

### 2.1 虚拟化列表 (P0) 🔴

**问题**: 文件树、搜索结果未虚拟化，大目录卡顿

**解决方案**:
```bash
pnpm add react-window react-window-infinite-loader
```

**需要修改的文件**:
- [ ] `app/src/components/Explorer/FileTree.tsx` - 使用 FixedSizeList
- [ ] `app/src/components/panels/search-panel.tsx` - 使用 VariableSizeList
- [ ] `app/src/components/CallersView/CallersView.tsx` - 使用 FixedSizeList

### 2.2 React.memo 优化 (P1)

**问题**: 大部分组件未 memo 化

**需要优化的组件**:
```typescript
// 优先级 P0 - 频繁渲染的组件
- [ ] FileTreeItem
- [ ] FunctionNode (FlowView)
- [ ] SearchResultItem
- [ ] OutlineItem

// 优先级 P1 - 中等频率组件
- [ ] NodeDetailPanel
- [ ] CallersView
- [ ] TerminalPanel
```

### 2.3 代码分割 (P0) 🔴

**问题**: 所有组件打包在一个 bundle

**需要懒加载的组件**:
```typescript
// app/src/App.tsx
const FlowView = lazy(() => import('./components/panels/flow-view'));
const CodeEditor = lazy(() => import('./components/panels/code-editor'));
const LlvmIrPanel = lazy(() => import('./components/LlvmIrPanel/LlvmIrPanel'));
const SearchPanel = lazy(() => import('./components/panels/search-panel'));
```

---

## 三、加载速度改进

### 3.1 资源预加载 (P1)

**需要添加**:
```html
<!-- index.html -->
<link rel="preload" href="/fonts/inter.woff2" as="font" crossorigin>
<link rel="preload" href="/icons/sprite.svg" as="image">
```

### 3.2 字体优化 (P2)

```css
/* 添加 font-display */
@font-face {
  font-family: 'Inter';
  font-display: swap;
  /* ... */
}
```

### 3.3 图片优化 (P2)

- [ ] 使用 WebP 格式
- [ ] 添加图片懒加载
- [ ] 使用 SVG sprite 替代多个图标文件

---

## 四、显示效果改进

### 4.1 响应式设计 (P1) 🟡

**问题**: 无移动端/平板适配

**需要添加**:
```css
/* app/src/styles/globals.css */
@media (max-width: 768px) {
  .sidebar { display: none; }
  .main-panel { width: 100%; }
}

@media (max-width: 1024px) {
  .right-panel { display: none; }
}
```

### 4.2 无障碍性 (P1)

**问题**: 部分按钮缺少 aria-label

**需要修改的组件**:
- [ ] 所有 IconButton 添加 `aria-label`
- [ ] 添加 `role` 属性到自定义组件
- [ ] 验证颜色对比度 (WCAG AA)

### 4.3 主题切换 (P2)

**问题**: 主题切换可能有闪烁

**解决方案**:
```css
/* 添加过渡动画 */
:root {
  --transition-theme: color 0.2s, background-color 0.2s;
}
```

---

## 五、后端性能改进

### 5.1 并发优化 (P0) 🔴

**问题**: 索引使用 Mutex，并发瓶颈

```rust
// ❌ 当前
static INDEX: Lazy<Mutex<SymbolIndex>> = ...

// ✅ 改进
static INDEX: Lazy<RwLock<SymbolIndex>> = ...
// 或使用 DashMap
static INDEX: Lazy<DashMap<String, FunctionDef>> = ...
```

### 5.2 异步化 (P0)

**问题**: Tauri 命令标记为 async 但实际同步

```rust
// ✅ 改进 - 使用 spawn_blocking
pub async fn analyze_file(path: String) -> Result<AnalysisResult, String> {
    let path = PathBuf::from(&path);
    tokio::task::spawn_blocking(move || {
        // CPU 密集型操作
    }).await.map_err(|e| e.to_string())?
}
```

### 5.3 内存优化 (P1)

**问题**: 大量不必要的 clone()

```rust
// ❌ 当前
return Ok((*cached).clone());

// ✅ 改进 - 返回 Arc
return Ok(Arc::clone(&cached));
```

### 5.4 缓存增强 (P1)

- [ ] 动态缓存容量（根据内存调整）
- [ ] 使用 DashMap 替代 RwLock<HashMap>
- [ ] 添加持久化缓存（sled）

---

## 六、测试框架改进

### 6.1 视觉回归测试 (P0) 🔴

**当前状态**: ❌ 缺失

**实现方案**:
```typescript
// app/tests/visual/visual-regression.spec.ts
import { test, expect } from '@playwright/test';

test('执行流视图视觉回归', async ({ page }) => {
  await page.goto('/');
  // ... 操作
  await expect(page).toHaveScreenshot('flow-view.png', {
    threshold: 0.2,
  });
});
```

**需要创建**:
- [ ] `app/tests/visual/visual-regression.spec.ts`
- [ ] `app/tests/visual/baseline/` - 基线图片目录
- [ ] CI 集成视觉回归检测

### 6.2 性能基准测试 (P0) 🔴

**当前状态**: ❌ 缺失

**Rust 基准测试**:
```rust
// crates/flowsight-analysis/benches/parsing.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_parse_large_file(c: &mut Criterion) {
    c.bench_function("parse_10000_lines", |b| {
        b.iter(|| parser.parse(&large_source))
    });
}

criterion_group!(benches, bench_parse_large_file);
criterion_main!(benches);
```

**前端性能测试**:
```typescript
// app/tests/performance/lighthouse.spec.ts
import lighthouse from 'lighthouse';

test('性能评分 > 90', async () => {
  const result = await lighthouse(url);
  expect(result.lhr.categories.performance.score).toBeGreaterThan(0.9);
});
```

### 6.3 大规模测试 (P1)

**当前状态**: ⚠️ 只测试小文件

**需要添加**:
- [ ] 10000+ 行文件解析测试
- [ ] 1000+ 文件项目索引测试
- [ ] 长时间运行稳定性测试（内存泄漏检测）

### 6.4 可访问性测试 (P1)

```typescript
// app/tests/a11y/accessibility.spec.ts
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('无障碍性检查', async ({ page }) => {
  await page.goto('/');
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});
```

### 6.5 单元测试覆盖 (P1)

**当前状态**: 99% 组件缺少单元测试

**需要添加测试的组件**:
```
优先级 P0:
- [ ] FlowView
- [ ] CodeEditor
- [ ] FileTree

优先级 P1:
- [ ] NodeDetailPanel
- [ ] SearchPanel
- [ ] OutlinePanel
```

---

## 七、测试评审改进

### 7.1 评审标准完善

**新增评审项**:
- [ ] 视觉回归检查
- [ ] 性能基准检查
- [ ] 大规模场景覆盖

### 7.2 自动化评审

**CI 集成**:
```yaml
# .github/workflows/test.yml
- name: Visual Regression
  run: pnpm test:visual

- name: Performance Benchmark
  run: cargo bench

- name: Accessibility Check
  run: pnpm test:a11y
```

---

## 八、执行计划

### Phase 1: 核心性能 (Week 1-2)

| 任务 | 优先级 | 状态 |
|------|--------|------|
| 虚拟化列表 | P0 | ✅ 组件已创建，待集成 |
| 代码分割 | P0 | ✅ 已完成 |
| 后端并发优化 | P0 | ✅ 已完成 (Mutex→RwLock) |
| 全局错误边界 | P0 | ✅ 已完成 |

### Phase 2: 测试完善 (Week 3-4)

| 任务 | 优先级 | 状态 |
|------|--------|------|
| 视觉回归测试 | P0 | ✅ 已完成 |
| 性能基准测试 | P0 | ✅ 已完成 |
| 单元测试补充 | P1 | ⏳ 待开发 |
| 大规模测试 | P1 | ⏳ 待开发 |

### Phase 3: 体验优化 (Week 5-6)

| 任务 | 优先级 | 状态 |
|------|--------|------|
| 骨架屏 | P1 | ✅ 组件已创建并集成 |
| 响应式设计 | P1 | ✅ 已完成 |
| 无障碍性 | P1 | ✅ 已完成 |
| 快捷键完善 | P1 | ⏳ 待开发 |

---

## 九、验收标准

### 性能指标

| 指标 | 目标 | 测试方法 |
|------|------|---------|
| 首屏加载 | < 2s | Lighthouse |
| 文件树渲染 (1000 项) | < 100ms | Performance API |
| 执行流分析 | < 500ms | Benchmark |
| 内存占用 | < 500MB | Memory Profiler |

### 测试指标

| 指标 | 目标 |
|------|------|
| 单元测试覆盖率 | > 80% |
| E2E 测试通过率 | 100% |
| 视觉回归通过率 | 100% |
| 性能基准无退化 | ✓ |

### 用户体验指标

| 指标 | 目标 |
|------|------|
| 错误恢复率 | 100% (所有错误可重试) |
| 快捷键覆盖 | 100% (文档一致) |
| 无障碍性 | WCAG AA 标准 |

---

## 十、相关文档

- [测试失效复盘](./TEST-FAILURE-POSTMORTEM.md)
- [测试计划](../testing/TEST-PLAN.md)
- [测试评审标准](../../.claude/agents/test-reviewer.md)
- [UI 优化计划](./ui-optimization.md)
