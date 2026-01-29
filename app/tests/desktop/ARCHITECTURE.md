# FlowSight Desktop UI 测试框架架构

## 概述

一个AI驱动的桌面IDE UI自动化测试框架，类似Claude Computer Use，专为Tauri应用设计。

## 架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                      TestRunner (测试运行器)                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐  │
│  │ LayoutTests │ │ VisualTests │ │Interactive  │ │StateTests │  │
│  │   布局测试   │ │   视觉测试   │ │  交互测试   │ │  状态测试  │  │
│  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                 WebViewTests (WebView测试)                │  │
│  │         Playwright + CDP 测试 Tauri WebView 内容          │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                      DesktopAdapter (桌面适配层)                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐   │
│  │ MacOSAdapter│ │WindowsAdapter│ │      LinuxAdapter       │   │
│  │  AppleScript│ │  pywinauto   │ │xdotool + python-atspi │   │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                     VisualAnalyzer (AI视觉分析)                   │
│              Claude API + 截图分析 + 元素识别                      │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                  AestheticEvaluator (美学评估器)                  │
│  Cursor │ macOS │ 21st.dev │ shadcn/ui  设计规范评估              │
└─────────────────────────────────────────────────────────────────┘
```

## 核心模块

### 1. DesktopAdapter (跨平台桌面自动化)
**文件**: `core/desktop_adapter.py`

- **MacOSAdapter**: AppleScript + AX API
- **WindowsAdapter**: pywin32 + pywinauto
- **LinuxAdapter**: xdotool + python-atspi

**功能**:
- 窗口查找与激活
- 截图捕获
- 鼠标/键盘操作
- 无障碍元素遍历

### 2. VisualAnalyzer (AI视觉分析)
**文件**: `core/visual_analyzer.py`

- 截图AI分析
- UI元素识别
- 布局理解
- OCR文本提取

### 3. AestheticEvaluator (美学评估)
**文件**: `aesthetic/aesthetic_evaluator.py`

评估维度:
| 设计系统 | 特点 |
|---------|------|
| Cursor  | 深色主题、紫色强调、代码编辑器风格 |
| macOS   | 半透明、圆角、精致阴影 |
| 21st    | 大胆排版、渐变、现代感 |
| shadcn  | CSS变量、4px网格、原子化组件 |

### 4. 测试模块

#### 4.1 LayoutTests (布局测试)
**文件**: `tests/layout_tests.py`

- 元素对齐检测
- 间距一致性
- 重叠检测
- 响应式布局
- shadcn布局合规

#### 4.2 VisualTests (视觉测试)
**文件**: `tests/visual_tests.py`

- 颜色对比度 (WCAG AA/AAA)
- 字体渲染
- 图标渲染
- 颜色一致性
- 暗色模式支持

#### 4.3 InteractiveTests (交互测试)
**文件**: `tests/interactive_tests.py`

- 按钮点击
- 悬停效果
- 菜单导航
- 键盘快捷键

#### 4.4 StateTests (状态测试)
**文件**: `tests/state_tests.py`

- 组件状态转换
- 表单状态
- 异步操作
- 错误处理

### 5. WebViewTests (WebView测试)
**文件**: `playwright/webview_tests.py`

针对Tauri WebView的深度测试:

```python
# DOM结构测试
- 语义化HTML标签
- 必要元数据
- 标题层级

# CSS合规测试
- CSS变量使用
- 颜色格式
- 字体栈合规

# JavaScript功能测试
- React组件渲染
- 事件监听
- 控制台错误

# 性能测试
- FCP, LCP, TTI
- 内存使用
- DOM节点数

# 无障碍测试
- 图片alt属性
- ARIA标签
- 焦点管理
- 键盘导航
```

**WebView检测器**: `playwright/webview_detector.py`
- 自动扫描远程调试端口
- 多平台WebView发现

**WebView适配器**: `playwright/webview_adapter.py`
- Playwright CDP连接
- 页面操作API
- 性能监控
- 控制台日志捕获

## 测试执行流程

```
Setup
  ↓
Layout (布局测试) ─────────────────┐
  ↓                                 │
Visual (视觉测试) ──────────────────┤
  ↓                                 │  截图 + AI分析
Interactive (交互测试) ────────────┤
  ↓                                 │
State (状态测试) ──────────────────┘
  ↓
WebView (WebView测试) ──────────── CDP协议深度测试
  ↓
Aesthetic (美学评估) ───────────── 设计规范评分
  ↓
Teardown
  ↓
Report Generation (报告生成)
```

## 报告生成

**JSON报告**: 结构化数据，便于CI/CD集成
**HTML报告**: 可视化展示，包含:
- 测试摘要统计
- 质量评分图表
- 详细测试结果表格
- 设计改进建议

## 使用示例

```python
from desktop import DesktopAdapter, TestRunner, VisualAnalyzer

# 初始化
adapter = DesktopAdapter.for_current_platform("FlowSight")
visual = VisualAnalyzer(api_key="...")
runner = TestRunner(adapter, visual)

# 注册并运行测试
runner.register_default_tests()
report = await runner.run_all_tests()

# 查看结果
print(f"总体评分: {report.overall_score}")
print(f"美学评分: {report.aesthetic_score}")
```

## 技术栈

| 组件 | 技术 |
|-----|------|
| 桌面自动化 | AppleScript / pywinauto / xdotool |
| AI分析 | Claude API (视觉能力) |
| WebView测试 | Playwright + CDP |
| 图像处理 | Pillow |
| OCR | easyocr / pytesseract |
| 报告 | JSON + HTML |

## 文件结构

```
desktop/
├── __init__.py              # 主入口
├── ARCHITECTURE.md          # 本文件
├── requirements.txt         # 依赖
├── example.py              # 使用示例
│
├── core/                   # 核心模块
│   ├── __init__.py
│   ├── desktop_adapter.py  # 跨平台适配器
│   ├── visual_analyzer.py  # AI视觉分析
│   └── test_runner.py      # 测试运行器
│
├── aesthetic/              # 美学评估
│   ├── __init__.py
│   └── aesthetic_evaluator.py
│
├── tests/                  # 测试模块
│   ├── __init__.py
│   ├── layout_tests.py
│   ├── visual_tests.py
│   ├── interactive_tests.py
│   └── state_tests.py
│
└── playwright/             # WebView测试
    ├── __init__.py
    ├── webview_detector.py
    ├── webview_adapter.py
    └── webview_tests.py
```

## 扩展指南

### 添加新测试类型

1. 在 `tests/` 创建新模块
2. 继承或参考现有测试类
3. 在 `TestRunner.register_default_tests()` 注册
4. 添加对应的 `TestPhase`

### 添加新设计系统

1. 在 `aesthetic_evaluator.py` 添加设计系统定义
2. 实现评估方法
3. 更新 `evaluate()` 方法

### 支持新平台

1. 在 `desktop_adapter.py` 创建新适配器类
2. 实现 `DesktopPlatformAdapter` 接口
3. 在 `DesktopAdapter.for_current_platform()` 添加平台检测
