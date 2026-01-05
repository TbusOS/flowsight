/**
 * FlowView - 执行流可视化组件
 * 
 * 使用 React Flow 显示代码执行流程图
 */

import React, { useCallback, useMemo, useRef, useEffect } from 'react'
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
  useReactFlow,
  ReactFlowProvider,
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
  selectedFunction?: string // 新增：当前选中的函数名
}

// 计算树的高度（子节点数量）
function getTreeHeight(node: FlowTreeNode): number {
  if (!node.children || node.children.length === 0) return 1
  return node.children.reduce((sum, child) => sum + getTreeHeight(child), 0)
}

// 将 FlowTree 转换为 React Flow 的节点和边 - 改进的水平布局
function convertToReactFlow(
  flowTrees: FlowTreeNode[]
): { nodes: Node[]; edges: Edge[]; nodeMap: Map<string, string> } {
  const nodes: Node[] = []
  const edges: Edge[] = []
  const nodeMap = new Map<string, string>() // 函数名 -> 节点ID
  
  const xSpacing = 280
  const ySpacing = 80
  let globalIndex = 0

  function processNode(
    node: FlowTreeNode,
    depth: number,
    parentId: string | null,
    yStart: number
  ): { nodeId: string; height: number } {
    const nodeId = `node-${globalIndex++}`
    const treeHeight = getTreeHeight(node)
    const nodeY = yStart + (treeHeight * ySpacing) / 2 - ySpacing / 2
    
    // 保存函数名到节点ID的映射
    nodeMap.set(node.name, nodeId)
    
    nodes.push({
      id: nodeId,
      type: 'flowNode',
      position: { x: depth * xSpacing, y: nodeY },
      data: {
        label: node.display_name || node.name,
        name: node.name,
        nodeType: node.node_type,
        description: node.description,
        icon: getNodeIcon(node.node_type),
        childCount: node.children?.length || 0,
      },
    })

    // 添加边
    if (parentId) {
      const edgeType = getEdgeType(node.node_type)
      const isAsync = edgeType === 'async'
      edges.push({
        id: `${parentId}-${nodeId}`,
        source: parentId,
        target: nodeId,
        type: 'smoothstep',
        animated: isAsync,
        style: {
          stroke: isAsync ? '#fbbf24' : '#475569',
          strokeWidth: isAsync ? 2 : 1.5,
        },
        markerEnd: {
          type: MarkerType.ArrowClosed,
          color: isAsync ? '#fbbf24' : '#475569',
          width: 15,
          height: 15,
        },
        label: isAsync ? '⚡异步' : undefined,
        labelStyle: { fill: '#fbbf24', fontSize: 10, fontWeight: 500 },
        labelBgStyle: { fill: '#1e293b', fillOpacity: 0.8 },
        labelBgPadding: [4, 4] as [number, number],
      })
    }

    // 递归处理子节点
    let currentY = yStart
    if (node.children && node.children.length > 0) {
      // 限制显示的子节点数量，避免太长
      const maxChildren = 8
      const childrenToShow = node.children.slice(0, maxChildren)
      
      childrenToShow.forEach((child) => {
        const childResult = processNode(child, depth + 1, nodeId, currentY)
        currentY += childResult.height * ySpacing
      })
      
      // 如果有更多子节点，显示省略节点
      if (node.children.length > maxChildren) {
        const moreId = `more-${globalIndex++}`
        nodes.push({
          id: moreId,
          type: 'flowNode',
          position: { x: (depth + 1) * xSpacing, y: currentY },
          data: {
            label: `... 还有 ${node.children.length - maxChildren} 个`,
            name: 'more',
            nodeType: 'External',
            isMore: true,
          },
        })
        edges.push({
          id: `${nodeId}-${moreId}`,
          source: nodeId,
          target: moreId,
          type: 'smoothstep',
          style: { stroke: '#475569', strokeDasharray: '5,5' },
        })
      }
    }

    return { nodeId, height: treeHeight }
  }

  let currentY = 0
  flowTrees.forEach((tree) => {
    const result = processNode(tree, 0, null, currentY)
    currentY += result.height * ySpacing + ySpacing * 2 // 树之间的间距
  })

  return { nodes, edges, nodeMap }
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

// 内部组件，用于访问 ReactFlow 实例
function FlowViewInner({ flowTrees, onNodeClick, selectedFunction }: FlowViewProps) {
  const { nodes: convertedNodes, edges: convertedEdges, nodeMap } = useMemo(
    () => convertToReactFlow(flowTrees),
    [flowTrees]
  )

  const [nodes, setNodes, onNodesChange] = useNodesState(convertedNodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(convertedEdges)
  const nodeMapRef = useRef(nodeMap)
  const { fitView, setCenter, getNode } = useReactFlow()
  
  // Update refs
  useEffect(() => {
    nodeMapRef.current = nodeMap
  }, [nodeMap])
  
  // Update nodes and edges when flowTrees change
  useEffect(() => {
    setNodes(convertedNodes)
    setEdges(convertedEdges)
    // 自动适应视图
    setTimeout(() => fitView({ padding: 0.2 }), 100)
  }, [convertedNodes, convertedEdges, setNodes, setEdges, fitView])
  
  // 当选中函数改变时，自动跳转到对应节点
  useEffect(() => {
    if (selectedFunction && nodeMapRef.current.has(selectedFunction)) {
      const nodeId = nodeMapRef.current.get(selectedFunction)!
      const node = getNode(nodeId)
      if (node) {
        // 平滑滚动到节点位置
        setCenter(node.position.x + 100, node.position.y + 30, { 
          zoom: 1.2, 
          duration: 500 
        })
        
        // 高亮选中的节点
        setNodes(nds => nds.map(n => ({
          ...n,
          data: {
            ...n.data,
            selected: n.id === nodeId,
          }
        })))
      }
    }
  }, [selectedFunction, getNode, setCenter, setNodes])

  const handleNodeClick = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      if (onNodeClick && node.data.name !== 'more') {
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
    <ReactFlow
      nodes={nodes}
      edges={edges}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onNodeClick={handleNodeClick}
      nodeTypes={nodeTypes}
      fitView
      fitViewOptions={{ padding: 0.2 }}
      minZoom={0.1}
      maxZoom={2}
      defaultEdgeOptions={{
        type: 'smoothstep',
      }}
    >
      <Background variant={BackgroundVariant.Dots} gap={20} size={1} color="#1e293b" />
      <Controls 
        showZoom={true}
        showFitView={true}
        showInteractive={false}
      />
    </ReactFlow>
  )
}

export function FlowView(props: FlowViewProps) {
  if (props.flowTrees.length === 0) {
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
      <ReactFlowProvider>
        <FlowViewInner {...props} />
      </ReactFlowProvider>
    </div>
  )
}

export default FlowView

