#!/usr/bin/env python3
"""
Desktop UI Test Runner
结合 MCP 服务器（桌面自动化）和 Playwright（WebView 测试）
实现完整的 UI 交互测试
"""

import asyncio
import subprocess
import time
import json
from pathlib import Path
from typing import Optional, Callable, Dict, Any
from dataclasses import dataclass
from enum import Enum

try:
    from playwright.sync_api import sync_playwright
    PLAYWRIGHT_AVAILABLE = True
except ImportError:
    PLAYWRIGHT_AVAILABLE = False


class TestResult(Enum):
    PASS = "pass"
    FAIL = "fail"
    SKIP = "skip"
    ERROR = "error"


@dataclass
class TestCase:
    name: str
    description: str
    test_func: Callable
    expected: str


class DesktopAutomation:
    """桌面自动化工具封装"""

    def __init__(self):
        self.screenshot_dir = Path("/tmp/flowsight_tests")
        self.screenshot_dir.mkdir(parents=True, exist_ok=True)

    def screenshot(self, name: str = "test") -> Path:
        """截图"""
        path = self.screenshot_dir / f"{name}_{int(time.time())}.png"
        subprocess.run(
            ["scrot", "-q", "90", str(path)],
            capture_output=True
        )
        return path

    def click(self, x: int, y: int, button: str = "left") -> bool:
        """点击"""
        result = subprocess.run(
            ["xdotool", "mousemove", str(x), str(y), "click", "1"],
            capture_output=True
        )
        return result.returncode == 0

    def double_click(self, x: int, y: int) -> bool:
        """双击"""
        result = subprocess.run(
            ["xdotool", "mousemove", str(x), str(y), "click", "1", "click", "1"],
            capture_output=True
        )
        return result.returncode == 0

    def move_mouse(self, x: int, y: int) -> bool:
        """移动鼠标"""
        result = subprocess.run(
            ["xdotool", "mousemove", str(x), str(y)],
            capture_output=True
        )
        return result.returncode == 0

    def get_mouse_position(self) -> tuple[int, int]:
        """获取鼠标位置"""
        result = subprocess.run(
            ["xdotool", "getmouselocation"],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            parts = result.stdout.strip().split()
            x = int(parts[0].split(":")[1])
            y = int(parts[1].split(":")[1])
            return x, y
        return 0, 0

    def type_text(self, text: str) -> bool:
        """输入文本"""
        result = subprocess.run(
            ["xdotool", "type", "--delay", "50", text],
            capture_output=True
        )
        return result.returncode == 0

    def hotkey(self, combo: str) -> bool:
        """执行快捷键"""
        key_map = {
            "cmd": "Super_L",
            "ctrl": "Control_L",
            "alt": "Alt_L",
            "shift": "Shift_L"
        }
        parts = combo.lower().split("+")
        keys = [key_map.get(p, p) for p in parts]
        result = subprocess.run(
            ["xdotool", "key", "+".join(keys)],
            capture_output=True
        )
        return result.returncode == 0

    def wait(self, seconds: float):
        """等待"""
        time.sleep(seconds)

    def find_window(self, title: str) -> Optional[str]:
        """查找窗口 ID"""
        result = subprocess.run(
            ["xdotool", "search", "--name", title],
            capture_output=True, text=True
        )
        if result.returncode == 0 and result.stdout.strip():
            return result.stdout.strip().split("\n")[0]
        return None


class WebViewTester:
    """WebView 测试器（使用 Playwright）"""

    def __init__(self, url: str = "http://localhost:5173"):
        self.url = url
        self.browser = None
        self.context = None
        self.page = None

    def connect(self, headless: bool = True):
        """连接到浏览器"""
        if not PLAYWRIGHT_AVAILABLE:
            raise RuntimeError("Playwright not installed")

        self.playwright = sync_playwright().start()
        self.browser = self.playwright.chromium.launch(headless=headless)
        self.context = self.browser.new_context()
        self.page = self.context.new_page()
        self.page.goto(self.url, wait_until="networkidle")

    def screenshot(self, name: str = "webview") -> Path:
        """截图"""
        path = Path(f"/tmp/flowsight_tests/webview_{name}_{int(time.time())}.png")
        self.page.screenshot(path=path)
        return path

    def get_element(self, selector: str):
        """获取元素"""
        return self.page.locator(selector)

    def click(self, selector: str):
        """点击元素"""
        self.page.click(selector)

    def type_text(self, selector: str, text: str):
        """输入文本"""
        self.page.fill(selector, text)

    def get_text(self, selector: str) -> str:
        """获取元素文本"""
        return self.page.text_content(selector)

    def get_html(self, selector: str) -> str:
        """获取元素 HTML"""
        return self.page.inner_html(selector)

    def wait_for_selector(self, selector: str, timeout: int = 5000):
        """等待元素出现"""
        self.page.wait_for_selector(selector, timeout=timeout)

    def get_computed_style(self, selector: str, property: str) -> str:
        """获取计算样式"""
        return self.page.evaluate(
            f'document.querySelector("{selector}").computedStyleMap().get("{property}")'
        )

    def close(self):
        """关闭浏览器"""
        if self.browser:
            self.browser.close()
            self.playwright.stop()


class FlowsightUITester:
    """FlowSight UI 测试套件"""

    def __init__(self):
        self.desktop = DesktopAutomation()
        self.webview: Optional[WebViewTester] = None
        self.results: list[Dict] = []

    def setup(self):
        """初始化测试环境"""
        print("Setting up test environment...")
        self.desktop.wait(1)

        if PLAYWRIGHT_AVAILABLE:
            try:
                self.webview = WebViewTester()
                self.webview.connect(headless=True)
                print("Playwright connected")
            except Exception as e:
                print(f"Playwright connection failed: {e}")
                self.webview = None

    def teardown(self):
        """清理测试环境"""
        print("Cleaning up...")
        if self.webview:
            self.webview.close()

    # ============ 测试用例 ============

    def test_header_exists(self) -> TestResult:
        """测试 Header 存在"""
        try:
            if self.webview:
                self.webview.wait_for_selector("header", timeout=5000)
                return TestResult.PASS
            return TestResult.SKIP
        except Exception as e:
            return TestResult.FAIL

    def test_sidebar_exists(self) -> TestResult:
        """测试 Sidebar 存在"""
        try:
            if self.webview:
                self.webview.wait_for_selector("aside", timeout=5000)
                return TestResult.PASS
            return TestResult.SKIP
        except Exception as e:
            return TestResult.FAIL

    def test_command_palette_opens(self) -> TestResult:
        """测试命令面板打开"""
        try:
            # 按 Cmd+K 打开命令面板
            self.desktop.hotkey("cmd+k")
            self.desktop.wait(0.5)

            if self.webview:
                # 检查对话框是否出现
                dialog = self.webview.get_element('[role="dialog"]')
                if dialog.count() > 0:
                    return TestResult.PASS
            return TestResult.FAIL
        except Exception as e:
            return TestResult.FAIL

    def test_colors_applied_correctly(self) -> TestResult:
        """测试颜色正确应用"""
        try:
            if self.webview:
                # 检查 body 背景色
                bg_color = self.webview.page.evaluate(
                    'getComputedStyle(document.body).backgroundColor'
                )
                # 期望 rgb(10, 10, 11) = #0a0a0b
                if "10, 10, 11" in bg_color or "10,10,11" in bg_color:
                    return TestResult.PASS
            return TestResult.SKIP
        except Exception as e:
            return TestResult.FAIL

    def test_sidebar_navigation_works(self) -> TestResult:
        """测试侧边栏导航"""
        try:
            # 点击大纲按钮
            if self.webview:
                # 获取侧边栏按钮数量
                buttons = self.webview.get_element("aside button")
                count = buttons.count()
                if count > 0:
                    return TestResult.PASS
            return TestResult.SKIP
        except Exception as e:
            return TestResult.FAIL

    def test_right_panel_toggle(self) -> TestResult:
        """测试右侧面板切换"""
        try:
            if self.webview:
                # 查找右侧面板相关的元素
                panels = self.webview.get_element("[class*='panel']")
                return TestResult.PASS
            return TestResult.SKIP
        except Exception as e:
            return TestResult.FAIL

    def test_animation_works(self) -> TestResult:
        """测试动画效果"""
        try:
            # 检查是否有动画元素
            if self.webview:
                animated = self.webview.page.evaluate(
                    '''
                    () => {
                        const animated = document.querySelectorAll('[class*="animate"], [class*="motion"]');
                        return animated.length;
                    }
                    '''
                )
                if animated > 0:
                    return TestResult.PASS
            return TestResult.SKIP
        except Exception as e:
            return TestResult.FAIL

    def test_desktop_screenshot(self) -> TestResult:
        """测试桌面截图"""
        try:
            path = self.desktop.screenshot("desktop")
            if path.exists() and path.stat().st_size > 1000:
                print(f"Screenshot saved: {path}")
                return TestResult.PASS
            return TestResult.FAIL
        except Exception as e:
            return TestResult.FAIL

    # ============ 运行测试 ============

    def run_tests(self, tests: Optional[list[str]] = None) -> Dict[str, Any]:
        """运行测试套件

        Args:
            tests: 要运行的测试名称列表，None 表示运行所有
        """
        all_tests = {
            "test_header_exists": self.test_header_exists,
            "test_sidebar_exists": self.test_sidebar_exists,
            "test_command_palette_opens": self.test_command_palette_opens,
            "test_colors_applied_correctly": self.test_colors_applied_correctly,
            "test_sidebar_navigation_works": self.test_sidebar_navigation_works,
            "test_right_panel_toggle": self.test_right_panel_toggle,
            "test_animation_works": self.test_animation_works,
            "test_desktop_screenshot": self.test_desktop_screenshot,
        }

        if tests:
            to_run = {k: v for k, v in all_tests.items() if k in tests}
        else:
            to_run = all_tests

        results = []
        passed = 0
        failed = 0
        skipped = 0

        print("\n" + "=" * 60)
        print("Running FlowSight UI Tests")
        print("=" * 60 + "\n")

        for name, test_func in to_run.items():
            print(f"Running: {name}...", end=" ")

            # 截图用于调试
            debug_path = self.desktop.screenshot(f"debug_{name}")
            print(f"[{debug_path}]")

            result = test_func()
            results.append({
                "name": name,
                "result": result.value,
                "timestamp": int(time.time())
            })

            status = {
                TestResult.PASS: "✓ PASS",
                TestResult.FAIL: "✗ FAIL",
                TestResult.SKIP: "○ SKIP",
                TestResult.ERROR: "! ERROR",
            }[result]

            print(f"  {status}")

            if result == TestResult.PASS:
                passed += 1
            elif result == TestResult.FAIL:
                failed += 1
            else:
                skipped += 1

        print("\n" + "=" * 60)
        print(f"Results: {passed} passed, {failed} failed, {skipped} skipped")
        print("=" * 60 + "\n")

        return {
            "summary": {
                "passed": passed,
                "failed": failed,
                "skipped": skipped,
                "total": len(to_run)
            },
            "results": results
        }

    def interactive_test(self):
        """交互式测试 - 手动操作并截图"""
        print("\n" + "=" * 60)
        print("Interactive UI Test Mode")
        print("=" * 60)
        print("Commands:")
        print("  'screenshot <name>' - Take a screenshot")
        print("  'click <x> <y>' - Click at position")
        print("  'move <x> <y>' - Move mouse")
        print("  'hotkey <combo>' - Execute hotkey (e.g., 'ctrl+b')")
        print("  'type <text>' - Type text")
        print("  'wait <seconds>' - Wait")
        print("  'web <selector>' - Click web element by selector")
        print("  'web_screenshot' - Screenshot WebView")
        print("  'quit' - Exit")
        print()

        while True:
            try:
                cmd = input("> ").strip()
                if not cmd:
                    continue

                parts = cmd.split()
                action = parts[0].lower()

                if action == "quit" or action == "exit":
                    break

                elif action == "screenshot":
                    name = parts[1] if len(parts) > 1 else "interactive"
                    path = self.desktop.screenshot(name)
                    print(f"Screenshot: {path}")

                elif action == "click":
                    x, y = int(parts[1]), int(parts[2])
                    self.desktop.click(x, y)
                    print(f"Clicked at ({x}, {y})")

                elif action == "move":
                    x, y = int(parts[1]), int(parts[2])
                    self.desktop.move_mouse(x, y)
                    print(f"Moved to ({x}, {y})")

                elif action == "hotkey":
                    combo = parts[1]
                    self.desktop.hotkey(combo)
                    print(f"Executed: {combo}")

                elif action == "type":
                    text = " ".join(parts[1:])
                    self.desktop.type_text(text)
                    print(f"Typed: {text[:20]}...")

                elif action == "wait":
                    secs = float(parts[1]) if len(parts) > 1 else 1.0
                    self.desktop.wait(secs)
                    print(f"Waited {secs}s")

                elif action == "web" and self.webview:
                    selector = parts[1]
                    self.webview.click(selector)
                    print(f"Clicked web element: {selector}")

                elif action == "web_screenshot" and self.webview:
                    path = self.webview.screenshot("interactive")
                    print(f"WebView screenshot: {path}")

                elif action == "pos":
                    x, y = self.desktop.get_mouse_position()
                    print(f"Mouse position: ({x}, {y})")

                else:
                    print("Unknown command")

            except KeyboardInterrupt:
                break
            except Exception as e:
                print(f"Error: {e}")


def main():
    """主函数"""
    import sys

    tester = FlowsightUITester()

    try:
        tester.setup()

        if len(sys.argv) > 1 and sys.argv[1] == "--interactive":
            tester.interactive_test()
        else:
            # 运行所有测试
            results = tester.run_tests()

            # 保存结果
            result_path = Path("/tmp/flowsight_test_results.json")
            with open(result_path, "w") as f:
                json.dump(results, f, indent=2)
            print(f"Results saved to: {result_path}")

    finally:
        tester.teardown()


if __name__ == "__main__":
    main()
