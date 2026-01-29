"""
macOS 平台适配器实现
使用 AppleScript 和 CoreGraphics API
"""

import subprocess
from typing import Optional, List
from PIL import Image
import io
import logging

from ..core.platform_adapter import (
    PlatformAdapter, Point, Rectangle, AccessibilityNode,
    WindowInfo, Platform
)

logger = logging.getLogger(__name__)


class MacOSAdapter(PlatformAdapter):
    """macOS 平台自动化适配器"""

    def __init__(self):
        super().__init__()
        self.platform = Platform.MACOS
        self._check_permissions()

    def _check_permissions(self):
        """检查必要的辅助功能权限"""
        # macOS 需要辅助功能权限才能控制鼠标和键盘
        script = '''
        tell application "System Events"
            return UI elements enabled
        end tell
        '''
        try:
            result = subprocess.run(
                ['osascript', '-e', script],
                capture_output=True,
                text=True,
                timeout=5
            )
            if 'true' not in result.stdout.lower():
                logger.warning("辅助功能权限未启用，请在 系统设置 > 隐私与安全性 > 辅助功能 中授权")
        except Exception as e:
            logger.warning(f"无法检查辅助功能权限: {e}")

    # ==================== 截图相关 ====================

    def screenshot(self) -> Image.Image:
        """截取整个屏幕"""
        # 使用 screencapture 命令
        cmd = ['screencapture', '-x', '-c']
        result = subprocess.run(cmd, capture_output=True)

        if result.returncode == 0:
            # 从剪贴板读取图片数据
            from PIL import ImageGrab
            image = ImageGrab.grab()
            self._notify_screenshot(image)
            return image
        else:
            raise RuntimeError(f"截图失败: {result.stderr.decode()}")

    def screenshot_region(self, region: Rectangle) -> Image.Image:
        """截取指定区域"""
        # screencapture 支持区域截图 -R x,y,w,h
        cmd = [
            'screencapture',
            '-x',
            '-R', f"{region.x},{region.y},{region.width},{region.height}",
            '-c'
        ]
        result = subprocess.run(cmd, capture_output=True)

        if result.returncode == 0:
            from PIL import ImageGrab
            image = ImageGrab.grab(bbox=(region.x, region.y, region.right, region.bottom))
            self._notify_screenshot(image)
            return image
        else:
            raise RuntimeError(f"区域截图失败: {result.stderr.decode()}")

    def screenshot_window(self, window_title: Optional[str] = None) -> Image.Image:
        """截取指定窗口"""
        if window_title:
            # 获取窗口 ID
            window_id = self._get_window_id_by_title(window_title)
            if window_id:
                cmd = ['screencapture', '-x', '-w', window_id, '-c']
            else:
                raise ValueError(f"未找到窗口: {window_title}")
        else:
            # 截取活动窗口
            cmd = ['screencapture', '-x', '-W', '-c']

        result = subprocess.run(cmd, capture_output=True)
        if result.returncode == 0:
            from PIL import ImageGrab
            image = ImageGrab.grab()
            self._notify_screenshot(image)
            return image
        else:
            raise RuntimeError(f"窗口截图失败: {result.stderr.decode()}")

    def _get_window_id_by_title(self, title: str) -> Optional[str]:
        """根据标题获取窗口 ID"""
        script = f'''
        tell application "System Events"
            set windowList to {{}}
            tell application process "FlowSight"
                repeat with w in windows
                    set end of windowList to (id of w as string) & ":" & (name of w)
                end repeat
            end tell
            return windowList as string
        end tell
        '''
        # 简化实现，实际应该解析窗口列表
        return None

    # ==================== 鼠标控制 ====================

    def mouse_move(self, point: Point):
        """移动鼠标到指定位置"""
        # 使用 AppleScript 或 pyobjc
        script = f'''
        tell application "System Events"
            set mouseLoc to {{{point.x}, {point.y}}}
        end tell
        '''
        # 使用更好的方式：CoreGraphics
        try:
            import Quartz
            Quartz.CGEventPost(
                Quartz.kCGHIDEventTap,
                Quartz.CGEventCreateMouseEvent(
                    None,
                    Quartz.kCGEventMouseMoved,
                    (point.x, point.y),
                    Quartz.kCGMouseButtonLeft
                )
            )
            self._notify_action('mouse_move', {'x': point.x, 'y': point.y})
        except ImportError:
            logger.error("需要安装 pyobjc-framework-Quartz: pip install pyobjc-framework-Quartz")
            raise

    def mouse_click(self, point: Point, button: str = 'left'):
        """在指定位置点击鼠标"""
        try:
            import Quartz

            button_map = {
                'left': Quartz.kCGMouseButtonLeft,
                'right': Quartz.kCGMouseButtonRight,
                'middle': Quartz.kCGMouseButtonCenter
            }
            cg_button = button_map.get(button, Quartz.kCGMouseButtonLeft)

            # 移动鼠标
            self.mouse_move(point)

            # 按下
            down_event = Quartz.CGEventCreateMouseEvent(
                None,
                Quartz.kCGEventLeftMouseDown if button == 'left' else Quartz.kCGEventRightMouseDown,
                (point.x, point.y),
                cg_button
            )
            Quartz.CGEventPost(Quartz.kCGHIDEventTap, down_event)

            # 释放
            up_event = Quartz.CGEventCreateMouseEvent(
                None,
                Quartz.kCGEventLeftMouseUp if button == 'left' else Quartz.kCGEventRightMouseUp,
                (point.x, point.y),
                cg_button
            )
            Quartz.CGEventPost(Quartz.kCGHIDEventTap, up_event)

            self._notify_action('mouse_click', {'x': point.x, 'y': point.y, 'button': button})
        except ImportError:
            logger.error("需要安装 pyobjc-framework-Quartz")
            raise

    def mouse_double_click(self, point: Point, button: str = 'left'):
        """在指定位置双击鼠标"""
        self.mouse_click(point, button)
        self.wait(0.1)
        self.mouse_click(point, button)
        self._notify_action('mouse_double_click', {'x': point.x, 'y': point.y, 'button': button})

    def mouse_drag(self, start: Point, end: Point, button: str = 'left'):
        """拖拽鼠标"""
        try:
            import Quartz

            button_map = {
                'left': Quartz.kCGMouseButtonLeft,
                'right': Quartz.kCGMouseButtonRight,
                'middle': Quartz.kCGMouseButtonCenter
            }
            cg_button = button_map.get(button, Quartz.kCGMouseButtonLeft)

            # 移动到起始位置
            self.mouse_move(start)

            # 按下
            down_event = Quartz.CGEventCreateMouseEvent(
                None,
                Quartz.kCGEventLeftMouseDown,
                (start.x, start.y),
                cg_button
            )
            Quartz.CGEventPost(Quartz.kCGHIDEventTap, down_event)

            # 移动到结束位置（拖动）
            drag_event = Quartz.CGEventCreateMouseEvent(
                None,
                Quartz.kCGEventLeftMouseDragged,
                (end.x, end.y),
                cg_button
            )
            Quartz.CGEventPost(Quartz.kCGHIDEventTap, drag_event)

            # 释放
            up_event = Quartz.CGEventCreateMouseEvent(
                None,
                Quartz.kCGEventLeftMouseUp,
                (end.x, end.y),
                cg_button
            )
            Quartz.CGEventPost(Quartz.kCGHIDEventTap, up_event)

            self._notify_action('mouse_drag', {
                'start_x': start.x, 'start_y': start.y,
                'end_x': end.x, 'end_y': end.y,
                'button': button
            })
        except ImportError:
            logger.error("需要安装 pyobjc-framework-Quartz")
            raise

    def mouse_scroll(self, delta: int, direction: str = 'vertical'):
        """滚动鼠标滚轮"""
        try:
            import Quartz

            if direction == 'vertical':
                event = Quartz.CGEventCreateScrollWheelEvent(
                    None,
                    Quartz.kCGScrollEventUnitPixel,
                    1,
                    delta
                )
            else:
                event = Quartz.CGEventCreateScrollWheelEvent(
                    None,
                    Quartz.kCGScrollEventUnitPixel,
                    2,
                    0,
                    delta
                )

            Quartz.CGEventPost(Quartz.kCGHIDEventTap, event)
            self._notify_action('mouse_scroll', {'delta': delta, 'direction': direction})
        except ImportError:
            logger.error("需要安装 pyobjc-framework-Quartz")
            raise

    def get_mouse_position(self) -> Point:
        """获取当前鼠标位置"""
        try:
            import Quartz
            loc = Quartz.NSEvent.mouseLocation()
            # 注意：macOS 的坐标系原点在左下角，需要转换
            screen_height = Quartz.CGDisplayBounds(Quartz.CGMainDisplayID()).size.height
            return Point(int(loc.x), int(screen_height - loc.y))
        except ImportError:
            logger.error("需要安装 pyobjc-framework-Quartz")
            raise

    # ==================== 键盘控制 ====================

    def key_press(self, key: str):
        """按下并释放单个按键"""
        key_map = {
            'enter': 36, 'return': 36,
            'esc': 53, 'escape': 53,
            'tab': 48,
            'space': 49,
            'delete': 51, 'backspace': 51,
            'up': 126, 'down': 125,
            'left': 123, 'right': 124,
            'cmd': 55, 'command': 55,
            'shift': 56,
            'caps': 57, 'capslock': 57,
            'option': 58, 'alt': 58,
            'ctrl': 59, 'control': 59,
        }

        try:
            import Quartz

            key_code = key_map.get(key.lower())
            if key_code:
                # 特殊按键
                event = Quartz.CGEventCreateKeyboardEvent(None, key_code, True)
                Quartz.CGEventPost(Quartz.kCGHIDEventTap, event)
                event = Quartz.CGEventCreateKeyboardEvent(None, key_code, False)
                Quartz.CGEventPost(Quartz.kCGHIDEventTap, event)
            else:
                # 普通字符，使用 AppleScript
                script = f'''
                tell application "System Events"
                    keystroke "{key}"
                end tell
                '''
                subprocess.run(['osascript', '-e', script], capture_output=True)

            self._notify_action('key_press', {'key': key})
        except ImportError:
            logger.error("需要安装 pyobjc-framework-Quartz")
            raise

    def key_down(self, key: str):
        """按住按键不放"""
        key_map = {
            'cmd': 55, 'command': 55,
            'shift': 56,
            'option': 58, 'alt': 58,
            'ctrl': 59, 'control': 59,
        }

        try:
            import Quartz
            key_code = key_map.get(key.lower())
            if key_code:
                event = Quartz.CGEventCreateKeyboardEvent(None, key_code, True)
                Quartz.CGEventPost(Quartz.kCGHIDEventTap, event)
        except ImportError:
            logger.error("需要安装 pyobjc-framework-Quartz")
            raise

    def key_up(self, key: str):
        """释放按键"""
        key_map = {
            'cmd': 55, 'command': 55,
            'shift': 56,
            'option': 58, 'alt': 58,
            'ctrl': 59, 'control': 59,
        }

        try:
            import Quartz
            key_code = key_map.get(key.lower())
            if key_code:
                event = Quartz.CGEventCreateKeyboardEvent(None, key_code, False)
                Quartz.CGEventPost(Quartz.kCGHIDEventTap, event)
        except ImportError:
            logger.error("需要安装 pyobjc-framework-Quartz")
            raise

    def type_text(self, text: str, interval: float = 0.01):
        """输入文本"""
        # 使用 AppleScript 输入文本更安全
        script = f'''
        tell application "System Events"
            keystroke "{text}"
        end tell
        '''
        subprocess.run(['osascript', '-e', script], capture_output=True)
        self._notify_action('type_text', {'text': text})

    # ==================== 窗口管理 ====================

    def get_window_list(self) -> List[WindowInfo]:
        """获取所有窗口列表"""
        script = '''
        tell application "System Events"
            set windowList to {}
            repeat with proc in application processes
                repeat with w in windows of proc
                    set wTitle to name of w
                    set wPos to position of w
                    set wSize to size of w
                    set wActive to value of attribute "AXMain" of w
                    set end of windowList to (wTitle & "|" & (item 1 of wPos) & "," & (item 2 of wPos) & "," & (item 1 of wSize) & "," & (item 2 of wSize) & "," & wActive)
                end repeat
            end repeat
            return windowList as string
        end tell
        '''
        # 简化实现，实际需要解析返回的字符串
        return []

    def find_window(self, title_pattern: str) -> Optional[WindowInfo]:
        """根据标题查找窗口"""
        windows = self.get_window_list()
        for window in windows:
            if title_pattern.lower() in window.title.lower():
                return window
        return None

    def activate_window(self, window_id: str):
        """激活指定窗口"""
        script = f'''
        tell application "System Events"
            set frontmost of first application process whose name contains "{window_id}" to true
        end tell
        '''
        subprocess.run(['osascript', '-e', script], capture_output=True)
        self._notify_action('activate_window', {'window_id': window_id})

    def resize_window(self, window_id: str, width: int, height: int):
        """调整窗口大小"""
        script = f'''
        tell application "System Events"
            tell application process "FlowSight"
                set size of front window to {{{width}, {height}}}
            end tell
        end tell
        '''
        subprocess.run(['osascript', '-e', script], capture_output=True)
        self._notify_action('resize_window', {'window_id': window_id, 'width': width, 'height': height})

    def move_window(self, window_id: str, x: int, y: int):
        """移动窗口位置"""
        script = f'''
        tell application "System Events"
            tell application process "FlowSight"
                set position of front window to {{{x}, {y}}}
            end tell
        end tell
        '''
        subprocess.run(['osascript', '-e', script], capture_output=True)
        self._notify_action('move_window', {'window_id': window_id, 'x': x, 'y': y})

    # ==================== 无障碍树访问 ====================

    def get_accessibility_tree(self) -> Optional[AccessibilityNode]:
        """获取完整的无障碍树"""
        # macOS AX API 需要通过 pyobjc 访问
        logger.warning("无障碍树访问需要实现 AX API 绑定")
        return None

    def find_accessibility_node(
        self,
        role: Optional[str] = None,
        label: Optional[str] = None,
        value: Optional[str] = None
    ) -> Optional[AccessibilityNode]:
        """在无障碍树中查找节点"""
        tree = self.get_accessibility_tree()
        if not tree:
            return None
        return self._search_node(tree, role, label, value)

    def _search_node(
        self,
        node: AccessibilityNode,
        role: Optional[str],
        label: Optional[str],
        value: Optional[str]
    ) -> Optional[AccessibilityNode]:
        """递归搜索节点"""
        if role and node.role == role:
            return node
        if label and node.label == label:
            return node
        if value and node.value == value:
            return node

        for child in node.children:
            found = self._search_node(child, role, label, value)
            if found:
                return found
        return None

    def cleanup(self):
        """清理资源"""
        pass
