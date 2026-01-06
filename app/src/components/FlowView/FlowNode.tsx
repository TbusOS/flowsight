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
  onContextMenu?: (e: React.MouseEvent) => void
  // 详细信息用于 tooltip
  file?: string
  line?: number
  nodeType?: string
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
    onContextMenu,
    file,
    line,
    nodeType,
  } = nodeData

  const handleToggleClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    onToggle()
  }

  // 构建详细 tooltip
  const buildTooltip = () => {
    const parts = [`📌 ${name}()`]
    if (nodeType) {
      const typeLabels: Record<string, string> = {
        'user': '👤 用户定义函数',
        'kernel-api': '🔧 内核 API',
        'external': '📦 外部函数',
        'callback': '⚡ 回调函数',
        'async-callback': '⏰ 异步回调',
      }
      parts.push(`${typeLabels[nodeType] || nodeType}`)
    }
    // 异步机制信息
    if (asyncLabel) {
      const asyncInfo: Record<string, string> = {
        'WorkQueue': '🔄 工作队列 (进程上下文，可睡眠)',
        'Timer': '⏱️ 定时器 (软中断上下文，不可睡眠)',
        'IRQ': '⚡ 硬中断 (中断上下文，不可睡眠)',
        'Tasklet': '📋 Tasklet (软中断上下文)',
        'KThread': '🧵 内核线程 (进程上下文，可睡眠)',
        'Async': '⏳ 异步调用',
      }
      parts.push(asyncInfo[asyncLabel] || `异步: ${asyncLabel}`)
    }
    if (file) {
      const fileName = file.split('/').pop()
      parts.push(`📄 ${fileName}`)
    }
    if (line !== undefined) {
      parts.push(`📍 第 ${line} 行`)
    }
    if (hasChildren) {
      parts.push(`📊 调用 ${childCount} 个函数`)
    }
    return parts.join('\n')
  }

  return (
    <div 
      className={`flow-node node-${nodeClass} ${isSelected ? 'selected' : ''}`}
      onContextMenu={onContextMenu}
      title={buildTooltip()}
    >
      <Handle type="target" position={Position.Left} />
      
      {/* 异步标签 - 根据类型显示不同颜色 */}
      {asyncLabel && (
        <div 
          className={`node-async-badge async-${asyncLabel.toLowerCase()}`}
          data-async-type={asyncLabel}
        >
          {asyncLabel}
        </div>
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
        <span className="node-name">{name}()</span>
        
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
