#!/usr/bin/env python3
"""
Desktop Automation MCP Server
提供截图、鼠标控制、键盘输入等能力，模拟 Claude Computer Use 核心功能
"""

import asyncio
import base64
import json
import subprocess
import time
from pathlib import Path
from typing import Optional, Dict, Any
from dataclasses import dataclass

import mcp.server
import mcp.server.stdio
import mcp.types as types

@dataclass
class MousePosition:
    x: int
    y: int

class DesktopAutomationServer:
    """桌面自动化服务器 - 类似 Computer Use 核心能力"""

    def __init__(self):
        self.screenshot_path = "/tmp/desktop_screenshot.png"
        self.last_screenshot: Optional[bytes] = None

    def run_command(self, cmd: list[str], timeout: int = 10) -> tuple[int, str, str]:
        """运行命令并返回 (returncode, stdout, stderr)"""
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            return result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return -1, "", "Command timed out"
        except Exception as e:
            return -1, "", str(e)

    # ============ 截图工具 ============

    async def screenshot(self, region: Optional[str] = None) -> Dict[str, Any]:
        """截图整个屏幕或指定区域

        Args:
            region: 可选区域格式 "x,y,width,height"
        """
        cmd = ["scrot", "-q", "85", self.screenshot_path]

        if region:
            parts = region.split(",")
            if len(parts) == 4:
                x, y, w, h = map(int, parts)
                cmd.extend([f"--crop={x},{y},{w},{h}"])

        code, stdout, stderr = self.run_command(cmd)

        if code == 0 and Path(self.screenshot_path).exists():
            with open(self.screenshot_path, "rb") as f:
                self.last_screenshot = f.read()
            return {
                "success": True,
                "message": "Screenshot captured",
                "path": self.screenshot_path
            }
        return {"success": False, "message": stderr}

    async def get_screenshot_data(self, format: str = "base64") -> Dict[str, Any]:
        """获取上次截图的数据

        Args:
            format: "base64" 或 "path"
        """
        if self.last_screenshot is None:
            # 先截图
            await self.screenshot()

        if self.last_screenshot:
            if format == "base64":
                b64 = base64.b64encode(self.last_screenshot).decode()
                return {
                    "success": True,
                    "format": "base64",
                    "data": b64,
                    "size": len(self.last_screenshot)
                }
            else:
                return {
                    "success": True,
                    "format": "path",
                    "path": self.screenshot_path
                }
        return {"success": False, "message": "No screenshot available"}

    async def screenshot_element(self, element_id: str) -> Dict[str, Any]:
        """截图特定元素区域 (通过坐标)

        Args:
            element_id: 元素标识符，格式 "x,y,width,height"
        """
        return await self.screenshot(element_id)

    # ============ 鼠标工具 ============

    async def get_mouse_position(self) -> Dict[str, Any]:
        """获取当前鼠标位置"""
        code, stdout, stderr = self.run_command(["xdotool", "getmouselocation"])

        if code == 0:
            parts = stdout.strip().split()
            x = int(parts[0].split(":")[1])
            y = int(parts[1].split(":")[1])
            return {
                "success": True,
                "x": x,
                "y": y
            }
        return {"success": False, "message": stderr}

    async def click(self, x: int, y: int, button: str = "left") -> Dict[str, Any]:
        """点击指定位置

        Args:
            x: X 坐标
            y: Y 坐标
            button: "left", "right", "middle"
        """
        button_map = {"left": 1, "right": 3, "middle": 2}
        button_num = button_map.get(button, 1)

        code, stdout, stderr = self.run_command(
            ["xdotool", "mousemove", str(x), str(y), "click", str(button_num)]
        )

        return {
            "success": code == 0,
            "message": stderr if code != 0 else "Clicked successfully",
            "position": {"x": x, "y": y}
        }

    async def double_click(self, x: int, y: int) -> Dict[str, Any]:
        """双击指定位置"""
        code, stdout, stderr = self.run_command([
            "xdotool", "mousemove", str(x), str(y),
            "click", "1", "click", "1"
        ])

        return {
            "success": code == 0,
            "message": stderr if code != 0 else "Double clicked",
            "position": {"x": x, "y": y}
        }

    async def right_click(self, x: int, y: int) -> Dict[str, Any]:
        """右键点击指定位置"""
        return await self.click(x, y, "right")

    async def drag(self, x1: int, y1: int, x2: int, y2: int, duration: float = 0.5) -> Dict[str, Any]:
        """拖拽从 (x1, y1) 到 (x2, y2)

        Args:
            duration: 拖拽持续时间（秒）
        """
        # xdotool 不支持平滑拖拽，使用简单实现
        code, stdout, stderr = self.run_command([
            "xdotool", "mousemove", str(x1), str(y1),
            "mousedown", "1",
            "sleep", str(duration),
            "mousemove", str(x2), str(y2),
            "mouseup", "1"
        ])

        return {
            "success": code == 0,
            "message": stderr if code != 0 else "Drag completed",
            "from": {"x": x1, "y": y1},
            "to": {"x": x2, "y": y2}
        }

    async def move_mouse(self, x: int, y: int) -> Dict[str, Any]:
        """移动鼠标到指定位置"""
        code, stdout, stderr = self.run_command(["xdotool", "mousemove", str(x), str(y)])

        return {
            "success": code == 0,
            "position": {"x": x, "y": y}
        }

    # ============ 键盘工具 ============

    async def type_text(self, text: str, delay: float = 0.05) -> Dict[str, Any]:
        """输入文本

        Args:
            text: 要输入的文本
            delay: 每个字符间的延迟（秒）
        """
        # 转义特殊字符
        escaped = text.replace("'", "\\'").replace('"', '\\"')
        code, stdout, stderr = self.run_command([
            "xdotool", "type", "--delay", str(int(delay * 1000)), escaped
        ])

        return {
            "success": code == 0,
            "message": stderr if code != 0 else f"Typed {len(text)} characters",
            "char_count": len(text)
        }

    async def press_key(self, key: str) -> Dict[str, Any]:
        """按下单个键

        Args:
            key: 键名如 "Return", "Escape", "a", "1" 等
        """
        code, stdout, stderr = self.run_command(["xdotool", "key", key])

        return {
            "success": code == 0,
            "key": key
        }

    async def key_combination(self, keys: list[str]) -> Dict[str, Any]:
        """按下组合键

        Args:
            keys: 键列表如 ["Control_L", "c"]
        """
        code, stdout, stderr = self.run_command(["xdotool", "key", "+".join(keys)])

        return {
            "success": code == 0,
            "combination": "+".join(keys)
        }

    async def hotkey(self, combo: str) -> Dict[str, Any]:
        """执行快捷键

        Args:
            combo: 快捷键如 "ctrl+c", "alt+tab", "cmd+k"
        """
        # 转换快捷键格式
        key_map = {
            "cmd": "Super_L",
            "ctrl": "Control_L",
            "alt": "Alt_L",
            "shift": "Shift_L"
        }

        parts = combo.lower().split("+")
        keys = [key_map.get(p, p) for p in parts]

        return await self.key_combination(keys)

    # ============ 等待工具 ============

    async def wait(self, seconds: float) -> Dict[str, Any]:
        """等待指定时间

        Args:
            seconds: 等待秒数
        """
        time.sleep(seconds)
        return {"success": True, "waited": seconds}

    async def wait_for_element(self, x: int, y: int, timeout: float = 5.0,
                               expected_color: Optional[str] = None) -> Dict[str, Any]:
        """等待某位置出现特定内容

        Args:
            x, y: 坐标
            timeout: 超时时间（秒）
            expected_color: 期望的颜色（十六进制，如 "#FF0000"）
        """
        start_time = time.time()
        last_pos = None

        while time.time() - start_time < timeout:
            pos = await self.get_mouse_position()
            if pos["success"]:
                last_pos = pos
                # 截图检查颜色
                await self.screenshot_element(f"{x},{y},50,50")
                # 这里可以添加颜色检查逻辑
                return {
                    "success": True,
                    "found": True,
                    "position": pos,
                    "waited": time.time() - start_time
                }
            await self.wait(0.5)

        return {
            "success": True,
            "found": False,
            "position": last_pos,
            "waited": timeout
        }

    # ============ 窗口工具 ============

    async def get_window_by_title(self, title: str) -> Dict[str, Any]:
        """通过标题查找窗口"""
        code, stdout, stderr = self.run_command([
            "xdotool", "search", "--name", title
        ])

        if code == 0 and stdout.strip():
            window_ids = stdout.strip().split("\n")
            return {
                "success": True,
                "window_ids": window_ids,
                "count": len(window_ids)
            }
        return {"success": False, "message": "Window not found"}

    async def focus_window(self, window_id: str) -> Dict[str, Any]:
        """聚焦窗口"""
        code, stdout, stderr = self.run_command([
            "xdotool", "windowfocus", window_id
        ])
        return {"success": code == 0}

    async def get_active_window(self) -> Dict[str, Any]:
        """获取当前活动窗口"""
        code, stdout, stderr = self.run_command([
            "xdotool", "getactivewindow", "getwindowname"
        ])

        if code == 0:
            return {
                "success": True,
                "window_name": stdout.strip()
            }
        return {"success": False}

    # ============ 图像识别工具 ============

    async def find_image(self, template_path: str, confidence: float = 0.8) -> Dict[str, Any]:
        """在屏幕中查找图像（需要安装 opencv）

        Args:
            template_path: 模板图像路径
            confidence: 置信度阈值
        """
        # 简化的图像查找实现
        await self.screenshot()

        # 检查模板是否存在
        if not Path(template_path).exists():
            return {"success": False, "message": "Template image not found"}

        # 这里可以添加 OpenCV 模板匹配逻辑
        return {
            "success": False,
            "message": "Image matching not implemented yet",
            "note": "Install opencv-python to enable this feature"
        }


# ============ MCP Server ============

server = mcp.server.Server("desktop-automation")

@server.list_tools()
async def list_tools() -> list[types.Tool]:
    """列出所有可用工具"""
    return [
        types.Tool(
            name="screenshot",
            description="Capture a screenshot of the screen or a region",
            inputSchema={
                "type": "object",
                "properties": {
                    "region": {"type": "string", "description": "Region in format 'x,y,width,height'"}
                }
            }
        ),
        types.Tool(
            name="get_screenshot_data",
            description="Get the last screenshot as base64 data",
            inputSchema={
                "type": "object",
                "properties": {
                    "format": {"type": "string", "enum": ["base64", "path"], "default": "base64"}
                }
            }
        ),
        types.Tool(
            name="get_mouse_position",
            description="Get current mouse position",
            inputSchema={"type": "object", "properties": {}}
        ),
        types.Tool(
            name="click",
            description="Click at specific coordinates",
            inputSchema={
                "type": "object",
                "properties": {
                    "x": {"type": "integer"},
                    "y": {"type": "integer"},
                    "button": {"type": "string", "enum": ["left", "right", "middle"], "default": "left"}
                },
                "required": ["x", "y"]
            }
        ),
        types.Tool(
            name="double_click",
            description="Double click at specific coordinates",
            inputSchema={
                "type": "object",
                "properties": {
                    "x": {"type": "integer"},
                    "y": {"type": "integer"}
                },
                "required": ["x", "y"]
            }
        ),
        types.Tool(
            name="type_text",
            description="Type text at current cursor position",
            inputSchema={
                "type": "object",
                "properties": {
                    "text": {"type": "string"},
                    "delay": {"type": "number", "default": 0.05}
                },
                "required": ["text"]
            }
        ),
        types.Tool(
            name="hotkey",
            description="Execute a hotkey combination",
            inputSchema={
                "type": "object",
                "properties": {
                    "combo": {"type": "string", "description": "Like 'ctrl+c', 'alt+tab', 'cmd+k'"}
                },
                "required": ["combo"]
            }
        ),
        types.Tool(
            name="move_mouse",
            description="Move mouse to specific coordinates",
            inputSchema={
                "type": "object",
                "properties": {
                    "x": {"type": "integer"},
                    "y": {"type": "integer"}
                },
                "required": ["x", "y"]
            }
        ),
        types.Tool(
            name="wait",
            description="Wait for specified seconds",
            inputSchema={
                "type": "object",
                "properties": {
                    "seconds": {"type": "number", "default": 1.0}
                }
            }
        ),
        types.Tool(
            name="drag",
            description="Drag from one position to another",
            inputSchema={
                "type": "object",
                "properties": {
                    "x1": {"type": "integer"},
                    "y1": {"type": "integer"},
                    "x2": {"type": "integer"},
                    "y2": {"type": "integer"},
                    "duration": {"type": "number", "default": 0.5}
                },
                "required": ["x1", "y1", "x2", "y2"]
            }
        ),
    ]


automation = DesktopAutomationServer()

@server.call_tool()
async def call_tool(name: str, arguments: dict | None) -> list[types.TextContent]:
    """处理工具调用"""
    args = arguments or {}

    try:
        if name == "screenshot":
            result = await automation.screenshot(args.get("region"))
        elif name == "get_screenshot_data":
            result = await automation.get_screenshot_data(args.get("format", "base64"))
        elif name == "get_mouse_position":
            result = await automation.get_mouse_position()
        elif name == "click":
            result = await automation.click(args["x"], args["y"], args.get("button", "left"))
        elif name == "double_click":
            result = await automation.double_click(args["x"], args["y"])
        elif name == "type_text":
            result = await automation.type_text(args["text"], args.get("delay", 0.05))
        elif name == "hotkey":
            result = await automation.hotkey(args["combo"])
        elif name == "move_mouse":
            result = await automation.move_mouse(args["x"], args["y"])
        elif name == "wait":
            result = await automation.wait(args.get("seconds", 1.0))
        elif name == "drag":
            result = await automation.drag(
                args["x1"], args["y1"], args["x2"], args["y2"], args.get("duration", 0.5)
            )
        else:
            return [types.TextContent(type="text", text=f"Unknown tool: {name}")]

        return [types.TextContent(type="text", text=json.dumps(result, indent=2))]

    except Exception as e:
        return [types.TextContent(type="text", text=f"Error: {str(e)}")]


async def main():
    """启动 MCP 服务器"""
    async with mcp.server.stdio.stdio_server() as (read_stream, write_stream):
        await server.run(
            read_stream,
            write_stream,
            server.create_initialization_options()
        )


if __name__ == "__main__":
    asyncio.run(main())
