"""
核心模块
========

DesktopAdapter    - 跨平台桌面自动化适配器
VisualAnalyzer    - AI视觉分析器
TestRunner        - 测试运行器
"""

from .desktop_adapter import (
    DesktopAdapter,
    DesktopPlatformAdapter,
    MacOSAdapter,
    WindowsAdapter,
    LinuxAdapter,
    ScreenshotResult,
    UIElement,
    Point,
    Rect
)
from .visual_analyzer import (
    VisualAnalyzer,
    VisualAnalysis,
    UIElement
)
from .test_runner import (
    TestRunner,
    TestPhase,
    TestPriority,
    TestCase,
    TestResult,
    PhaseResult,
    TestSuiteReport
)

__all__ = [
    # Desktop Adapter
    'DesktopAdapter',
    'DesktopPlatformAdapter',
    'MacOSAdapter',
    'WindowsAdapter',
    'LinuxAdapter',
    'ScreenshotResult',
    'UIElement',
    'Point',
    'Rect',
    # Visual Analyzer
    'VisualAnalyzer',
    'VisualAnalysis',
    'UIElement',
    # Test Runner
    'TestRunner',
    'TestPhase',
    'TestPriority',
    'TestCase',
    'TestResult',
    'PhaseResult',
    'TestSuiteReport',
]
