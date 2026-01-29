#!/usr/bin/env python3
"""
桌面自动化测试入口
==================

使用方法:
    # 从 app 目录运行
    cd app && python3 -m tests.desktop --smoke

    # 运行特定阶段
    python3 -m tests.desktop --target "FlowSight" --phase visual

    # 查看帮助
    python3 -m tests.desktop --help
"""

import sys
import os
import asyncio

# 确保 tests.desktop 在路径中
desktop_path = os.path.dirname(os.path.abspath(__file__))
app_path = os.path.dirname(os.path.dirname(desktop_path))

if app_path not in sys.path:
    sys.path.insert(0, app_path)

from tests.desktop.core.test_runner import TestRunner, TestPhase
from tests.desktop.playwright.webview_adapter import WebViewAdapter
from tests.desktop.playwright.webview_tests import WebViewTests


async def run_webview_tests(target: str, phases: list = None, verbose: bool = False):
    """运行 WebView 测试 (适用于 Tauri/Electron 应用)"""
    print(f"\n正在连接到: {target}")

    adapter = WebViewAdapter()
    connected = await adapter.connect_to_tauri_app(target, timeout=30)

    if not connected:
        print(f"❌ 无法连接到 {target}")
        print("请确保应用正在运行 (如 `pnpm tauri dev`)，然后重试。")
        return None

    print(f"✅ 已连接到 {target} (WebView)")

    tests = WebViewTests(adapter, verbose=verbose)

    if phases:
        phase_names = [p.value for p in phases]
        print(f"\n运行测试阶段: {', '.join(phase_names)}")
    else:
        print("\n运行完整测试套件")

    report = await tests.test_all(phases=phases)
    return report


def main():
    """主入口"""
    import argparse

    parser = argparse.ArgumentParser(
        description="FlowSight 桌面自动化 UI 测试",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例:
    python3 -m tests.desktop --smoke              # 冒烟测试
    python3 -m tests.desktop --phase visual       # 视觉测试
    python3 -m tests.desktop --phase interactive  # 交互测试
    python3 -m tests.desktop --full               # 完整测试
        """
    )

    parser.add_argument(
        "--target",
        default="FlowSight",
        help="测试目标 (默认: FlowSight)"
    )

    parser.add_argument(
        "--phase",
        choices=["layout", "visual", "interactive", "state", "aesthetic"],
        help="运行特定测试阶段"
    )

    parser.add_argument(
        "--smoke",
        action="store_true",
        help="运行冒烟测试 (layout + visual)"
    )

    parser.add_argument(
        "--full",
        action="store_true",
        help="运行完整测试套件"
    )

    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="详细输出"
    )

    args = parser.parse_args()

    # 确定运行哪些阶段
    phases = None
    if args.phase:
        phases = [TestPhase(args.phase)]
    elif args.smoke:
        phases = [TestPhase.LAYOUT, TestPhase.VISUAL]

    print(f"\n{'='*60}")
    print(f"FlowSight 桌面自动化测试")
    print(f"{'='*60}")
    print(f"目标: {args.target}")
    print(f"模式: WebView (Tauri/CDP)")
    print(f"阶段: {args.phase or '全部' if not args.smoke else 'smoke'}")
    print(f"{'='*60}\n")

    # 运行异步测试
    try:
        report = asyncio.run(run_webview_tests(
            target=args.target,
            phases=phases,
            verbose=args.verbose
        ))

        if report is None:
            print("\n测试未完成 - 无法连接到应用")
            sys.exit(1)

        # 输出结果摘要
        print(f"\n{'='*60}")
        print("测试结果摘要")
        print(f"{'='*60}")
        print(f"总测试数: {report.total_tests}")
        print(f"通过: {report.passed_tests} ✅")
        print(f"失败: {report.failed_tests} ❌")
        print(f"跳过: {report.skipped_tests} ⏭️")
        if hasattr(report, 'duration_ms'):
            print(f"耗时: {report.duration_ms:.2f}ms")
        print(f"{'='*60}\n")

        # 退出码
        sys.exit(0 if report.failed_tests == 0 else 1)

    except ImportError as e:
        print(f"\n❌ 导入错误: {e}")
        print("\n可能需要安装 Playwright:")
        print("  pip install playwright")
        print("  playwright install")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ 测试失败: {e}")
        if args.verbose:
            import traceback
            traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()
