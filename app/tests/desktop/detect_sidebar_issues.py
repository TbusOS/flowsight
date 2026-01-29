#!/usr/bin/env python3
"""
边栏截断检测 - 增强版
====================

专门检测边栏底部菜单项被截断的问题。
这是FlowSight UI中的常见问题（设置菜单显示不全）。
"""

import asyncio
import os
import sys
from pathlib import Path

# 添加路径
current_dir = Path(__file__).parent
for p in [str(current_dir), str(current_dir.parent)]:
    if p not in sys.path:
        sys.path.insert(0, p)

from desktop import DesktopAdapter
from desktop.core.desktop_adapter import ScreenshotResult
from PIL import Image
import logging

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def analyze_sidebar_bottom(screenshot: Image.Image) -> dict:
    """
    分析边栏底部区域，检测"设置"等菜单项是否被截断
    """
    width, height = screenshot.size
    sidebar_width = int(width * 0.12)

    # 提取边栏
    sidebar = screenshot.crop((0, 0, sidebar_width, height))
    sidebar_rgb = sidebar.convert('RGB')

    results = {
        "sidebar_bounds": {"x": 0, "y": 0, "width": sidebar_width, "height": height},
        "items": [],
        "truncation_detected": False,
        "truncated_items": [],
        "severity": "none",  # none, low, medium, high
        "recommendations": []
    }

    # 将边栏分成6个区域（FlowSight边栏有6个主要项目）
    item_height = height // 6

    for i in range(6):
        y_start = i * item_height
        y_end = (i + 1) * item_height

        # 提取该项目区域
        item_region = sidebar.crop((0, y_start, sidebar_width, y_end))
        item_rgb = item_region.convert('RGB')
        pixels = list(item_rgb.getdata())

        # 计算亮度分布
        brightness = [sum(p) / 3 for p in pixels]

        # 检查右侧是否有内容
        right_quarter = [p for idx, p in enumerate(pixels)
                        if (idx % sidebar_width) >= sidebar_width * 0.75]
        right_brightness = [sum(p) / 3 for p in right_quarter]

        # 左侧内容
        left_quarter = [p for idx, p in enumerate(pixels)
                       if (idx % sidebar_width) < sidebar_width * 0.25]
        left_brightness = [sum(p) / 3 for p in left_quarter]

        # 计算内容密度（亮度变化大=有内容）
        left_variance = max(left_brightness) - min(left_brightness)
        right_variance = max(right_brightness) - min(right_brightness)

        item_info = {
            "index": i,
            "y_range": (y_start, y_end),
            "left_variance": left_variance,
            "right_variance": right_variance,
            "has_icon": left_variance > 30,
            "has_text": right_variance > 20,
        }

        # 检测截断：如果左侧有图标但右侧内容缺失
        if left_variance > 30 and right_variance < 10 and i >= 4:
            item_info["truncated"] = True
            results["truncation_detected"] = True
            results["severity"] = "high"
            results["truncated_items"].append({
                "index": i,
                "type": "right_side_content_missing",
                "left_variance": left_variance,
                "right_variance": right_variance
            })
            results["recommendations"].append(
                f"Item {i} (index) appears truncated - right side content missing"
            )
        else:
            item_info["truncated"] = False

        results["items"].append(item_info)

    # 检查整体边栏完整性
    bottom_region = sidebar.crop((0, int(height * 0.85), sidebar_width, height))
    bottom_pixels = list(bottom_region.convert('RGB').getdata())

    # 底部边缘亮度检测
    bottom_edge_brightness = [sum(p) / 3 for p in bottom_pixels[:100]]
    has_content_at_bottom = any(b < 200 for b in bottom_edge_brightness)

    if not has_content_at_bottom and results["truncation_detected"]:
        results["severity"] = "medium"
        results["recommendations"].append(
            "Consider increasing sidebar height or adding scroll"
        )

    return results


def check_layout_overflow(screenshot: Image.Image) -> dict:
    """
    检测布局溢出问题 - 内容超出容器边界
    """
    width, height = screenshot.size

    # 检测右侧溢出（如果边栏太宽）
    right_edge = screenshot.crop((width - 50, 0, width, height))
    right_pixels = list(right_edge.convert('RGB').getdata())

    # 检测左侧溢出
    left_edge = screenshot.crop((0, 0, 50, height))
    left_pixels = list(left_edge.convert('RGB').getdata())

    results = {
        "right_edge_content": False,
        "left_edge_content": False,
        "overflow_detected": False
    }

    # 检查边缘是否有内容（可能表示溢出）
    right_has_content = any(sum(p) < 150 for p in right_pixels[:500])
    left_has_content = any(sum(p) < 150 for p in left_pixels[:500])

    results["right_edge_content"] = right_has_content
    results["left_edge_content"] = left_has_content

    if right_has_content and left_has_content:
        results["overflow_detected"] = True

    return results


async def detect_sidebar_issues():
    """主检测函数"""
    logger.info("=" * 60)
    logger.info("边栏截断问题检测")
    logger.info("=" * 60)

    # 截图
    adapter = DesktopAdapter()
    screenshot_result = adapter.get_screenshot()

    if isinstance(screenshot_result, ScreenshotResult):
        screenshot = screenshot_result.image
    else:
        screenshot = screenshot_result

    # 保存截图
    screenshot.save("/tmp/detection_screenshot.png")
    logger.info("📸 截图已保存")

    # 分��边栏
    logger.info("\n🔍 分析边栏...")
    sidebar_result = analyze_sidebar_bottom(screenshot)

    logger.info(f"\n边栏分析结果:")
    logger.info(f"  边栏宽度: {sidebar_result['sidebar_bounds']['width']}px")
    logger.info(f"  边栏高度: {sidebar_result['sidebar_bounds']['height']}px")
    logger.info(f"  截断检测: {'✓' if sidebar_result['truncation_detected'] else '✗'}")
    logger.info(f"  严重程度: {sidebar_result['severity']}")

    for item in sidebar_result["items"]:
        status = "⚠️ 截断" if item["truncated"] else "✓"
        logger.info(f"  项目 {item['index']}: {status} (左:{item['left_variance']:.1f}, 右:{item['right_variance']:.1f})")

    # 检查布局溢出
    logger.info("\n🔍 检查布局溢出...")
    overflow_result = check_layout_overflow(screenshot)
    logger.info(f"  右侧溢出: {'✓' if overflow_result['right_edge_content'] else '✗'}")
    logger.info(f"  左侧溢出: {'✓' if overflow_result['left_edge_content'] else '✗'}")

    # 综合结论
    logger.info("\n" + "=" * 60)
    logger.info("检测结论")
    logger.info("=" * 60)

    has_issues = sidebar_result["truncation_detected"] or overflow_result["overflow_detected"]

    if has_issues:
        if sidebar_result["truncation_detected"]:
            logger.error("❌ 边栏截断问题已检测!")
            logger.error(f"   严重程度: {sidebar_result['severity']}")
            logger.error(f"   受影响项目: {len(sidebar_result['truncated_items'])}")

            for item in sidebar_result["truncated_items"]:
                logger.error(f"     - 项目 {item['index']}: 右方差={item['right_variance']:.1f}")

            logger.info("\n📋 修复建议:")
            for rec in sidebar_result["recommendations"]:
                logger.info(f"   • {rec}")
        else:
            logger.warning("⚠️ 检测到布局溢出问题")

        return {
            "status": "issues_detected",
            "sidebar": sidebar_result,
            "overflow": overflow_result
        }
    else:
        logger.info("✓ 未检测到明显的UI问题")
        return {
            "status": "normal",
            "sidebar": sidebar_result,
            "overflow": overflow_result
        }


async def main():
    result = await detect_sidebar_issues()

    # 保存报告
    import json
    report_path = "/tmp/sidebar_issues_report.json"
    with open(report_path, 'w', encoding='utf-8') as f:
        json.dump(result, f, indent=2, ensure_ascii=False)
    logger.info(f"\n📄 报告已保存: {report_path}")

    return 0 if result["status"] == "normal" else 1


if __name__ == "__main__":
    exit_code = asyncio.run(main())
    sys.exit(exit_code)
