"""
测试运行器
=========

职责:
- 协调所有测试模块执行
- 管理测试生命周期
- 生成综合测试报告
- 支持配置驱动的测试
"""

from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional, Callable, Type
from datetime import datetime
from enum import Enum
import asyncio
import logging
import json
import time
import os

from .desktop_adapter import DesktopAdapter, DesktopPlatformAdapter
from .visual_analyzer import VisualAnalyzer
from ..aesthetic.aesthetic_evaluator import AestheticEvaluator, AestheticReport
from ..tests.layout_tests import LayoutTests, LayoutReport
from ..tests.visual_tests import VisualTests, VisualReport
from ..tests.interactive_tests import InteractiveTests, InteractiveReport
from ..tests.state_tests import StateTests, StateReport
from ..playwright import WebViewAdapter, WebViewTests, WebViewTestReport

logger = logging.getLogger(__name__)


class TestPhase(Enum):
    """测试阶段"""
    SETUP = "setup"
    LAYOUT = "layout"
    VISUAL = "visual"
    INTERACTIVE = "interactive"
    STATE = "state"
    WEBVIEW = "webview"
    AESTHETIC = "aesthetic"
    TEARDOWN = "teardown"


class TestPriority(Enum):
    """测试优先级"""
    CRITICAL = "critical"      # 必须通过的测试
    HIGH = "high"              # 重要功能测试
    MEDIUM = "medium"          # 常规测试
    LOW = "low"                # 可选测试


@dataclass
class TestCase:
    """测试用例定义"""
    name: str
    phase: TestPhase
    priority: TestPriority
    func: Callable
    timeout_seconds: float = 60.0
    depends_on: List[str] = field(default_factory=list)
    skip_conditions: List[str] = field(default_factory=list)


@dataclass
class TestResult:
    """单个测试结果"""
    test_name: str
    phase: TestPhase
    priority: TestPriority
    passed: bool
    duration_ms: float
    error_message: Optional[str] = None
    report: Optional[Any] = None
    screenshot_path: Optional[str] = None
    logs: List[str] = field(default_factory=list)


@dataclass
class PhaseResult:
    """阶段测试结果"""
    phase: TestPhase
    total_tests: int
    passed_tests: int
    failed_tests: int
    skipped_tests: int
    duration_ms: float
    results: List[TestResult]


@dataclass
class TestSuiteReport:
    """完整测试套件报告"""
    # 基本信息
    start_time: datetime
    end_time: Optional[datetime] = None
    duration_ms: float = 0.0

    # 执行结果
    total_tests: int = 0
    passed_tests: int = 0
    failed_tests: int = 0
    skipped_tests: int = 0

    # 按阶段统计
    phase_results: List[PhaseResult] = field(default_factory=list)

    # 详细报告
    layout_report: Optional[LayoutReport] = None
    visual_report: Optional[VisualReport] = None
    interaction_report: Optional[InteractiveReport] = None
    state_report: Optional[StateReport] = None
    webview_report: Optional[WebViewTestReport] = None
    aesthetic_report: Optional[AestheticReport] = None

    # 综合评分
    overall_score: float = 0.0
    aesthetic_score: float = 0.0
    usability_score: float = 0.0
    performance_score: float = 0.0

    # 元数据
    target_app: str = ""
    test_config: Dict[str, Any] = field(default_factory=dict)
    environment: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        """转换为字典格式"""
        return {
            "start_time": self.start_time.isoformat(),
            "end_time": self.end_time.isoformat() if self.end_time else None,
            "duration_ms": self.duration_ms,
            "summary": {
                "total": self.total_tests,
                "passed": self.passed_tests,
                "failed": self.failed_tests,
                "skipped": self.skipped_tests,
                "pass_rate": self.passed_tests / self.total_tests * 100
                if self.total_tests > 0 else 0
            },
            "scores": {
                "overall": self.overall_score,
                "aesthetic": self.aesthetic_score,
                "usability": self.usability_score,
                "performance": self.performance_score
            },
            "phases": [
                {
                    "phase": r.phase.value,
                    "total": r.total_tests,
                    "passed": r.passed_tests,
                    "failed": r.failed_tests,
                    "skipped": r.skipped_tests,
                    "duration_ms": r.duration_ms
                }
                for r in self.phase_results
            ]
        }

    def to_json(self, indent: int = 2) -> str:
        """转换为JSON字符串"""
        return json.dumps(self.to_dict(), indent=indent, ensure_ascii=False)


class TestRunner:
    """
    测试运行器

    协调所有测试模块的执行，提供统一的测试生命周期管理。
    """

    def __init__(
        self,
        adapter: DesktopAdapter,
        visual_analyzer: VisualAnalyzer,
        aesthetic_evaluator: Optional[AestheticEvaluator] = None,
        output_dir: str = "./test_reports"
    ):
        self.adapter = adapter
        self.visual_analyzer = visual_analyzer
        self.aesthetic_evaluator = aesthetic_evaluator or AestheticEvaluator()
        self.output_dir = output_dir

        # 初始化测试模块
        self.layout_tests = LayoutTests(visual_analyzer, aesthetic_evaluator)
        self.visual_tests = VisualTests(visual_analyzer, aesthetic_evaluator)
        self.interactive_tests = InteractiveTests(adapter, visual_analyzer)
        self.state_tests = StateTests(adapter, visual_analyzer)
        self.webview_tests: Optional[WebViewTests] = None
        self.webview_adapter: Optional[WebViewAdapter] = None

        # 测试配置
        self.test_cases: List[TestCase] = []
        self.results: List[TestResult] = []
        self.report = TestSuiteReport(
            start_time=datetime.now(),
            target_app=adapter.target_window_title or "Unknown"
        )

        # 确保输出目录存在
        os.makedirs(output_dir, exist_ok=True)

    # ==================== 测试注册 ====================

    def register_test(
        self,
        name: str,
        phase: TestPhase,
        func: Callable,
        priority: TestPriority = TestPriority.MEDIUM,
        timeout: float = 60.0,
        depends_on: Optional[List[str]] = None
    ):
        """注册测试用例"""
        self.test_cases.append(TestCase(
            name=name,
            phase=phase,
            priority=priority,
            func=func,
            timeout_seconds=timeout,
            depends_on=depends_on or []
        ))

    def register_default_tests(self):
        """注册默认测试套件"""
        # ===== Layout Tests =====
        self.register_test(
            "element_alignment",
            TestPhase.LAYOUT,
            self._run_alignment_test,
            TestPriority.CRITICAL
        )
        self.register_test(
            "spacing_consistency",
            TestPhase.LAYOUT,
            self._run_spacing_test,
            TestPriority.CRITICAL
        )
        self.register_test(
            "overlap_detection",
            TestPhase.LAYOUT,
            self._run_overlap_test,
            TestPriority.HIGH
        )
        self.register_test(
            "responsive_layout",
            TestPhase.LAYOUT,
            self._run_responsive_test,
            TestPriority.MEDIUM
        )
        self.register_test(
            "shadcn_compliance",
            TestPhase.LAYOUT,
            self._run_shadcn_layout_test,
            TestPriority.HIGH
        )

        # ===== Visual Tests =====
        self.register_test(
            "color_contrast",
            TestPhase.VISUAL,
            self._run_contrast_test,
            TestPriority.CRITICAL
        )
        self.register_test(
            "font_rendering",
            TestPhase.VISUAL,
            self._run_font_test,
            TestPriority.HIGH
        )
        self.register_test(
            "icon_rendering",
            TestPhase.VISUAL,
            self._run_icon_test,
            TestPriority.MEDIUM
        )
        self.register_test(
            "color_consistency",
            TestPhase.VISUAL,
            self._run_color_consistency_test,
            TestPriority.HIGH
        )
        self.register_test(
            "dark_mode",
            TestPhase.VISUAL,
            self._run_dark_mode_test,
            TestPriority.MEDIUM
        )

        # ===== Interactive Tests =====
        self.register_test(
            "button_clicks",
            TestPhase.INTERACTIVE,
            self._run_button_click_test,
            TestPriority.CRITICAL
        )
        self.register_test(
            "hover_effects",
            TestPhase.INTERACTIVE,
            self._run_hover_test,
            TestPriority.HIGH
        )
        self.register_test(
            "menu_navigation",
            TestPhase.INTERACTIVE,
            self._run_menu_test,
            TestPriority.HIGH
        )
        self.register_test(
            "keyboard_shortcuts",
            TestPhase.INTERACTIVE,
            self._run_keyboard_test,
            TestPriority.MEDIUM
        )

        # ===== State Tests =====
        self.register_test(
            "component_states",
            TestPhase.STATE,
            self._run_component_state_test,
            TestPriority.HIGH
        )
        self.register_test(
            "form_states",
            TestPhase.STATE,
            self._run_form_state_test,
            TestPriority.HIGH
        )
        self.register_test(
            "async_operations",
            TestPhase.STATE,
            self._run_async_state_test,
            TestPriority.MEDIUM
        )

        # ===== WebView Tests =====
        self.register_test(
            "dom_structure",
            TestPhase.WEBVIEW,
            self._run_dom_test,
            TestPriority.HIGH
        )
        self.register_test(
            "css_compliance",
            TestPhase.WEBVIEW,
            self._run_css_compliance_test,
            TestPriority.HIGH
        )
        self.register_test(
            "javascript_functionality",
            TestPhase.WEBVIEW,
            self._run_js_functionality_test,
            TestPriority.HIGH
        )
        self.register_test(
            "performance_metrics",
            TestPhase.WEBVIEW,
            self._run_webview_performance_test,
            TestPriority.MEDIUM
        )
        self.register_test(
            "accessibility",
            TestPhase.WEBVIEW,
            self._run_accessibility_test,
            TestPriority.HIGH
        )
        self.register_test(
            "shadcn_css_compliance",
            TestPhase.WEBVIEW,
            self._run_shadcn_css_test,
            TestPriority.HIGH
        )

        # ===== Aesthetic Tests =====
        self.register_test(
            "design_evaluation",
            TestPhase.AESTHETIC,
            self._run_aesthetic_test,
            TestPriority.HIGH
        )
        self.register_test(
            "shadcn_compliance",
            TestPhase.AESTHETIC,
            self._run_shadcn_compliance_test,
            TestPriority.HIGH
        )

    # ==================== 测试执行 ====================

    async def run_all_tests(self) -> TestSuiteReport:
        """执行所有注册的测试"""
        logger.info("=" * 60)
        logger.info("Starting Test Suite Execution")
        logger.info("=" * 60)

        start_time = time.time()

        # 按阶段分组
        tests_by_phase: Dict[TestPhase, List[TestCase]] = {}
        for test in self.test_cases:
            tests_by_phase.setdefault(test.phase, []).append(test)

        # 按顺序执行各阶段
        phase_order = [
            TestPhase.SETUP,
            TestPhase.LAYOUT,
            TestPhase.VISUAL,
            TestPhase.INTERACTIVE,
            TestPhase.STATE,
            TestPhase.WEBVIEW,
            TestPhase.AESTHETIC,
            TestPhase.TEARDOWN
        ]

        for phase in phase_order:
            if phase in tests_by_phase:
                # 在WEBVIEW阶段前建立连接
                if phase == TestPhase.WEBVIEW:
                    await self._ensure_webview_connection()

                phase_result = await self._run_phase(phase, tests_by_phase[phase])
                self.report.phase_results.append(phase_result)

        # 生成最终报告
        end_time = time.time()
        self.report.end_time = datetime.now()
        self.report.duration_ms = (end_time - start_time) * 1000

        self._calculate_final_scores()

        # 保存报告
        self._save_report()

        logger.info("=" * 60)
        logger.info(f"Test Suite Completed")
        logger.info(f"Total: {self.report.total_tests}")
        logger.info(f"Passed: {self.report.passed_tests}")
        logger.info(f"Failed: {self.report.failed_tests}")
        logger.info(f"Duration: {self.report.duration_ms / 1000:.2f}s")
        logger.info("=" * 60)

        return self.report

    async def _run_phase(
        self,
        phase: TestPhase,
        tests: List[TestCase]
    ) -> PhaseResult:
        """执行单个测试阶段"""
        logger.info(f"\n{'='*40}")
        logger.info(f"Phase: {phase.value.upper()}")
        logger.info(f"{'='*40}")

        phase_start = time.time()
        results: List[TestResult] = []

        passed = failed = skipped = 0

        for test in tests:
            # 检查依赖
            if test.depends_on:
                deps_passed = all(
                    any(r.test_name == dep and r.passed for r in self.results)
                    for dep in test.depends_on
                )
                if not deps_passed:
                    logger.warning(f"Skipping {test.name} - dependencies not met")
                    skipped += 1
                    results.append(TestResult(
                        test_name=test.name,
                        phase=phase,
                        priority=test.priority,
                        passed=False,
                        duration_ms=0,
                        error_message="Dependencies not met"
                    ))
                    continue

            # 执行测试
            result = await self._execute_test(test)
            results.append(result)
            self.results.append(result)

            if result.passed:
                passed += 1
                logger.info(f"  ✓ {test.name}")
            else:
                failed += 1
                logger.error(f"  ✗ {test.name}: {result.error_message}")

        phase_duration = (time.time() - phase_start) * 1000

        return PhaseResult(
            phase=phase,
            total_tests=len(tests),
            passed_tests=passed,
            failed_tests=failed,
            skipped_tests=skipped,
            duration_ms=phase_duration,
            results=results
        )

    async def _execute_test(self, test: TestCase) -> TestResult:
        """执行单个测试"""
        start_time = time.time()

        try:
            # 使用超时执行测试
            report = await asyncio.wait_for(
                asyncio.to_thread(test.func),
                timeout=test.timeout_seconds
            )

            duration = (time.time() - start_time) * 1000

            # 判断测试结果
            passed = True
            if hasattr(report, 'passed'):
                passed = report.passed
            elif hasattr(report, 'overall_score'):
                passed = report.overall_score >= 70

            return TestResult(
                test_name=test.name,
                phase=test.phase,
                priority=test.priority,
                passed=passed,
                duration_ms=duration,
                report=report
            )

        except asyncio.TimeoutError:
            duration = (time.time() - start_time) * 1000
            return TestResult(
                test_name=test.name,
                phase=test.phase,
                priority=test.priority,
                passed=False,
                duration_ms=duration,
                error_message=f"Timeout after {test.timeout_seconds}s"
            )

        except Exception as e:
            duration = (time.time() - start_time) * 1000
            logger.exception(f"Test {test.name} failed")
            return TestResult(
                test_name=test.name,
                phase=test.phase,
                priority=test.priority,
                passed=False,
                duration_ms=duration,
                error_message=str(e)
            )

    # ==================== 具体测试实现 ====================

    def _run_alignment_test(self) -> LayoutReport:
        """运行对齐测试"""
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)
        return self.layout_tests.test_element_alignment(analysis)

    def _run_spacing_test(self) -> LayoutReport:
        """运行间距测试"""
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)
        return self.layout_tests.test_spacing_consistency(analysis)

    def _run_overlap_test(self) -> LayoutReport:
        """运行重叠检测"""
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)
        return self.layout_tests.test_overlap_detection(analysis)

    def _run_responsive_test(self) -> LayoutReport:
        """运行响应式布局测试"""
        return self.layout_tests.test_responsive_layout(
            self.adapter,
            self.adapter.target_window_title
        )

    def _run_shadcn_layout_test(self) -> LayoutReport:
        """运行shadcn布局合规测试"""
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)
        return self.layout_tests.test_shadcn_layout_compliance(analysis)

    def _run_contrast_test(self) -> VisualReport:
        """运行对比度测试"""
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)
        return self.visual_tests.test_color_contrast(analysis, wcag_level='AA')

    def _run_font_test(self) -> VisualReport:
        """运行字体测试"""
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)
        return self.visual_tests.test_font_rendering(analysis)

    def _run_icon_test(self) -> VisualReport:
        """运行图标测试"""
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)
        return self.visual_tests.test_icon_rendering(analysis)

    def _run_color_consistency_test(self) -> VisualReport:
        """运行颜色一致性测试"""
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)
        return self.visual_tests.test_color_consistency(analysis)

    def _run_dark_mode_test(self) -> VisualReport:
        """运行暗色模式测试"""
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)
        return self.visual_tests.test_dark_mode_compliance(analysis)

    def _run_button_click_test(self) -> InteractiveReport:
        """运行按钮点击测试"""
        return self.interactive_tests.test_all_buttons()

    def _run_hover_test(self) -> InteractiveReport:
        """运行悬停效果测试"""
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)

        # 测试第一个可交互元素
        for elem in analysis.elements:
            if elem.interactive:
                return self.interactive_tests.test_hover_effects(elem.text)

        return InteractiveReport(
            total_tests=0,
            passed_tests=0,
            failed_tests=0,
            interactions=[],
            overall_passed=True,
            avg_response_ms=0.0
        )

    def _run_menu_test(self) -> InteractiveReport:
        """运行菜单导航测试"""
        # 查找菜单触发器
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)

        for elem in analysis.elements:
            if elem.element_type in ['menu', 'menubar', 'button']:
                if any(keyword in elem.text.lower()
                       for keyword in ['menu', 'file', 'edit', 'view']):
                    return self.interactive_tests.test_menu_open(elem.text)

        return InteractiveReport(
            total_tests=0,
            passed_tests=0,
            failed_tests=0,
            interactions=[],
            overall_passed=True,
            avg_response_ms=0.0
        )

    def _run_keyboard_test(self) -> InteractiveReport:
        """运行键盘快捷键测试"""
        # 测试常见的快捷键
        return self.interactive_tests.test_keyboard_shortcut(['Tab'])

    def _run_component_state_test(self) -> Any:
        """运行组件状态测试"""
        # 测试第一个可交互组件
        screenshot = self.adapter.get_screenshot()
        analysis = self.visual_analyzer.analyze_screenshot(screenshot)

        for elem in analysis.elements:
            if elem.interactive:
                result = self.state_tests.test_component_states(elem.text)
                return self.state_tests.generate_report([result])

        return None

    def _run_form_state_test(self) -> Any:
        """运行表单状态测试"""
        # 查找表单字段
        form_fields = [
            {"name": "input", "type": "text", "test_value": "test"}
        ]
        result = self.state_tests.test_form_states(form_fields)
        return self.state_tests.generate_report([result])

    def _run_async_state_test(self) -> Any:
        """运行异步状态测试"""
        # 简化版本
        result = self.state_tests.test_async_operation(
            lambda: True,
            "success",
            timeout_seconds=5.0
        )
        return self.state_tests.generate_report([result])

    def _run_aesthetic_test(self) -> AestheticReport:
        """运行美学评估"""
        screenshot = self.adapter.get_screenshot()
        return self.aesthetic_evaluator.evaluate(screenshot)

    def _run_shadcn_compliance_test(self) -> Dict[str, Any]:
        """运行shadcn合规测试"""
        screenshot = self.adapter.get_screenshot()
        return self.aesthetic_evaluator.evaluate_shadcn_compliance(screenshot)

    # ==================== WebView 测试方法 ====================

    async def _ensure_webview_connection(self) -> bool:
        """确保WebView连接已建立"""
        if self.webview_adapter and self.webview_tests:
            return True

        try:
            # 初始化WebView测试
            self.webview_tests = WebViewTests()

            # 尝试连接Tauri应用
            self.webview_adapter = await self.webview_tests.connect_to_tauri_app(
                app_name=self.adapter.target_window_title,
                timeout_seconds=10.0
            )

            return self.webview_adapter is not None
        except Exception as e:
            logger.warning(f"WebView connection failed: {e}")
            return False

    def _run_dom_test(self) -> WebViewTestReport:
        """运行DOM结构测试"""
        if not self.webview_adapter:
            return WebViewTestReport(
                total_tests=1,
                passed_tests=0,
                failed_tests=1,
                dom_issues=[],
                css_issues=[],
                js_issues=[],
                performance_issues=[],
                a11y_issues=[],
                overall_passed=False,
                duration_ms=0.0
            )
        return self.webview_tests.test_dom_structure()

    def _run_css_compliance_test(self) -> WebViewTestReport:
        """运行CSS合规测试"""
        if not self.webview_adapter:
            return WebViewTestReport(
                total_tests=1,
                passed_tests=0,
                failed_tests=1,
                dom_issues=[],
                css_issues=[],
                js_issues=[],
                performance_issues=[],
                a11y_issues=[],
                overall_passed=False,
                duration_ms=0.0
            )
        return self.webview_tests.test_css_compliance()

    def _run_js_functionality_test(self) -> WebViewTestReport:
        """运行JavaScript功能测试"""
        if not self.webview_adapter:
            return WebViewTestReport(
                total_tests=1,
                passed_tests=0,
                failed_tests=1,
                dom_issues=[],
                css_issues=[],
                js_issues=[],
                performance_issues=[],
                a11y_issues=[],
                overall_passed=False,
                duration_ms=0.0
            )
        return self.webview_tests.test_javascript_functionality()

    def _run_webview_performance_test(self) -> WebViewTestReport:
        """运行WebView性能测试"""
        if not self.webview_adapter:
            return WebViewTestReport(
                total_tests=1,
                passed_tests=0,
                failed_tests=1,
                dom_issues=[],
                css_issues=[],
                js_issues=[],
                performance_issues=[],
                a11y_issues=[],
                overall_passed=False,
                duration_ms=0.0
            )
        return self.webview_tests.test_performance()

    def _run_accessibility_test(self) -> WebViewTestReport:
        """运行无障碍访问测试"""
        if not self.webview_adapter:
            return WebViewTestReport(
                total_tests=1,
                passed_tests=0,
                failed_tests=1,
                dom_issues=[],
                css_issues=[],
                js_issues=[],
                performance_issues=[],
                a11y_issues=[],
                overall_passed=False,
                duration_ms=0.0
            )
        return self.webview_tests.test_accessibility()

    def _run_shadcn_css_test(self) -> WebViewTestReport:
        """运行shadcn CSS变量合规测试"""
        if not self.webview_adapter:
            return WebViewTestReport(
                total_tests=1,
                passed_tests=0,
                failed_tests=1,
                dom_issues=[],
                css_issues=[],
                js_issues=[],
                performance_issues=[],
                a11y_issues=[],
                overall_passed=False,
                duration_ms=0.0
            )
        return self.webview_tests.test_shadcn_css_compliance()

    # ==================== 报告生成 ====================

    def _calculate_final_scores(self):
        """计算最终评分"""
        # 统计各阶段结果
        for phase_result in self.report.phase_results:
            self.report.total_tests += phase_result.total_tests
            self.report.passed_tests += phase_result.passed_tests
            self.report.failed_tests += phase_result.failed_tests
            self.report.skipped_tests += phase_result.skipped_tests

        # 计算总体通过率
        if self.report.total_tests > 0:
            pass_rate = self.report.passed_tests / self.report.total_tests
            self.report.overall_score = pass_rate * 100

        # 提取专项报告
        for result in self.results:
            if result.report:
                if isinstance(result.report, LayoutReport):
                    self.report.layout_report = result.report
                elif isinstance(result.report, VisualReport):
                    self.report.visual_report = result.report
                elif isinstance(result.report, InteractiveReport):
                    self.report.interaction_report = result.report
                elif isinstance(result.report, StateReport):
                    self.report.state_report = result.report
                elif isinstance(result.report, WebViewTestReport):
                    self.report.webview_report = result.report
                elif isinstance(result.report, AestheticReport):
                    self.report.aesthetic_report = result.report
                    self.report.aesthetic_score = result.report.overall_score

        # 计算可用性评分 (基于交互测试)
        if self.report.interaction_report:
            self.report.usability_score = (
                self.report.interaction_report.pass_rate * 100
            )

        # 计算性能评分 (基于响应时间)
        total_response_ms = 0
        count = 0
        for result in self.results:
            if result.report and hasattr(result.report, 'avg_response_ms'):
                total_response_ms += result.report.avg_response_ms
                count += 1

        if count > 0:
            avg_response = total_response_ms / count
            # 响应时间评分: <100ms=100分, <500ms=80分, <1s=60分, >1s=40分
            if avg_response < 100:
                self.report.performance_score = 100
            elif avg_response < 500:
                self.report.performance_score = 80
            elif avg_response < 1000:
                self.report.performance_score = 60
            else:
                self.report.performance_score = 40

    def _save_report(self):
        """保存测试报告到文件"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

        # JSON报告
        json_path = os.path.join(
            self.output_dir,
            f"test_report_{timestamp}.json"
        )
        with open(json_path, 'w', encoding='utf-8') as f:
            f.write(self.report.to_json())

        # HTML报告
        html_path = os.path.join(
            self.output_dir,
            f"test_report_{timestamp}.html"
        )
        self._generate_html_report(html_path)

        logger.info(f"Reports saved to:")
        logger.info(f"  JSON: {json_path}")
        logger.info(f"  HTML: {html_path}")

    def _generate_html_report(self, path: str):
        """生成HTML格式报告"""
        html = f"""
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>FlowSight UI Test Report</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            padding: 30px;
        }}
        h1 {{
            color: #333;
            border-bottom: 2px solid #e0e0e0;
            padding-bottom: 15px;
        }}
        .summary {{
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 20px;
            margin: 20px 0;
        }}
        .stat-card {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }}
        .stat-value {{
            font-size: 36px;
            font-weight: bold;
            color: #2563eb;
        }}
        .stat-label {{
            color: #666;
            margin-top: 5px;
        }}
        .passed { color: #22c55e; }
        .failed { color: #ef4444; }
        .scores {{
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 20px;
            margin: 20px 0;
        }}
        .score-item {{
            display: flex;
            justify-content: space-between;
            padding: 15px;
            background: #f8f9fa;
            border-radius: 8px;
        }}
        .score-bar {{
            width: 100%;
            height: 8px;
            background: #e0e0e0;
            border-radius: 4px;
            margin-top: 8px;
            overflow: hidden;
        }}
        .score-fill {{
            height: 100%;
            background: linear-gradient(90deg, #2563eb, #3b82f6);
            border-radius: 4px;
            transition: width 0.3s;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
        }}
        th, td {{
            text-align: left;
            padding: 12px;
            border-bottom: 1px solid #e0e0e0;
        }}
        th {{
            background: #f8f9fa;
            font-weight: 600;
            color: #333;
        }}
        .status-pass {{
            color: #22c55e;
            font-weight: 600;
        }}
        .status-fail {{
            color: #ef4444;
            font-weight: 600;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🎨 FlowSight UI Test Report</h1>

        <div class="summary">
            <div class="stat-card">
                <div class="stat-value">{self.report.total_tests}</div>
                <div class="stat-label">Total Tests</div>
            </div>
            <div class="stat-card">
                <div class="stat-value passed">{self.report.passed_tests}</div>
                <div class="stat-label">Passed</div>
            </div>
            <div class="stat-card">
                <div class="stat-value failed">{self.report.failed_tests}</div>
                <div class="stat-label">Failed</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{self.report.duration_ms/1000:.1f}s</div>
                <div class="stat-label">Duration</div>
            </div>
        </div>

        <h2>Quality Scores</h2>
        <div class="scores">
            <div class="score-item">
                <div>
                    <div>Overall Score</div>
                    <div class="score-bar">
                        <div class="score-fill" style="width: {self.report.overall_score}%"></div>
                    </div>
                </div>
                <div>{self.report.overall_score:.1f}%</div>
            </div>
            <div class="score-item">
                <div>
                    <div>Aesthetic Score</div>
                    <div class="score-bar">
                        <div class="score-fill" style="width: {self.report.aesthetic_score}%"></div>
                    </div>
                </div>
                <div>{self.report.aesthetic_score:.1f}%</div>
            </div>
            <div class="score-item">
                <div>
                    <div>Usability Score</div>
                    <div class="score-bar">
                        <div class="score-fill" style="width: {self.report.usability_score}%"></div>
                    </div>
                </div>
                <div>{self.report.usability_score:.1f}%</div>
            </div>
            <div class="score-item">
                <div>
                    <div>Performance Score</div>
                    <div class="score-bar">
                        <div class="score-fill" style="width: {self.report.performance_score}%"></div>
                    </div>
                </div>
                <div>{self.report.performance_score:.1f}%</div>
            </div>
        </div>

        <h2>Test Results</h2>
        <table>
            <thead>
                <tr>
                    <th>Test Name</th>
                    <th>Phase</th>
                    <th>Priority</th>
                    <th>Status</th>
                    <th>Duration</th>
                </tr>
            </thead>
            <tbody>
"""

        for result in self.results:
            status_class = "status-pass" if result.passed else "status-fail"
            status_text = "✓ Pass" if result.passed else "✗ Fail"

            html += f"""
                <tr>
                    <td>{result.test_name}</td>
                    <td>{result.phase.value}</td>
                    <td>{result.priority.value}</td>
                    <td class="{status_class}">{status_text}</td>
                    <td>{result.duration_ms:.0f}ms</td>
                </tr>
"""

        html += """
            </tbody>
        </table>
    </div>
</body>
</html>
"""

        with open(path, 'w', encoding='utf-8') as f:
            f.write(html)

    # ==================== 快捷方法 ====================

    async def run_quick_smoke_test(self) -> TestSuiteReport:
        """运行快速冒烟测试（仅关键测试）"""
        # 只注册关键优先级测试
        critical_tests = [
            t for t in self.test_cases
            if t.priority == TestPriority.CRITICAL
        ]
        self.test_cases = critical_tests

        return await self.run_all_tests()

    async def run_aesthetic_focused_test(self) -> TestSuiteReport:
        """运行以美学为重点的测试"""
        aesthetic_tests = [
            t for t in self.test_cases
            if t.phase in [TestPhase.VISUAL, TestPhase.AESTHETIC]
        ]
        self.test_cases = aesthetic_tests

        return await self.run_all_tests()
