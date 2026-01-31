"use client"

import * as React from "react"
import {
  ReactFlow,
  Node,
  Edge,
  Controls,
  Background,
  BackgroundVariant,
  useNodesState,
  useEdgesState,
  MarkerType,
  Handle,
  Position,
} from "@xyflow/react"
import "@xyflow/react/dist/style.css"
import { invoke } from "../../lib/tauri-api"
import { Loader2, Zap, Play, RefreshCw, Download, Copy, Check, Search, Filter, X } from "lucide-react"
import { cn } from "../../lib/utils"
import { useAtomValue, useSetAtom } from "jotai"
import { currentFileAtom, setSelectedNodeAtom, type SelectedNodeDetail } from "../../lib/atoms/layout-atoms"
import { useAnalysisStore } from "../../store/analysisStore"

// 自定义节点类型
interface FunctionNodeData extends Record<string, unknown> {
  label: string
  type: "entry" | "function" | "async" | "callback"
  line?: number
  isCallback?: boolean
}

// 自定义函数节点组件
function FunctionNode({ data }: { data: FunctionNodeData }) {
  const getNodeStyle = () => {
    switch (data.type) {
      case "entry":
        return "bg-[var(--accent)] text-white border-[var(--accent)]"
      case "async":
        return "bg-purple-500/20 text-purple-300 border-purple-500"
      case "callback":
        return "bg-amber-500/20 text-amber-300 border-amber-500"
      default:
        return "bg-[var(--bg-tertiary)] text-[var(--text-primary)] border-[var(--border-light)]"
    }
  }

  return (
    <div
      className={cn(
        "px-3 py-2 rounded-lg border-2 shadow-lg min-w-[120px] text-center",
        getNodeStyle()
      )}
    >
      <Handle type="target" position={Position.Top} className="!bg-[var(--accent)]" />
      <div className="text-xs font-medium">{data.label}</div>
      {data.line && (
        <div className="text-[10px] opacity-60 mt-0.5">行 {data.line}</div>
      )}
      <Handle type="source" position={Position.Bottom} className="!bg-[var(--accent)]" />
    </div>
  )
}

const nodeTypes = {
  function: FunctionNode,
}

interface FlowViewProps {
  className?: string
}

export function FlowView({ className }: FlowViewProps) {
  const currentFile = useAtomValue(currentFileAtom)
  const currentProject = useAnalysisStore((state) => state.currentProject)
  const setSelectedNode = useSetAtom(setSelectedNodeAtom)
  const getFunctionDetail = useAnalysisStore((state) => state.getFunctionDetail)
  const [nodes, setNodes, onNodesChange] = useNodesState<Node<FunctionNodeData>>([])
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([])
  const [loading, setLoading] = React.useState(false)
  const [selectedFunction, setSelectedFunction] = React.useState<string | null>(null)
  const [error, setError] = React.useState<string | null>(null)

  // 处理节点点击 - 更新选中节点详情
  const handleNodeClick = React.useCallback(async (_: React.MouseEvent, node: Node<FunctionNodeData>) => {
    const funcName = node.data.label
    setSelectedFunction(funcName)

    // 获取详细信息并更新 atom
    if (currentFile) {
      try {
        const detail = await getFunctionDetail(funcName, currentFile)
        if (detail) {
          const nodeDetail: SelectedNodeDetail = {
            id: node.id,
            name: detail.name,
            return_type: detail.return_type,
            parameters: detail.params.map(p => ({ name: p.name, type: p.type_name })),
            file_path: detail.file,
            line: detail.line,
            is_callback: detail.is_callback,
            callback_context: detail.callback_context || undefined,
            calls: detail.calls,
            called_by: detail.called_by,
            node_type: node.data.type,
            description: undefined,
          }
          setSelectedNode(nodeDetail)
        }
      } catch (err) {
        console.error("获取节点详情失败:", err)
        // 即使获取详情失败，也设置基本信息
        setSelectedNode({
          id: node.id,
          name: funcName,
          return_type: "unknown",
          parameters: [],
          file_path: currentFile,
          line: node.data.line || 0,
          is_callback: node.data.isCallback || false,
          calls: [],
          called_by: [],
          node_type: node.data.type,
        })
      }
    }
  }, [currentFile, getFunctionDetail, setSelectedNode])

  // 导出状态
  const [copied, setCopied] = React.useState(false)

  // 搜索和过滤状态
  const [searchQuery, setSearchQuery] = React.useState("")
  const [filterType, setFilterType] = React.useState<"all" | "async" | "callback" | "function">("all")
  const [showSearch, setShowSearch] = React.useState(false)

  // 过滤后的节点和边
  const filteredNodes = React.useMemo(() => {
    let result = nodes
    
    // 按类型过滤
    if (filterType !== "all") {
      result = result.filter(node => node.data.type === filterType)
    }
    
    // 按搜索关键词过滤
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase()
      result = result.filter(node => 
        node.data.label.toLowerCase().includes(query)
      )
    }
    
    return result
  }, [nodes, filterType, searchQuery])

  // 过滤后的边（只保留两端节点都存在的边）
  const filteredEdges = React.useMemo(() => {
    const nodeIds = new Set(filteredNodes.map(n => n.id))
    return edges.filter(edge => 
      nodeIds.has(edge.source) && nodeIds.has(edge.target)
    )
  }, [edges, filteredNodes])

  // 高亮搜索匹配的节点
  const highlightedNodes = React.useMemo(() => {
    if (!searchQuery.trim()) return filteredNodes
    
    const query = searchQuery.toLowerCase()
    return filteredNodes.map(node => ({
      ...node,
      style: {
        ...node.style,
        boxShadow: node.data.label.toLowerCase().includes(query) 
          ? "0 0 0 2px var(--accent)" 
          : undefined
      }
    }))
  }, [filteredNodes, searchQuery])

  // 导出执行流为文本格式
  const exportFlowAsText = React.useCallback(() => {
    if (nodes.length === 0) return ""
    
    let output = `# 执行流分析结果\n`
    output += `入口函数: ${selectedFunction || "unknown"}\n`
    output += `文件: ${currentFile || "unknown"}\n`
    output += `节点数: ${nodes.length}\n`
    output += `边数: ${edges.length}\n\n`
    
    output += `## 函数调用链\n\n`
    nodes.forEach((node, i) => {
      const indent = "  ".repeat(Math.floor(node.position.y / 100))
      const typeIcon = node.data.type === "async" ? "⚡" : 
                       node.data.type === "callback" ? "↩" : "→"
      output += `${indent}${typeIcon} ${node.data.label}`
      if (node.data.line) output += ` (行 ${node.data.line})`
      output += "\n"
    })
    
    output += `\n## 调用关系\n\n`
    edges.forEach(edge => {
      const sourceNode = nodes.find(n => n.id === edge.source)
      const targetNode = nodes.find(n => n.id === edge.target)
      if (sourceNode && targetNode) {
        const isAsync = edge.animated ? " [异步]" : ""
        output += `${sourceNode.data.label} → ${targetNode.data.label}${isAsync}\n`
      }
    })
    
    return output
  }, [nodes, edges, selectedFunction, currentFile])

  // 复制到剪贴板
  const copyToClipboard = React.useCallback(async () => {
    const text = exportFlowAsText()
    if (!text) return
    
    try {
      await navigator.clipboard.writeText(text)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (err) {
      console.error("复制失败:", err)
    }
  }, [exportFlowAsText])

  // 下载为文件
  const downloadAsFile = React.useCallback(() => {
    const text = exportFlowAsText()
    if (!text) return
    
    const blob = new Blob([text], { type: "text/plain;charset=utf-8" })
    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url
    a.download = `execution-flow-${selectedFunction || "analysis"}.txt`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }, [exportFlowAsText, selectedFunction])

  // 加载执行流
  const loadExecutionFlow = React.useCallback(async (entryFunction: string) => {
    if (!currentFile) return

    setLoading(true)
    setError(null)
    try {
      // 调用后端 build_execution_flow 命令
      const flow = await invoke<{
        entry_function: string
        nodes: Array<{
          id: string
          label: string
          node_type: string
          line: number
        }>
        edges: Array<{
          source: string
          target: string
          edge_type: string
        }>
      }>("build_execution_flow", {
        file_path: currentFile,
        entry_function: entryFunction,
        options: { max_depth: 5, expand_async: true },
      })

      // 转换为 React Flow 格式
      const flowNodes: Node<FunctionNodeData>[] = flow.nodes.map((node, index) => ({
        id: node.id,
        type: "function",
        position: { x: 250, y: index * 100 },
        data: {
          label: node.label,
          type: node.node_type as FunctionNodeData["type"],
          line: node.line,
        },
      }))

      const flowEdges: Edge[] = flow.edges.map((edge, index) => ({
        id: `e-${index}`,
        source: edge.source,
        target: edge.target,
        type: "smoothstep",
        animated: edge.edge_type === "async",
        markerEnd: { type: MarkerType.ArrowClosed },
        style: {
          stroke: edge.edge_type === "async" ? "#a855f7" : "var(--accent)",
        },
      }))

      // 自动布局 - 简单的层级布局
      const layoutNodes = autoLayout(flowNodes, flowEdges)
      setNodes(layoutNodes)
      setEdges(flowEdges)
    } catch (err) {
      console.error("加载执行流失败:", err)
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [currentFile, setNodes, setEdges])

  // 简单的自动布局算法
  const autoLayout = (
    nodes: Node<FunctionNodeData>[],
    edges: Edge[]
  ): Node<FunctionNodeData>[] => {
    if (nodes.length === 0) return nodes

    // 构建邻接表
    const children = new Map<string, string[]>()
    const parents = new Map<string, string[]>()

    edges.forEach((edge) => {
      if (!children.has(edge.source)) children.set(edge.source, [])
      children.get(edge.source)!.push(edge.target)
      if (!parents.has(edge.target)) parents.set(edge.target, [])
      parents.get(edge.target)!.push(edge.source)
    })

    // 找到根节点
    const roots = nodes.filter((n) => !parents.has(n.id) || parents.get(n.id)!.length === 0)

    // BFS 计算层级
    const levels = new Map<string, number>()
    const queue = [...roots.map((r) => r.id)]
    queue.forEach((id) => levels.set(id, 0))

    while (queue.length > 0) {
      const current = queue.shift()!
      const currentLevel = levels.get(current) || 0
      const childIds = children.get(current) || []

      childIds.forEach((childId) => {
        if (!levels.has(childId)) {
          levels.set(childId, currentLevel + 1)
          queue.push(childId)
        }
      })
    }

    // 按层级分组
    const levelGroups = new Map<number, string[]>()
    levels.forEach((level, id) => {
      if (!levelGroups.has(level)) levelGroups.set(level, [])
      levelGroups.get(level)!.push(id)
    })

    // 计算位置
    const nodeMap = new Map(nodes.map((n) => [n.id, n]))
    const result: Node<FunctionNodeData>[] = []

    levelGroups.forEach((ids, level) => {
      const startX = (ids.length - 1) * -100
      ids.forEach((id, index) => {
        const node = nodeMap.get(id)
        if (node) {
          result.push({
            ...node,
            position: {
              x: startX + index * 200,
              y: level * 120,
            },
          })
        }
      })
    })

    return result
  }

  // 没有项目时显示空状态
  if (!currentProject) {
    return (
      <div className={cn("flex h-full w-full items-center justify-center bg-[var(--bg-primary)]", className)}>
        <div className="text-center">
          <Zap className="h-12 w-12 mx-auto mb-4 text-[var(--text-muted)]" />
          <h3 className="text-sm font-medium text-[var(--text-primary)] mb-2">执行流视图</h3>
          <p className="text-xs text-[var(--text-muted)]">打开项目后查看执行流</p>
        </div>
      </div>
    )
  }

  // 没有选择函数时显示提示
  if (!selectedFunction && nodes.length === 0) {
    return (
      <div className={cn("flex h-full w-full flex-col bg-[var(--bg-primary)]", className)}>
        {/* 工具栏 */}
        <div className="flex items-center gap-2 px-3 py-2 border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
          <Zap className="h-4 w-4 text-[var(--accent)]" />
          <span className="text-xs font-medium text-[var(--text-primary)]">执行流</span>
          {currentFile && (
            <span className="text-[10px] text-[var(--text-muted)] truncate">
              {currentFile.split("/").pop()}
            </span>
          )}
        </div>

        {/* 空状态 */}
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <Zap className="h-12 w-12 mx-auto mb-4 text-[var(--text-muted)]" />
            <h3 className="text-sm font-medium text-[var(--text-primary)] mb-2">选择入口函数</h3>
            <p className="text-xs text-[var(--text-muted)] mb-4">
              从大纲面板选择一个函数开始分析
            </p>
            {currentFile && (
              <button
                onClick={() => loadExecutionFlow("main")}
                className="inline-flex items-center gap-2 px-3 py-1.5 text-xs bg-[var(--accent)] text-white rounded-md hover:bg-[var(--accent)]/90"
              >
                <Play className="h-3 w-3" />
                分析 main 函数
              </button>
            )}
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className={cn("flex h-full w-full flex-col bg-[var(--bg-primary)]", className)}>
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
        <div className="flex items-center gap-2">
          <Zap className="h-4 w-4 text-[var(--accent)]" />
          <span className="text-xs font-medium text-[var(--text-primary)]">执行流</span>
          {selectedFunction && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-[var(--accent)]/10 text-[var(--accent)]">
              {selectedFunction}
            </span>
          )}
        </div>
        <div className="flex items-center gap-1">
          {/* 搜索按钮 */}
          <button
            onClick={() => setShowSearch(!showSearch)}
            disabled={nodes.length === 0}
            className={cn(
              "p-1.5 rounded text-[var(--text-muted)] hover:text-[var(--text-primary)] disabled:opacity-50",
              showSearch ? "bg-[var(--accent)]/20 text-[var(--accent)]" : "hover:bg-[var(--bg-tertiary)]"
            )}
            title="搜索函数"
          >
            <Search className="h-3.5 w-3.5" />
          </button>
          {/* 过滤下拉 */}
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value as typeof filterType)}
            disabled={nodes.length === 0}
            className="h-7 px-1.5 text-[10px] bg-[var(--bg-tertiary)] border border-[var(--border-subtle)] rounded text-[var(--text-primary)] disabled:opacity-50"
            title="过滤节点类型"
          >
            <option value="all">全部</option>
            <option value="function">普通函数</option>
            <option value="async">异步调用</option>
            <option value="callback">回调函数</option>
          </select>
          {/* 分隔符 */}
          <div className="w-px h-4 bg-[var(--border-subtle)] mx-1" />
          {/* 复制按钮 */}
          <button
            onClick={copyToClipboard}
            disabled={nodes.length === 0}
            className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:text-[var(--text-primary)] disabled:opacity-50"
            title={copied ? "已复制!" : "复制执行流"}
          >
            {copied ? <Check className="h-3.5 w-3.5 text-green-500" /> : <Copy className="h-3.5 w-3.5" />}
          </button>
          {/* 下载按钮 */}
          <button
            onClick={downloadAsFile}
            disabled={nodes.length === 0}
            className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:text-[var(--text-primary)] disabled:opacity-50"
            title="导出执行流"
          >
            <Download className="h-3.5 w-3.5" />
          </button>
          {/* 刷新按钮 */}
          <button
            onClick={() => selectedFunction && loadExecutionFlow(selectedFunction)}
            disabled={loading}
            className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:text-[var(--text-primary)] disabled:opacity-50"
            title="刷新"
          >
            <RefreshCw className={cn("h-3.5 w-3.5", loading && "animate-spin")} />
          </button>
        </div>
      </div>

      {/* 搜索栏 */}
      {showSearch && (
        <div className="flex items-center gap-2 px-3 py-2 border-b border-[var(--border-subtle)] bg-[var(--bg-tertiary)]">
          <Search className="h-3.5 w-3.5 text-[var(--text-muted)]" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="搜索函数名..."
            className="flex-1 bg-transparent text-xs text-[var(--text-primary)] placeholder:text-[var(--text-muted)] outline-none"
            autoFocus
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className="p-1 rounded hover:bg-[var(--bg-secondary)] text-[var(--text-muted)]"
            >
              <X className="h-3 w-3" />
            </button>
          )}
          <span className="text-[10px] text-[var(--text-muted)]">
            {filteredNodes.length}/{nodes.length} 个节点
          </span>
        </div>
      )}

      {/* 流程图 */}
      <div className="flex-1">
        {loading ? (
          <div className="flex items-center justify-center h-full">
            <Loader2 className="h-6 w-6 animate-spin text-[var(--accent)]" />
          </div>
        ) : error ? (
          <div className="flex items-center justify-center h-full">
            <p className="text-xs text-red-400">{error}</p>
          </div>
        ) : (
          <ReactFlow
            nodes={highlightedNodes}
            edges={filteredEdges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onNodeClick={handleNodeClick}
            nodeTypes={nodeTypes}
            fitView
            attributionPosition="bottom-left"
            style={{ background: "var(--bg-primary)" }}
          >
            <Controls
              style={{
                background: "var(--bg-secondary)",
                borderColor: "var(--border-light)",
              }}
            />
            <Background
              variant={BackgroundVariant.Dots}
              gap={20}
              size={1}
              color="var(--text-muted)"
              style={{ opacity: 0.3 }}
            />
          </ReactFlow>
        )}
      </div>
    </div>
  )
}
