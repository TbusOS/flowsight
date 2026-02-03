# Pattern 精准度验证报告

验证时间: Wed Feb  4 02:27:51 CST 2026
内核路径: /Users/sky/linux-kernel/linux

## Pattern 提取

- 知识库 Pattern 总数: 

## Pattern 匹配验证

| Pattern | 匹配文件数 | 匹配行数 | 状态 |
|---------|-----------|----------|------|
| `kmalloc\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `kzalloc\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `kfree\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `vmalloc\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `ioremap\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `readl\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `writel\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `spin_lock\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `spin_unlock\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `mutex_lock\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `mutex_unlock\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `local_irq_disable\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `local_irq_save\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `request_irq\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `devm_request_irq\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `free_irq\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `tasklet_schedule\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `INIT_WORK\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `schedule_work\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `queue_work\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `timer_setup\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `mod_timer\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `\.probe\s*=` | 0 | 0 | ❌ 知识库缺失 |
| `\.remove\s*=` | 0 | 0 | ❌ 知识库缺失 |
| `\.open\s*=` | 0 | 0 | ❌ 知识库缺失 |
| `\.read\s*=` | 0 | 0 | ❌ 知识库缺失 |
| `\.write\s*=` | 0 | 0 | ❌ 知识库缺失 |
| `\.ioctl\s*=` | 0 | 0 | ❌ 知识库缺失 |
| `module_init\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `module_exit\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `platform_driver_register\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `usb_register\s*\(` | 0 | 0 | ❌ 知识库缺失 |

## 误报检测 (抽样)

随机抽取 5 个匹配结果验证:

### 中断请求 (`request_irq`)
```
```

### WorkQueue初始化 (`INIT_WORK`)
```
```

### probe回调 (`\.probe\s*=`)
```
```

## 漏报检测 (API 变体)

检查知识库是否覆盖常见 API 变体:

| API 基础 | 变体 | 内核使用 | 知识库覆盖 |
|----------|------|----------|-----------|
| kmalloc | `kmalloc` | 0 | ❌ |
| kmalloc | `kmalloc_array` | 0 | ❌ |
| kmalloc | `kmalloc_node` | 0 | ❌ |
| kmalloc | `__kmalloc` | 0 | ❌ |
| kmalloc | `krealloc` | 0 | ❌ |
| spin_lock | `spin_lock` | 0 | ❌ |
| spin_lock | `spin_lock_irq` | 0 | ❌ |
| spin_lock | `spin_lock_irqsave` | 0 | ❌ |
| spin_lock | `spin_lock_bh` | 0 | ❌ |
| spin_lock | `spin_trylock` | 0 | ❌ |
| wait_event | `wait_event` | 0 | ❌ |
| wait_event | `wait_event_interruptible` | 0 | ❌ |
| wait_event | `wait_event_timeout` | 0 | ❌ |
| wait_event | `wait_event_interruptible_timeout` | 0 | ❌ |
| wait_event | `wait_event_killable` | 0 | ❌ |

## Context 标注检查

检查关键 API 的 can_sleep/context 标注是否正确:

| API | 正确 Context | 知识库标注 | 状态 |
|-----|-------------|-----------|------|
| `kmalloc` | GFP_KERNEL时可睡眠 |  | ⚠️ 未标注 |
| `request_irq` | 可睡眠(进程上下文) |  | ⚠️ 未标注 |
| `in_interrupt` | 任意上下文 |  | ⚠️ 未标注 |
| `schedule` | 可睡眠 |  | ⚠️ 未标注 |
| `spin_lock` | 不可睡眠 |  | ⚠️ 未标注 |
| `tasklet_schedule` | 不可睡眠 |  | ⚠️ 未标注 |
| `local_irq_disable` | 任意上下文 |  | ⚠️ 未标注 |
| `spin_lock_irq` | 不可睡眠 |  | ⚠️ 未标注 |
| `mutex_lock` | 可睡眠 |  | ⚠️ 未标注 |

## 调用链验证

检查知识库中的调用链是否与内核源码一致:

### schedule() 调用链

内核源码 (`kernel/sched/core.c`):
```
asmlinkage __visible void __sched schedule(void)
{
	struct task_struct *tsk = current;

```

### request_irq() 实现

内核源码 (`kernel/irq/manage.c`):
```
```

## 验证总结

- ✅ 有效 Pattern: 0
0
- ❌ 缺失/问题: 47
- ⚠️ 需关注: 9

### 需要改进的问题

| `kmalloc\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `kzalloc\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `kfree\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `vmalloc\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `ioremap\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `readl\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `writel\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `spin_lock\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `spin_unlock\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `mutex_lock\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `mutex_unlock\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `local_irq_disable\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `local_irq_save\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `request_irq\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `devm_request_irq\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `free_irq\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `tasklet_schedule\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `INIT_WORK\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `schedule_work\s*\(` | 0 | 0 | ❌ 知识库缺失 |
| `queue_work\s*\(` | 0 | 0 | ❌ 知识库缺失 |

---
验证完成。报告位置: .claude/reports/kb-validations/validation-2026-02-04.md
