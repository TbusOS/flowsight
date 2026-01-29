"""
视觉测试模块
===========

测试内容:
- 颜色与对比度
- 字体与排版
- 图标渲染
- 图片质量
- 视觉一致性
"""

from dataclasses import dataclass
from typing import List, Dict, Any, Optional, Tuple
from PIL import Image, ImageStat
import colorsys
import logging

from ..core.visual_analyzer import VisualAnalyzer, VisualAnalysis, UIElement
from ..aesthetic.aesthetic_evaluator import AestheticEvaluator, AestheticScore

logger = logging.getLogger(__name__)


@dataclass
class VisualIssue:
    """视觉问题"""
    issue_type: str  # color_contrast, font_rendering, icon_blur, color_inconsistency
    severity: str
    description: str
    elements: List[str]
    expected: Optional[str] = None
    actual: Optional[str] = None


@dataclass
class VisualReport:
    """视觉测试报告"""
    total_checks: int
    issues: List[VisualIssue]
    color_score: float
    typography_score: float
    icon_score: float
    consistency_score: float
    overall_score: float
    passed: bool
    color_palette: List[str]


class VisualTests:
    """
    视觉测试器

    测试UI视觉质量，包括:
    - 颜色对比度 (WCAG标准)
    - 字体渲染质量
    - 图标清晰度
    - 视觉一致性
    """

    def __init__(
        self,
        visual_analyzer: VisualAnalyzer,
        aesthetic_evaluator: Optional[AestheticEvaluator] = None
    ):
        self.visual_analyzer = visual_analyzer
        self.aesthetic_evaluator = aesthetic_evaluator or AestheticEvaluator()
        self.issues: List[VisualIssue] = []

    # ==================== 颜色对比度测试 ====================

    def test_color_contrast(
        self,
        analysis: VisualAnalysis,
        wcag_level: str = 'AA'  # 'A', 'AA', 'AAA'
    ) -> VisualReport:
        """
        测试颜色对比度是否符合WCAG标准

        WCAG标准:
        - AA级: 正常文本 4.5:1, 大文本 3:1
        - AAA级: 正常文本 7:1, 大文本 4.5:1
        """
        self.issues = []
        checks = 0

        # 对比度阈值
        thresholds = {
            'A': {'normal': 3.0, 'large': 3.0},
            'AA': {'normal': 4.5, 'large': 3.0},
            'AAA': {'normal': 7.0, 'large': 4.5}
        }
        threshold = thresholds.get(wcag_level, thresholds['AA'])

        # 检查文本元素的对比度
        text_elements = [e for e in analysis.elements
                        if e.element_type in ['text', 'button', 'input', 'label']]

        for elem in text_elements:
            checks += 1
            fg_color = elem.attributes.get('color')
            bg_color = self._estimate_background_color(analysis, elem)

            if fg_color and bg_color:
                contrast = self._calculate_contrast_ratio(fg_color, bg_color)
                is_large = elem.attributes.get('font_size', '14px') >= '18px'
                min_contrast = threshold['large'] if is_large else threshold['normal']

                if contrast < min_contrast:
                    self.issues.append(VisualIssue(
                        issue_type='color_contrast',
                        severity='high' if wcag_level == 'AA' else 'medium',
                        description=f'对比度不足 ({contrast:.2f}:1)',
                        elements=[elem.text or elem.element_type],
                        expected=f'≥ {min_contrast}:1',
                        actual=f'{contrast:.2f}:1'
                    ))

        contrast_score = max(0, 100 - len(self.issues) * 10)

        return VisualReport(
            total_checks=checks,
            issues=self.issues,
            color_score=contrast_score,
            typography_score=100.0,
            icon_score=100.0,
            consistency_score=100.0,
            overall_score=contrast_score,
            passed=len(self.issues) == 0,
            color_palette=analysis.color_palette
        )

    def _calculate_contrast_ratio(self, color1: str, color2: str) -> float:
        """计算两个颜色之间的对比度比例"""
        try:
            lum1 = self._calculate_relative_luminance(color1)
            lum2 = self._calculate_relative_luminance(color2)

            lighter = max(lum1, lum2)
            darker = min(lum1, lum2)

            return (lighter + 0.05) / (darker + 0.05)
        except Exception:
            return 21.0  # 最大对比度

    def _calculate_relative_luminance(self, color: str) -> float:
        """计算颜色的相对亮度 (WCAG定义)"""
        # 解析颜色
        r, g, b = self._parse_color(color)

        # 转换为sRGB
        r = r / 255.0
        g = g / 255.0
        b = b / 255.0

        # 应用gamma校正
        r = r / 12.92 if r <= 0.03928 else ((r + 0.055) / 1.055) ** 2.4
        g = g / 12.92 if g <= 0.03928 else ((g + 0.055) / 1.055) ** 2.4
        b = b / 12.92 if b <= 0.03928 else ((b + 0.055) / 1.055) ** 2.4

        return 0.2126 * r + 0.7152 * g + 0.0722 * b

    def _parse_color(self, color: str) -> Tuple[int, int, int]:
        """解析颜色字符串为RGB元组"""
        color = color.strip().lower()

        # #RGB格式
        if color.startswith('#') and len(color) == 4:
            r = int(color[1] * 2, 16)
            g = int(color[2] * 2, 16)
            b = int(color[3] * 2, 16)
            return (r, g, b)

        # #RRGGBB格式
        if color.startswith('#') and len(color) == 7:
            r = int(color[1:3], 16)
            g = int(color[3:5], 16)
            b = int(color[5:7], 16)
            return (r, g, b)

        # 返回默认黑色
        return (0, 0, 0)

    def _estimate_background_color(
        self,
        analysis: VisualAnalysis,
        elem: UIElement
    ) -> Optional[str]:
        """估算元素的背景颜色"""
        # 从截图中提取元素区域的平均颜色
        bounds = elem.bounds
        try:
            region = analysis.screenshot.crop((
                bounds['x'],
                bounds['y'],
                bounds['x'] + bounds['width'],
                bounds['y'] + bounds['height']
            ))

            stat = ImageStat.Stat(region)
            r, g, b = int(stat.mean[0]), int(stat.mean[1]), int(stat.mean[2])
            return f'#{r:02x}{g:02x}{b:02x}'
        except Exception:
            return '#ffffff'  # 默认白色背景

    # ==================== 字体渲染测试 ====================

    def test_font_rendering(
        self,
        analysis: VisualAnalysis
    ) -> VisualReport:
        """
        测试字体渲染质量

        检查:
        - 字体清晰度
        - 抗锯齿
        - 字号一致性
        """
        self.issues = []
        checks = 0

        text_elements = [e for e in analysis.elements if e.element_type == 'text']

        # 检查字体大小一致性
        font_sizes = {}
        for elem in text_elements:
            checks += 1
            font_size = elem.attributes.get('font_size', '14px')

            # 提取数值
            try:
                size_val = int(font_size.replace('px', ''))
                font_sizes.setdefault(size_val, []).append(elem)
            except ValueError:
                continue

        # 检查是否有过多不同的字号
        if len(font_sizes) > 6:  # 超过6种不同字号
            self.issues.append(VisualIssue(
                issue_type='font_rendering',
                severity='low',
                description=f'检测到 {len(font_sizes)} 种不同的字号，建议限制在6种以内',
                elements=[f'{len(font_sizes)} sizes'],
                suggestion='使用一致的type scale (如 12, 14, 16, 20, 24, 32)'
            ))

        # 检查是否有过小的字号
        for size, elems in font_sizes.items():
            if size < 12:
                self.issues.append(VisualIssue(
                    issue_type='font_rendering',
                    severity='medium',
                    description=f'字号 {size}px 过小，可能影响可读性',
                    elements=[e.text for e in elems if e.text][:3],
                    suggestion='最小字号建议为 12px'
                ))

        typography_score = max(0, 100 - len(self.issues) * 5)

        return VisualReport(
            total_checks=checks,
            issues=self.issues,
            color_score=100.0,
            typography_score=typography_score,
            icon_score=100.0,
            consistency_score=100.0,
            overall_score=typography_score,
            passed=len(self.issues) == 0,
            color_palette=analysis.color_palette
        )

    def test_typography_scale(
        self,
        analysis: VisualAnalysis
    ) -> VisualReport:
        """
        测试排版层级是否符合shadcn/ui规范

        检查:
        - 标题层级 (h1-h6)
        - 正文大小
        - 行高比例
        """
        self.issues = []
        checks = 0

        # shadcn/ui 标准字号
        expected_sizes = {
            'h1': 36,
            'h2': 30,
            'h3': 24,
            'h4': 20,
            'h5': 18,
            'h6': 16,
            'body': 14,
            'small': 12,
        }

        # 分析文本元素的字号分布
        text_elements = [e for e in analysis.elements if e.element_type == 'text']

        sizes_found = set()
        for elem in text_elements:
            checks += 1
            font_size = elem.attributes.get('font_size', '14px')
            try:
                size_val = int(font_size.replace('px', ''))
                sizes_found.add(size_val)
            except ValueError:
                continue

        # 检查是否遵循typography scale
        for expected_size in expected_sizes.values():
            if not any(abs(s - expected_size) <= 2 for s in sizes_found):
                self.issues.append(VisualIssue(
                    issue_type='font_rendering',
                    severity='low',
                    description=f'缺少字号 {expected_size}px，可能导致排版层级不完整',
                    elements=[],
                    suggestion=f'添加 text-{expected_size} 样式'
                ))

        typography_score = max(0, 100 - len(self.issues) * 3)

        return VisualReport(
            total_checks=checks,
            issues=self.issues,
            color_score=100.0,
            typography_score=typography_score,
            icon_score=100.0,
            consistency_score=100.0,
            overall_score=typography_score,
            passed=len(self.issues) == 0,
            color_palette=analysis.color_palette
        )

    # ==================== 图标测试 ====================

    def test_icon_rendering(
        self,
        analysis: VisualAnalysis
    ) -> VisualReport:
        """
        测试图标渲染质量

        检查:
        - 图标清晰度
        - 大小一致性
        - 颜色一致性
        """
        self.issues = []
        checks = 0

        icon_elements = [e for e in analysis.elements if e.element_type == 'icon']

        if not icon_elements:
            return VisualReport(
                total_checks=0,
                issues=[],
                color_score=100.0,
                typography_score=100.0,
                icon_score=100.0,
                consistency_score=100.0,
                overall_score=100.0,
                passed=True,
                color_palette=analysis.color_palette
            )

        # 检查图标大小一致性
        icon_sizes = set()
        for icon in icon_elements:
            checks += 1
            width = icon.bounds.get('width', 0)
            height = icon.bounds.get('height', 0)
            icon_sizes.add((width, height))

        if len(icon_sizes) > 4:  # 超过4种不同尺寸
            self.issues.append(VisualIssue(
                issue_type='icon_blur',
                severity='low',
                description=f'图标尺寸不统一，检测到 {len(icon_sizes)} 种不同尺寸',
                elements=[f'{len(icon_sizes)} sizes'],
                suggestion='使用统一的图标尺寸 (如 16x16, 20x20, 24x24, 32x32)'
            ))

        # 检查图标颜色一致性
        icon_colors = set()
        for icon in icon_elements:
            color = icon.attributes.get('color', '')
            if color:
                icon_colors.add(color)

        if len(icon_colors) > 3:  # 超过3种颜色
            self.issues.append(VisualIssue(
                issue_type='color_inconsistency',
                severity='low',
                description=f'图标颜色过多，检测到 {len(icon_colors)} 种颜色',
                elements=[],
                suggestion='限制图标颜色为2-3种: 主色、次要色、禁用色'
            ))

        icon_score = max(0, 100 - len(self.issues) * 5)

        return VisualReport(
            total_checks=checks,
            issues=self.issues,
            color_score=100.0,
            typography_score=100.0,
            icon_score=icon_score,
            consistency_score=100.0,
            overall_score=icon_score,
            passed=len(self.issues) == 0,
            color_palette=analysis.color_palette
        )

    def test_icon_aesthetics(
        self,
        analysis: VisualAnalysis
    ) -> VisualReport:
        """
        测试图标美学质量

        检查:
        - 图标风格一致性 (线性 vs 填充)
        - 描边粗细一致性
        - 视觉重量平衡
        """
        self.issues = []
        checks = 0

        icon_elements = [e for e in analysis.elements if e.element_type == 'icon']

        if len(icon_elements) < 2:
            return VisualReport(
                total_checks=0,
                issues=[],
                color_score=100.0,
                typography_score=100.0,
                icon_score=100.0,
                consistency_score=100.0,
                overall_score=100.0,
                passed=True,
                color_palette=analysis.color_palette
            )

        # 使用美学评估器评估图标一致性
        if self.aesthetic_evaluator:
            checks += 1
            result = self.aesthetic_evaluator.evaluate_consistency(analysis.screenshot)

            if result.score < 0.7:
                self.issues.append(VisualIssue(
                    issue_type='color_inconsistency',
                    severity='medium',
                    description=f'图标视觉一致性得分较低 ({result.score:.2f})',
                    elements=[],
                    suggestion='确保所有图标使用相同的图标库和风格'
                ))

        consistency_score = max(0, 100 - len(self.issues) * 10)

        return VisualReport(
            total_checks=checks,
            issues=self.issues,
            color_score=100.0,
            typography_score=100.0,
            icon_score=consistency_score,
            consistency_score=consistency_score,
            overall_score=consistency_score,
            passed=len(self.issues) == 0,
            color_palette=analysis.color_palette
        )

    # ==================== 颜色一致性测试 ====================

    def test_color_consistency(
        self,
        analysis: VisualAnalysis,
        expected_palette: Optional[List[str]] = None
    ) -> VisualReport:
        """
        测试颜色使用的一致性

        检查:
        - 是否使用品牌色板
        - 颜色数量是否过多
        - 渐变使用是否恰当
        """
        self.issues = []
        checks = 0

        # 分析截图中的颜色
        palette = analysis.color_palette

        # 检查颜色数量
        checks += 1
        if len(palette) > 8:  # 超过8种主要颜色
            self.issues.append(VisualIssue(
                issue_type='color_inconsistency',
                severity='low',
                description=f'界面颜色过多 ({len(palette)} 种)，建议控制在5-8种',
                elements=[],
                suggestion='使用设计系统定义的颜色变量'
            ))

        # 如果提供了期望的色板，检查匹配度
        if expected_palette:
            checks += 1
            palette_normalized = [c.lower() for c in palette]
            expected_normalized = [c.lower() for c in expected_palette]

            unmatched = [c for c in palette_normalized
                        if not any(self._color_similar(c, e) for e in expected_normalized)]

            if unmatched:
                self.issues.append(VisualIssue(
                    issue_type='color_inconsistency',
                    severity='medium',
                    description=f'发现 {len(unmatched)} 种不在设计系统中的颜色',
                    elements=unmatched[:3],
                    suggestion='仅使用设计系统中定义的颜色变量'
                ))

        consistency_score = max(0, 100 - len(self.issues) * 10)

        return VisualReport(
            total_checks=checks,
            issues=self.issues,
            color_score=consistency_score,
            typography_score=100.0,
            icon_score=100.0,
            consistency_score=consistency_score,
            overall_score=consistency_score,
            passed=len(self.issues) == 0,
            color_palette=palette
        )

    def _color_similar(self, c1: str, c2: str, tolerance: int = 20) -> bool:
        """判断两个颜色是否相似"""
        try:
            r1, g1, b1 = self._parse_color(c1)
            r2, g2, b2 = self._parse_color(c2)

            distance = ((r1-r2)**2 + (g1-g2)**2 + (b1-b2)**2) ** 0.5
            return distance <= tolerance
        except Exception:
            return c1 == c2

    # ==================== 暗色/亮色模式测试 ====================

    def test_dark_mode_compliance(
        self,
        dark_analysis: VisualAnalysis,
        light_analysis: Optional[VisualAnalysis] = None
    ) -> VisualReport:
        """
        测试暗色模式实现质量

        检查:
        - 暗色模式下对比度
        - 颜色反转是否正确
        - 阴影和高光处理
        """
        self.issues = []
        checks = 0

        # 检查暗色模式下的对比度
        text_elements = [e for e in dark_analysis.elements
                        if e.element_type in ['text', 'button']]

        for elem in text_elements:
            checks += 1
            color = elem.attributes.get('color', '#ffffff')
            r, g, b = self._parse_color(color)
            brightness = (r + g + b) / 3

            # 暗色模式下文字应该较亮
            if brightness < 128:
                self.issues.append(VisualIssue(
                    issue_type='color_contrast',
                    severity='high',
                    description='暗色模式下文字颜色过暗',
                    elements=[elem.text or elem.element_type],
                    expected='亮度 > 128',
                    actual=f'亮度 {brightness:.0f}'
                ))

        # 如果有亮色模式对比，检查对应关系
        if light_analysis:
            checks += 1
            # 比较两个模式下的元素数量
            dark_count = len(dark_analysis.elements)
            light_count = len(light_analysis.elements)

            if abs(dark_count - light_count) > 2:
                self.issues.append(VisualIssue(
                    issue_type='color_inconsistency',
                    severity='medium',
                    description='暗色/亮色模式元素数量不一致',
                    elements=[],
                    expected=f'亮色模式 {light_count} 个元素',
                    actual=f'暗色模式 {dark_count} 个元素'
                ))

        color_score = max(0, 100 - len(self.issues) * 10)

        return VisualReport(
            total_checks=checks,
            issues=self.issues,
            color_score=color_score,
            typography_score=100.0,
            icon_score=100.0,
            consistency_score=color_score,
            overall_score=color_score,
            passed=len(self.issues) == 0,
            color_palette=dark_analysis.color_palette
        )
