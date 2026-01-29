"""
Windows 平台适配器实现
使用 pywinauto + ctypes
"""

from typing import Optional, List
from PIL import Image
import logging

from ..core.platform_adapter import (
    PlatformAdapter, Point, Rectangle, AccessibilityNode,
    WindowInfo, Platform
)

logger = logging.getLogger(__name__)


class WindowsAdapter(PlatformAdapter):
    """Windows 平台自动化适配器"""

    def __init__(self):
        super().__init__()
        self.platform = Platform.WINDOWS
        self._hwnd_cache = {}

    def _init_win32(self):
        """初始化 Windows API"""
        try:
            import win32gui
            import win32con
            import win32api
            import win32ui
            return win32gui, win32con, win32api, win32ui
        except ImportError:
            logger.error("需要安装 pywin32: pip install pywin32")
            raise

    # ==================== 截图相关 ====================

    def screenshot(self) -> Image.Image:
        """截取整个屏幕"""
        try:
            import pyautogui
            image = pyautogui.screenshot()
            self._notify_screenshot(image)
            return image
        except ImportError:
            # 回退到 Windows API
            return self._screenshot_winapi()

    def _screenshot_winapi(self) -> Image.Image:
        """使用 Windows API 截图"""
        win32gui, win32con, win32api, win32ui = self._init_win32()

        # 获取屏幕 DC
        hwnd = win32gui.GetDesktopWindow()
        width = win32api.GetSystemMetrics(win32con.SM_CXVIRTUALSCREEN)
        height = win32api.GetSystemMetrics(win32con.SM_CYVIRTUALSCREEN)
        left = win32api.GetSystemMetrics(win32con.SM_XVIRTUALSCREEN)
        top = win32api.GetSystemMetrics(win32con.SM_YVIRTUALSCREEN)

        hwndDC = win32gui.GetWindowDC(hwnd)
        mfcDC = win32ui.CreateDCFromHandle(hwndDC)
        saveDC = mfcDC.CreateCompatibleDC()

        saveBitMap = win32ui.CreateBitmap()
        saveBitMap.CreateCompatibleBitmap(mfcDC, width, height)
        saveDC.SelectObject(saveBitMap)
        saveDC.BitBlt((0, 0), (width, height), mfcDC, (left, top), win32con.SRCCOPY)

        bmpinfo = saveBitMap.GetInfo()
        bmpstr = saveBitMap.GetBitmapBits(True)
        image = Image.frombuffer(
            'RGB',
            (bmpinfo['bmWidth'], bmpinfo['bmHeight']),
            bmpstr, 'raw', 'BGRX', 0, 1
        )

        win32gui.DeleteObject(saveBitMap.GetHandle())
        saveDC.DeleteDC()
        mfcDC.DeleteDC()
        win32gui.ReleaseDC(hwnd, hwndDC)

        self._notify_screenshot(image)
        return image

    def screenshot_region(self, region: Rectangle) -> Image.Image:
        """截取指定区域"""
        full = self.screenshot()
        cropped = full.crop((region.x, region.y, region.right, region.bottom))
        return cropped

    def screenshot_window(self, window_title: Optional[str] = None) -> Image.Image:
        """截取指定窗口"""
        win32gui, win32con, win32api, win32ui = self._init_win32()

        if window_title:
            hwnd = win32gui.FindWindow(None, window_title)
            if not hwnd:
                raise ValueError(f"未找到窗口: {window_title}")
        else:
            hwnd = win32gui.GetForegroundWindow()

        # 获取窗口尺寸
        left, top, right, bottom = win32gui.GetWindowRect(hwnd)
        width = right - left
        height = bottom - top

        hwndDC = win32gui.GetWindowDC(hwnd)
        mfcDC = win32ui.CreateDCFromHandle(hwndDC)
        saveDC = mfcDC.CreateCompatibleDC()

        saveBitMap = win32ui.CreateBitmap()
        saveBitMap.CreateCompatibleBitmap(mfcDC, width, height)
        saveDC.SelectObject(saveBitMap)
        saveDC.BitBlt((0, 0), (width, height), mfcDC, (0, 0), win32con.SRCCOPY)

        bmpinfo = saveBitMap.GetInfo()
        bmpstr = saveBitMap.GetBitmapBits(True)
        image = Image.frombuffer(
            'RGB',
            (bmpinfo['bmWidth'], bmpinfo['bmHeight']),
            bmpstr, 'raw', 'BGRX', 0, 1
        )

        win32gui.DeleteObject(saveBitMap.GetHandle())
        saveDC.DeleteDC()
        mfcDC.DeleteDC()
        win32gui.ReleaseDC(hwnd, hwndDC)

        self._notify_screenshot(image)
        return image

    # ==================== 鼠标控制 ====================

    def mouse_move(self, point: Point):
        """移动鼠标到指定位置"""
        try:
            import pyautogui
            pyautogui.moveTo(point.x, point.y)
            self._notify_action('mouse_move', {'x': point.x, 'y': point.y})
        except ImportError:
            # 使用 ctypes
            import ctypes
            ctypes.windll.user32.SetCursorPos(point.x, point.y)

    def mouse_click(self, point: Point, button: str = 'left'):
        """在指定位置点击鼠标"""
        try:
            import pyautogui
            self.mouse_move(point)
            pyautogui.click(button=button)
            self._notify_action('mouse_click', {'x': point.x, 'y': point.y, 'button': button})
        except ImportError:
            win32gui, win32con, win32api, win32ui = self._init_win32()
            # 使用 mouse_event
            if button == 'left':
                win32api.mouse_event(win32con.MOUSEEVENTF_LEFTDOWN, point.x, point.y, 0, 0)
                win32api.mouse_event(win32con.MOUSEEVENTF_LEFTUP, point.x, point.y, 0, 0)
            elif button == 'right':
                win32api.mouse_event(win32con.MOUSEEVENTF_RIGHTDOWN, point.x, point.y, 0, 0)
                win32api.mouse_event(win32con.MOUSEEVENTF_RIGHTUP, point.x, point.y, 0, 0)

    def mouse_double_click(self, point: Point, button: str = 'left'):
        """在指定位置双击鼠标"""
        try:
            import pyautogui
            self.mouse_move(point)
            pyautogui.doubleClick(button=button)
            self._notify_action('mouse_double_click', {'x': point.x, 'y': point.y, 'button': button})
        except ImportError:
            self.mouse_click(point, button)
            self.wait(0.1)
            self.mouse_click(point, button)

    def mouse_drag(self, start: Point, end: Point, button: str = 'left'):
        """拖拽鼠标"""
        try:
            import pyautogui
            pyautogui.moveTo(start.x, start.y)
            pyautogui.dragTo(end.x, end.y, button=button)
            self._notify_action('mouse_drag', {
                'start_x': start.x, 'start_y': start.y,
                'end_x': end.x, 'end_y': end.y,
                'button': button
            })
        except ImportError:
            self.mouse_move(start)
            win32gui, win32con, win32api, win32ui = self._init_win32()
            if button == 'left':
                win32api.mouse_event(win32con.MOUSEEVENTF_LEFTDOWN, start.x, start.y, 0, 0)
                win32api.mouse_event(win32con.MOUSEEVENTF_MOVE, end.x - start.x, end.y - start.y, 0, 0)
                win32api.mouse_event(win32con.MOUSEEVENTF_LEFTUP, end.x, end.y, 0, 0)

    def mouse_scroll(self, delta: int, direction: str = 'vertical'):
        """滚动鼠标滚轮"""
        try:
            import pyautogui
            if direction == 'vertical':
                pyautogui.scroll(delta)
            else:
                pyautogui.hscroll(delta)
            self._notify_action('mouse_scroll', {'delta': delta, 'direction': direction})
        except ImportError:
            win32gui, win32con, win32api, win32ui = self._init_win32()
            if direction == 'vertical':
                win32api.mouse_event(win32con.MOUSEEVENTF_WHEEL, 0, 0, delta, 0)

    def get_mouse_position(self) -> Point:
        """获取当前鼠标位置"""
        try:
            import pyautogui
            x, y = pyautogui.position()
            return Point(x, y)
        except ImportError:
            import ctypes
            class POINT(ctypes.Structure):
                _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]
            pt = POINT()
            ctypes.windll.user32.GetCursorPos(ctypes.byref(pt))
            return Point(pt.x, pt.y)

    # ==================== 键盘控制 ====================

    def key_press(self, key: str):
        """按下并释放单个按键"""
        try:
            import pyautogui
            pyautogui.press(key)
            self._notify_action('key_press', {'key': key})
        except ImportError:
            # 使用 keybd_event
            pass

    def key_down(self, key: str):
        """按住按键不放"""
        try:
            import pyautogui
            pyautogui.keyDown(key)
        except ImportError:
            pass

    def key_up(self, key: str):
        """释放按键"""
        try:
            import pyautogui
            pyautogui.keyUp(key)
        except ImportError:
            pass

    def type_text(self, text: str, interval: float = 0.01):
        """输入文本"""
        try:
            import pyautogui
            pyautogui.typewrite(text, interval=interval)
            self._notify_action('type_text', {'text': text})
        except ImportError:
            for char in text:
                self.key_press(char)

    # ==================== 窗口管理 ====================

    def get_window_list(self) -> List[WindowInfo]:
        """获取所有窗口列表"""
        win32gui, win32con, win32api, win32ui = self._init_win32()
        windows = []

        def enum_callback(hwnd, results):
            if win32gui.IsWindowVisible(hwnd):
                title = win32gui.GetWindowText(hwnd)
                if title:
                    rect = win32gui.GetWindowRect(hwnd)
                    bounds = Rectangle(rect[0], rect[1], rect[2] - rect[0], rect[3] - rect[1])
                    is_active = hwnd == win32gui.GetForegroundWindow()
                    windows.append(WindowInfo(
                        title=title,
                        bounds=bounds,
                        is_active=is_active,
                        window_id=str(hwnd)
                    ))

        win32gui.EnumWindows(enum_callback, None)
        return windows

    def find_window(self, title_pattern: str) -> Optional[WindowInfo]:
        """根据标题查找窗口"""
        win32gui, win32con, win32api, win32ui = self._init_win32()
        hwnd = win32gui.FindWindow(None, title_pattern)
        if hwnd:
            rect = win32gui.GetWindowRect(hwnd)
            bounds = Rectangle(rect[0], rect[1], rect[2] - rect[0], rect[3] - rect[1])
            return WindowInfo(
                title=title_pattern,
                bounds=bounds,
                is_active=hwnd == win32gui.GetForegroundWindow(),
                window_id=str(hwnd)
            )
        return None

    def activate_window(self, window_id: str):
        """激活指定窗口"""
        win32gui, win32con, win32api, win32ui = self._init_win32()
        hwnd = int(window_id)
        if win32gui.IsIconic(hwnd):
            win32gui.ShowWindow(hwnd, win32con.SW_RESTORE)
        win32gui.SetForegroundWindow(hwnd)
        self._notify_action('activate_window', {'window_id': window_id})

    def resize_window(self, window_id: str, width: int, height: int):
        """调整窗口大小"""
        win32gui, win32con, win32api, win32ui = self._init_win32()
        hwnd = int(window_id)
        win32gui.SetWindowPos(hwnd, 0, 0, 0, width, height,
                              win32con.SWP_NOMOVE | win32con.SWP_NOZORDER)
        self._notify_action('resize_window', {'window_id': window_id, 'width': width, 'height': height})

    def move_window(self, window_id: str, x: int, y: int):
        """移动窗口位置"""
        win32gui, win32con, win32api, win32ui = self._init_win32()
        hwnd = int(window_id)
        win32gui.SetWindowPos(hwnd, 0, x, y, 0, 0,
                              win32con.SWP_NOSIZE | win32con.SWP_NOZORDER)
        self._notify_action('move_window', {'window_id': window_id, 'x': x, 'y': y})

    # ==================== 无障碍树访问 ====================

    def get_accessibility_tree(self) -> Optional[AccessibilityNode]:
        """获取完整的无障碍树"""
        logger.warning("Windows UIA 无障碍树访问需要实现")
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
        self._hwnd_cache.clear()
