/**
 * FlowSight 测试示例项目 - 工具函数
 * 
 * 提供计算和输出功能。
 */

#include "utils.h"
#include <stdio.h>

/**
 * 计算两个整数的和
 * 
 * @param a 第一个操作数
 * @param b 第二个操作数
 * @return 两数之和
 */
int calculate(int a, int b)
{
    int result;
    
    printf("  Calculating: %d + %d\n", a, b);
    result = a + b;
    
    return result;
}

/**
 * 打印计算结果
 * 
 * @param result 要打印的结果值
 */
void print_result(int result)
{
    printf("  Result: %d\n", result);
    
    /* 额外的结果分析 */
    if (result > 100) {
        printf("  (Large value)\n");
    } else if (result < 0) {
        printf("  (Negative value)\n");
    } else {
        printf("  (Normal value)\n");
    }
}

/**
 * 验证输入参数
 * 
 * @param value 要验证的值
 * @param min 最小值
 * @param max 最大值
 * @return 1 如果有效，0 如果无效
 */
int validate_input(int value, int min, int max)
{
    if (value < min || value > max) {
        printf("  Validation failed: %d not in [%d, %d]\n", value, min, max);
        return 0;
    }
    return 1;
}

/**
 * 格式化输出
 * 
 * @param prefix 前缀字符串
 * @param value 值
 * @param suffix 后缀字符串
 */
void format_output(const char *prefix, int value, const char *suffix)
{
    printf("%s%d%s\n", prefix, value, suffix);
}
