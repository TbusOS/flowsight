#!/usr/bin/env python3
"""
FlowSight Desktop UI 测试框架使用示例
======================================

演示如何运行自动化UI测试来验证FlowSight IDE的视觉质量。

使用方法:
    1. 确保FlowSight应用正在运行
    2. 设置环境变量: export ANTHROPIC_API_KEY="your-key"
    3. 运行: python example.py
"""

import asyncio
import os
import sys
import logging

# 设置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# 添加父目录到路径
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from desktop import (
    DesktopAdapter,
    VisualAnalyzer,
    TestRunner,
    AestheticEvaluator,
)


async def main():
    """主函数 - 运行完整的UI测试套件"""

    # 检查 API 密钥
    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        logger.error("请设置 ANTHROPIC_API_KEY 环境变量")
        logger.error("例如: export ANTHROPIC_API_KEY='your-api-key'")
        return 1

    # 初始化组件
    logger.info("🚀 初始化测试框架...")

    # 1. 创建桌面适配器
    adapter = DesktopAdapter.for_current_platform("FlowSight")
    logger.info(f"✓ 桌面适配器已创建 (平台: {adapter.platform.value})")

    # 2. 创建视觉分析器
    visual_analyzer = VisualAnalyzer(api_key=api_key)
    logger.info("✓ 视觉分析器已创建")

    # 3. 创建美学评估器
    aesthetic_evaluator = AestheticEvaluator(api_key=api_key)
    logger.info("✓ 美学评估器已创建")

    # 4. 创建测试运行器
    runner = TestRunner(
        adapter=adapter,
        visual_analyzer=visual_analyzer,
        aesthetic_evaluator=aesthetic_evaluator,
        output_dir="./reports"
    )
    logger.info("✓ 测试运行器已创建")

    # 5. 注册默认测试套件
    runner.register_default_tests()
    logger.info("✓ 默认测试套件已注册")

    # 6. 运行测试
    logger.info("\n" + "="*60)
    logger.info("开始执行测试...")
    logger.info("="*60)

    try:
        report = await runner.run_all_tests()

        # 打印结果摘要
        logger.info("\n" + "="*60)
        logger.info("测试结果摘要")
        logger.info("="*60)
        logger.info(f"总测试数: {report.total_tests}")
        logger.info(f"通过: {report.passed_tests} ✅")
        logger.info(f"失败: {report.failed_tests} ❌")
        logger.info(f"跳过: {report.skipped_tests} ⏭️")
        logger.info(f"通过率: {report.passed_tests/report.total_tests*100:.1f}%")
        logger.info("-"*60)
        logger.info(f"总体评分: {report.overall_score:.1f}/100")
        logger.info(f"美学评分: {report.aesthetic_score:.1f}/100")
        logger.info(f"可用性评分: {report.usability_score:.1f}/100")
        logger.info(f"性能评分: {report.performance_score:.1f}/100")
        logger.info("="*60)

        # 返回退出码
        return 0 if report.overall_score >= 70 else 1

    except Exception as e:
        logger.exception("测试执行失败")
        return 1


async def run_quick_test():
    """快速测试 - 只运行关键测试"""
    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        logger.error("请设置 ANTHROPIC_API_KEY 环境变量")
        return 1

    adapter = DesktopAdapter.for_current_platform("FlowSight")
    visual_analyzer = VisualAnalyzer(api_key=api_key)
    aesthetic_evaluator = AestheticEvaluator(api_key=api_key)

    runner = TestRunner(
        adapter=adapter,
        visual_analyzer=visual_analyzer,
        aesthetic_evaluator=aesthetic_evaluator,
        output_dir="./reports"
    )

    # 只运行关键测试
    report = await runner.run_quick_smoke_test()

    logger.info("\n快速测试结果:")
    logger.info(f"通过: {report.passed_tests}/{report.total_tests}")
    logger.info(f"总体评分: {report.overall_score:.1f}/100")

    return 0 if report.overall_score >= 60 else 1


async def run_aesthetic_test():
    """专注于美学设计的测试"""
    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        logger.error("请设置 ANTHROPIC_API_KEY 环境变量")
        return 1

    adapter = DesktopAdapter.for_current_platform("FlowSight")
    visual_analyzer = VisualAnalyzer(api_key=api_key)
    aesthetic_evaluator = AestheticEvaluator(api_key=api_key)

    runner = TestRunner(
        adapter=adapter,
        visual_analyzer=visual_analyzer,
        aesthetic_evaluator=aesthetic_evaluator,
        output_dir="./reports"
    )

    # 只运行美学相关测试
    report = await runner.run_aesthetic_focused_test()

    if report.aesthetic_report:
        logger.info("\n美学评估结果:")
        logger.info(f"总体美学评分: {report.aesthetic_report.overall_score:.1f}/100")

        for system, score in report.aesthetic_report.design_system_scores.items():
            logger.info(f"  {system}: {score:.1f}/100")

        # 打印设计建议
        if report.aesthetic_report.recommendations:
            logger.info("\n设计改进建议:")
            for i, rec in enumerate(report.aesthetic_report.recommendations[:5], 1):
                logger.info(f"  {i}. [{rec.priority.upper()}] {rec.suggestion}")

    return 0


if __name__ == "__main__":
    # 根据命令行参数选择测试模式
    import argparse

    parser = argparse.ArgumentParser(
        description="FlowSight Desktop UI 自动化测试"
    )
    parser.add_argument(
        "--mode",
        choices=["full", "quick", "aesthetic"],
        default="full",
        help="测试模式: full=完整测试, quick=快速测试, aesthetic=美学测试"
    )

    args = parser.parse_args()

    if args.mode == "quick":
        exit_code = asyncio.run(run_quick_test())
    elif args.mode == "aesthetic":
        exit_code = asyncio.run(run_aesthetic_test())
    else:
        exit_code = asyncio.run(main())

    sys.exit(exit_code)
