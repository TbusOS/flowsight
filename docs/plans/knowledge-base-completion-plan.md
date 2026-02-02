# Linux 内核知识库完善计划

## 📋 目标

将已实现的 73 个知识库文件从当前的 **~40% 平均完整度** 提升到 **90%+ 完整度**。

---

## 🎯 完善标准

每个知识库文件必须包含：

| 组件 | 要求 | 权重 |
|------|------|------|
| **注册模式** | 所有注册/注销 API | 15% |
| **回调函数** | 所有回调 + context + can_sleep | 25% |
| **数据结构** | 核心结构体字段说明 | 15% |
| **调用链** | 至少 3 个关键调用链 | 20% |
| **辅助函数** | 常用 helper 函数 | 10% |
| **代码示例** | 至少 2 个完整示例 | 10% |
| **内核版本** | 标注适用版本 | 5% |

---

## 📊 优先级分类

### P0 - 核心子系统 (最高优先级)

| 文件 | 当前完整度 | 目标 | 负责 Agent |
|------|-----------|------|-----------|
| `core/memory.yaml` | 30% | 90% | KB-Memory |
| `core/sched.yaml` | 40% | 90% | KB-Sched |
| `core/irq.yaml` | 60% | 95% | KB-IRQ |
| `core/vfs.yaml` | 50% | 90% | KB-FS |
| `fs/vfs_ops.yaml` | 50% | 90% | KB-FS |

### P1 - 重要子系统

| 文件 | 当前完整度 | 目标 | 负责 Agent |
|------|-----------|------|-----------|
| `core/workqueue.yaml` | 80% | 95% | KB-Core |
| `core/timer.yaml` | 70% | 90% | KB-Core |
| `core/rcu.yaml` | 60% | 90% | KB-Sync |
| `core/softirq.yaml` | 70% | 90% | KB-IRQ |
| `core/kthread.yaml` | 70% | 90% | KB-Core |
| `core/security.yaml` | 70% | 90% | KB-Security |
| `core/tracing.yaml` | 70% | 90% | KB-Debug |
| `core/ipc.yaml` | 75% | 90% | KB-Core |
| `core/cgroup.yaml` | 70% | 90% | KB-Core |

### P2 - 驱动框架 (42 个)

| 分组 | 文件数 | 负责 Agent |
|------|--------|-----------|
| 总线驱动 | 8 | KB-Bus |
| 输入/输出 | 10 | KB-IO |
| 电源管理 | 6 | KB-Power |
| 存储 | 5 | KB-Storage |
| 多媒体 | 5 | KB-Media |
| 其他 | 8 | KB-Misc |

### P3 - 网络子系统

| 文件 | 当前完整度 | 目标 | 负责 Agent |
|------|-----------|------|-----------|
| `net/netdev.yaml` | 60% | 90% | KB-Net |
| `net/socket.yaml` | 50% | 90% | KB-Net |
| `net/netfilter.yaml` | 60% | 90% | KB-Net |

### P4 - 同步原语

| 文件 | 当前完整度 | 目标 | 负责 Agent |
|------|-----------|------|-----------|
| `sync/locking.yaml` | 70% | 95% | KB-Sync |
| `sync/completion.yaml` | 80% | 95% | KB-Sync |
| `sync/rwlock.yaml` | 80% | 95% | KB-Sync |
| `sync/semaphore.yaml` | 80% | 95% | KB-Sync |

---

## 🤖 Agent 团队

### 开发 Agents (7 个)

| Agent | 职责 | 技能要求 |
|-------|------|----------|
| **KB-Memory** | 内存管理知识库 | mm/, 页表, 回收, OOM |
| **KB-Sched** | 调度器知识库 | sched/, CFS, 负载均衡 |
| **KB-IRQ** | 中断知识库 | irq/, softirq, tasklet |
| **KB-FS** | 文件系统知识库 | fs/, VFS, 页缓存 |
| **KB-Net** | 网络知识库 | net/, socket, netfilter |
| **KB-Drivers** | 驱动框架知识库 | drivers/ 各子系统 |
| **KB-Sync** | 同步原语知识库 | locking, RCU, 屏障 |

### 审核 Agent (1 个)

| Agent | 职责 |
|-------|------|
| **KB-Reviewer** | 审核知识库完整性、准确性、一致性 |

---

## 📝 完善清单

### memory.yaml 需要添加的内容

```yaml
# P0 - 必须添加
- 页表管理 (pgd, pud, pmd, pte)
- 内存回收 (shrink_node, reclaim)
- OOM killer (oom_kill_process)
- 页缓存 (address_space, page cache)
- NUMA (node, zone, zonelist)
- 内存映射 (do_mmap, vm_area_struct)

# P1 - 应该添加
- 写时复制 (COW)
- 透明大页 (THP)
- 内存压缩 (compaction)
- 交换 (swap)

# P2 - 可选添加
- 内存热插拔
- 内存调试 (KASAN, KMEMLEAK)
```

### sched.yaml 需要添加的内容

```yaml
# P0 - 必须添加
- 负载均衡 (load_balance, idle_balance)
- 等待队列 (wait_queue, wait_event)
- 睡眠/唤醒机制
- CPU 带宽控制 (cfs_bandwidth)

# P1 - 应该添加
- PELT (Per-Entity Load Tracking)
- 调度组 (task_group)
- Deadline 调度器详细

# P2 - 可选添加
- 能耗感知调度 (EAS)
- 核心调度 (core scheduling)
```

### irq.yaml 需要添加的内容

```yaml
# P0 - 必须添加
- IRQ Domain (irq_domain)
- IRQ Chip (irq_chip, irq_data)
- 中断亲和性

# P1 - 应该添加
- MSI/MSI-X
- 级联中断
- IPI

# P2 - 可选添加
- NMI
- 中断控制器驱动
```

### vfs.yaml 需要添加的内容

```yaml
# P0 - 必须添加
- address_space_operations (页缓存核心)
- 直接 I/O
- 文件锁

# P1 - 应该添加
- 块层集成 (bio, request)
- 具体文件系统框架

# P2 - 可选添加
- ACL 和扩展属性
- overlayfs
```

---

## 📅 执行计划

### Phase 1: P0 核心子系统 (5 个文件)

```
并行执行:
- KB-Memory: memory.yaml (30% → 90%)
- KB-Sched: sched.yaml (40% → 90%)
- KB-IRQ: irq.yaml (60% → 95%)
- KB-FS: vfs.yaml + vfs_ops.yaml (50% → 90%)

审核:
- KB-Reviewer: 审核所有 P0 文件
```

### Phase 2: P1 重要子系统 (9 个文件)

```
并行执行:
- KB-Core: workqueue, timer, kthread, ipc, cgroup
- KB-Sync: rcu
- KB-Security: security
- KB-Debug: tracing
- KB-IRQ: softirq

审核:
- KB-Reviewer: 审核所有 P1 文件
```

### Phase 3: P2 驱动框架 (42 个文件)

```
分批并行执行:
- KB-Drivers: 按分组完善

审核:
- KB-Reviewer: 抽查审核
```

### Phase 4: P3-P4 网络 & 同步 (7 个文件)

```
并行执行:
- KB-Net: netdev, socket, netfilter
- KB-Sync: locking, completion, rwlock, semaphore

审核:
- KB-Reviewer: 审核所有文件
```

---

## ✅ 验收标准

每个知识库文件通过以下检查：

| 检查项 | 要求 |
|--------|------|
| **结构完整** | 包含所有必需组件 |
| **Pattern 正确** | 正则表达式可匹配真实代码 |
| **Context 准确** | 执行上下文标注正确 |
| **调用链完整** | 覆盖主要执行路径 |
| **示例可编译** | 示例代码语法正确 |
| **版本标注** | 标注适用内核版本 |

---

## 📊 进度追踪

| 阶段 | 文件数 | 状态 | 完成日期 |
|------|--------|------|----------|
| Phase 1 | 5 | 待开始 | - |
| Phase 2 | 9 | 待开始 | - |
| Phase 3 | 42 | 待开始 | - |
| Phase 4 | 7 | 待开始 | - |
| **总计** | **63** | - | - |
