/**
 * ScenarioResults - 显示场景化分析结果
 * 
 * 核心功能：
 * - 显示执行路径
 * - 显示每个节点的变量状态
 * - 高亮当前选中的节点
 */

import { useState, useMemo } from 'react'
import './ScenarioResults.css'

interface ScenarioState {
  location: string
  variables: Record<string, string>
}

interface ScenarioResultsProps {
  isOpen: boolean
  onClose: () => void
  scenarioName?: string
  path: string[]
  states: ScenarioState[]
  onNodeClick?: (funcName: string) => void
}

export function ScenarioResults({
  isOpen,
  onClose,
  scenarioName,
  path,
  states,
  onNodeClick,
}: ScenarioResultsProps) {
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null)
  const [showVariables, setShowVariables] = useState(true)
  
  // 获取所有唯一变量名
  const allVariables = useMemo(() => {
    const vars = new Set<string>()
    states.forEach(s => {
      Object.keys(s.variables).forEach(v => vars.add(v))
    })
    return Array.from(vars).sort()
  }, [states])
  
  // 检测变量值变化
  const getVariableChange = (varName: string, index: number): 'unchanged' | 'changed' | 'new' => {
    if (index === 0) return 'new'
    const currentVal = states[index].variables[varName]
    const prevVal = states[index - 1].variables[varName]
    if (prevVal === undefined) return 'new'
    if (currentVal !== prevVal) return 'changed'
    return 'unchanged'
  }
  
  if (!isOpen || states.length === 0) return null
  
  return (
    <div className="scenario-results-overlay" onClick={onClose}>
      <div className="scenario-results" onClick={e => e.stopPropagation()}>
        <div className="results-header">
          <h2>📊 场景执行结果</h2>
          {scenarioName && <span className="scenario-name">{scenarioName}</span>}
          <button className="close-btn" onClick={onClose}>×</button>
        </div>
        
        <div className="results-toolbar">
          <span className="path-info">执行路径: {path.length} 个节点</span>
          <label className="toggle-vars">
            <input
              type="checkbox"
              checked={showVariables}
              onChange={e => setShowVariables(e.target.checked)}
            />
            显示变量
          </label>
        </div>
        
        <div className="results-content">
          {/* 执行路径时间线 */}
          <div className="execution-timeline">
            {states.map((state, index) => {
              const funcName = path[index] || '?'
              const isSelected = selectedIndex === index
              
              return (
                <div
                  key={index}
                  className={`timeline-node ${isSelected ? 'selected' : ''}`}
                  onClick={() => {
                    setSelectedIndex(index)
                    if (onNodeClick) onNodeClick(funcName)
                  }}
                >
                  <div className="timeline-marker">
                    <span className="node-index">{index + 1}</span>
                    <div className="timeline-line" />
                  </div>
                  
                  <div className="node-content">
                    <div className="node-header">
                      <code className="func-name">{funcName}()</code>
                      <span className="location">{state.location}</span>
                    </div>
                    
                    {showVariables && Object.keys(state.variables).length > 0 && (
                      <div className="variables">
                        {Object.entries(state.variables).map(([key, value]) => {
                          const change = getVariableChange(key, index)
                          return (
                            <span
                              key={key}
                              className={`variable ${change}`}
                              title={`${key} = ${value}`}
                            >
                              <span className="var-name">{key}</span>
                              <span className="var-eq">=</span>
                              <span className="var-value">{value}</span>
                            </span>
                          )
                        })}
                      </div>
                    )}
                  </div>
                </div>
              )
            })}
          </div>
          
          {/* 变量变化表格 */}
          {showVariables && allVariables.length > 0 && (
            <div className="variables-table-section">
              <h3>📋 变量追踪</h3>
              <div className="variables-table-wrapper">
                <table className="variables-table">
                  <thead>
                    <tr>
                      <th>步骤</th>
                      <th>函数</th>
                      {allVariables.map(v => (
                        <th key={v}>{v}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {states.map((state, index) => (
                      <tr
                        key={index}
                        className={selectedIndex === index ? 'selected' : ''}
                        onClick={() => setSelectedIndex(index)}
                      >
                        <td className="step-num">{index + 1}</td>
                        <td className="func-cell">
                          <code>{path[index]}</code>
                        </td>
                        {allVariables.map(v => {
                          const value = state.variables[v]
                          const change = getVariableChange(v, index)
                          return (
                            <td
                              key={v}
                              className={`var-cell ${change}`}
                            >
                              {value || '-'}
                            </td>
                          )
                        })}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
        
        <div className="results-footer">
          <div className="legend">
            <span className="legend-item">
              <span className="legend-dot new"></span>
              新值
            </span>
            <span className="legend-item">
              <span className="legend-dot changed"></span>
              变化
            </span>
            <span className="legend-item">
              <span className="legend-dot unchanged"></span>
              不变
            </span>
          </div>
          <button className="export-btn" onClick={() => {
            // Export to JSON
            const data = {
              scenario: scenarioName,
              path,
              states,
            }
            const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
            const url = URL.createObjectURL(blob)
            const a = document.createElement('a')
            a.href = url
            a.download = `scenario-${scenarioName || 'result'}.json`
            a.click()
            URL.revokeObjectURL(url)
          }}>
            📥 导出 JSON
          </button>
        </div>
      </div>
    </div>
  )
}

export default ScenarioResults

