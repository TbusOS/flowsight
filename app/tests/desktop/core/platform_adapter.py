"""
跨平台桌面自动化适配器基类
提供统一的截图、鼠标、键盘、无障碍树访问接口
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum, auto
from typing import Optional, List, Callable
import sys
from PIL import Image
import logging

logger = logging.getLogger(__name__)


class Platform(Enum):
    """支持的操作系统平台"""
    MACOS = auto()
    WINDOWS = auto()
    LINUX = auto()


@dataclass
class Point:
    """二维坐标点"""
    x: int
    y: int


@dataclass
class Rectangle:
    """矩形区域"""
    x: int
    y: int
    width: int
    height: int

    @property
    def center(self) -> Point:
        return Point(self.x + self.width // 2, self.y + self.height // 2)

    @property
    def right(self) -> int:
        return self.x + self.width

    @property
    def bottom(self) -> int:
        return self.y + self.height


@dataclass
class AccessibilityNode:
    """无障碍树节点"""
    role: str
    label: Optional[str]
    value: Optional[str]
    bounds: Rectangle
    children: List['AccessibilityNode']
    properties: dict


@dataclass
class WindowInfo:
    """窗口信息"""
    title: str
    bounds: Rectangle
    is_active: bool
    window_id: Optional[str] = None


class PlatformAdapter(ABC):
    """跨平台适配器抽象基类"""

    def __init__(self):
        self.platform = self._detect_platform()
        self._screenshot_listeners: List[Callable[[Image.Image], None]] = []
        self._action_listeners: List[Callable[[str, dict], None]] = []

    def _detect_platform(self) -> Platform:
        """检测当前操作系统"""
        if sys.platform == 'darwin':
            return Platform.MACOS
        elif sys.platform == 'win32':
            return Platform.WINDOWS
        else:
            return Platform.LINUX

    @classmethod
    def create(cls) -> 'PlatformAdapter':
        """工厂方法：创建适合当前平台的适配器实例"""
        platform = cls._detect_platform_static()

        if platform == Platform.MACOS:
            from ..adapters.macos_adapter import MacOSAdapter
            return MacOSAdapter()
        elif platform == Platform.WINDOWS:
            from ..adapters.windows_adapter import WindowsAdapter
            return WindowsAdapter()
        else:
            from ..adapters.linux_adapter import LinuxAdapter
            return LinuxAdapter()

    @staticmethod
    def _detect_platform_static() -> Platform:
        """静态方法检测平台（供工厂方法使用）"""
        if sys.platform == 'darwin':
            return Platform.MACOS
        elif sys.platform == 'win32':
            return Platform.WINDOWS
        else:
            return Platform.LINUX

    # ==================== 截图相关 ====================

    @abstractmethod
    def screenshot(self) -> Image.Image:
        """截取整个屏幕"""
        pass

    @abstractmethod
    def screenshot_region(self, region: Rectangle) -> Image.Image:
        """截取指定区域"""
        pass

    @abstractmethod
    def screenshot_window(self, window_title: Optional[str] = None) -> Image.Image:
        """截取指定窗口（默认活动窗口）"""
        pass

    def add_screenshot_listener(self, listener: Callable[[Image.Image], None]):
        """添加截图监听器（用于 AI 分析）"""
        self._screenshot_listeners.append(listener)

    def _notify_screenshot(self, image: Image.Image):
        """通知所有截图监听器"""
        for listener in self._screenshot_listeners:
            try:
                listener(image)
            except Exception as e:
                logger.error(f"Screenshot listener error: {e}")

    # ==================== 鼠标控制 ====================

    @abstractmethod
    def mouse_move(self, point: Point):
        """移动鼠标到指定位置"""
        pass

    @abstractmethod
    def mouse_click(self, point: Point, button: str = 'left'):
        """在指定位置点击鼠标"""
        pass

    @abstractmethod
    def mouse_double_click(self, point: Point, button: str = 'left'):
        """在指定位置双击鼠标"""
        pass

    @abstractmethod
    def mouse_drag(self, start: Point, end: Point, button: str = 'left'):
        """拖拽鼠标"""
        pass

    @abstractmethod
    def mouse_scroll(self, delta: int, direction: str = 'vertical'):
        """滚动鼠标滚轮"""
        pass

    @abstractmethod
    def get_mouse_position(self) -> Point:
        """获取当前鼠标位置"""
        pass

    # ==================== 键盘控制 ====================

    @abstractmethod
    def key_press(self, key: str):
        """按下并释放单个按键"""
        pass

    @abstractmethod
    def key_down(self, key: str):
        """按住按键不放"""
        pass

    @abstractmethod
    def key_up(self, key: str):
        """释放按键"""
        pass

    @abstractmethod
    def type_text(self, text: str, interval: float = 0.01):
        """输入文本"""
        pass

    def hotkey(self, *keys: str):
        """按下组合键（如 Ctrl+C）"""
        for key in keys:
            self.key_down(key)
        for key in reversed(keys):
            self.key_up(key)

    # ==================== 窗口管理 ====================

    @abstractmethod
    def get_window_list(self) -> List[WindowInfo]:
        """获取所有窗口列表"""
        pass

    @abstractmethod
    def find_window(self, title_pattern: str) -> Optional[WindowInfo]:
        """根据标题查找窗口"""
        pass

    @abstractmethod
    def activate_window(self, window_id: str):
        """激活指定窗口"""
        pass

    @abstractmethod
    def resize_window(self, window_id: str, width: int, height: int):
        """调整窗口大小"""
        pass

    @abstractmethod
    def move_window(self, window_id: str, x: int, y: int):
        """移动窗口位置"""
        pass

    # ==================== 无障碍树访问 ====================

    @abstractmethod
    def get_accessibility_tree(self) -> Optional[AccessibilityNode]:
        """获取完整的无障碍树"""
        pass

    @abstractmethod
    def find_accessibility_node(
        self,
        role: Optional[str] = None,
        label: Optional[str] = None,
        value: Optional[str] = None
    ) -> Optional[AccessibilityNode]:
        """在无障碍树中查找节点"""
        pass

    # ==================== 实用方法 ====================

    def wait(self, seconds: float):
        """等���指定秒数"""
        import time
        time.sleep(seconds)

    def click_element(self, bounds: Rectangle, button: str = 'left'):
        """点击元素中心位置"""
        center = bounds.center
        self.mouse_click(center, button)

    def add_action_listener(self, listener: Callable[[str, dict], None]):
        """添加动作监听器（用于记录测试步骤）"""
        self._action_listeners.append(listener)

    def _notify_action(self, action: str, params: dict):
        """通知所有动作监听器"""
        for listener in self._action_listeners:
            try:
                listener(action, params)
            except Exception as e:
                logger.error(f"Action listener error: {e}")

    def __enter__(self):
        """上下文管理器入口"""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """上下文管理器出口，清理资源"""
        self.cleanup()

    @abstractmethod
    def cleanup(self):
        """清理资源"""
        pass
