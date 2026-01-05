/**
 * FlowNode - 可折叠的执行流节点
 */

import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import './FlowNode.css'

interface FlowNodeData {
  name: string
  icon: string
  nodeClass: string
  asyncLabel?: string | null
  isExpanded: boolean
  hasChildren: boolean
  childCount: number
  isSelected: boolean
  onToggle: () => void
}

export const FlowNodeComponent = memo(({ data }: NodeProps) => {
  const nodeData = data as unknown as FlowNodeData
  const {
    name,
    icon,
    nodeClass,
    asyncLabel,
    isExpanded,
    hasChildren,
    childCount,
    isSelected,
    onToggle,
  } = nodeData

  const handleToggleClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    onToggle()
  }

  return (
    <div className={`flow-node node-${nodeClass} ${isSelected ? 'selected' : ''}`}>
      <Handle type="target" position={Position.Left} />
      
      {/* 异步标签 */}
      {asyncLabel && (
        <div className="node-async-badge">{asyncLabel}</div>
      )}
      
      <div className="node-main">
        {/* 展开/收起按钮 */}
        {hasChildren && (
          <button 
            className={`node-toggle ${isExpanded ? 'expanded' : ''}`}
            onClick={handleToggleClick}
            title={isExpanded ? '收起' : `展开 (${childCount})`}
          >
            {isExpanded ? '▼' : '▶'}
          </button>
        )}
        
        {/* 图标 */}
        <span className="node-icon">{icon}</span>
        
        {/* 函数名 */}
        <span className="node-name" title={`${name}()`}>{name}()</span>
        
        {/* 子节点数量 */}
        {hasChildren && !isExpanded && (
          <span className="node-count">{childCount}</span>
        )}
      </div>
      
      <Handle type="source" position={Position.Right} />
    </div>
  )
})

FlowNodeComponent.displayName = 'FlowNodeComponent'
