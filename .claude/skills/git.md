# /sc:git - Git 智能操作

> FlowSight Git 智能操作技能

## 自动触发条件

当任务描述包含以下关键词时，自动激活此技能：

| 触发关键词 | 说明 |
|-----------|------|
| "git", "提交" | Git 操作 |
| "commit", "push" | 版本控制 |
| "PR", "merge" | 代码合并 |

## 使用方式

```
/sc:git "提交更改"
/sc:git "创建 PR"
/sc:git "合并分支"
```

## Git 操作

### 提交更改

```bash
# 查看更改
git status
git diff

# 暂存文件
git add <files>

# 创建提交
git commit -m "feat: 添加新功能

- 详细描述
- 变更内容

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### 推送代码

```bash
# 推送到远程
git push origin <branch>

# 强制推送 (谨慎使用)
git push --force origin <branch>
```

### 分支操作

```bash
# 创建新分支
git checkout -b feature/new-feature

# 切换分支
git checkout <branch>

# 删除分支
git branch -d <branch>
```

## 提交规范

### 提交信息格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 类型标识

| 类型 | 说明 |
|------|------|
| feat | 新功能 |
| fix | 修复 bug |
| docs | 文档更新 |
| style | 代码格式 |
| refactor | 重构 |
| test | 测试 |
| chore | 构建/工具 |

### 示例

```
feat(analysis): 添加 LLVM IR 解析器

- 支持基本指令解析
- 支持类型定义
- 添加单元测试

Closes #123
```

## PR 创建

### PR 信息

- 标题清晰描述变更
- 详细说明变更内容
- 列出测试计划
- 关联相关 issue

### PR 模板

```markdown
## 变更描述

<!-- 描述变更内容 -->

## 测试计划

<!-- 测试步骤 -->

## 检查清单

- [ ] 代码审查通过
- [ ] 测试通过
- [ ] 文档已更新
```

## 与其他 Skills 配合

```
1. /sc:implement "实现功能"
2. /sc:test "运行测试"
3. /sc:build "构建验证"
4. /sc:git "提交代码"
```

---

**快捷命令**:

```
/sc:git "提交更改"    # Git 操作
```

---

> FlowSight 专用 - Git 智能操作
