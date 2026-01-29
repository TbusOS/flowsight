"""
AI 视觉理解模块
基于 Claude API 的截图分析和元素识别
"""

from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any, Tuple
from PIL import Image
import base64
import io
import json
import logging
import os
import time

logger = logging.getLogger(__name__)


@dataclass
class UIElement:
    """UI 元素 - 视觉识别结果"""
    element_type: str  # button, text, input, icon, menu, sidebar, header, card, modal, tab, dropdown, etc.
    text: Optional[str]
    bounds: Dict[str, int]  # x, y, width, height
    confidence: float
    attributes: Dict[str, Any] = field(default_factory=dict)

    @property
    def center(self) -> Tuple[int, int]:
        return (
            self.bounds.get('x', 0) + self.bounds.get('width', 0) // 2,
            self.bounds.get('y', 0) + self.bounds.get('height', 0) // 2
        )

    @property
    def area(self) -> int:
        return self.bounds.get('width', 0) * self.bounds.get('height', 0)


@dataclass
class VisualAnalysis:
    """视觉分析结果"""
    screenshot: Image.Image
    elements: List[UIElement]
    layout_description: str
    color_palette: List[str]
    text_content: List[str]
    issues: List[str]
    raw_response: str
    timestamp: float

    def get_elements_by_type(self, element_type: str) -> List[UIElement]:
        """按类型获取元素"""
        return [e for e in self.elements if e.element_type == element_type]

    def get_elements_by_text(self, text: str, partial: bool = True) -> List[UIElement]:
        """按文本查找元素"""
        results = []
        for e in self.elements:
            if e.text:
                if partial and text.lower() in e.text.lower():
                    results.append(e)
                elif not partial and text == e.text:
                    results.append(e)
        return results

    def get_element_at_position(self, x: int, y: int) -> Optional[UIElement]:
        """获取指定位置的元素"""
        for element in self.elements:
            b = element.bounds
            if (b.get('x', 0) <= x <= b.get('x', 0) + b.get('width', 0) and
                b.get('y', 0) <= y <= b.get('y', 0) + b.get('height', 0)):
                return element
        return None


@dataclass
class ComparisonResult:
    """截图对比结果"""
    differences: List[Dict[str, Any]]
    added_elements: List[UIElement]
    removed_elements: List[UIElement]
    changed_elements: List[Dict[str, Any]]
    summary: str
    similarity_score: float


class VisualAnalyzer:
    """AI 视觉分析器 - 使用 Claude 分析截图"""

    # 视觉分析提示词模板
    ANALYSIS_PROMPT = """
你是一个专业的 UI 测试分析助手。请分析这张界面截图，识别所有可见的 UI 元素。

请按以下 JSON 格式输出分析结果：

{
    "elements": [
        {
            "element_type": "button|text|input|icon|menu|sidebar|header|footer|card|modal|tab|dropdown|toolbar|statusbar|panel",
            "text": "元素上的文本内容（如果有）",
            "bounds": {"x": 100, "y": 200, "width": 120, "height": 40},
            "confidence": 0.95,
            "attributes": {
                "state": "enabled|disabled|active|hover|selected",
                "color": "#RRGGBB",
                "font_size": "14px",
                "font_weight": "normal|bold",
                "icon_name": "如果识别出是图标，描述图标含义"
            }
        }
    ],
    "layout_description": "简要描述整体布局结构（如：左侧边栏+主内容区+右侧面板）",
    "color_palette": ["#0a0a0b", "#141416", "#ffffff"],
    "text_content": ["所有可见的文本内容列表"],
    "issues": ["发现的任何问题，如重叠、截断、对齐问题等"]
}

注意事项：
1. 坐标是相对于截图左上角的像素位置
2. confidence 表示识别置信度（0-1）
3. 尽可能识别所有交互元素（按钮、输入框、菜单、侧边栏等）
4. 指出任何视觉问题（模糊、重叠、颜色不一致等）
5. 识别图标时，尝试理解图标功能含义
"""

    # IDE 特定分析提示词
    IDE_ANALYSIS_PROMPT = """
你是一个专业的 IDE 界面分析助手。请分析这张 IDE 截图，识别所有界面元素。

IDE 常见元素类型：
- menu: 菜单栏（File, Edit, View, 等）
- sidebar: 侧边栏（文件树、大纲等）
- editor: 代码编辑器区域
- tab: 标签页
- toolbar: 工具栏
- statusbar: 状态栏
- panel: 面板（终端、输出等）
- button: 按钮（运行、调试、设置等）
- icon: 图标
- input: 输入框、搜索框
- dropdown: 下拉菜单
- tooltip: 提示框

请按以下 JSON 格式输出：

{
    "elements": [
        {
            "element_type": "类型",
            "text": "文本内容",
            "bounds": {"x": 0, "y": 0, "width": 100, "height": 30},
            "confidence": 0.95,
            "attributes": {
                "active": true,
                "collapsed": false,
                "icon_type": "folder|file|settings|run|debug|etc"
            }
        }
    ],
    "layout_structure": {
        "type": "三栏布局|双栏布局|单栏布局",
        "sidebar_width": 250,
        "editor_area": {"x": 250, "y": 50, "width": 800, "height": 600},
        "panels": ["terminal", "output"]
    },
    "color_scheme": "dark|light",
    "detected_theme": "猜测的主题名称",
    "issues": []
}
"""

    def __init__(self, api_key: Optional[str] = None):
        self.api_key = api_key or os.environ.get("ANTHROPIC_API_KEY")
        self._client = None
        self._analysis_history: List[VisualAnalysis] = []

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
        """将图片转换为 base64"""
        image_bytes = io.BytesIO()
        image.save(image_bytes, format='PNG')
        return base64.b64encode(image_bytes.getvalue()).decode()

    def analyze_screenshot(
        self,
        screenshot: Image.Image,
        context: Optional[str] = None,
        is_ide: bool = True
    ) -> VisualAnalysis:
        """
        分析截图，识别所有 UI 元素

        Args:
            screenshot: PIL Image 对象
            context: 可选的上下文信息（当前页面、测试场景等）
            is_ide: 是否是 IDE 界面分析

        Returns:
            VisualAnalysis 分析结果
        """
        image_base64 = self._image_to_base64(screenshot)

        # 选择提示词
        prompt = self.IDE_ANALYSIS_PROMPT if is_ide else self.ANALYSIS_PROMPT
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

            # 解析 JSON 响应
            response_text = message.content[0].text if message.content else ""
            analysis_data = self._parse_json_response(response_text)

            # 构建 VisualAnalysis 对象
            elements = [
                UIElement(
                    element_type=e.get("element_type", "unknown"),
                    text=e.get("text"),
                    bounds=e.get("bounds", {}),
                    confidence=e.get("confidence", 0.5),
                    attributes=e.get("attributes", {})
                )
                for e in analysis_data.get("elements", [])
            ]

            analysis = VisualAnalysis(
                screenshot=screenshot,
                elements=elements,
                layout_description=analysis_data.get("layout_description",
                                                     analysis_data.get("layout_structure", {}).get("type", "")),
                color_palette=analysis_data.get("color_palette", []),
                text_content=analysis_data.get("text_content", []),
                issues=analysis_data.get("issues", []),
                raw_response=response_text,
                timestamp=time.time()
            )

            self._analysis_history.append(analysis)
            return analysis

        except Exception as e:
            logger.error(f"视觉分析失败: {e}")
            # 返回空的分析结果
            return VisualAnalysis(
                screenshot=screenshot,
                elements=[],
                layout_description="",
                color_palette=[],
                text_content=[],
                issues=[f"分析失败: {e}"],
                raw_response=str(e),
                timestamp=time.time()
            )

    def _parse_json_response(self, response: str) -> Dict:
        """从响应文本中提取 JSON 数据"""
        try:
            # 尝试直接解析
            return json.loads(response)
        except json.JSONDecodeError:
            # 尝试从文本中提取 JSON 块
            import re
            json_match = re.search(r'```json\s*(.*?)\s*```', response, re.DOTALL)
            if json_match:
                return json.loads(json_match.group(1))

            # 尝试找到最外层的大括号
            json_match = re.search(r'\{.*\}', response, re.DOTALL)
            if json_match:
                try:
                    return json.loads(json_match.group(0))
                except json.JSONDecodeError:
                    pass

            return {}

    def find_element(
        self,
        analysis: VisualAnalysis,
        element_type: Optional[str] = None,
        text: Optional[str] = None,
        partial_match: bool = True
    ) -> Optional[UIElement]:
        """
        在分析结果中查找特定元素

        Args:
            analysis: 视觉分析结果
            element_type: 元素类型过滤
            text: 文本内容过滤
            partial_match: 是否允许部分匹配文本

        Returns:
            匹配的元素或 None
        """
        for element in analysis.elements:
            # 类型匹配
            if element_type and element.element_type != element_type:
                continue

            # 文本匹配
            if text:
                element_text = element.text or ""
                if partial_match:
                    if text.lower() not in element_text.lower():
                        continue
                else:
                    if text != element_text:
                        continue

            return element

        return None

    def find_elements(
        self,
        analysis: VisualAnalysis,
        element_type: Optional[str] = None
    ) -> List[UIElement]:
        """
        查找所有符合类型的元素

        Args:
            analysis: 视觉分析结果
            element_type: 元素类型过滤

        Returns:
            匹配的元素列表
        """
        if not element_type:
            return analysis.elements

        return [e for e in analysis.elements if e.element_type == element_type]

    def compare_screenshots(
        self,
        screenshot1: Image.Image,
        screenshot2: Image.Image,
        context: Optional[str] = None
    ) -> ComparisonResult:
        """
        比较两张截图，找出差异

        Returns:
            差异分析结果
        """
        image1_base64 = self._image_to_base64(screenshot1)
        image2_base64 = self._image_to_base64(screenshot2)

        prompt = """
请比较这两张界面截图，分析它们之间的差异。

请按以下格式输出：

{
    "differences": [
        {
            "type": "added|removed|changed|moved",
            "description": "变化描述",
            "element": "相关元素",
            "severity": "high|medium|low"
        }
    ],
    "added_elements": ["新增的元素列表"],
    "removed_elements": ["移除的元素列表"],
    "changed_elements": ["变化的元素列表"],
    "summary": "差异总结",
    "similarity_score": 0.85
}
"""
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
                                "type": "text",
                                "text": "第一张截图（基准）:"
                            },
                            {
                                "type": "image",
                                "source": {
                                    "type": "base64",
                                    "media_type": "image/png",
                                    "data": image1_base64
                                }
                            },
                            {
                                "type": "text",
                                "text": "第二张截图（对比）:"
                            },
                            {
                                "type": "image",
                                "source": {
                                    "type": "base64",
                                    "media_type": "image/png",
                                    "data": image2_base64
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

            return ComparisonResult(
                differences=result_data.get("differences", []),
                added_elements=[],  # 简化处理，实际可以解析详细元素
                removed_elements=[],
                changed_elements=[],
                summary=result_data.get("summary", ""),
                similarity_score=result_data.get("similarity_score", 0.0)
            )

        except Exception as e:
            logger.error(f"截图比较失败: {e}")
            return ComparisonResult(
                differences=[],
                added_elements=[],
                removed_elements=[],
                changed_elements=[],
                summary=f"比较失败: {e}",
                similarity_score=0.0
            )

    def get_element_at_position(
        self,
        analysis: VisualAnalysis,
        x: int,
        y: int
    ) -> Optional[UIElement]:
        """
        获取指定位置的元素

        Args:
            analysis: 视觉分析结果
            x: 横坐标
            y: 纵坐标

        Returns:
            该位置的元素或 None
        """
        for element in analysis.elements:
            bounds = element.bounds
            if (bounds.get('x', 0) <= x <= bounds.get('x', 0) + bounds.get('width', 0) and
                bounds.get('y', 0) <= y <= bounds.get('y', 0) + bounds.get('height', 0)):
                return element
        return None

    def detect_visual_issues(self, screenshot: Image.Image) -> List[Dict[str, Any]]:
        """
        专门检测视觉问题

        Returns:
            检测到的问题列表
        """
        image_base64 = self._image_to_base64(screenshot)

        prompt = """
请仔细分析这张界面截图，专门查找视觉问题和 UI 缺陷。

请检查以下方面：
1. 文本截断或溢出
2. 元素重叠
3. 对齐问题
4. 颜色对比度不足
5. 模糊或失真
6. 间距不一致
7. 布局错乱

请按以下格式输出：

{
    "issues": [
        {
            "type": "text_truncation|overlap|misalignment|contrast|blur|spacing|layout",
            "severity": "critical|high|medium|low",
            "description": "问题描述",
            "location": {"x": 100, "y": 200, "width": 50, "height": 20},
            "suggestion": "修复建议"
        }
    ]
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
            result_data = self._parse_json_response(response_text)
            return result_data.get("issues", [])

        except Exception as e:
            logger.error(f"视觉问题检测失败: {e}")
            return []

    def get_analysis_history(self) -> List[VisualAnalysis]:
        """获取分析历史"""
        return self._analysis_history.copy()

    def clear_history(self):
        """清空分析历史"""
        self._analysis_history.clear()

    def analyze_region(
        self,
        screenshot: Image.Image,
        region: Tuple[int, int, int, int],  # x, y, width, height
        context: Optional[str] = None
    ) -> VisualAnalysis:
        """
        分析截图的特定区域

        Args:
            screenshot: 完整截图
            region: 区域坐标 (x, y, width, height)
            context: 上下文信息

        Returns:
            区域分析结果
        """
        # 裁剪区域
        x, y, w, h = region
        cropped = screenshot.crop((x, y, x + w, y + h))

        # 分析裁剪后的图像
        analysis = self.analyze_screenshot(cropped, context, is_ide=True)

        # 调整元素坐标到原图坐标系
        for element in analysis.elements:
            element.bounds['x'] += x
            element.bounds['y'] += y

        return analysis
