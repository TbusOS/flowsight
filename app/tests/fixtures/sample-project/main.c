/**
 * FlowSight 测试示例项目 - 主程序
 * 
 * 这是一个简单的 C 程序，用于测试 FlowSight 的代码分析功能。
 * 包含入口函数、初始化、数据处理和清理逻辑。
 */

#include "utils.h"
#include <stdio.h>
#include <stdlib.h>

/* 全局状态 */
static int g_initialized = 0;
static int g_data_count = 0;

/**
 * 入口函数
 * 
 * 程序主入口，执行完整的初始化-处理-清理流程。
 * 
 * @return 0 表示成功，非零表示错误
 */
int main(void)
{
    int ret;
    
    printf("FlowSight Sample Project\n");
    printf("========================\n\n");
    
    /* 初始化系统 */
    ret = init_system();
    if (ret != 0) {
        fprintf(stderr, "Failed to initialize system: %d\n", ret);
        return ret;
    }
    
    /* 处理数据 */
    process_data();
    
    /* 清理资源 */
    cleanup();
    
    printf("\nProgram completed successfully.\n");
    return 0;
}

/**
 * 初始化系统
 * 
 * 设置全局状态，准备数据处理环境。
 * 
 * @return 0 表示成功
 */
int init_system(void)
{
    printf("Initializing system...\n");
    
    g_initialized = 1;
    g_data_count = 0;
    
    printf("System initialized.\n");
    return 0;
}

/**
 * 处理数据
 * 
 * 执行示例计算并打印结果。
 * 演示函数调用链: process_data -> calculate -> print_result
 */
void process_data(void)
{
    int result;
    
    if (!g_initialized) {
        fprintf(stderr, "Error: System not initialized!\n");
        return;
    }
    
    printf("\nProcessing data...\n");
    
    /* 执行计算 */
    result = calculate(10, 20);
    print_result(result);
    
    result = calculate(100, 200);
    print_result(result);
    
    result = calculate(-5, 15);
    print_result(result);
    
    g_data_count = 3;
    printf("Processed %d data items.\n", g_data_count);
}

/**
 * 清理资源
 * 
 * 释放资源，重置全局状态。
 */
void cleanup(void)
{
    printf("\nCleaning up...\n");
    
    g_initialized = 0;
    g_data_count = 0;
    
    printf("Cleanup completed.\n");
}

/**
 * 异步回调示例 (模拟)
 * 
 * 这是一个模拟的回调函数，用于测试 FlowSight 的回调检测功能。
 * 在实际的内核代码中，这类函数会被注册为中断处理器或工作队列处理器。
 * 
 * @param data 回调数据指针
 */
void sample_callback(void *data)
{
    printf("Callback invoked with data: %p\n", data);
    
    /* 模拟回调处理逻辑 */
    if (data != NULL) {
        int *value = (int *)data;
        printf("Callback data value: %d\n", *value);
    }
}

/**
 * 错误处理函数
 * 
 * 统一的错误处理入口。
 * 
 * @param error_code 错误码
 * @param message 错误消息
 */
void handle_error(int error_code, const char *message)
{
    fprintf(stderr, "Error [%d]: %s\n", error_code, message);
    
    /* 执行清理 */
    cleanup();
    
    /* 退出程序 */
    exit(error_code);
}
