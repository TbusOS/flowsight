/**
 * FlowSight 测试示例项目 - 工具函数头文件
 * 
 * 声明工具函数和类型。
 */

#ifndef UTILS_H
#define UTILS_H

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * 函数声明
 * ============================================================================ */

/**
 * 计算两个整数的和
 * 
 * @param a 第一个操作数
 * @param b 第二个操作数
 * @return 两数之和
 */
int calculate(int a, int b);

/**
 * 打印计算结果
 * 
 * @param result 要打印的结果值
 */
void print_result(int result);

/**
 * 验证输入参数
 * 
 * @param value 要验证的值
 * @param min 最小值
 * @param max 最大值
 * @return 1 如果有效，0 如果无效
 */
int validate_input(int value, int min, int max);

/**
 * 格式化输出
 * 
 * @param prefix 前缀字符串
 * @param value 值
 * @param suffix 后缀字符串
 */
void format_output(const char *prefix, int value, const char *suffix);

/* ============================================================================
 * main.c 中的函数声明
 * ============================================================================ */

/**
 * 初始化系统
 * @return 0 表示成功
 */
int init_system(void);

/**
 * 处理数据
 */
void process_data(void);

/**
 * 清理资源
 */
void cleanup(void);

/**
 * 异步回调示例
 * @param data 回调数据指针
 */
void sample_callback(void *data);

/**
 * 错误处理函数
 * @param error_code 错误码
 * @param message 错误消息
 */
void handle_error(int error_code, const char *message);

/* ============================================================================
 * 常量定义
 * ============================================================================ */

#define MAX_DATA_SIZE 1024
#define DEFAULT_BUFFER_SIZE 256

/* 错误码 */
#define ERR_SUCCESS      0
#define ERR_INIT_FAILED  1
#define ERR_INVALID_ARG  2
#define ERR_NO_MEMORY    3

#ifdef __cplusplus
}
#endif

#endif /* UTILS_H */
