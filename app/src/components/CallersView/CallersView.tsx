/**
 * CallersView - 显示谁调用了这个函数
 * 
 * 核心功能：
 * - 反向调用图: 显示所有调用者
 * - 调用上下文: 直接调用 / 异步调用
 * - 递归查找: 向上追溯调用链
 */

import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './CallersView.css'

interface CallerInfo {
  name: string
  file: string
  line: number
  call_type: 'direct' | 'async' | 'indirect'
  async_mechanism?: string
}

interface CallersViewProps {
  isOpen: boolean
  onClose: () => void
  functionName: string
  projectPath?: string
  onFunctionClick?: (funcName: string, file?: string, line?: number) => void
}

export function CallersView({
  isOpen,
  onClose,
  functionName,
  projectPath,
  onFunctionClick,
}: CallersViewProps) {
  const [callers, setCallers] = useState<CallerInfo[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [expandedCallers, setExpandedCallers] = useState<Set<string>>(new Set())
  const [secondLevelCallers, setSecondLevelCallers] = useState<Record<string, CallerInfo[]>>({})
  
  // 获取调用者
  const fetchCallers = useCallback(async (funcName: string) => {
    if (!funcName) return
    
    setLoading(true)
    setError(null)
    
    try {
      // 尝试从索引中获取调用者
      const result = await invoke<{
        callers: CallerInfo[]
      }>('get_function_callers', {
        functionName: funcName,
        projectPath: projectPath,
      }).catch(() => null)
      
      if (result?.callers) {
        setCallers(result.callers)
      } else {
        // 如果没有索引数据，显示提示
        setCallers([])
        setError('请先打开项目以获取完整的调用信息')
      }
    } catch (e) {
      setError(`获取调用者失败: ${e}`)
    } finally {
      setLoading(false)
    }
  }, [projectPath])
  
  useEffect(() => {
    if (isOpen && functionName) {
      fetchCallers(functionName)
    }
  }, [isOpen, functionName, fetchCallers])
  
  // 展开/收起二级调用者
  const toggleExpand = useCallback(async (callerName: string) => {
    if (expandedCallers.has(callerName)) {
      setExpandedCallers(prev => {
        const next = new Set(prev)
        next.delete(callerName)
        return next
      })
    } else {
      setExpandedCallers(prev => new Set(prev).add(callerName))
      
      // 获取二级调用者
      if (!secondLevelCallers[callerName]) {
        try {
          const result = await invoke<{
            callers: CallerInfo[]
          }>('get_function_callers', {
            functionName: callerName,
            projectPath: projectPath,
          }).catch(() => null)
          
          if (result?.callers) {
            setSecondLevelCallers(prev => ({
              ...prev,
              [callerName]: result.callers,
            }))
          }
        } catch (e) {
          console.error('Failed to get second level callers:', e)
        }
      }
    }
  }, [expandedCallers, secondLevelCallers, projectPath])
  
  if (!isOpen) return null
  
  const getCallTypeIcon = (type: string, mechanism?: string) => {
    switch (type) {
      case 'async':
        if (mechanism?.includes('WorkQueue')) return '🔄'
        if (mechanism?.includes('Timer')) return '⏱️'
        if (mechanism?.includes('Interrupt')) return '⚡'
        return '⏳'
      case 'indirect':
        return '↩️'
      default:
        return '📞'
    }
  }
  
  const getCallTypeLabel = (type: string, mechanism?: string) => {
    switch (type) {
      case 'async':
        return mechanism || '异步调用'
      case 'indirect':
        return '间接调用 (函数指针)'
      default:
        return '直接调用'
    }
  }
  
  return (
    <div className="callers-view-overlay" onClick={onClose}>
      <div className="callers-view" onClick={e => e.stopPropagation()}>
        <div className="callers-header">
          <h2>📥 调用者分析</h2>
          <div className="target-function">
            <span className="label">谁调用了</span>
            <code>{functionName}()</code>
          </div>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        
        <div className="callers-content">
          {loading ? (
            <div className="loading">
              <span className="spinner">⏳</span>
              正在分析调用关系...
            </div>
          ) : error ? (
            <div className="error-message">
              <span>⚠️</span>
              {error}
            </div>
          ) : callers.length === 0 ? (
            <div className="no-callers">
              <span>📭</span>
              <p>没有找到调用者</p>
              <p className="hint">
                这可能是入口函数 (如 module_init) 或未被调用的函数
              </p>
            </div>
          ) : (
            <div className="callers-list">
              <div className="list-header">
                找到 {callers.length} 个调用者
              </div>
              
              {callers.map((caller, index) => (
                <div key={index} className="caller-item">
                  <div 
                    className="caller-main"
                    onClick={() => onFunctionClick?.(caller.name, caller.file, caller.line)}
                  >
                    <button 
                      className="expand-btn"
                      onClick={(e) => {
                        e.stopPropagation()
                        toggleExpand(caller.name)
                      }}
                    >
                      {expandedCallers.has(caller.name) ? '▼' : '▶'}
                    </button>
                    
                    <span className="call-icon" title={getCallTypeLabel(caller.call_type, caller.async_mechanism)}>
                      {getCallTypeIcon(caller.call_type, caller.async_mechanism)}
                    </span>
                    
                    <div className="caller-info">
                      <code className="func-name">{caller.name}()</code>
                      <span className="location">
                        {caller.file.split('/').pop()}:{caller.line}
                      </span>
                    </div>
                    
                    <span className={`call-type ${caller.call_type}`}>
                      {getCallTypeLabel(caller.call_type, caller.async_mechanism)}
                    </span>
                  </div>
                  
                  {/* 二级调用者 */}
                  {expandedCallers.has(caller.name) && (
                    <div className="second-level">
                      {secondLevelCallers[caller.name] ? (
                        secondLevelCallers[caller.name].length > 0 ? (
                          secondLevelCallers[caller.name].map((sc, si) => (
                            <div 
                              key={si} 
                              className="second-level-caller"
                              onClick={() => onFunctionClick?.(sc.name, sc.file, sc.line)}
                            >
                              <span className="call-icon">
                                {getCallTypeIcon(sc.call_type, sc.async_mechanism)}
                              </span>
                              <code>{sc.name}()</code>
                              <span className="location">
                                {sc.file.split('/').pop()}:{sc.line}
                              </span>
                            </div>
                          ))
                        ) : (
                          <div className="no-more-callers">无更多调用者</div>
                        )
                      ) : (
                        <div className="loading-second">正在加载...</div>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
        
        <div className="callers-footer">
          <div className="legend">
            <span className="legend-item">
              <span>📞</span> 直接调用
            </span>
            <span className="legend-item">
              <span>⏳</span> 异步调用
            </span>
            <span className="legend-item">
              <span>↩️</span> 函数指针
            </span>
          </div>
        </div>
      </div>
    </div>
  )
}

export default CallersView

