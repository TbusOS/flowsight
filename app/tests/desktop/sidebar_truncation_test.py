#!/usr/bin/env python3
"""
边栏截断检测测试
================

演示如何使用测试框架检测UI截断问题（如边栏显示不全）。

这个测试会:
1. 截图并分析边栏区域
2. 检测菜单项是否被截断
3. 检查元素完整性
"""

import asyncio
import os
import sys
from pathlib import Path

# 添加测试框架路径
current_dir = Path(__file__).parent
tests_dir = current_dir
app_dir = tests_dir.parent
project_root = app_dir.parent

for path in [str(tests_dir), str(app_dir), str(project_root)]:
    if path not in sys.path:
        sys.path.insert(0, path)

from desktop import (
    DesktopAdapter,
    VisualAnalyzer,
    TestRunner,
    AestheticEvaluator,
    TestPhase,
    TestPriority,
)
from desktop.core.desktop_adapter import ScreenshotResult
from PIL import Image, ImageDraw
import logging

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def detect_sidebar_truncation(screenshot: Image.Image) -> dict:
    """
    检测边栏是否被截断

    通过分析截图中的边栏区域，检查底部菜单项是否完整显示
    """
    width, height = screenshot.size
    results = {
        "sidebar_detected": False,
        "sidebar_bounds": None,
        "items_found": [],
        "truncated_items": [],
        "issue_detected": False,
        "issue_description": None,
    }

    # 边栏通常在左侧，宽度约为总宽度的 10-15%
    sidebar_width = int(width * 0.12)
    sidebar_area = screenshot.crop((0, 0, sidebar_width, height))

    # 转换为RGB进行分析
    sidebar_rgb = sidebar_area.convert('RGB')
    pixels = list(sidebar_rgb.getdata())

    # 边栏背景通常是深色的，检测深色区域
    dark_threshold = 100  # RGB值低于这个认为是非背景
    dark_pixels = sum(1 for p in pixels if sum(p) < dark_threshold * 3)
    dark_ratio = dark_pixels / len(pixels)

    if dark_ratio > 0.3:
        results["sidebar_detected"] = True
        results["sidebar_bounds"] = {
            "x": 0,
            "y": 0,
            "width": sidebar_width,
            "height": height
        }

        # 分割边栏为多个区域，检测菜单项
        item_regions = []
        item_height = height // 6  # 假设有6个主要项目

        for i in range(6):
            y_start = i * item_height
            y_end = (i + 1) * item_height
            region = sidebar_area.crop((0, y_start, sidebar_width, y_end))
            region_rgb = region.convert('RGB')

            # 检测是否有文本/图标内容（非纯色）
            region_pixels = list(region_rgb.getdata())
            unique_colors = len(set(region_pixels))

            # 计算颜色亮度方差
            brightness_values = [sum(p) / 3 for p in region_pixels]
            brightness_variance = max(brightness_values) - min(brightness_values) if brightness_values else 0

            item_info = {
                "index": i,
                "y_range": (y_start, y_end),
                "has_content": unique_colors > 100,
                "color_variance": brightness_variance
            }
            results["items_found"].append(item_info)

            # 检测底部项目是否被截断
            # 底部项目如果接近窗口边缘，可能被截断
            if i >= 4:  # 最后两个项目
                bottom_edge = y_end
                if bottom_edge > height - 50:  # 接近底部
                    # 检查边缘是否有渐变或截断
                    edge_region = sidebar_area.crop((0, height - 30, sidebar_width, height))
                    edge_pixels = list(edge_region.getdata())
                    # 如果底部边缘有内容但突然消失，可能是截断
                    item_info["potentially_truncated"] = True
                    results["truncated_items"].append({
                        "index": i,
                        "reason": "Item near bottom edge, may be truncated"
                    })

        # 额外的截断检测：检查底部是否有突然的内容消失
        bottom_region = sidebar_area.crop((0, int(height * 0.8), sidebar_width, height))
        bottom_pixels = list(bottom_region.getdata())

        # 检测底部区域是否有内容但没有完整显示
        has_content_in_bottom = any(sum(p) < 200 for p in bottom_pixels[:100])  # 左上角
        has_content_in_bottom_right = any(sum(p) < 200 for p in bottom_pixels[-100:])  # 右上角

        if has_content_in_bottom and not has_content_in_bottom_right:
            results["issue_detected"] = True
            results["issue_description"] = "Sidebar content appears truncated at bottom-right corner"

        # 检查"设置"菜单项完整性
        settings_region = sidebar_area.crop((0, int(height * 0.83), sidebar_width, height))
        settings_pixels = list(settings_region.convert('RGB').getdata())

        # 检测设置区域是否有不完整的元素
        left_half = [p for i, p in enumerate(settings_pixels) if (i % sidebar_width) < sidebar_width // 2]
        right_half = [p for i, p in enumerate(settings_pixels) if (i % sidebar_width) >= sidebar_width // 2]

        # 计算亮度方差
        left_brightness = [sum(p) / 3 for p in left_half]
        right_brightness = [sum(p) / 3 for p in right_half]

        left_variance = max(left_brightness) - min(left_brightness) if left_brightness else 0
        right_variance = max(right_brightness) - min(right_brightness) if right_brightness else 0

        if left_variance > 50 and right_variance < 10:
            results["issue_detected"] = True
            results["issue_description"] = "Settings item appears truncated (right side missing)"

    return results


def check_element_continuity(screenshot: Image.Image) -> dict:
    """
    检查元素连续性 - 检测被截断的元素
    """
    width, height = screenshot.size
    issues = []

    # 左侧区域分析
    left_region = screenshot.crop((0, 0, int(width * 0.15), height))
    left_pixels = list(left_region.getdata())

    # 按行分析，找出不连续的边缘
    row_heights = []
    row_size = 10

    for y in range(0, height - row_size, row_size):
        crop_height = min(row_size, height - y)
        row = left_region.crop((0, y, left_region.width, y + crop_height))
        row_pixels = list(row.getdata())

        # 找到有内容的区域
        content_start = None
        content_end = None
        for x, p in enumerate(row_pixels):
            if sum(p) < 200:  # 深色像素
                if content_start is None:
                    content_start = x
                content_end = x

        if content_start is not None:
            row_heights.append({
                "y": y,
                "content_start": content_start,
                "content_end": content_end,
                "content_width": content_end - content_start if content_end else 0
            })

    # 检查是否有宽度突变（可能表示截断）
    for i in range(1, len(row_heights)):
        prev = row_heights[i - 1]
        curr = row_heights[i]

        if prev["content_width"] > 0 and curr["content_width"] > 0:
            width_change = curr["content_width"] - prev["content_width"]
            if abs(width_change) > 10:  # 宽度变化超过10像素
                issues.append({
                    "y": curr["y"],
                    "type": "width_anomaly",
                    "description": f"Element width changed by {width_change}px between rows"
                })

    return {
        "issues_found": len(issues),
        "issues": issues,
        "has_truncation": len(issues) > 2  # 多个异常表示可能截断
    }


async def run_sidebar_test():
    """运行边栏截断检测测试"""
    logger.info("=" * 60)
    logger.info("边栏截断检测测试")
    logger.info("=" * 60)

    # 检查API密钥
    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        logger.error("请设置 ANTHROPIC_API_KEY 环境变量")
        return None

    # 初始化组件
    logger.info("🚀 初始化测试组件...")

    adapter = DesktopAdapter()  # 自动检测平台
    adapter.target_window_title = "FlowSight"  # 设置目标窗口标题
    logger.info(f"✓ 桌面适配器已创建 (平台: {adapter.platform})")

    visual_analyzer = VisualAnalyzer(api_key=api_key)
    aesthetic_evaluator = AestheticEvaluator(api_key=api_key)

    runner = TestRunner(
        adapter=adapter,
        visual_analyzer=visual_analyzer,
        aesthetic_evaluator=aesthetic_evaluator,
        output_dir="./test_reports"
    )

    # 1. 截图
    logger.info("📸 截图...")
    screenshot_result = adapter.get_screenshot()

    # 处理 ScreenshotResult
    if isinstance(screenshot_result, ScreenshotResult):
        screenshot = screenshot_result.image
    else:
        screenshot = screenshot_result

    # 保存原始截图
    screenshot_path = "/tmp/flowsight_sidebar_test.png"
    screenshot.save(screenshot_path)
    logger.info(f"截图已保存: {screenshot_path}")

    # 2. 边栏截断检测
    logger.info("🔍 检测边栏截断...")
    truncation_result = detect_sidebar_truncation(screenshot)

    logger.info(f"\n边栏检测结果:")
    logger.info(f"  - 边栏检测: {'✓' if truncation_result['sidebar_detected'] else '✗'}")
    logger.info(f"  - 找到项目数: {len(truncation_result['items_found'])}")
    logger.info(f"  - 潜在截断项目: {len(truncation_result['truncated_items'])}")
    logger.info(f"  - 问题检测: {'✓' if truncation_result['issue_detected'] else '✗'}")

    if truncation_result['issue_detected']:
        logger.warning(f"  ⚠️ 问题描述: {truncation_result['issue_description']}")

    # 3. 元素连续性检测
    logger.info("\n🔍 检查元素连续性...")
    continuity_result = check_element_continuity(screenshot)

    logger.info(f"\n连续性检测结果:")
    logger.info(f"  - 发现异常数: {continuity_result['issues_found']}")
    logger.info(f"  - 可能截断: {'✓' if continuity_result['has_truncation'] else '✗'}")

    if continuity_result['issues']:
        for issue in continuity_result['issues'][:3]:
            logger.warning(f"  ⚠️ {issue['description']} (位置: y={issue['y']})")

    # 4. 综合判断
    logger.info("\n" + "=" * 60)
    logger.info("综合检测结论")
    logger.info("=" * 60)

    has_issues = truncation_result['issue_detected'] or continuity_result['has_truncation']

    if has_issues:
        logger.error("❌ 检测到UI截断问题!")
        logger.error(f"   {truncation_result.get('issue_description', '边栏元素可能显示不全')}")
    else:
        logger.info("✓ 未检测到明显的UI截断问题")

    # 5. 使用AI视觉分析进行更详细的检测 (如果API key有效)
    logger.info("\n🤖 使用AI进行视觉分析...")
    try:
        analysis = visual_analyzer.analyze_screenshot(screenshot)

        # 查找左侧导航区域
        sidebar_elements = [e for e in analysis.elements if 'sidebar' in e.text.lower() or
                            '导航' in e.text or '设置' in e.text or
                            '菜单' in e.text]

        logger.info(f"AI识别到 {len(sidebar_elements)} 个可能的边栏元素")

        for elem in sidebar_elements[:5]:
            logger.info(f"  - {elem.text[:20]}... (类型: {elem.element_type})")
            if hasattr(elem, 'confidence') and elem.confidence < 0.8:
                logger.warning(f"    ⚠️ 识别置信度较低: {elem.confidence:.2f}")
    except Exception as e:
        logger.warning(f"AI视觉分析跳过: {e}")

    # 生成测试报告
    report = {
        "test_name": "sidebar_truncation_detection",
        "timestamp": str(asyncio.get_event_loop().time()),
        "results": {
            "truncation_check": truncation_result,
            "continuity_check": continuity_result,
        },
        "conclusion": {
            "has_issues": has_issues,
            "issues_description": truncation_result.get('issue_description', None)
        }
    }

    # 保存报告
    import json
    report_path = "/tmp/sidebar_test_report.json"
    with open(report_path, 'w', encoding='utf-8') as f:
        json.dump(report, f, indent=2, ensure_ascii=False)
    logger.info(f"\n📄 报告已保存: {report_path}")

    return report


async def main():
    """主函数"""
    report = await run_sidebar_test()

    if report:
        print("\n" + "=" * 60)
        print("测试完成")
        print("=" * 60)

        if report['conclusion']['has_issues']:
            print("\n❌ UI截断问题已检测!")
            print(f"   问题: {report['conclusion']['issues_description']}")
            print("\n建议:")
            print("   1. 检查边栏容器的 overflow 属性")
            print("   2. 调整边栏高度或容器大小")
            print("   3. 添加滚动支持")
            return 1
        else:
            print("\n✓ UI显示正常")
            return 0
    return 1


if __name__ == "__main__":
    exit_code = asyncio.run(main())
    sys.exit(exit_code)
