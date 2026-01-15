/**
 * FlowView - Enhanced Execution Flow Visualization
 *
 * Enhanced with:
 * - Dagre automatic layout algorithm
 * - Node grouping by file/async mechanism
 * - Hover previews
 * - Path highlighting
 * - Performance optimizations for large trees
 */

import { useCallback, useMemo, useState, useEffect, useRef } from 'react'
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
  useViewport,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import dagre from 'dagre'

import { FlowNodeComponent } from './FlowNode'
import { toPng, toSvg } from 'html-to-image'
import type { FlowTreeNode, FlowNodeType } from '../../types'
import './FlowView.css'

// Node types
const nodeTypes: NodeTypes = {
  flowNode: FlowNodeComponent,
}

// Layout configuration
const LAYOUT_CONFIG = {
  nodeWidth: 180,
  nodeHeight: 60,
  rankSpacing: 80,
  nodeSpacing: 40,
}

// Async mechanism colors
const ASYNC_COLORS: Record<string, string> = {
  WorkQueue: '#f59e0b',
  Timer: '#22c55e',
  Irq: '#ef4444',
  Tasklet: '#a855f7',
  KThread: '#3b82f6',
  Softirq: '#ec4899',
  Completion: '#14b8a6',
  Rcu: '#f97316',
}

// Interface
interface FlowViewProps {
  flowTrees: FlowTreeNode[]
  onNodeClick?: (nodeId: string, functionName: string) => void
  selectedFunction?: string
  layout?: 'auto' | 'tree' | 'dagre'  // Layout algorithm selection
  groupBy?: 'none' | 'file' | 'async'  // Node grouping
}

// Helper: Get node icon
function getNodeIcon(nodeType: FlowNodeType): string {
  if (typeof nodeType === 'string') {
    switch (nodeType) {
      case 'Function': return '📦'
      case 'EntryPoint': return '🚀'
      case 'KernelApi': return '⚙️'
      case 'External': return '🔗'
      default: return '📦'
    }
  }
  if (typeof nodeType === 'object' && 'AsyncCallback' in nodeType) {
    return '⚡'
  }
  return '📦'
}

// Helper: Get async mechanism label
function getAsyncLabel(nodeType: FlowNodeType): string | null {
  if (typeof nodeType === 'object' && nodeType && 'AsyncCallback' in nodeType) {
    const mechanism = nodeType.AsyncCallback?.mechanism
    if (typeof mechanism === 'object') {
      const keys = Object.keys(mechanism)
      if (keys.length > 0) return keys[0]
    }
    return 'Async'
  }
  return null
}

// Helper: Get async mechanism color
function getAsyncColor(nodeType: FlowNodeType): string | null {
  const label = getAsyncLabel(nodeType)
  return label ? ASYNC_COLORS[label] || '#f59e0b' : null
}

// Helper: Get node class
function getNodeClass(nodeType: FlowNodeType): string {
  if (typeof nodeType === 'string') {
    switch (nodeType) {
      case 'EntryPoint': return 'entry'
      case 'KernelApi': return 'kernel'
      case 'External': return 'external'
      default: return 'function'
    }
  }
  if (typeof nodeType === 'object' && 'AsyncCallback' in nodeType) {
    return 'async'
  }
  return 'function'
}

// Helper: Build function map
function buildFunctionMap(flowTrees: FlowTreeNode[]): Map<string, FlowTreeNode> {
  const map = new Map<string, FlowTreeNode>()

  function traverse(node: FlowTreeNode) {
    if (!map.has(node.name)) {
      map.set(node.name, node)
    }
    node.children?.forEach(traverse)
  }

  flowTrees.forEach(traverse)
  return map
}

// Dagre layout algorithm
function applyDagreLayout(nodes: Node[], edges: Edge[]): Node[] {
  const g = new dagre.graphlib.Graph()
  g.setGraph({ rankdir: 'LR', ranksep: LAYOUT_CONFIG.rankSpacing, nodesep: LAYOUT_CONFIG.nodeSpacing })
  g.setDefaultEdgeLabel(() => ({}))

  // Add nodes to graph
  nodes.forEach(node => {
    g.setNode(node.id, {
      width: LAYOUT_CONFIG.nodeWidth,
      height: LAYOUT_CONFIG.nodeHeight,
    })
  })

  // Add edges to graph
  edges.forEach(edge => {
    g.setEdge(edge.source, edge.target)
  })

  // Calculate layout
  dagre.layout(g)

  // Apply positions to nodes
  return nodes.map(node => {
    const nodeData = g.node(node.id)
    if (nodeData) {
      return {
        ...node,
        position: {
          x: nodeData.x - LAYOUT_CONFIG.nodeWidth / 2,
          y: nodeData.y - LAYOUT_CONFIG.nodeHeight / 2,
        },
      }
    }
    return node
  })
}

// Build nodes and edges from flow trees
interface BuildResult {
  nodes: Node[]
  edges: Edge[]
  nodeIdMap: Map<string, string>
}

function buildFlowGraph(
  flowTrees: FlowTreeNode[],
  expandedNodes: Record<string, boolean>,
  hideKernelApi: boolean,
  selectedFunction?: string,
  highlightPath?: string[]
): BuildResult {
  const nodes: Node[] = []
  const edges: Edge[] = []
  const nodeIdMap = new Map<string, string>()
  let nodeIndex = 0

  function processNode(node: FlowTreeNode, depth: number, parentId: string | null): void {
    // Process children first if node is expanded (for better layout)
    if (expandedNodes[node.name] && node.children) {
      const visibleChildren = hideKernelApi
        ? node.children.filter(c => !isKernelApiNode(c))
        : node.children

      visibleChildren.forEach((child, index) => {
        const existingIndex = nodes.findIndex(n => n.data.name === child.name && depth > 0)

        if (existingIndex === -1 || index === 0) {
          processNode(child, depth + 1, `node-${nodeIndex}`)
        }
      })
    }

    // Create node
    const nodeId = `node-${nodeIndex++}`
    const isExpanded = expandedNodes[node.name] || false
    const hasChildren = (node.children?.length || 0) > 0
    const isSelected = selectedFunction === node.name
    const isHighlighted = highlightPath?.includes(node.name)

    // Get async info
    const asyncLabel = getAsyncLabel(node.node_type)
    const asyncColor = getAsyncColor(node.node_type)
    const isAsync = !!asyncLabel

    nodes.push({
      id: nodeId,
      type: 'flowNode',
      position: { x: 0, y: 0 }, // Will be set by layout
      data: {
        name: node.name,
        icon: getNodeIcon(node.node_type),
        nodeClass: getNodeClass(node.node_type),
        asyncLabel,
        asyncColor,
        isExpanded,
        hasChildren,
        childCount: node.children?.length || 0,
        isSelected,
        isHighlighted,
        file: node.location?.file,
        line: node.location?.line,
        confidence: node.confidence,
        description: node.description,
        // Hover preview data
        hoverData: {
          returnType: node.node_type === 'Function' ? 'int' : 'void',
          params: [],
          callCount: node.children?.length || 0,
        },
      },
    })

    nodeIdMap.set(node.name, nodeId)

    // Create edge to parent
    if (parentId) {
      edges.push({
        id: `${parentId}-${nodeId}`,
        source: parentId,
        target: nodeId,
        type: 'smoothstep',
        animated: isAsync,
        label: isAsync ? asyncLabel : undefined,
        labelStyle: {
          fill: asyncColor || '#475569',
          fontSize: 10,
          fontWeight: 600
        },
        labelBgStyle: { fill: '#0f1419', fillOpacity: 0.8 },
        labelBgPadding: [4, 2] as [number, number],
        style: {
          stroke: asyncColor || '#475569',
          strokeWidth: isAsync ? 2 : 1,
          strokeOpacity: isHighlighted ? 1 : 0.8,
        },
        markerEnd: {
          type: MarkerType.ArrowClosed,
          color: asyncColor || '#475569',
          width: 12,
          height: 12,
        },
      })
    }
  }

  // Process all trees
  flowTrees.forEach((tree, treeIndex) => {
    processNode(tree, 0, null)

    // Add spacing between trees
    if (treeIndex < flowTrees.length - 1) {
      // Add invisible node for spacing
      const spacerId = `spacer-${treeIndex}`
      nodes.push({
        id: spacerId,
        type: 'default',
        position: { x: 0, y: 0 },
        data: { label: '', isSpacer: true },
        style: { opacity: 0 },
      })
    }
  })

  // Apply Dagre layout
  const layoutedNodes = applyDagreLayout(nodes, edges)

  return { nodes: layoutedNodes, edges, nodeIdMap }
}

// Check if node is kernel API
function isKernelApiNode(node: FlowTreeNode): boolean {
  const kernelPatterns = [
    /^kmalloc/, /^kzalloc/, /^kcalloc/, /^kfree/,
    /^vmalloc/, /^vzalloc/, /^vfree/,
    /^spin_lock/, /^spin_unlock/,
    /^mutex_lock/, /^mutex_unlock/,
    /^printk/, /^pr_/, /^dev_/,
    /^container_of/, /^list_/,
  ]
  return kernelPatterns.some(p => p.test(node.name))
}

// Storage keys
const STORAGE_KEY_EXPANDED = 'flowsight_expanded_nodes'
const STORAGE_KEY_HIDE_KERNEL = 'flowsight_hide_kernel_api'

// Load persisted state
function loadPersistedState(): { expanded: Record<string, boolean>; hideKernel: boolean } {
  try {
    const expanded = localStorage.getItem(STORAGE_KEY_EXPANDED)
    const hideKernel = localStorage.getItem(STORAGE_KEY_HIDE_KERNEL)
    return {
      expanded: expanded ? JSON.parse(expanded) : {},
      hideKernel: hideKernel === 'true',
    }
  } catch {
    return { expanded: {}, hideKernel: false }
  }
}

// Save state
function persistState(expanded: Record<string, boolean>, hideKernel: boolean) {
  try {
    localStorage.setItem(STORAGE_KEY_EXPANDED, JSON.stringify(expanded))
    localStorage.setItem(STORAGE_KEY_HIDE_KERNEL, String(hideKernel))
  } catch (e) {
    console.warn('Failed to persist state:', e)
  }
}

// Inner component
function FlowViewInner({ flowTrees, onNodeClick, selectedFunction }: FlowViewProps) {
  // State
  const [expandedNodes, setExpandedNodes] = useState<Record<string, boolean>>({})
  const [hideKernelApi, setHideKernelApi] = useState(false)
  const [hoveredNode, setHoveredNode] = useState<string | null>(null)
  const [hoverPosition, setHoverPosition] = useState({ x: 0, y: 0 })
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<string[]>([])
  const [searchIndex, setSearchIndex] = useState(0)
  const [highlightPath, setHighlightPath] = useState<string[] | undefined>()

  // Refs
  const functionMap = useMemo(() => buildFunctionMap(flowTrees), [flowTrees])
  const isInitialized = useRef(false)
  const prevFlowTreesRef = useRef<FlowTreeNode[]>([])

  // React Flow hooks
  const { fitView, setCenter, getNode, zoomIn, zoomOut } = useReactFlow()
  const { zoom } = useViewport()

  // Load persisted state
  useEffect(() => {
    const state = loadPersistedState()
    setExpandedNodes(state.expanded)
    setHideKernelApi(state.hideKernel)
  }, [])

  // Initialize on first load
  useEffect(() => {
    if (flowTrees !== prevFlowTreesRef.current && flowTrees.length > 0) {
      prevFlowTreesRef.current = flowTrees
      const initial: Record<string, boolean> = {}
      flowTrees.forEach(tree => {
        initial[tree.name] = true
      })
      setExpandedNodes(initial)
      isInitialized.current = false
    }
  }, [flowTrees])

  // Build nodes and edges
  const { nodes: layoutedNodes, edges, nodeIdMap } = useMemo(() => {
    return buildFlowGraph(flowTrees, expandedNodes, hideKernelApi, selectedFunction, highlightPath)
  }, [flowTrees, expandedNodes, hideKernelApi, selectedFunction, highlightPath])

  // React Flow state
  const [flowNodes, setFlowNodes, onNodesChange] = useNodesState(layoutedNodes)
  const [flowEdges, setFlowEdges, onEdgesChange] = useEdgesState(edges)

  // Update nodes when layout changes
  useEffect(() => {
    setFlowNodes(layoutedNodes)
    setFlowEdges(edges)
  }, [layoutedNodes, edges, setFlowNodes, setFlowEdges])

  // Fit view on initial load
  useEffect(() => {
    if (!isInitialized.current && flowNodes.length > 0) {
      isInitialized.current = true
      setTimeout(() => fitView({ padding: 0.2 }), 150)
    }
  }, [flowNodes.length, fitView])

  // Center on selected function
  useEffect(() => {
    if (selectedFunction && nodeIdMap.has(selectedFunction)) {
      const nodeId = nodeIdMap.get(selectedFunction)!
      const node = getNode(nodeId)
      if (node) {
        setCenter(node.position.x + 100, node.position.y + 20, { zoom: 1.2, duration: 300 })
      }
    }
  }, [selectedFunction, nodeIdMap, getNode, setCenter])

  // Toggle kernel API filter
  const toggleKernelApi = useCallback(() => {
    setHideKernelApi(prev => {
      const newVal = !prev
      persistState(expandedNodes, newVal)
      return newVal
    })
  }, [expandedNodes])

  // Expand all
  const expandAll = useCallback(() => {
    const all: Record<string, boolean> = {}
    functionMap.forEach((_, name) => all[name] = true)
    setExpandedNodes(all)
  }, [functionMap])

  // Collapse all
  const collapseAll = useCallback(() => {
    const initial: Record<string, boolean> = {}
    flowTrees.forEach(tree => initial[tree.name] = true)
    setExpandedNodes(initial)
  }, [flowTrees])

  // Collapse to depth
  const collapseToDepth = useCallback((maxDepth: number) => {
    const result: Record<string, boolean> = {}

    const traverse = (node: FlowTreeNode, depth: number) => {
      if (depth < maxDepth) result[node.name] = true
      if (node.children && depth < maxDepth) {
        node.children.forEach(child => traverse(child, depth + 1))
      }
    }

    flowTrees.forEach(tree => traverse(tree, 0))
    setExpandedNodes(result)
    persistState(result, hideKernelApi)
  }, [flowTrees, hideKernelApi])

  // Handle node click
  const handleNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    if (onNodeClick) {
      onNodeClick(node.id, node.data.name as string)
    }
  }, [onNodeClick])

  // Handle node hover
  const handleNodeMouseEnter = useCallback((_: React.MouseEvent, node: Node) => {
    setHoveredNode(node.data.name as string)
  }, [])

  const handleNodeMouseLeave = useCallback(() => {
    setHoveredNode(null)
  }, [])

  // Handle mouse move for hover position
  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    setHoverPosition({ x: e.clientX, y: e.clientY })
  }, [])

  // Search functions
  const handleSearch = useCallback((query: string) => {
    setSearchQuery(query)
    if (!query.trim()) {
      setSearchResults([])
      setHighlightPath(undefined)
      return
    }

    const lowerQuery = query.toLowerCase()
    const results = Array.from(functionMap.keys())
      .filter(name => name.toLowerCase().includes(lowerQuery))

    setSearchResults(results)
    setSearchIndex(0)

    if (results.length > 0) {
      jumpToFunction(results[0])
    }
  }, [functionMap])

  // Jump to function
  const jumpToFunction = useCallback((funcName: string) => {
    // Expand parents
    setExpandedNodes(prev => ({
      ...prev,
      [funcName]: true,
    }))

    // Find path from entry point
    const findPath = (node: FlowTreeNode, target: string, path: string[] = []): string[] | null => {
      if (node.name === target) return path
      if (node.children) {
        for (const child of node.children) {
          const result = findPath(child, target, [...path, node.name])
          if (result) return result
        }
      }
      return null
    }

    for (const tree of flowTrees) {
      const path = findPath(tree, funcName)
      if (path) {
        setHighlightPath(path)
        break
      }
    }

    // Center on node
    setTimeout(() => {
      const node = getNode(funcName)
      if (node) {
        setCenter(node.position.x + 100, node.position.y + 30, { zoom: 1.2, duration: 300 })
      }
    }, 100)
  }, [flowTrees, getNode, setCenter])

  // Navigate search results
  const nextSearchResult = useCallback(() => {
    if (searchResults.length === 0) return
    const nextIndex = (searchIndex + 1) % searchResults.length
    setSearchIndex(nextIndex)
    jumpToFunction(searchResults[nextIndex])
  }, [searchResults, searchIndex, jumpToFunction])

  const prevSearchResult = useCallback(() => {
    if (searchResults.length === 0) return
    const prevIndex = (searchIndex - 1 + searchResults.length) % searchResults.length
    setSearchIndex(prevIndex)
    jumpToFunction(searchResults[prevIndex])
  }, [searchResults, searchIndex, jumpToFunction])

  // Export functions
  const exportToPng = useCallback(async () => {
    const flowElement = document.querySelector('.react-flow__viewport') as HTMLElement
    if (!flowElement) return
    try {
      const dataUrl = await toPng(flowElement, { backgroundColor: '#0c1222', pixelRatio: 2 })
      const link = document.createElement('a')
      link.download = 'flowsight-execution-flow.png'
      link.href = dataUrl
      link.click()
    } catch (err) {
      console.error('Export failed:', err)
    }
  }, [])

  const exportToSvg = useCallback(async () => {
    const flowElement = document.querySelector('.react-flow__viewport') as HTMLElement
    if (!flowElement) return
    try {
      const dataUrl = await toSvg(flowElement, { backgroundColor: '#0c1222' })
      const link = document.createElement('a')
      link.download = 'flowsight-execution-flow.svg'
      link.href = dataUrl
      link.click()
    } catch (err) {
      console.error('SVG export failed:', err)
    }
  }, [])

  // Fit view
  const handleFitView = useCallback(() => {
    fitView({ padding: 0.2 })
  }, [fitView])

  // Get hover data
  const hoverData = useMemo(() => {
    if (!hoveredNode) return null
    const node = functionMap.get(hoveredNode)
    if (!node) return null

    return {
      name: node.name,
      icon: getNodeIcon(node.node_type),
      file: node.location?.file,
      line: node.location?.line,
      description: node.description,
      confidence: node.confidence,
      asyncLabel: getAsyncLabel(node.node_type),
    }
  }, [hoveredNode, functionMap])

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return

      const num = parseInt(e.key)
      if (num >= 1 && num <= 5) {
        collapseToDepth(num)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [collapseToDepth])

  return (
    <div className="flow-view-inner enhanced" onMouseMove={handleMouseMove}>
      {/* Toolbar */}
      <div className="flow-toolbar">
        <button onClick={expandAll} title="Expand all">📂</button>
        <button onClick={collapseAll} title="Collapse all">📁</button>

        {/* Async legend */}
        <div className="async-legend">
          {Object.entries(ASYNC_COLORS).map(([name, color]) => (
            <span key={name} className="legend-item" title={name}>
              <span className="legend-dot" style={{ backgroundColor: color }}></span>
              {name}
            </span>
          ))}
        </div>

        {/* Depth selector */}
        <div className="depth-selector">
          {[1, 2, 3, 4, 5].map(depth => (
            <button
              key={depth}
              onClick={() => collapseToDepth(depth)}
              className="depth-btn"
              title={`Expand to depth ${depth}`}
            >
              {depth}
            </button>
          ))}
        </div>

        <div className="toolbar-divider" />

        <button onClick={handleFitView} title="Fit view">🎯</button>
        <button
          onClick={toggleKernelApi}
          className={hideKernelApi ? 'active' : ''}
          title={hideKernelApi ? 'Show kernel APIs' : 'Hide kernel APIs'}
        >
          ⚙️
        </button>

        <div className="toolbar-divider" />

        <button onClick={exportToPng} title="Export PNG">📷</button>
        <button onClick={exportToSvg} title="Export SVG">🖼️</button>

        {/* Search */}
        <div className="flow-search">
          <input
            type="text"
            placeholder="🔍 Search..."
            value={searchQuery}
            onChange={(e) => handleSearch(e.target.value)}
          />
          {searchResults.length > 0 && (
            <>
              <span className="search-count">{searchIndex + 1}/{searchResults.length}</span>
              <button onClick={prevSearchResult}>▲</button>
              <button onClick={nextSearchResult}>▼</button>
            </>
          )}
        </div>
      </div>

      {/* Hover preview tooltip */}
      {hoverData && (
        <div
          className="hover-preview"
          style={{
            left: hoverPosition.x + 15,
            top: hoverPosition.y + 15,
          }}
        >
          <div className="preview-header">
            <span className="preview-icon">{hoverData.icon}</span>
            <span className="preview-name">{hoverData.name}</span>
            {hoverData.asyncLabel && (
              <span className="preview-async" style={{ color: ASYNC_COLORS[hoverData.asyncLabel] }}>
                {hoverData.asyncLabel}
              </span>
            )}
          </div>
          {hoverData.file && (
            <div className="preview-location">
              📍 {hoverData.file?.split('/').pop()}:{hoverData.line}
            </div>
          )}
          {hoverData.description && (
            <div className="preview-description">{hoverData.description}</div>
          )}
          {hoverData.confidence && (
            <div className="preview-confidence">
              Confidence: <span className={`confidence-${hoverData.confidence.level.toLowerCase()}`}>
                {hoverData.confidence.level}
              </span>
            </div>
          )}
        </div>
      )}

      {/* React Flow */}
      <ReactFlow
        nodes={flowNodes}
        edges={flowEdges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeClick={handleNodeClick}
        onNodeMouseEnter={handleNodeMouseEnter}
        onNodeMouseLeave={handleNodeMouseLeave}
        nodeTypes={nodeTypes}
        fitView={false}
        minZoom={0.1}
        maxZoom={3}
        defaultEdgeOptions={{ type: 'smoothstep' }}
      >
        <Background variant={BackgroundVariant.Dots} gap={20} size={1} color="#1e293b" />
        <Controls showZoom showFitView showInteractive={false} />

        {/* Zoom controls */}
        <div className="zoom-controls">
          <button onClick={() => zoomOut()}>−</button>
          <span className="zoom-level">{Math.round(zoom * 100)}%</span>
          <button onClick={() => zoomIn()}>+</button>
          <button onClick={handleFitView}>⊙</button>
        </div>
      </ReactFlow>
    </div>
  )
}

// Main component
export function FlowView(props: FlowViewProps) {
  if (props.flowTrees.length === 0) {
    return (
      <div className="flow-view-empty">
        <div className="empty-icon">📊</div>
        <h3>No execution flow data</h3>
        <p>Please analyze a source file first</p>
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
