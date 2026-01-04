/**
 * FlowNode - 自定义执行流节点组件
 */

import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import './FlowNode.css'

interface FlowNodeData {
  label: string
  name: string
  nodeType: string | { AsyncCallback?: { mechanism: string } }
  description?: string
}

function getNodeIcon(nodeType: FlowNodeData['nodeType']): string {
  if (typeof nodeType === 'string') {
    switch (nodeType) {
      case 'Function':
        return '📦'
      case 'EntryPoint':
        return '🚀'
      case 'KernelApi':
        return '⚙️'
      case 'External':
        return '🔗'
      default:
        return '📍'
    }
  } else if (nodeType.AsyncCallback) {
    const mechanism = nodeType.AsyncCallback.mechanism
    if (typeof mechanism === 'object') {
      if ('WorkQueue' in mechanism) return '⚡'
      if ('Timer' in mechanism) return '⏲️'
      if ('Interrupt' in mechanism) return '🔌'
      if ('Tasklet' in mechanism) return '🔄'
      if ('KThread' in mechanism) return '🧵'
    }
    return '⚡'
  }
  return '📍'
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
  } else if (nodeType.AsyncCallback) {
    return 'node-async'
  }
  return 'node-function'
}

export const FlowNodeComponent = memo(({ data }: NodeProps) => {
  const nodeData = data as FlowNodeData
  const icon = getNodeIcon(nodeData.nodeType)
  const nodeClass = getNodeClass(nodeData.nodeType)

  return (
    <div className={`flow-node ${nodeClass}`}>
      <Handle type="target" position={Position.Left} />
      
      <div className="node-content">
        <span className="node-icon">{icon}</span>
        <span className="node-label">{nodeData.label}</span>
      </div>
      
      {nodeData.description && (
        <div className="node-description">{nodeData.description}</div>
      )}
      
      <Handle type="source" position={Position.Right} />
    </div>
  )
})

FlowNodeComponent.displayName = 'FlowNodeComponent'

