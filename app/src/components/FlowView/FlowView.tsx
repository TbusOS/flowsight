/**
 * FlowView - 执行流可视化组件
 * 
 * 使用 React Flow 显示代码执行流程图
 */

import React, { useCallback, useMemo } from 'react'
import {
  ReactFlow,
  Node,
  Edge,
  Controls,
  Background,
  BackgroundVariant,
  useNodesState,
  useEdgesState,
  NodeTypes,
  MarkerType,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { FlowNodeComponent } from './FlowNode'
import type { FlowTreeNode } from '../../types'
import './FlowView.css'

// 自定义节点类型
const nodeTypes: NodeTypes = {
  flowNode: FlowNodeComponent,
}

interface FlowViewProps {
  flowTrees: FlowTreeNode[]
  onNodeClick?: (nodeId: string, functionName: string) => void
}

// 将 FlowTree 转换为 React Flow 的节点和边
function convertToReactFlow(
  flowTrees: FlowTreeNode[]
): { nodes: Node[]; edges: Edge[] } {
  const nodes: Node[] = []
  const edges: Edge[] = []
  
  let yOffset = 0
  const xSpacing = 250
  const ySpacing = 100

  function processNode(
    node: FlowTreeNode,
    depth: number,
    parentId: string | null,
    index: number
  ): string {
    const nodeId = `${node.name}-${depth}-${index}`
    
    nodes.push({
      id: nodeId,
      type: 'flowNode',
      position: { x: depth * xSpacing, y: yOffset },
      data: {
        label: node.display_name || node.name,
        name: node.name,
        nodeType: node.node_type,
        description: node.description,
        icon: getNodeIcon(node.node_type),
      },
    })
    
    yOffset += ySpacing

    // 添加边
    if (parentId) {
      const edgeType = getEdgeType(node.node_type)
      edges.push({
        id: `${parentId}-${nodeId}`,
        source: parentId,
        target: nodeId,
        type: 'smoothstep',
        animated: edgeType === 'async',
        style: {
          stroke: edgeType === 'async' ? '#fbbf24' : '#64748b',
          strokeWidth: 2,
        },
        markerEnd: {
          type: MarkerType.ArrowClosed,
          color: edgeType === 'async' ? '#fbbf24' : '#64748b',
        },
        label: edgeType === 'async' ? '异步' : undefined,
        labelStyle: { fill: '#fbbf24', fontSize: 10 },
      })
    }

    // 递归处理子节点
    if (node.children) {
      node.children.forEach((child, idx) => {
        processNode(child, depth + 1, nodeId, idx)
      })
    }

    return nodeId
  }

  flowTrees.forEach((tree, index) => {
    processNode(tree, 0, null, index)
    yOffset += ySpacing // 树之间的间距
  })

  return { nodes, edges }
}

function getEdgeType(nodeType: FlowTreeNode['node_type']): 'sync' | 'async' {
  if (typeof nodeType === 'object' && 'AsyncCallback' in nodeType) {
    return 'async'
  }
  return 'sync'
}

// 获取节点图标
function getNodeIcon(nodeType: FlowTreeNode['node_type']): string {
  if (typeof nodeType === 'string') {
    switch (nodeType) {
      case 'Function': return '📦'
      case 'EntryPoint': return '🚀'
      case 'KernelApi': return '⚙️'
      case 'External': return '🔗'
      default: return '📦'
    }
  }
  if ('AsyncCallback' in nodeType) {
    const mechanism = nodeType.AsyncCallback.mechanism
    if (typeof mechanism === 'object') {
      if ('WorkQueue' in mechanism) return '⚙️'
      if ('Timer' in mechanism) return '⏲️'
      if ('Tasklet' in mechanism) return '⚡'
      if ('Irq' in mechanism) return '🔌'
      if ('Completion' in mechanism) return '✅'
    }
    return '⚡'
  }
  return '📦'
}

export function FlowView({ flowTrees, onNodeClick }: FlowViewProps) {
  const { nodes: convertedNodes, edges: convertedEdges } = useMemo(
    () => convertToReactFlow(flowTrees),
    [flowTrees]
  )

  const [nodes, setNodes, onNodesChange] = useNodesState(convertedNodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(convertedEdges)
  
  // Update nodes and edges when flowTrees change
  React.useEffect(() => {
    setNodes(convertedNodes)
    setEdges(convertedEdges)
  }, [convertedNodes, convertedEdges, setNodes, setEdges])

  const handleNodeClick = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      if (onNodeClick) {
        onNodeClick(node.id, node.data.name as string)
      }
    },
    [onNodeClick]
  )

  if (flowTrees.length === 0) {
    return (
      <div className="flow-view-empty">
        <div className="empty-icon">📊</div>
        <h3>暂无执行流数据</h3>
        <p>请先分析源代码文件</p>
      </div>
    )
  }

  return (
    <div className="flow-view">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeClick={handleNodeClick}
        nodeTypes={nodeTypes}
        fitView
        minZoom={0.1}
        maxZoom={2}
        defaultEdgeOptions={{
          type: 'smoothstep',
        }}
      >
        <Background variant={BackgroundVariant.Dots} gap={20} size={1} color="#2a2a4a" />
        <Controls 
          showZoom={true}
          showFitView={true}
          showInteractive={false}
        />
      </ReactFlow>
    </div>
  )
}

export default FlowView

