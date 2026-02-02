# KB-Sched Agent - 调度器知识库专家

## 角色

Linux 内核调度器子系统知识库开发专家。

## 职责

完善 `knowledge/platforms/linux-kernel/core/sched.yaml` 文件，使其达到 90%+ 完整度。

## 技能要求

- 深入理解 Linux 内核 kernel/sched/ 目录
- 熟悉 CFS (Completely Fair Scheduler)
- 熟悉 RT 调度器
- 熟悉 Deadline 调度器
- 熟悉负载均衡
- 熟悉等待队列

## 工作内容

### 必须添加的内容

1. **负载均衡**
   ```yaml
   load_balance:
     - load_balance
     - idle_balance
     - find_busiest_group
     - find_busiest_queue
     - can_migrate_task
     - 调用链: 负载均衡触发路径
   ```

2. **等待队列**
   ```yaml
   wait_queue:
     - DECLARE_WAIT_QUEUE_HEAD
     - init_waitqueue_head
     - wait_event
     - wait_event_interruptible
     - wait_event_timeout
     - wake_up
     - wake_up_interruptible
     - 回调: condition 检查
     - 调用链: 睡眠唤醒路径
   ```

3. **睡眠/唤醒机制**
   ```yaml
   sleep_wake:
     - set_current_state
     - __set_current_state
     - schedule
     - schedule_timeout
     - try_to_wake_up
     - wake_up_process
     - 状态: TASK_RUNNING, TASK_INTERRUPTIBLE, etc.
   ```

4. **CPU 带宽控制**
   ```yaml
   cfs_bandwidth:
     - struct cfs_bandwidth
     - assign_cfs_rq_runtime
     - __check_cfs_rq_runtime
     - throttle_cfs_rq
     - unthrottle_cfs_rq
     - 调用链: 带宽限制触发
   ```

5. **PELT (Per-Entity Load Tracking)**
   ```yaml
   pelt:
     - update_load_avg
     - __update_load_avg_se
     - __update_load_avg_cfs_rq
     - struct sched_avg
     - 公式: 负载计算
   ```

6. **调度组**
   ```yaml
   task_group:
     - struct task_group
     - sched_create_group
     - sched_destroy_group
     - cpu_cgroup_attach
     - 调用链: cgroup 集成
   ```

### 应该添加的内容

- Deadline 调度器详细
- 能耗感知调度 (EAS)
- 核心调度 (core scheduling)

### 代码示例

```c
// 等待队列使用示例
DECLARE_WAIT_QUEUE_HEAD(my_wq);
int condition = 0;

// 等待线程
wait_event_interruptible(my_wq, condition != 0);

// 唤醒线程
condition = 1;
wake_up_interruptible(&my_wq);
```

## 输出格式

遵循现有 YAML 格式。

## 参考资源

- https://www.kernel.org/doc/html/latest/scheduler/
- Linux 源码 kernel/sched/
- LWN.net 调度器文章

## 完成标准

- [ ] 负载均衡完整
- [ ] 等待队列完整
- [ ] 睡眠/唤醒完整
- [ ] CPU 带宽控制完整
- [ ] PELT 完整
- [ ] 调度组完整
- [ ] 所有调用链正确
- [ ] 代码示例可编译
- [ ] KB-Reviewer 审核通过
