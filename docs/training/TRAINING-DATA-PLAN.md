# FlowSight AI 训练数据完整计划

## 目录

1. [总体目标](#1-总体目标)
2. [覆盖范围](#2-覆盖范围)
3. [数据规模估算](#3-数据规模估算)
4. [分阶段实施计划](#4-分阶段实施计划)
5. [数据质量标准](#5-数据质量标准)
6. [实施进度跟踪](#6-实施进度跟踪)

---

## 1. 总体目标

### 1.1 最终目标

训练一个能够理解 **Linux 内核完整知识体系** 的专用 AI 模型，能够：

- 追踪任意函数指针的目标函数
- 分析任意异步执行流程
- 重建完整的调用链
- 识别各种代码模式
- 理解内核各子系统的工作原理

### 1.2 数据总量目标

| 阶段 | 覆盖范围 | 样本数量 | 状态 |
|------|----------|----------|------|
| Phase 1 | 核心驱动框架 + 异步机制 | 10,000 | 🔴 待开始 |
| Phase 2 | 内存/进程/文件系统 | 15,000 | 🔴 待开始 |
| Phase 3 | 网络/块设备/安全 | 15,000 | 🔴 待开始 |
| Phase 4 | 完整外设驱动 | 20,000 | 🔴 待开始 |
| Phase 5 | 边界情况 + 多样化 | 10,000 | 🔴 待开始 |
| **总计** | **完整 Linux 内核** | **70,000** | |

---

## 2. 覆盖范围

### 2.1 Phase 1：核心驱动框架 + 异步机制（10,000 样本）

#### 2.1.1 总线驱动框架（3,000 样本）

| 框架 | 结构体 | 主要回调 | 样本数 | 状态 |
|------|--------|----------|--------|------|
| **USB** | `struct usb_driver` | probe, disconnect, suspend, resume, reset_resume, pre_reset, post_reset | 300 | 🔴 |
| **PCI/PCIe** | `struct pci_driver` | probe, remove, suspend, resume, shutdown, sriov_configure | 300 | 🔴 |
| **Platform** | `struct platform_driver` | probe, remove, shutdown, suspend, resume | 300 | 🔴 |
| **I2C** | `struct i2c_driver` | probe, remove, shutdown, alert, detect, address_list | 200 | 🔴 |
| **SPI** | `struct spi_driver` | probe, remove, shutdown | 200 | 🔴 |
| **AMBA** | `struct amba_driver` | probe, remove, shutdown | 150 | 🔴 |
| **MDIO** | `struct mdio_driver` | probe, remove | 100 | 🔴 |
| **SDIO** | `struct sdio_driver` | probe, remove | 100 | 🔴 |
| **ACPI** | `struct acpi_driver` | add, remove, notify | 150 | 🔴 |
| **OF (设备树)** | `of_device_id` | match, probe | 200 | 🔴 |
| **MFD** | `struct mfd_cell` | probe, remove | 150 | 🔴 |
| **Virtio** | `struct virtio_driver` | probe, remove, config_changed | 150 | 🔴 |
| **Thunderbolt** | `struct tb_service_driver` | probe, remove | 100 | 🔴 |
| **SPMI** | `struct spmi_driver` | probe, remove | 100 | 🔴 |
| **SLIMbus** | `struct slim_driver` | probe, remove, device_status | 100 | 🔴 |
| **SoundWire** | `struct sdw_driver` | probe, remove, update_status | 100 | 🔴 |
| **Serdev** | `struct serdev_device_driver` | probe, remove | 100 | 🔴 |
| **Auxiliary** | `struct auxiliary_driver` | probe, remove | 100 | 🔴 |

#### 2.1.2 字符设备框架（1,500 样本）

| 框架 | 结构体 | 主要回调 | 样本数 | 状态 |
|------|--------|----------|--------|------|
| **file_operations** | `struct file_operations` | open, release, read, write, mmap, poll, unlocked_ioctl, compat_ioctl, fsync, llseek, fasync, splice_read, splice_write | 500 | 🔴 |
| **cdev** | `struct cdev` | 注册/注销流程 | 200 | 🔴 |
| **misc_device** | `struct miscdevice` | 注册/注销流程 | 200 | 🔴 |
| **tty_operations** | `struct tty_operations` | open, close, write, write_room, chars_in_buffer, ioctl, set_termios, throttle, unthrottle, stop, start, hangup, tiocmget, tiocmset | 300 | 🔴 |
| **uart_ops** | `struct uart_ops` | startup, shutdown, tx_empty, set_mctrl, get_mctrl, stop_tx, start_tx, stop_rx, enable_ms, break_ctl, set_termios, pm, type, request_port, release_port, config_port, verify_port | 300 | 🔴 |

#### 2.1.3 异步机制（2,500 样本）

| 机制 | 相关 API | 调用链 | 样本数 | 状态 |
|------|----------|--------|--------|------|
| **WorkQueue** | INIT_WORK, schedule_work, queue_work, flush_work, cancel_work_sync, INIT_DELAYED_WORK, schedule_delayed_work, mod_delayed_work | 400 | 🔴 |
| **Timer** | timer_setup, mod_timer, add_timer, del_timer, del_timer_sync, DEFINE_TIMER | 300 | 🔴 |
| **HRTimer** | hrtimer_init, hrtimer_start, hrtimer_cancel, hrtimer_forward | 200 | 🔴 |
| **Tasklet** | tasklet_init, tasklet_setup, tasklet_schedule, tasklet_kill, tasklet_disable, tasklet_enable | 200 | 🔴 |
| **SoftIRQ** | open_softirq, raise_softirq, raise_softirq_irqoff | 150 | 🔴 |
| **IRQ** | request_irq, free_irq, devm_request_irq, enable_irq, disable_irq, irq_set_affinity | 300 | 🔴 |
| **Threaded IRQ** | request_threaded_irq, devm_request_threaded_irq | 200 | 🔴 |
| **Completion** | init_completion, wait_for_completion, wait_for_completion_timeout, complete, complete_all, reinit_completion | 200 | 🔴 |
| **WaitQueue** | init_waitqueue_head, wait_event, wait_event_interruptible, wait_event_timeout, wake_up, wake_up_interruptible, wake_up_all | 250 | 🔴 |
| **KThread** | kthread_create, kthread_run, kthread_stop, kthread_should_stop, kthread_park, kthread_unpark | 150 | 🔴 |
| **RCU** | rcu_read_lock, rcu_read_unlock, synchronize_rcu, call_rcu, rcu_assign_pointer, rcu_dereference | 150 | 🔴 |

#### 2.1.4 同步原语（1,500 样本）

| 原语 | 相关 API | 样本数 | 状态 |
|------|----------|--------|------|
| **Spinlock** | spin_lock, spin_unlock, spin_lock_irqsave, spin_unlock_irqrestore, spin_lock_bh, spin_trylock | 250 | 🔴 |
| **Mutex** | mutex_lock, mutex_unlock, mutex_trylock, mutex_lock_interruptible, mutex_is_locked | 250 | 🔴 |
| **Semaphore** | down, up, down_interruptible, down_trylock | 150 | 🔴 |
| **RW Spinlock** | read_lock, read_unlock, write_lock, write_unlock | 150 | 🔴 |
| **RW Semaphore** | down_read, up_read, down_write, up_write | 150 | 🔴 |
| **Seqlock** | read_seqbegin, read_seqretry, write_seqlock, write_sequnlock | 100 | 🔴 |
| **Atomic** | atomic_read, atomic_set, atomic_add, atomic_sub, atomic_inc, atomic_dec, atomic_cmpxchg | 150 | 🔴 |
| **Per-CPU** | DEFINE_PER_CPU, this_cpu_ptr, get_cpu_var, put_cpu_var | 100 | 🔴 |
| **Memory Barrier** | mb, rmb, wmb, smp_mb, smp_rmb, smp_wmb, barrier | 100 | 🔴 |

#### 2.1.5 设备模型（1,500 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **Device** | struct device, device_register, device_unregister, device_add, device_del, get_device, put_device | 250 | 🔴 |
| **Driver** | struct device_driver, driver_register, driver_unregister | 200 | 🔴 |
| **Bus** | struct bus_type, bus_register, bus_unregister, bus_for_each_dev, bus_for_each_drv | 200 | 🔴 |
| **Class** | struct class, class_register, class_unregister, class_create, class_destroy | 200 | 🔴 |
| **Kobject** | struct kobject, kobject_init, kobject_add, kobject_put, kobject_del | 200 | 🔴 |
| **Sysfs** | sysfs_create_file, sysfs_remove_file, DEVICE_ATTR, DRIVER_ATTR, BUS_ATTR, CLASS_ATTR | 250 | 🔴 |
| **devres** | devm_kzalloc, devm_request_irq, devm_ioremap, devm_clk_get, devres_add, devres_remove | 200 | 🔴 |

---

### 2.2 Phase 2：内存/进程/文件系统（15,000 样本）

#### 2.2.1 内存管理（5,000 样本）

| 模块 | 相关 API | 样本数 | 状态 |
|------|----------|--------|------|
| **页分配器** | alloc_pages, __free_pages, __get_free_pages, free_pages, page_address | 400 | 🔴 |
| **Slab 分配器** | kmalloc, kfree, kzalloc, kcalloc, krealloc, kmem_cache_create, kmem_cache_alloc, kmem_cache_free | 500 | 🔴 |
| **Vmalloc** | vmalloc, vfree, vzalloc, vmap, vunmap | 300 | 🔴 |
| **CMA** | dma_alloc_coherent, dma_free_coherent, dma_alloc_attrs, cma_alloc, cma_release | 300 | 🔴 |
| **内存映射** | mmap, munmap, mremap, mprotect, madvise, mlock, munlock | 400 | 🔴 |
| **缺页处理** | do_page_fault, handle_mm_fault, do_anonymous_page, do_fault, do_wp_page | 500 | 🔴 |
| **页面回收** | try_to_free_pages, shrink_node, shrink_lruvec, shrink_page_list | 400 | 🔴 |
| **交换** | swap_readpage, swap_writepage, add_to_swap, delete_from_swap_cache | 300 | 🔴 |
| **大页** | hugetlb_fault, alloc_huge_page, free_huge_page | 300 | 🔴 |
| **内存压缩** | compact_zone, isolate_migratepages, migrate_pages | 300 | 🔴 |
| **NUMA** | numa_node_id, node_data, alloc_pages_node, numa_balancing | 300 | 🔴 |
| **memcg** | mem_cgroup_charge, mem_cgroup_uncharge, memcg_kmem_charge | 300 | 🔴 |
| **OOM** | out_of_memory, oom_killer, select_bad_process | 200 | 🔴 |
| **VMA** | vm_area_struct, find_vma, vma_merge, split_vma | 400 | 🔴 |
| **页表** | pgd, pud, pmd, pte, pte_alloc, pte_free | 300 | 🔴 |

#### 2.2.2 进程管理（5,000 样本）

| 模块 | 相关 API | 样本数 | 状态 |
|------|----------|--------|------|
| **进程创建** | fork, vfork, clone, clone3, copy_process, dup_task_struct, copy_mm, copy_files | 500 | 🔴 |
| **进程退出** | do_exit, exit_group, release_task, wait_task_zombie | 300 | 🔴 |
| **exec** | do_execve, exec_binprm, load_elf_binary, setup_new_exec | 400 | 🔴 |
| **调度核心** | schedule, __schedule, pick_next_task, context_switch, switch_to | 500 | 🔴 |
| **CFS 调度** | task_fork_fair, enqueue_task_fair, dequeue_task_fair, pick_next_task_fair, put_prev_task_fair | 400 | 🔴 |
| **RT 调度** | enqueue_task_rt, dequeue_task_rt, pick_next_task_rt | 300 | 🔴 |
| **Deadline 调度** | enqueue_task_dl, dequeue_task_dl, pick_next_task_dl | 200 | 🔴 |
| **负载均衡** | load_balance, find_busiest_group, can_migrate_task, move_queued_task | 300 | 🔴 |
| **CPU 热插拔** | cpu_up, cpu_down, _cpu_up, _cpu_down, cpuhp_invoke_callback | 300 | 🔴 |
| **进程状态** | set_current_state, __set_current_state, wake_up_process, wake_up_state | 300 | 🔴 |
| **优先级** | set_user_nice, sched_setscheduler, sched_setparam | 200 | 🔴 |
| **cgroup CPU** | cpu_cgroup_attach, cpu_cgroup_css_alloc, cpu_cgroup_css_free | 300 | 🔴 |
| **信号** | do_signal, handle_signal, send_signal, complete_signal, dequeue_signal | 500 | 🔴 |
| **Namespace** | create_new_namespaces, copy_namespaces, switch_task_namespaces | 300 | 🔴 |
| **Credentials** | prepare_creds, commit_creds, override_creds, revert_creds | 200 | 🔴 |

#### 2.2.3 文件系统（5,000 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **VFS 核心** | struct inode, struct dentry, struct file, struct super_block | 500 | 🔴 |
| **inode_operations** | lookup, create, link, unlink, mkdir, rmdir, rename, getattr, setattr, permission | 500 | 🔴 |
| **file_operations** (VFS) | read, write, read_iter, write_iter, llseek, mmap, fsync, splice_read | 400 | 🔴 |
| **address_space_ops** | readpage, writepage, readpages, writepages, set_page_dirty, direct_IO | 400 | 🔴 |
| **super_operations** | alloc_inode, destroy_inode, write_inode, evict_inode, sync_fs, statfs, remount_fs | 400 | 🔴 |
| **dentry_operations** | d_revalidate, d_hash, d_compare, d_delete, d_release | 300 | 🔴 |
| **页面缓存** | find_get_page, add_to_page_cache, delete_from_page_cache, read_cache_page | 400 | 🔴 |
| **路径查找** | path_lookupat, link_path_walk, lookup_fast, lookup_slow | 400 | 🔴 |
| **文件锁** | flock, fcntl, posix_lock_file, locks_alloc_lock | 300 | 🔴 |
| **ext4** | ext4 特有的回调和流程 | 400 | 🔴 |
| **xfs** | xfs 特有的回调和流程 | 300 | 🔴 |
| **btrfs** | btrfs 特有的回调和流程 | 300 | 🔴 |
| **procfs** | proc_ops, proc_create, proc_mkdir, seq_file | 300 | 🔴 |
| **sysfs** | kernfs_ops, sysfs_create_group | 200 | 🔴 |
| **debugfs** | debugfs_create_file, debugfs_create_dir | 200 | 🔴 |

---

### 2.3 Phase 3：网络/块设备/安全（15,000 样本）

#### 2.3.1 网络子系统（7,000 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **net_device_ops** | ndo_open, ndo_stop, ndo_start_xmit, ndo_get_stats64, ndo_set_rx_mode, ndo_set_mac_address, ndo_validate_addr, ndo_do_ioctl, ndo_change_mtu, ndo_vlan_rx_add_vid | 600 | 🔴 |
| **ethtool_ops** | get_settings, set_settings, get_drvinfo, get_regs_len, get_regs, get_link, get_ringparam, set_ringparam | 400 | 🔴 |
| **NAPI** | napi_enable, napi_disable, napi_schedule, napi_complete, netif_napi_add, napi_gro_receive | 400 | 🔴 |
| **Socket** | sock_create, sock_release, kernel_connect, kernel_bind, kernel_listen, kernel_accept, kernel_sendmsg, kernel_recvmsg | 500 | 🔴 |
| **proto_ops** | release, bind, connect, accept, listen, sendmsg, recvmsg, mmap, poll | 400 | 🔴 |
| **sk_buff** | alloc_skb, kfree_skb, skb_clone, skb_copy, skb_put, skb_push, skb_pull, skb_reserve | 400 | 🔴 |
| **TCP** | tcp_v4_connect, tcp_sendmsg, tcp_recvmsg, tcp_close, tcp_transmit_skb | 500 | 🔴 |
| **UDP** | udp_sendmsg, udp_recvmsg, udp_queue_rcv_skb | 300 | 🔴 |
| **IP** | ip_rcv, ip_local_deliver, ip_queue_xmit, ip_local_out | 400 | 🔴 |
| **ARP/Neighbor** | arp_rcv, neigh_resolve_output, neigh_lookup | 200 | 🔴 |
| **Netfilter** | nf_register_net_hook, nf_unregister_net_hook, NF_HOOK, nf_hook | 400 | 🔴 |
| **TC** | qdisc_ops, qdisc_create, qdisc_destroy, qdisc_enqueue, qdisc_dequeue | 300 | 🔴 |
| **Bridge** | br_dev_xmit, br_handle_frame, br_forward | 200 | 🔴 |
| **VLAN** | vlan_dev_hard_start_xmit, vlan_dev_set_egress_priority | 200 | 🔴 |
| **Netlink** | netlink_kernel_create, netlink_unicast, netlink_broadcast | 300 | 🔴 |
| **XDP/eBPF** | bpf_prog_run_xdp, xdp_do_redirect | 300 | 🔴 |
| **WiFi (cfg80211)** | cfg80211_ops, ieee80211_ops | 500 | 🔴 |
| **Bluetooth** | hci_register_dev, bt_sock_ops | 300 | 🔴 |

#### 2.3.2 块设备层（4,000 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **block_device_operations** | open, release, ioctl, compat_ioctl, getgeo, submit_bio | 400 | 🔴 |
| **blk_mq_ops** | queue_rq, commit_rqs, complete, init_request, exit_request, init_hctx, exit_hctx, poll | 500 | 🔴 |
| **gendisk** | alloc_disk, add_disk, del_gendisk, put_disk | 300 | 🔴 |
| **bio** | bio_alloc, bio_put, bio_add_page, submit_bio, bio_endio | 400 | 🔴 |
| **request** | blk_mq_alloc_request, blk_mq_free_request, blk_mq_start_request, blk_mq_end_request | 300 | 🔴 |
| **I/O 调度** | elevator_type, elevator_ops | 300 | 🔴 |
| **Device Mapper** | dm_target_type, dm_register_target, dm_table_create | 400 | 🔴 |
| **MD/RAID** | md_personality, md_register, md_unregister | 300 | 🔴 |
| **NVMe** | nvme_ctrl_ops, nvme_queue, nvme_command | 400 | 🔴 |
| **SCSI** | scsi_host_template, scsi_device, scsi_cmnd, scsi_transport_template | 500 | 🔴 |
| **ATA/SATA** | ata_port_operations, ata_device, ata_queued_cmd | 300 | 🔴 |

#### 2.3.3 安全子系统（2,000 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **LSM** | security_hook_heads, security_add_hooks | 300 | 🔴 |
| **Capabilities** | capable, ns_capable, has_capability | 200 | 🔴 |
| **Credentials** | current_cred, override_creds, prepare_creds | 200 | 🔴 |
| **SELinux** | selinux_hooks, avc_has_perm | 300 | 🔴 |
| **AppArmor** | apparmor_hooks | 200 | 🔴 |
| **seccomp** | seccomp_filter, seccomp_run_filters | 200 | 🔴 |
| **Audit** | audit_log, audit_syscall_entry | 200 | 🔴 |
| **Keys** | key_type, request_key, keyring_alloc | 200 | 🔴 |
| **Crypto** | crypto_alg, crypto_register_alg, crypto_alloc_tfm | 200 | 🔴 |

#### 2.3.4 电源管理（2,000 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **System PM** | pm_suspend, pm_resume, suspend_ops | 300 | 🔴 |
| **Device PM** | dev_pm_ops, pm_runtime_get, pm_runtime_put, pm_runtime_resume | 400 | 🔴 |
| **CPUFreq** | cpufreq_driver, cpufreq_policy | 300 | 🔴 |
| **CPUIdle** | cpuidle_driver, cpuidle_state | 300 | 🔴 |
| **Regulator** | regulator_ops, regulator_register | 300 | 🔴 |
| **Clock** | clk_ops, clk_hw_register | 400 | 🔴 |

---

### 2.4 Phase 4：完整外设驱动（20,000 样本）

#### 2.4.1 GPIO/Pinctrl/PWM（2,000 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **gpio_chip** | direction_input, direction_output, get, set, get_direction, set_config, to_irq | 600 | 🔴 |
| **pinctrl_ops** | get_groups_count, get_group_name, get_group_pins | 300 | 🔴 |
| **pinmux_ops** | get_functions_count, get_function_name, set_mux | 300 | 🔴 |
| **pinconf_ops** | pin_config_get, pin_config_set | 300 | 🔴 |
| **pwm_ops** | request, free, config, set_polarity, enable, disable, apply | 300 | 🔴 |
| **LED** | led_classdev, led_brightness_set, led_trigger_register | 200 | 🔴 |

#### 2.4.2 Input 子系统（2,000 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **input_dev** | input_register_device, input_unregister_device, input_event, input_report_key, input_report_abs | 500 | 🔴 |
| **input_handler** | connect, disconnect, event, events | 300 | 🔴 |
| **evdev** | evdev_open, evdev_release, evdev_read, evdev_write | 300 | 🔴 |
| **HID** | hid_driver, hid_register_driver, hid_parse, hid_hw_start | 500 | 🔴 |
| **Touchscreen** | touchscreen_report_pos, touchscreen_parse_properties | 200 | 🔴 |
| **Keyboard/Mouse** | keyboard 和 mouse 特有处理 | 200 | 🔴 |

#### 2.4.3 IIO 子系统（1,500 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **iio_dev** | iio_device_register, iio_device_unregister | 300 | 🔴 |
| **iio_info** | read_raw, write_raw, read_event_config, write_event_config | 400 | 🔴 |
| **iio_buffer_ops** | preenable, postenable, predisable, postdisable | 300 | 🔴 |
| **iio_trigger_ops** | set_trigger_state, validate_device | 200 | 🔴 |
| **IIO channels** | iio_chan_spec, iio_push_to_buffers | 300 | 🔴 |

#### 2.4.4 媒体子系统（3,000 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **V4L2** | v4l2_device, v4l2_subdev, video_device | 600 | 🔴 |
| **v4l2_ioctl_ops** | vidioc_querycap, vidioc_enum_fmt_vid_cap, vidioc_g_fmt_vid_cap, vidioc_s_fmt_vid_cap, vidioc_reqbufs, vidioc_querybuf, vidioc_qbuf, vidioc_dqbuf, vidioc_streamon, vidioc_streamoff | 500 | 🔴 |
| **v4l2_subdev_ops** | core, video, pad | 400 | 🔴 |
| **vb2_ops** | queue_setup, buf_prepare, buf_queue, start_streaming, stop_streaming | 400 | 🔴 |
| **DRM** | drm_driver, drm_crtc_funcs, drm_encoder_funcs, drm_connector_funcs, drm_plane_funcs | 600 | 🔴 |
| **ALSA** | snd_pcm_ops, snd_soc_dai_ops, snd_soc_component_driver | 500 | 🔴 |

#### 2.4.5 存储子系统（2,500 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **MMC** | mmc_host_ops, mmc_request, mmc_ios | 500 | 🔴 |
| **MTD** | mtd_info, mtd_oops, nand_chip | 500 | 🔴 |
| **NAND** | nand_manufacturer_ops, nand_controller_ops | 300 | 🔴 |
| **NOR** | spi_nor, spi_nor_flash_parameter | 300 | 🔴 |
| **NVMEM** | nvmem_config, nvmem_register | 300 | 🔴 |
| **RTC** | rtc_class_ops, rtc_device_register | 300 | 🔴 |
| **Watchdog** | watchdog_ops, watchdog_register_device | 300 | 🔴 |

#### 2.4.6 网络硬件（3,000 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **PHY** | phy_driver, phy_device, mdio_device | 500 | 🔴 |
| **MAC** | 各种 MAC 驱动模式 | 400 | 🔴 |
| **WiFi MAC80211** | ieee80211_ops 完整 | 600 | 🔴 |
| **Bluetooth HCI** | hci_dev, bt_host | 400 | 🔴 |
| **NFC** | nfc_dev, nfc_ops | 300 | 🔴 |
| **CAN** | can_priv, can_ml_priv | 300 | 🔴 |
| **Ethernet Switch** | switchdev_ops, dsa_switch_ops | 300 | 🔴 |
| **InfiniBand** | ib_device_ops | 200 | 🔴 |

#### 2.4.7 其他驱动（6,000 样本）

| 模块 | 相关结构/API | 样本数 | 状态 |
|------|--------------|--------|------|
| **Thermal** | thermal_zone_device_ops, thermal_cooling_device_ops | 400 | 🔴 |
| **Hwmon** | hwmon_chip_info, hwmon_ops | 400 | 🔴 |
| **EDAC** | edac_mc_ops | 200 | 🔴 |
| **IOMMU** | iommu_ops, iommu_domain_ops | 400 | 🔴 |
| **DMA Engine** | dma_device, dma_async_tx_descriptor | 400 | 🔴 |
| **Mailbox** | mbox_chan_ops | 200 | 🔴 |
| **Remoteproc** | rproc_ops | 300 | 🔴 |
| **RPMsg** | rpmsg_driver, rpmsg_endpoint_ops | 300 | 🔴 |
| **TEE** | tee_driver_ops, tee_shm_pool_ops | 300 | 🔴 |
| **FPGA** | fpga_manager_ops, fpga_bridge_ops | 300 | 🔴 |
| **Counter** | counter_device, counter_ops | 200 | 🔴 |
| **PTP** | ptp_clock_ops | 200 | 🔴 |
| **UIO** | uio_info | 200 | 🔴 |
| **VFIO** | vfio_device_ops | 300 | 🔴 |
| **Virtio** | virtio_device, virtqueue | 400 | 🔴 |
| **Greybus** | greybus_driver | 200 | 🔴 |
| **Android Binder** | binder_proc, binder_thread | 300 | 🔴 |
| **Perf** | perf_event, pmu | 400 | 🔴 |
| **Tracing** | tracer, ftrace_ops | 400 | 🔴 |
| **KProbes** | kprobe, kretprobe | 200 | 🔴 |

---

### 2.5 Phase 5：边界情况 + 多样化（10,000 样本）

#### 2.5.1 复杂调用链（3,000 样本）

- 跨子系统调用链
- 10+ 层函数调用
- 多条件分支组合
- 间接调用链
- 递归调用

#### 2.5.2 运行时动态场景（2,000 样本）

- 运行时函数指针修改
- 多驱动共存
- 热插拔
- 动态电源管理
- 错误路径

#### 2.5.3 问题形式多样化（3,000 样本）

- 同一代码不同问法
- 从用户命令追踪到内核
- 错误分析
- 性能分析
- 调试场景

#### 2.5.4 真实代码片段（2,000 样本）

- 从主流驱动提取
- 包含完整上下文
- 真实的注释和代码风格

---

## 3. 数据规模估算

### 3.1 各阶段样本数

| 阶段 | 样本数 | 累计 | 训练时间估算 |
|------|--------|------|--------------|
| Phase 1 | 10,000 | 10,000 | ~3小时 (A100) |
| Phase 2 | 15,000 | 25,000 | +5小时 |
| Phase 3 | 15,000 | 40,000 | +5小时 |
| Phase 4 | 20,000 | 60,000 | +7小时 |
| Phase 5 | 10,000 | 70,000 | +3小时 |
| **总计** | **70,000** | **70,000** | **~23小时** |

### 3.2 存储需求

- 原始数据：~500MB (JSONL)
- 处理后数据：~1GB
- 模型检查点：~30GB

---

## 4. 分阶段实施计划

### 4.1 Phase 1 实施（预计 2-3 周）

```
Week 1:
├── Day 1-2: 总线驱动框架数据生成脚本
├── Day 3-4: 字符设备框架数据生成脚本
└── Day 5-7: 异步机制数据生成脚本

Week 2:
├── Day 1-2: 同步原语数据生成脚本
├── Day 3-4: 设备模型数据生成脚本
└── Day 5-7: 数据质量检查和修正

Week 3:
├── Day 1-3: 数据增强和多样化
├── Day 4-5: 首次训练测试
└── Day 6-7: 模型评估和调整
```

### 4.2 Phase 2-5 实施

每个阶段预计 2-3 周，根据 Phase 1 的经验调整。

---

## 5. 数据质量标准

### 5.1 每个样本必须包含

```json
{
  "id": "唯一标识符",
  "category": "分类",
  "difficulty": "easy|medium|hard|expert",
  "code": "真实代码（非简化示例）",
  "question": "多样化的问题形式",
  "thinking": "详细的推理过程（CoT）",
  "answer": "完整的答案（包含确定性说明）",
  "source": "代码来源",
  "concepts": ["涉及的概念列表"]
}
```

### 5.2 代码要求

- ✅ 使用真实 Linux 内核代码
- ✅ 包含足够的上下文
- ✅ 保留原始注释
- ❌ 不使用简化的模板代码
- ❌ 不使用人工编造的示例

### 5.3 推理过程要求

- ✅ 逐步分析
- ✅ 说明每步的依据
- ✅ 处理分支情况
- ✅ 明确不确定性

### 5.4 答案要求

- ✅ 给出明确结论
- ✅ 说明确定性（100%/多种可能/未知）
- ✅ 包含调用链/时间线
- ✅ 注明关键点

---

## 6. 实施进度跟踪

### 6.1 Phase 1 进度

| 模块 | 目标样本 | 已完成 | 进度 | 状态 |
|------|----------|--------|------|------|
| 总线驱动框架 | 3,000 | 0 | 0% | 🔴 待开始 |
| 字符设备框架 | 1,500 | 0 | 0% | 🔴 待开始 |
| 异步机制 | 2,500 | 0 | 0% | 🔴 待开始 |
| 同步原语 | 1,500 | 0 | 0% | 🔴 待开始 |
| 设备模型 | 1,500 | 0 | 0% | 🔴 待开始 |
| **Phase 1 总计** | **10,000** | **0** | **0%** | |

### 6.2 总体进度

| 阶段 | 目标样本 | 已完成 | 进度 | 状态 |
|------|----------|--------|------|------|
| Phase 1 | 10,000 | 0 | 0% | 🔴 待开始 |
| Phase 2 | 15,000 | 0 | 0% | ⚪ 未开始 |
| Phase 3 | 15,000 | 0 | 0% | ⚪ 未开始 |
| Phase 4 | 20,000 | 0 | 0% | ⚪ 未开始 |
| Phase 5 | 10,000 | 0 | 0% | ⚪ 未开始 |
| **总计** | **70,000** | **0** | **0%** | |

---

## 更新日志

| 日期 | 更新内容 |
|------|----------|
| 2026-01-07 | 创建完整训练数据计划 |

