"""
跨平台桌面自动化适配层
统一封装 macOS/Windows/Linux 的原生自动化接口
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional, List, Dict, Any, Tuple, Callable
from PIL import Image
import platform
import logging
import time
import subprocess
import os

logger = logging.getLogger(__name__)


@dataclass
class Point:
    """屏幕坐标点"""
    x: int
    y: int


@dataclass
class Rect:
    """屏幕矩形区域"""
    x: int
    y: int
    width: int
    height: int

    @property
    def center(self) -> Point:
        return Point(self.x + self.width // 2, self.y + self.height // 2)

    def contains(self, point: Point) -> bool:
        return (self.x <= point.x <= self.x + self.width and
                self.y <= point.y <= self.y + self.height)


@dataclass
class UIElement:
    """UI 元素信息"""
    element_id: Optional[str]
    element_type: str  # button, text, input, menu, window, etc.
    name: Optional[str]
    bounds: Rect
    value: Optional[str] = None
    enabled: bool = True
    focused: bool = False
    children: List['UIElement'] = None

    def __post_init__(self):
        if self.children is None:
            self.children = []


@dataclass
class ScreenshotResult:
    """截图结果"""
    image: Image.Image
    timestamp: float
    window_bounds: Optional[Rect] = None


@dataclass
class ApplicationInfo:
    """应用信息"""
    pid: int
    name: str
    bundle_id: Optional[str] = None  # macOS bundle ID
    window_handles: List[Any] = None

    def __post_init__(self):
        if self.window_handles is None:
            self.window_handles = []


class DesktopPlatformAdapter(ABC):
    """桌面平台适配器抽象基类"""

    @abstractmethod
    def get_screenshot(self, window_id: Optional[str] = None) -> ScreenshotResult:
        """获取屏幕或窗口截图"""
        pass

    @abstractmethod
    def find_application(self, app_name: str) -> Optional[ApplicationInfo]:
        """查找运行的应用"""
        pass

    @abstractmethod
    def activate_application(self, app_info: ApplicationInfo) -> bool:
        """激活应用窗口"""
        pass

    @abstractmethod
    def click(self, point: Point, button: str = "left") -> bool:
        """模拟鼠标点击"""
        pass

    @abstractmethod
    def double_click(self, point: Point) -> bool:
        """模拟鼠标双击"""
        pass

    @abstractmethod
    def right_click(self, point: Point) -> bool:
        """模拟鼠标右键"""
        pass

    @abstractmethod
    def move_mouse(self, point: Point) -> bool:
        """移动鼠标到指定位置"""
        pass

    @abstractmethod
    def drag(self, start: Point, end: Point, duration: float = 0.5) -> bool:
        """模拟拖拽操作"""
        pass

    @abstractmethod
    def scroll(self, point: Point, delta_x: int = 0, delta_y: int = 0) -> bool:
        """模拟滚动"""
        pass

    @abstractmethod
    def type_text(self, text: str, interval: float = 0.01) -> bool:
        """输入文本"""
        pass

    @abstractmethod
    def press_key(self, key: str, modifiers: List[str] = None) -> bool:
        """按下按键（可选修饰键）"""
        pass

    @abstractmethod
    def get_accessibility_tree(self, window_id: Optional[str] = None) -> Optional[UIElement]:
        """获取无障碍树（语义结构）"""
        pass

    @abstractmethod
    def find_element_by_name(self, name: str, element_type: Optional[str] = None) -> Optional[UIElement]:
        """通过名称查找元素"""
        pass

    @abstractmethod
    def get_screen_size(self) -> Tuple[int, int]:
        """获取屏幕尺寸"""
        pass


class MacOSAdapter(DesktopPlatformAdapter):
    """macOS 平台适配器 - 基于 AppleScript 和 Cocoa"""

    def __init__(self):
        self._current_app: Optional[ApplicationInfo] = None
        self._AXUIElement_classes = None

    def _run_applescript(self, script: str) -> Tuple[bool, str]:
        """运行 AppleScript"""
        try:
            result = subprocess.run(
                ['osascript', '-e', script],
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.returncode == 0, result.stdout.strip()
        except Exception as e:
            logger.error(f"AppleScript execution failed: {e}")
            return False, str(e)

    def get_screenshot(self, window_id: Optional[str] = None) -> ScreenshotResult:
        """使用 screencapture 获取截图"""
        import tempfile

        with tempfile.NamedTemporaryFile(suffix='.png', delete=False) as f:
            temp_path = f.name

        try:
            if window_id:
                # 截取特定窗口
                cmd = ['screencapture', '-w', '-x', temp_path]
            else:
                # 截取整个屏幕
                cmd = ['screencapture', '-x', temp_path]

            subprocess.run(cmd, check=True, capture_output=True)
            image = Image.open(temp_path)

            return ScreenshotResult(
                image=image,
                timestamp=time.time()
            )
        except Exception as e:
            logger.error(f"Screenshot failed: {e}")
            # 返回空图像作为回退
            return ScreenshotResult(
                image=Image.new('RGB', (1920, 1080), color='black'),
                timestamp=time.time()
            )
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def find_application(self, app_name: str) -> Optional[ApplicationInfo]:
        """查找运行的应用"""
        script = f'''
        tell application "System Events"
            try
                set appProc to first application process whose name contains "{app_name}"
                return (id of appProc as string) & "," & (name of appProc)
            on error
                return ""
            end try
        end tell
        '''
        success, result = self._run_applescript(script)
        if success and result:
            parts = result.split(',')
            if len(parts) >= 2:
                return ApplicationInfo(
                    pid=int(parts[0]),
                    name=parts[1]
                )
        return None

    def activate_application(self, app_info: ApplicationInfo) -> bool:
        """激活应用"""
        script = f'''
        tell application "{app_info.name}"
            activate
        end tell
        '''
        success, _ = self._run_applescript(script)
        if success:
            self._current_app = app_info
            time.sleep(0.5)  # 等待激活动画
        return success

    def click(self, point: Point, button: str = "left") -> bool:
        """使用 cliclick 或直接事件模拟点击"""
        try:
            # 首先尝试使用 cliclick (需要安装: brew install cliclick)
            button_map = {"left": "-", "right": "-r", "middle": "-m"}
            cmd = ['cliclick', button_map.get(button, "-"), f'{point.x},{point.y}']
            subprocess.run(cmd, check=True, capture_output=True)
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            # 回退到 AppleScript
            script = f'''
            tell application "System Events"
                click at {{{point.x}, {point.y}}}
            end tell
            '''
            success, _ = self._run_applescript(script)
            return success

    def double_click(self, point: Point) -> bool:
        """双击"""
        try:
            cmd = ['cliclick', '-w', '50', 'dc', f'{point.x},{point.y}']
            subprocess.run(cmd, check=True, capture_output=True)
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            # 手动双击
            self.click(point)
            time.sleep(0.1)
            self.click(point)
            return True

    def right_click(self, point: Point) -> bool:
        """右键点击"""
        return self.click(point, button="right")

    def move_mouse(self, point: Point) -> bool:
        """移动鼠标"""
        try:
            cmd = ['cliclick', 'm', f'{point.x},{point.y}']
            subprocess.run(cmd, check=True, capture_output=True)
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            return False

    def drag(self, start: Point, end: Point, duration: float = 0.5) -> bool:
        """拖拽"""
        try:
            cmd = ['cliclick', '-w', '50',
                   'dd', f'{start.x},{start.y}',
                   'dm', f'{end.x},{end.y}',
                   'du', f'{end.x},{end.y}']
            subprocess.run(cmd, check=True, capture_output=True)
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            # AppleScript 回退
            script = f'''
            tell application "System Events"
                key down option
                delay 0.1
                key up option
            end tell
            '''
            self._run_applescript(script)
            return False

    def scroll(self, point: Point, delta_x: int = 0, delta_y: int = 0) -> bool:
        """滚动"""
        try:
            direction = "-" if delta_y > 0 else "+"
            clicks = abs(delta_y) // 10
            cmd = ['cliclick', 'scroll', direction * clicks, f'{point.x},{point.y}']
            subprocess.run(cmd, check=True, capture_output=True)
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            return False

    def type_text(self, text: str, interval: float = 0.01) -> bool:
        """输入文本"""
        # 转义特殊字符
        escaped = text.replace('"', '\\"').replace('\\', '\\\\')
        script = f'''
        tell application "System Events"
            keystroke "{escaped}"
        end tell
        '''
        success, _ = self._run_applescript(script)
        return success

    def press_key(self, key: str, modifiers: List[str] = None) -> bool:
        """按键"""
        key_map = {
            'enter': 'return',
            'return': 'return',
            'tab': 'tab',
            'space': 'space',
            'escape': 'escape',
            'up': 'up',
            'down': 'down',
            'left': 'left',
            'right': 'right',
            'delete': 'delete',
            'backspace': 'delete',
        }

        actual_key = key_map.get(key.lower(), key)
        modifier_map = {
            'ctrl': 'control down',
            'alt': 'option down',
            'shift': 'shift down',
            'command': 'command down',
            'cmd': 'command down',
        }

        modifier_str = ' using {'.join(modifier_map.get(m.lower(), '') for m in (modifiers or []))
        if modifiers:
            modifier_str = f' using {{{", ".join(modifier_map.get(m.lower(), m) for m in modifiers)}}}'
        else:
            modifier_str = ''

        script = f'''
        tell application "System Events"
            key code {self._get_keycode(actual_key)}{modifier_str}
        end tell
        '''
        success, _ = self._run_applescript(script)
        return success

    def _get_keycode(self, key: str) -> str:
        """获取键码（简化版）"""
        keycodes = {
            'return': '36',
            'tab': '48',
            'space': '49',
            'delete': '51',
            'escape': '53',
            'command': '55',
            'shift': '56',
            'option': '58',
            'control': '59',
            'up': '126',
            'down': '125',
            'left': '123',
            'right': '124',
        }
        return keycodes.get(key.lower(), f'"{key}"')

    def get_accessibility_tree(self, window_id: Optional[str] = None) -> Optional[UIElement]:
        """获取无障碍树"""
        # 使用 pyobjc 和 Cocoa 框架获取
        try:
            from Cocoa import AXUIElementCreateApplication, AXUIElementCreateSystemWide
            from Cocoa import kAXTitleAttribute, kAXRoleAttribute, kAXPositionAttribute
            from Cocoa import kAXSizeAttribute, kAXChildrenAttribute, kAXEnabledAttribute
            from Cocoa import kAXFocusedAttribute, kAXValueAttribute

            # 获取系统范围的 AXUIElement
            system = AXUIElementCreateSystemWide()

            # 获取前台应用
            script = 'tell application "System Events" to return name of first application process whose frontmost is true'
            success, app_name = self._run_applescript(script)

            if not success or not app_name:
                return None

            # 查找应用并构建树
            # 这是一个简化版本，实际实现需要递归遍历
            return UIElement(
                element_id=None,
                element_type="application",
                name=app_name,
                bounds=Rect(0, 0, 1920, 1080)
            )
        except ImportError:
            logger.warning("pyobjc not installed, accessibility tree unavailable")
            return None

    def find_element_by_name(self, name: str, element_type: Optional[str] = None) -> Optional[UIElement]:
        """通过名称查找元素"""
        tree = self.get_accessibility_tree()
        if not tree:
            return None

        def search(element: UIElement) -> Optional[UIElement]:
            if element.name and name.lower() in element.name.lower():
                if element_type is None or element.element_type == element_type:
                    return element
            for child in element.children:
                result = search(child)
                if result:
                    return result
            return None

        return search(tree)

    def get_screen_size(self) -> Tuple[int, int]:
        """获取屏幕尺寸"""
        try:
            from Quartz import CGDisplayBounds, CGMainDisplayID
            main_display = CGMainDisplayID()
            bounds = CGDisplayBounds(main_display)
            return (int(bounds.size.width), int(bounds.size.height))
        except ImportError:
            # 回退方式
            return (1920, 1080)


class WindowsAdapter(DesktopPlatformAdapter):
    """Windows 平台适配器 - 基于 pywinauto"""

    def __init__(self):
        self._backend = None
        self._current_window = None

    def _get_backend(self):
        """延迟加载 pywinauto"""
        if self._backend is None:
            try:
                from pywinauto import Desktop
                self._backend = Desktop(backend="uia")
            except ImportError:
                logger.error("pywinauto not installed. Run: pip install pywinauto")
                raise
        return self._backend

    def get_screenshot(self, window_id: Optional[str] = None) -> ScreenshotResult:
        """获取截图"""
        try:
            if window_id and self._current_window:
                # 截取特定窗口
                img = self._current_window.capture_as_image()
            else:
                # 截取整个屏幕
                from PIL import ImageGrab
                img = ImageGrab.grab()

            return ScreenshotResult(
                image=img,
                timestamp=time.time()
            )
        except Exception as e:
            logger.error(f"Screenshot failed: {e}")
            return ScreenshotResult(
                image=Image.new('RGB', (1920, 1080), color='black'),
                timestamp=time.time()
            )

    def find_application(self, app_name: str) -> Optional[ApplicationInfo]:
        """查找应用"""
        try:
            backend = self._get_backend()
            windows = backend.windows(title_re=f".*{app_name}.*")

            for window in windows:
                try:
                    pid = window.process_id()
                    return ApplicationInfo(
                        pid=pid,
                        name=app_name,
                        window_handles=[window.handle]
                    )
                except Exception:
                    continue
        except Exception as e:
            logger.error(f"Find application failed: {e}")
        return None

    def activate_application(self, app_info: ApplicationInfo) -> bool:
        """激活应用"""
        try:
            from pywinauto import Application
            app = Application(backend="uia").connect(process=app_info.pid)
            self._current_window = app.window()
            self._current_window.set_focus()
            time.sleep(0.5)
            return True
        except Exception as e:
            logger.error(f"Activate application failed: {e}")
            return False

    def click(self, point: Point, button: str = "left") -> bool:
        """点击"""
        try:
            import pyautogui
            pyautogui.click(point.x, point.y, button=button)
            return True
        except Exception as e:
            logger.error(f"Click failed: {e}")
            return False

    def double_click(self, point: Point) -> bool:
        """双击"""
        try:
            import pyautogui
            pyautogui.doubleClick(point.x, point.y)
            return True
        except Exception as e:
            logger.error(f"Double click failed: {e}")
            return False

    def right_click(self, point: Point) -> bool:
        """右键"""
        return self.click(point, button="right")

    def move_mouse(self, point: Point) -> bool:
        """移动鼠标"""
        try:
            import pyautogui
            pyautogui.moveTo(point.x, point.y)
            return True
        except Exception as e:
            logger.error(f"Move mouse failed: {e}")
            return False

    def drag(self, start: Point, end: Point, duration: float = 0.5) -> bool:
        """拖拽"""
        try:
            import pyautogui
            pyautogui.moveTo(start.x, start.y)
            pyautogui.dragTo(end.x, end.y, duration=duration)
            return True
        except Exception as e:
            logger.error(f"Drag failed: {e}")
            return False

    def scroll(self, point: Point, delta_x: int = 0, delta_y: int = 0) -> bool:
        """滚动"""
        try:
            import pyautogui
            pyautogui.scroll(delta_y, point.x, point.y)
            return True
        except Exception as e:
            logger.error(f"Scroll failed: {e}")
            return False

    def type_text(self, text: str, interval: float = 0.01) -> bool:
        """输入文本"""
        try:
            import pyautogui
            pyautogui.typewrite(text, interval=interval)
            return True
        except Exception as e:
            logger.error(f"Type text failed: {e}")
            return False

    def press_key(self, key: str, modifiers: List[str] = None) -> bool:
        """按键"""
        try:
            import pyautogui

            key_map = {
                'ctrl': 'ctrl',
                'alt': 'alt',
                'shift': 'shift',
                'command': 'win',
                'cmd': 'win',
                'enter': 'enter',
                'return': 'enter',
                'tab': 'tab',
                'space': 'space',
                'escape': 'esc',
                'up': 'up',
                'down': 'down',
                'left': 'left',
                'right': 'right',
                'delete': 'delete',
                'backspace': 'backspace',
            }

            if modifiers:
                mod_keys = [key_map.get(m.lower(), m) for m in modifiers]
                actual_key = key_map.get(key.lower(), key)
                pyautogui.hotkey(*mod_keys, actual_key)
            else:
                actual_key = key_map.get(key.lower(), key)
                pyautogui.press(actual_key)

            return True
        except Exception as e:
            logger.error(f"Press key failed: {e}")
            return False

    def get_accessibility_tree(self, window_id: Optional[str] = None) -> Optional[UIElement]:
        """获取无障碍树"""
        try:
            if not self._current_window:
                return None

            window = self._current_window

            def build_tree(wrapper) -> UIElement:
                rect = wrapper.rectangle()
                element = UIElement(
                    element_id=str(wrapper.handle) if hasattr(wrapper, 'handle') else None,
                    element_type=wrapper.element_info.control_type or "unknown",
                    name=wrapper.element_info.name,
                    bounds=Rect(rect.left, rect.top, rect.width(), rect.height()),
                    enabled=wrapper.is_enabled() if hasattr(wrapper, 'is_enabled') else True,
                    focused=wrapper.has_focus() if hasattr(wrapper, 'has_focus') else False
                )

                # 递归获取子元素
                try:
                    children = wrapper.children()
                    element.children = [build_tree(child) for child in children]
                except Exception:
                    pass

                return element

            return build_tree(window.wrapper_object())
        except Exception as e:
            logger.error(f"Get accessibility tree failed: {e}")
            return None

    def find_element_by_name(self, name: str, element_type: Optional[str] = None) -> Optional[UIElement]:
        """通过名称查找元素"""
        tree = self.get_accessibility_tree()
        if not tree:
            return None

        def search(element: UIElement) -> Optional[UIElement]:
            if element.name and name.lower() in element.name.lower():
                if element_type is None or element.element_type == element_type:
                    return element
            for child in element.children:
                result = search(child)
                if result:
                    return result
            return None

        return search(tree)

    def get_screen_size(self) -> Tuple[int, int]:
        """获取屏幕尺寸"""
        try:
            import pyautogui
            return pyautogui.size()
        except Exception:
            return (1920, 1080)


class LinuxAdapter(DesktopPlatformAdapter):
    """Linux 平台适配器 - 基于 xdotool 和 AT-SPI"""

    def __init__(self):
        self._display = None
        self._current_window_id = None

    def _run_xdotool(self, args: List[str]) -> Tuple[bool, str]:
        """运行 xdotool 命令"""
        try:
            result = subprocess.run(
                ['xdotool'] + args,
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.returncode == 0, result.stdout.strip()
        except Exception as e:
            logger.error(f"xdotool execution failed: {e}")
            return False, str(e)

    def get_screenshot(self, window_id: Optional[str] = None) -> ScreenshotResult:
        """获取截图"""
        import tempfile

        with tempfile.NamedTemporaryFile(suffix='.png', delete=False) as f:
            temp_path = f.name

        try:
            if window_id:
                # 使用 gnome-screenshot 或 import (ImageMagick)
                cmd = ['gnome-screenshot', '-w', '-f', temp_path]
            else:
                cmd = ['gnome-screenshot', '-f', temp_path]

            subprocess.run(cmd, check=True, capture_output=True)
            image = Image.open(temp_path)

            return ScreenshotResult(
                image=image,
                timestamp=time.time()
            )
        except Exception as e:
            logger.error(f"Screenshot failed: {e}")
            # 尝试使用 ImageMagick
            try:
                cmd = ['import', '-window', 'root', temp_path]
                subprocess.run(cmd, check=True, capture_output=True)
                image = Image.open(temp_path)
                return ScreenshotResult(
                    image=image,
                    timestamp=time.time()
                )
            except Exception:
                pass

            return ScreenshotResult(
                image=Image.new('RGB', (1920, 1080), color='black'),
                timestamp=time.time()
            )
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def find_application(self, app_name: str) -> Optional[ApplicationInfo]:
        """查找应用"""
        try:
            # 搜索窗口
            success, window_ids = self._run_xdotool(
                ['search', '--name', f'.*{app_name}.*']
            )

            if success and window_ids:
                window_id = window_ids.split('\n')[0]
                # 获取 PID
                success, pid_str = self._run_xdotool(['getwindowpid', window_id])
                if success and pid_str:
                    return ApplicationInfo(
                        pid=int(pid_str),
                        name=app_name,
                        window_handles=[window_id]
                    )
        except Exception as e:
            logger.error(f"Find application failed: {e}")
        return None

    def activate_application(self, app_info: ApplicationInfo) -> bool:
        """激活应用"""
        if not app_info.window_handles:
            return False

        window_id = app_info.window_handles[0]
        success, _ = self._run_xdotool(['windowactivate', window_id])
        if success:
            self._current_window_id = window_id
            time.sleep(0.5)
        return success

    def click(self, point: Point, button: str = "left") -> bool:
        """点击"""
        button_map = {"left": "1", "right": "3", "middle": "2"}
        success, _ = self._run_xdotool([
            'mousemove', str(point.x), str(point.y),
            'click', button_map.get(button, "1")
        ])
        return success

    def double_click(self, point: Point) -> bool:
        """双击"""
        success, _ = self._run_xdotool([
            'mousemove', str(point.x), str(point.y),
            'click', '--repeat', '2', '--delay', '50', '1'
        ])
        return success

    def right_click(self, point: Point) -> bool:
        """右键"""
        return self.click(point, button="right")

    def move_mouse(self, point: Point) -> bool:
        """移动鼠标"""
        success, _ = self._run_xdotool([
            'mousemove', str(point.x), str(point.y)
        ])
        return success

    def drag(self, start: Point, end: Point, duration: float = 0.5) -> bool:
        """拖拽"""
        # xdotool 的拖拽
        steps = int(duration * 20)  # 20 steps per second
        success, _ = self._run_xdotool([
            'mousemove', str(start.x), str(start.y),
            'mousedown', '1'
        ])

        if not success:
            return False

        # 渐变移动
        for i in range(1, steps + 1):
            t = i / steps
            x = int(start.x + (end.x - start.x) * t)
            y = int(start.y + (end.y - start.y) * t)
            self._run_xdotool(['mousemove', str(x), str(y)])
            time.sleep(duration / steps)

        success, _ = self._run_xdotool([
            'mouseup', '1'
        ])
        return success

    def scroll(self, point: Point, delta_x: int = 0, delta_y: int = 0) -> bool:
        """滚动"""
        # xdotool 点击滚动
        button = '4' if delta_y < 0 else '5'  # 4=up, 5=down
        clicks = abs(delta_y) // 10

        success, _ = self._run_xdotool([
            'mousemove', str(point.x), str(point.y)
        ])

        if not success:
            return False

        for _ in range(clicks):
            success, _ = self._run_xdotool(['click', button])
            if not success:
                return False
            time.sleep(0.01)

        return True

    def type_text(self, text: str, interval: float = 0.01) -> bool:
        """输入文本"""
        success, _ = self._run_xdotool(['type', text])
        return success

    def press_key(self, key: str, modifiers: List[str] = None) -> bool:
        """按键"""
        key_map = {
            'ctrl': 'ctrl',
            'alt': 'alt',
            'shift': 'shift',
            'command': 'super',
            'cmd': 'super',
            'enter': 'Return',
            'return': 'Return',
            'tab': 'Tab',
            'space': 'space',
            'escape': 'Escape',
            'up': 'Up',
            'down': 'Down',
            'left': 'Left',
            'right': 'Right',
            'delete': 'Delete',
            'backspace': 'BackSpace',
        }

        actual_key = key_map.get(key.lower(), key)

        if modifiers:
            mod_str = '+'.join(key_map.get(m.lower(), m) for m in modifiers)
            key_combo = f"{mod_str}+{actual_key}"
        else:
            key_combo = actual_key

        success, _ = self._run_xdotool(['key', key_combo])
        return success

    def get_accessibility_tree(self, window_id: Optional[str] = None) -> Optional[UIElement]:
        """获取无障碍树 - 使用 AT-SPI"""
        try:
            import pyatspi

            # 获取桌面
            desktop = pyatspi.Registry.getDesktop(0)

            # 查找应用
            for app in desktop:
                if app is None:
                    continue

                def build_tree(accessible) -> UIElement:
                    # 获取位置
                    try:
                        pos = accessible.queryComponent().getPosition(pyatspi.DESKTOP_COORDS)
                        size = accessible.queryComponent().getSize()
                        bounds = Rect(pos[0], pos[1], size[0], size[1])
                    except Exception:
                        bounds = Rect(0, 0, 0, 0)

                    # 获取名称和角色
                    name = accessible.name or ""
                    role = accessible.getRoleName() or "unknown"

                    element = UIElement(
                        element_id=str(accessible.accessibleId) if hasattr(accessible, 'accessibleId') else None,
                        element_type=role,
                        name=name,
                        bounds=bounds
                    )

                    # 递归子元素
                    for i in range(accessible.childCount):
                        try:
                            child = accessible.getChildAtIndex(i)
                            if child:
                                element.children.append(build_tree(child))
                        except Exception:
                            pass

                    return element

                return build_tree(app)

        except ImportError:
            logger.warning("pyatspi not installed, accessibility tree unavailable")
            return None
        except Exception as e:
            logger.error(f"Get accessibility tree failed: {e}")
            return None

    def find_element_by_name(self, name: str, element_type: Optional[str] = None) -> Optional[UIElement]:
        """通过名称查找元素"""
        tree = self.get_accessibility_tree()
        if not tree:
            return None

        def search(element: UIElement) -> Optional[UIElement]:
            if element.name and name.lower() in element.name.lower():
                if element_type is None or element.element_type == element_type:
                    return element
            for child in element.children:
                result = search(child)
                if result:
                    return result
            return None

        return search(tree)

    def get_screen_size(self) -> Tuple[int, int]:
        """获取屏幕尺寸"""
        try:
            success, output = self._run_xdotool(['getdisplaygeometry'])
            if success:
                parts = output.split()
                return (int(parts[0]), int(parts[1]))
        except Exception:
            pass
        return (1920, 1080)


class DesktopAdapter:
    """桌面适配器工厂和主控类"""

    def __init__(self):
        self._platform = platform.system()
        self._adapter: Optional[DesktopPlatformAdapter] = None
        self._initialize_adapter()

    def _initialize_adapter(self):
        """根据平台初始化适配器"""
        if self._platform == "Darwin":
            self._adapter = MacOSAdapter()
            logger.info("Initialized macOS adapter")
        elif self._platform == "Windows":
            self._adapter = WindowsAdapter()
            logger.info("Initialized Windows adapter")
        elif self._platform == "Linux":
            self._adapter = LinuxAdapter()
            logger.info("Initialized Linux adapter")
        else:
            raise RuntimeError(f"Unsupported platform: {self._platform}")

    @property
    def platform(self) -> str:
        return self._platform

    @property
    def adapter(self) -> DesktopPlatformAdapter:
        return self._adapter

    # 便捷方法委托给适配器
    def get_screenshot(self, window_id: Optional[str] = None) -> ScreenshotResult:
        return self._adapter.get_screenshot(window_id)

    def find_application(self, app_name: str) -> Optional[ApplicationInfo]:
        return self._adapter.find_application(app_name)

    def activate_application(self, app_info: ApplicationInfo) -> bool:
        return self._adapter.activate_application(app_info)

    def find_and_activate(self, app_name: str) -> Optional[ApplicationInfo]:
        """查找并激活应用"""
        app_info = self.find_application(app_name)
        if app_info:
            self.activate_application(app_info)
        return app_info

    def click(self, point: Point, button: str = "left") -> bool:
        return self._adapter.click(point, button)

    def click_element(self, element: UIElement, button: str = "left") -> bool:
        """点击元素中心"""
        return self.click(element.bounds.center, button)

    def double_click(self, point: Point) -> bool:
        return self._adapter.double_click(point)

    def right_click(self, point: Point) -> bool:
        return self._adapter.right_click(point)

    def move_mouse(self, point: Point) -> bool:
        return self._adapter.move_mouse(point)

    def drag(self, start: Point, end: Point, duration: float = 0.5) -> bool:
        return self._adapter.drag(start, end, duration)

    def scroll(self, point: Point, delta_x: int = 0, delta_y: int = 0) -> bool:
        return self._adapter.scroll(point, delta_x, delta_y)

    def type_text(self, text: str, interval: float = 0.01) -> bool:
        return self._adapter.type_text(text, interval)

    def press_key(self, key: str, modifiers: List[str] = None) -> bool:
        return self._adapter.press_key(key, modifiers)

    def shortcut(self, *keys: str) -> bool:
        """执行快捷键组合"""
        if not keys:
            return False
        return self.press_key(keys[-1], list(keys[:-1]))

    def get_accessibility_tree(self, window_id: Optional[str] = None) -> Optional[UIElement]:
        return self._adapter.get_accessibility_tree(window_id)

    def find_element_by_name(self, name: str, element_type: Optional[str] = None) -> Optional[UIElement]:
        return self._adapter.find_element_by_name(name, element_type)

    def get_screen_size(self) -> Tuple[int, int]:
        return self._adapter.get_screen_size()

    def wait_for_element(self, name: str, timeout: float = 10.0,
                         poll_interval: float = 0.5) -> Optional[UIElement]:
        """等待元素出现"""
        start_time = time.time()
        while time.time() - start_time < timeout:
            element = self.find_element_by_name(name)
            if element:
                return element
            time.sleep(poll_interval)
        return None

    def take_screenshot_to_file(self, filepath: str, window_id: Optional[str] = None) -> bool:
        """截图保存到文件"""
        try:
            result = self.get_screenshot(window_id)
            result.image.save(filepath)
            return True
        except Exception as e:
            logger.error(f"Save screenshot failed: {e}")
            return False


# 全局单例
def get_desktop_adapter() -> DesktopAdapter:
    """获取桌面适配器实例"""
    return DesktopAdapter()
