/**
 * 执行流格式化工具
 * 
 * 支持多种输出格式：
 * - ftrace: 类似 Linux ftrace 的缩进格式
 * - tree: 简洁的树形缩进
 * - markdown: Markdown 文档格式
 * - json: 结构化 JSON
 */

import { FlowTreeNode, AsyncMechanism } from '../types'

// ftrace 风格配置
interface FtraceConfig {
  showCpu?: boolean        // 显示 CPU 列
  showTime?: boolean       // 显示时间占位
}

// 获取异步机制的图标、名称和简短标记
function getAsyncInfo(mechanism: AsyncMechanism): { icon: string; name: string; tag: string } {
  if (typeof mechanism === 'string') {
    switch (mechanism) {
      case 'Tasklet': return { icon: '🔄', name: 'Tasklet', tag: 'TL' }
      case 'Softirq': return { icon: '⚡', name: 'SoftIRQ', tag: 'SI' }
      case 'KThread': return { icon: '🧵', name: 'KThread', tag: 'KT' }
      case 'RcuCallback': return { icon: '🔒', name: 'RCU', tag: 'RCU' }
      case 'Notifier': return { icon: '📢', name: 'Notifier', tag: 'NF' }
      default: return { icon: '📍', name: String(mechanism), tag: 'AS' }
    }
  }
  
  if (typeof mechanism === 'object' && mechanism !== null) {
    if ('WorkQueue' in mechanism) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const wq = mechanism.WorkQueue as any
      const delayed = wq?.delayed === true
      return { 
        icon: '⚙️', 
        name: delayed ? 'DelayedWork' : 'WorkQueue',
        tag: delayed ? 'DW' : 'WQ'
      }
    }
    if ('Timer' in mechanism) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const timer = mechanism.Timer as any
      const highRes = timer?.high_resolution === true
      return { 
        icon: '⏲️', 
        name: highRes ? 'HRTimer' : 'Timer',
        tag: highRes ? 'HR' : 'TM'
      }
    }
    if ('Interrupt' in mechanism) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const irq = mechanism.Interrupt as any
      const threaded = irq?.threaded === true
      return { 
        icon: '⚡', 
        name: threaded ? 'ThreadedIRQ' : 'IRQ',
        tag: threaded ? 'TI' : 'IRQ'
      }
    }
    if ('Custom' in mechanism) {
      const customName = String(mechanism.Custom)
      return { icon: '📍', name: customName, tag: customName.slice(0, 3).toUpperCase() }
    }
  }
  
  return { icon: '📍', name: 'Async', tag: 'AS' }
}

// 格式化信息列（左侧的行号/类型标记）
function formatInfoColumn(node: FlowTreeNode): string {
  const asyncMech = isAsyncCallback(node.node_type)
  
  if (asyncMech) {
    // 异步回调：显示机制类型标记
    const { tag } = getAsyncInfo(asyncMech)
    return `   [${tag}]`.padEnd(14)
  }
  
  if (node.location?.line) {
    // 用户函数：显示行号
    return `   :${node.location.line}`.padEnd(14)
  }
  
  if (node.node_type === 'KernelApi') {
    // 内核 API：显示 [K] 标记
    return '   [K]'.padEnd(14)
  }
  
  if (node.node_type === 'External') {
    // 外部函数：显示 [E] 标记
    return '   [E]'.padEnd(14)
  }
  
  // 默认空白
  return ''.padEnd(14)
}

// 检查节点是否为异步回调
function isAsyncCallback(nodeType: FlowTreeNode['node_type']): AsyncMechanism | null {
  if (typeof nodeType === 'object' && 'AsyncCallback' in nodeType) {
    return nodeType.AsyncCallback.mechanism
  }
  return null
}

/**
 * 将 FlowTree 转换为 ftrace 风格文本
 * 
 * 信息列显示规则：
 * - 用户函数: 显示行号 (如 :45)
 * - 内核 API: 显示 [K]
 * - 异步回调: 显示机制标记 (如 [WQ], [TM])
 * - 外部函数: 显示 [E]
 */
export function toFtraceFormat(
  trees: FlowTreeNode[], 
  config: FtraceConfig = {}
): string {
  const { showCpu = true } = config
  const lines: string[] = []
  
  function renderNode(node: FlowTreeNode, depth: number): void {
    const indent = '  '.repeat(depth)
    const cpuCol = showCpu ? ' 0)' : ''
    const infoCol = formatInfoColumn(node)
    
    const asyncMech = isAsyncCallback(node.node_type)
    const children = node.children || []
    const hasChildren = children.length > 0
    
    if (asyncMech) {
      // 异步回调入口
      const { name } = getAsyncInfo(asyncMech)
      lines.push(`${cpuCol}${infoCol}|${indent}/* ${name} */ ${node.name}() {`)
    } else if (hasChildren) {
      // 有子节点，开启块
      lines.push(`${cpuCol}${infoCol}|${indent}${node.name}() {`)
    } else {
      // 叶子节点
      lines.push(`${cpuCol}${infoCol}|${indent}${node.name}();`)
    }
    
    // 递归渲染子节点
    children.forEach((child) => {
      renderNode(child, depth + 1)
    })
    
    // 关闭块
    if (hasChildren || asyncMech) {
      lines.push(`${cpuCol}${''.padEnd(14)}|${indent}}`)
    }
  }
  
  trees.forEach(tree => {
    renderNode(tree, 0)
    lines.push('') // 入口点之间空一行
  })
  
  return lines.join('\n')
}

/**
 * 将 FlowTree 转换为简洁树形格式
 */
export function toTreeFormat(trees: FlowTreeNode[]): string {
  const lines: string[] = []
  
  function renderNode(node: FlowTreeNode, prefix: string, isLast: boolean): void {
    const connector = isLast ? '└── ' : '├── '
    const asyncMech = isAsyncCallback(node.node_type)
    const children = node.children || []
    
    let label = `${node.name}()`
    if (asyncMech) {
      const { icon, name } = getAsyncInfo(asyncMech)
      label = `${icon} [${name}] ${node.name}()`
    } else if (node.node_type === 'KernelApi') {
      label = `📦 ${node.name}()`
    } else if (node.node_type === 'EntryPoint') {
      label = `🔌 ${node.name}()`
    }
    
    lines.push(`${prefix}${connector}${label}`)
    
    const childPrefix = prefix + (isLast ? '    ' : '│   ')
    children.forEach((child, i) => {
      renderNode(child, childPrefix, i === children.length - 1)
    })
  }
  
  trees.forEach((tree, i) => {
    const asyncMech = isAsyncCallback(tree.node_type)
    const children = tree.children || []
    let rootLabel = `${tree.name}()`
    if (asyncMech) {
      const { icon, name } = getAsyncInfo(asyncMech)
      rootLabel = `${icon} [${name}] ${tree.name}()`
    } else if (tree.node_type === 'EntryPoint') {
      rootLabel = `🚀 ${tree.name}()`
    }
    
    lines.push(rootLabel)
    children.forEach((child, j) => {
      renderNode(child, '', j === children.length - 1)
    })
    
    if (i < trees.length - 1) lines.push('')
  })
  
  return lines.join('\n')
}

/**
 * 将 FlowTree 转换为 Markdown 格式
 */
export function toMarkdownFormat(
  trees: FlowTreeNode[], 
  options: { title?: string } = {}
): string {
  const { title = '执行流分析' } = options
  const lines: string[] = []
  
  lines.push(`# ${title}`)
  lines.push('')
  lines.push(`> 生成时间: ${new Date().toLocaleString('zh-CN')}`)
  lines.push('')
  
  // 入口点概览
  lines.push('## 入口点')
  lines.push('')
  trees.forEach(tree => {
    const asyncMech = isAsyncCallback(tree.node_type)
    if (asyncMech) {
      const { name } = getAsyncInfo(asyncMech)
      lines.push(`- \`${tree.name}()\` — ${name} 回调`)
    } else {
      lines.push(`- \`${tree.name}()\``)
    }
  })
  lines.push('')
  
  // 详细执行流
  lines.push('## 执行流详情')
  lines.push('')
  
  trees.forEach(tree => {
    lines.push(`### ${tree.name}()`)
    lines.push('')
    lines.push('```')
    lines.push(toTreeFormat([tree]))
    lines.push('```')
    lines.push('')
    
    // ftrace 风格
    lines.push('<details>')
    lines.push('<summary>ftrace 风格输出</summary>')
    lines.push('')
    lines.push('```')
    lines.push(toFtraceFormat([tree]))
    lines.push('```')
    lines.push('')
    lines.push('</details>')
    lines.push('')
  })
  
  return lines.join('\n')
}

/**
 * 导出格式选项
 */
export type ExportFormat = 'ftrace' | 'tree' | 'markdown' | 'json'

/**
 * 统一导出接口
 */
export function exportFlowTrees(
  trees: FlowTreeNode[], 
  format: ExportFormat,
  options?: { title?: string }
): string {
  switch (format) {
    case 'ftrace':
      return toFtraceFormat(trees)
    case 'tree':
      return toTreeFormat(trees)
    case 'markdown':
      return toMarkdownFormat(trees, options)
    case 'json':
      return JSON.stringify(trees, null, 2)
    default:
      return toFtraceFormat(trees)
  }
}

/**
 * 获取导出文件扩展名
 */
export function getExportExtension(format: ExportFormat): string {
  switch (format) {
    case 'markdown': return '.md'
    case 'json': return '.json'
    default: return '.txt'
  }
}

