/**
 * FlowNode - 自定义执行流节点组件
 */

import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import './FlowNode.css'

interface FlowNodeData {
  label: string
  name: string
  nodeType: string | { AsyncCallback?: { mechanism: any } }
  description?: string
  icon?: string
  selected?: boolean
  childCount?: number
  isMore?: boolean
}

function getNodeClass(nodeType: FlowNodeData['nodeType']): string {
  if (typeof nodeType === 'string') {
    switch (nodeType) {
      case 'EntryPoint':
        return 'node-entry'
      case 'KernelApi':
        return 'node-kernel'
      case 'External':
        return 'node-external'
      default:
        return 'node-function'
    }
  } else if (nodeType && typeof nodeType === 'object' && 'AsyncCallback' in nodeType) {
    return 'node-async'
  }
  return 'node-function'
}

function getAsyncLabel(nodeType: FlowNodeData['nodeType']): string | null {
  if (typeof nodeType === 'object' && nodeType && 'AsyncCallback' in nodeType) {
    const mechanism = nodeType.AsyncCallback?.mechanism
    if (typeof mechanism === 'object') {
      if ('WorkQueue' in mechanism) {
        const wq = mechanism.WorkQueue as { delayed?: boolean }
        return wq.delayed ? 'WorkQueue { delayed: true }' : 'WorkQueue'
      }
      if ('Timer' in mechanism) return 'Timer'
      if ('Tasklet' in mechanism) return 'Tasklet'
      if ('Irq' in mechanism) return 'IRQ'
      if ('KThread' in mechanism) return 'KThread'
      if ('Completion' in mechanism) return 'Completion'
    }
    return 'Async'
  }
  return null
}

export const FlowNodeComponent = memo(({ data }: NodeProps) => {
  const nodeData = data as unknown as FlowNodeData
  const nodeClass = getNodeClass(nodeData.nodeType)
  const asyncLabel = getAsyncLabel(nodeData.nodeType)
  const isSelected = nodeData.selected
  const isMore = nodeData.isMore

  if (isMore) {
    return (
      <div className="flow-node node-more">
        <Handle type="target" position={Position.Left} />
        <div className="node-content">
          <span className="node-label">{nodeData.label}</span>
        </div>
      </div>
    )
  }

  return (
    <div className={`flow-node ${nodeClass} ${isSelected ? 'selected' : ''}`}>
      <Handle type="target" position={Position.Left} />
      
      {/* 异步类型标签 */}
      {asyncLabel && (
        <div className="node-async-badge">{asyncLabel}</div>
      )}
      
      <div className="node-content">
        <span className="node-icon">{nodeData.icon || '📦'}</span>
        <span className="node-label">{nodeData.name}()</span>
      </div>
      
      {/* 子节点数量指示 */}
      {nodeData.childCount !== undefined && nodeData.childCount > 0 && (
        <div className="node-child-count">
          → {nodeData.childCount}
        </div>
      )}
      
      <Handle type="source" position={Position.Right} />
    </div>
  )
})

FlowNodeComponent.displayName = 'FlowNodeComponent'

