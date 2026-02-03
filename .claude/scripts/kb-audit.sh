#!/bin/bash
# Linux 内核知识库审计脚本
# 用于 KB-Auditor Agent

set -e

KERNEL_PATH="${KERNEL_PATH:-/Users/sky/linux-kernel/linux}"
KB_PATH="${KB_PATH:-knowledge/platforms/linux-kernel}"
REPORT_DIR=".claude/reports/kb-audits"

mkdir -p "$REPORT_DIR"

DATE=$(date +%Y-%m-%d)
REPORT_FILE="$REPORT_DIR/audit-$DATE.md"

echo "# 知识库审计报告" > "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "审计时间: $(date)" >> "$REPORT_FILE"
echo "内核路径: $KERNEL_PATH" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# ============================================================
# 1. 核心 API 覆盖率审计
# ============================================================

echo "## 核心 API 覆盖率" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "| API | 内核使用 | 知识库 Pattern | 状态 |" >> "$REPORT_FILE"
echo "|-----|----------|----------------|------|" >> "$REPORT_FILE"

# 核心 API 列表
CORE_APIS=(
    # 内存管理
    "kmalloc" "kzalloc" "kfree" "vmalloc" "vfree"
    "ioremap" "iounmap" "readl" "writel" "readb" "writeb"
    "dma_alloc_coherent" "dma_map_single" "dma_unmap_single"
    "alloc_pages" "free_pages" "__get_free_pages"
    "kmem_cache_create" "kmem_cache_alloc" "kmem_cache_free"
    # 同步原语
    "spin_lock" "spin_unlock" "mutex_lock" "mutex_unlock"
    "down" "up" "rw_lock" "rwlock"
    "atomic_read" "atomic_set" "atomic_add" "atomic_inc"
    "wait_event" "wake_up" "complete" "wait_for_completion"
    # 中断
    "request_irq" "free_irq" "disable_irq" "enable_irq"
    "local_irq_disable" "local_irq_enable" "local_irq_save"
    "tasklet_schedule" "tasklet_init" "tasklet_setup"
    "raise_softirq" "open_softirq"
    # 调度
    "schedule" "schedule_timeout" "cond_resched" "might_sleep"
    "kthread_create" "kthread_run" "kthread_stop"
    "set_current_state" "msleep" "usleep_range"
    # 文件系统
    "register_filesystem" "unregister_filesystem"
    "iget_locked" "iput" "d_alloc" "dput"
    "proc_create" "debugfs_create_file" "sysfs_create_file"
    # 驱动注册
    "platform_driver_register" "pci_register_driver" "usb_register"
    "i2c_add_driver" "spi_register_driver"
    "module_init" "module_exit"
    # 设备模型
    "device_create" "device_destroy" "class_create"
    "dev_set_drvdata" "dev_get_drvdata"
    "devm_kzalloc" "devm_request_irq" "devm_ioremap"
)

for api in "${CORE_APIS[@]}"; do
    kernel_count=$(grep -r "$api\s*(" "$KERNEL_PATH/drivers" "$KERNEL_PATH/kernel" "$KERNEL_PATH/mm" 2>/dev/null | wc -l | tr -d ' ')
    kb_count=$(grep -rE "pattern:.*'.*$api" "$KB_PATH" 2>/dev/null | wc -l | tr -d ' ')
    
    if [ "$kb_count" -eq 0 ]; then
        status="❌ 缺失"
    elif [ "$kb_count" -lt 3 ]; then
        status="⚠️ 不足"
    else
        status="✅ 良好"
    fi
    
    echo "| \`$api\` | $kernel_count | $kb_count | $status |" >> "$REPORT_FILE"
done

echo "" >> "$REPORT_FILE"

# ============================================================
# 2. 按子系统统计
# ============================================================

echo "## 子系统覆盖统计" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

for subdir in core mm sync fs drivers net arch; do
    if [ -d "$KB_PATH/$subdir" ]; then
        file_count=$(find "$KB_PATH/$subdir" -name "*.yaml" | wc -l | tr -d ' ')
        total_lines=$(wc -l "$KB_PATH/$subdir"/*.yaml 2>/dev/null | tail -1 | awk '{print $1}')
        pattern_count=$(grep -r "pattern:" "$KB_PATH/$subdir" 2>/dev/null | wc -l | tr -d ' ')
        callback_count=$(grep -r "callbacks:" "$KB_PATH/$subdir" 2>/dev/null | wc -l | tr -d ' ')
        chain_count=$(grep -r "call_chains:" "$KB_PATH/$subdir" 2>/dev/null | wc -l | tr -d ' ')
        
        echo "### $subdir/" >> "$REPORT_FILE"
        echo "- 文件数: $file_count" >> "$REPORT_FILE"
        echo "- 总行数: $total_lines" >> "$REPORT_FILE"
        echo "- Pattern 数: $pattern_count" >> "$REPORT_FILE"
        echo "- Callback 定义: $callback_count" >> "$REPORT_FILE"
        echo "- 调用链: $chain_count" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    fi
done

# ============================================================
# 3. 缺失 API 发现
# ============================================================

echo "## 高频使用但缺失的 API" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "| API | 内核使用次数 | 建议负责 |" >> "$REPORT_FILE"
echo "|-----|-------------|----------|" >> "$REPORT_FILE"

# 检测高频但缺失的 API
HIGH_FREQ_APIS=(
    "dev_err:KB-Drivers"
    "dev_info:KB-Drivers"
    "dev_dbg:KB-Drivers"
    "clk_prepare_enable:KB-Drivers"
    "clk_disable_unprepare:KB-Drivers"
    "regmap_read:KB-Drivers"
    "regmap_write:KB-Drivers"
    "pm_runtime_get_sync:KB-Drivers"
    "pm_runtime_put:KB-Drivers"
    "of_property_read_u32:KB-Drivers"
    "gpiod_get:KB-Drivers"
    "gpiod_set_value:KB-Drivers"
)

for item in "${HIGH_FREQ_APIS[@]}"; do
    api=$(echo "$item" | cut -d: -f1)
    agent=$(echo "$item" | cut -d: -f2)
    
    kernel_count=$(grep -r "$api\s*(" "$KERNEL_PATH/drivers" 2>/dev/null | wc -l | tr -d ' ')
    kb_count=$(grep -rE "pattern:.*'.*$api" "$KB_PATH" 2>/dev/null | wc -l | tr -d ' ')
    
    if [ "$kb_count" -eq 0 ] && [ "$kernel_count" -gt 100 ]; then
        echo "| \`$api\` | $kernel_count | $agent |" >> "$REPORT_FILE"
    fi
done

echo "" >> "$REPORT_FILE"

# ============================================================
# 4. 审计总结
# ============================================================

total_patterns=$(grep -r "pattern:" "$KB_PATH" 2>/dev/null | wc -l | tr -d ' ')
total_callbacks=$(grep -r "callbacks:" "$KB_PATH" 2>/dev/null | wc -l | tr -d ' ')
total_chains=$(grep -r "call_chains:" "$KB_PATH" 2>/dev/null | wc -l | tr -d ' ')
total_files=$(find "$KB_PATH" -name "*.yaml" | wc -l | tr -d ' ')

echo "## 审计总结" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "- 知识库文件数: $total_files" >> "$REPORT_FILE"
echo "- Pattern 总数: $total_patterns" >> "$REPORT_FILE"
echo "- Callback 定义: $total_callbacks" >> "$REPORT_FILE"
echo "- 调用链: $total_chains" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "审计报告已生成: $REPORT_FILE"
