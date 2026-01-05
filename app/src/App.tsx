/**
 * FlowSight - 跨平台执行流可视化 IDE
 */

import { useState, useCallback, useEffect, useRef, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { FlowView, FlowTextView } from './components/FlowView'
import { CodeEditor } from './components/Editor'
import { FileTree, FileNode } from './components/Explorer'
import { Outline, OutlineItem } from './components/Outline'
import { CommandPalette } from './components/CommandPalette'
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

  const handleNodeClick = useCallback(async (_nodeId: string, functionName: string) => {
    setSelectedFunction(functionName)
    
    // First try to find from current file's function list
    const funcFromOutline = outlineItems.find(item => item.name === functionName)
    let targetLine: number | undefined
    
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
          setFunctionDetail({
            name: node.name,
            return_type: 'unknown',
            file: node.location?.file || null,
            line: node.location?.line || 0,
            end_line: (node.location?.line || 0) + 10,
            is_callback: typeof node.node_type === 'object' && 'AsyncCallback' in node.node_type,
            callback_context: node.description || null,
            calls: node.children?.map(c => c.name) || [],
            called_by: [],
            params: [],
          })
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
        }
      }
    }
    
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
    // Only analyze C/H files
    const ext = path.split('.').pop()?.toLowerCase()
    if (['c', 'h', 'cpp', 'hpp', 'cc', 'cxx'].includes(ext || '')) {
      await handleAnalyze(path)
    } else {
      // Just show the file content
      try {
        const content = await invoke<string>('read_file', { path })
        setFileContent(content)
        setFilePath(path)
        setResult(null)
        // 记录导航历史
        pushNavHistory({ filePath: path, selectedFunction: null })
      } catch (e) {
        setError(String(e))
      }
    }
  }

  const handleEditorLineClick = (line: number) => {
    console.log('Clicked line:', line)
  }
  
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
    <div className="app">
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
                {fileContent ? (
                  <CodeEditor
                    content={fileContent}
                    filePath={filePath}
                    goToLine={goToLine}
                    highlightLines={highlightLines}
                    onLineClick={handleEditorLineClick}
                    onWordAtCursor={handleWordAtCursor}
                    knownFunctions={knownFunctions}
                    readOnly={true}
                  />
                ) : (
                  <div className="empty-editor">
                    <p>📄 打开文件查看代码</p>
                  </div>
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
            
            <h2>📝 函数详情</h2>
            
            {functionDetail ? (
              <div className="function-detail">
                <div className="detail-header">
                  <h3>
                    {functionDetail.is_callback && <span className="callback-badge">⚡</span>}
                    {functionDetail.name}()
                  </h3>
                  <span className="return-type">{functionDetail.return_type}</span>
                </div>
                
                {functionDetail.callback_context && (
                  <div className="detail-badge">
                    🔌 {functionDetail.callback_context}
                  </div>
                )}
                
                {functionDetail.params.length > 0 && (
                  <div className="detail-section">
                    <h4>参数</h4>
                    <ul className="param-list">
                      {functionDetail.params.map((p, i) => (
                        <li key={i}>
                          <span className="param-type">{p.type_name}</span>
                          <span className="param-name">{p.name}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
                
                {functionDetail.calls.length > 0 && (
                  <div className="detail-section">
                    <h4>调用 ({functionDetail.calls.length})</h4>
                    <ul className="call-list">
                      {functionDetail.calls.slice(0, 10).map((c, i) => (
                        <li key={i} onClick={() => handleNodeClick('', c)}>
                          <code>{c}()</code>
                        </li>
                      ))}
                      {functionDetail.calls.length > 10 && (
                        <li className="more">...还有 {functionDetail.calls.length - 10} 个</li>
                      )}
                    </ul>
                  </div>
                )}
                
                {functionDetail.file && (
                  <div className="detail-location">
                    📍 {functionDetail.file.split('/').pop()}:{functionDetail.line}
                  </div>
                )}
              </div>
            ) : selectedFunction ? (
              <div className="function-detail">
                <h3>{selectedFunction}()</h3>
                <p className="detail-hint">外部函数</p>
              </div>
            ) : (
              <div className="detail-placeholder">
                <p>点击节点查看详情</p>
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
    </div>
  )
}

export default App
