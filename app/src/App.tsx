/**
 * FlowSight - 跨平台执行流可视化 IDE
 */

import { useState, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { FlowView } from './components/FlowView'
import { CodeEditor } from './components/Editor'
import { FileTree, FileNode } from './components/Explorer'
import { 
  AnalysisResult, 
  FlowTreeNode, 
  ProjectInfo, 
  SearchResult, 
  IndexStats,
  FunctionDetail 
} from './types'

type ViewMode = 'flow' | 'code' | 'split'

function App() {
  const [result, setResult] = useState<AnalysisResult | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedFunction, setSelectedFunction] = useState<string | null>(null)
  const [filePath, setFilePath] = useState('')
  const [fileContent, setFileContent] = useState('')
  const [goToLine, setGoToLine] = useState<number | undefined>()
  const [viewMode, setViewMode] = useState<ViewMode>('split')
  
  // Project state
  const [project, setProject] = useState<ProjectInfo | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<SearchResult[]>([])
  const [indexStats, setIndexStats] = useState<IndexStats | null>(null)
  const [functionDetail, setFunctionDetail] = useState<FunctionDetail | null>(null)
  const [fileTree, setFileTree] = useState<FileNode[]>([])

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
        
        // Load file tree
        const tree = await invoke<FileNode[]>('list_directory', { path: selected, recursive: true })
        setFileTree(tree)
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
  const handleAnalyze = async (path?: string) => {
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
    
    // Get function detail
    try {
      const detail = await invoke<FunctionDetail | null>('get_function_detail', { name: functionName })
      setFunctionDetail(detail)
      
      if (detail && detail.line > 0) {
        setGoToLine(detail.line)
      }
    } catch (e) {
      console.error('Failed to get function detail:', e)
    }
    
    // Find function line and jump to it
    if (result) {
      // Search in flow trees for the function location
      const findLine = (nodes: FlowTreeNode[]): number | null => {
        for (const node of nodes) {
          if (node.name === functionName && node.location) {
            return node.location.line
          }
          if (node.children) {
            const found = findLine(node.children)
            if (found) return found
          }
        }
        return null
      }
      
      const line = findLine(result.flow_trees)
      if (line) {
        setGoToLine(line)
      }
    }
  }, [result])

  const handleSearchResultClick = (searchResult: SearchResult) => {
    if (searchResult.file) {
      handleAnalyze(searchResult.file)
      if (searchResult.line) {
        setGoToLine(searchResult.line)
      }
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
      } catch (e) {
        setError(String(e))
      }
    }
  }

  const handleEditorLineClick = (line: number) => {
    console.log('Clicked line:', line)
  }

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
            <p>看见代码的"灵魂" — 执行流可视化 IDE</p>
          </div>
          <div className="header-actions">
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
        {/* 左侧面板 - 文件浏览器 */}
        <div className="panel sidebar explorer-sidebar">
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

        {/* 中间区域 - 代码/执行流可视化 */}
        <div className="panel main-content">
          <div className="panel-header">
            <h2>
              {viewMode === 'code' ? '📝 代码编辑器' : 
               viewMode === 'flow' ? '📊 执行流视图' : 
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
                <FlowView flowTrees={flowTrees} onNodeClick={handleNodeClick} />
              </div>
            )}
          </div>
        </div>

        {/* 右侧面板 - 分析详情 */}
        <div className="panel sidebar">
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
              <p className="detail-hint">加载中...</p>
            </div>
          ) : (
            <div className="detail-placeholder">
              <p>点击执行流图中的节点查看详情</p>
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
      </main>

      <footer className="footer">
        <p>FlowSight v0.1.0 - 用 ❤️ 为想要真正理解代码的开发者打造</p>
      </footer>
    </div>
  )
}

export default App
