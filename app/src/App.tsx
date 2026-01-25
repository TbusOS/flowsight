/**
 * FlowSight - 跨平台执行流可视化 IDE
 */

import { useState, useCallback, useEffect, useRef, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { FlowView, FlowTextView } from './components/FlowView'
import { CodeEditor } from './components/Editor'
import { FileTree, FileNode } from './components/Explorer'
import { Outline, OutlineItem } from './components/Outline'
import { CommandPalette } from './components/CommandPalette'
import { TabBar, Tab } from './components/Tabs'
import { Breadcrumb } from './components/Breadcrumb'
import { StatusBar } from './components/StatusBar'
import { Welcome } from './components/Welcome'
import { Settings, defaultSettings, type AppSettings } from './components/Settings'
import { FindReplace, type FindMatch } from './components/FindReplace'
import { KeyboardShortcuts } from './components/KeyboardShortcuts'
import { ScenarioPanel } from './components/ScenarioPanel'
import { ScenarioResults } from './components/ScenarioResults'
import { CallersView } from './components/CallersView'
import { NodeDetailPanel, type NodeDetailData } from './components/NodeDetailPanel'
import { LlvmIrPanel, type LlvmIrParseResult } from './components/LlvmIrPanel'
import { GoToLine } from './components/GoToLine'
import { ToastContainer, useToast } from './components/Toast'
import { AboutDialog } from './components/AboutDialog'
import { ConfirmDialog } from './components/ConfirmDialog'
import { QuickOpen } from './components/QuickOpen'
import { addRecentFile, getRecentFiles } from './utils/recentFiles'
import { 
  AnalysisResult, 
  FlowTreeNode, 
  ProjectInfo, 
  SearchResult, 
  IndexStats,
  FunctionDetail 
} from './types'

type ViewMode = 'flow' | 'code' | 'split'
type FlowDisplayMode = 'graph' | 'text'  // 执行流显示模式：图形 vs 文本

// 导航历史记录项
interface NavigationEntry {
  filePath: string
  selectedFunction: string | null
  line?: number
  timestamp: number
}

function App() {
  const [result, setResult] = useState<AnalysisResult | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedFunction, setSelectedFunction] = useState<string | null>(null)
  const [filePath, setFilePath] = useState('')
  const [fileContent, setFileContent] = useState('')
  
  // 多标签页状态
  interface TabData extends Tab {
    content: string
    analysisResult?: AnalysisResult | null
  }
  const [tabs, setTabs] = useState<TabData[]>([])
  const [activeTabId, setActiveTabId] = useState<string | null>(null)
  // goToLine 包含时间戳，确保每次点击都能触发跳转
  const [goToLine, setGoToLine] = useState<{ line: number; timestamp: number } | undefined>()
  const [viewMode, setViewMode] = useState<ViewMode>('split')
  const [flowDisplayMode, setFlowDisplayMode] = useState<FlowDisplayMode>('graph')
  
  // Project state
  const [project, setProject] = useState<ProjectInfo | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<SearchResult[]>([])
  const [indexStats, setIndexStats] = useState<IndexStats | null>(null)
  const [functionDetail, setFunctionDetail] = useState<FunctionDetail | null>(null)
  const [fileTree, setFileTree] = useState<FileNode[]>([])
  const [outlineItems, setOutlineItems] = useState<OutlineItem[]>([])

  // 导航历史状态
  const [navHistory, setNavHistory] = useState<NavigationEntry[]>([])
  const [navIndex, setNavIndex] = useState(-1)
  const isNavigating = useRef(false) // 防止导航时重复记录历史

  // Panel visibility state
  const [leftPanelOpen, setLeftPanelOpen] = useState(true)
  const [rightPanelOpen, setRightPanelOpen] = useState(true)
  
  // 命令面板状态
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false)
  
  // 设置面板状态
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [appSettings, setAppSettings] = useState<AppSettings>(defaultSettings)
  
  // 应用主题到 document
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', appSettings.theme)
  }, [appSettings.theme])

  // 监听索引进度事件
  useEffect(() => {
    const unlisten = listen<{
      phase: string
      current: number
      total: number
      message: string
      files?: number
      functions?: number
    }>('index-progress', (event) => {
      setIndexProgress(event.payload)
      if (event.payload.phase === 'done') {
        // 更新索引统计
        setIndexStats({
          files: event.payload.files || event.payload.total,
          functions: event.payload.functions || 0,
          structs: 0,
        })
        // 3秒后清除进度
        setTimeout(() => setIndexProgress(null), 3000)
      }
    })
    return () => { unlisten.then(fn => fn()) }
  }, [])
  
  // 查找替换状态
  const [findReplaceOpen, setFindReplaceOpen] = useState(false)
  const [, setFindMatches] = useState<FindMatch[]>([])
  
  // 快捷键帮助状态
  const [shortcutsOpen, setShortcutsOpen] = useState(false)
  
  // 跳转行号状态
  const [goToLineOpen, setGoToLineOpen] = useState(false)
  
  // Toast 通知
  const { toasts, removeToast, success, error: showError, info } = useToast()
  
  // 拖放文件状态
  const [isDragging, setIsDragging] = useState(false)
  const [droppedFile, setDroppedFile] = useState<string | null>(null)
  
  // 关于对话框状态
  const [aboutOpen, setAboutOpen] = useState(false)
  
  // 关闭未保存标签确认对话框状态
  const [closeConfirm, setCloseConfirm] = useState<{ tabId: string; fileName: string } | null>(null)
  
  // 快速打开状态
  const [quickOpenOpen, setQuickOpenOpen] = useState(false)
  
  // 场景化数据流分析状态
  const [scenarioPanelOpen, setScenarioPanelOpen] = useState(false)
  const [scenarioResultsOpen, setScenarioResultsOpen] = useState(false)
  const [currentScenarioName, setCurrentScenarioName] = useState('')
  const [scenarioResults, setScenarioResults] = useState<{
    path: string[]
    states: { location: string; variables: Record<string, string> }[]
  } | null>(null)
  
  // 调用者分析状态
  const [callersViewOpen, setCallersViewOpen] = useState(false)
  const [_callersTargetFunc] = useState('')

  // 节点详情面板状态
  const [_nodeDetailOpen, setNodeDetailOpen] = useState(false)
  const [nodeDetailData, setNodeDetailData] = useState<NodeDetailData | null>(null)

  // LLVM IR 面板状态
  const [llvmIrData, setLlvmIrData] = useState<LlvmIrParseResult | null>(null)
  const [selectedLlvmFunction, setSelectedLlvmFunction] = useState<string | null>(null)

  // 索引进度状态
  const [indexProgress, setIndexProgress] = useState<{
    phase: string
    current: number
    total: number
    message: string
  } | null>(null)
  
  // 拖放文件处理
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(true)
  }, [])
  
  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)
  }, [])
  
  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)

    const files = Array.from(e.dataTransfer.files)
    if (files.length === 0) return

    // 获取文件路径 (Tauri 需要特殊处理)
    const file = files[0]
    const path = (file as any).path as string | undefined

    if (path) {
      info(`正在打开: ${file.name}`)
      setDroppedFile(path)
    } else {
      showError('无法获取文件路径，请使用菜单打开文件')
    }
  }, [info, showError])
  
  // Panel width state (percentage)
  const [leftPanelWidth, setLeftPanelWidth] = useState(220)
  const [rightPanelWidth, setRightPanelWidth] = useState(280)
  
  // Resizing state
  const isResizingLeft = useRef(false)
  const isResizingRight = useRef(false)

  // Handle mouse move for resizing
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (isResizingLeft.current) {
        const newWidth = Math.max(180, Math.min(400, e.clientX))
        setLeftPanelWidth(newWidth)
      }
      if (isResizingRight.current) {
        const newWidth = Math.max(200, Math.min(450, window.innerWidth - e.clientX))
        setRightPanelWidth(newWidth)
      }
    }

    const handleMouseUp = () => {
      isResizingLeft.current = false
      isResizingRight.current = false
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
    
    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [])

  const startResizeLeft = () => {
    isResizingLeft.current = true
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
  }

  const startResizeRight = () => {
    isResizingRight.current = true
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
  }

  // 导航历史管理
  const pushNavHistory = useCallback((entry: Omit<NavigationEntry, 'timestamp'>) => {
    if (isNavigating.current) return // 正在导航时不记录
    if (!entry.filePath) return // 没有文件时不记录
    
    setNavHistory(prev => {
      // 如果和当前位置相同，不记录
      const current = prev[navIndex]
      if (current && 
          current.filePath === entry.filePath && 
          current.selectedFunction === entry.selectedFunction &&
          current.line === entry.line) {
        return prev
      }
      
      // 清除前进历史（从当前位置之后的所有记录）
      const newHistory = prev.slice(0, navIndex + 1)
      // 添加新记录
      newHistory.push({ ...entry, timestamp: Date.now() })
      // 限制历史长度
      if (newHistory.length > 50) {
        newHistory.shift()
        return newHistory
      }
      return newHistory
    })
    setNavIndex(prev => Math.min(prev + 1, 49))
  }, [navIndex])

  const canGoBack = navIndex > 0
  const canGoForward = navIndex < navHistory.length - 1

  const goBack = useCallback(async () => {
    if (!canGoBack) return
    
    isNavigating.current = true
    const newIndex = navIndex - 1
    const entry = navHistory[newIndex]
    
    setNavIndex(newIndex)
    
    // 如果是不同文件，需要加载文件
    if (entry.filePath !== filePath) {
      try {
        const content = await invoke<string>('read_file', { path: entry.filePath })
        setFileContent(content)
        setFilePath(entry.filePath)
        
        // 分析文件
        const ext = entry.filePath.split('.').pop()?.toLowerCase()
        if (['c', 'h', 'cpp', 'hpp', 'cc', 'cxx'].includes(ext || '')) {
          const analysis = await invoke<AnalysisResult>('analyze_file', { path: entry.filePath })
          setResult(analysis)
          
          const functions = await invoke<Array<{
            name: string
            return_type: string
            line: number
            is_callback: boolean
          }>>('get_functions', { path: entry.filePath })
          
          setOutlineItems(functions.map(f => ({
            name: f.name,
            kind: 'function' as const,
            line: f.line,
            isCallback: f.is_callback,
            returnType: f.return_type,
          })))
        }
      } catch (e) {
        console.error('Navigation error:', e)
      }
    }
    
    setSelectedFunction(entry.selectedFunction)
    if (entry.line) {
      setGoToLine({ line: entry.line, timestamp: Date.now() })
    }
    
    isNavigating.current = false
  }, [canGoBack, navIndex, navHistory, filePath])

  const goForward = useCallback(async () => {
    if (!canGoForward) return
    
    isNavigating.current = true
    const newIndex = navIndex + 1
    const entry = navHistory[newIndex]
    
    setNavIndex(newIndex)
    
    // 如果是不同文件，需要加载文件
    if (entry.filePath !== filePath) {
      try {
        const content = await invoke<string>('read_file', { path: entry.filePath })
        setFileContent(content)
        setFilePath(entry.filePath)
        
        // 分析文件
        const ext = entry.filePath.split('.').pop()?.toLowerCase()
        if (['c', 'h', 'cpp', 'hpp', 'cc', 'cxx'].includes(ext || '')) {
          const analysis = await invoke<AnalysisResult>('analyze_file', { path: entry.filePath })
          setResult(analysis)
          
          const functions = await invoke<Array<{
            name: string
            return_type: string
            line: number
            is_callback: boolean
          }>>('get_functions', { path: entry.filePath })
          
          setOutlineItems(functions.map(f => ({
            name: f.name,
            kind: 'function' as const,
            line: f.line,
            isCallback: f.is_callback,
            returnType: f.return_type,
          })))
        }
      } catch (e) {
        console.error('Navigation error:', e)
      }
    }
    
    setSelectedFunction(entry.selectedFunction)
    if (entry.line) {
      setGoToLine({ line: entry.line, timestamp: Date.now() })
    }
    
    isNavigating.current = false
  }, [canGoForward, navIndex, navHistory, filePath])

  // 键盘快捷键支持
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+P or Cmd+P 打开命令面板
      if ((e.ctrlKey || e.metaKey) && e.key === 'p') {
        e.preventDefault()
        setCommandPaletteOpen(true)
      }
      // Ctrl+F 打开查找
      if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
        e.preventDefault()
        setFindReplaceOpen(true)
      }
      // Ctrl+H 打开查找替换
      if ((e.ctrlKey || e.metaKey) && e.key === 'h') {
        e.preventDefault()
        setFindReplaceOpen(true)
      }
      // ? 打开快捷键帮助 (只有在没有焦点在输入框时)
      if (e.key === '?' && !['INPUT', 'TEXTAREA'].includes((e.target as HTMLElement)?.tagName)) {
        e.preventDefault()
        setShortcutsOpen(true)
      }
      // Ctrl+B 切换侧边栏
      if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault()
        setLeftPanelOpen(prev => !prev)
      }
      // Ctrl+G 跳转行号
      if ((e.ctrlKey || e.metaKey) && e.key === 'g') {
        e.preventDefault()
        setGoToLineOpen(true)
      }
      // Ctrl+E 快速打开最近文件
      if ((e.ctrlKey || e.metaKey) && e.key === 'e') {
        e.preventDefault()
        setQuickOpenOpen(true)
        setGoToLineOpen(true)
      }
      // Ctrl+W 关闭当前标签
      if ((e.ctrlKey || e.metaKey) && e.key === 'w') {
        e.preventDefault()
        if (activeTabId) {
          closeTab(activeTabId)
        }
      }
      // Ctrl+1/2/3 切换视图
      if ((e.ctrlKey || e.metaKey) && e.key === '1') {
        e.preventDefault()
        setViewMode('code')
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '2') {
        e.preventDefault()
        setViewMode('split')
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '3') {
        e.preventDefault()
        setViewMode('flow')
      }
      // Alt+Left or Cmd+[ 后退
      if ((e.altKey && e.key === 'ArrowLeft') || (e.metaKey && e.key === '[')) {
        e.preventDefault()
        goBack()
      }
      // Alt+Right or Cmd+] 前进
      if ((e.altKey && e.key === 'ArrowRight') || (e.metaKey && e.key === ']')) {
        e.preventDefault()
        goForward()
      }
      // 鼠标侧键支持 (通过 keyCode 3 和 4，但这在 keydown 中不可用，需要 mouse event)
    }
    
    // 鼠标侧键支持
    const handleMouseButton = (e: MouseEvent) => {
      if (e.button === 3) { // 后退键
        e.preventDefault()
        goBack()
      } else if (e.button === 4) { // 前进键
        e.preventDefault()
        goForward()
      }
    }
    
    document.addEventListener('keydown', handleKeyDown)
    document.addEventListener('mouseup', handleMouseButton)
    
    return () => {
      document.removeEventListener('keydown', handleKeyDown)
      document.removeEventListener('mouseup', handleMouseButton)
    }
  }, [goBack, goForward])

  // Open project directory
  const handleOpenProject = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择项目目录'
      })
      
      if (selected && typeof selected === 'string') {
        setLoading(true)
        setError(null)
        const info = await invoke<ProjectInfo>('open_project', { path: selected })
        setProject(info)
        addRecentFile(selected, true) // 记录最近项目
        const stats = await invoke<IndexStats>('get_index_stats')
        setIndexStats(stats)
        
        // Load file tree (non-recursive for performance)
        const tree = await invoke<FileNode[]>('list_directory', { path: selected, recursive: false })
        setFileTree(tree)
        setLeftPanelOpen(true) // Open left panel when project loaded
      }
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  // Search symbols
  useEffect(() => {
    if (searchQuery.length < 2) {
      setSearchResults([])
      return
    }

    const timer = setTimeout(async () => {
      try {
        const results = await invoke<SearchResult[]>('search_symbols', { query: searchQuery })
        setSearchResults(results)
      } catch (e) {
        console.error('Search error:', e)
      }
    }, 300)

    return () => clearTimeout(timer)
  }, [searchQuery])

  // === 标签页管理 ===
  
  // 生成唯一 Tab ID
  const generateTabId = () => `tab-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
  
  // 打开文件到标签页（如果已存在则切换）
  const openFileInTab = useCallback(async (path: string) => {
    // 检查是否已打开
    const existingTab = tabs.find(t => t.filePath === path)
    if (existingTab) {
      setActiveTabId(existingTab.id)
      setFilePath(existingTab.filePath)
      setFileContent(existingTab.content)
      if (existingTab.analysisResult) {
        setResult(existingTab.analysisResult)
      }
      return existingTab.id
    }
    
    // 加载新文件
    try {
      const content = await invoke<string>('read_file', { path })
      const fileName = path.split('/').pop() || path
      
      const newTab: TabData = {
        id: generateTabId(),
        filePath: path,
        fileName,
        content,
        isDirty: false,
        analysisResult: null,
      }
      
      setTabs(prev => [...prev, newTab])
      setActiveTabId(newTab.id)
      setFilePath(path)
      setFileContent(content)
      addRecentFile(path, false) // 记录最近文件
      
      return newTab.id
    } catch (err) {
      console.error('打开文件失败:', err)
      return null
    }
  }, [tabs])
  
  // 关闭标签页
  const closeTab = useCallback(async (tabId: string, force = false) => {
    const tabIndex = tabs.findIndex(t => t.id === tabId)
    if (tabIndex === -1) return
    
    const tab = tabs[tabIndex]
    
    // 如果有未保存的更改，显示确认对话框
    if (tab.isDirty && !force) {
      setCloseConfirm({ tabId, fileName: tab.fileName })
      return
    }
    
    const newTabs = tabs.filter(t => t.id !== tabId)
    setTabs(newTabs)
    
    // 如果关闭的是当前标签，切换到相邻标签
    if (activeTabId === tabId) {
      if (newTabs.length === 0) {
        setActiveTabId(null)
        setFilePath('')
        setFileContent('')
        setResult(null)
        setOutlineItems([])
      } else {
        // 切换到左边的标签，如果是第一个则切换到右边
        const newIndex = Math.max(0, tabIndex - 1)
        const newActiveTab = newTabs[newIndex]
        setActiveTabId(newActiveTab.id)
        setFilePath(newActiveTab.filePath)
        setFileContent(newActiveTab.content)
        if (newActiveTab.analysisResult) {
          setResult(newActiveTab.analysisResult)
        }
      }
    }
  }, [tabs, activeTabId])
  
  // 切换标签页
  const switchTab = useCallback((tabId: string) => {
    const tab = tabs.find(t => t.id === tabId)
    if (!tab) return
    
    setActiveTabId(tabId)
    setFilePath(tab.filePath)
    setFileContent(tab.content)
    if (tab.analysisResult) {
      setResult(tab.analysisResult)
    }
  }, [tabs])
  
  // 重新排序标签页
  const reorderTabs = useCallback((fromIndex: number, toIndex: number) => {
    setTabs(prev => {
      const newTabs = [...prev]
      const [movedTab] = newTabs.splice(fromIndex, 1)
      newTabs.splice(toIndex, 0, movedTab)
      return newTabs
    })
  }, [])
  
  // 更新当前标签的分析结果
  const updateCurrentTabAnalysis = useCallback((analysis: AnalysisResult) => {
    if (!activeTabId) return
    setTabs(prev => prev.map(tab => 
      tab.id === activeTabId 
        ? { ...tab, analysisResult: analysis }
        : tab
    ))
  }, [activeTabId])

  // Analyze and load file
  const handleAnalyze = async (path?: string, skipHistory = false) => {
    const targetPath = path || filePath
    if (!targetPath) return
    
    setLoading(true)
    setError(null)
    
    try {
      // Load file content
      const content = await invoke<string>('read_file', { path: targetPath })
      setFileContent(content)
      setFilePath(targetPath)
      
      // Analyze file
      const analysis = await invoke<AnalysisResult>('analyze_file', { path: targetPath })
      setResult(analysis)
      updateCurrentTabAnalysis(analysis) // 更新标签页的分析结果
      
      // Get function list for outline
      const functions = await invoke<Array<{
        name: string
        return_type: string
        line: number
        is_callback: boolean
      }>>('get_functions', { path: targetPath })
      
      setOutlineItems(functions.map(f => ({
        name: f.name,
        kind: 'function' as const,
        line: f.line,
        isCallback: f.is_callback,
        returnType: f.return_type,
      })))
      
      // 记录导航历史
      if (!skipHistory) {
        pushNavHistory({ filePath: targetPath, selectedFunction: null })
      }
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  // Open file dialog
  const handleOpenFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        title: '选择源文件',
        filters: [
          { name: 'C/C++ Files', extensions: ['c', 'h', 'cpp', 'hpp', 'cc'] },
          { name: 'All Files', extensions: ['*'] }
        ]
      })
      
      if (selected && typeof selected === 'string') {
        handleAnalyze(selected)
      }
    } catch (e) {
      setError(String(e))
    }
  }

  // 处理拖放的文件
  useEffect(() => {
    if (droppedFile) {
      handleAnalyze(droppedFile)
      setDroppedFile(null)
    }
  }, [droppedFile])

  const handleNodeClick = useCallback(async (_nodeId: string, functionName: string) => {
    setSelectedFunction(functionName)
    setNodeDetailOpen(true)

    // First try to find from current file's function list
    const funcFromOutline = outlineItems.find(item => item.name === functionName)
    let targetLine: number | undefined
    let nodeDetail: NodeDetailData | null = null

    if (funcFromOutline) {
      targetLine = funcFromOutline.line
      // Build detail from outline
      setFunctionDetail({
        name: funcFromOutline.name,
        return_type: funcFromOutline.returnType || 'void',
        file: filePath || null,
        line: funcFromOutline.line,
        end_line: funcFromOutline.line + 10,
        is_callback: funcFromOutline.isCallback || false,
        callback_context: null,
        calls: [],
        called_by: [],
        params: [],
      })

      // Build node detail data for NodeDetailPanel
      nodeDetail = {
        name: funcFromOutline.name,
        nodeType: funcFromOutline.isCallback ? { AsyncCallback: { mechanism: { Custom: 'callback' } } } : 'Function',
        location: funcFromOutline.line ? {
          file: filePath || '',
          line: funcFromOutline.line,
        } : undefined,
        description: funcFromOutline.isCallback ? '回调函数' : undefined,
        confidence: { level: 'Certain', reason: '从函数定义确定' },
        children: [],
        returnType: funcFromOutline.returnType || 'void',
        params: [],
        callers: [],
      }
      setGoToLine({ line: funcFromOutline.line, timestamp: Date.now() })
    } else {
      // Try to find in flow trees for line info
      if (result) {
        const findInTree = (nodes: FlowTreeNode[]): FlowTreeNode | null => {
          for (const node of nodes) {
            if (node.name === functionName) return node
            if (node.children) {
              const found = findInTree(node.children)
              if (found) return found
            }
          }
          return null
        }

        const node = findInTree(result.flow_trees)
        if (node) {
          targetLine = node.location?.line
          const isAsync = typeof node.node_type === 'object' && 'AsyncCallback' in node.node_type

          setFunctionDetail({
            name: node.name,
            return_type: 'unknown',
            file: node.location?.file || null,
            line: node.location?.line || 0,
            end_line: (node.location?.line || 0) + 10,
            is_callback: isAsync,
            callback_context: node.description || null,
            calls: node.children?.map(c => c.name) || [],
            called_by: [],
            params: [],
          })

          // Build node detail data for NodeDetailPanel
          nodeDetail = {
            name: node.name,
            nodeType: node.node_type,
            location: node.location ? {
              file: node.location.file,
              line: node.location.line,
            } : undefined,
            description: node.description,
            confidence: node.confidence,
            children: node.children,
            llvmIr: [
              `define ${node.node_type === 'Function' ? 'void' : 'i32'} @${node.name}() {`,
              `entry:`,
              `  ; Function body from ${node.location?.file || 'unknown'}`,
              `  ret ${node.node_type === 'Function' ? 'void' : 'i32'} undef`,
              `}`,
            ],
            returnType: node.node_type === 'Function' ? 'void' : 'i32',
            params: [],
            callers: [],
          }

          if (node.location?.line) {
            setGoToLine({ line: node.location.line, timestamp: Date.now() })
          }
        } else {
          // External function, show basic info
          setFunctionDetail({
            name: functionName,
            return_type: 'unknown',
            file: null,
            line: 0,
            end_line: 0,
            is_callback: false,
            callback_context: null,
            calls: [],
            called_by: [],
            params: [],
          })

          nodeDetail = {
            name: functionName,
            nodeType: 'External',
            description: '外部函数',
            children: [],
            llvmIr: [
              `declare ${'void'} @${functionName}()`,
              `; External function definition not found`,
            ],
            returnType: 'unknown',
            params: [],
            callers: [],
          }
        }
      }
    }

    setNodeDetailData(nodeDetail)

    // 生成 LLVM IR 数据用于 LlvmIrPanel
    const llvmIr: LlvmIrParseResult = {
      moduleName: filePath ? filePath.split('/').pop() || 'module' : 'module',
      functions: {
        [functionName]: {
          name: functionName,
          returnType: nodeDetail?.returnType || 'void',
          parameters: nodeDetail?.params?.map(p => ({
            name: p.name,
            typeStr: p.type,
          })) || [],
          blocks: [
            {
              name: 'entry',
              instructions: [
                { opcode: 'alloca', typeStr: 'i32*', operands: [], dest: 'ptr' },
                { opcode: 'load', typeStr: 'i32', operands: ['ptr'], dest: 'val' },
                { opcode: 'icmp', typeStr: 'i32', operands: ['val', '0'], dest: 'cmp' },
              ],
              predecessors: [],
              successors: ['then', 'else'],
              terminator: { opcode: 'br', typeStr: 'label', operands: ['%then'] },
            },
            {
              name: 'then',
              instructions: [
                { opcode: 'call', typeStr: 'void', operands: ['@handle_then'], dest: '' },
              ],
              predecessors: ['entry'],
              successors: ['merge'],
              terminator: { opcode: 'br', typeStr: 'label', operands: ['%merge'] },
            },
            {
              name: 'else',
              instructions: [
                { opcode: 'call', typeStr: 'void', operands: ['@handle_else'], dest: '' },
              ],
              predecessors: ['entry'],
              successors: ['merge'],
              terminator: { opcode: 'br', typeStr: 'label', operands: ['%merge'] },
            },
            {
              name: 'merge',
              instructions: [],
              predecessors: ['then', 'else'],
              successors: [],
              terminator: { opcode: 'ret', typeStr: 'void', operands: [] },
            },
          ],
          isCallback: typeof nodeDetail?.nodeType === 'object' && 'AsyncCallback' in nodeDetail.nodeType,
          callbackContext: typeof nodeDetail?.nodeType === 'object' && 'AsyncCallback' in nodeDetail.nodeType
            ? 'Async callback handler'
            : undefined,
        },
      },
    }
    setLlvmIrData(llvmIr)
    setSelectedLlvmFunction(functionName)

    // 记录导航历史
    if (filePath) {
      pushNavHistory({ filePath, selectedFunction: functionName, line: targetLine })
    }
  }, [result, outlineItems, filePath, pushNavHistory])

  const handleSearchResultClick = async (searchResult: SearchResult) => {
    if (searchResult.file) {
      await handleAnalyze(searchResult.file)
      if (searchResult.line) {
        setGoToLine({ line: searchResult.line, timestamp: Date.now() })
      }
      // 记录导航历史
      pushNavHistory({ 
        filePath: searchResult.file, 
        selectedFunction: searchResult.name, 
        line: searchResult.line || undefined 
      })
    }
    setSelectedFunction(searchResult.name)
    setSearchQuery('')
    setSearchResults([])
  }

  // Handle file selection from tree
  const handleFileSelect = async (path: string) => {
    // 在标签页中打开文件
    await openFileInTab(path)
    
    // Only analyze C/H files
    const ext = path.split('.').pop()?.toLowerCase()
    if (['c', 'h', 'cpp', 'hpp', 'cc', 'cxx'].includes(ext || '')) {
      await handleAnalyze(path)
    } else {
      // 非 C 文件不分析，清空分析结果
      setResult(null)
      setOutlineItems([])
    }
    
    // 记录导航历史
    pushNavHistory({ filePath: path, selectedFunction: null })
  }

  const handleEditorLineClick = (line: number) => {
    console.log('Clicked line:', line)
  }
  
  // 实时分析定时器
  const reanalyzeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  
  // 处理编辑器内容变化
  const handleContentChange = useCallback((newContent: string) => {
    setFileContent(newContent)
    
    // 标记当前标签为已修改
    if (activeTabId) {
      setTabs(prev => prev.map(tab => 
        tab.id === activeTabId 
          ? { ...tab, content: newContent, isDirty: true }
          : tab
      ))
    }
    
    // 实时重新分析（防抖 1.5 秒）
    if (appSettings.autoReanalyze && filePath) {
      if (reanalyzeTimerRef.current) {
        clearTimeout(reanalyzeTimerRef.current)
      }
      
      reanalyzeTimerRef.current = setTimeout(async () => {
        try {
          // 先保存到临时文件然后分析
          const analysisResult = await invoke<AnalysisResult>('analyze_file', { path: filePath })

          if (analysisResult.flow_trees && analysisResult.flow_trees.length > 0) {
            setResult(analysisResult)

            // 更新大纲
            const funcs = await invoke<Array<{
              name: string
              return_type: string
              line: number
              is_callback: boolean
              callback_context?: string
            }>>('get_functions', { path: filePath })
            const items: OutlineItem[] = funcs.map(f => ({
              name: f.name,
              kind: 'function' as const,
              line: f.line,
              isCallback: f.is_callback,
              returnType: f.return_type,
            }))
            items.sort((a, b) => a.line - b.line)
            setOutlineItems(items)
          }
        } catch (e) {
          // 静默失败，不打扰用户
          console.debug('Auto-reanalyze failed:', e)
        }
      }, 1500)
    }
  }, [activeTabId, appSettings.autoSave, filePath])
  
  // 保存当前文件
  const saveCurrentFile = useCallback(async () => {
    if (!filePath || !activeTabId) return
    
    const currentTab = tabs.find(t => t.id === activeTabId)
    if (!currentTab || !currentTab.isDirty) return
    
    try {
      await invoke('write_file', { path: filePath, contents: fileContent })
      
      // 标记为已保存
      setTabs(prev => prev.map(tab => 
        tab.id === activeTabId 
          ? { ...tab, isDirty: false }
          : tab
      ))
      
      success(`文件已保存: ${filePath.split('/').pop()}`)
    } catch (err) {
      console.error('保存失败:', err)
      showError(`保存失败: ${err}`)
    }
  }, [filePath, fileContent, activeTabId, tabs, success, showError])
  
  // Ctrl+S 保存快捷键
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault()
        saveCurrentFile()
      }
    }
    
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [saveCurrentFile])
  
  // 自动保存功能
  const autoSaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  
  useEffect(() => {
    // 清除之前的定时器
    if (autoSaveTimerRef.current) {
      clearTimeout(autoSaveTimerRef.current)
      autoSaveTimerRef.current = null
    }
    
    // 如果启用了自动保存，且当前标签有未保存更改
    if (appSettings.autoSave && activeTabId) {
      const currentTab = tabs.find(t => t.id === activeTabId)
      if (currentTab?.isDirty) {
        autoSaveTimerRef.current = setTimeout(() => {
          saveCurrentFile()
        }, appSettings.autoSaveDelay)
      }
    }
    
    return () => {
      if (autoSaveTimerRef.current) {
        clearTimeout(autoSaveTimerRef.current)
      }
    }
  }, [appSettings.autoSave, appSettings.autoSaveDelay, activeTabId, tabs, saveCurrentFile])
  
  // 代码-图联动：光标所在函数名变化时高亮图中节点
  const handleWordAtCursor = useCallback((word: string | null) => {
    // 只更新选中状态，不记录导航历史
    if (word) {
      setSelectedFunction(word)
    }
  }, [])
  
  // 已知函数名列表（用于代码-图联动判断）
  const knownFunctions = useMemo(() => {
    const names = new Set<string>()
    // 从大纲获取
    outlineItems.forEach(item => names.add(item.name))
    // 从执行流树获取
    if (result) {
      const addFromTree = (nodes: FlowTreeNode[]) => {
        nodes.forEach(node => {
          names.add(node.name)
          if (node.children) {
            addFromTree(node.children)
          }
        })
      }
      addFromTree(result.flow_trees)
    }
    return Array.from(names)
  }, [outlineItems, result])
  
  // Handle outline item click
  const handleOutlineClick = (item: OutlineItem) => {
    setSelectedFunction(item.name)
    setGoToLine({ line: item.line, timestamp: Date.now() })
    // 记录导航历史
    if (filePath) {
      pushNavHistory({ filePath, selectedFunction: item.name, line: item.line })
    }
  }

  // 命令面板选择处理
  const handleCommandSelect = useCallback(async (item: { type: string; path?: string; line?: number; name: string }) => {
    if (item.path) {
      await handleAnalyze(item.path)
      if (item.line) {
        setGoToLine({ line: item.line, timestamp: Date.now() })
      }
      if (item.type === 'symbol') {
        setSelectedFunction(item.name)
      }
      // 记录导航历史
      pushNavHistory({ 
        filePath: item.path, 
        selectedFunction: item.type === 'symbol' ? item.name : null,
        line: item.line 
      })
    }
  }, [handleAnalyze, pushNavHistory])

  // 为命令面板准备文件列表（递归获取所有文件）
  const allFiles = useMemo(() => {
    const files: Array<{ name: string; path: string; isDir: boolean }> = []
    
    const collectFiles = (nodes: FileNode[]) => {
      for (const node of nodes) {
        if (!node.is_dir) {
          files.push({ name: node.name, path: node.path, isDir: false })
        }
        if (node.children) {
          collectFiles(node.children)
        }
      }
    }
    
    collectFiles(fileTree)
    return files
  }, [fileTree])

  // 为命令面板准备符号列表
  const allSymbols = useMemo(() => {
    return outlineItems.map(item => ({
      name: item.name,
      kind: item.kind,
      file: filePath || undefined,
      line: item.line,
      isCallback: item.isCallback,
    }))
  }, [outlineItems, filePath])

  // Get highlight lines from async handlers
  const highlightLines: number[] = []
  if (result) {
    // Could add logic to highlight callback function lines
  }

  const flowTrees: FlowTreeNode[] = result?.flow_trees || []

  return (
    <div 
      className={`app ${isDragging ? 'dragging' : ''}`}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* 拖放提示层 */}
      {isDragging && (
        <div className="drop-overlay">
          <div className="drop-hint">
            <span className="drop-icon">📂</span>
            <span>释放以打开文件</span>
          </div>
        </div>
      )}
      
      <header className="header">
        <div className="header-content">
          <div className="header-title">
            <h1>🔭 FlowSight</h1>
          </div>
          <div className="header-actions">
            {/* 导航按钮 */}
            <div className="nav-buttons">
              <button 
                onClick={goBack} 
                disabled={!canGoBack}
                className="button nav-btn"
                title="后退 (Alt+←)"
              >
                ◀
              </button>
              <button 
                onClick={goForward} 
                disabled={!canGoForward}
                className="button nav-btn"
                title="前进 (Alt+→)"
              >
                ▶
              </button>
            </div>
            <button onClick={handleOpenProject} className="button secondary">
              📂 项目
            </button>
            <button onClick={handleOpenFile} className="button secondary">
              📄 文件
            </button>
            <div className="search-container">
              <input
                type="text"
                className="search-input"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="🔍 搜索函数或结构体..."
              />
              {searchResults.length > 0 && (
                <div className="search-dropdown">
                  {searchResults.map((r, i) => (
                    <div 
                      key={i} 
                      className="search-item"
                      onClick={() => handleSearchResultClick(r)}
                    >
                      <span className="search-icon">
                        {r.kind === 'function' ? (r.is_callback ? '⚡' : '📦') : '🏗️'}
                      </span>
                      <span className="search-name">{r.name}</span>
                      <span className="search-kind">{r.kind}</span>
                      {r.file && (
                        <span className="search-file">{r.file.split('/').pop()}</span>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
            <div className="view-toggle">
              <button 
                className={`toggle-btn ${viewMode === 'code' ? 'active' : ''}`}
                onClick={() => setViewMode('code')}
                title="代码视图"
              >
                📝
              </button>
              <button 
                className={`toggle-btn ${viewMode === 'split' ? 'active' : ''}`}
                onClick={() => setViewMode('split')}
                title="分屏视图"
              >
                ⚡
              </button>
              <button 
                className={`toggle-btn ${viewMode === 'flow' ? 'active' : ''}`}
                onClick={() => setViewMode('flow')}
                title="执行流视图"
              >
                📊
              </button>
            </div>
            <button onClick={() => handleAnalyze()} disabled={loading || !filePath} className="button primary">
              {loading ? '⏳' : '🔄'}
            </button>
            <button onClick={() => setShortcutsOpen(true)} className="button icon" title="快捷键帮助 (?)">
              ⌨️
            </button>
            <button onClick={() => setSettingsOpen(true)} className="button icon" title="设置">
              ⚙️
            </button>
            <button onClick={() => setAboutOpen(true)} className="button icon" title="关于 FlowSight">
              ℹ️
            </button>
          </div>
        </div>
      </header>

      <main className="main">
        {/* 左侧栏折叠按钮 */}
        <button 
          className={`panel-toggle left ${leftPanelOpen ? '' : 'collapsed'}`}
          onClick={() => setLeftPanelOpen(!leftPanelOpen)}
          title={leftPanelOpen ? '收起左侧栏' : '展开左侧栏'}
        >
          {leftPanelOpen ? '◀' : '▶'}
        </button>

        {/* 左侧面板 - 文件浏览器 */}
        {leftPanelOpen && (
          <>
            <div 
              className="panel sidebar explorer-sidebar"
              style={{ width: leftPanelWidth }}
            >
              {project ? (
                <>
                  <div className="project-header">
                    <h2>📁 {project.path.split('/').pop()}</h2>
                    <div className="project-stats">
                      <span>{indexStats?.files || 0} 文件</span>
                      <span>•</span>
                      <span>{indexStats?.functions || 0} 函数</span>
                    </div>
                  </div>

                  {/* 索引进度条 */}
                  {indexProgress && (
                    <div className="index-progress">
                      <div className="progress-message">{indexProgress.message}</div>
                      {indexProgress.total > 0 && (
                        <div className="progress-bar">
                          <div
                            className="progress-fill"
                            style={{ width: `${(indexProgress.current / indexProgress.total) * 100}%` }}
                          />
                        </div>
                      )}
                    </div>
                  )}

                  <div className="file-tree-container">
                    <FileTree 
                      nodes={fileTree}
                      onFileSelect={handleFileSelect}
                      selectedPath={filePath}
                    />
                  </div>
                </>
              ) : (
                <div className="welcome-project">
                  <h2>👋 开始使用</h2>
                  <p>点击"项目"打开代码目录</p>
                  <p>或点击"文件"打开单个文件</p>
                </div>
              )}
              
              {error && (
                <div className="error">
                  <strong>❌ 错误：</strong> {error}
                </div>
              )}
            </div>
            
            {/* 左侧拖动条 */}
            <div 
              className="resize-handle"
              onMouseDown={startResizeLeft}
            />
          </>
        )}

        {/* 中间区域 - 代码/执行流可视化 */}
        <div className="panel main-content">
          <div className="panel-header">
            <h2>
              {viewMode === 'code' ? '📝 代码' : 
               viewMode === 'flow' ? '📊 执行流' : 
               '⚡ 代码 + 执行流'}
            </h2>
            {selectedFunction && (
              <span className="selected-info">
                已选择: <code>{selectedFunction}()</code>
              </span>
            )}
          </div>
          
          <div className={`content-area ${viewMode}`}>
            {(viewMode === 'code' || viewMode === 'split') && (
              <div className="editor-panel">
                {/* 标签栏 */}
                <TabBar
                  tabs={tabs}
                  activeTabId={activeTabId}
                  onTabSelect={switchTab}
                  onTabClose={closeTab}
                  onTabReorder={reorderTabs}
                />
                
                {/* 面包屑导航 */}
                <Breadcrumb
                  projectRoot={project?.path}
                  filePath={filePath}
                  currentFunction={selectedFunction}
                  onPathClick={(path) => {
                    // 打开目录或文件
                    handleFileSelect(path)
                  }}
                  onFunctionClick={() => {
                    // 跳转到当前函数定义
                    if (selectedFunction) {
                      const func = outlineItems.find(item => item.name === selectedFunction)
                      if (func) {
                        setGoToLine({ line: func.line, timestamp: Date.now() })
                      }
                    }
                  }}
                />
                
                {/* 查找替换面板 */}
                <FindReplace
                  isOpen={findReplaceOpen}
                  onClose={() => setFindReplaceOpen(false)}
                  content={fileContent}
                  onFindResult={setFindMatches}
                  onReplaceAll={(newContent) => {
                    setFileContent(newContent)
                    handleContentChange(newContent)
                  }}
                  onGoToMatch={(match) => {
                    setGoToLine({ line: match.line, timestamp: Date.now() })
                  }}
                />
                
                {fileContent ? (
                  <CodeEditor
                    content={fileContent}
                    filePath={filePath}
                    goToLine={goToLine}
                    highlightLines={highlightLines}
                    onLineClick={handleEditorLineClick}
                    onWordAtCursor={handleWordAtCursor}
                    knownFunctions={knownFunctions}
                    onChange={handleContentChange}
                    readOnly={false}
                    theme={appSettings.theme}
                    fontSize={appSettings.fontSize}
                  />
                ) : (
                  <Welcome 
                    onOpenFile={handleOpenFile}
                    onOpenProject={handleOpenProject}
                    onOpenRecentFile={(path) => handleAnalyze(path)}
                    onOpenRecentProject={async (path) => {
                      try {
                        setLoading(true)
                        const info = await invoke<ProjectInfo>('open_project', { path })
                        setProject(info)
                        const stats = await invoke<IndexStats>('get_index_stats')
                        setIndexStats(stats)
                        const tree = await invoke<FileNode[]>('list_directory', { path, recursive: false })
                        setFileTree(tree)
                        setLeftPanelOpen(true)
                      } catch (e) {
                        setError(String(e))
                      } finally {
                        setLoading(false)
                      }
                    }}
                  />
                )}
              </div>
            )}
            {(viewMode === 'flow' || viewMode === 'split') && (
              <div className="flow-panel">
                {/* 执行流视图模式切换 */}
                <div className="flow-mode-toggle">
                  <button 
                    className={flowDisplayMode === 'graph' ? 'active' : ''}
                    onClick={() => setFlowDisplayMode('graph')}
                    title="图形视图"
                  >
                    📊 图形
                  </button>
                  <button 
                    className={flowDisplayMode === 'text' ? 'active' : ''}
                    onClick={() => setFlowDisplayMode('text')}
                    title="文本视图 (ftrace风格)"
                  >
                    📝 文本
                  </button>
                </div>
                
                {flowDisplayMode === 'graph' ? (
                  <FlowView 
                    flowTrees={flowTrees} 
                    onNodeClick={handleNodeClick}
                    selectedFunction={selectedFunction || undefined}
                  />
                ) : (
                  <FlowTextView 
                    flowTrees={flowTrees}
                    onNodeClick={(name) => handleNodeClick('', name)}
                    selectedFunction={selectedFunction || undefined}
                  />
                )}
              </div>
            )}
          </div>
        </div>

        {/* 右侧拖动条 */}
        {rightPanelOpen && (
          <div 
            className="resize-handle"
            onMouseDown={startResizeRight}
          />
        )}

        {/* 右侧面板 - 分析详情 */}
        {rightPanelOpen && (
          <div 
            className="panel sidebar right-sidebar"
            style={{ width: rightPanelWidth }}
          >
            {/* 分析概览 */}
            {result && (
              <div className="analysis-overview">
                <h2>📋 分析概览</h2>
                <div className="overview-stats">
                  <div className="stat-item">
                    <span className="stat-value">{result.functions_count}</span>
                    <span className="stat-label">函数</span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-value">{result.structs_count}</span>
                    <span className="stat-label">结构体</span>
                  </div>
                  <div className="stat-item highlight">
                    <span className="stat-value">{result.async_handlers_count}</span>
                    <span className="stat-label">异步</span>
                  </div>
                </div>
                
                {result.entry_points.length > 0 && (
                  <div className="entry-points">
                    <h3>🚀 入口点</h3>
                    <ul>
                      {result.entry_points.map((entry, i) => (
                        <li 
                          key={i} 
                          className={selectedFunction === entry ? 'selected' : ''}
                          onClick={() => handleNodeClick('', entry)}
                        >
                          <code>{entry}()</code>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
                <hr className="divider" />
              </div>
            )}
            
            {/* 代码大纲 */}
            {outlineItems.length > 0 && (
              <div className="outline-section-wrapper">
                <h2>📋 大纲</h2>
                <div className="outline-container">
                  <Outline 
                    items={outlineItems}
                    onItemClick={handleOutlineClick}
                    selectedItem={selectedFunction || undefined}
                  />
                </div>
                <hr className="divider" />
              </div>
            )}
            
            <h2>📝 节点详情</h2>

            {/* 节点详情面板 */}
            {nodeDetailData && (
              <div className="node-detail-wrapper">
                <NodeDetailPanel
                  node={nodeDetailData}
                  title={`${nodeDetailData.name}()`}
                  theme={appSettings.theme}
                  onNodeClick={(nodeName) => handleNodeClick('', nodeName)}
                />
              </div>
            )}

            {(!nodeDetailData || !selectedFunction) && (
              <div className="detail-placeholder">
                <p>点击节点查看详情</p>
              </div>
            )}

            {/* LLVM IR 可视化面板 */}
            {llvmIrData && selectedLlvmFunction && (
              <div className="llvm-ir-wrapper">
                <LlvmIrPanel
                  parseResult={llvmIrData}
                  selectedFunction={selectedLlvmFunction}
                  theme={appSettings.theme}
                  onFunctionSelect={(funcName) => setSelectedLlvmFunction(funcName)}
                />
              </div>
            )}

            <div className="legend">
              <h3>图例</h3>
              <ul>
                <li><span className="legend-icon entry">🚀</span> 入口点</li>
                <li><span className="legend-icon async">⚡</span> 异步回调</li>
                <li><span className="legend-icon kernel">⚙️</span> 内核 API</li>
                <li><span className="legend-icon func">📦</span> 普通函数</li>
              </ul>
            </div>
          </div>
        )}

        {/* 右侧栏折叠按钮 */}
        <button 
          className={`panel-toggle right ${rightPanelOpen ? '' : 'collapsed'}`}
          onClick={() => setRightPanelOpen(!rightPanelOpen)}
          title={rightPanelOpen ? '收起右侧栏' : '展开右侧栏'}
        >
          {rightPanelOpen ? '▶' : '◀'}
        </button>
      </main>
      
      {/* 命令面板 */}
      <CommandPalette
        isOpen={commandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
        onSelect={handleCommandSelect}
        files={allFiles}
        symbols={allSymbols}
      />
      
      {/* 设置面板 */}
      <Settings
        isOpen={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        settings={appSettings}
        onSettingsChange={setAppSettings}
      />
      
      {/* 快捷键帮助 */}
      <KeyboardShortcuts
        isOpen={shortcutsOpen}
        onClose={() => setShortcutsOpen(false)}
      />
      
      {/* 跳转行号 */}
      <GoToLine
        isOpen={goToLineOpen}
        onClose={() => setGoToLineOpen(false)}
        onGoTo={(line) => setGoToLine({ line, timestamp: Date.now() })}
        totalLines={fileContent?.split('\n').length || 1}
      />
      
      {/* 状态栏 */}
      <StatusBar
        filePath={filePath}
        functionCount={outlineItems.length}
        analysisStatus={loading ? 'analyzing' : result ? 'done' : 'idle'}
        isDirty={tabs.find(t => t.id === activeTabId)?.isDirty}
        fileContent={fileContent}
      />
      
      {/* Toast 通知 */}
      <ToastContainer toasts={toasts} onRemove={removeToast} />
      
      {/* 关于对话框 */}
      <AboutDialog isOpen={aboutOpen} onClose={() => setAboutOpen(false)} />
      
      {/* 关闭未保存文件确认 */}
      <ConfirmDialog
        isOpen={!!closeConfirm}
        title="未保存的更改"
        message={`文件 "${closeConfirm?.fileName}" 有未保存的更改。确定要关闭吗？`}
        confirmText="关闭"
        cancelText="取消"
        variant="danger"
        onConfirm={() => {
          if (closeConfirm) {
            closeTab(closeConfirm.tabId, true)
          }
          setCloseConfirm(null)
        }}
        onCancel={() => setCloseConfirm(null)}
      />
      
      {/* 快速打开 */}
      <QuickOpen
        isOpen={quickOpenOpen}
        onClose={() => setQuickOpenOpen(false)}
        recentFiles={getRecentFiles().map(rf => ({
          path: rf.path,
          name: rf.name,
          timestamp: rf.timestamp
        }))}
        onSelect={handleAnalyze}
      />
      
      {/* 场景化分析结果显示 */}
      {scenarioResults && (
        <ScenarioResults
          isOpen={scenarioResultsOpen}
          onClose={() => setScenarioResultsOpen(false)}
          scenarioName={currentScenarioName}
          path={scenarioResults.path}
          states={scenarioResults.states}
          onNodeClick={(funcName) => {
            // 跳转到对应函数
            handleNodeClick('', funcName)
          }}
        />
      )}
      
      {/* 调用者分析视图 */}
      <CallersView
        isOpen={callersViewOpen}
        onClose={() => setCallersViewOpen(false)}
        functionName={_callersTargetFunc}
        projectPath={project?.path}
        onFunctionClick={(funcName, file, line) => {
          setCallersViewOpen(false)
          if (file && line) {
            handleAnalyze(file)
            setGoToLine({ line, timestamp: Date.now() })
          }
          handleNodeClick('', funcName)
        }}
      />
      
      {/* 场景化数据流分析面板 */}
      <ScenarioPanel
        isOpen={scenarioPanelOpen}
        onClose={() => setScenarioPanelOpen(false)}
        entryFunction={functionDetail?.name || ''}
        params={functionDetail?.params || []}
        onExecute={async (scenario) => {
          try {
            setScenarioPanelOpen(false)
            info(`正在执行场景 "${scenario.name}"...`)

            const scenarioResult = await invoke<{
              success: boolean
              path: { function: string; line: number; variables: Record<string, string> }[]
              annotated_flow_tree: FlowTreeNode | null
              error: string | null
            }>('execute_scenario', {
              filePath: filePath,
              scenario: {
                name: scenario.name,
                entry_function: scenario.entryFunction,
                bindings: scenario.bindings.map(b => ({
                  path: b.path,
                  value: b.value,
                  type: b.type,
                })),
                options: null,
              },
            })
            
            if (scenarioResult.success) {
              // 更新执行流树显示带注解的版本
              if (scenarioResult.annotated_flow_tree) {
                setResult(prev => prev ? {
                  ...prev,
                  flow_trees: [scenarioResult.annotated_flow_tree!]
                } : null)
              }
              setScenarioResults({
                path: scenarioResult.path.map(p => p.function),
                states: scenarioResult.path.map(p => ({
                  location: `${p.function}:${p.line}`,
                  variables: p.variables,
                })),
              })
              setCurrentScenarioName(scenario.name)
              setScenarioResultsOpen(true)
              success(`场景 "${scenario.name}" 分析完成！执行路径包含 ${scenarioResult.path.length} 个节点`)
            } else {
              showError(`场景分析失败: ${scenarioResult.error}`)
            }
          } catch (e) {
            showError(`场景分析出错: ${e}`)
          }
        }}
      />
    </div>
  )
}

export default App
