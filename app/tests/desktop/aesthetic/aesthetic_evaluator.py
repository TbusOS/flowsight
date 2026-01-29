"""
视觉美学评估器
基于 Cursor / macOS / 21st.dev / shadcn-ui 设计规范的审美判断
"""

from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any, Tuple
from PIL import Image
import base64
import io
import json
import logging
import os
import colorsys
from collections import Counter

logger = logging.getLogger(__name__)


@dataclass
class AestheticScore:
    """美学评分"""
    category: str  # 评分类别
    score: float  # 0-100
    details: Dict[str, Any] = field(default_factory=dict)
    suggestions: List[str] = field(default_factory=list)

    def __repr__(self):
        return f"{self.category}: {self.score:.1f}/100"


@dataclass
class AestheticReport:
    """美学评估报告"""
    overall_score: float
    scores: List[AestheticScore]
    summary: str
    design_system_alignment: Dict[str, float]  # cursor, macos, 21st, shadcn
    screenshots_with_annotations: List[Image.Image] = field(default_factory=list)
    raw_analysis: Dict[str, Any] = field(default_factory=dict)

    def get_score_by_category(self, category: str) -> Optional[AestheticScore]:
        """获取特定类别的评分"""
        for score in self.scores:
            if score.category.lower() == category.lower():
                return score
        return None


class AestheticEvaluator:
    """
    视觉美学评估器

    设计理念对标：
    - Cursor: 深色沉浸、代码优先、最小干扰
    - macOS: 毛玻璃效果、圆角一致、精致动画
    - 21st.dev: 现代极简、留白呼吸感、排版精致
    - shadcn-ui: 组件原子化、可组合、主题一致
    """

    # 设计系统特征定义
    DESIGN_SYSTEMS = {
        "cursor": {
            "description": "深色沉浸式代码编辑器",
            "key_traits": ["深色主题", "高对比度", "聚焦状态", "最小干扰"],
            "color_palette": ["#0a0a0b", "#141416", "#2a2a2c", "#3e3e42", "#ffffff", "#4fc1ff"],
            "typography": "JetBrains Mono / SF Mono",
            "spacing": "紧凑但有呼吸感",
        },
        "macos": {
            "description": "苹果原生设计语言",
            "key_traits": ["毛玻璃效果", "圆角一致", "精致动画", "层次清晰"],
            "color_palette": ["#000000", "#1c1c1e", "#2c2c2e", "#ffffff", "#0a84ff", "#34c759"],
            "typography": "SF Pro / SF Pro Display",
            "spacing": "舒适、有节奏",
        },
        "21st": {
            "description": "现代极简主义",
            "key_traits": ["留白呼吸感", "排版精致", "几何简洁", "视觉平衡"],
            "color_palette": ["#fafafa", "#f5f5f5", "#e5e5e5", "#171717", "#6366f1"],
            "typography": "Inter / Geist",
            "spacing": "大量留白",
        },
        "shadcn": {
            "description": "原子化组件系统",
            "key_traits": ["组件原子化", "可组合", "主题一致", "4px网格"],
            "color_palette": ["hsl(var(--background))", "hsl(var(--foreground))"],
            "typography": "Geist / Inter",
            "spacing": "4px倍数系统",
        }
    }

    # AI 审美评估提示词
    AESTHETIC_PROMPT = """
你是一位资深 UI/UX 设计专家，精通 Cursor、macOS、21st.dev、shadcn-ui 等现代设计语言。

请对这张界面截图进行专业的美学评估，从以下几个维度打分（0-100分）：

1. **极简主义 (Minimalism)**
   - 元素密度是否合适（不拥挤）
   - 是否存在多余装饰
   - 留白是否充足

2. **排版美学 (Typography)**
   - 字体层级清晰（H1 > H2 > body > small）
   - 行高舒适（1.5-1.7）
   - 字间距恰当
   - 中西文混排协调

3. **色彩和谐 (Color Harmony)**
   - 主色/辅色比例（60-30-10 法则）
   - 色彩过渡自然
   - 主题切换一致性
   - 对比度符合 WCAG 标准

4. **视觉层级 (Visual Hierarchy)**
   - 重要元素突出
   - 次要元素弱化
   - 视线引导清晰
   - F型/Z型阅读流

5. **设计一致性 (Consistency)**
   - 圆角统一（shadcn-ui: --radius）
   - 间距阶梯（4px 倍数系统）
   - 组件样式统一
   - 图标风格一致

6. **图标美学 (Icon Aesthetics)**
   - 描边一致性（1.5px-2px）
   - 尺寸和谐（16/20/24px）
   - 风格统一（线性/填充）

请按以下 JSON 格式输出：

{
    "overall_score": 85,
    "scores": [
        {
            "category": "minimalism",
            "score": 88,
            "details": {"element_density": "适中", "white_space": "充足"},
            "suggestions": ["可以进一步简化工具栏图标"]
        },
        {
            "category": "typography",
            "score": 82,
            "details": {"font_stack": "Geist", "hierarchy": "清晰", "line_height": "1.6"},
            "suggestions": ["标题字重可以稍微加重"]
        },
        {
            "category": "color_harmony",
            "score": 85,
            "details": {"primary_ratio": 10, "secondary_ratio": 30, "neutral_ratio": 60},
            "suggestions": []
        },
        {
            "category": "visual_hierarchy",
            "score": 87,
            "details": {"focus_clear": true, "guidance_strong": true},
            "suggestions": []
        },
        {
            "category": "consistency",
            "score": 90,
            "details": {"border_radius": "统一8px", "spacing_grid": "4px"},
            "suggestions": []
        },
        {
            "category": "icon_aesthetics",
            "score": 78,
            "details": {"stroke_width": "1.5px", "size_consistency": "较好"},
            "suggestions": ["部分图标大小不一致"]
        }
    ],
    "design_system_alignment": {
        "cursor": 0.85,
        "macos": 0.70,
        "21st": 0.80,
        "shadcn": 0.90
    },
    "summary": "整体设计风格统一，符合现代极简主义。建议优化图标一致性。",
    "strengths": ["色彩和谐", "视觉层级清晰"],
    "weaknesses": ["图标大小不够统一"]
}
"""

    def __init__(self, api_key: Optional[str] = None):
        self.api_key = api_key or os.environ.get("ANTHROPIC_API_KEY")
        self._client = None
        self._analysis_cache: Dict[str, AestheticReport] = {}

    def _get_client(self):
        """获取 Claude API 客户端"""
        if self._client is None:
            try:
                import anthropic
                self._client = anthropic.Anthropic(api_key=self.api_key)
            except ImportError:
                logger.error("需要安装 anthropic: pip install anthropic")
                raise
        return self._client

    def _image_to_base64(self, image: Image.Image) -> str:
        """将图片转换�� base64"""
        image_bytes = io.BytesIO()
        image.save(image_bytes, format='PNG')
        return base64.b64encode(image_bytes.getvalue()).decode()

    def evaluate(self, screenshot: Image.Image, context: Optional[str] = None) -> AestheticReport:
        """
        对截图进行全面的美学评估

        Args:
            screenshot: 界面截图
            context: 上下文信息

        Returns:
            AestheticReport 美学评估报告
        """
        image_base64 = self._image_to_base64(screenshot)

        prompt = self.AESTHETIC_PROMPT
        if context:
            prompt = f"上下文: {context}\n\n{prompt}"

        try:
            client = self._get_client()
            message = client.messages.create(
                model="claude-3-opus-20240229",
                max_tokens=4096,
                messages=[
                    {
                        "role": "user",
                        "content": [
                            {
                                "type": "image",
                                "source": {
                                    "type": "base64",
                                    "media_type": "image/png",
                                    "data": image_base64
                                }
                            },
                            {
                                "type": "text",
                                "text": prompt
                            }
                        ]
                    }
                ]
            )

            response_text = message.content[0].text if message.content else ""
            result_data = self._parse_json_response(response_text)

            # 构建评分列表
            scores = []
            for score_data in result_data.get("scores", []):
                scores.append(AestheticScore(
                    category=score_data.get("category", "unknown"),
                    score=score_data.get("score", 0),
                    details=score_data.get("details", {}),
                    suggestions=score_data.get("suggestions", [])
                ))

            report = AestheticReport(
                overall_score=result_data.get("overall_score", 0),
                scores=scores,
                summary=result_data.get("summary", ""),
                design_system_alignment=result_data.get("design_system_alignment", {}),
                raw_analysis=result_data
            )

            return report

        except Exception as e:
            logger.error(f"美学评估失败: {e}")
            # 返回默认报告
            return AestheticReport(
                overall_score=0,
                scores=[],
                summary=f"评估失败: {e}",
                design_system_alignment={}
            )

    def _parse_json_response(self, response: str) -> Dict:
        """从响应文本中提取 JSON 数据"""
        try:
            return json.loads(response)
        except json.JSONDecodeError:
            import re
            json_match = re.search(r'```json\s*(.*?)\s*```', response, re.DOTALL)
            if json_match:
                return json.loads(json_match.group(1))
            json_match = re.search(r'\{.*\}', response, re.DOTALL)
            if json_match:
                try:
                    return json.loads(json_match.group(0))
                except json.JSONDecodeError:
                    pass
            return {}

    def evaluate_minimalism(self, screenshot: Image.Image) -> AestheticScore:
        """
        极简主义评分
        """
        report = self.evaluate(screenshot, "专注于评估极简主义设计")
        return report.get_score_by_category("minimalism") or AestheticScore(
            category="minimalism", score=0, details={}
        )

    def evaluate_typography(self, screenshot: Image.Image) -> AestheticScore:
        """
        排版美学评分
        """
        report = self.evaluate(screenshot, "专注于评估排版设计")
        return report.get_score_by_category("typography") or AestheticScore(
            category="typography", score=0, details={}
        )

    def evaluate_color_harmony(self, screenshot: Image.Image) -> AestheticScore:
        """
        色彩和谐评分
        """
        report = self.evaluate(screenshot, "专注于评估色彩和谐")
        return report.get_score_by_category("color_harmony") or AestheticScore(
            category="color_harmony", score=0, details={}
        )

    def evaluate_visual_hierarchy(self, screenshot: Image.Image) -> AestheticScore:
        """
        视觉层级评分
        """
        report = self.evaluate(screenshot, "专注于评估视觉层级")
        return report.get_score_by_category("visual_hierarchy") or AestheticScore(
            category="visual_hierarchy", score=0, details={}
        )

    def evaluate_consistency(self, screenshot: Image.Image) -> AestheticScore:
        """
        设计一致性评分
        """
        report = self.evaluate(screenshot, "专注于评估设计一致性")
        return report.get_score_by_category("consistency") or AestheticScore(
            category="consistency", score=0, details={}
        )

    def evaluate_icon_aesthetics(self, screenshot: Image.Image) -> AestheticScore:
        """
        图标美学评分
        """
        report = self.evaluate(screenshot, "专注于评估图标美学")
        return report.get_score_by_category("icon_aesthetics") or AestheticScore(
            category="icon_aesthetics", score=0, details={}
        )

    def extract_color_palette(self, screenshot: Image.Image, num_colors: int = 8) -> List[str]:
        """
        提取截图的主色调

        Args:
            screenshot: 截图
            num_colors: 提取颜色数量

        Returns:
            颜色列表（十六进制）
        """
        try:
            # 缩小图片以提高性能
            small = screenshot.resize((150, 150))
            # 转换为 RGB
            if small.mode != 'RGB':
                small = small.convert('RGB')

            # 获取所有像素
            pixels = list(small.getdata())

            # 聚类提取主色
            from collections import Counter
            # 简化颜色（减少精度）
            simplified = [(r//10*10, g//10*10, b//10*10) for r, g, b in pixels]
            most_common = Counter(simplified).most_common(num_colors)

            # 转换回十六进制
            colors = []
            for (r, g, b), count in most_common:
                hex_color = f"#{r:02x}{g:02x}{b:02x}"
                colors.append(hex_color)

            return colors
        except Exception as e:
            logger.error(f"提取颜色失败: {e}")
            return []

    def calculate_contrast_ratio(self, color1: str, color2: str) -> float:
        """
        计算两个颜色的对比度比率（WCAG标准）

        Args:
            color1: 十六进制颜色 #RRGGBB
            color2: 十六进制颜色 #RRGGBB

        Returns:
            对比度比率（1-21）
        """
        def hex_to_luminance(hex_color: str) -> float:
            hex_color = hex_color.lstrip('#')
            r = int(hex_color[0:2], 16) / 255
            g = int(hex_color[2:4], 16) / 255
            b = int(hex_color[4:6], 16) / 255

            # 转换为 sRGB
            r = r / 12.92 if r <= 0.03928 else ((r + 0.055) / 1.055) ** 2.4
            g = g / 12.92 if g <= 0.03928 else ((g + 0.055) / 1.055) ** 2.4
            b = b / 12.92 if b <= 0.03928 else ((b + 0.055) / 1.055) ** 2.4

            return 0.2126 * r + 0.7152 * g + 0.0722 * b

        try:
            l1 = hex_to_luminance(color1)
            l2 = hex_to_luminance(color2)

            lighter = max(l1, l2)
            darker = min(l1, l2)

            return (lighter + 0.05) / (darker + 0.05)
        except Exception:
            return 1.0

    def is_wcag_compliant(self, color1: str, color2: str, level: str = "AA") -> bool:
        """
        检查颜色是否符合 WCAG 对比度标准

        Args:
            color1: 前景色
            color2: 背景色
            level: AA 或 AAA

        Returns:
            是否合规
        """
        ratio = self.calculate_contrast_ratio(color1, color2)

        if level == "AA":
            return ratio >= 4.5
        elif level == "AAA":
            return ratio >= 7.0
        return False

    def compare_to_design_system(
        self,
        screenshot: Image.Image,
        design_system: str
    ) -> Dict[str, Any]:
        """
        对比截图与指定设计系统的一致性

        Args:
            screenshot: 截图
            design_system: cursor | macos | 21st | shadcn

        Returns:
            对比结果
        """
        if design_system not in self.DESIGN_SYSTEMS:
            return {"error": f"Unknown design system: {design_system}"}

        system = self.DESIGN_SYSTEMS[design_system]

        # 获取整体评估
        report = self.evaluate(screenshot, f"对比 {design_system} 设计规范")

        # 获取该设计系统的对齐分数
        alignment_score = report.design_system_alignment.get(design_system, 0)

        return {
            "design_system": design_system,
            "description": system["description"],
            "alignment_score": alignment_score,
            "overall_score": report.overall_score,
            "matches": report.raw_analysis.get("strengths", []),
            "mismatches": report.raw_analysis.get("weaknesses", []),
            "suggestions": self._generate_design_system_suggestions(design_system, report)
        }

    def _generate_design_system_suggestions(
        self,
        design_system: str,
        report: AestheticReport
    ) -> List[str]:
        """生成针对设计系统的改进建议"""
        suggestions = []
        system = self.DESIGN_SYSTEMS.get(design_system, {})

        if design_system == "cursor":
            # Cursor 建议
            color_score = report.get_score_by_category("color_harmony")
            if color_score and color_score.score < 80:
                suggestions.append("考虑使用更深的背景色以增强代码沉浸感")

        elif design_system == "macos":
            # macOS 建议
            consistency = report.get_score_by_category("consistency")
            if consistency and consistency.score < 80:
                suggestions.append("统一圆角大小，建议使用 8px 或 12px 作为标准")

        elif design_system == "21st":
            # 21st 建议
            minimalism = report.get_score_by_category("minimalism")
            if minimalism and minimalism.score < 80:
                suggestions.append("增加留白空间，提升呼吸感")

        elif design_system == "shadcn":
            # shadcn-ui 建议
            consistency = report.get_score_by_category("consistency")
            if consistency and consistency.score < 80:
                suggestions.append("检查 CSS 变量应用是否一致，确保使用 4px 网格系统")

        return suggestions

    def evaluate_shadcn_compliance(self, screenshot: Image.Image) -> Dict[str, Any]:
        """
        专门针对 shadcn-ui 的合规性检查

        Returns:
            合规性报告
        """
        image_base64 = self._image_to_base64(screenshot)

        prompt = """
请专门评估这张截图是否符合 shadcn-ui 设计规范：

shadcn-ui 关键规范：
1. CSS 变量系统（--background, --foreground, --primary, 等）
2. 4px 网格系统（间距为 4 的倍数）
3. 组件原子化（按钮、输入框等可复用）
4. 圆角一致性（--radius 变量）
5. 色彩使用 hsl() 格式
6. 支持深色/浅色主题切换

请按以下格式输出：

{
    "compliance_score": 85,
    "css_variables_detected": ["--background", "--foreground"],
    "spacing_grid_compliance": "符合 4px 系统",
    "component_atomicity": "良好",
    "border_radius_consistency": "统一 8px",
    "theme_support": "支持深色/浅色",
    "issues": ["发现的问题"],
    "suggestions": ["改进建议"]
}
"""

        try:
            client = self._get_client()
            message = client.messages.create(
                model="claude-3-opus-20240229",
                max_tokens=4096,
                messages=[
                    {
                        "role": "user",
                        "content": [
                            {
                                "type": "image",
                                "source": {
                                    "type": "base64",
                                    "media_type": "image/png",
                                    "data": image_base64
                                }
                            },
                            {
                                "type": "text",
                                "text": prompt
                            }
                        ]
                    }
                ]
            )

            response_text = message.content[0].text if message.content else ""
            return self._parse_json_response(response_text)

        except Exception as e:
            logger.error(f"shadcn 合规性检查失败: {e}")
            return {"error": str(e)}

    def generate_aesthetic_report_html(self, report: AestheticReport) -> str:
        """
        生成 HTML 格式的美学报告

        Args:
            report: 美学评估报告

        Returns:
            HTML 字符串
        """
        scores_html = ""
        for score in report.scores:
            bar_width = score.score
            color = "#22c55e" if score.score >= 80 else "#eab308" if score.score >= 60 else "#ef4444"

            suggestions_html = ""
            if score.suggestions:
                suggestions_html = "<ul>" + "".join(f"<li>{s}</li>" for s in score.suggestions) + "</ul>"

            scores_html += f"""
            <div style="margin-bottom: 20px;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 5px;">
                    <span style="font-weight: bold;">{score.category.replace('_', ' ').title()}</span>
                    <span>{score.score:.1f}/100</span>
                </div>
                <div style="background: #e5e5e5; height: 20px; border-radius: 10px; overflow: hidden;">
                    <div style="background: {color}; width: {bar_width}%; height: 100%;"></div>
                </div>
                {suggestions_html}
            </div>
            """

        design_system_html = ""
        for ds, score in report.design_system_alignment.items():
            design_system_html += f"""
            <div style="display: flex; justify-content: space-between; padding: 8px 0; border-bottom: 1px solid #e5e5e5;">
                <span>{ds.upper()}</span>
                <span>{score*100:.0f}%</span>
            </div>
            """

        return f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Aesthetic Evaluation Report</title>
            <style>
                body {{ font-family: system-ui, -apple-system, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
                .header {{ text-align: center; margin-bottom: 30px; }}
                .score-circle {{ width: 120px; height: 120px; border-radius: 50%; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); display: flex; align-items: center; justify-content: center; margin: 0 auto; color: white; font-size: 36px; font-weight: bold; }}
                .section {{ background: #f9fafb; border-radius: 12px; padding: 20px; margin-bottom: 20px; }}
                .section h2 {{ margin-top: 0; }}
            </style>
        </head>
        <body>
            <div class="header">
                <div class="score-circle">{report.overall_score:.0f}</div>
                <h1>美学评估报告</h1>
            </div>

            <div class="section">
                <h2>详细评分</h2>
                {scores_html}
            </div>

            <div class="section">
                <h2>设计系统对齐度</h2>
                {design_system_html}
            </div>

            <div class="section">
                <h2>总结</h2>
                <p>{report.summary}</p>
            </div>
        </body>
        </html>
        """


class IconAestheticEvaluator:
    """图标美学评估器"""

    LUCIDE_GUIDELINES = {
        "stroke_width": 1.5,
        "stroke_linecap": "round",
        "stroke_linejoin": "round",
        "sizes": [16, 20, 24],
        "grid": 24,
    }

    def __init__(self, visual_analyzer=None):
        self.visual_analyzer = visual_analyzer

    def evaluate_icon_stroke_consistency(self, screenshot: Image.Image, icons: List[Dict]) -> AestheticScore:
        """
        评估图标描边一致性
        """
        if not icons:
            return AestheticScore(
                category="icon_stroke_consistency",
                score=0,
                details={"error": "未检测到图标"},
                suggestions=["确保图标可见且可识别"]
            )

        # 分析每个图标的描边
        stroke_widths = []
        for icon in icons:
            attrs = icon.get("attributes", {})
            if "stroke_width" in attrs:
                stroke_widths.append(float(attrs["stroke_width"]))

        if not stroke_widths:
            return AestheticScore(
                category="icon_stroke_consistency",
                score=50,
                details={},
                suggestions=["无法识别图标描边宽度"]
            )

        # 检查一致性
        avg_width = sum(stroke_widths) / len(stroke_widths)
        variance = sum((w - avg_width) ** 2 for w in stroke_widths) / len(stroke_widths)
        consistency = max(0, 100 - variance * 100)

        # 检查是否符合 Lucide 规范
        lucide_compliance = 100 if 1.4 <= avg_width <= 1.6 else 70 if 1 <= avg_width <= 2 else 50

        score = (consistency + lucide_compliance) / 2

        return AestheticScore(
            category="icon_stroke_consistency",
            score=score,
            details={
                "detected_stroke_widths": stroke_widths,
                "average_width": avg_width,
                "variance": variance,
                "lucide_compliance": lucide_compliance
            },
            suggestions=[] if score >= 80 else ["统一图标描边宽度为 1.5px"]
        )

    def evaluate_icon_size_harmony(self, screenshot: Image.Image, icons: List[Dict]) -> AestheticScore:
        """
        评估图标尺寸和谐度
        """
        if not icons:
            return AestheticScore(
                category="icon_size_harmony",
                score=0,
                details={"error": "未检测到图标"}
            )

        # 收集图标尺寸
        sizes = []
        for icon in icons:
            bounds = icon.get("bounds", {})
            size = max(bounds.get("width", 0), bounds.get("height", 0))
            if size > 0:
                sizes.append(size)

        if not sizes:
            return AestheticScore(
                category="icon_size_harmony",
                score=50,
                details={},
                suggestions=["无法识别图标尺寸"]
            )

        # 检查是否遵循 4px 网格
        grid_compliance = sum(1 for s in sizes if s % 4 == 0) / len(sizes) * 100

        # 检查是否使用标准尺寸
        standard_sizes = {16, 20, 24}
        standard_compliance = sum(1 for s in sizes if s in standard_sizes) / len(sizes) * 100

        score = (grid_compliance + standard_compliance) / 2

        suggestions = []
        if grid_compliance < 80:
            suggestions.append("图标尺寸应遵循 4px 网格系统")
        if standard_compliance < 80:
            suggestions.append("建议使用标准图标尺寸：16px、20px、24px")

        return AestheticScore(
            category="icon_size_harmony",
            score=score,
            details={
                "detected_sizes": sizes,
                "grid_compliance": grid_compliance,
                "standard_compliance": standard_compliance
            },
            suggestions=suggestions
        )


class TypographyAestheticEvaluator:
    """排版美学评估器"""

    IDEAL_LINE_HEIGHT = 1.6
    IDEAL_TYPE_SCALE = [12, 14, 16, 20, 24, 32, 40]  # 理想字体大小序列

    def __init__(self, visual_analyzer=None):
        self.visual_analyzer = visual_analyzer

    def evaluate_font_stack(self, screenshot: Image.Image, text_elements: List[Dict]) -> AestheticScore:
        """
        评估字体栈优雅度
        """
        detected_fonts = set()
        for elem in text_elements:
            attrs = elem.get("attributes", {})
            if "font_family" in attrs:
                detected_fonts.add(attrs["font_family"])

        # 现代字体栈推荐
        modern_fonts = {"Geist", "Inter", "SF Pro", "SF Mono", "JetBrains Mono"}

        modern_count = sum(1 for f in detected_fonts if any(m in f for m in modern_fonts))
        modern_ratio = modern_count / len(detected_fonts) if detected_fonts else 0

        score = modern_ratio * 100

        return AestheticScore(
            category="font_stack",
            score=score,
            details={
                "detected_fonts": list(detected_fonts),
                "modern_ratio": modern_ratio
            },
            suggestions=[] if score >= 80 else ["建议使用现代字体如 Geist、Inter"]
        )

    def evaluate_type_scale_rhythm(self, screenshot: Image.Image, text_elements: List[Dict]) -> AestheticScore:
        """
        评估字体大小节奏
        """
        font_sizes = []
        for elem in text_elements:
            attrs = elem.get("attributes", {})
            if "font_size" in attrs:
                size_str = attrs["font_size"]
                try:
                    size = int(size_str.replace("px", ""))
                    font_sizes.append(size)
                except ValueError:
                    pass

        if not font_sizes:
            return AestheticScore(
                category="type_scale_rhythm",
                score=50,
                details={},
                suggestions=["无法识别字体大小"]
            )

        unique_sizes = sorted(set(font_sizes))

        # 检查是否遵循某种比例（如 1.25 或 1.5）
        ratios = []
        for i in range(1, len(unique_sizes)):
            if unique_sizes[i-1] > 0:
                ratios.append(unique_sizes[i] / unique_sizes[i-1])

        if ratios:
            avg_ratio = sum(ratios) / len(ratios)
            # 理想比例接近 1.25（Major Third）或 1.2（Minor Third）
            ratio_score = max(0, 100 - abs(avg_ratio - 1.25) * 200)
        else:
            ratio_score = 50

        return AestheticScore(
            category="type_scale_rhythm",
            score=ratio_score,
            details={
                "detected_sizes": unique_sizes,
                "average_ratio": avg_ratio if ratios else None
            },
            suggestions=[] if ratio_score >= 80 else ["建议使用一致的字体比例（如 1.25）"]
        )
