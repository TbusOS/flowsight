"""
Linux 平台适配器实现
使用 xdotool + AT-SPI + X11
"""

from typing import Optional, List
from PIL import Image
import logging
import subprocess
import os

from ..core.platform_adapter import (
    PlatformAdapter, Point, Rectangle, AccessibilityNode,
    WindowInfo, Platform
)

logger = logging.getLogger(__name__)


class LinuxAdapter(PlatformAdapter):
    """Linux 平台自动化适配器 (X11/Wayland)"""

    def __init__(self):
        super().__init__()
        self.platform = Platform.LINUX
        self._display = os.environ.get('DISPLAY', ':0')
        self._is_wayland = self._detect_wayland()

    def _detect_wayland(self) -> bool:
        """检测是否在 Wayland 环境下运行"""
        wayland_display = os.environ.get('WAYLAND_DISPLAY')
        xdg_session_type = os.environ.get('XDG_SESSION_TYPE', '').lower()
        return wayland_display is not None or xdg_session_type == 'wayland'

    def _check_xdotool(self):
        """检查 xdotool 是否安装"""
        result = subprocess.run(['which', 'xdotool'], capture_output=True)
        if result.returncode != 0:
            raise RuntimeError("需要安装 xdotool: sudo apt-get install xdotool (或对应包管理器)")

    # ==================== 截图相关 ====================

    def screenshot(self) -> Image.Image:
        """截取整个屏幕"""
        if self._is_wayland:
            # Wayland 环境使用 grim
            return self._screenshot_wayland()
        else:
            # X11 环境
            return self._screenshot_x11()

    def _screenshot_wayland(self) -> Image.Image:
        """在 Wayland 环境下截图"""
        import tempfile
        with tempfile.NamedTemporaryFile(suffix='.png', delete=False) as f:
            temp_path = f.name

        # 使用 grim (Wayland 截图工具)
        result = subprocess.run(['grim', temp_path], capture_output=True)
        if result.returncode == 0:
            image = Image.open(temp_path)
            os.unlink(temp_path)
            self._notify_screenshot(image)
            return image
        else:
            # 回退到使用 GIMP 或 ImageMagick
            result = subprocess.run(['import', '-window', 'root', temp_path], capture_output=True)
            if result.returncode == 0:
                image = Image.open(temp_path)
                os.unlink(temp_path)
                self._notify_screenshot(image)
                return image
            raise RuntimeError("Wayland 截图失败，请安装 grim 或 ImageMagick")

    def _screenshot_x11(self) -> Image.Image:
        """在 X11 环境下截图"""
        import tempfile
        with tempfile.NamedTemporaryFile(suffix='.png', delete=False) as f:
            temp_path = f.name

        # 尝试使用 gnome-screenshot
        result = subprocess.run(['gnome-screenshot', '-f', temp_path], capture_output=True)
        if result.returncode != 0:
            # 回退到 import (ImageMagick)
            result = subprocess.run(['import', '-window', 'root', temp_path], capture_output=True)

        if result.returncode == 0:
            image = Image.open(temp_path)
            os.unlink(temp_path)
            self._notify_screenshot(image)
            return image
        else:
            raise RuntimeError("X11 截图失败，请安装 gnome-screenshot 或 ImageMagick")

    def screenshot_region(self, region: Rectangle) -> Image.Image:
        """截取指定区域"""
        import tempfile
        with tempfile.NamedTemporaryFile(suffix='.png', delete=False) as f:
            temp_path = f.name

        if self._is_wayland:
            # Wayland: grim -g "x,y w,h"
            geometry = f"{region.x},{region.y} {region.width}x{region.height}"
            result = subprocess.run(['grim', '-g', geometry, temp_path], capture_output=True)
        else:
            # X11
            result = subprocess.run([
                'import', '-window', 'root',
                '-crop', f"{region.width}x{region.height}+{region.x}+{region.y}",
                temp_path
            ], capture_output=True)

        if result.returncode == 0:
            image = Image.open(temp_path)
            os.unlink(temp_path)
            return image
        else:
            # 回退到全屏截图后裁剪
            full = self.screenshot()
            return full.crop((region.x, region.y, region.right, region.bottom))

    def screenshot_window(self, window_title: Optional[str] = None) -> Image.Image:
        """截取指定窗口"""
        if window_title:
            window_id = self._get_window_id_by_title(window_title)
            if not window_id:
                raise ValueError(f"未找到窗口: {window_title}")
        else:
            window_id = self._get_active_window_id()

        import tempfile
        with tempfile.NamedTemporaryFile(suffix='.png', delete=False) as f:
            temp_path = f.name

        if self._is_wayland:
            # Wayland: 需要获取窗口几何信息
            geometry = self._get_window_geometry(window_id)
            return self.screenshot_region(geometry)
        else:
            # X11: 使用 import -window window_id
            result = subprocess.run(['import', '-window', window_id, temp_path], capture_output=True)
            if result.returncode == 0:
                image = Image.open(temp_path)
                os.unlink(temp_path)
                self._notify_screenshot(image)
                return image
            else:
                raise RuntimeError(f"窗口截图失败: {result.stderr.decode()}")

    def _get_window_id_by_title(self, title: str) -> Optional[str]:
        """根据标题获取窗口 ID"""
        try:
            result = subprocess.run(
                ['xdotool', 'search', '--name', title],
                capture_output=True, text=True
            )
            if result.returncode == 0 and result.stdout.strip():
                return result.stdout.strip().split('\n')[0]
        except Exception as e:
            logger.error(f"获取窗口 ID 失败: {e}")
        return None

    def _get_active_window_id(self) -> str:
        """获取活动窗口 ID"""
        result = subprocess.run(
            ['xdotool', 'getactivewindow'],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            return result.stdout.strip()
        raise RuntimeError("无法获取活动窗口")

    def _get_window_geometry(self, window_id: str) -> Rectangle:
        """获取窗口几何信息"""
        result = subprocess.run(
            ['xdotool', 'getwindowgeometry', window_id],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            # 解析输出: Position: 100,200 (screen: 0)
            #          Geometry: 800x600
            lines = result.stdout.strip().split('\n')
            pos_line = [l for l in lines if 'Position:' in l][0]
            geo_line = [l for l in lines if 'Geometry:' in l][0]

            pos_parts = pos_line.split(':')[1].split('(')[0].strip().split(',')
            x, y = int(pos_parts[0]), int(pos_parts[1])

            geo_parts = geo_line.split(':')[1].strip().split('x')
            w, h = int(geo_parts[0]), int(geo_parts[1])

            return Rectangle(x, y, w, h)
        raise RuntimeError("无法获取窗口几何信息")

    # ==================== 鼠标控制 ====================

    def mouse_move(self, point: Point):
        """移动鼠标到指定位置"""
        self._check_xdotool()
        subprocess.run(['xdotool', 'mousemove', str(point.x), str(point.y)], capture_output=True)
        self._notify_action('mouse_move', {'x': point.x, 'y': point.y})

    def mouse_click(self, point: Point, button: str = 'left'):
        """在指定位置点击鼠标"""
        self._check_xdotool()
        self.mouse_move(point)
        btn = {'left': '1', 'right': '3', 'middle': '2'}.get(button, '1')
        subprocess.run(['xdotool', 'click', btn], capture_output=True)
        self._notify_action('mouse_click', {'x': point.x, 'y': point.y, 'button': button})

    def mouse_double_click(self, point: Point, button: str = 'left'):
        """在指定位置双击鼠标"""
        self._check_xdotool()
        self.mouse_move(point)
        btn = {'left': '1', 'right': '3', 'middle': '2'}.get(button, '1')
        subprocess.run(['xdotool', 'click', '--repeat', '2', btn], capture_output=True)
        self._notify_action('mouse_double_click', {'x': point.x, 'y': point.y, 'button': button})

    def mouse_drag(self, start: Point, end: Point, button: str = 'left'):
        """拖拽鼠标"""
        self._check_xdotool()
        self.mouse_move(start)

        btn_map = {'left': '1', 'right': '3', 'middle': '2'}
        btn = btn_map.get(button, '1')

        # 按下
        subprocess.run(['xdotool', 'mousedown', btn], capture_output=True)
        # 移动
        subprocess.run(['xdotool', 'mousemove', str(end.x), str(end.y)], capture_output=True)
        # 释放
        subprocess.run(['xdotool', 'mouseup', btn], capture_output=True)

        self._notify_action('mouse_drag', {
            'start_x': start.x, 'start_y': start.y,
            'end_x': end.x, 'end_y': end.y,
            'button': button
        })

    def mouse_scroll(self, delta: int, direction: str = 'vertical'):
        """滚动鼠标滚轮"""
        self._check_xdotool()
        if direction == 'vertical':
            button = '4' if delta < 0 else '5'  # 4=up, 5=down
        else:
            button = '6' if delta < 0 else '7'  # 6=left, 7=right

        clicks = abs(delta)
        subprocess.run(['xdotool', 'click', '--repeat', str(clicks), button], capture_output=True)
        self._notify_action('mouse_scroll', {'delta': delta, 'direction': direction})

    def get_mouse_position(self) -> Point:
        """获取当前鼠标位置"""
        self._check_xdotool()
        result = subprocess.run(
            ['xdotool', 'getmouselocation'],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            # 解析: x:123 y:456 screen:0 window:789
            parts = result.stdout.strip().split()
            x = int(parts[0].split(':')[1])
            y = int(parts[1].split(':')[1])
            return Point(x, y)
        raise RuntimeError("无法获取鼠标位置")

    # ==================== 键盘控制 ====================

    def key_press(self, key: str):
        """按下并释放单个按键"""
        self._check_xdotool()
        subprocess.run(['xdotool', 'key', key], capture_output=True)
        self._notify_action('key_press', {'key': key})

    def key_down(self, key: str):
        """按住按键不放"""
        self._check_xdotool()
        subprocess.run(['xdotool', 'keydown', key], capture_output=True)

    def key_up(self, key: str):
        """释放按键"""
        self._check_xdotool()
        subprocess.run(['xdotool', 'keyup', key], capture_output=True)

    def type_text(self, text: str, interval: float = 0.01):
        """输入文本"""
        self._check_xdotool()
        # 转义特殊字符
        escaped = text.replace("'", "'\"'\"'")
        subprocess.run(['xdotool', 'type', '--delay', str(int(interval * 1000)), escaped], capture_output=True)
        self._notify_action('type_text', {'text': text})

    # ==================== 窗口管理 ====================

    def get_window_list(self) -> List[WindowInfo]:
        """获取所有窗口列表"""
        windows = []
        try:
            result = subprocess.run(
                ['xdotool', 'search', '--onlyvisible', '.'],
                capture_output=True, text=True
            )
            if result.returncode == 0:
                for window_id in result.stdout.strip().split('\n'):
                    if window_id:
                        title = self._get_window_title(window_id)
                        if title:
                            geometry = self._get_window_geometry(window_id)
                            is_active = window_id == self._get_active_window_id()
                            windows.append(WindowInfo(
                                title=title,
                                bounds=geometry,
                                is_active=is_active,
                                window_id=window_id
                            ))
        except Exception as e:
            logger.error(f"获取窗口列表失败: {e}")
        return windows

    def _get_window_title(self, window_id: str) -> Optional[str]:
        """获取窗口标题"""
        result = subprocess.run(
            ['xdotool', 'getwindowname', window_id],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            return result.stdout.strip()
        return None

    def find_window(self, title_pattern: str) -> Optional[WindowInfo]:
        """根据标题查找窗口"""
        try:
            result = subprocess.run(
                ['xdotool', 'search', '--name', title_pattern],
                capture_output=True, text=True
            )
            if result.returncode == 0 and result.stdout.strip():
                window_id = result.stdout.strip().split('\n')[0]
                title = self._get_window_title(window_id)
                geometry = self._get_window_geometry(window_id)
                is_active = window_id == self._get_active_window_id()
                return WindowInfo(
                    title=title or title_pattern,
                    bounds=geometry,
                    is_active=is_active,
                    window_id=window_id
                )
        except Exception as e:
            logger.error(f"查找窗口失败: {e}")
        return None

    def activate_window(self, window_id: str):
        """激活指定窗口"""
        subprocess.run(['xdotool', 'windowactivate', window_id], capture_output=True)
        self._notify_action('activate_window', {'window_id': window_id})

    def resize_window(self, window_id: str, width: int, height: int):
        """调整窗口大小"""
        subprocess.run(['xdotool', 'windowsize', window_id, str(width), str(height)], capture_output=True)
        self._notify_action('resize_window', {'window_id': window_id, 'width': width, 'height': height})

    def move_window(self, window_id: str, x: int, y: int):
        """移动窗口位置"""
        subprocess.run(['xdotool', 'windowmove', window_id, str(x), str(y)], capture_output=True)
        self._notify_action('move_window', {'window_id': window_id, 'x': x, 'y': y})

    # ==================== 无障碍树访问 ====================

    def get_accessibility_tree(self) -> Optional[AccessibilityNode]:
        """获取完整的无障碍树 (AT-SPI)"""
        try:
            import pyatspi
            # 获取桌面对象
            desktop = pyatspi.Registry.getDesktop(0)
            # 构建树
            return self._build_atspi_tree(desktop)
        except ImportError:
            logger.warning("pyatspi 未安装，无法访问无障碍树")
            return None
        except Exception as e:
            logger.error(f"访问 AT-SPI 失败: {e}")
            return None

    def _build_atspi_tree(self, obj) -> AccessibilityNode:
        """递归构建 AT-SPI 树"""
        try:
            role = obj.getRoleName()
            name = obj.name
            bounds = obj.queryComponent().getExtents(0) if obj.queryComponent() else None

            rect = Rectangle(bounds.x, bounds.y, bounds.width, bounds.height) if bounds else Rectangle(0, 0, 0, 0)

            children = []
            try:
                for i in range(obj.childCount):
                    child = obj.getChildAtIndex(i)
                    if child:
                        children.append(self._build_atspi_tree(child))
            except:
                pass

            return AccessibilityNode(
                role=role,
                label=name,
                value=None,
                bounds=rect,
                children=children,
                properties={}
            )
        except Exception as e:
            logger.error(f"构建 AT-SPI 节点失败: {e}")
            return AccessibilityNode(
                role="unknown",
                label=None,
                value=None,
                bounds=Rectangle(0, 0, 0, 0),
                children=[],
                properties={}
            )

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
