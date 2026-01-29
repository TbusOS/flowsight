"""
WebView 专项测试模块
===================

针对 Tauri WebView 的专项测试，包括:
- DOM 结构和语义化
- CSS 样式和响应式
- JavaScript 功能
- 性能指标
- 可访问性 (a11y)
- 控制台错误
"""

import asyncio
from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional, Callable
import logging
import time

from .webview_adapter import WebViewAdapter, WebViewMetrics

try:
    from playwright.async_api import Page
    PLAYWRIGHT_AVAILABLE = True
except ImportError:
    PLAYWRIGHT_AVAILABLE = False
    Page = Any

logger = logging.getLogger(__name__)


@dataclass
class DOMIssue:
    """DOM 问题"""
    issue_type: str  # missing_alt, invalid_nesting, duplicate_id
    severity: str    # critical, high, medium, low
    element: str
    description: str
    suggestion: str


@dataclass
class CSSIssue:
    """CSS 问题"""
    issue_type: str  # unused_css, inline_style, invalid_property
    severity: str
    selector: str
    description: str
    suggestion: str


@dataclass
class JSIssue:
    """JavaScript 问题"""
    issue_type: str  # console_error, unhandled_rejection, memory_leak
    severity: str
    message: str
    location: Optional[str] = None


@dataclass
class PerformanceIssue:
    """性能问题"""
    metric: str
    value: float
    threshold: float
    severity: str
    suggestion: str


@dataclass
class A11yIssue:
    """可访问性问题"""
    rule: str  # color_contrast, missing_label, no_focus_indicator
    severity: str
    element: str
    description: str
    wcag_reference: str


@dataclass
class WebViewTestReport:
    """WebView 测试报告"""
    # 测试统计
    total_tests: int = 0
    passed_tests: int = 0
    failed_tests: int = 0
    warnings: int = 0

    # DOM 测试
    dom_issues: List[DOMIssue] = field(default_factory=list)
    semantic_score: float = 0.0

    # CSS 测试
    css_issues: List[CSSIssue] = field(default_factory=list)
    responsive_score: float = 0.0

    # JavaScript 测试
    js_issues: List[JSIssue] = field(default_factory=list)
    console_errors: int = 0
    console_warnings: int = 0

    # 性能测试
    performance_issues: List[PerformanceIssue] = field(default_factory=list)
    performance_score: float = 0.0
    metrics: Optional[WebViewMetrics] = None

    # 可访问性测试
    a11y_issues: List[A11yIssue] = field(default_factory=list)
    a11y_score: float = 0.0

    # 总体评分
    overall_score: float = 0.0

    def to_dict(self) -> Dict[str, Any]:
        """转换为字典"""
        return {
            'total_tests': self.total_tests,
            'passed_tests': self.passed_tests,
            'failed_tests': self.failed_tests,
            'warnings': self.warnings,
            'semantic_score': round(self.semantic_score, 2),
            'responsive_score': round(self.responsive_score, 2),
            'console_errors': self.console_errors,
            'console_warnings': self.console_warnings,
            'performance_score': round(self.performance_score, 2),
            'a11y_score': round(self.a11y_score, 2),
            'overall_score': round(self.overall_score, 2),
            'dom_issues_count': len(self.dom_issues),
            'css_issues_count': len(self.css_issues),
            'js_issues_count': len(self.js_issues),
            'performance_issues_count': len(self.performance_issues),
            'a11y_issues_count': len(self.a11y_issues),
        }


class WebViewTests:
    """
    WebView 专项测试

    提供针对 WebView 内容的全面测试，包括结构、样式、功能和可访问性。
    """

    def __init__(self, adapter: WebViewAdapter):
        self.adapter = adapter
        self._custom_tests: List[Callable] = []

    async def test_all(self) -> WebViewTestReport:
        """运行所有 WebView 测试"""
        logger.info("开始 WebView 全面测试...")
        report = WebViewTestReport()

        # 1. DOM 结构测试
        dom_report = await self.test_dom_structure()
        report.dom_issues = dom_report.dom_issues
        report.semantic_score = dom_report.semantic_score
        report.total_tests += dom_report.total_tests
        report.passed_tests += dom_report.passed_tests
        report.failed_tests += dom_report.failed_tests

        # 2. CSS 测试
        css_report = await self.test_css_compliance()
        report.css_issues = css_report.css_issues
        report.responsive_score = css_report.responsive_score
        report.total_tests += css_report.total_tests
        report.passed_tests += css_report.passed_tests
        report.failed_tests += css_report.failed_tests

        # 3. JavaScript 测试
        js_report = await self.test_javascript_functionality()
        report.js_issues = js_report.js_issues
        report.console_errors = js_report.console_errors
        report.console_warnings = js_report.console_warnings
        report.total_tests += js_report.total_tests
        report.passed_tests += js_report.passed_tests
        report.failed_tests += js_report.failed_tests

        # 4. 性能测试
        perf_report = await self.test_performance()
        report.performance_issues = perf_report.performance_issues
        report.performance_score = perf_report.performance_score
        report.metrics = perf_report.metrics
        report.total_tests += perf_report.total_tests
        report.passed_tests += perf_report.passed_tests
        report.failed_tests += perf_report.failed_tests

        # 5. 可访问性测试
        a11y_report = await self.test_accessibility()
        report.a11y_issues = a11y_report.a11y_issues
        report.a11y_score = a11y_report.a11y_score
        report.total_tests += a11y_report.total_tests
        report.passed_tests += a11y_report.passed_tests
        report.failed_tests += a11y_report.failed_tests

        # 6. 运行自定义测试
        for test_func in self._custom_tests:
            try:
                await test_func(self.adapter, report)
            except Exception as e:
                logger.warning(f"自定义测试失败: {e}")

        # 计算总体评分
        report.overall_score = (
            report.semantic_score * 0.2 +
            report.responsive_score * 0.2 +
            max(0, 100 - report.console_errors * 10 - report.console_warnings * 2) * 0.2 +
            report.performance_score * 0.2 +
            report.a11y_score * 0.2
        )

        report.warnings = (
            len(report.dom_issues) +
            len(report.css_issues) +
            len(report.js_issues) +
            len(report.performance_issues) +
            len(report.a11y_issues)
        )

        logger.info(f"WebView 测试完成: 总体评分 {report.overall_score:.1f}/100")
        return report

    # ==================== DOM 结构测试 ====================

    async def test_dom_structure(self) -> WebViewTestReport:
        """测试 DOM 结构和语义化"""
        logger.info("测试 DOM 结构...")
        report = WebViewTestReport()

        if not self.adapter.is_connected:
            logger.error("WebView 未连接")
            return report

        issues = []
        passed = 0
        failed = 0

        # 1. 检查图片是否有 alt 属性
        images = await self.adapter.query_selector_all("img")
        for img in images:
            if not img.get('alt') and not img.get('aria-label'):
                issues.append(DOMIssue(
                    issue_type="missing_alt",
                    severity="high",
                    element=f"img (src={img.get('textContent', '')[:50]}...)",
                    description="图片缺少 alt 属性",
                    suggestion="为图片添加描述性的 alt 属性"
                ))
                failed += 1
            else:
                passed += 1

        # 2. 检查重复的 ID
        all_ids = await self.adapter.evaluate("""
            () => {
                const elements = document.querySelectorAll('[id]');
                const ids = {};
                elements.forEach(el => {
                    if (ids[el.id]) {
                        ids[el.id].count++;
                    } else {
                        ids[el.id] = { count: 1, tagName: el.tagName };
                    }
                });
                return Object.entries(ids)
                    .filter(([id, info]) => info.count > 1)
                    .map(([id, info]) => ({ id, count: info.count, tagName: info.tagName }));
            }
        """)

        if all_ids:
            for dup in all_ids:
                issues.append(DOMIssue(
                    issue_type="duplicate_id",
                    severity="critical",
                    element=f"#{dup['id']} ({dup['tagName']})",
                    description=f"ID '{dup['id']}' 被使用��� {dup['count']} 次",
                    suggestion="确保每个 ID 在页面中唯一"
                ))
                failed += 1

        # 3. 检查表单标签
        inputs = await self.adapter.query_selector_all("input, select, textarea")
        for inp in inputs:
            input_id = inp.get('id', '')
            input_name = inp.get('textContent', '')

            # 检查是否有关联的 label
            has_label = False
            if input_id:
                has_label = await self.adapter.is_visible(f"label[for='{input_id}']")

            # 检查 aria-label 或 aria-labelledby
            has_aria = inp.get('aria-label') or inp.get('aria-labelledby')

            # 检查 placeholder（不推荐作为主要标签）
            has_placeholder = inp.get('placeholder')

            if not has_label and not has_aria and not has_placeholder:
                issues.append(DOMIssue(
                    issue_type="missing_label",
                    severity="medium",
                    element=f"input (name={input_name})",
                    description="表单控件缺少标签",
                    suggestion="为表单控件添加 label 或 aria-label"
                ))
                failed += 1
            else:
                passed += 1

        # 4. 检查语义化标签使用
        semantic_tags = ['header', 'nav', 'main', 'article', 'section', 'aside', 'footer']
        semantic_count = 0
        for tag in semantic_tags:
            elements = await self.adapter.query_selector_all(tag)
            semantic_count += len(elements)

        div_count = len(await self.adapter.query_selector_all("div"))

        # 语义化评分
        if div_count > 0:
            semantic_ratio = semantic_count / (semantic_count + div_count)
        else:
            semantic_ratio = 1.0

        if semantic_ratio < 0.1:
            issues.append(DOMIssue(
                issue_type="poor_semantics",
                severity="medium",
                element="document",
                description="页面语义化程度低，过度使用 div",
                suggestion="使用 header, nav, main, section 等语义化标签"
            ))

        # 5. 检查标题层级
        h1_count = len(await self.adapter.query_selector_all("h1"))
        if h1_count == 0:
            issues.append(DOMIssue(
                issue_type="missing_h1",
                severity="high",
                element="document",
                description="页面缺少 h1 标题",
                suggestion="添加一个 h1 标题作为页面主标题"
            ))
            failed += 1
        elif h1_count > 1:
            issues.append(DOMIssue(
                issue_type="multiple_h1",
                severity="medium",
                element="document",
                description=f"页面有 {h1_count} 个 h1 标题",
                suggestion="一个页面应该只有一个 h1 标题"
            ))

        # 计算评分
        total_checks = passed + failed + len(issues)
        report.dom_issues = issues
        report.semantic_score = max(0, 100 - len(issues) * 5 - failed * 10)
        report.total_tests = total_checks
        report.passed_tests = passed
        report.failed_tests = failed

        logger.info(f"DOM 测试完成: {report.semantic_score:.1f}/100, {len(issues)} 个问题")
        return report

    # ==================== CSS 测试 ====================

    async def test_css_compliance(self) -> WebViewTestReport:
        """测试 CSS 合规性"""
        logger.info("测试 CSS 合规性...")
        report = WebViewTestReport()

        if not self.adapter.is_connected:
            return report

        issues = []
        passed = 0
        failed = 0

        # 1. 检查响应式视口设置
        viewport_meta = await self.adapter.query_selector_all("meta[name='viewport']")
        if not viewport_meta:
            issues.append(CSSIssue(
                issue_type="missing_viewport",
                severity="critical",
                selector="head",
                description="缺少 viewport meta 标签",
                suggestion="添加 <meta name='viewport' content='width=device-width, initial-scale=1'>"
            ))
            failed += 1
        else:
            passed += 1

        # 2. 检查行内样式
        inline_styles = await self.adapter.query_selector_all("[style]")
        if len(inline_styles) > 5:
            issues.append(CSSIssue(
                issue_type="inline_style",
                severity="low",
                selector=f"{len(inline_styles)} elements",
                description=f"发现 {len(inline_styles)} 个行内��式",
                suggestion="将行内样式迁移到 CSS 类"
            ))

        # 3. 测试响应式断点
        breakpoints = [375, 768, 1024, 1440]  # 移动端、平板、桌面、大屏
        responsive_results = []

        for width in breakpoints:
            await self.adapter.page.set_viewport_size({"width": width, "height": 800})
            await asyncio.sleep(0.5)  # 等待响应式调整

            # 检查水平滚动
            has_overflow = await self.adapter.evaluate("""
                () => document.documentElement.scrollWidth > window.innerWidth
            """)
            responsive_results.append((width, not has_overflow))

            if has_overflow:
                issues.append(CSSIssue(
                    issue_type="horizontal_scroll",
                    severity="high",
                    selector=f"viewport@{width}px",
                    description=f"在 {width}px 宽度下出现水平滚动",
                    suggestion="检查固定宽度元素和溢出处理"
                ))
                failed += 1
            else:
                passed += 1

        # 恢复原始大小
        await self.adapter.page.set_viewport_size({"width": 1280, "height": 720})

        # 计算响应式评分
        responsive_ratio = sum(1 for _, ok in responsive_results if ok) / len(responsive_results)
        report.responsive_score = responsive_ratio * 100

        report.css_issues = issues
        report.total_tests = passed + failed
        report.passed_tests = passed
        report.failed_tests = failed

        logger.info(f"CSS 测试完成: {report.responsive_score:.1f}/100")
        return report

    # ==================== JavaScript 测试 ====================

    async def test_javascript_functionality(self) -> WebViewTestReport:
        """测试 JavaScript 功能"""
        logger.info("测试 JavaScript 功能...")
        report = WebViewTestReport()

        if not self.adapter.is_connected:
            return report

        # 清空之前的日志
        self.adapter.clear_console_logs()

        # 等待一段时间收集日志
        await asyncio.sleep(2)

        # 获取控制台日志
        logs = self.adapter.get_console_logs()
        errors = [log for log in logs if log.type == 'error']
        warnings = [log for log in logs if log.type == 'warning']

        issues = []

        # 记录 JavaScript 错误
        for error in errors:
            issues.append(JSIssue(
                issue_type="console_error",
                severity="high",
                message=error.text,
                location=str(error.location)
            ))

        # 记录警告
        for warning in warnings[:10]:  # 限制警告数量
            issues.append(JSIssue(
                issue_type="console_warning",
                severity="low",
                message=warning.text,
                location=str(warning.location)
            ))

        # 检查未处理的 Promise 拒绝
        unhandled = await self.adapter.evaluate("""
            () => {
                if (window.__unhandledRejections) {
                    return window.__unhandledRejections;
                }
                return [];
            }
        """)

        if unhandled:
            for rejection in unhandled:
                issues.append(JSIssue(
                    issue_type="unhandled_rejection",
                    severity="critical",
                    message=rejection
                ))

        report.js_issues = issues
        report.console_errors = len(errors)
        report.console_warnings = len(warnings)
        report.total_tests = 1
        report.passed_tests = 1 if len(errors) == 0 else 0
        report.failed_tests = len(errors)

        logger.info(f"JS 测试完成: {len(errors)} 错误, {len(warnings)} 警告")
        return report

    # ==================== 性能测试 ====================

    async def test_performance(self) -> WebViewTestReport:
        """测试性能指标"""
        logger.info("测试性能...")
        report = WebViewTestReport()

        if not self.adapter.is_connected:
            return report

        # 获取性能指标
        metrics = await self.adapter.get_performance_metrics()
        report.metrics = metrics

        issues = []
        passed = 0
        failed = 0

        # 性能阈值检查
        thresholds = {
            'load_time_ms': (3000, '页面加载时间'),
            'dom_content_loaded_ms': (1500, 'DOM 加载时间'),
            'first_paint_ms': (1000, '首次绘制'),
            'first_contentful_paint_ms': (1500, '首次内容绘制'),
        }

        for metric_name, (threshold, description) in thresholds.items():
            value = getattr(metrics, metric_name, 0)
            if value > threshold:
                issues.append(PerformanceIssue(
                    metric=metric_name,
                    value=value,
                    threshold=threshold,
                    severity="high" if value > threshold * 2 else "medium",
                    suggestion=f"优化 {description}，目标 < {threshold}ms"
                ))
                failed += 1
            else:
                passed += 1

        # 检查内存使用
        if metrics.js_heap_used_size > 100 * 1024 * 1024:  # 100MB
            issues.append(PerformanceIssue(
                metric="js_heap_used_size",
                value=metrics.js_heap_used_size / 1024 / 1024,
                threshold=100,
                severity="medium",
                suggestion="JavaScript 内存使用过高，检查内存泄漏"
            ))

        # 计算性能评分
        if issues:
            report.performance_score = max(0, 100 - len(issues) * 15)
        else:
            report.performance_score = 100

        report.performance_issues = issues
        report.total_tests = passed + failed
        report.passed_tests = passed
        report.failed_tests = failed

        logger.info(f"性能测试完成: {report.performance_score:.1f}/100")
        return report

    # ==================== 可访问性测试 ====================

    async def test_accessibility(self) -> WebViewTestReport:
        """测试可访问性"""
        logger.info("测试可访问性...")
        report = WebViewTestReport()

        if not self.adapter.is_connected:
            return report

        issues = []
        passed = 0
        failed = 0

        # 1. 检查颜色对比度（简化检查）
        low_contrast_elements = await self.adapter.evaluate("""
            () => {
                const elements = document.querySelectorAll('p, span, a, button, h1, h2, h3, h4, h5, h6');
                const issues = [];
                elements.forEach(el => {
                    const style = window.getComputedStyle(el);
                    const color = style.color;
                    const bgColor = style.backgroundColor;
                    // 简化的对比度检查（实际应使用更精确的算法）
                    if (color.includes('200') && bgColor.includes('255')) {
                        issues.push(el.tagName + (el.className ? '.' + el.className : ''));
                    }
                });
                return issues.slice(0, 10); // 限制数量
            }
        """)

        if low_contrast_elements:
            issues.append(A11yIssue(
                rule="color_contrast",
                severity="high",
                element=str(low_contrast_elements[:3]),
                description="部分文本颜色对比度可能不足",
                wcag_reference="WCAG 2.1 AA 1.4.3"
            ))
            failed += 1
        else:
            passed += 1

        # 2. 检查焦点指示器
        focusable = await self.adapter.query_selector_all(
            "a, button, input, select, textarea, [tabindex]:not([tabindex='-1'])"
        )

        # 3. 检查 ARIA 属性
        invalid_aria = await self.adapter.evaluate("""
            () => {
                const validRoles = ['alert', 'alertdialog', 'application', 'article', 'banner',
                    'button', 'cell', 'checkbox', 'columnheader', 'combobox', 'complementary',
                    'contentinfo', 'definition', 'dialog', 'directory', 'document', 'feed',
                    'figure', 'form', 'grid', 'gridcell', 'group', 'heading', 'img', 'link',
                    'list', 'listbox', 'listitem', 'log', 'main', 'marquee', 'math', 'menu',
                    'menubar', 'menuitem', 'menuitemcheckbox', 'menuitemradio', 'navigation',
                    'none', 'note', 'option', 'presentation', 'progressbar', 'radio',
                    'radiogroup', 'region', 'row', 'rowgroup', 'rowheader', 'scrollbar',
                    'search', 'searchbox', 'separator', 'slider', 'spinbutton', 'status',
                    'switch', 'tab', 'table', 'tablist', 'tabpanel', 'term', 'textbox',
                    'timer', 'toolbar', 'tooltip', 'tree', 'treegrid', 'treeitem'];

                const elements = document.querySelectorAll('[role]');
                const invalid = [];
                elements.forEach(el => {
                    const role = el.getAttribute('role');
                    if (!validRoles.includes(role)) {
                        invalid.push(el.tagName + '[role="' + role + '"]');
                    }
                });
                return invalid.slice(0, 5);
            }
        """)

        if invalid_aria:
            issues.append(A11yIssue(
                rule="invalid_aria_role",
                severity="medium",
                element=str(invalid_aria),
                description="使用了无效的 ARIA role 属性",
                wcag_reference="WCAG 2.1 ARIA"
            ))
            failed += 1
        else:
            passed += 1

        # 4. 检查跳转到主内容链接
        skip_link = await self.adapter.query_selector_all("a[href^='#']")
        has_skip_link = any(
            'skip' in link.get('textContent', '').lower()
            for link in skip_link
        )

        if not has_skip_link:
            issues.append(A11yIssue(
                rule="missing_skip_link",
                severity="low",
                element="body",
                description="缺少跳转到主内容的链接",
                wcag_reference="WCAG 2.1 AAA 2.4.1"
            ))

        # 计算可访问性评分
        report.a11y_score = max(0, 100 - len(issues) * 10)
        report.a11y_issues = issues
        report.total_tests = passed + failed
        report.passed_tests = passed
        report.failed_tests = failed

        logger.info(f"可访问性测试完成: {report.a11y_score:.1f}/100")
        return report

    # ==================== 自定义测试 ====================

    def register_custom_test(self, test_func: Callable):
        """
        注册自定义测试函数

        Args:
            test_func: 测试函数，接收 (adapter, report) 参数
        """
        self._custom_tests.append(test_func)

    async def test_component(self, selector: str, interactions: List[Dict]) -> Dict[str, Any]:
        """
        测试特定组件

        Args:
            selector: 组件选择器
            interactions: 交互列表 [{"action": "click", "target": "..."}]

        Returns:
            测试结果
        """
        if not self.adapter.is_connected:
            return {"success": False, "error": "未连接"}

        results = {
            "selector": selector,
            "exists": False,
            "visible": False,
            "interactions": []
        }

        # 检查元素存在
        exists = await self.adapter.page.query_selector(selector) is not None
        results["exists"] = exists

        if not exists:
            return results

        # 检查可见性
        visible = await self.adapter.is_visible(selector)
        results["visible"] = visible

        # 执行交互
        for interaction in interactions:
            action = interaction.get("action")
            target = interaction.get("target", selector)
            start_time = time.time()

            try:
                if action == "click":
                    await self.adapter.click(target)
                elif action == "fill":
                    await self.adapter.fill(target, interaction.get("text", ""))
                elif action == "hover":
                    await self.adapter.page.hover(target)
                elif action == "focus":
                    await self.adapter.page.focus(target)

                results["interactions"].append({
                    "action": action,
                    "success": True,
                    "duration_ms": (time.time() - start_time) * 1000
                })
            except Exception as e:
                results["interactions"].append({
                    "action": action,
                    "success": False,
                    "error": str(e),
                    "duration_ms": (time.time() - start_time) * 1000
                })

        return results
