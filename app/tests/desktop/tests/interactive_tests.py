"""
交互测试模块
===========

测试内容:
- 鼠标交互 (点击、悬停、拖拽)
- 键盘交互 (快捷键、Tab导航)
- 菜单交互 (下拉菜单、右键菜单)
- 表单交互 (输入、验证、提交)
- 拖拽与排序
"""

from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional, Callable
from PIL import Image
import logging
import time

from ..core.desktop_adapter import DesktopAdapter, Point, Rect
from ..core.visual_analyzer import VisualAnalyzer, VisualAnalysis, UIElement

logger = logging.getLogger(__name__)


@dataclass
class InteractionResult:
    """交互结果"""
    action: str
    success: bool
    duration_ms: float
    before_screenshot: Optional[Image.Image] = None
    after_screenshot: Optional[Image.Image] = None
    error_message: Optional[str] = None
    state_changes: List[str] = field(default_factory=list)


@dataclass
class InteractiveReport:
    """交互测试报告"""
    total_tests: int
    passed: int
    failed: int
    results: List[InteractionResult]
    mouse_score: float
    keyboard_score: float
    menu_score: float
    form_score: float
    overall_score: float


class InteractiveTests:
    """
    交互测试器

    测试UI交互功能，包括:
    - 鼠标点击和悬停
    - 键盘快捷键和导航
    - 菜单展开和选择
    - 表单输入和验证
    """

    def __init__(
        self,
        adapter: DesktopAdapter,
        visual_analyzer: VisualAnalyzer
    ):
        self.adapter = adapter
        self.visual_analyzer = visual_analyzer
        self.results: List[InteractionResult] = []

    # ==================== 鼠标交互测试 ====================

    def test_button_click(
        self,
        button_text: str,
        expected_result: Optional[Callable[[VisualAnalysis], bool]] = None
    ) -> InteractionResult:
        """
        测试按钮点击

        Args:
            button_text: 按钮上的文本
            expected_result: 可选的验证函数
        """
        logger.info(f"测试按钮点击: {button_text}")

        start_time = time.time()

        try:
            # 截图（点击前）
            before = self.adapter.screenshot()

            # 分析截图找到按钮
            analysis = self.visual_analyzer.analyze_screenshot(before)
            button = self.visual_analyzer.find_element(
                analysis,
                element_type='button',
                text=button_text
            )

            if not button:
                duration = (time.time() - start_time) * 1000
                return InteractionResult(
                    action=f'click_button_{button_text}',
                    success=False,
                    duration_ms=duration,
                    before_screenshot=before,
                    error_message=f'未找到按钮: {button_text}'
                )

            # 点击按钮中心
            center = Point(
                button.bounds['x'] + button.bounds['width'] // 2,
                button.bounds['y'] + button.bounds['height'] // 2
            )
            self.adapter.mouse_click(center)
            self.adapter.wait(0.3)

            # 截图（点击后）
            after = self.adapter.screenshot()
            after_analysis = self.visual_analyzer.analyze_screenshot(after)

            # 验证结果
            success = True
            if expected_result:
                success = expected_result(after_analysis)

            duration = (time.time() - start_time) * 1000

            result = InteractionResult(
                action=f'click_button_{button_text}',
                success=success,
                duration_ms=duration,
                before_screenshot=before,
                after_screenshot=after,
                state_changes=self._detect_state_changes(analysis, after_analysis)
            )
            self.results.append(result)
            return result

        except Exception as e:
            duration = (time.time() - start_time) * 1000
            result = InteractionResult(
                action=f'click_button_{button_text}',
                success=False,
                duration_ms=duration,
                error_message=str(e)
            )
            self.results.append(result)
            return result

    def test_hover_effects(
        self,
        element_text: str,
        element_type: str = 'button'
    ) -> InteractionResult:
        """
        测试悬停效果

        检查悬停时是否有视觉反馈
        """
        logger.info(f"测试悬停效果: {element_text}")

        start_time = time.time()

        try:
            # 初始截图
            before = self.adapter.screenshot()
            analysis = self.visual_analyzer.analyze_screenshot(before)

            element = self.visual_analyzer.find_element(
                analysis,
                element_type=element_type,
                text=element_text
            )

            if not element:
                duration = (time.time() - start_time) * 1000
                return InteractionResult(
                    action=f'hover_{element_type}_{element_text}',
                    success=False,
                    duration_ms=duration,
                    before_screenshot=before,
                    error_message=f'未找到元素: {element_text}'
                )

            # 移动鼠标到元素
            center = Point(
                element.bounds['x'] + element.bounds['width'] // 2,
                element.bounds['y'] + element.bounds['height'] // 2
            )
            self.adapter.mouse_move(center)
            self.adapter.wait(0.2)  # 等待悬停效果

            # 悬停后截图
            after = self.adapter.screenshot()

            # 检测视觉变化
            changes = self._detect_visual_changes(before, after, element.bounds)

            duration = (time.time() - start_time) * 1000

            # 悬停应该有视觉反馈
            success = len(changes) > 0

            result = InteractionResult(
                action=f'hover_{element_type}_{element_text}',
                success=success,
                duration_ms=duration,
                before_screenshot=before,
                after_screenshot=after,
                state_changes=changes if success else ['无悬停效果']
            )
            self.results.append(result)
            return result

        except Exception as e:
            duration = (time.time() - start_time) * 1000
            result = InteractionResult(
                action=f'hover_{element_type}_{element_text}',
                success=False,
                duration_ms=duration,
                error_message=str(e)
            )
            self.results.append(result)
            return result

    def test_drag_and_drop(
        self,
        source_text: str,
        target_text: str
    ) -> InteractionResult:
        """
        测试拖拽功能
        """
        logger.info(f"测试拖拽: {source_text} -> {target_text}")

        start_time = time.time()

        try:
            before = self.adapter.screenshot()
            analysis = self.visual_analyzer.analyze_screenshot(before)

            # 查找源和目标
            source = self.visual_analyzer.find_element(analysis, text=source_text)
            target = self.visual_analyzer.find_element(analysis, text=target_text)

            if not source or not target:
                duration = (time.time() - start_time) * 1000
                return InteractionResult(
                    action=f'drag_{source_text}_to_{target_text}',
                    success=False,
                    duration_ms=duration,
                    before_screenshot=before,
                    error_message='未找到源或目标元素'
                )

            # 执行拖拽
            start_point = Point(
                source.bounds['x'] + source.bounds['width'] // 2,
                source.bounds['y'] + source.bounds['height'] // 2
            )
            end_point = Point(
                target.bounds['x'] + target.bounds['width'] // 2,
                target.bounds['y'] + target.bounds['height'] // 2
            )

            self.adapter.mouse_drag(start_point, end_point)
            self.adapter.wait(0.5)

            after = self.adapter.screenshot()
            after_analysis = self.visual_analyzer.analyze_screenshot(after)

            duration = (time.time() - start_time) * 1000

            result = InteractionResult(
                action=f'drag_{source_text}_to_{target_text}',
                success=True,
                duration_ms=duration,
                before_screenshot=before,
                after_screenshot=after,
                state_changes=self._detect_state_changes(analysis, after_analysis)
            )
            self.results.append(result)
            return result

        except Exception as e:
            duration = (time.time() - start_time) * 1000
            result = InteractionResult(
                action=f'drag_{source_text}_to_{target_text}',
                success=False,
                duration_ms=duration,
                error_message=str(e)
            )
            self.results.append(result)
            return result

    # ==================== 键盘交互测试 ====================

    def test_keyboard_navigation(
        self,
        expected_focus_order: List[str]
    ) -> InteractionResult:
        """
        测试Tab键导航顺序
        """
        logger.info("测试键盘导航")

        start_time = time.time()

        try:
            before = self.adapter.screenshot()

            focus_order = []
            for _ in range(len(expected_focus_order) + 2):  # 多按几次确保循环
                self.adapter.key_press('tab')
                self.adapter.wait(0.1)

                after = self.adapter.screenshot()
                analysis = self.visual_analyzer.analyze_screenshot(after)

                # 查找当前聚焦的元素
                focused = self._find_focused_element(analysis)
                if focused:
                    focus_order.append(focused)

            duration = (time.time() - start_time) * 1000

            # 验证顺序
            success = focus_order == expected_focus_order

            result = InteractionResult(
                action='keyboard_navigation',
                success=success,
                duration_ms=duration,
                before_screenshot=before,
                state_changes=[f'焦点顺序: {focus_order}']
            )
            self.results.append(result)
            return result

        except Exception as e:
            duration = (time.time() - start_time) * 1000
            result = InteractionResult(
                action='keyboard_navigation',
                success=False,
                duration_ms=duration,
                error_message=str(e)
            )
            self.results.append(result)
            return result

    def test_keyboard_shortcut(
        self,
        keys: List[str],
        expected_result: Optional[Callable[[VisualAnalysis], bool]] = None
    ) -> InteractionResult:
        """
        测试键盘快捷键

        Args:
            keys: 按键序列，如 ['ctrl', 's']
        """
        logger.info(f"测试快捷键: {'+'.join(keys)}")

        start_time = time.time()

        try:
            before = self.adapter.screenshot()
            before_analysis = self.visual_analyzer.analyze_screenshot(before)

            # 发送快捷键
            self.adapter.hotkey(*keys)
            self.adapter.wait(0.3)

            after = self.adapter.screenshot()
            after_analysis = self.visual_analyzer.analyze_screenshot(after)

            success = True
            if expected_result:
                success = expected_result(after_analysis)

            duration = (time.time() - start_time) * 1000

            result = InteractionResult(
                action=f'hotkey_{"_".join(keys)}',
                success=success,
                duration_ms=duration,
                before_screenshot=before,
                after_screenshot=after,
                state_changes=self._detect_state_changes(before_analysis, after_analysis)
            )
            self.results.append(result)
            return result

        except Exception as e:
            duration = (time.time() - start_time) * 1000
            result = InteractionResult(
                action=f'hotkey_{"_".join(keys)}',
                success=False,
                duration_ms=duration,
                error_message=str(e)
            )
            self.results.append(result)
            return result

    # ==================== 菜单交互测试 ====================

    def test_menu_open(
        self,
        menu_trigger_text: str
    ) -> InteractionResult:
        """
        测试菜单展开
        """
        logger.info(f"测试菜单展开: {menu_trigger_text}")

        start_time = time.time()

        try:
            before = self.adapter.screenshot()
            analysis = self.visual_analyzer.analyze_screenshot(before)

            # 找到菜单触发器
            trigger = self.visual_analyzer.find_element(
                analysis,
                text=menu_trigger_text
            )

            if not trigger:
                duration = (time.time() - start_time) * 1000
                return InteractionResult(
                    action=f'open_menu_{menu_trigger_text}',
                    success=False,
                    duration_ms=duration,
                    before_screenshot=before,
                    error_message=f'未找到菜单触发器: {menu_trigger_text}'
                )

            # 点击触发器
            center = Point(
                trigger.bounds['x'] + trigger.bounds['width'] // 2,
                trigger.bounds['y'] + trigger.bounds['height'] // 2
            )
            self.adapter.mouse_click(center)
            self.adapter.wait(0.3)

            after = self.adapter.screenshot()
            after_analysis = self.visual_analyzer.analyze_screenshot(after)

            # 检查菜单是否展开（寻找menu或dropdown元素）
            menus = self.visual_analyzer.find_elements(after_analysis, 'menu')
            dropdowns = self.visual_analyzer.find_elements(after_analysis, 'dropdown')
            success = len(menus) > 0 or len(dropdowns) > 0

            duration = (time.time() - start_time) * 1000

            result = InteractionResult(
                action=f'open_menu_{menu_trigger_text}',
                success=success,
                duration_ms=duration,
                before_screenshot=before,
                after_screenshot=after,
                state_changes=[f'菜单项数: {len(menus) + len(dropdowns)}']
            )
            self.results.append(result)
            return result

        except Exception as e:
            duration = (time.time() - start_time) * 1000
            result = InteractionResult(
                action=f'open_menu_{menu_trigger_text}',
                success=False,
                duration_ms=duration,
                error_message=str(e)
            )
            self.results.append(result)
            return result

    def test_context_menu(
        self,
        target_element_text: str
    ) -> InteractionResult:
        """
        测试右键菜单
        """
        logger.info(f"测试右键菜单: {target_element_text}")

        start_time = time.time()

        try:
            before = self.adapter.screenshot()
            analysis = self.visual_analyzer.analyze_screenshot(before)

            element = self.visual_analyzer.find_element(analysis, text=target_element_text)

            if not element:
                duration = (time.time() - start_time) * 1000
                return InteractionResult(
                    action=f'context_menu_{target_element_text}',
                    success=False,
                    duration_ms=duration,
                    before_screenshot=before,
                    error_message=f'未找到元素: {target_element_text}'
                )

            # 右键点击
            center = Point(
                element.bounds['x'] + element.bounds['width'] // 2,
                element.bounds['y'] + element.bounds['height'] // 2
            )
            self.adapter.mouse_click(center, button='right')
            self.adapter.wait(0.3)

            after = self.adapter.screenshot()
            after_analysis = self.visual_analyzer.analyze_screenshot(after)

            # 检查右键菜单是否出现
            menus = self.visual_analyzer.find_elements(after_analysis, 'menu')
            success = len(menus) > 0

            duration = (time.time() - start_time) * 1000

            result = InteractionResult(
                action=f'context_menu_{target_element_text}',
                success=success,
                duration_ms=duration,
                before_screenshot=before,
                after_screenshot=after,
                state_changes=[f'菜单项数: {len(menus)}']
            )
            self.results.append(result)
            return result

        except Exception as e:
            duration = (time.time() - start_time) * 1000
            result = InteractionResult(
                action=f'context_menu_{target_element_text}',
                success=False,
                duration_ms=duration,
                error_message=str(e)
            )
            self.results.append(result)
            return result

    # ==================== 表单交互测试 ====================

    def test_text_input(
        self,
        input_label: str,
        input_text: str
    ) -> InteractionResult:
        """
        测试文本输入
        """
        logger.info(f"测试文本输入: {input_label}")

        start_time = time.time()

        try:
            before = self.adapter.screenshot()
            analysis = self.visual_analyzer.analyze_screenshot(before)

            # 找到输入框
            input_elem = self.visual_analyzer.find_element(
                analysis,
                element_type='input',
                text=input_label
            )

            if not input_elem:
                # 尝试通过标签找到输入框
                label = self.visual_analyzer.find_element(
                    analysis,
                    element_type='text',
                    text=input_label
                )
                if label:
                    # 点击标签旁边的区域
                    input_elem = label

            if not input_elem:
                duration = (time.time() - start_time) * 1000
                return InteractionResult(
                    action=f'text_input_{input_label}',
                    success=False,
                    duration_ms=duration,
                    before_screenshot=before,
                    error_message=f'未找到输入框: {input_label}'
                )

            # 点击输入框
            center = Point(
                input_elem.bounds['x'] + input_elem.bounds['width'] // 2,
                input_elem.bounds['y'] + input_elem.bounds['height'] // 2
            )
            self.adapter.mouse_click(center)
            self.adapter.wait(0.1)

            # 输入文本
            self.adapter.type_text(input_text)
            self.adapter.wait(0.2)

            after = self.adapter.screenshot()

            duration = (time.time() - start_time) * 1000

            result = InteractionResult(
                action=f'text_input_{input_label}',
                success=True,
                duration_ms=duration,
                before_screenshot=before,
                after_screenshot=after,
                state_changes=[f'输入: {input_text[:20]}...' if len(input_text) > 20 else f'输入: {input_text}']
            )
            self.results.append(result)
            return result

        except Exception as e:
            duration = (time.time() - start_time) * 1000
            result = InteractionResult(
                action=f'text_input_{input_label}',
                success=False,
                duration_ms=duration,
                error_message=str(e)
            )
            self.results.append(result)
            return result

    def test_form_submission(
        self,
        submit_button_text: str = 'Submit'
    ) -> InteractionResult:
        """
        测试表单提交
        """
        logger.info(f"测试表单提交: {submit_button_text}")

        return self.test_button_click(submit_button_text)

    # ==================== 辅助方法 ====================

    def _detect_state_changes(
        self,
        before: VisualAnalysis,
        after: VisualAnalysis
    ) -> List[str]:
        """检测两个分析结果之间的状态变化"""
        changes = []

        # 比较元素数量
        if len(before.elements) != len(after.elements):
            changes.append(f'元素数量变化: {len(before.elements)} -> {len(after.elements)}')

        # 检查新增元素
        before_set = {(e.element_type, e.text) for e in before.elements}
        after_set = {(e.element_type, e.text) for e in after.elements}

        added = after_set - before_set
        removed = before_set - after_set

        for elem_type, text in added:
            changes.append(f'新增: {elem_type} ({text or "no text"})')

        for elem_type, text in removed:
            changes.append(f'移除: {elem_type} ({text or "no text"})')

        return changes

    def _detect_visual_changes(
        self,
        before: Image.Image,
        after: Image.Image,
        region: Dict[str, int]
    ) -> List[str]:
        """检测特定区域的视觉变化"""
        changes = []

        try:
            # 裁剪区域
            before_region = before.crop((
                region['x'], region['y'],
                region['x'] + region['width'],
                region['y'] + region['height']
            ))
            after_region = after.crop((
                region['x'], region['y'],
                region['x'] + region['width'],
                region['y'] + region['height']
            ))

            # 比较像素差异
            diff_pixels = 0
            total_pixels = before_region.width * before_region.height

            before_pixels = list(before_region.getdata())
            after_pixels = list(after_region.getdata())

            for p1, p2 in zip(before_pixels, after_pixels):
                if p1 != p2:
                    diff_pixels += 1

            if diff_pixels > total_pixels * 0.05:  # 5%以上像素变化
                changes.append(f'视觉变化: {diff_pixels / total_pixels:.1%} 像素改变')

        except Exception as e:
            logger.warning(f'视觉变化检测失败: {e}')

        return changes

    def _find_focused_element(self, analysis: VisualAnalysis) -> Optional[str]:
        """查找当前聚焦的元素"""
        # 通过查找focus ring或outline样式来识别
        for elem in analysis.elements:
            if elem.attributes.get('state') == 'focused':
                return elem.text or elem.element_type
        return None

    def generate_report(self) -> InteractiveReport:
        """生成完整的交互测试报告"""
        total = len(self.results)
        passed = sum(1 for r in self.results if r.success)
        failed = total - passed

        # 分类统计
        mouse_tests = [r for r in self.results if 'click' in r.action or 'hover' in r.action or 'drag' in r.action]
        keyboard_tests = [r for r in self.results if 'hotkey' in r.action or 'navigation' in r.action]
        menu_tests = [r for r in self.results if 'menu' in r.action]
        form_tests = [r for r in self.results if 'input' in r.action or 'submit' in r.action]

        def calc_score(tests):
            if not tests:
                return 100.0
            return sum(1 for t in tests if t.success) / len(tests) * 100

        overall = calc_score(self.results) if self.results else 100.0

        return InteractiveReport(
            total_tests=total,
            passed=passed,
            failed=failed,
            results=self.results,
            mouse_score=calc_score(mouse_tests),
            keyboard_score=calc_score(keyboard_tests),
            menu_score=calc_score(menu_tests),
            form_score=calc_score(form_tests),
            overall_score=overall
        )
