/**
 * Outline - 代码大纲组件
 * 
 * 显示当前文件的函数、结构体列表
 */

import './Outline.css'

export interface OutlineItem {
  name: string
  kind: 'function' | 'struct' | 'variable' | 'macro'
  line: number
  isCallback?: boolean
  returnType?: string
}

interface OutlineProps {
  items: OutlineItem[]
  onItemClick: (item: OutlineItem) => void
  selectedItem?: string
}

const kindIcons: Record<OutlineItem['kind'], string> = {
  function: '📦',
  struct: '🏗️',
  variable: '📌',
  macro: '🔧',
}

const kindColors: Record<OutlineItem['kind'], string> = {
  function: 'var(--accent)',
  struct: 'var(--accent-pink)',
  variable: 'var(--warning)',
  macro: 'var(--success)',
}

export function Outline({ items, onItemClick, selectedItem }: OutlineProps) {
  // 按类型分组
  const functions = items.filter(i => i.kind === 'function')
  const structs = items.filter(i => i.kind === 'struct')
  const others = items.filter(i => i.kind !== 'function' && i.kind !== 'struct')

  if (items.length === 0) {
    return (
      <div className="outline-empty">
        <p>暂无符号</p>
      </div>
    )
  }

  return (
    <div className="outline">
      {functions.length > 0 && (
        <div className="outline-section">
          <h4 className="outline-section-title">
            <span>📦 函数</span>
            <span className="count">{functions.length}</span>
          </h4>
          <ul className="outline-list">
            {functions.map((item, i) => (
              <li 
                key={i}
                className={`outline-item ${selectedItem === item.name ? 'selected' : ''} ${item.isCallback ? 'callback' : ''}`}
                onClick={() => onItemClick(item)}
              >
                <span className="item-icon" style={{ color: kindColors[item.kind] }}>
                  {item.isCallback ? '⚡' : kindIcons[item.kind]}
                </span>
                <span className="item-name">{item.name}</span>
                {item.returnType && (
                  <span className="item-type">{item.returnType}</span>
                )}
                <span className="item-line">:{item.line}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {structs.length > 0 && (
        <div className="outline-section">
          <h4 className="outline-section-title">
            <span>🏗️ 结构体</span>
            <span className="count">{structs.length}</span>
          </h4>
          <ul className="outline-list">
            {structs.map((item, i) => (
              <li 
                key={i}
                className={`outline-item ${selectedItem === item.name ? 'selected' : ''}`}
                onClick={() => onItemClick(item)}
              >
                <span className="item-icon" style={{ color: kindColors[item.kind] }}>
                  {kindIcons[item.kind]}
                </span>
                <span className="item-name">{item.name}</span>
                <span className="item-line">:{item.line}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {others.length > 0 && (
        <div className="outline-section">
          <h4 className="outline-section-title">
            <span>📌 其他</span>
            <span className="count">{others.length}</span>
          </h4>
          <ul className="outline-list">
            {others.map((item, i) => (
              <li 
                key={i}
                className={`outline-item ${selectedItem === item.name ? 'selected' : ''}`}
                onClick={() => onItemClick(item)}
              >
                <span className="item-icon" style={{ color: kindColors[item.kind] }}>
                  {kindIcons[item.kind]}
                </span>
                <span className="item-name">{item.name}</span>
                <span className="item-line">:{item.line}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  )
}

export default Outline

