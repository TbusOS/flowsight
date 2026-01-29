"""
WebView 检测器
=============

自动检测 Tauri 应用的 WebView 并获取其调试端口。
支持 macOS、Windows 和 Linux 平台。
"""

import asyncio
import json
import re
import subprocess
from dataclasses import dataclass
from enum import Enum, auto
from typing import List, Optional, Dict, Any
import logging
import aiohttp

logger = logging.getLogger(__name__)


class WebViewType(Enum):
    """WebView 类型"""
    TAURI_WEBVIEW = auto()      # Tauri WebView (WKWebView/WebView2/WebKitGTK)
    ELECTRON = auto()           # Electron 应用
    CHROME_APP = auto()         # Chrome 应用
    OTHER = auto()              # 其他 WebView


@dataclass
class WebViewInfo:
    """WebView 信息"""
    pid: int
    title: str
    url: str
    ws_url: Optional[str]  # WebSocket URL for CDP
    webview_type: WebViewType
    debug_port: int
    metadata: Dict[str, Any]

    def __str__(self) -> str:
        return f"WebView(pid={self.pid}, title='{self.title}', port={self.debug_port})"


class WebViewDetector:
    """
    WebView 检测器

    自动扫描系统进程，发现支持远程调试的 WebView 实例。
    主要用于 Tauri 应用，但也支持其他基于 Chromium/WebKit 的应用。
    """

    # 默认调试端口范围
    DEFAULT_PORT_RANGE = (9000, 10000)

    def __init__(self):
        self.platform = self._detect_platform()
        self._cached_webviews: List[WebViewInfo] = []

    def _detect_platform(self) -> str:
        """检测当前平台"""
        import sys
        if sys.platform == 'darwin':
            return 'macos'
        elif sys.platform == 'win32':
            return 'windows'
        else:
            return 'linux'

    async def find_tauri_webviews(
        self,
        app_name: Optional[str] = None,
        timeout: float = 10.0
    ) -> List[WebViewInfo]:
        """
        查找 Tauri WebView 实例

        Args:
            app_name: 应用名称（可选，用于过滤）
            timeout: 搜索超时时间

        Returns:
            发现的 WebView 列表
        """
        logger.info(f"扫描 Tauri WebView (app_name={app_name})...")

        if self.platform == 'macos':
            webviews = await self._find_macos_webviews(app_name)
        elif self.platform == 'windows':
            webviews = await self._find_windows_webviews(app_name)
        else:
            webviews = await self._find_linux_webviews(app_name)

        # 验证并过滤可调试的 WebView
        valid_webviews = []
        for webview in webviews:
            if await self._verify_debuggable(webview):
                valid_webviews.append(webview)

        self._cached_webviews = valid_webviews
        logger.info(f"发现 {len(valid_webviews)} 个可调试 WebView")
        return valid_webviews

    async def _find_macos_webviews(
        self,
        app_name: Optional[str] = None
    ) -> List[WebViewInfo]:
        """查找 macOS WebView"""
        webviews = []

        # 1. 查找正在运行的 Tauri 应用进程
        try:
            cmd = ['pgrep', '-x', app_name] if app_name else ['pgrep', '-f', 'WebKit']
            result = subprocess.run(cmd, capture_output=True, text=True)
            if result.returncode == 0:
                pids = [int(p.strip()) for p in result.stdout.strip().split('\n') if p.strip()]
            else:
                pids = []
        except Exception as e:
            logger.warning(f"pgrep 执行失败: {e}")
            pids = []

        # 2. 扫描 WebView 远程调试端口
        # Tauri 使用 WKWebView，通常通过 --remote-debugging-port 参数启动
        for pid in pids[:5]:  # 限制扫描数量
            try:
                # 获取进程命令行参数
                cmd_result = subprocess.run(
                    ['ps', '-p', str(pid), '-o', 'command='],
                    capture_output=True,
                    text=True
                )
                cmdline = cmd_result.stdout.strip()

                # 查找调试端口参数
                port_match = re.search(r'--remote-debugging-port=(\d+)', cmdline)
                if port_match:
                    port = int(port_match.group(1))
                else:
                    # 尝试常见端口
                    port = await self._scan_debug_port(pid)
                    if not port:
                        continue

                # 获取 WebView 基本信息
                webview = await self._get_webview_info_from_port(pid, port)
                if webview:
                    webviews.append(webview)

            except Exception as e:
                logger.debug(f"检查进程 {pid} 失败: {e}")

        return webviews

    async def _find_windows_webviews(
        self,
        app_name: Optional[str] = None
    ) -> List[WebViewInfo]:
        """查找 Windows WebView2"""
        webviews = []

        try:
            # 使用 PowerShell 查找 WebView2 进程
            ps_cmd = f"""
            Get-CimInstance Win32_Process |
            Where-Object {{ $_.Name -like '*{app_name}*' -or $_.CommandLine -like '*WebView2*' }} |
            Select-Object ProcessId, CommandLine
            """

            result = subprocess.run(
                ['powershell', '-Command', ps_cmd],
                capture_output=True,
                text=True
            )

            for line in result.stdout.split('\n'):
                if not line.strip() or 'ProcessId' in line:
                    continue

                # 解析 PID 和命令行
                parts = line.split(None, 1)
                if len(parts) >= 2:
                    try:
                        pid = int(parts[0])
                        cmdline = parts[1]

                        # 查找调试端口
                        port_match = re.search(r'--remote-debugging-port=(\d+)', cmdline)
                        if port_match:
                            port = int(port_match.group(1))
                        else:
                            port = await self._scan_debug_port(pid)

                        if port:
                            webview = await self._get_webview_info_from_port(pid, port)
                            if webview:
                                webviews.append(webview)
                    except ValueError:
                        continue

        except Exception as e:
            logger.warning(f"Windows WebView 检测失败: {e}")

        return webviews

    async def _find_linux_webviews(
        self,
        app_name: Optional[str] = None
    ) -> List[WebViewInfo]:
        """查找 Linux WebKitGTK WebView"""
        webviews = []

        try:
            # 查找 WebKit 进程
            cmd = ['pgrep', '-f', app_name or 'webkit']
            result = subprocess.run(cmd, capture_output=True, text=True)

            if result.returncode == 0:
                pids = [int(p.strip()) for p in result.stdout.strip().split('\n') if p.strip()]

                for pid in pids[:5]:
                    # 扫描调试端口
                    port = await self._scan_debug_port(pid)
                    if port:
                        webview = await self._get_webview_info_from_port(pid, port)
                        if webview:
                            webviews.append(webview)

        except Exception as e:
            logger.warning(f"Linux WebView 检测失败: {e}")

        return webviews

    async def _scan_debug_port(self, pid: int) -> Optional[int]:
        """扫描进程的调试端口"""
        # 常用调试端口范围
        common_ports = [9222, 9223, 9224, 9225, 9229, 9230, 8324, 12345]

        for port in common_ports:
            try:
                async with aiohttp.ClientSession() as session:
                    async with session.get(
                        f'http://localhost:{port}/json',
                        timeout=aiohttp.ClientTimeout(total=2)
                    ) as response:
                        if response.status == 200:
                            return port
            except Exception:
                continue

        return None

    async def _verify_debuggable(self, webview: WebViewInfo) -> bool:
        """验证 WebView 是否可调试"""
        if not webview.ws_url:
            return False

        try:
            # 尝试连接 WebSocket
            import aiohttp
            async with aiohttp.ClientSession() as session:
                async with session.ws_connect(
                    webview.ws_url,
                    timeout=aiohttp.ClientTimeout(total=5)
                ):
                    return True
        except Exception:
            return False

    async def _get_webview_info_from_port(
        self,
        pid: int,
        port: int
    ) -> Optional[WebViewInfo]:
        """从调试端口获取 WebView 信息"""
        try:
            async with aiohttp.ClientSession() as session:
                async with session.get(
                    f'http://localhost:{port}/json',
                    timeout=aiohttp.ClientTimeout(total=5)
                ) as response:
                    if response.status != 200:
                        return None

                    pages = await response.json()
                    if not pages:
                        return None

                    # 获取第一个页面的信息
                    page = pages[0]

                    return WebViewInfo(
                        pid=pid,
                        title=page.get('title', 'Unknown'),
                        url=page.get('url', ''),
                        ws_url=page.get('webSocketDebuggerUrl'),
                        webview_type=WebViewType.TAURI_WEBVIEW,
                        debug_port=port,
                        metadata={
                            'devtoolsFrontendUrl': page.get('devtoolsFrontendUrl'),
                            'id': page.get('id'),
                            'type': page.get('type', 'page')
                        }
                    )

        except Exception as e:
            logger.debug(f"获取 WebView 信息失败 (port={port}): {e}")
            return None

    async def wait_for_webview(
        self,
        app_name: str,
        timeout: float = 30.0,
        poll_interval: float = 1.0
    ) -> Optional[WebViewInfo]:
        """
        等待 WebView 启动

        Args:
            app_name: 应用名称
            timeout: 最大等待时间
            poll_interval: 轮询间隔

        Returns:
            WebView 信息，如果超时返回 None
        """
        logger.info(f"等待 {app_name} WebView 启动...")

        start_time = asyncio.get_event_loop().time()
        while asyncio.get_event_loop().time() - start_time < timeout:
            webviews = await self.find_tauri_webviews(app_name)
            if webviews:
                logger.info(f"✓ WebView 已启动: {webviews[0]}")
                return webviews[0]

            await asyncio.sleep(poll_interval)

        logger.error(f"✗ 等待 WebView 超时 ({timeout}s)")
        return None

    def get_cached_webviews(self) -> List[WebViewInfo]:
        """获取缓存的 WebView 列表"""
        return self._cached_webviews.copy()

    async def kill_webview(self, webview: WebViewInfo) -> bool:
        """
        终止 WebView 进程

        Args:
            webview: WebView 信息

        Returns:
            是否成功终止
        """
        try:
            if self.platform == 'windows':
                subprocess.run(['taskkill', '/F', '/PID', str(webview.pid)], check=True)
            else:
                import signal
                import os
                os.kill(webview.pid, signal.SIGTERM)

                # 等待进程退出
                for _ in range(10):
                    try:
                        os.kill(webview.pid, 0)
                        await asyncio.sleep(0.5)
                    except ProcessLookupError:
                        return True

                # 强制终止
                os.kill(webview.pid, signal.SIGKILL)

            return True

        except Exception as e:
            logger.error(f"终止 WebView 失败: {e}")
            return False
