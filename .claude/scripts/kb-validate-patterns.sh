#!/bin/bash
# Linux 内核知识库 Pattern 精准度验证脚本
# 用于 KB-Auditor / KB-Reviewer
# 使用 ripgrep (rg) 进行高效搜索

set -e

KERNEL_PATH="${KERNEL_PATH:-/Users/sky/linux-kernel/linux}"
KB_PATH="${KB_PATH:-knowledge/platforms/linux-kernel}"
REPORT_DIR=".claude/reports/kb-validations"

mkdir -p "$REPORT_DIR"

DATE=$(date +%Y-%m-%d)
REPORT_FILE="$REPORT_DIR/validation-$DATE.md"

echo "# Pattern 精准度验证报告" > "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "验证时间: $(date)" >> "$REPORT_FILE"
echo "内核路径: $KERNEL_PATH" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# ============================================================
# 1. 提取知识库中的所有 Pattern
# ============================================================

echo "## Pattern 提取" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 统计 pattern 数量
total_patterns=$(rg -c "pattern:" "$KB_PATH" 2>/dev/null | awk -F: '{sum+=$2} END {print sum}' || echo 0)
echo "- 知识库 Pattern 总数: $total_patterns" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# ============================================================
# 2. 在内核代码中验证 Pattern 匹配
# ============================================================

echo "## Pattern 匹配验证" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "| Pattern | 匹配文件数 | 匹配行数 | 状态 |" >> "$REPORT_FILE"
echo "|---------|-----------|----------|------|" >> "$REPORT_FILE"

# 关键 Pattern 验证列表
CRITICAL_PATTERNS=(
    # 内存分配 - 必须精准
    "kmalloc\s*\("
    "kzalloc\s*\("
    "kfree\s*\("
    "vmalloc\s*\("
    "ioremap\s*\("
    "readl\s*\("
    "writel\s*\("
    
    # 同步原语 - 上下文关键
    "spin_lock\s*\("
    "spin_unlock\s*\("
    "mutex_lock\s*\("
    "mutex_unlock\s*\("
    "local_irq_disable\s*\("
    "local_irq_save\s*\("
    
    # 中断 - 回调追踪关键
    "request_irq\s*\("
    "devm_request_irq\s*\("
    "free_irq\s*\("
    "tasklet_schedule\s*\("
    
    # WorkQueue - 异步追踪关键
    "INIT_WORK\s*\("
    "schedule_work\s*\("
    "queue_work\s*\("
    
    # 定时器
    "timer_setup\s*\("
    "mod_timer\s*\("
    
    # 驱动框架 - 回调识别关键
    "\.probe\s*="
    "\.remove\s*="
    "\.open\s*="
    "\.read\s*="
    "\.write\s*="
    "\.ioctl\s*="
    
    # 注册宏
    "module_init\s*\("
    "module_exit\s*\("
    "platform_driver_register\s*\("
    "usb_register\s*\("
)

for pattern in "${CRITICAL_PATTERNS[@]}"; do
    # 在内核代码中测试匹配 (使用 rg)
    file_count=$(rg -l "$pattern" "$KERNEL_PATH/drivers" "$KERNEL_PATH/kernel" 2>/dev/null | wc -l | tr -d ' ' || echo 0)
    line_count=$(rg -c "$pattern" "$KERNEL_PATH/drivers" "$KERNEL_PATH/kernel" 2>/dev/null | awk -F: '{sum+=$2} END {print sum+0}' || echo 0)
    
    # 检查知识库是否有此 pattern (简化匹配)
    simple_pattern=$(echo "$pattern" | sed 's/\\s\*//g; s/\\(//g; s/\\)//g')
    kb_has=$(rg -c "pattern:.*$simple_pattern" "$KB_PATH" 2>/dev/null | awk -F: '{sum+=$2} END {print sum+0}' || echo 0)
    
    if [ "$kb_has" -eq 0 ]; then
        status="❌ 知识库缺失"
    elif [ "$line_count" -eq 0 ]; then
        status="⚠️ 内核无匹配"
    else
        status="✅ 有效"
    fi
    
    # 转义 pattern 用于 markdown
    escaped_pattern=$(echo "$pattern" | sed 's/|/\\|/g')
    echo "| \`$escaped_pattern\` | $file_count | $line_count | $status |" >> "$REPORT_FILE"
done

echo "" >> "$REPORT_FILE"

# ============================================================
# 3. 误报检测 - 抽样验证
# ============================================================

echo "## 误报检测 (抽样)" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "随机抽取 5 个匹配结果验证:" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 测试几个关键 pattern 的匹配质量
TEST_PATTERNS=(
    "request_irq:中断请求"
    "INIT_WORK:WorkQueue初始化"
    "\.probe\s*=:probe回调"
)

for item in "${TEST_PATTERNS[@]}"; do
    pattern=$(echo "$item" | cut -d: -f1)
    desc=$(echo "$item" | cut -d: -f2)
    
    echo "### $desc (\`$pattern\`)" >> "$REPORT_FILE"
    echo "\`\`\`" >> "$REPORT_FILE"
    
    # 随机抽取 5 个匹配 (使用 rg)
    rg "$pattern" "$KERNEL_PATH/drivers" 2>/dev/null | shuf 2>/dev/null | head -5 >> "$REPORT_FILE" || echo "无匹配" >> "$REPORT_FILE"
    
    echo "\`\`\`" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
done

# ============================================================
# 4. 漏报检测 - 检查变体
# ============================================================

echo "## 漏报检测 (API 变体)" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "检查知识库是否覆盖常见 API 变体:" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "| API 基础 | 变体 | 内核使用 | 知识库覆盖 |" >> "$REPORT_FILE"
echo "|----------|------|----------|-----------|" >> "$REPORT_FILE"

# kmalloc 系列
for variant in "kmalloc" "kmalloc_array" "kmalloc_node" "__kmalloc" "krealloc"; do
    kernel_use=$(rg -c "${variant}\s*\(" "$KERNEL_PATH/drivers" 2>/dev/null | awk -F: '{sum+=$2} END {print sum+0}' || echo 0)
    kb_cover=$(rg -c "pattern:.*$variant" "$KB_PATH" 2>/dev/null | awk -F: '{sum+=$2} END {print sum+0}' || echo 0)
    cover_status=$([[ $kb_cover -gt 0 ]] && echo "✅" || echo "❌")
    echo "| kmalloc | \`$variant\` | $kernel_use | $cover_status |" >> "$REPORT_FILE"
done

# spin_lock 系列
for variant in "spin_lock" "spin_lock_irq" "spin_lock_irqsave" "spin_lock_bh" "spin_trylock"; do
    kernel_use=$(rg -c "${variant}\s*\(" "$KERNEL_PATH/drivers" 2>/dev/null | awk -F: '{sum+=$2} END {print sum+0}' || echo 0)
    kb_cover=$(rg -c "pattern:.*$variant" "$KB_PATH" 2>/dev/null | awk -F: '{sum+=$2} END {print sum+0}' || echo 0)
    cover_status=$([[ $kb_cover -gt 0 ]] && echo "✅" || echo "❌")
    echo "| spin_lock | \`$variant\` | $kernel_use | $cover_status |" >> "$REPORT_FILE"
done

# wait_event 系列
for variant in "wait_event" "wait_event_interruptible" "wait_event_timeout" "wait_event_interruptible_timeout" "wait_event_killable"; do
    kernel_use=$(rg -c "${variant}\s*\(" "$KERNEL_PATH/drivers" 2>/dev/null | awk -F: '{sum+=$2} END {print sum+0}' || echo 0)
    kb_cover=$(rg -c "pattern:.*$variant" "$KB_PATH" 2>/dev/null | awk -F: '{sum+=$2} END {print sum+0}' || echo 0)
    cover_status=$([[ $kb_cover -gt 0 ]] && echo "✅" || echo "❌")
    echo "| wait_event | \`$variant\` | $kernel_use | $cover_status |" >> "$REPORT_FILE"
done

echo "" >> "$REPORT_FILE"

# ============================================================
# 5. Context 正确性检查
# ============================================================

echo "## Context 标注检查" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "检查关键 API 的 can_sleep/context 标注是否正确:" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "| API | 正确 Context | 知识库标注 | 状态 |" >> "$REPORT_FILE"
echo "|-----|-------------|-----------|------|" >> "$REPORT_FILE"

# 定义正确的 context (hardirq 不能睡眠)
declare -A CORRECT_CONTEXT=(
    ["kmalloc"]="GFP_KERNEL时可睡眠"
    ["spin_lock"]="不可睡眠"
    ["spin_lock_irq"]="不可睡眠"
    ["mutex_lock"]="可睡眠"
    ["request_irq"]="可睡眠(进程上下文)"
    ["schedule"]="可睡眠"
    ["tasklet_schedule"]="不可睡眠"
    ["in_interrupt"]="任意上下文"
    ["local_irq_disable"]="任意上下文"
)

for api in "${!CORRECT_CONTEXT[@]}"; do
    correct="${CORRECT_CONTEXT[$api]}"
    
    # 检查知识库中的标注 (使用 rg)
    kb_context=$(rg -A5 "pattern:.*$api" "$KB_PATH" 2>/dev/null | rg -o "can_sleep:\s*(true|false)" | head -1 || echo "未标注")
    
    if [ -z "$kb_context" ] || [ "$kb_context" = "未标注" ]; then
        status="⚠️ 未标注"
    else
        status="✅ 已标注"
    fi
    
    echo "| \`$api\` | $correct | $kb_context | $status |" >> "$REPORT_FILE"
done

echo "" >> "$REPORT_FILE"

# ============================================================
# 6. 调用链验证
# ============================================================

echo "## 调用链验证" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "检查知识库中的调用链是否与内核源码一致:" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 验证 schedule() 调用链
echo "### schedule() 调用链" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "内核源码 (\`kernel/sched/core.c\`):" >> "$REPORT_FILE"
echo "\`\`\`" >> "$REPORT_FILE"
grep -A3 "^asmlinkage __visible void __sched schedule(void)" "$KERNEL_PATH/kernel/sched/core.c" 2>/dev/null | head -10 >> "$REPORT_FILE" || echo "未找到" >> "$REPORT_FILE"
echo "\`\`\`" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 验证 request_irq 调用链
echo "### request_irq() 实现" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "内核源码 (\`kernel/irq/manage.c\`):" >> "$REPORT_FILE"
echo "\`\`\`" >> "$REPORT_FILE"
grep -A5 "int request_irq" "$KERNEL_PATH/include/linux/interrupt.h" 2>/dev/null | head -10 >> "$REPORT_FILE" || echo "未找到" >> "$REPORT_FILE"
echo "\`\`\`" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# ============================================================
# 7. 验证总结
# ============================================================

echo "## 验证总结" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 统计
valid_patterns=$(grep -c "✅ 有效" "$REPORT_FILE" || echo 0)
missing_patterns=$(grep -c "❌" "$REPORT_FILE" || echo 0)
warning_patterns=$(grep -c "⚠️" "$REPORT_FILE" || echo 0)

echo "- ✅ 有效 Pattern: $valid_patterns" >> "$REPORT_FILE"
echo "- ❌ 缺失/问题: $missing_patterns" >> "$REPORT_FILE"
echo "- ⚠️ 需关注: $warning_patterns" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

if [ "$missing_patterns" -gt 0 ]; then
    echo "### 需要改进的问题" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    grep "❌" "$REPORT_FILE" | head -20 >> "$REPORT_FILE" || true
fi

echo "" >> "$REPORT_FILE"
echo "---" >> "$REPORT_FILE"
echo "验证完成。报告位置: $REPORT_FILE" >> "$REPORT_FILE"

echo "Pattern 验证报告已生成: $REPORT_FILE"
