/**
 * FlowSight - 跨平台执行流可视化 IDE
 */

import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { FlowView } from './components/FlowView'
import { AnalysisResult, FlowTreeNode } from './types'

function App() {
  const [result, setResult] = useState<AnalysisResult | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedFunction, setSelectedFunction] = useState<string | null>(null)
  const [filePath, setFilePath] = useState('/tmp/test.c')

  const handleAnalyze = async () => {
    setLoading(true)
    setError(null)
    
    try {
      const analysis = await invoke<AnalysisResult>('analyze_file', { path: filePath })
      setResult(analysis)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  const handleNodeClick = useCallback((nodeId: string, functionName: string) => {
    setSelectedFunction(functionName)
    console.log('Selected function:', functionName)
  }, [])

  // 获取要显示的执行流树
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
            <input
              type="text"
              className="file-input"
              value={filePath}
              onChange={(e) => setFilePath(e.target.value)}
              placeholder="输入文件路径..."
            />
            <button onClick={handleAnalyze} disabled={loading} className="button primary">
              {loading ? '⏳ 分析中...' : '🔍 分析代码'}
            </button>
          </div>
        </div>
      </header>

      <main className="main">
        {/* 左侧面板 - 分析信息 */}
        <div className="panel sidebar">
          <h2>📋 分析概览</h2>
          
          {error && (
            <div className="error">
              <strong>❌ 错误：</strong> {error}
            </div>
          )}

          {result ? (
            <div className="analysis-info">
              <div className="info-card">
                <span className="info-label">文件</span>
                <span className="info-value">{result.file.split('/').pop()}</span>
              </div>
              <div className="info-card">
                <span className="info-label">函数</span>
                <span className="info-value">{result.functions_count}</span>
              </div>
              <div className="info-card">
                <span className="info-label">结构体</span>
                <span className="info-value">{result.structs_count}</span>
              </div>
              <div className="info-card">
                <span className="info-label">异步处理器</span>
                <span className="info-value highlight">{result.async_handlers_count}</span>
              </div>
              
              <div className="entry-points">
                <h3>🚀 入口点</h3>
                <ul>
                  {result.entry_points.map((entry, i) => (
                    <li key={i} className={selectedFunction === entry ? 'selected' : ''}>
                      <code>{entry}()</code>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          ) : (
            <div className="welcome">
              <p>FlowSight 帮助你理解代码执行流程：</p>
              <ul>
                <li>📊 函数调用图谱</li>
                <li>⚡ 异步机制追踪</li>
                <li>🔌 回调函数解析</li>
                <li>📦 数据结构关系</li>
              </ul>
              <p className="hint">输入源码文件路径开始分析</p>
            </div>
          )}
        </div>

        {/* 中间区域 - 执行流可视化 */}
        <div className="panel main-content">
          <div className="panel-header">
            <h2>📊 执行流视图</h2>
            {selectedFunction && (
              <span className="selected-info">
                已选择: <code>{selectedFunction}()</code>
              </span>
            )}
          </div>
          <div className="flow-container">
            <FlowView flowTrees={flowTrees} onNodeClick={handleNodeClick} />
          </div>
        </div>

        {/* 右侧面板 - 详情 */}
        <div className="panel sidebar">
          <h2>📝 详情</h2>
          
          {selectedFunction ? (
            <div className="function-detail">
              <h3>{selectedFunction}()</h3>
              <p className="detail-hint">
                点击节点查看函数详情
              </p>
              {/* 后续添加更多详情信息 */}
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
