"""
FlowSight Desktop UI 自动化测试框架
===================================

一个AI驱动的桌面IDE UI自动化测试工具，类似Claude Computer Use。

主要功能:
- 跨平台桌面自动化 (macOS/Windows/Linux)
- AI视觉理解 (截图分析、元素识别)
- 视觉美学评估 (Cursor/macOS/21st/shadcn设计规范)
- 五大测试模块 (Layout/Visual/Interactive/State/WebView)
- Playwright WebView集成 (CDP协议测试Tauri WebView)
- 综合测试报告生成

快速开始:
    from app.tests.desktop import DesktopAdapter, TestRunner

    # 初始化适配器
    adapter = DesktopAdapter.for_current_platform("FlowSight")

    # 创建测试运行器
    runner = TestRunner(adapter, visual_analyzer)
    runner.register_default_tests()

    # 运行测试
    report = await runner.run_all_tests()

模块结构:
    core/           - 核心模块
        desktop_adapter.py   - 跨平台适配层
        visual_analyzer.py   - AI视觉分析
        test_runner.py       - 测试运行器

    aesthetic/      - 美学评估
        aesthetic_evaluator.py - 视觉美学评估

    tests/          - 测试模块
        layout_tests.py      - 布局测试
        visual_tests.py      - 视觉测试
        interactive_tests.py - 交互测试
        state_tests.py       - 状态测试

    playwright/     - Playwright WebView测试
        webview_detector.py  - WebView检测器
        webview_adapter.py   - Playwright适配器
        webview_tests.py     - WebView测试套件

    reports/        - 测试报告输出目录
"""

__version__ = "0.1.0"
__author__ = "FlowSight Team"

# 便捷导入
from .core import (
    DesktopAdapter,
    DesktopPlatformAdapter,
    VisualAnalyzer,
    TestRunner,
    TestPhase,
    TestPriority,
    TestSuiteReport,
    Point,
    Rect,
)
from .aesthetic import AestheticEvaluator, AestheticReport
from .tests import LayoutTests, VisualTests, InteractiveTests, StateTests
from .playwright import (
    WebViewAdapter,
    WebViewTests,
    WebViewTestReport,
    WebViewDetector,
    WebViewInfo,
)

__all__ = [
    # 核心类
    'DesktopAdapter',
    'DesktopPlatformAdapter',
    'VisualAnalyzer',
    'TestRunner',
    # 枚举和报告
    'TestPhase',
    'TestPriority',
    'TestSuiteReport',
    'AestheticReport',
    'WebViewTestReport',
    # 数据类
    'Point',
    'Rect',
    'WebViewInfo',
    # 测试模块
    'AestheticEvaluator',
    'LayoutTests',
    'VisualTests',
    'InteractiveTests',
    'StateTests',
    # Playwright WebView
    'WebViewAdapter',
    'WebViewTests',
    'WebViewDetector',
]
