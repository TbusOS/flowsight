/**
 * ScenarioPanel - 场景化数据流分析面板
 * 
 * 核心亮点功能：
 * - 允许用户绑定参数值
 * - 模拟代码执行路径
 * - 显示每个节点的变量状态
 */

import { useState, useCallback } from 'react'
import './ScenarioPanel.css'

interface ValueBinding {
  id: string
  path: string
  value: string
  type: 'integer' | 'string' | 'pointer' | 'range' | 'unknown'
}

interface Scenario {
  name: string
  entryFunction: string
  bindings: ValueBinding[]
}

interface ScenarioPanelProps {
  isOpen: boolean
  onClose: () => void
  entryFunction: string
  params: { name: string; type_name: string }[]
  onExecute: (scenario: Scenario) => void
}

export function ScenarioPanel({
  isOpen,
  onClose,
  entryFunction,
  params,
  onExecute,
}: ScenarioPanelProps) {
  const [scenarioName, setScenarioName] = useState('')
  const [bindings, setBindings] = useState<ValueBinding[]>(() => 
    params.map((p, i) => ({
      id: `binding-${i}`,
      path: p.name,
      value: '',
      type: guessType(p.type_name),
    }))
  )
  const [showAdvanced, setShowAdvanced] = useState(false)
  const [savedScenarios, setSavedScenarios] = useState<Scenario[]>([])

  // 根据类型猜测值类型
  function guessType(typeName: string): ValueBinding['type'] {
    if (typeName.includes('*')) return 'pointer'
    if (typeName.includes('int') || typeName.includes('long') || typeName.includes('u32') || typeName.includes('u16')) return 'integer'
    if (typeName.includes('char *') || typeName.includes('string')) return 'string'
    return 'unknown'
  }

  const updateBinding = useCallback((id: string, field: keyof ValueBinding, value: string) => {
    setBindings(prev => prev.map(b => 
      b.id === id ? { ...b, [field]: value } : b
    ))
  }, [])

  const addBinding = useCallback(() => {
    setBindings(prev => [...prev, {
      id: `binding-${Date.now()}`,
      path: '',
      value: '',
      type: 'unknown',
    }])
  }, [])

  const removeBinding = useCallback((id: string) => {
    setBindings(prev => prev.filter(b => b.id !== id))
  }, [])

  const handleExecute = useCallback(() => {
    const scenario: Scenario = {
      name: scenarioName || `${entryFunction}_scenario`,
      entryFunction,
      bindings: bindings.filter(b => b.path && b.value),
    }
    onExecute(scenario)
  }, [scenarioName, entryFunction, bindings, onExecute])

  const handleSave = useCallback(() => {
    const scenario: Scenario = {
      name: scenarioName || `${entryFunction}_scenario_${Date.now()}`,
      entryFunction,
      bindings: bindings.filter(b => b.path && b.value),
    }
    setSavedScenarios(prev => [...prev, scenario])
    // TODO: 保存到本地存储
  }, [scenarioName, entryFunction, bindings])

  const loadScenario = useCallback((scenario: Scenario) => {
    setScenarioName(scenario.name)
    setBindings(scenario.bindings.map((b, i) => ({ ...b, id: `binding-${i}` })))
  }, [])

  if (!isOpen) return null

  return (
    <div className="scenario-panel-overlay" onClick={onClose}>
      <div className="scenario-panel" onClick={e => e.stopPropagation()}>
        <div className="scenario-header">
          <h2>🎯 场景化数据流分析</h2>
          <p className="scenario-subtitle">
            设置参数值，追踪代码执行路径和变量变化
          </p>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>

        <div className="scenario-content">
          {/* 入口函数信息 */}
          <div className="entry-function">
            <label>入口函数</label>
            <code>{entryFunction}()</code>
          </div>

          {/* 场景名称 */}
          <div className="scenario-name-input">
            <label>场景名称 (可选)</label>
            <input
              type="text"
              value={scenarioName}
              onChange={e => setScenarioName(e.target.value)}
              placeholder={`${entryFunction}_scenario`}
            />
          </div>

          {/* 参数绑定列表 */}
          <div className="bindings-section">
            <div className="section-header">
              <h3>📝 参数绑定</h3>
              <button className="add-binding-btn" onClick={addBinding}>
                + 添加变量
              </button>
            </div>

            <div className="bindings-list">
              {bindings.map((binding, index) => (
                <div key={binding.id} className="binding-row">
                  <div className="binding-index">{index + 1}</div>
                  
                  <div className="binding-path">
                    <input
                      type="text"
                      value={binding.path}
                      onChange={e => updateBinding(binding.id, 'path', e.target.value)}
                      placeholder="变量路径 (如 id->idVendor)"
                    />
                  </div>

                  <div className="binding-type">
                    <select
                      value={binding.type}
                      onChange={e => updateBinding(binding.id, 'type', e.target.value)}
                    >
                      <option value="integer">整数</option>
                      <option value="string">字符串</option>
                      <option value="pointer">指针</option>
                      <option value="range">范围</option>
                      <option value="unknown">未知</option>
                    </select>
                  </div>

                  <div className="binding-value">
                    {binding.type === 'pointer' ? (
                      <select
                        value={binding.value}
                        onChange={e => updateBinding(binding.id, 'value', e.target.value)}
                      >
                        <option value="">选择...</option>
                        <option value="valid">有效指针</option>
                        <option value="null">NULL</option>
                      </select>
                    ) : binding.type === 'range' ? (
                      <input
                        type="text"
                        value={binding.value}
                        onChange={e => updateBinding(binding.id, 'value', e.target.value)}
                        placeholder="0..100"
                      />
                    ) : (
                      <input
                        type={binding.type === 'integer' ? 'text' : 'text'}
                        value={binding.value}
                        onChange={e => updateBinding(binding.id, 'value', e.target.value)}
                        placeholder={binding.type === 'integer' ? '0x1234 或 42' : '值'}
                      />
                    )}
                  </div>

                  <button
                    className="remove-binding-btn"
                    onClick={() => removeBinding(binding.id)}
                    title="删除"
                  >
                    🗑️
                  </button>
                </div>
              ))}

              {bindings.length === 0 && (
                <div className="no-bindings">
                  <p>没有参数绑定</p>
                  <p className="hint">点击"添加变量"来绑定参数值</p>
                </div>
              )}
            </div>
          </div>

          {/* 高级选项 */}
          <div className="advanced-section">
            <button 
              className="toggle-advanced"
              onClick={() => setShowAdvanced(!showAdvanced)}
            >
              {showAdvanced ? '▼' : '▶'} 高级选项
            </button>
            
            {showAdvanced && (
              <div className="advanced-options">
                <div className="option-group">
                  <label>
                    <input type="checkbox" defaultChecked />
                    追踪异步回调
                  </label>
                </div>
                <div className="option-group">
                  <label>
                    <input type="checkbox" defaultChecked />
                    显示内核 API 调用
                  </label>
                </div>
                <div className="option-group">
                  <label>最大递归深度</label>
                  <input type="number" defaultValue={10} min={1} max={50} />
                </div>
              </div>
            )}
          </div>

          {/* 已保存的场景 */}
          {savedScenarios.length > 0 && (
            <div className="saved-scenarios">
              <h3>💾 已保存的场景</h3>
              <div className="scenarios-list">
                {savedScenarios.map((s, i) => (
                  <button
                    key={i}
                    className="saved-scenario-btn"
                    onClick={() => loadScenario(s)}
                  >
                    {s.name}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* 使用示例 */}
          <div className="examples-section">
            <h4>📚 使用示例</h4>
            <div className="example">
              <p><strong>USB 设备枚举场景：</strong></p>
              <code>id-&gt;idVendor = 0x1234</code><br />
              <code>id-&gt;idProduct = 0x5678</code>
            </div>
            <div className="example">
              <p><strong>错误处理场景：</strong></p>
              <code>ptr = NULL</code>
            </div>
          </div>
        </div>

        <div className="scenario-footer">
          <button className="save-btn" onClick={handleSave}>
            💾 保存场景
          </button>
          <button className="execute-btn" onClick={handleExecute}>
            ▶️ 执行分析
          </button>
        </div>
      </div>
    </div>
  )
}

export default ScenarioPanel

