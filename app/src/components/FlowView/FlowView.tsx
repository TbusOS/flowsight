/**
 * FlowView - 执行流可视化组件
 * 
 * 支持折叠展开、聚焦模式、内核API过滤
 */

import React, { useCallback, useMemo, useState, useEffect, useRef } from 'react'
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

import { FlowNodeComponent } from './FlowNode'
import { toPng } from 'html-to-image'
import type { FlowTreeNode } from '../../types'
import './FlowView.css'

const nodeTypes: NodeTypes = {
  flowNode: FlowNodeComponent,
}

// 常见内核 API 函数列表（会被过滤隐藏）
const KERNEL_API_LIST = new Set([
  // 内存管理
  'kmalloc', 'kzalloc', 'kcalloc', 'krealloc', 'kfree',
  'vmalloc', 'vzalloc', 'vfree',
  'kmem_cache_alloc', 'kmem_cache_free', 'kmem_cache_create', 'kmem_cache_destroy',
  'get_zeroed_page', 'free_page', '__get_free_pages', 'free_pages',
  'devm_kmalloc', 'devm_kzalloc', 'devm_kcalloc', 'devm_kfree',
  
  // 打印/调试
  'printk', 'pr_info', 'pr_err', 'pr_warn', 'pr_debug', 'pr_notice', 'pr_emerg',
  'dev_info', 'dev_err', 'dev_warn', 'dev_dbg', 'dev_notice',
  'dump_stack', 'WARN', 'WARN_ON', 'WARN_ONCE', 'BUG', 'BUG_ON',
  
  // 自旋锁
  'spin_lock', 'spin_unlock', 'spin_lock_irq', 'spin_unlock_irq',
  'spin_lock_irqsave', 'spin_unlock_irqrestore', 'spin_lock_bh', 'spin_unlock_bh',
  'spin_lock_init', 'spin_trylock',
  
  // 互斥锁
  'mutex_lock', 'mutex_unlock', 'mutex_trylock', 'mutex_init',
  'mutex_lock_interruptible', 'mutex_lock_killable',
  
  // 读写锁
  'read_lock', 'read_unlock', 'write_lock', 'write_unlock',
  'down_read', 'up_read', 'down_write', 'up_write',
  
  // 原子操作
  'atomic_set', 'atomic_read', 'atomic_inc', 'atomic_dec',
  'atomic_add', 'atomic_sub', 'atomic_inc_return', 'atomic_dec_return',
  'atomic_cmpxchg', 'atomic_xchg', 'test_and_set_bit', 'test_and_clear_bit',
  
  // 引用计数
  'kref_init', 'kref_get', 'kref_put',
  'get_device', 'put_device',
  
  // 字符串操作
  'memset', 'memcpy', 'memmove', 'memcmp',
  'strcpy', 'strncpy', 'strcmp', 'strncmp', 'strlen', 'strnlen',
  'sprintf', 'snprintf', 'sscanf', 'kstrdup', 'kstrndup',
  
  // 链表操作
  'list_add', 'list_add_tail', 'list_del', 'list_del_init',
  'list_empty', 'list_for_each', 'list_for_each_safe',
  'INIT_LIST_HEAD', 'list_move', 'list_move_tail',
  
  // 等待/完成
  'wait_for_completion', 'complete', 'init_completion',
  'wait_event', 'wait_event_interruptible', 'wake_up', 'wake_up_interruptible',
  
  // 时间/延迟
  'jiffies', 'msleep', 'usleep_range', 'udelay', 'mdelay', 'ndelay',
  'schedule', 'schedule_timeout', 'cond_resched',
  
  // 错误处理
  'IS_ERR', 'PTR_ERR', 'ERR_PTR', 'IS_ERR_OR_NULL',
  
  // 其他常用
  'container_of', 'likely', 'unlikely', 'ACCESS_ONCE',
  'cpu_to_le16', 'cpu_to_le32', 'le16_to_cpu', 'le32_to_cpu',
  'min', 'max', 'clamp', 'ARRAY_SIZE',
])

interface FlowViewProps {
  flowTrees: FlowTreeNode[]
  onNodeClick?: (nodeId: string, functionName: string) => void
  selectedFunction?: string
}

// 构建扁平的函数映射
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
  if (typeof nodeType === 'object' && 'AsyncCallback' in nodeType) {
    return '⚡'
  }
  return '📦'
}

// 获取异步标签
function getAsyncLabel(nodeType: FlowTreeNode['node_type']): string | null {
  if (typeof nodeType === 'object' && nodeType && 'AsyncCallback' in nodeType) {
    const mechanism = nodeType.AsyncCallback?.mechanism
    if (typeof mechanism === 'object') {
      if ('WorkQueue' in mechanism) return 'WorkQueue'
      if ('Timer' in mechanism) return 'Timer'
      if ('Tasklet' in mechanism) return 'Tasklet'
      if ('Irq' in mechanism) return 'IRQ'
      if ('KThread' in mechanism) return 'KThread'
    }
    return 'Async'
  }
  return null
}

// 获取节点类型类名
function getNodeClass(nodeType: FlowTreeNode['node_type']): string {
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

interface ExpandState {
  [key: string]: boolean
}

// 检查是否为内核 API
function isKernelApi(name: string): boolean {
  return KERNEL_API_LIST.has(name)
}

// 本地存储键
const STORAGE_KEY_EXPANDED = 'flowsight_expanded_nodes'
const STORAGE_KEY_HIDE_KERNEL = 'flowsight_hide_kernel_api'

// 加载保存的状态
function loadPersistedState(): { expanded: ExpandState; hideKernel: boolean } {
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

// 保存状态
function persistExpandedState(state: ExpandState) {
  try {
    localStorage.setItem(STORAGE_KEY_EXPANDED, JSON.stringify(state))
  } catch (e) {
    console.warn('Failed to persist expanded state:', e)
  }
}

function persistHideKernel(hide: boolean) {
  try {
    localStorage.setItem(STORAGE_KEY_HIDE_KERNEL, String(hide))
  } catch (e) {
    console.warn('Failed to persist hide kernel state:', e)
  }
}

// 内部组件
function FlowViewInner({ flowTrees, onNodeClick, selectedFunction }: FlowViewProps) {
  const persistedState = useMemo(loadPersistedState, [])
  const [expandedNodes, setExpandedNodes] = useState<ExpandState>(persistedState.expanded)
  const [hideKernelApi, setHideKernelApi] = useState(persistedState.hideKernel) // 隐藏内核API开关
  const [focusedNode, setFocusedNode] = useState<string | null>(null) // 聚焦的节点
  const [isFullscreen, setIsFullscreen] = useState(false) // 全屏模式
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; nodeName: string } | null>(null)
  const functionMap = useMemo(() => buildFunctionMap(flowTrees), [flowTrees])
  const { fitView, setCenter, getNode, zoomIn, zoomOut, setViewport } = useReactFlow()
  const { zoom } = useViewport()
  const flowRef = useRef<HTMLDivElement>(null)
  
  // 导出为 PNG
  const exportToPng = useCallback(async () => {
    const flowElement = document.querySelector('.react-flow__viewport') as HTMLElement
    if (!flowElement) return
    
    try {
      const dataUrl = await toPng(flowElement, {
        backgroundColor: '#0c1222',
        pixelRatio: 2,
      })
      
      // 创建下载链接
      const link = document.createElement('a')
      link.download = 'flowsight-execution-flow.png'
      link.href = dataUrl
      link.click()
    } catch (err) {
      console.error('导出失败:', err)
    }
  }, [])
  
  // 全屏切换
  const toggleFullscreen = useCallback(() => {
    setIsFullscreen(prev => !prev)
  }, [])
  
  // ESC 退出全屏
  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isFullscreen) {
        setIsFullscreen(false)
      }
    }
    window.addEventListener('keydown', handleEsc)
    return () => window.removeEventListener('keydown', handleEsc)
  }, [isFullscreen])
  const isInitialized = useRef(false)
  const prevFlowTreesRef = useRef<FlowTreeNode[]>([])
  
  // 初始化：展开所有入口点的第一层
  useEffect(() => {
    // 只在 flowTrees 变化时重置
    if (flowTrees !== prevFlowTreesRef.current && flowTrees.length > 0) {
      prevFlowTreesRef.current = flowTrees
      const initial: ExpandState = {}
      flowTrees.forEach(tree => {
        initial[tree.name] = true
      })
      setExpandedNodes(initial)
      isInitialized.current = false
    }
  }, [flowTrees])

  // 切换节点展开状态
  const toggleExpand = useCallback((nodeName: string) => {
    setExpandedNodes(prev => {
      const newState = {
        ...prev,
        [nodeName]: !prev[nodeName]
      }
      persistExpandedState(newState)
      return newState
    })
  }, [])
  
  // 处理右键菜单
  const handleContextMenu = useCallback((e: React.MouseEvent, nodeName: string) => {
    e.preventDefault()
    setContextMenu({ x: e.clientX, y: e.clientY, nodeName })
  }, [])

  // 构建可视化节点和边
  const { nodes, edges, nodeIdMap } = useMemo(() => {
    const nodes: Node[] = []
    const edges: Edge[] = []
    const nodeIdMap = new Map<string, string>()
    const processedNodes = new Set<string>()
    
    const xSpacing = 280
    const ySpacing = 55
    let globalY = 0

    function processNode(
      node: FlowTreeNode,
      depth: number,
      parentId: string | null
    ): void {
      // 如果启用了内核API过滤，跳过内核API节点
      if (hideKernelApi && isKernelApi(node.name)) {
        // 但如果有子节点，仍然处理子节点（直接连到父节点）
        if (node.children && expandedNodes[node.name]) {
          node.children.forEach(child => {
            processNode(child, depth, parentId)
          })
        }
        return
      }
      
      // 避免循环引用
      const nodeKey = `${node.name}-${depth}`
      if (processedNodes.has(nodeKey) && depth > 0) {
        return
      }
      processedNodes.add(nodeKey)
      
      const nodeId = `node-${nodes.length}`
      const isExpanded = expandedNodes[node.name] || false
      
      // 计算实际可见的子节点数
      let visibleChildren = node.children || []
      if (hideKernelApi) {
        visibleChildren = visibleChildren.filter(c => !isKernelApi(c.name))
      }
      const hasChildren = visibleChildren.length > 0
      const childCount = visibleChildren.length
      
      nodeIdMap.set(node.name, nodeId)
      
      nodes.push({
        id: nodeId,
        type: 'flowNode',
        position: { x: depth * xSpacing, y: globalY },
        data: {
          name: node.name,
          icon: getNodeIcon(node.node_type),
          nodeClass: getNodeClass(node.node_type),
          asyncLabel: getAsyncLabel(node.node_type),
          isExpanded,
          hasChildren,
          childCount,
          isSelected: selectedFunction === node.name,
          onToggle: () => toggleExpand(node.name),
          onContextMenu: (e: React.MouseEvent) => handleContextMenu(e, node.name),
          // 详细信息
          file: node.file,
          line: node.line,
          nodeType: getNodeClass(node.node_type),
        },
      })

      globalY += ySpacing

      // 添加边
      if (parentId) {
        const isAsync = typeof node.node_type === 'object' && 'AsyncCallback' in node.node_type
        const asyncLabel = getAsyncLabel(node.node_type)
        
        // 根据异步类型选择颜色
        let edgeColor = '#475569' // 默认
        if (isAsync) {
          const mechanism = (node.node_type as any)?.AsyncCallback?.mechanism
          if (mechanism) {
            if ('WorkQueue' in mechanism) edgeColor = '#f59e0b' // 橙色 - 工作队列
            else if ('Timer' in mechanism) edgeColor = '#22c55e' // 绿色 - 定时器
            else if ('Interrupt' in mechanism || 'Irq' in mechanism) edgeColor = '#ef4444' // 红色 - 中断
            else if ('Tasklet' in mechanism) edgeColor = '#a855f7' // 紫色 - Tasklet
            else if ('KThread' in mechanism) edgeColor = '#3b82f6' // 蓝色 - 内核线程
            else edgeColor = '#f59e0b' // 默认橙色
          }
        }
        
        edges.push({
          id: `${parentId}-${nodeId}`,
          source: parentId,
          target: nodeId,
          type: 'smoothstep',
          animated: isAsync,
          label: isAsync ? asyncLabel : undefined,
          labelStyle: { fill: edgeColor, fontSize: 10, fontWeight: 600 },
          labelBgStyle: { fill: '#0f1419', fillOpacity: 0.8 },
          labelBgPadding: [4, 2] as [number, number],
          style: {
            stroke: edgeColor,
            strokeWidth: isAsync ? 2 : 1,
          },
          markerEnd: {
            type: MarkerType.ArrowClosed,
            color: edgeColor,
            width: 12,
            height: 12,
          },
        })
      }

      // 如果展开，递归处理子节点
      if (isExpanded && node.children) {
        node.children.forEach(child => {
          processNode(child, depth + 1, nodeId)
        })
      }
    }

    // 在树中查找聚焦节点
    const findNode = (node: FlowTreeNode, name: string): FlowTreeNode | null => {
      if (node.name === name) return node
      if (node.children) {
        for (const child of node.children) {
          const found = findNode(child, name)
          if (found) return found
        }
      }
      return null
    }
    
    // 处理所有入口点 (如果有聚焦节点，只处理该子树)
    if (focusedNode) {
      for (const tree of flowTrees) {
        const focused = findNode(tree, focusedNode)
        if (focused) {
          processNode(focused, 0, null)
          break
        }
      }
    } else {
      flowTrees.forEach(tree => {
        processNode(tree, 0, null)
        globalY += 20 // 入口点之间的间距
      })
    }

    return { nodes, edges, nodeIdMap }
  }, [flowTrees, expandedNodes, selectedFunction, toggleExpand, hideKernelApi, focusedNode, handleContextMenu])

  const [flowNodes, setNodes, onNodesChange] = useNodesState(nodes)
  const [flowEdges, setEdges, onEdgesChange] = useEdgesState(edges)

  // 更新节点（不触发 fitView）
  useEffect(() => {
    setNodes(nodes)
    setEdges(edges)
  }, [nodes, edges, setNodes, setEdges])

  // 只在初始加载时 fitView
  useEffect(() => {
    if (!isInitialized.current && flowNodes.length > 0) {
      isInitialized.current = true
      setTimeout(() => fitView({ padding: 0.2 }), 100)
    }
  }, [flowNodes.length, fitView])

  // 选中函数时跳转到节点
  useEffect(() => {
    if (selectedFunction && nodeIdMap.has(selectedFunction)) {
      const nodeId = nodeIdMap.get(selectedFunction)!
      const node = getNode(nodeId)
      if (node) {
        setCenter(node.position.x + 100, node.position.y + 20, {
          zoom: 1.2,
          duration: 300,
        })
      }
    }
  }, [selectedFunction, nodeIdMap, getNode, setCenter])

  const handleNodeClick = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      if (onNodeClick) {
        onNodeClick(node.id, node.data.name as string)
      }
    },
    [onNodeClick]
  )

  // 展开全部
  const expandAll = useCallback(() => {
    const all: ExpandState = {}
    functionMap.forEach((_, name) => {
      all[name] = true
    })
    setExpandedNodes(all)
  }, [functionMap])

  // 收起全部
  const collapseAll = useCallback(() => {
    const initial: ExpandState = {}
    flowTrees.forEach(tree => {
      initial[tree.name] = true // 只保留入口点展开
    })
    setExpandedNodes(initial)
  }, [flowTrees])
  
  // 折叠到指定深度
  const collapseToDepth = useCallback((maxDepth: number) => {
    const result: ExpandState = {}
    
    // 递归遍历树，只展开到指定深度
    const traverse = (node: FlowTreeNode, depth: number) => {
      if (depth < maxDepth) {
        result[node.name] = true // 展开
      }
      if (node.children && depth < maxDepth) {
        node.children.forEach(child => traverse(child, depth + 1))
      }
    }
    
    flowTrees.forEach(tree => traverse(tree, 0))
    setExpandedNodes(result)
    persistExpandedState(result)
  }, [flowTrees])
  
  // 聚焦到子树
  const focusOnNode = useCallback((nodeName: string) => {
    setFocusedNode(nodeName)
    // 展开聚焦节点
    setExpandedNodes(prev => ({
      ...prev,
      [nodeName]: true,
    }))
  }, [])
  
  // 恢复全部视图
  const clearFocus = useCallback(() => {
    setFocusedNode(null)
  }, [])
  
  // 关闭右键菜单
  const closeContextMenu = useCallback(() => {
    setContextMenu(null)
  }, [])
  
  // 点击空白处关闭右键菜单
  useEffect(() => {
    const handleClick = () => closeContextMenu()
    window.addEventListener('click', handleClick)
    return () => window.removeEventListener('click', handleClick)
  }, [closeContextMenu])

  // 手动 fitView
  const handleFitView = useCallback(() => {
    fitView({ padding: 0.2 })
  }, [fitView])

  // 切换内核API过滤
  const toggleKernelApiFilter = useCallback(() => {
    setHideKernelApi(prev => {
      const newVal = !prev
      persistHideKernel(newVal)
      return newVal
    })
  }, [])
  
  // 键盘快捷键: 数字 1-5 折叠到对应层级
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // 只在没有焦点到输入框时响应
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return
      }
      
      // 数字键 1-5 折叠到对应层级
      const num = parseInt(e.key)
      if (num >= 1 && num <= 5) {
        collapseToDepth(num)
      }
    }
    
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [collapseToDepth])

  return (
    <div className={`flow-view-inner ${isFullscreen ? 'fullscreen' : ''}`}>
      <div className="flow-toolbar">
        <button onClick={expandAll} title="展开全部">
          📂 展开
        </button>
        <button onClick={collapseAll} title="收起全部">
          📁 收起
        </button>
        
        {/* 异步机制图例 */}
        <div className="async-legend">
          <span className="legend-item" title="工作队列 (进程上下文，可睡眠)">
            <span className="legend-dot workqueue"></span>WorkQueue
          </span>
          <span className="legend-item" title="定时器 (软中断上下文)">
            <span className="legend-dot timer"></span>Timer
          </span>
          <span className="legend-item" title="中断 (中断上下文，不可睡眠)">
            <span className="legend-dot irq"></span>IRQ
          </span>
          <span className="legend-item" title="Tasklet (软中断上下文)">
            <span className="legend-dot tasklet"></span>Tasklet
          </span>
        </div>
        <div className="depth-selector">
          <span className="depth-label">层级:</span>
          {[1, 2, 3, 4, 5].map(depth => (
            <button
              key={depth}
              onClick={() => collapseToDepth(depth)}
              className="depth-btn"
              title={`展开到第 ${depth} 层`}
            >
              {depth}
            </button>
          ))}
        </div>
        <div className="toolbar-divider" />
        <button onClick={handleFitView} title="适应视图">
          🎯 适应
        </button>
        <div className="toolbar-divider" />
        <button 
          onClick={toggleKernelApiFilter} 
          className={hideKernelApi ? 'active' : ''}
          title={hideKernelApi ? '显示内核API (已隐藏 kmalloc、printk 等)' : '隐藏内核API'}
        >
          {hideKernelApi ? '🔇 已过滤' : '⚙️ 内核API'}
        </button>
        {focusedNode && (
          <>
            <div className="toolbar-divider" />
            <span className="focus-indicator">
              🔍 聚焦: <code>{focusedNode}</code>
            </span>
            <button onClick={clearFocus} className="clear-focus-btn" title="显示全部">
              ✖ 退出聚焦
            </button>
          </>
        )}
        <div className="toolbar-divider" />
        <button onClick={exportToPng} title="导出为 PNG 图片">
          📷 导出
        </button>
        <button onClick={toggleFullscreen} title={isFullscreen ? '退出全屏 (Esc)' : '全屏显示'}>
          {isFullscreen ? '⊗' : '⛶'} {isFullscreen ? '退出' : '全屏'}
        </button>
      </div>
      
      {/* 右键菜单 */}
      {contextMenu && (
        <div
          className="context-menu"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onClick={closeContextMenu}
        >
          <button onClick={() => { focusOnNode(contextMenu.nodeName); closeContextMenu() }}>
            🔍 只看此分支
          </button>
          <button onClick={() => { toggleExpand(contextMenu.nodeName); closeContextMenu() }}>
            {expandedNodes[contextMenu.nodeName] ? '📁 收起' : '📂 展开'}
          </button>
        </div>
      )}
      <ReactFlow
        nodes={flowNodes}
        edges={flowEdges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeClick={handleNodeClick}
        nodeTypes={nodeTypes}
        fitView={false}
        minZoom={0.1}
        maxZoom={3}
        defaultEdgeOptions={{ type: 'smoothstep' }}
      >
        <Background variant={BackgroundVariant.Dots} gap={20} size={1} color="#1e293b" />
        <Controls showZoom showFitView showInteractive={false} />
        
        {/* 自定义缩放控制 */}
        <div className="zoom-controls">
          <button onClick={() => zoomOut()} title="缩小 (-)">−</button>
          <span className="zoom-level">{Math.round(zoom * 100)}%</span>
          <button onClick={() => zoomIn()} title="放大 (+)">+</button>
          <button onClick={() => fitView({ padding: 0.2 })} title="适应视图">⊙</button>
        </div>
      </ReactFlow>
    </div>
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
