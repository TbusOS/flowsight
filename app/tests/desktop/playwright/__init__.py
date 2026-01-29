"""
Playwright WebView 测试模块
==========================

提供对 Tauri 应用内部 WebView 的自动化测试能力。

功能:
- WebView 检测和连接
- Chrome DevTools Protocol (CDP) 集成
- DOM 操作和断言
- 性能监控
- 控制台日志捕获

使用示例:
    from app.tests.desktop.playwright import WebViewAdapter, WebViewTests

    # 连接到 Tauri WebView
    adapter = WebViewAdapter()
    await adapter.connect_to_tauri_app("FlowSight")

    # 运行 WebView 测试
    tests = WebViewTests(adapter)
    report = await tests.test_all()
"""

from .webview_detector import WebViewDetector, WebViewInfo, WebViewType
from .webview_adapter import WebViewAdapter, WebViewConnectionError
from .webview_tests import WebViewTests, WebViewTestReport

__all__ = [
    'WebViewDetector',
    'WebViewInfo',
    'WebViewType',
    'WebViewAdapter',
    'WebViewConnectionError',
    'WebViewTests',
    'WebViewTestReport',
]
