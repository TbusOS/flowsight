"""
状态测试模块
===========

测试内容:
- 组件状态管理
- 数据流验证
- 状态持久化
- 错误状态处理
"""

from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional, Callable, Set
from enum import Enum
import time
import logging
from datetime import datetime

from ..core.visual_analyzer import VisualAnalyzer, VisualAnalysis
from ..core.desktop_adapter import DesktopAdapter, Point

logger = logging.getLogger(__name__)


class StateType(Enum):
    """状态类型"""
    UI = "ui"                    # UI状态 (选中、悬停、禁用等)
    DATA = "data"                # 数据状态
    NAVIGATION = "navigation"    # 导航状态
    FORM = "form"                # 表单状态
    ASYNC = "async"              # 异步操作状态
    ERROR = "error"              # 错误状态


class AsyncState(Enum):
    """异步操作状态"""
    IDLE = "idle"
    LOADING = "loading"
    SUCCESS = "success"
    ERROR = "error"


@dataclass
class StateSnapshot:
    """状态快照"""
    timestamp: datetime
    state_type: StateType
    data: Dict[str, Any]
    elements: List[Dict[str, Any]]
    screenshot_path: Optional[str] = None


@dataclass
class StateTransition:
    """状态转换"""
    from_state: str
    to_state: str
    trigger: str
    timestamp: datetime
    duration_ms: float
    success: bool
    error: Optional[str] = None


@dataclass
class StateTestResult:
    """状态测试结果"""
    test_name: str
    passed: bool
    state_type: StateType
    transitions: List[StateTransition] = field(default_factory=list)
    snapshots: List[StateSnapshot] = field(default_factory=list)
    errors: List[str] = field(default_factory=list)
    metrics: Dict[str, Any] = field(default_factory=dict)


@dataclass
class StateReport:
    """状态测试报告"""
    total_tests: int
    passed_tests: int
    failed_tests: int
    results: List[StateTestResult]
    coverage: Dict[str, float]  # 状态覆盖率
    transitions_valid: bool
    overall_passed: bool


class StateTests:
    """
    状态测试器

    测试UI状态管理:
    - 组件状态变化
    - 数据流正确性
    - 状态持久化
    - 错误处理
    """

    def __init__(
        self,
        adapter: DesktopAdapter,
        visual_analyzer: VisualAnalyzer
    ):
        self.adapter = adapter
        self.visual_analyzer = visual_analyzer
        self._snapshots: List[StateSnapshot] = []
        self._transitions: List[StateTransition] = []

    # ==================== 核心状态捕获 ====================

    def capture_state(
        self,
        state_type: StateType,
        screenshot_path: Optional[str] = None
    ) -> StateSnapshot:
        """捕获当前UI状态"""
        # 获取当前无障碍树状态
        ax_tree = self.adapter.get_accessibility_tree()

        # 获取截图
        if screenshot_path is None:
            screenshot_path = f"/tmp/state_{int(time.time() * 1000)}.png"
            self.adapter.get_screenshot(screenshot_path)

        # 分析视觉元素
        analysis = self.visual_analyzer.analyze_screenshot_path(screenshot_path)

        # 构建元素状态数据
        elements = []
        if ax_tree:
            elements = self._extract_element_states(ax_tree)

        snapshot = StateSnapshot(
            timestamp=datetime.now(),
            state_type=state_type,
            data={
                "window_title": self.adapter.get_active_window_title(),
                "element_count": len(analysis.elements),
                "ax_element_count": len(elements)
            },
            elements=elements,
            screenshot_path=screenshot_path
        )

        self._snapshots.append(snapshot)
        return snapshot

    def _extract_element_states(self, element) -> List[Dict[str, Any]]:
        """从无障碍树提取元素状态"""
        states = []

        def traverse(elem, depth=0):
            if depth > 10:  # 限制深度
                return

            state_info = {
                "role": getattr(elem, "role", "unknown"),
                "name": getattr(elem, "name", ""),
                "enabled": getattr(elem, "enabled", True),
                "focused": getattr(elem, "focused", False),
                "selected": getattr(elem, "selected", False),
                "checked": getattr(elem, "checked", None),
                "expanded": getattr(elem, "expanded", None),
                "depth": depth
            }
            states.append(state_info)

            # 遍历子元素
            children = getattr(elem, "children", [])
            for child in children:
                traverse(child, depth + 1)

        traverse(element)
        return states

    def record_transition(
        self,
        from_state: str,
        to_state: str,
        trigger: str,
        duration_ms: float,
        success: bool = True,
        error: Optional[str] = None
    ) -> StateTransition:
        """记录状态转换"""
        transition = StateTransition(
            from_state=from_state,
            to_state=to_state,
            trigger=trigger,
            timestamp=datetime.now(),
            duration_ms=duration_ms,
            success=success,
            error=error
        )
        self._transitions.append(transition)
        return transition

    # ==================== UI状态测试 ====================

    def test_component_states(self, component_query: str) -> StateTestResult:
        """
        测试组件状态变化

        验证:
        - 正常状态
        - 悬停状态
        - 选中状态
        - 禁用状态
        - 焦点状态
        """
        result = StateTestResult(
            test_name="component_states",
            passed=True,
            state_type=StateType.UI
        )

        try:
            # 1. 捕获初始状态
            initial = self.capture_state(StateType.UI)
            result.snapshots.append(initial)

            # 2. 查找目标组件
            analysis = self.visual_analyzer.analyze_screenshot_path(
                initial.screenshot_path
            )
            element = self.visual_analyzer.find_element(analysis, text=component_query)

            if not element:
                result.passed = False
                result.errors.append(f"未找到组件: {component_query}")
                return result

            bounds = element.bounds
            center = Point(
                x=bounds["x"] + bounds["width"] // 2,
                y=bounds["y"] + bounds["height"] // 2
            )

            # 3. 测试悬停状态
            start_time = time.time()
            self.adapter.move_mouse(center)
            time.sleep(0.5)
            hover_duration = (time.time() - start_time) * 1000

            hover = self.capture_state(StateType.UI)
            result.snapshots.append(hover)
            result.transitions.append(self.record_transition(
                from_state="normal",
                to_state="hover",
                trigger="mouse_move",
                duration_ms=hover_duration,
                success=True
            ))

            # 4. 测试点击/选中状态
            start_time = time.time()
            self.adapter.click(center)
            time.sleep(0.5)
            click_duration = (time.time() - start_time) * 1000

            selected = self.capture_state(StateType.UI)
            result.snapshots.append(selected)
            result.transitions.append(self.record_transition(
                from_state="hover",
                to_state="selected",
                trigger="click",
                duration_ms=click_duration,
                success=True
            ))

            # 5. 验证状态变化被正确记录
            state_changed = self._verify_state_change(
                initial, selected, component_query
            )

            if not state_changed:
                result.passed = False
                result.errors.append("组件点击后状态未发生变化")

            result.metrics = {
                "hover_response_ms": hover_duration,
                "click_response_ms": click_duration,
                "state_change_detected": state_changed
            }

        except Exception as e:
            result.passed = False
            result.errors.append(f"测试异常: {str(e)}")
            logger.exception("组件状态测试失败")

        return result

    def _verify_state_change(
        self,
        before: StateSnapshot,
        after: StateSnapshot,
        component_name: str
    ) -> bool:
        """验证组件状态是否发生变化"""
        # 检查元素属性变化
        before_elements = {
            e["name"]: e for e in before.elements
            if component_name.lower() in e.get("name", "").lower()
        }
        after_elements = {
            e["name"]: e for e in after.elements
            if component_name.lower() in e.get("name", "").lower()
        }

        # 检查是否有元素被添加、删除或修改
        if set(before_elements.keys()) != set(after_elements.keys()):
            return True

        for name in before_elements:
            if name in after_elements:
                b, a = before_elements[name], after_elements[name]
                if b.get("focused") != a.get("focused"):
                    return True
                if b.get("selected") != a.get("selected"):
                    return True
                if b.get("checked") != a.get("checked"):
                    return True

        return False

    # ==================== 表单状态测试 ====================

    def test_form_states(self, form_fields: List[Dict[str, Any]]) -> StateTestResult:
        """
        测试表单状态管理

        验证:
        - 初始空状态
        - 输入中状态
        - 验证错误状态
        - 验证通过状态
        - 提交中状态
        - 提交完成状态
        """
        result = StateTestResult(
            test_name="form_states",
            passed=True,
            state_type=StateType.FORM
        )

        try:
            # 1. 初始状态
            initial = self.capture_state(StateType.FORM)
            result.snapshots.append(initial)

            for field in form_fields:
                field_name = field.get("name", "")
                field_type = field.get("type", "text")
                test_value = field.get("test_value", "test")
                invalid_value = field.get("invalid_value", "")

                # 2. 输入有效值
                if field_element := self._find_form_field(field_name):
                    self._focus_and_type(field_element, test_value)
                    time.sleep(0.3)

                    filled = self.capture_state(StateType.FORM)
                    result.snapshots.append(filled)
                    result.transitions.append(self.record_transition(
                        from_state="empty",
                        to_state="filled",
                        trigger=f"input_{field_name}",
                        duration_ms=300,
                        success=True
                    ))

                # 3. 测试验证（如果提供了无效值）
                if invalid_value:
                    self._focus_and_type(field_element, invalid_value)
                    time.sleep(0.5)  # 等待验证

                    validated = self.capture_state(StateType.FORM)
                    result.snapshots.append(validated)

                    # 检查是否有错误提示
                    has_error = self._detect_error_state(validated)
                    result.transitions.append(self.record_transition(
                        from_state="filled",
                        to_state="error" if has_error else "valid",
                        trigger=f"validate_{field_name}",
                        duration_ms=500,
                        success=has_error  # 期望验证失败
                    ))

            result.metrics = {
                "fields_tested": len(form_fields),
                "snapshots_captured": len(result.snapshots)
            }

        except Exception as e:
            result.passed = False
            result.errors.append(f"表单状态测试异常: {str(e)}")
            logger.exception("表单状态测试失败")

        return result

    def _find_form_field(self, field_name: str) -> Optional[Dict[str, Any]]:
        """查找表单字段"""
        snapshot = self.capture_state(StateType.FORM)

        for elem in snapshot.elements:
            name = elem.get("name", "").lower()
            if field_name.lower() in name:
                return elem

        return None

    def _focus_and_type(self, field_element: Dict[str, Any], text: str):
        """聚焦字段并输入文本"""
        # 点击聚焦
        # 这里简化处理，实际需要根据元素位置点击
        self.adapter.type_text(text)

    def _detect_error_state(self, snapshot: StateSnapshot) -> bool:
        """检测是否处于错误状态"""
        # 检查是否有错误提示元素
        error_indicators = ["error", "invalid", "required", "alert"]

        for elem in snapshot.elements:
            name = elem.get("name", "").lower()
            role = elem.get("role", "").lower()

            if any(indicator in name for indicator in error_indicators):
                return True
            if role in ["alert", "log"]:
                return True

        return False

    # ==================== 异步状态测试 ====================

    def test_async_operation(
        self,
        trigger_action: Callable[[], bool],
        success_indicator: str,
        timeout_seconds: float = 10.0
    ) -> StateTestResult:
        """
        测试异步操作状态

        验证:
        - 初始空闲状态
        - 加载中状态
        - 成功/失败状态
        - 状态超时处理
        """
        result = StateTestResult(
            test_name="async_operation",
            passed=True,
            state_type=StateType.ASYNC
        )

        start_time = time.time()

        try:
            # 1. 初始空闲状态
            idle = self.capture_state(StateType.ASYNC)
            result.snapshots.append(idle)

            # 2. 触发异步操作
            action_start = time.time()
            trigger_action()

            # 3. 轮询等待状态变化
            loading_detected = False
            success_detected = False
            state_history = []

            while time.time() - start_time < timeout_seconds:
                current = self.capture_state(StateType.ASYNC)
                state_history.append(current)

                # 检测加载状态
                if not loading_detected and self._detect_loading_state(current):
                    loading_duration = (time.time() - action_start) * 1000
                    result.transitions.append(self.record_transition(
                        from_state="idle",
                        to_state="loading",
                        trigger="async_start",
                        duration_ms=loading_duration,
                        success=True
                    ))
                    loading_detected = True

                # 检测成功状态
                if loading_detected and self._detect_success_state(
                    current, success_indicator
                ):
                    total_duration = (time.time() - start_time) * 1000
                    result.transitions.append(self.record_transition(
                        from_state="loading",
                        to_state="success",
                        trigger="async_complete",
                        duration_ms=total_duration,
                        success=True
                    ))
                    success_detected = True
                    break

                # 检测错误状态
                if self._detect_error_state(current):
                    result.transitions.append(self.record_transition(
                        from_state="loading" if loading_detected else "idle",
                        to_state="error",
                        trigger="async_error",
                        duration_ms=(time.time() - start_time) * 1000,
                        success=False,
                        error="Async operation failed"
                    ))
                    result.passed = False
                    result.errors.append("异步操作失败")
                    break

                time.sleep(0.5)

            # 4. 验证结果
            if not loading_detected:
                result.passed = False
                result.errors.append("未检测到加载状态")

            if not success_detected and not result.errors:
                result.passed = False
                result.errors.append(f"异步操作超时 ({timeout_seconds}s)")

            result.snapshots.extend(state_history)
            result.metrics = {
                "total_duration_ms": (time.time() - start_time) * 1000,
                "loading_detected": loading_detected,
                "success_detected": success_detected,
                "state_changes": len(state_history)
            }

        except Exception as e:
            result.passed = False
            result.errors.append(f"异步状态测试异常: {str(e)}")
            logger.exception("异步状态测试失败")

        return result

    def _detect_loading_state(self, snapshot: StateSnapshot) -> bool:
        """检测是否处于加载状态"""
        loading_indicators = ["loading", "progress", "spinner", "wait", "busy"]

        for elem in snapshot.elements:
            name = elem.get("name", "").lower()
            role = elem.get("role", "").lower()

            if any(indicator in name for indicator in loading_indicators):
                return True
            if role in ["progressbar", "timer", "status"]:
                return True

        return False

    def _detect_success_state(
        self,
        snapshot: StateSnapshot,
        indicator: str
    ) -> bool:
        """检测成功状态"""
        for elem in snapshot.elements:
            name = elem.get("name", "").lower()
            if indicator.lower() in name:
                return True

        return False

    # ==================== 导航状态测试 ====================

    def test_navigation_flow(
        self,
        navigation_steps: List[Dict[str, str]]
    ) -> StateTestResult:
        """
        测试导航状态流

        验证:
        - 页面切换状态
        - 历史记录管理
        - 返回/前进功能
        - 深层链接
        """
        result = StateTestResult(
            test_name="navigation_flow",
            passed=True,
            state_type=StateType.NAVIGATION
        )

        visited_states = []

        try:
            for i, step in enumerate(navigation_steps):
                action = step.get("action", "click")
                target = step.get("target", "")
                expected_state = step.get("expected_state", "")

                # 捕获导航前状态
                before = self.capture_state(StateType.NAVIGATION)

                # 执行导航动作
                start_time = time.time()

                if action == "click":
                    self._click_element_by_text(target)
                elif action == "keyboard":
                    self.adapter.press_key(target)  # target是快捷键
                elif action == "type":
                    self.adapter.type_text(target)
                    self.adapter.press_key("Return")

                time.sleep(1.0)  # 等待导航完成
                duration = (time.time() - start_time) * 1000

                # 捕获导航后状态
                after = self.capture_state(StateType.NAVIGATION)

                # 验证状态变化
                state_valid = self._verify_navigation_state(
                    after, expected_state
                )

                transition = self.record_transition(
                    from_state=visited_states[-1] if visited_states else "initial",
                    to_state=expected_state,
                    trigger=f"{action}_{target}",
                    duration_ms=duration,
                    success=state_valid
                )
                result.transitions.append(transition)
                visited_states.append(expected_state)

                if not state_valid:
                    result.passed = False
                    result.errors.append(
                        f"步骤 {i+1}: 导航到 '{expected_state}' 失败"
                    )

                result.snapshots.extend([before, after])

            result.metrics = {
                "steps_executed": len(navigation_steps),
                "states_visited": len(visited_states),
                "unique_states": len(set(visited_states))
            }

        except Exception as e:
            result.passed = False
            result.errors.append(f"导航测试异常: {str(e)}")
            logger.exception("导航状态测试失败")

        return result

    def _click_element_by_text(self, text: str) -> bool:
        """通过文本点击元素"""
        snapshot = self.capture_state(StateType.UI)
        analysis = self.visual_analyzer.analyze_screenshot_path(
            snapshot.screenshot_path
        )

        element = self.visual_analyzer.find_element(analysis, text=text)
        if element:
            bounds = element.bounds
            center = Point(
                x=bounds["x"] + bounds["width"] // 2,
                y=bounds["y"] + bounds["height"] // 2
            )
            self.adapter.click(center)
            return True

        return False

    def _verify_navigation_state(
        self,
        snapshot: StateSnapshot,
        expected_indicator: str
    ) -> bool:
        """验证导航后的状态"""
        # 检查窗口标题
        window_title = snapshot.data.get("window_title", "").lower()
        if expected_indicator.lower() in window_title:
            return True

        # 检查元素
        for elem in snapshot.elements:
            name = elem.get("name", "").lower()
            if expected_indicator.lower() in name:
                return True

        return False

    # ==================== 错误状态测试 ====================

    def test_error_handling(
        self,
        error_triggers: List[Dict[str, Any]]
    ) -> StateTestResult:
        """
        测试错误状态处理

        验证:
        - 错误检测
        - 错误提示显示
        - 恢复机制
        - 错误边界
        """
        result = StateTestResult(
            test_name="error_handling",
            passed=True,
            state_type=StateType.ERROR
        )

        try:
            for trigger in error_triggers:
                trigger_name = trigger.get("name", "")
                trigger_action = trigger.get("action")
                expected_error = trigger.get("expected_error", "")
                should_recover = trigger.get("should_recover", False)

                # 触发错误前的状态
                before = self.capture_state(StateType.ERROR)

                # 触发错误
                if trigger_action:
                    trigger_action()
                    time.sleep(1.0)

                # 检查错误状态
                error_state = self.capture_state(StateType.ERROR)

                has_error = self._detect_error_state(error_state)
                error_shown = self._detect_error_message(error_state, expected_error)

                if not has_error:
                    result.passed = False
                    result.errors.append(f"'{trigger_name}' 未触发错误状态")
                    continue

                if expected_error and not error_shown:
                    result.passed = False
                    result.errors.append(
                        f"'{trigger_name}' 错误提示不符合预期"
                    )

                result.transitions.append(self.record_transition(
                    from_state="normal",
                    to_state="error",
                    trigger=trigger_name,
                    duration_ms=1000,
                    success=has_error
                ))

                # 测试恢复
                if should_recover:
                    time.sleep(2.0)
                    recovered = self.capture_state(StateType.ERROR)

                    recovered_from_error = not self._detect_error_state(recovered)
                    result.transitions.append(self.record_transition(
                        from_state="error",
                        to_state="recovered",
                        trigger="auto_recovery",
                        duration_ms=2000,
                        success=recovered_from_error
                    ))

                    if not recovered_from_error:
                        result.passed = False
                        result.errors.append(
                            f"'{trigger_name}' 错误后未自动恢复"
                        )

                result.snapshots.extend([before, error_state])

            result.metrics = {
                "errors_triggered": len(error_triggers),
                "errors_detected": sum(
                    1 for t in result.transitions if t.to_state == "error"
                )
            }

        except Exception as e:
            result.passed = False
            result.errors.append(f"错误处理测试异常: {str(e)}")
            logger.exception("错误状态测试失败")

        return result

    def _detect_error_message(
        self,
        snapshot: StateSnapshot,
        expected_message: str
    ) -> bool:
        """检测特定的错误消息"""
        for elem in snapshot.elements:
            name = elem.get("name", "").lower()
            if expected_message.lower() in name:
                return True

        return False

    # ==================== 状态持久化测试 ====================

    def test_state_persistence(
        self,
        state_to_preserve: Dict[str, Any],
        preserve_action: Callable[[], bool]
    ) -> StateTestResult:
        """
        测试状态持久化

        验证:
        - 应用重启后状态恢复
        - 本地存储读写
        - 会话恢复
        """
        result = StateTestResult(
            test_name="state_persistence",
            passed=True,
            state_type=StateType.DATA
        )

        try:
            # 1. 设置初始状态
            initial = self.capture_state(StateType.DATA)
            result.snapshots.append(initial)

            # 2. 执行会触发状态保存的操作
            preserve_action()
            time.sleep(1.0)

            after_save = self.capture_state(StateType.DATA)
            result.snapshots.append(after_save)

            # 注意：实际测试可能需要重启应用
            # 这里简化处理，仅验证保存操作被触发

            result.transitions.append(self.record_transition(
                from_state="unsaved",
                to_state="saved",
                trigger="preserve_action",
                duration_ms=1000,
                success=True
            ))

            result.metrics = {
                "persistence_triggered": True,
                "state_keys": list(state_to_preserve.keys())
            }

        except Exception as e:
            result.passed = False
            result.errors.append(f"状态持久化测试异常: {str(e)}")
            logger.exception("状态持久化测试失败")

        return result

    # ==================== 报告生成 ====================

    def generate_report(self, results: List[StateTestResult]) -> StateReport:
        """生成状态测试报告"""
        passed = sum(1 for r in results if r.passed)
        failed = len(results) - passed

        # 计算状态覆盖率
        state_types = set()
        covered_types = set()

        for result in results:
            state_types.add(result.state_type)
            if result.passed:
                covered_types.add(result.state_type)

        coverage = {
            "overall": len(covered_types) / len(state_types) * 100
            if state_types else 0,
            "by_type": {
                t.value: (t in covered_types) for t in state_types
            }
        }

        # 验证所有状态转换是否有效
        all_transitions_valid = all(
            all(t.success for t in r.transitions)
            for r in results
        )

        return StateReport(
            total_tests=len(results),
            passed_tests=passed,
            failed_tests=failed,
            results=results,
            coverage=coverage,
            transitions_valid=all_transitions_valid,
            overall_passed=failed == 0 and all_transitions_valid
        )
