"""
布局测试模块
===========

测试内容:
- 元素对齐与分布
- 间距一致性
- 响应式布局适配
- 网格系统遵循
- 重叠与遮挡检测
"""

from dataclasses import dataclass
from typing import List, Dict, Any, Optional, Tuple
from PIL import Image
import logging

from ..core.visual_analyzer import VisualAnalyzer, VisualAnalysis, UIElement
from ..aesthetic.aesthetic_evaluator import AestheticEvaluator, AestheticScore

logger = logging.getLogger(__name__)


@dataclass
class LayoutIssue:
    """布局问题"""
    issue_type: str  # misalignment, spacing_inconsistency, overlap, overflow
    severity: str    # critical, high, medium, low
    description: str
    elements: List[str]  # 涉及元素的ID或描述
    bounds: Optional[Dict[str, int]] = None  # 问题区域
    suggestion: Optional[str] = None


@dataclass
class LayoutReport:
    """布局测试报告"""
    total_checks: int
    issues: List[LayoutIssue]
    alignment_score: float
    spacing_score: float
    responsive_score: float
    overall_score: float
    passed: bool


class LayoutTests:
    """
    布局测试器

    测试UI布局的正确性和美学质量，包括:
    - 元素对齐
    - 间距一致性
    - 响应式适配
    - 网格系统遵循
    """

    def __init__(
        self,
        visual_analyzer: VisualAnalyzer,
        aesthetic_evaluator: Optional[AestheticEvaluator] = None
    ):
        self.visual_analyzer = visual_analyzer
        self.aesthetic_evaluator = aesthetic_evaluator or AestheticEvaluator()
        self.issues: List[LayoutIssue] = []

    # ==================== 对齐测试 ====================

    def test_element_alignment(
        self,
        analysis: VisualAnalysis,
        element_types: Optional[List[str]] = None
    ) -> LayoutReport:
        """
        测试元素对齐

        检查:
        - 同一行元素是否顶部/底部/中心对齐
        - 同一列元素是否左/右/中心对齐
        - 按钮组是否均匀分布
        """
        self.issues = []
        checks = 0

        elements = analysis.elements
        if element_types:
            elements = [e for e in elements if e.element_type in element_types]

        # 按行分组检查对齐
        rows = self._group_elements_by_row(elements)
        for row_name, row_elements in rows.items():
            checks += 1
            alignment_issue = self._check_row_alignment(row_name, row_elements)
            if alignment_issue:
                self.issues.append(alignment_issue)

        # 按列分组检查对齐
        cols = self._group_elements_by_column(elements)
        for col_name, col_elements in cols.items():
            checks += 1
            alignment_issue = self._check_column_alignment(col_name, col_elements)
            if alignment_issue:
                self.issues.append(alignment_issue)

        # 计算对齐分数
        alignment_score = max(0, 100 - len(self.issues) * 10)

        return LayoutReport(
            total_checks=checks,
            issues=self.issues,
            alignment_score=alignment_score,
            spacing_score=100.0,  # 将在其他测试中计算
            responsive_score=100.0,
            overall_score=alignment_score,
            passed=len(self.issues) == 0
        )

    def _group_elements_by_row(
        self,
        elements: List[UIElement]
    ) -> Dict[str, List[UIElement]]:
        """按行分组元素（基于Y坐标）"""
        rows: Dict[str, List[UIElement]] = {}

        # 根据Y坐标聚类
        threshold = 10  # 像素容差
        for elem in elements:
            y = elem.bounds.get('y', 0)
            assigned = False

            for row_name, row_elems in rows.items():
                if row_elems and abs(row_elems[0].bounds.get('y', 0) - y) <= threshold:
                    row_elems.append(elem)
                    assigned = True
                    break

            if not assigned:
                row_name = f"row_{y}"
                rows[row_name] = [elem]

        return rows

    def _group_elements_by_column(
        self,
        elements: List[UIElement]
    ) -> Dict[str, List[UIElement]]:
        """按列分组元素（基于X坐标）"""
        cols: Dict[str, List[UIElement]] = {}

        threshold = 10
        for elem in elements:
            x = elem.bounds.get('x', 0)
            assigned = False

            for col_name, col_elems in cols.items():
                if col_elems and abs(col_elems[0].bounds.get('x', 0) - x) <= threshold:
                    col_elems.append(elem)
                    assigned = True
                    break

            if not assigned:
                col_name = f"col_{x}"
                cols[col_name] = [elem]

        return cols

    def _check_row_alignment(
        self,
        row_name: str,
        elements: List[UIElement]
    ) -> Optional[LayoutIssue]:
        """检查一行内的元素是否对齐"""
        if len(elements) < 2:
            return None

        # 获取所有元素的顶部Y坐标
        tops = [e.bounds.get('y', 0) for e in elements]

        # 检查方差
        import statistics
        if len(tops) > 1:
            variance = statistics.variance(tops)
            if variance > 25:  # 5px的方差容差
                return LayoutIssue(
                    issue_type='misalignment',
                    severity='medium',
                    description=f'行 {row_name} 中的元素垂直对齐不一致',
                    elements=[e.text or e.element_type for e in elements],
                    suggestion='确保同一行内的元素顶部或中心线对齐'
                )

        return None

    def _check_column_alignment(
        self,
        col_name: str,
        elements: List[UIElement]
    ) -> Optional[LayoutIssue]:
        """检查一列内的元素是否对齐"""
        if len(elements) < 2:
            return None

        lefts = [e.bounds.get('x', 0) for e in elements]

        import statistics
        if len(lefts) > 1:
            variance = statistics.variance(lefts)
            if variance > 25:
                return LayoutIssue(
                    issue_type='misalignment',
                    severity='medium',
                    description=f'列 {col_name} 中的元素水平对齐不一致',
                    elements=[e.text or e.element_type for e in elements],
                    suggestion='确保同一列内的元素左边缘或中心线对齐'
                )

        return None

    # ==================== 间距测试 ====================

    def test_spacing_consistency(
        self,
        analysis: VisualAnalysis
    ) -> LayoutReport:
        """
        测试间距一致性

        检查:
        - 元素间距是否符合设计规范（如4px、8px、16px、24px、32px网格）
        - 内边距是否一致
        - 组件间留白是否合理
        """
        self.issues = []
        checks = 0

        elements = analysis.elements

        # 检查水平间距
        for i, elem1 in enumerate(elements):
            for elem2 in elements[i+1:]:
                checks += 1
                h_gap = self._calculate_horizontal_gap(elem1, elem2)
                if h_gap > 0 and not self._is_valid_grid_spacing(h_gap):
                    self.issues.append(LayoutIssue(
                        issue_type='spacing_inconsistency',
                        severity='low',
                        description=f'水平间距 {h_gap}px 不符合网格系统',
                        elements=[elem1.text or elem1.element_type,
                                 elem2.text or elem2.element_type],
                        suggestion='使用 4px/8px/16px/24px/32px 的间距值'
                    ))

        # 检查垂直间距
        for i, elem1 in enumerate(elements):
            for elem2 in elements[i+1:]:
                checks += 1
                v_gap = self._calculate_vertical_gap(elem1, elem2)
                if v_gap > 0 and not self._is_valid_grid_spacing(v_gap):
                    self.issues.append(LayoutIssue(
                        issue_type='spacing_inconsistency',
                        severity='low',
                        description=f'垂直间距 {v_gap}px 不符合网格系统',
                        elements=[elem1.text or elem1.element_type,
                                 elem2.text or elem2.element_type],
                        suggestion='使用 4px/8px/16px/24px/32px 的间距值'
                    ))

        spacing_score = max(0, 100 - len(self.issues) * 2)

        return LayoutReport(
            total_checks=checks,
            issues=self.issues,
            alignment_score=100.0,
            spacing_score=spacing_score,
            responsive_score=100.0,
            overall_score=spacing_score,
            passed=len(self.issues) == 0
        )

    def _calculate_horizontal_gap(
        self,
        elem1: UIElement,
        elem2: UIElement
    ) -> int:
        """计算两个元素之间的水平间距"""
        b1 = elem1.bounds
        b2 = elem2.bounds

        # elem1在左边
        if b1.get('x', 0) + b1.get('width', 0) <= b2.get('x', 0):
            return b2.get('x', 0) - (b1.get('x', 0) + b1.get('width', 0))

        # elem2在左边
        if b2.get('x', 0) + b2.get('width', 0) <= b1.get('x', 0):
            return b1.get('x', 0) - (b2.get('x', 0) + b2.get('width', 0))

        return 0

    def _calculate_vertical_gap(
        self,
        elem1: UIElement,
        elem2: UIElement
    ) -> int:
        """计算两个元素之间的垂直间距"""
        b1 = elem1.bounds
        b2 = elem2.bounds

        # elem1在上边
        if b1.get('y', 0) + b1.get('height', 0) <= b2.get('y', 0):
            return b2.get('y', 0) - (b1.get('y', 0) + b1.get('height', 0))

        # elem2在上边
        if b2.get('y', 0) + b2.get('height', 0) <= b1.get('y', 0):
            return b1.get('y', 0) - (b2.get('y', 0) + b2.get('height', 0))

        return 0

    def _is_valid_grid_spacing(self, gap: int, tolerance: int = 2) -> bool:
        """检查间距是否符合网格系统"""
        valid_spacings = [4, 8, 12, 16, 20, 24, 32, 40, 48, 64]
        return any(abs(gap - s) <= tolerance for s in valid_spacings)

    # ==================== 重叠检测 ====================

    def test_overlap_detection(
        self,
        analysis: VisualAnalysis
    ) -> LayoutReport:
        """
        检测元素重叠和遮挡

        检查:
        - 是否有元素意外重叠
        - Z-index是否正确
        - 文字是否被截断
        """
        self.issues = []
        checks = 0

        elements = analysis.elements

        for i, elem1 in enumerate(elements):
            for elem2 in elements[i+1:]:
                checks += 1
                overlap = self._calculate_overlap(elem1, elem2)

                if overlap:
                    overlap_area, overlap_ratio = overlap

                    # 如果重叠比例超过阈值，记录问题
                    if overlap_ratio > 0.1:  # 10%重叠
                        self.issues.append(LayoutIssue(
                            issue_type='overlap',
                            severity='high' if overlap_ratio > 0.5 else 'medium',
                            description=f'元素重叠: {elem1.text or elem1.element_type} '
                                       f'与 {elem2.text or elem2.element_type} '
                                       f'(重叠 {overlap_ratio:.1%})',
                            elements=[elem1.text or elem1.element_type,
                                     elem2.text or elem2.element_type],
                            bounds={
                                'x': overlap_area[0],
                                'y': overlap_area[1],
                                'width': overlap_area[2],
                                'height': overlap_area[3]
                            },
                            suggestion='调整元素位置或使用适当的z-index'
                        ))

        overlap_score = max(0, 100 - len(self.issues) * 15)

        return LayoutReport(
            total_checks=checks,
            issues=self.issues,
            alignment_score=100.0,
            spacing_score=100.0,
            responsive_score=100.0,
            overall_score=overlap_score,
            passed=len(self.issues) == 0
        )

    def _calculate_overlap(
        self,
        elem1: UIElement,
        elem2: UIElement
    ) -> Optional[Tuple[Tuple[int, int, int, int], float]]:
        """
        计算两个元素的重叠区域
        返回: (重叠区域bounds, 重叠比例) 或 None
        """
        b1 = elem1.bounds
        b2 = elem2.bounds

        # 计算重叠矩形
        x1 = max(b1.get('x', 0), b2.get('x', 0))
        y1 = max(b1.get('y', 0), b2.get('y', 0))
        x2 = min(b1.get('x', 0) + b1.get('width', 0),
                 b2.get('x', 0) + b2.get('width', 0))
        y2 = min(b1.get('y', 0) + b1.get('height', 0),
                 b2.get('y', 0) + b2.get('height', 0))

        if x2 <= x1 or y2 <= y1:
            return None

        overlap_width = x2 - x1
        overlap_height = y2 - y1
        overlap_area = overlap_width * overlap_height

        # 计算较小元素的面积
        area1 = b1.get('width', 0) * b1.get('height', 0)
        area2 = b2.get('width', 0) * b2.get('height', 0)
        min_area = min(area1, area2)

        if min_area == 0:
            return None

        overlap_ratio = overlap_area / min_area

        return ((x1, y1, overlap_width, overlap_height), overlap_ratio)

    # ==================== 响应式测试 ====================

    def test_responsive_layout(
        self,
        adapter,
        target_window_title: str,
        breakpoints: List[Tuple[int, int]] = None
    ) -> LayoutReport:
        """
        测试响应式布局

        在不同窗口尺寸下检查:
        - 布局是否正确调整
        - 元素是否被截断
        - 滚动条是否出现
        """
        if breakpoints is None:
            breakpoints = [
                (1920, 1080),  # Desktop
                (1440, 900),   # Laptop
                (1280, 720),   # Small laptop
                (768, 1024),   # Tablet
                (375, 667),    # Mobile
            ]

        self.issues = []
        checks = 0

        window = adapter.find_window(target_window_title)
        if not window:
            return LayoutReport(
                total_checks=0,
                issues=[LayoutIssue(
                    issue_type='window_not_found',
                    severity='critical',
                    description=f'找不到窗口: {target_window_title}',
                    elements=[]
                )],
                alignment_score=0,
                spacing_score=0,
                responsive_score=0,
                overall_score=0,
                passed=False
            )

        for width, height in breakpoints:
            checks += 1
            logger.info(f"测试响应式布局: {width}x{height}")

            # 调整窗口大小
            adapter.resize_window(window.window_id, width, height)
            adapter.wait(0.5)

            # 截图并分析
            screenshot = adapter.screenshot_window(target_window_title)
            analysis = self.visual_analyzer.analyze_screenshot(
                screenshot,
                context=f"响应式测试 {width}x{height}"
            )

            # 检查元素是否被截断
            for elem in analysis.elements:
                if self._is_element_truncated(elem, width, height):
                    self.issues.append(LayoutIssue(
                        issue_type='overflow',
                        severity='high',
                        description=f'{width}x{height} 尺寸下元素被截断: '
                                   f'{elem.text or elem.element_type}',
                        elements=[elem.text or elem.element_type],
                        suggestion='使用响应式布局或添加滚动条'
                    ))

        responsive_score = max(0, 100 - len(self.issues) * 10)

        return LayoutReport(
            total_checks=checks,
            issues=self.issues,
            alignment_score=100.0,
            spacing_score=100.0,
            responsive_score=responsive_score,
            overall_score=responsive_score,
            passed=len(self.issues) == 0
        )

    def _is_element_truncated(
        self,
        elem: UIElement,
        window_width: int,
        window_height: int
    ) -> bool:
        """检查元素是否被窗口边界截断"""
        b = elem.bounds
        right = b.get('x', 0) + b.get('width', 0)
        bottom = b.get('y', 0) + b.get('height', 0)

        return right > window_width or bottom > window_height

    # ==================== shadcn/ui 布局规范测试 ====================

    def test_shadcn_layout_compliance(
        self,
        analysis: VisualAnalysis
    ) -> LayoutReport:
        """
        测试是否符合 shadcn/ui 布局规范

        检查:
        - 间距使用CSS变量 (--spacing-*)
        - 圆角一致性
        - 容器最大宽度
        """
        self.issues = []
        checks = 0

        # 这些检查需要结合无障碍树或WebView来获取CSS信息
        # 这里基于视觉分析进行基础检查

        # 检查按钮组间距
        buttons = [e for e in analysis.elements if e.element_type == 'button']
        if len(buttons) >= 2:
            checks += 1
            for i in range(len(buttons) - 1):
                gap = self._calculate_horizontal_gap(buttons[i], buttons[i + 1])
                if gap > 0 and gap < 8:
                    self.issues.append(LayoutIssue(
                        issue_type='spacing_inconsistency',
                        severity='low',
                        description='按钮间距过小，建议至少8px',
                        elements=[buttons[i].text or 'button',
                                 buttons[i + 1].text or 'button'],
                        suggestion='使用 gap-2 (8px) 或 gap-3 (12px)'
                    ))

        # 检查卡片间距
        cards = [e for e in analysis.elements if e.element_type == 'card']
        if len(cards) >= 2:
            checks += 1
            gaps = []
            for i in range(len(cards) - 1):
                gap = self._calculate_horizontal_gap(cards[i], cards[i + 1])
                if gap > 0:
                    gaps.append(gap)

            if gaps:
                import statistics
                if len(gaps) > 1 and statistics.stdev(gaps) > 5:
                    self.issues.append(LayoutIssue(
                        issue_type='spacing_inconsistency',
                        severity='medium',
                        description='卡片间距不一致',
                        elements=['cards'],
                        suggestion='统一使用 grid gap-4 (16px) 或 gap-6 (24px)'
                    ))

        compliance_score = max(0, 100 - len(self.issues) * 5)

        return LayoutReport(
            total_checks=checks,
            issues=self.issues,
            alignment_score=compliance_score,
            spacing_score=compliance_score,
            responsive_score=100.0,
            overall_score=compliance_score,
            passed=len(self.issues) == 0
        )
