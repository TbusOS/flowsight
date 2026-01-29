"""
WebView Playwright 适配器
========================

通过 Chrome DevTools Protocol (CDP) 连接和控制 WebView。
提供与 Playwright 类似的高级 API。
"""

import asyncio
import json
from dataclasses import dataclass, field
from typing import Optional, Dict, Any, List, Callable, Union
from contextlib import asynccontextmanager
import logging

logger = logging.getLogger(__name__)

# 尝试导入 Playwright
try:
    from playwright.async_api import async_playwright, Page, Browser, BrowserContext
    from playwright.async_api import Error as PlaywrightError
    PLAYWRIGHT_AVAILABLE = True
except ImportError:
    PLAYWRIGHT_AVAILABLE = False
    logger.warning("Playwright 未安装，WebView 测试功能不可用")
    # 定义占位符类型
    Page = Any
    Browser = Any
    BrowserContext = Any


class WebViewConnectionError(Exception):
    """WebView 连接错误"""
    pass


@dataclass
class ConsoleMessage:
    """控制台消息"""
    type: str  # log, warn, error, info
    text: str
    location: Dict[str, Any] = field(default_factory=dict)
    timestamp: float = 0.0


@dataclass
class PerformanceMetric:
    """性能指标"""
    name: str
    value: float
    timestamp: float


@dataclass
class WebViewMetrics:
    """WebView 性能指标集合"""
    load_time_ms: float = 0.0
    dom_content_loaded_ms: float = 0.0
    first_paint_ms: float = 0.0
    first_contentful_paint_ms: float = 0.0
    largest_contentful_paint_ms: float = 0.0
    js_heap_used_size: int = 0
    js_heap_total_size: int = 0


class WebViewAdapter:
    """
    WebView Playwright 适配器

    通过 CDP 连接到 Tauri WebView，提供 Playwright 风格的 API。
    支持 DOM 操作、事件监听、性能监控等功能。

    使用示例:
        adapter = WebViewAdapter()
        await adapter.connect_to_tauri_app("FlowSight")

        # 导航到页面
        await adapter.goto("https://example.com")

        # 点击元素
        await adapter.click("button#submit")

        # 获取元素文本
        text = await adapter.get_text("h1")

        # 截图
        screenshot = await adapter.screenshot()
    """

    def __init__(self):
        if not PLAYWRIGHT_AVAILABLE:
            raise RuntimeError(
                "Playwright 未安装。请运行: pip install playwright && playwright install"
            )

        self._playwright = None
        self._browser: Optional[Browser] = None
        self._context: Optional[BrowserContext] = None
        self._page: Optional[Page] = None
        self._cdp_session: Optional[Any] = None
        self._is_connected = False

        # 事件监听
        self._console_listeners: List[Callable[[ConsoleMessage], None]] = []
        self._error_listeners: List[Callable[[str], None]] = []

        # 性能数据
        self._metrics: List[PerformanceMetric] = []
        self._console_logs: List[ConsoleMessage] = []

        # WebView 信息
        self._webview_info: Optional[Any] = None

    @property
    def is_connected(self) -> bool:
        """是否已连接"""
        return self._is_connected and self._page is not None

    @property
    def page(self) -> Optional[Page]:
        """获取当前页面"""
        return self._page

    # ==================== 连接管理 ====================

    async def connect_to_tauri_app(
        self,
        app_name: str,
        timeout: float = 30.0
    ) -> bool:
        """
        连接到 Tauri 应用的 WebView

        Args:
            app_name: 应用名称
            timeout: 连接超时时间

        Returns:
            是否连接成功
        """
        from .webview_detector import WebViewDetector

        detector = WebViewDetector()

        # 等待 WebView 启动
        webview = await detector.wait_for_webview(app_name, timeout)
        if not webview:
            raise WebViewConnectionError(
                f"无法找到应用 '{app_name}' 的 WebView"
            )

        return await self.connect_to_webview(webview)

    async def connect_to_webview(self, webview_info) -> bool:
        """
        连接到指定的 WebView

        Args:
            webview_info: WebViewInfo 对象

        Returns:
            是否连接成功
        """
        try:
            self._playwright = await async_playwright().start()

            # 通过 CDP 连接
            self._browser = await self._playwright.chromium.connect_over_cdp(
                f"http://localhost:{webview_info.debug_port}"
            )

            # 获取或创建上下文
            contexts = self._browser.contexts
            if contexts:
                self._context = contexts[0]
            else:
                self._context = await self._browser.new_context()

            # 获取或创建页面
            pages = self._context.pages
            if pages:
                self._page = pages[0]
            else:
                self._page = await self._context.new_page()

            # 设置事件监听
            await self._setup_event_listeners()

            self._webview_info = webview_info
            self._is_connected = True

            logger.info(f"✓ 已连接到 WebView: {webview_info.title}")
            return True

        except Exception as e:
            logger.error(f"连接 WebView 失败: {e}")
            await self.disconnect()
            raise WebViewConnectionError(f"连接失败: {e}")

    async def connect_to_port(self, port: int) -> bool:
        """
        直接通过调试端口连接

        Args:
            port: 调试端口

        Returns:
            是否连接成功
        """
        try:
            self._playwright = await async_playwright().start()

            self._browser = await self._playwright.chromium.connect_over_cdp(
                f"http://localhost:{port}"
            )

            contexts = self._browser.contexts
            if contexts:
                self._context = contexts[0]
                pages = self._context.pages
                if pages:
                    self._page = pages[0]

            if not self._page:
                raise WebViewConnectionError("无法获取页面")

            await self._setup_event_listeners()
            self._is_connected = True

            logger.info(f"✓ 已连接到端口 {port}")
            return True

        except Exception as e:
            logger.error(f"连接端口 {port} 失败: {e}")
            await self.disconnect()
            raise

    async def disconnect(self):
        """断开连接"""
        try:
            if self._page:
                await self._page.close()
                self._page = None

            if self._context:
                await self._context.close()
                self._context = None

            if self._browser:
                await self._browser.close()
                self._browser = None

            if self._playwright:
                await self._playwright.stop()
                self._playwright = None

            self._is_connected = False
            logger.info("已断开 WebView 连接")

        except Exception as e:
            logger.warning(f"断开连接时出错: {e}")

    async def _setup_event_listeners(self):
        """设置页面事件监听"""
        if not self._page:
            return

        # 控制台消息
        self._page.on("console", self._on_console_message)

        # 页面错误
        self._page.on("pageerror", self._on_page_error)

        # 请求失败
        self._page.on("requestfailed", self._on_request_failed)

        # 性能指标
        await self._page.evaluate("""
            () => {
                window.performanceMetrics = [];
                new PerformanceObserver((list) => {
                    for (const entry of list.getEntries()) {
                        window.performanceMetrics.push({
                            name: entry.name,
                            value: entry.startTime,
                            duration: entry.duration
                        });
                    }
                }).observe({entryTypes: ['paint', 'largest-contentful-paint', 'measure']});
            }
        """)

    async def _on_console_message(self, msg):
        """处理控制台消息"""
        console_msg = ConsoleMessage(
            type=msg.type,
            text=msg.text,
            location=msg.location if hasattr(msg, 'location') else {},
            timestamp=asyncio.get_event_loop().time()
        )
        self._console_logs.append(console_msg)

        # 通知监听器
        for listener in self._console_listeners:
            try:
                if asyncio.iscoroutinefunction(listener):
                    await listener(console_msg)
                else:
                    listener(console_msg)
            except Exception as e:
                logger.debug(f"控制台监听器出错: {e}")

    async def _on_page_error(self, error):
        """处理页面错误"""
        error_msg = str(error)
        logger.warning(f"页面错误: {error_msg}")

        for listener in self._error_listeners:
            try:
                if asyncio.iscoroutinefunction(listener):
                    await listener(error_msg)
                else:
                    listener(error_msg)
            except Exception as e:
                logger.debug(f"错误监听器出错: {e}")

    async def _on_request_failed(self, request):
        """处理请求失败"""
        logger.warning(f"请求失败: {request.url}")

    # ==================== 导航和页面操作 ====================

    async def goto(self, url: str, wait_until: str = "networkidle") -> bool:
        """
        导航到指定 URL

        Args:
            url: 目标 URL
            wait_until: 等待条件 (load, domcontentloaded, networkidle)

        Returns:
            是否导航成功
        """
        if not self._page:
            raise WebViewConnectionError("未连接到 WebView")

        try:
            response = await self._page.goto(url, wait_until=wait_until)
            return response is not None and response.ok
        except Exception as e:
            logger.error(f"导航到 {url} 失败: {e}")
            return False

    async def reload(self, wait_until: str = "networkidle") -> bool:
        """刷新页面"""
        if not self._page:
            return False

        try:
            await self._page.reload(wait_until=wait_until)
            return True
        except Exception as e:
            logger.error(f"刷新页面失败: {e}")
            return False

    async def go_back(self) -> bool:
        """返回上一页"""
        if not self._page:
            return False

        try:
            await self._page.go_back()
            return True
        except Exception as e:
            logger.error(f"返回失败: {e}")
            return False

    async def get_title(self) -> str:
        """获取页面标题"""
        if not self._page:
            return ""
        return await self._page.title()

    async def get_url(self) -> str:
        """获取当前 URL"""
        if not self._page:
            return ""
        return self._page.url

    # ==================== DOM 操作 ====================

    async def click(self, selector: str, timeout: float = 5.0) -> bool:
        """
        点击元素

        Args:
            selector: CSS 选择器
            timeout: 超时时间

        Returns:
            是否点击成功
        """
        if not self._page:
            return False

        try:
            await self._page.click(selector, timeout=timeout * 1000)
            return True
        except Exception as e:
            logger.error(f"点击元素 {selector} 失败: {e}")
            return False

    async def fill(self, selector: str, text: str) -> bool:
        """
        填充输入框

        Args:
            selector: CSS 选择器
            text: 要输入的文本

        Returns:
            是否填充成功
        """
        if not self._page:
            return False

        try:
            await self._page.fill(selector, text)
            return True
        except Exception as e:
            logger.error(f"填充 {selector} 失败: {e}")
            return False

    async def clear(self, selector: str) -> bool:
        """清空输入框"""
        if not self._page:
            return False

        try:
            await self._page.fill(selector, "")
            return True
        except Exception as e:
            logger.error(f"清空 {selector} 失败: {e}")
            return False

    async def get_text(self, selector: str) -> str:
        """
        获取元素文本内容

        Args:
            selector: CSS 选择器

        Returns:
            文本内容
        """
        if not self._page:
            return ""

        try:
            return await self._page.inner_text(selector)
        except Exception as e:
            logger.error(f"获取文本失败: {e}")
            return ""

    async def get_attribute(self, selector: str, attribute: str) -> Optional[str]:
        """
        获取元素属性

        Args:
            selector: CSS 选择器
            attribute: 属性名

        Returns:
            属性值
        """
        if not self._page:
            return None

        try:
            return await self._page.get_attribute(selector, attribute)
        except Exception as e:
            logger.error(f"获取属性失败: {e}")
            return None

    async def is_visible(self, selector: str) -> bool:
        """检查元素是否可见"""
        if not self._page:
            return False

        try:
            return await self._page.is_visible(selector)
        except Exception:
            return False

    async def is_enabled(self, selector: str) -> bool:
        """检查元素是否可用"""
        if not self._page:
            return False

        try:
            return await self._page.is_enabled(selector)
        except Exception:
            return False

    async def wait_for_selector(
        self,
        selector: str,
        state: str = "visible",
        timeout: float = 10.0
    ) -> bool:
        """
        等待元素出现

        Args:
            selector: CSS 选择器
            state: 等待状态 (visible, hidden, attached, detached)
            timeout: 超时时间

        Returns:
            是否等到元素
        """
        if not self._page:
            return False

        try:
            await self._page.wait_for_selector(
                selector,
                state=state,
                timeout=timeout * 1000
            )
            return True
        except Exception:
            return False

    async def query_selector_all(self, selector: str) -> List[Dict[str, Any]]:
        """
        查询所有匹配的元素

        Args:
            selector: CSS 选择器

        Returns:
            元素信息列表
        """
        if not self._page:
            return []

        try:
            elements = await self._page.query_selector_all(selector)
            results = []
            for elem in elements:
                info = await elem.evaluate("""el => ({
                    tagName: el.tagName,
                    textContent: el.textContent,
                    className: el.className,
                    id: el.id,
                    disabled: el.disabled,
                    checked: el.checked,
                    value: el.value
                })""")
                results.append(info)
            return results
        except Exception as e:
            logger.error(f"查询元素失败: {e}")
            return []

    # ==================== 键盘操作 ====================

    async def press(self, key: str) -> bool:
        """
        按下键盘按键

        Args:
            key: 按键名 (Enter, Tab, Escape, ArrowUp 等)

        Returns:
            是否成功
        """
        if not self._page:
            return False

        try:
            await self._page.keyboard.press(key)
            return True
        except Exception as e:
            logger.error(f"按键 {key} 失败: {e}")
            return False

    async def type_text(self, text: str, delay: float = 0.0) -> bool:
        """
        输入文本

        Args:
            text: 要输入的文本
            delay: 每个字符之间的延迟（秒）

        Returns:
            是否成功
        """
        if not self._page:
            return False

        try:
            await self._page.keyboard.type(text, delay=delay * 1000)
            return True
        except Exception as e:
            logger.error(f"输入文本失败: {e}")
            return False

    # ==================== 截图 ====================

    async def screenshot(
        self,
        selector: Optional[str] = None,
        full_page: bool = False
    ) -> Optional[bytes]:
        """
        截图

        Args:
            selector: 元素选择器（可选，只截取该元素）
            full_page: 是否截取整个页面

        Returns:
            PNG 图像数据
        """
        if not self._page:
            return None

        try:
            if selector:
                element = await self._page.query_selector(selector)
                if element:
                    return await element.screenshot()
                return None
            else:
                return await self._page.screenshot(full_page=full_page)
        except Exception as e:
            logger.error(f"截图失败: {e}")
            return None

    # ==================== 性能监控 ====================

    async def get_performance_metrics(self) -> WebViewMetrics:
        """
        获取性能指标

        Returns:
            WebViewMetrics 对象
        """
        if not self._page:
            return WebViewMetrics()

        try:
            # 获取导航计时
            timing = await self._page.evaluate("""() => {
                const t = performance.timing;
                return {
                    loadTime: t.loadEventEnd - t.navigationStart,
                    domContentLoaded: t.domContentLoadedEventEnd - t.navigationStart,
                    firstPaint: 0,
                    firstContentfulPaint: 0
                };
            }""")

            # 获取 Paint 计时
            paint_entries = await self._page.evaluate("""() => {
                return performance.getEntriesByType('paint').map(e => ({
                    name: e.name,
                    startTime: e.startTime
                }));
            }""")

            first_paint = 0
            first_contentful = 0
            for entry in paint_entries:
                if entry['name'] == 'first-paint':
                    first_paint = entry['startTime']
                elif entry['name'] == 'first-contentful-paint':
                    first_contentful = entry['startTime']

            # 获取内存使用
            memory = await self._page.evaluate("""() => {
                return performance.memory ? {
                    usedJSHeapSize: performance.memory.usedJSHeapSize,
                    totalJSHeapSize: performance.memory.totalJSHeapSize
                } : { usedJSHeapSize: 0, totalJSHeapSize: 0 };
            }""")

            return WebViewMetrics(
                load_time_ms=timing.get('loadTime', 0),
                dom_content_loaded_ms=timing.get('domContentLoaded', 0),
                first_paint_ms=first_paint,
                first_contentful_paint_ms=first_contentful,
                js_heap_used_size=memory.get('usedJSHeapSize', 0),
                js_heap_total_size=memory.get('totalJSHeapSize', 0)
            )

        except Exception as e:
            logger.error(f"获取性能指标失败: {e}")
            return WebViewMetrics()

    # ==================== 事件监听 ====================

    def add_console_listener(self, listener: Callable[[ConsoleMessage], None]):
        """添加控制台消息监听器"""
        self._console_listeners.append(listener)

    def remove_console_listener(self, listener: Callable[[ConsoleMessage], None]):
        """移除控制台消息监听器"""
        if listener in self._console_listeners:
            self._console_listeners.remove(listener)

    def add_error_listener(self, listener: Callable[[str], None]):
        """添加错误监听器"""
        self._error_listeners.append(listener)

    def get_console_logs(self, log_type: Optional[str] = None) -> List[ConsoleMessage]:
        """
        获取控制台日志

        Args:
            log_type: 日志类型过滤 (log, warn, error, info)

        Returns:
            日志列表
        """
        if log_type:
            return [log for log in self._console_logs if log.type == log_type]
        return self._console_logs.copy()

    def clear_console_logs(self):
        """清空控制台日志"""
        self._console_logs.clear()

    # ==================== JavaScript 执行 ====================

    async def evaluate(self, script: str) -> Any:
        """
        执行 JavaScript

        Args:
            script: JavaScript 代码

        Returns:
            执行结果
        """
        if not self._page:
            return None

        try:
            return await self._page.evaluate(script)
        except Exception as e:
            logger.error(f"执行脚本失败: {e}")
            return None

    async def expose_function(self, name: str, func: Callable):
        """
        暴露���数到页面

        Args:
            name: 函数名
            func: Python 函数
        """
        if not self._page:
            return

        try:
            await self._page.expose_function(name, func)
        except Exception as e:
            logger.error(f"暴露函数失败: {e}")

    # ==================== 上下文管理 ====================

    @asynccontextmanager
    async def session(self, app_name: str):
        """
        异步上下文管理器

        使用示例:
            async with adapter.session("FlowSight") as page:
                await page.click("button")
        """
        try:
            await self.connect_to_tauri_app(app_name)
            yield self
        finally:
            await self.disconnect()
