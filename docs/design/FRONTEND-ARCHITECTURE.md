# FlowSight 前端架构设计

> 版本: 1.0.0
> 更新日期: 2024-01-24
> 状态: 正式发布

## 1. 概述

### 1.1 项目简介

FlowSight 是一个跨平台的函数执行流分析工具，基于 React + TypeScript + Tauri 构建。该工具能够不运行代码的情况下精准展示函数的执行过程，特别适用于 Linux 内核等大型代码库的分析。

### 1.2 技术栈

| 类别 | 技术选型 | 版本要求 |
|------|----------|----------|
| UI 框架 | React | 18.x |
| 语言 | TypeScript | 5.x |
| 桌面框架 | Tauri | 2.x |
| 代码编辑器 | Monaco Editor | 0.55.x |
| 流程图库 | @xyflow/react | 12.x |
| 状态管理 | Zustand | 4.x |
| 构建工具 | Vite | 5.x |

### 1.3 核心设计原则

1. **组件化设计**: 所有功能模块均采用独立组件，便于维护和复用
2. **响应式状态管理**: 使用 Zustand 实现全局状态共享，避免 props drilling
3. **性能优先**: 针对大数据量展示进行专项优化，确保流畅的用户体验
4. **可访问性**: 遵循 WCAG 2.1 AA 标准，支持键盘导航和屏幕阅读器
5. **模块化通信**: 与 Tauri 后端采用清晰的事件和命令机制通信

---

## 2. 组件结构设计

### 2.1 整体布局架构

```
app/src/
├── components/                    # 组件目录
│   ├── Layout/                    # 布局组件
│   │   ├── Header/               # 顶部导航栏
│   │   ├── Sidebar/              # 侧边栏（可折叠）
│   │   ├── MainArea/             # 主内容区域
│   │   ├── ResizeHandle/         # 面板调整手柄
│   │   └── index.ts
│   ├── Editor/                   # 代码编辑器模块
│   │   ├── CodeEditor.tsx        # Monaco Editor 包装器
│   │   ├── CodeLens/             # 代码透镜（装饰器）
│   │   ├── GlyphMargin/          # 边距图标
│   │   ├── Minimap/              # 小地图
│   │   └── index.ts
│   ├── FlowView/                 # 执行流可视化模块
│   │   ├── FlowView.tsx          # 主流程图组件 (@xyflow)
│   │   ├── FlowTextView.tsx      # 文本模式视图 (ftrace 风格)
│   │   ├── FlowNode.tsx          # 自定义节点组件
│   │   ├── FlowEdge.tsx          # 自定义边组件
│   │   ├── CallGraph/            # 调用图模块
│   │   ├── BranchNode.tsx        # 分支条件节点
│   │   ├── ParallelNode.tsx      # 并行执行节点
│   │   └── index.ts
│   ├── Explorer/                 # 文件浏览器模块
│   │   ├── FileTree.tsx          # 文件树组件
│   │   ├── FileNode.tsx          # 文件节点组件
│   │   ├── Breadcrumb.tsx        # 面包屑导航
│   │   └── index.ts
│   ├── Outline/                  # 代码大纲模块
│   │   ├── Outline.tsx           # 大纲列表
│   │   ├── OutlineItem.tsx       # 大纲项组件
│   │   └── index.ts
│   ├── Panels/                   # 侧边面板模块
│   │   ├── FunctionDetail/       # 函数详情面板
│   │   ├── CallersView/          # 调用者视图
│   │   ├── ScenarioPanel/        # 场景分析面板
│   │   ├── ScenarioResults/      # 场景分析结果
│   │   ├── AsyncInfoPanel/       # 异步信息面板
│   │   └── index.ts
│   ├── Dialogs/                  # 对话框模块
│   │   ├── CommandPalette/       # 命令面板
│   │   ├── Settings/             # 设置对话框
│   │   ├── KeyboardShortcuts/    # 快捷键帮助
│   │   ├── GoToLine/             # 跳转行号
│   │   ├── QuickOpen/            # 快速打开文件
│   │   ├── AboutDialog/          # 关于对话框
│   │   ├── ConfirmDialog/        # 确认对话框
│   │   └── index.ts
│   ├── Common/                   # 通用组件
│   │   ├── Button/               # 按钮
│   │   ├── Input/                # 输入框
│   │   ├── Select/               # 下拉选择
│   │   ├── Toggle/               # 开关
│   │   ├── Tabs/                 # 标签页
│   │   ├── Toast/                # 消息提示
│   │   ├── ProgressIndicator/    # 进度指示器
│   │   ├── LoadingSpinner/       # 加载动画
│   │   ├── EmptyState/           # 空状态
│   │   ├── Tooltip/              # 提示框
│   │   ├── ContextMenu/          # 右键菜单
│   │   ├── DiffView/             # 差异对比视图
│   │   └── index.ts
│   └── index.ts                  # 组件统一导出
├── store/                        # 状态管理
│   ├── index.ts                  # 统一导出
│   ├── editorStore.ts            # 编辑器状态
│   ├── analysisStore.ts          # 分析状态
│   ├── uiStore.ts                # UI 状态
│   └── projectStore.ts           # 项目状态 (新增)
├── hooks/                        # 自定义 Hooks
│   ├── index.ts
│   ├── useTauri.ts               # Tauri API 封装
│   ├── useKeyboard.ts            # 键盘事件处理
│   ├── useDragDrop.ts            # 拖拽功能
│   ├── useDebounce.ts            # 防抖/节流
│   ├── useIntersectionObserver.ts # 观察者模式
│   └── useMemoize.ts             # 记忆化计算
├── services/                     # 服务层
│   ├── index.ts
│   ├── tauriService.ts           # Tauri 命令封装
│   ├── analysisService.ts        # 分析服务
│   ├── fileService.ts            # 文件服务
│   ├── searchService.ts          # 搜索服务
│   └── themeService.ts           # 主题服务
├── utils/                        # 工具函数
│   ├── index.ts
│   ├── formatters.ts             # 格式化工具
│   ├── validators.ts             # 验证工具
│   ├── recentFiles.ts            # 最近文件管理
│   └── constants.ts              # 常量定义
├── types/                        # 类型定义
│   ├── index.ts
│   ├── analysis.ts               # 分析相关类型
│   ├── flow.ts                   # 流程图类型
│   ├── editor.ts                 # 编辑器类型
│   └── ui.ts                     # UI 类型
├── styles/                       # 全局样式
│   ├── index.css                 # 主样式文件
│   ├── variables.css             # CSS 变量
│   ├── themes/                   # 主题样式
│   │   ├── dark.css
│   │   └── light.css
│   └── utils.css                 # 工具类
└── App.tsx                       # 应用入口
```

### 2.2 组件分层架构

```
┌─────────────────────────────────────────────────────────────┐
│                     Application Layer                        │
│                    (App.tsx - 根组件)                         │
├─────────────────────────────────────────────────────────────┤
│                      Layout Layer                            │
│  ┌─────────────┬───────────────────────┬─────────────────┐  │
│  │   Header    │      MainArea         │    RightPanel   │  │
│  │  (工具栏)    │  ┌─────────────────┐  │   (详情面板)     │  │
│  │             │  │ SplitView       │  │                 │  │
│  │             │  │ ┌─────┬───────┐ │  │                 │  │
│  │             │  │ │Editor│ Flow │ │  │                 │  │
│  │             │  │ │     │ View │ │  │                 │  │
│  │             │  │ └─────┴───────┘ │  │                 │  │
│  │             │  └─────────────────┘  │                 │  │
│  └─────────────┴───────────────────────┴─────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                     Feature Layer                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │
│  │ Explorer │ │ Outline  │ │Command   │ │ Settings │      │
│  │          │ │          │ │Palette   │ │          │      │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘      │
├─────────────────────────────────────────────────────────────┤
│                    Base Component Layer                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │
│  │  Button  │ │  Input   │ │  Tabs    │ │  Toast   │      │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 核心组件详细设计

#### 2.3.1 代码编辑器组件 (CodeEditor)

```typescript
// 文件: app/src/components/Editor/CodeEditor.tsx

interface CodeEditorProps {
  // 内容管理
  content: string
  filePath: string
  onChange?: (value: string) => void

  // 导航功能
  goToLine?: { line: number; timestamp: number }
  goToFunction?: { functionName: string; timestamp: number }

  // 装饰器
  highlightLines?: number[]
  highlightSymbols?: { line: number; column: number }[]
  decorations?: Decoration[]

  // 交互回调
  onLineClick?: (line: number) => void
  onWordAtCursor?: (word: string | null) => void
  onContentChange?: (content: string) => void

  // 配置
  readOnly?: boolean
  theme?: 'dark' | 'light'
  fontSize?: number
  fontFamily?: string
  showMinimap?: boolean
  showLineNumbers?: boolean
  wordWrap?: 'off' | 'on' | 'wordWrapColumn'

  // 代码分析集成
  diagnostics?: Diagnostic[]
  foldingRanges?: FoldingRange[]
  outlineItems?: OutlineItem[]
}

// 装饰器类型定义
interface Decoration {
  range: { startLine: number; startCol: number; endLine: number; endCol: number }
  className: string
  hoverMessage?: string
  glyphMarginClassName?: string
}
```

#### 2.3.2 流程图组件 (FlowView)

```typescript
// 文件: app/src/components/FlowView/FlowView.tsx

interface FlowViewProps {
  // 数据源
  flowTrees: FlowTreeNode[]
  displayMode?: 'graph' | 'text'

  // 交互
  onNodeClick?: (nodeId: string, functionName: string) => void
  onNodeDoubleClick?: (nodeId: string, functionName: string) => void
  onEdgeClick?: (edgeId: string) => void
  onCanvasClick?: () => void

  // 配置
  selectedFunction?: string
  highlightedPath?: string[]    // 高亮执行路径
  showConfidence?: boolean      // 显示置信度
  showBranches?: boolean        // 显示分支
  nodeLimit?: number            // 节点数量限制（性能优化）

  // 视图状态
  zoomLevel?: number
  panPosition?: { x: number; y: number }
}

// 流程节点类型扩展
interface FlowNodeData {
  id: string
  name: string
  displayName: string
  nodeType: NodeType
  location?: Location
  confidence?: CallConfidence
  asyncMechanism?: AsyncMechanism
  isExpanded?: boolean
  childCount?: number

  // 交互回调
  onToggle?: () => void
  onContextMenu?: (e: React.MouseEvent) => void
}

// 节点类型
type NodeType =
  | 'function'
  | 'entry-point'
  | 'async-callback'
  | 'kernel-api'
  | 'external'
  | 'branch'
  | 'loop'
  | 'conditional'

// 分支节点
interface BranchNodeData extends FlowNodeData {
  condition: string
  trueBranch?: FlowTreeNode
  falseBranch?: FlowTreeNode
  cases?: { caseValue: string; branch: FlowTreeNode }[]
}
```

#### 2.3.3 执行流文本视图 (FlowTextView)

```typescript
// 文件: app/src/components/FlowView/FlowTextView.tsx

interface FlowTextViewProps {
  flowTrees: FlowTreeNode[]
  displayMode?: 'indented' | 'timeline' | 'ftrace'
  onNodeClick?: (functionName: string) => void
  selectedFunction?: string
  maxDepth?: number              // 最大缩进深度
  showLineNumbers?: boolean      // 显示行号
  showConfidence?: boolean       // 显示置信度
}

// ftrace 风格显示配置
interface FtraceStyleConfig {
  prefix: string                 // 每行前缀符号
  indentSize: number             // 缩进空格数
  showTimestamp?: boolean        // 显示时间戳
  showDuration?: boolean         // 显示执行时长
  maxWidth?: number              // 最大行宽度
}
```

#### 2.3.4 函数详情面板 (FunctionDetail)

```typescript
// 文件: app/src/components/Panels/FunctionDetail/FunctionDetail.tsx

interface FunctionDetailProps {
  functionDetail: FunctionDetail | null
  onScenarioAnalyze?: () => void
  onViewCallers?: () => void
  onViewCallees?: () => void
  onNavigateToSource?: (file: string, line: number) => void
}

// 函数详情数据
interface FunctionDetail {
  name: string
  returnType: string
  file: string | null
  line: number
  endLine: number
  signature: string

  // 回调信息
  isCallback: boolean
  callbackContext?: CallbackContext

  // 调用关系
  calls: FunctionCall[]
  calledBy: FunctionCall[]

  // 参数信息
  params: Parameter[]

  // 异步信息
  asyncInfo?: AsyncInfo

  // 置信度
  confidence?: CallConfidence
}

interface CallbackContext {
  mechanism: string              // WorkQueue, Timer, IRQ, etc.
  trigger: string                // 触发代码
  registrationLocation?: string  // 注册位置
}

interface FunctionCall {
  name: string
  file?: string
  line?: number
  isCallback?: boolean
  confidence?: ConfidenceLevel
}

interface Parameter {
  name: string
  type: string
  position: number
  description?: string
}

interface AsyncInfo {
  mechanism: AsyncMechanism
  executionContext: string
  triggerExplanation: string
}
```

### 2.4 组件通信模式

#### 2.4.1 Props 传递（父子组件）

```typescript
// 父组件传递 props 给子组件
<FlowView
  flowTrees={analysisResult.flow_trees}
  onNodeClick={handleNodeClick}
  selectedFunction={selectedFunction}
  displayMode={flowDisplayMode}
/>
```

#### 2.4.2 事件回调（子父通信）

```typescript
// 子组件通过回调通知父组件
const handleNodeClick = (nodeId: string, functionName: string) => {
  // 更新状态
  setSelectedFunction(functionName)
  // 导航
  navigateToFunction(functionName)
  // 记录历史
  pushNavHistory({ filePath, selectedFunction: functionName })
}
```

#### 2.4.3 Context 共享（跨层级）

```typescript
// 文件: app/src/components/Providers.tsx
import { createContext, useContext } from 'react'

interface AppContextType {
  // 项目状态
  project: ProjectInfo | null
  filePath: string

  // 分析状态
  analysisResult: AnalysisResult | null
  selectedFunction: string | null

  // UI 状态
  theme: 'dark' | 'light'
  viewMode: ViewMode

  // 操作
  onFunctionSelect: (name: string) => void
  onNavigate: (location: Location) => void
}

export const AppContext = createContext<AppContextType | null>(null)

export function AppProvider({ children }: { children: React.ReactNode }) {
  const stores = useStores()

  return (
    <AppContext.Provider value={{
      project: stores.project,
      filePath: stores.editor.filePath,
      analysisResult: stores.analysis.result,
      selectedFunction: stores.analysis.selectedFunction,
      theme: stores.ui.appSettings.theme,
      viewMode: stores.ui.viewMode,
      onFunctionSelect: (name) => stores.analysis.setSelectedFunction(name),
      onNavigate: (loc) => { /* 导航实现 */ },
    }}>
      {children}
    </AppContext.Provider>
  )
}

export function useAppContext() {
  const context = useContext(AppContext)
  if (!context) throw new Error('useAppContext must be used within AppProvider')
  return context
}
```

---

## 3. 状态管理方案

### 3.1 Zustand Store 结构

```
store/
├── index.ts                    # 统一导出
├── editorStore.ts              # 编辑器状态（文件、标签页等）
├── analysisStore.ts            # 分析结果状态
├── uiStore.ts                  # UI 状态（视图模式、面板等）
├── projectStore.ts             # 项目状态（新增）
└── tauriStore.ts               # Tauri 连接状态（新增）
```

### 3.2 Store 详细设计

#### 3.2.1 编辑器状态 (editorStore)

```typescript
// 文件: app/src/store/editorStore.ts

import { create } from 'zustand'
import { persist } from 'zustand/middleware'

// 标签页数据类型
export interface TabData {
  id: string
  filePath: string
  fileName: string
  content: string
  isDirty: boolean
  analysisResult: AnalysisResult | null
  cursorPosition?: { line: number; column: number }
  scrollPosition?: { top: number; left: number }
}

// 编辑器状态
interface EditorState {
  // 文件内容
  currentFilePath: string
  currentFileContent: string

  // 标签页管理
  tabs: TabData[]
  activeTabId: string | null

  // 项目状态
  project: ProjectInfo | null
  fileTree: FileNode[]

  // 加载状态
  loading: boolean
  error: string | null

  // === Actions ===

  // 标签页操作
  openFileInTab: (path: string) => Promise<string | null>
  closeTab: (tabId: string, force?: boolean) => 'needs-confirm' | void
  switchTab: (tabId: string) => void
  reorderTabs: (fromIndex: number, toIndex: number) => void
  updateTabContent: (tabId: string, content: string) => void
  markTabDirty: (tabId: string, isDirty: boolean) => void
  updateTabAnalysis: (tabId: string, analysis: AnalysisResult) => void
  saveTab: (tabId: string) => Promise<void>

  // 文件操作
  readFile: (path: string) => Promise<string>
  writeFile: (path: string, content: string) => Promise<void>

  // 项目操作
  openProject: (path: string) => Promise<void>
  loadFileTree: (path: string) => Promise<void>
}

export const useEditorStore = create<EditorState>()(
  persist(
    (set, get) => ({
      // 初始状态
      currentFilePath: '',
      currentFileContent: '',
      tabs: [],
      activeTabId: null,
      project: null,
      fileTree: [],
      loading: false,
      error: null,

      // 实现...
    }),
    {
      name: 'flowsight-editor',
      partialize: (state) => ({
        // 只持久化标签页信息，不持久化内容
        tabs: state.tabs.map(t => ({
          ...t,
          content: '',  // 不持久化内容
          isDirty: false,
          analysisResult: null,
        })),
      }),
    }
  )
)
```

#### 3.2.2 分析状态 (analysisStore)

```typescript
// 文件: app/src/store/analysisStore.ts

import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

// 分析状态
interface AnalysisState {
  // 核心数据
  result: AnalysisResult | null
  selectedFunction: string | null
  functionDetail: FunctionDetail | null

  // 搜索与导航
  searchQuery: string
  searchResults: SearchResult[]
  outlineItems: OutlineItem[]

  // 场景分析
  scenarioResults: ScenarioResult | null
  currentScenarioName: string

  // 调用图数据
  callGraph: CallGraphData | null
  callersGraph: CallersGraphData | null

  // 加载状态
  loading: boolean
  progress: AnalysisProgress | null

  // === Actions ===

  // 数据设置
  setResult: (result: AnalysisResult | null) => void
  setSelectedFunction: (func: string | null) => void
  setFunctionDetail: (detail: FunctionDetail | null) => void
  setSearchQuery: (query: string) => void
  setSearchResults: (results: SearchResult[]) => void
  setOutlineItems: (items: OutlineItem[]) => void

  // 分析操作
  analyzeFile: (path: string) => Promise<void>
  getFunctions: (path: string) => Promise<OutlineItem[]>
  searchSymbols: (query: string) => Promise<SearchResult[]>
  getFunctionDetail: (funcName: string, filePath?: string) => Promise<FunctionDetail | null>

  // 场景分析
  executeScenario: (scenario: Scenario) => Promise<ScenarioResult | null>

  // 调用图操作
  loadCallGraph: (rootFunction?: string) => Promise<CallGraphData | null>
  loadCallersGraph: (targetFunction: string) => Promise<CallersGraphData | null>

  // 工具函数
  findFunctionInOutline: (funcName: string) => OutlineItem | undefined
  findNodeInFlowTree: (funcName: string) => FlowTreeNode | null
  getKnownFunctions: () => string[]
  getAncestorFunctions: (funcName: string) => string[]
  getDescendantFunctions: (funcName: string) => string[]
}

export const useAnalysisStore = create<AnalysisState>()((set, get) => ({
  // 初始状态
  result: null,
  selectedFunction: null,
  functionDetail: null,
  searchQuery: '',
  searchResults: [],
  outlineItems: [],
  scenarioResults: null,
  currentScenarioName: '',
  callGraph: null,
  callersGraph: null,
  loading: false,
  progress: null,

  // 实现...
}))
```

#### 3.2.3 UI 状态 (uiStore)

```typescript
// 文件: app/src/store/uiStore.ts

import { create } from 'zustand'
import { persist } from 'zustand/middleware'

// 视图模式
export type ViewMode = 'code' | 'flow' | 'split'
export type FlowDisplayMode = 'graph' | 'text'
export type TextDisplayStyle = 'indented' | 'timeline' | 'ftrace'

// 导航历史记录
export interface NavigationEntry {
  filePath: string
  selectedFunction: string | null
  line?: number
  timestamp: number
}

// 索引进度
export interface IndexProgress {
  phase: string
  current: number
  total: number
  message: string
}

// UI 状态
interface UIState {
  // 视图配置
  viewMode: ViewMode
  flowDisplayMode: FlowDisplayMode
  textDisplayStyle: TextDisplayStyle

  // 面板状态
  leftPanelOpen: boolean
  rightPanelOpen: boolean
  leftPanelWidth: number
  rightPanelWidth: number
  leftPanelCollapsed: boolean
  rightPanelCollapsed: boolean

  // 导航历史
  navHistory: NavigationEntry[]
  navIndex: number

  // 模态框状态
  commandPaletteOpen: boolean
  settingsOpen: boolean
  shortcutsOpen: boolean
  goToLineOpen: boolean
  quickOpenOpen: boolean
  findReplaceOpen: boolean
  aboutOpen: boolean

  // 特殊面板状态
  scenarioPanelOpen: boolean
  scenarioResultsOpen: boolean
  callersViewOpen: boolean
  callersTargetFunc: string

  // 索引进度
  indexProgress: IndexProgress | null

  // 主题设置
  theme: 'dark' | 'light'
  sidebarTheme: 'dark' | 'light' | 'match'

  // === Actions ===

  // 视图模式
  setViewMode: (mode: ViewMode) => void
  setFlowDisplayMode: (mode: FlowDisplayMode) => void
  setTextDisplayStyle: (style: TextDisplayStyle) => void
  toggleViewMode: () => void

  // 面板控制
  setLeftPanelOpen: (open: boolean) => void
  setRightPanelOpen: (open: boolean) => void
  setLeftPanelWidth: (width: number) => void
  setRightPanelWidth: (width: number) => void
  toggleLeftPanel: () => void
  toggleRightPanel: () => void

  // 导航历史
  pushNavHistory: (entry: Omit<NavigationEntry, 'timestamp'>) => void
  goBack: () => Promise<NavigationEntry | undefined>
  goForward: () => Promise<NavigationEntry | undefined>
  clearNavHistory: () => void

  // 模态框
  setCommandPaletteOpen: (open: boolean) => void
  setSettingsOpen: (open: boolean) => void
  setShortcutsOpen: (open: boolean) => void
  setGoToLineOpen: (open: boolean) => void
  setQuickOpenOpen: (open: boolean) => void
  setFindReplaceOpen: (open: boolean) => void
  setAboutOpen: (open: boolean) => void

  // 特殊面板
  setScenarioPanelOpen: (open: boolean) => void
  setScenarioResultsOpen: (open: boolean) => void
  setCallersViewOpen: (open: boolean) => void
  setCallersTargetFunc: (func: string) => void

  // 索引
  setIndexProgress: (progress: IndexProgress | null) => void

  // 主题
  setTheme: (theme: 'dark' | 'light') => void
  setSidebarTheme: (theme: 'dark' | 'light' | 'match') => void

  // 计算属性
  canGoBack: () => boolean
  canGoForward: () => boolean
}

export const useUIStore = create<UIState>()(
  persist(
    (set, get) => ({
      // 初始状态
      viewMode: 'split',
      flowDisplayMode: 'graph',
      textDisplayStyle: 'indented',

      leftPanelOpen: true,
      rightPanelOpen: true,
      leftPanelWidth: 220,
      rightPanelWidth: 280,
      leftPanelCollapsed: false,
      rightPanelCollapsed: false,

      navHistory: [],
      navIndex: -1,

      commandPaletteOpen: false,
      settingsOpen: false,
      shortcutsOpen: false,
      goToLineOpen: false,
      quickOpenOpen: false,
      findReplaceOpen: false,
      aboutOpen: false,

      scenarioPanelOpen: false,
      scenarioResultsOpen: false,
      callersViewOpen: false,
      callersTargetFunc: '',

      indexProgress: null,

      theme: 'dark',
      sidebarTheme: 'match',

      // 实现...
    }),
    {
      name: 'flowsight-ui',
      partialize: (state) => ({
        viewMode: state.viewMode,
        flowDisplayMode: state.flowDisplayMode,
        textDisplayStyle: state.textDisplayStyle,
        leftPanelOpen: state.leftPanelOpen,
        rightPanelOpen: state.rightPanelOpen,
        leftPanelWidth: state.leftPanelWidth,
        rightPanelWidth: state.rightPanelWidth,
        theme: state.theme,
        sidebarTheme: state.sidebarTheme,
      }),
    }
  )
)
```

#### 3.2.4 项目状态 (projectStore) - 新增

```typescript
// 文件: app/src/store/projectStore.ts

import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

// 项目状态
interface ProjectState {
  // 项目信息
  projectPath: string | null
  projectInfo: ProjectInfo | null
  indexStats: IndexStats | null

  // 索引状态
  isIndexing: boolean
  indexProgress: IndexProgress | null

  // 配置
  projectConfig: ProjectConfig | null

  // === Actions ===

  // 项目管理
  openProject: (path: string) => Promise<void>
  closeProject: () => void

  // 索引操作
  startIndexing: () => Promise<void>
  reindexProject: () => Promise<void>

  // 配置管理
  loadProjectConfig: () => Promise<ProjectConfig | null>
  saveProjectConfig: (config: ProjectConfig) => Promise<void>
  updateProjectConfig: (updates: Partial<ProjectConfig>) => Promise<void>

  // 工具函数
  getProjectRoot: () => string | null
  isProjectOpen: () => boolean
}

interface ProjectConfig {
  // 分析配置
  analysis: {
    includePaths: string[]
    excludePaths: string[]
    targetArch: string
    optimizationLevel: string
  }

  // 显示配置
  display: {
    maxNodeCount: number
    showExternalCalls: boolean
    showConfidenceIndicators: boolean
    defaultFlowMode: FlowDisplayMode
  }

  // 快捷键配置
  shortcuts: ShortcutConfig
}

export const useProjectStore = create<ProjectState>()((set, get) => ({
  // 初始状态
  projectPath: null,
  projectInfo: null,
  indexStats: null,
  isIndexing: false,
  indexProgress: null,
  projectConfig: null,

  // 实现...
}))
```

#### 3.2.5 Tauri 连接状态 (tauriStore) - 新增

```typescript
// 文件: app/src/store/tauriStore.ts

import { create } from 'zustand'

// Tauri 连接状态
interface TauriState {
  // 连接状态
  isConnected: boolean
  tauriVersion: string
  platform: string
  arch: string

  // 事件监听
  eventListeners: Map<string, () => void>

  // === Actions ===

  // 连接管理
  initialize: () => Promise<void>
  disconnect: () => void

  // 事件订阅
  subscribe: <T>(event: string, callback: (data: T) => void) => () => void
  emit: (event: string, payload?: unknown) => Promise<void>
}

export const useTauriStore = create<TauriState>()((set, get) => ({
  // 初始状态
  isConnected: false,
  tauriVersion: '',
  platform: '',
  arch: '',
  eventListeners: new Map(),

  // 实现...
}))
```

### 3.3 Store 间通信

#### 3.3.1 跨 Store 引用

```typescript
// 在组件中使用多个 store
import { useEditorStore, useAnalysisStore, useUIStore } from '../store'

function MyComponent() {
  const { tabs, activeTabId } = useEditorStore()
  const { result, selectedFunction } = useAnalysisStore()
  const { viewMode, setViewMode } = useUIStore()

  // 组合使用
  const currentAnalysis = result
  const currentFile = tabs.find(t => t.id === activeTabId)

  // ...
}
```

#### 3.3.2 Store 选择器模式

```typescript
// 创建选择器以优化性能
const selectTabs = (state: EditorState) => state.tabs
const selectActiveTab = (state: EditorState) =>
  state.tabs.find(t => t.id === state.activeTabId)

// 使用选择器
const tabs = useEditorStore(selectTabs)
const activeTab = useEditorStore(selectActiveTab)
```

#### 3.3.3 Store 派生状态

```typescript
// 使用 useMemo 从 store 派生状态
function useKnownFunctions() {
  const outlineItems = useAnalysisStore(s => s.outlineItems)
  const flowTrees = useAnalysisStore(s => s.result?.flow_trees)

  return useMemo(() => {
    const names = new Set<string>()
    outlineItems.forEach(item => names.add(item.name))
    // 从 flowTrees 收集更多函数名
    if (flowTrees) {
      const collectFromTree = (nodes: FlowTreeNode[]) => {
        nodes.forEach(node => {
          names.add(node.name)
          if (node.children) collectFromTree(node.children)
        })
      }
      collectFromTree(flowTrees)
    }
    return Array.from(names)
  }, [outlineItems, flowTrees])
}
```

---

## 4. 与 Tauri 后端的通信方式

### 4.1 通信架构

```
┌─────────────────────────────────────────────────────────────┐
│                      Frontend Layer                          │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Services Layer                                     │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │    │
│  │  │tauriService│ │analysisSvc  │ │fileService  │   │    │
│  │  └─────────────┘ └─────────────┘ └─────────────┘   │    │
│  └─────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                     Tauri IPC Layer                          │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  invoke() - 命令调用                                │    │
│  │  listen() - 事件监听                                │    │
│  │  emit() - 事件发送                                  │    │
│  └─────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                      Backend Layer                           │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Rust Commands (src-tauri/src/commands.rs)         │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │    │
│  │  │analyze_file │ │ get_functions│ │ search_sym  │   │    │
│  │  └─────────────┘ └─────────────┘ └─────────────┘   │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Tauri 服务层封装

```typescript
// 文件: app/src/services/tauriService.ts

import { invoke, invoke_legacy } from '@tauri-apps/api/core'
import { listen, EventCallback } from '@tauri-apps/api/event'
import type { AnalysisResult, ProjectInfo, IndexStats } from '../types'

// 命令名称常量
const COMMANDS = {
  // 项目管理
  OPEN_PROJECT: 'open_project',
  CLOSE_PROJECT: 'close_project',
  LIST_DIRECTORY: 'list_directory',

  // 文件操作
  READ_FILE: 'read_file',
  WRITE_FILE: 'write_file',

  // 分析命令
  ANALYZE_FILE: 'analyze_file',
  GET_FUNCTIONS: 'get_functions',
  GET_FUNCTION_DETAIL: 'get_function_detail',
  SEARCH_SYMBOLS: 'search_symbols',

  // 场景分析
  EXECUTE_SCENARIO: 'execute_scenario',

  // 索引操作
  GET_INDEX_STATS: 'get_index_stats',
  START_INDEXING: 'start_indexing',

  // 搜索
  SEARCH_CODE: 'search_code',
} as const

// 错误处理
export class TauriError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: unknown
  ) {
    super(message)
    this.name = 'TauriError'
  }
}

// 封装 Tauri 调用
async function tauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    throw new TauriError(
      `Command "${command}" failed`,
      'TAURI_INVOKE_ERROR',
      error
    )
  }
}

// Tauri 服务接口
export interface TauriService {
  // 项目管理
  openProject(path: string): Promise<ProjectInfo>
  closeProject(): Promise<void>
  listDirectory(path: string, recursive?: boolean): Promise<import('../types').FileNode[]>

  // 文件操作
  readFile(path: string): Promise<string>
  writeFile(path: string, contents: string): Promise<void>

  // 分析命令
  analyzeFile(path: string): Promise<AnalysisResult>
  getFunctions(path: string): Promise<import('../types').OutlineItem[]>
  getFunctionDetail(funcName: string, filePath?: string): Promise<import('../types').FunctionDetail>
  searchSymbols(query: string): Promise<import('../types').SearchResult[]>

  // 场景分析
  executeScenario(filePath: string, scenario: Scenario): Promise<ScenarioResult>

  // 索引
  getIndexStats(): Promise<IndexStats>
  startIndexing(path: string): Promise<void>

  // 搜索
  searchCode(query: string, options?: SearchOptions): Promise<SearchResult[]>

  // 事件
  subscribe<T>(event: string, callback: EventCallback<T>): Promise<() => void>
  emit(event: string, payload?: unknown): Promise<void>
}

// Tauri 服务实现
export const tauriService: TauriService = {
  // 项目管理
  async openProject(path: string) {
    return tauriInvoke<ProjectInfo>(COMMANDS.OPEN_PROJECT, { path })
  },

  async closeProject() {
    return tauriInvoke<void>(COMMANDS.CLOSE_PROJECT)
  },

  async listDirectory(path: string, recursive = false) {
    return tauriInvoke<import('../types').FileNode[]>(COMMANDS.LIST_DIRECTORY, {
      path,
      recursive,
    })
  },

  // 文件操作
  async readFile(path: string) {
    return tauriInvoke<string>(COMMANDS.READ_FILE, { path })
  },

  async writeFile(path: string, contents: string) {
    return tauriInvoke<void>(COMMANDS.WRITE_FILE, { path, contents })
  },

  // 分析命令
  async analyzeFile(path: string) {
    return tauriInvoke<AnalysisResult>(COMMANDS.ANALYZE_FILE, { path })
  },

  async getFunctions(path: string) {
    return tauriInvoke<import('../types').OutlineItem[]>(COMMANDS.GET_FUNCTIONS, {
      path,
    })
  },

  async getFunctionDetail(funcName: string, filePath?: string) {
    return tauriInvoke<import('../types').FunctionDetail>(COMMANDS.GET_FUNCTION_DETAIL, {
      funcName,
      path: filePath,
    })
  },

  async searchSymbols(query: string) {
    return tauriInvoke<import('../types').SearchResult[]>(COMMANDS.SEARCH_SYMBOLS, {
      query,
    })
  },

  // 场景分析
  async executeScenario(filePath: string, scenario: Scenario) {
    return tauriInvoke<ScenarioResult>(COMMANDS.EXECUTE_SCENARIO, {
      filePath,
      scenario,
    })
  },

  // 索引
  async getIndexStats() {
    return tauriInvoke<IndexStats>(COMMANDS.GET_INDEX_STATS)
  },

  async startIndexing(path: string) {
    return tauriInvoke<void>(COMMANDS.START_INDEXING, { path })
  },

  // 搜索
  async searchCode(query: string, options?: SearchOptions) {
    return tauriInvoke<SearchResult[]>(COMMANDS.SEARCH_CODE, {
      query,
      ...options,
    })
  },

  // 事件
  async subscribe<T>(event: string, callback: EventCallback<T>) {
    return listen<T>(event, callback)
  },

  async emit(event: string, payload?: unknown) {
    return invoke_legacy('emit_event', { event, payload })
  },
}
```

### 4.3 分析服务层

```typescript
// 文件: app/src/services/analysisService.ts

import { tauriService } from './tauriService'
import type { AnalysisResult, FlowTreeNode, FunctionDetail } from '../types'

// 分析服务接口
export interface AnalysisService {
  // 基本分析
  analyzeFile(path: string): Promise<AnalysisResult>

  // 函数导航
  getFunctionLocation(funcName: string): Promise<{ file: string; line: number } | null>
  navigateToFunction(funcName: string): Promise<void>

  // 执行流操作
  getCallGraph(rootFunction?: string): Promise<CallGraphData>
  getReverseCallGraph(targetFunction: string): Promise<CallGraphData>

  // 场景分析
  executeScenario(scenario: Scenario): Promise<ScenarioResult>

  // 搜索
  searchSymbols(query: string): Promise<SearchResult[]>
  searchCode(query: string, options?: SearchOptions): Promise<SearchResult[]>
}

// 分析服务实现
export const analysisService: AnalysisService = {
  async analyzeFile(path: string) {
    return tauriService.analyzeFile(path)
  },

  async getFunctionLocation(funcName: string) {
    const detail = await tauriService.getFunctionDetail(funcName)
    if (detail.file && detail.line > 0) {
      return { file: detail.file, line: detail.line }
    }
    return null
  },

  async navigateToFunction(funcName: string) {
    const location = await this.getFunctionLocation(funcName)
    if (location) {
      // 导航到函数位置
      // 由调用方处理导航逻辑
      return location
    }
    throw new Error(`Function ${funcName} not found`)
  },

  async getCallGraph(rootFunction?: string) {
    // 从 flow_trees 构建调用图
    const result = await tauriService.analyzeFile(
      rootFunction ? await this.findFileOfFunction(rootFunction) : ''
    )
    return buildCallGraph(result.flow_trees, rootFunction)
  },

  async getReverseCallGraph(targetFunction: string) {
    // 获取调用指定函数的所有函数
    const results = await tauriService.searchSymbols('')
    // 过滤并构建反向调用图
    // ...
    return { nodes: [], edges: [] }
  },

  async executeScenario(scenario: Scenario) {
    return tauriService.executeScenario(scenario.filePath, scenario)
  },

  async searchSymbols(query: string) {
    if (query.length < 2) return []
    return tauriService.searchSymbols(query)
  },

  async searchCode(query: string, options?: SearchOptions) {
    return tauriService.searchCode(query, options)
  },

  // 辅助方法
  async findFileOfFunction(funcName: string): Promise<string> {
    // 实现查找函数所在文件的逻辑
    return ''
  },
}

// 构建调用图
function buildCallGraph(
  flowTrees: FlowTreeNode[],
  rootFunction?: string
): CallGraphData {
  const nodes: CallGraphNode[] = []
  const edges: CallGraphEdge[] = []

  const processNode = (node: FlowTreeNode, depth: number) => {
    if (depth > 50) return // 防止深度过大

    const nodeId = node.id || node.name
    if (!nodes.find(n => n.id === nodeId)) {
      nodes.push({
        id: nodeId,
        label: node.display_name || node.name,
        type: getNodeType(node.node_type),
        data: node,
      })
    }

    if (node.children) {
      node.children.forEach(child => {
        const childId = child.id || child.name
        if (!nodes.find(n => n.id === childId)) {
          nodes.push({
            id: childId,
            label: child.display_name || child.name,
            type: getNodeType(child.node_type),
            data: child,
          })
        }
        edges.push({
          id: `${nodeId}-${childId}`,
          source: nodeId,
          target: childId,
          confidence: child.confidence,
        })
        processNode(child, depth + 1)
      })
    }
  }

  flowTrees.forEach(tree => processNode(tree, 0))

  return { nodes, edges }
}

function getNodeType(nodeType: unknown): CallGraphNode['type'] {
  if (nodeType === 'EntryPoint') return 'entry'
  if (typeof nodeType === 'object' && nodeType !== null) {
    if ('AsyncCallback' in nodeType) return 'async'
  }
  return 'function'
}
```

### 4.4 事件监听机制

```typescript
// 文件: app/src/services/eventService.ts

import { tauriService } from './tauriService'
import { useProjectStore, useUIStore } from '../store'

// 事件名称常量
const EVENTS = {
  INDEX_PROGRESS: 'index-progress',
  ANALYSIS_PROGRESS: 'analysis-progress',
  FILE_CHANGED: 'file-changed',
  PROJECT_CLOSED: 'project-closed',
} as const

// 事件处理器类型
type EventHandler<T = unknown> = (data: T) => void

// 事件服务
export class EventService {
  private unsubscribers: Map<string, () => void> = new Map()

  // 初始化所有事件监听
  initialize(): void {
    // 索引进度事件
    this.subscribe(EVENTS.INDEX_PROGRESS, (data: IndexProgressEvent) => {
      useUIStore.getState().setIndexProgress(data)
      useProjectStore.getState().setIndexProgress(data)

      if (data.phase === 'done') {
        // 索引完成，刷新统计数据
        this.refreshIndexStats()
      }
    })

    // 分析进度事件
    this.subscribe(EVENTS.ANALYSIS_PROGRESS, (data: AnalysisProgressEvent) => {
      useUIStore.getState().setLoading(true)
      // 更新分析进度
    })

    // 文件变更事件
    this.subscribe(EVENTS.FILE_CHANGED, (data: FileChangedEvent) => {
      // 处理文件变更
      this.handleFileChange(data)
    })

    // 项目关闭事件
    this.subscribe(EVENTS.PROJECT_CLOSED, () => {
      useProjectStore.getState().closeProject()
    })
  }

  // 订阅事件
  subscribe<T>(event: string, handler: EventHandler<T>): void {
    tauriService.subscribe<T>(event, handler).then(unsub => {
      this.unsubscribers.set(event, unsub)
    })
  }

  // 取消订阅
  unsubscribe(event: string): void {
    const unsub = this.unsubscribers.get(event)
    if (unsub) {
      unsub()
      this.unsubscribers.delete(event)
    }
  }

  // 取消所有订阅
  unsubscribeAll(): void {
    this.unsubscribers.forEach(unsub => unsub())
    this.unsubscribers.clear()
  }

  // 刷新索引统计
  private async refreshIndexStats(): Promise<void> {
    try {
      const stats = await tauriService.getIndexStats()
      useProjectStore.getState().setIndexStats(stats)
    } catch (error) {
      console.error('Failed to refresh index stats:', error)
    }
  }

  // 处理文件变更
  private async handleFileChange(data: FileChangedEvent): Promise<void> {
    // 检查当前打开的文件是否被修改
    const { activeTabId, tabs } = useEditorStore.getState()
    const activeTab = tabs.find(t => t.id === activeTabId)

    if (activeTab && activeTab.filePath === data.path) {
      // 提示用户文件已更改
      // 可以选择自动重新加载或提示用户
    }
  }
}

// 导出单例
export const eventService = new EventService()

// 在应用入口初始化
// eventService.initialize()
```

### 4.5 命令类型定义

```typescript
// 文件: app/src/types/commands.ts

// 命令参数和返回值类型定义

// === 项目管理命令 ===

interface OpenProjectCommand {
  path: string
}

interface OpenProjectResult {
  path: string
  files_count: number
  functions_count: number
  structs_count: number
  indexed: boolean
}

// === 文件操作命令 ===

interface ReadFileCommand {
  path: string
}

interface ReadFileResult {
  content: string
}

interface WriteFileCommand {
  path: string
  contents: string
}

// === 分析命令 ===

interface AnalyzeFileCommand {
  path: string
  options?: {
    include_ir?: boolean
    max_depth?: number
    timeout?: number
  }
}

interface AnalyzeFileResult {
  file: string
  functions_count: number
  structs_count: number
  async_handlers_count: number
  entry_points: string[]
  flow_trees: FlowTreeNode[]
}

interface GetFunctionsCommand {
  path: string
}

interface GetFunctionsResult {
  name: string
  return_type: string
  line: number
  is_callback: boolean
}

interface SearchSymbolsCommand {
  query: string
  options?: {
    max_results?: number
    include_callbacks?: boolean
  }
}

interface SearchSymbolsResult {
  name: string
  kind: 'function' | 'struct' | 'variable' | 'macro'
  file: string | null
  line: number | null
  is_callback: boolean
}

// === 场景分析命令 ===

interface ExecuteScenarioCommand {
  filePath: string
  scenario: {
    name: string
    entry_function: string
    bindings: ParameterBinding[]
    options?: ScenarioOptions
  }
}

interface ParameterBinding {
  path: string
  value: string
  type: string
}

interface ScenarioOptions {
  max_path_length?: number
  timeout?: number
  generate_constraints?: boolean
}

interface ExecuteScenarioResult {
  success: boolean
  path: { function: string; line: number; variables: Record<string, string> }[]
  annotated_flow_tree: FlowTreeNode | null
  error: string | null
}

// === 索引命令 ===

interface StartIndexingCommand {
  path: string
  options?: {
    recursive?: boolean
    parallel?: boolean
  }
}

// === 搜索命令 ===

interface SearchCodeCommand {
  query: string
  options?: {
    file_patterns?: string[]
    exclude_patterns?: string[]
    case_sensitive?: boolean
    regex?: boolean
  }
}
```

---

## 5. 性能优化策略

### 5.1 渲染性能优化

#### 5.1.1 组件记忆化

```typescript
// 使用 React.memo 包装纯组件
import { memo } from 'react'

// 文件: app/src/components/FlowView/FlowNode.tsx
export const FlowNode = memo<FlowNodeProps>(({ data }) => {
  // 组件实现
}, (prevProps, nextProps) => {
  // 自定义比较逻辑
  return (
    prevProps.data.name === nextProps.data.name &&
    prevProps.data.isSelected === nextProps.data.isSelected &&
    prevProps.data.isExpanded === nextProps.data.isExpanded
  )
})

// 使用 useMemo 缓存计算结果
function useFlowTreeStats(flowTrees: FlowTreeNode[]) {
  return useMemo(() => {
    let totalNodes = 0
    let totalEdges = 0
    let maxDepth = 0

    const processNode = (node: FlowTreeNode, depth: number) => {
      totalNodes++
      maxDepth = Math.max(maxDepth, depth)
      totalEdges += node.children?.length || 0
      node.children?.forEach(child => processNode(child, depth + 1))
    }

    flowTrees.forEach(tree => processNode(tree, 0))

    return { totalNodes, totalEdges, maxDepth }
  }, [flowTrees])
}

// 使用 useCallback 缓存回调函数
const handleNodeClick = useCallback((nodeId: string, functionName: string) => {
  setSelectedFunction(functionName)
  pushNavHistory({ filePath, selectedFunction: functionName })
}, [filePath])
```

#### 5.1.2 虚拟列表

```typescript
// 文件: app/src/components/Outline/Outline.tsx

import { useVirtualizer } from '@tanstack/react-virtual'

interface OutlineProps {
  items: OutlineItem[]
  onItemClick: (item: OutlineItem) => void
  selectedItem?: string
}

export function Outline({ items, onItemClick, selectedItem }: OutlineProps) {
  const parentRef = useRef<HTMLDivElement>(null)

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 28, // 每项预估高度
    overscan: 5, // 预渲染区域
  })

  return (
    <div ref={parentRef} className="outline-container">
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative',
        }}
      >
        {virtualizer.getVirtualItems().map((virtualItem) => (
          <div
            key={virtualItem.key}
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              height: `${virtualItem.size}px`,
              transform: `translateY(${virtualItem.start}px)`,
            }}
          >
            <OutlineItem
              item={items[virtualItem.index]}
              isSelected={items[virtualItem.index].name === selectedItem}
              onClick={() => onItemClick(items[virtualItem.index])}
            />
          </div>
        ))}
      </div>
    </div>
  )
}
```

#### 5.1.3 延迟加载

```typescript
// 使用 React.lazy 和 Suspense 实现代码分割
import { lazy, Suspense } from 'react'

// 延迟加载大型组件
const CallGraphView = lazy(() => import('./CallGraph/CallGraphView'))
const DiffView = lazy(() => import('./DiffView/DiffView'))
const SettingsDialog = lazy(() => import('./Dialogs/Settings'))

// 在路由或条件渲染中使用
function MainContent({ showCallGraph }: { showCallGraph: boolean }) {
  return (
    <Suspense fallback={<LoadingSpinner />}>
      {showCallGraph ? (
        <CallGraphView />
      ) : (
        <FlowView />
      )}
    </Suspense>
  )
}

// 使用 intersection-observer 实现按需加载
function useLazyLoad<T extends HTMLElement>(
  threshold: number = 0.1
): [React.RefObject<T>, boolean] {
  const ref = useRef<T>(null)
  const [isVisible, setIsVisible] = useState(false)

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        setIsVisible(entry.isIntersecting)
      },
      { threshold }
    )

    if (ref.current) {
      observer.observe(ref.current)
    }

    return () => observer.disconnect()
  }, [threshold])

  return [ref, isVisible]
}
```

### 5.2 状态管理优化

#### 5.2.1 选择器模式

```typescript
// 文件: app/src/store/selectors.ts

// 创建高效的选择器
export const editorSelectors = {
  activeTab: (state: EditorState) =>
    state.tabs.find(t => t.id === state.activeTabId),

  activeTabContent: (state: EditorState) => {
    const tab = state.tabs.find(t => t.id === state.activeTabId)
    return tab?.content ?? ''
  },

  activeTabPath: (state: EditorState) => {
    const tab = state.tabs.find(t => t.id === state.activeTabId)
    return tab?.filePath ?? ''
  },

  isDirty: (state: EditorState) =>
    state.tabs.some(t => t.isDirty),

  tabCount: (state: EditorState) =>
    state.tabs.length,

  dirtyTabs: (state: EditorState) =>
    state.tabs.filter(t => t.isDirty),
}

// 使用选择器
const activeTab = useEditorStore(editorSelectors.activeTab)
const dirtyCount = useEditorStore(editorSelectors.isDirty)
```

#### 5.2.2 状态分片

```typescript
// 将大型状态拆分为多个小状态
interface LargeState {
  // 大量数据
  largeArray: BigData[]
  // 频繁更新的数据
  selection: string | null
  // 静态配置
  config: Config
}

// 拆分为多个 store
interface DataState {
  largeArray: BigData[]
  loadData: () => Promise<void>
}

interface UIState {
  selection: string | null
  setSelection: (id: string | null) => void
}

interface ConfigState {
  config: Config
  updateConfig: (updates: Partial<Config>) => void
}
```

### 5.3 数据处理优化

#### 5.3.1 防抖和节流

```typescript
// 文件: app/src/hooks/useDebounce.ts

import { useCallback, useRef, useEffect } from 'react'

// 防抖 Hook
export function useDebounce<T extends (...args: unknown[]) => unknown>(
  callback: T,
  delay: number
): T {
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  return useCallback((...args: Parameters<T>) => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
    }

    timeoutRef.current = setTimeout(() => {
      callback(...args)
    }, delay)
  }, [callback, delay]) as T
}

// 节流 Hook
export function useThrottle<T extends (...args: unknown[]) => unknown>(
  callback: T,
  limit: number
): T {
  const lastRunRef = useRef<number>(0)
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  return useCallback((...args: Parameters<T>) => {
    const now = Date.now()

    if (now - lastRunRef.current >= limit) {
      callback(...args)
      lastRunRef.current = now
    } else {
      if (timeoutRef.current) return

      timeoutRef.current = setTimeout(() => {
        callback(...args)
        lastRunRef.current = Date.now()
      }, limit - (now - lastRunRef.current))
    }
  }, [callback, limit]) as T
}

// 使用示例
const debouncedAnalyze = useDebounce(async (path: string) => {
  const result = await tauriService.analyzeFile(path)
  useAnalysisStore.getState().setResult(result)
}, 500)
```

#### 5.3.2 数据分页和增量加载

```typescript
// 文件: app/src/hooks/usePagination.ts

interface UsePaginationOptions<T> {
  items: T[]
  pageSize: number
  initialPage?: number
}

interface UsePaginationReturn<T> {
  // 当前页数据
  currentItems: T[]
  // 总页数
  totalPages: number
  // 当前页码
  currentPage: number
  // 导航
  nextPage: () => void
  prevPage: () => void
  goToPage: (page: number) => void
  // 状态
  hasNextPage: boolean
  hasPrevPage: boolean
}

export function usePagination<T>(
  options: UsePaginationOptions<T>
): UsePaginationReturn<T> {
  const { items, pageSize, initialPage = 1 } = options
  const [currentPage, setCurrentPage] = useState(initialPage)

  const totalPages = Math.ceil(items.length / pageSize)

  const getItemsForPage = useCallback((page: number) => {
    const start = (page - 1) * pageSize
    const end = start + pageSize
    return items.slice(start, end)
  }, [items, pageSize])

  const nextPage = useCallback(() => {
    setCurrentPage(prev => Math.min(prev + 1, totalPages))
  }, [totalPages])

  const prevPage = useCallback(() => {
    setCurrentPage(prev => Math.max(prev - 1, 1))
  }, [])

  const goToPage = useCallback((page: number) => {
    setCurrentPage(Math.max(1, Math.min(page, totalPages)))
  }, [totalPages])

  return {
    currentItems: getItemsForPage(currentPage),
    totalPages,
    currentPage,
    nextPage,
    prevPage,
    goToPage,
    hasNextPage: currentPage < totalPages,
    hasPrevPage: currentPage > 1,
  }
}
```

#### 5.3.3 Web Worker 处理大数据

```typescript
// 文件: app/src/workers/flowTreeWorker.ts

// 创建一个 Web Worker 处理大量数据

// flowTreeWorker.ts
self.onmessage = (event: MessageEvent) => {
  const { type, payload } = event.data

  switch (type) {
    case 'BUILD_GRAPH':
      const graph = buildFlowGraph(payload.flowTrees)
      self.postMessage({ type: 'GRAPH_BUILT', payload: graph })
      break

    case 'CALCULATE_METRICS':
      const metrics = calculateMetrics(payload.flowTrees)
      self.postMessage({ type: 'METRICS_CALCULATED', payload: metrics })
      break

    case 'FIND_PATHS':
      const paths = findAllPaths(payload.flowTrees, payload.target)
      self.postMessage({ type: 'PATHS_FOUND', payload: paths })
      break
  }
}

function buildFlowGraph(flowTrees: FlowTreeNode[]): CallGraphData {
  // 实现...
  return { nodes: [], edges: [] }
}

function calculateMetrics(flowTrees: FlowTreeNode[]): FlowMetrics {
  // 实现...
  return { nodeCount: 0, edgeCount: 0, maxDepth: 0 }
}

// 在组件中使用
function useFlowTreeWorker() {
  const workerRef = useRef<Worker | null>(null)
  const [result, setResult] = useState<unknown>(null)

  useEffect(() => {
    workerRef.current = new Worker(
      new URL('../workers/flowTreeWorker.ts', import.meta.url)
    )

    workerRef.current.onmessage = (event) => {
      const { type, payload } = event.data
      if (type === 'GRAPH_BUILT') {
        setResult(payload)
      }
    }

    return () => {
      workerRef.current?.terminate()
    }
  }, [])

  const buildGraph = useCallback((flowTrees: FlowTreeNode[]) => {
    workerRef.current?.postMessage({
      type: 'BUILD_GRAPH',
      payload: { flowTrees },
    })
  }, [])

  return { buildGraph, result }
}
```

### 5.4 流程图性能优化

#### 5.4.1 节点懒加载

```typescript
// 文件: app/src/components/FlowView/FlowView.tsx

import { useMemo, useState, useCallback } from 'react'
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  Panel,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'

interface FlowViewProps {
  flowTrees: FlowTreeNode[]
  onNodeClick?: (nodeId: string, functionName: string) => void
  nodeLimit?: number // 最大节点数限制
}

// 节点限制配置
const DEFAULT_NODE_LIMIT = 500
const EXPANDED_NODE_LIMIT = 1000

export function FlowView({
  flowTrees,
  onNodeClick,
  nodeLimit = DEFAULT_NODE_LIMIT,
}: FlowViewProps) {
  const [visibleNodes, setVisibleNodes] = useState<string[]>([])
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set())

  // 构建节点和边
  const { initialNodes, initialEdges } = useMemo(() => {
    const result = processFlowTrees(flowTrees, nodeLimit)
    return result
  }, [flowTrees, nodeLimit])

  // 展开/收起节点
  const handleNodeExpand = useCallback((nodeId: string) => {
    setExpandedNodes(prev => {
      const next = new Set(prev)
      if (next.has(nodeId)) {
        next.delete(nodeId)
      } else {
        next.add(nodeId)
      }
      return next
    })
  }, [])

  return (
    <div style={{ width: '100%', height: '100%' }}>
      <ReactFlow
        nodes={initialNodes}
        edges={initialEdges}
        onNodeClick={(_, node) => {
          onNodeClick?.(node.id, node.data.name)
        }}
        // 性能优化配置
        nodesDraggable={false}
        nodesConnectable={false}
        fitView
        // 虚拟化支持
        virtualization
        // 延迟渲染
        onlyRenderVisibleNodes
      >
        <Background />
        <Controls />
        <MiniMap />

        {/* 性能提示面板 */}
        <Panel position="top-right">
          {initialNodes.length >= nodeLimit && (
            <div className="performance-warning">
              显示 {initialNodes.length} / {flowTrees.length} 个节点
              <button onClick={() => setNodeLimit(EXPANDED_NODE_LIMIT)}>
                显示全部
              </button>
            </div>
          )}
        </Panel>
      </ReactFlow>
    </div>
  )
}

// 处理流程树，限制节点数量
function processFlowTrees(
  flowTrees: FlowTreeNode[],
  limit: number
): { nodes: Node[]; edges: Edge[] } {
  const nodes: Node[] = []
  const edges: Edge[] = []
  let count = 0

  const processNode = (node: FlowTreeNode, parentId?: string) => {
    if (count >= limit) return

    const nodeId = node.id || node.name
    nodes.push({
      id: nodeId,
      type: 'flowNode',
      position: { x: 0, y: 0 }, // 由布局算法计算
      data: {
        name: node.name,
        displayName: node.display_name,
        nodeType: node.node_type,
        location: node.location,
        confidence: node.confidence,
      },
    })

    if (parentId) {
      edges.push({
        id: `${parentId}-${nodeId}`,
        source: parentId,
        target: nodeId,
        type: 'smoothstep',
      })
    }

    count++

    node.children?.forEach(child => processNode(child, nodeId))
  }

  flowTrees.forEach(tree => processNode(tree))

  return { nodes, edges }
}
```

#### 5.4.2 布局优化

```typescript
// 文件: app/src/utils/flowLayout.ts

import dagre from 'dagre'
import type { Node, Edge, Position } from '@xyflow/react'

// 使用 dagre 进行自动布局
export function layoutFlowGraph(
  nodes: Node[],
  edges: Edge[],
  options?: LayoutOptions
): Node[] {
  const {
    nodeWidth = 180,
    nodeHeight = 60,
    rankDir = 'LR', // 从左到右
    rankSep = 100,
    nodeSep = 50,
  } = options || {}

  const g = new dagre.graphlib.Graph()
  g.setGraph({
    rankdir: rankDir,
    nodesep: nodeSep,
    ranksep: rankSep,
    marginx: 20,
    marginy: 20,
  })

  // 添加节点
  nodes.forEach(node => {
    g.setNode(node.id, { width: nodeWidth, height: nodeHeight })
  })

  // 添加边
  edges.forEach(edge => {
    g.setEdge(edge.source, edge.target)
  })

  // 计算布局
  dagre.layout(g)

  // 应用布局结果
  return nodes.map(node => {
    const nodeData = g.node(node.id)
    return {
      ...node,
      position: {
        x: nodeData.x - nodeWidth / 2,
        y: nodeData.y - nodeHeight / 2,
      },
      targetPosition: Position.Left,
      sourcePosition: Position.Right,
    }
  })
}

// 带分支的布局
export function layoutWithBranches(
  nodes: Node[],
  edges: Edge[],
  branches: BranchNode[]
): Node[] {
  // 对分支节点进行特殊处理
  // ...
  return layoutFlowGraph(nodes, edges)
}
```

### 5.5 内存优化

#### 5.5.1 图片和资源优化

```typescript
// 懒加载图片
function LazyImage({ src, alt }: { src: string; alt: string }) {
  const [isLoaded, setIsLoaded] = useState(false)
  const imgRef = useRef<HTMLImageElement>(null)

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          const img = imgRef.current
          if (img && img.dataset.src) {
            img.src = img.dataset.src
            img.onload = () => setIsLoaded(true)
            observer.unobserve(img)
          }
        }
      },
      { rootMargin: '100px' }
    )

    const img = imgRef.current
    if (img) observer.observe(img)

    return () => observer.disconnect()
  }, [])

  return (
    <img
      ref={imgRef}
      data-src={src}
      alt={alt}
      className={`lazy-image ${isLoaded ? 'loaded' : ''}`}
    />
  )
}

// 释放大型数据引用
function useLargeDataCleanup(data: BigData[]) {
  useEffect(() => {
    return () => {
      // 组件卸载时清理大型数据
      data.length = 0
    }
  }, [data])
}
```

#### 5.5.2 缓存策略

```typescript
// 文件: app/src/utils/cache.ts

// LRU 缓存实现
export class LRUCache<K, V> {
  private maxSize: number
  private cache: Map<K, V>

  constructor(maxSize: number = 100) {
    this.maxSize = maxSize
    this.cache = new Map()
  }

  get(key: K): V | undefined {
    const value = this.cache.get(key)
    if (value !== undefined) {
      // 移动到末尾（最近使用）
      this.cache.delete(key)
      this.cache.set(key, value)
    }
    return value
  }

  set(key: K, value: V): void {
    if (this.cache.has(key)) {
      this.cache.delete(key)
    } else if (this.cache.size >= this.maxSize) {
      // 删除最久未使用的项
      const firstKey = this.cache.keys().next().value
      this.cache.delete(firstKey)
    }
    this.cache.set(key, value)
  }

  has(key: K): boolean {
    return this.cache.has(key)
  }

  delete(key: K): void {
    this.cache.delete(key)
  }

  clear(): void {
    this.cache.clear()
  }

  size(): number {
    return this.cache.size
  }
}

// 分析结果缓存
export const analysisCache = new LRUCache<string, AnalysisResult>(50)

// 符号搜索缓存
export const symbolCache = new LRUCache<string, SearchResult[]>(100)
```

### 5.6 网络优化

#### 5.6.1 请求批处理

```typescript
// 请求队列和批处理
class RequestBatcher {
  private queue: Array<{
    id: string
    promise: (resolve: (value: unknown) => void, reject: (reason: unknown) => void) => void
  }> = []
  private batchTimeout: ReturnType<typeof setTimeout> | null = null

  async add<T>(
    id: string,
    request: () => Promise<T>
  ): Promise<T> {
    return new Promise((resolve, reject) => {
      this.queue.push({ id, promise: async (res, rej) => {
        try {
          const result = await request()
          res(result)
        } catch (error) {
          rej(error)
        }
      }})

      if (!this.batchTimeout) {
        this.batchTimeout = setTimeout(() => this.processBatch(), 16) // ~60fps
      }
    })
  }

  private async processBatch() {
    const batch = this.queue.splice(0)
    this.batchTimeout = null

    if (batch.length === 0) return

    // 如果只有一个请求，直接执行
    if (batch.length === 1) {
      const [item] = batch
      item.promise(
        () => {},
        () => {}
      )
      return
    }

    // 批量请求（如果后端支持）
    try {
      const results = await this.executeBatch(batch.map(b => b.id))
      // 分发结果
    } catch (error) {
      // 处理错误
    }
  }

  private async executeBatch(ids: string[]): Promise<unknown[]> {
    // 实现批量执行逻辑
    return []
  }
}

export const requestBatcher = new RequestBatcher()
```

---

## 6. 附录

### 6.1 目录结构总览

```
flowsight/
├── app/
│   ├── src/
│   │   ├── components/           # React 组件
│   │   ├── store/                # Zustand 状态管理
│   │   ├── hooks/                # 自定义 Hooks
│   │   ├── services/             # 服务层
│   │   ├── utils/                # 工具函数
│   │   ├── types/                # 类型定义
│   │   ├── styles/               # 样式文件
│   │   ├── App.tsx               # 应用入口
│   │   └── main.tsx              # 渲染入口
│   ├── src-tauri/                # Tauri 后端
│   └── package.json
├── crates/                       # Rust 核心模块
├── docs/                         # 文档
│   ├── design/                   # 设计文档
│   └── architecture/             # 架构文档
└── README.md
```

### 6.2 关键类型定义索引

| 文件 | 主要类型 | 描述 |
|------|----------|------|
| `types.ts` | `FlowTreeNode`, `AnalysisResult` | 核心数据模型 |
| `types/analysis.ts` | `FunctionDetail`, `SearchResult` | 分析相关类型 |
| `types/flow.ts` | `FlowNodeData`, `CallGraphData` | 流程图类型 |
| `types/editor.ts` | `TabData`, `Decoration` | 编辑器类型 |
| `types/commands.ts` | 命令参数/返回值类型 | Tauri 命令类型 |

### 6.3 Store 索引

| Store | 主要状态 | 职责 |
|-------|----------|------|
| `editorStore` | 文件内容、标签页、项目 | 编辑器状态管理 |
| `analysisStore` | 分析结果、函数详情、搜索 | 分析状态管理 |
| `uiStore` | 视图模式、面板状态、主题 | UI 状态管理 |
| `projectStore` | 项目信息、索引状态 | 项目状态管理 |
| `tauriStore` | 连接状态、事件监听 | Tauri 连接管理 |

### 6.4 性能指标目标

| 指标 | 目标值 | 优化策略 |
|------|--------|----------|
| 首屏加载时间 | < 2s | 代码分割、懒加载 |
| 交互响应时间 | < 100ms | 状态记忆化、防抖 |
| 流程图渲染 | < 500ms | 虚拟化、节点限制 |
| 内存占用 | < 500MB | 及时清理、LRU 缓存 |
| 大文件编辑 | 无卡顿 | Monaco Editor 优化 |

---

## 7. 修订历史

| 版本 | 日期 | 作者 | 变更说明 |
|------|------|------|----------|
| 1.0.0 | 2024-01-24 | FlowSight Team | 初始发布 |

---

*本文档由 FlowSight 团队维护，如有问题请提交 Issue 或 Pull Request。*
