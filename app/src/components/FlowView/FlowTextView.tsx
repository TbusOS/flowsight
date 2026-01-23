/**
 * 文本格式执行流视图 (ftrace风格)
 *
 * 支持 ftrace、tree 等纯文本格式显示
 * 交互式行选择、异步回调展示、置信度标注
 */

import { useRef, useEffect, useState, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { FlowTreeNode, AsyncCallback } from '../../types'
import { toFtraceFormat, toTreeFormat, ExportFormat, exportFlowTrees, getExportExtension } from '../../utils/flowFormatters'
import './FlowTextView.css'

type TextViewMode = 'ftrace' | 'tree'

export interface TextLine {
  id: string
  cpu: string
  depth: string
  async: string | null
  function: string
  isKernel: boolean
  isUser: boolean
  isAsyncCallback: boolean
  asyncMechanism: string | null
  confidence: string
  isError: boolean
  lineNumber: number
}

export interface FlowTextViewProps {
  flowTrees: FlowTreeNode[]
  onNodeClick?: (functionName: string) => void
  selectedFunction?: string
  asyncCallbacks?: AsyncCallback[]
}

export function FlowTextView({ flowTrees, onNodeClick, selectedFunction, asyncCallbacks = [] }: FlowTextViewProps) {
  const [viewMode, setViewMode] = useState<TextViewMode>('ftrace')
  const [content, setContent] = useState('')
  const [showHelp, setShowHelp] = useState(false)
  const [selectedLineId, setSelectedLineId] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)
  const preRef = useRef<HTMLDivElement>(null)

  // 帮助/图例内容
  const LEGEND_CONTENT = `
## ftrace 格式说明

### 信息列标记

| 标记 | 含义 | 说明 |
|------|------|------|
| \`:45\` | 行号 | 用户定义函数所在行，可点击跳转 |
| \`[K]\` | 内核 API | 如 kmalloc, printk 等内核函数 |
| \`[E]\` | 外部函数 | 定义在其他文件的函数 |

### 异步机制标记

| 标记 | 机制 | 执行上下文 |
|------|------|------------|
| \`[WQ]\` | WorkQueue | Process (可睡眠) |
| \`[TM]\` | Timer | SoftIRQ (不可睡眠) |
| \`[IRQ]\` | 中断处理 | HardIRQ (不可睡眠) |
| \`[TI]\` | Threaded IRQ | Process (可睡眠) |
| \`[TL]\` | Tasklet | SoftIRQ (不可睡眠) |

### 格式示例

\`\`\`
 0)   :45       |  storage_probe() {
 0)   :120      |    storage_inquiry() {
 0)   [K]       |      kmalloc();
 0)   [WQ]      |      /* WorkQueue */ handler() {
 0)   :89       |        do_work();
 0)              |      }
 0)              |    }
 0)              |  }
\`\`\`
`.trim()

  // 解析 ftrace 格式内容为可交互的行
  const parseFtraceLines = useMemo((): TextLine[] => {
    if (!content) return []

    const lines: TextLine[] = []
    const contentLines = content.split('\n')

    contentLines.forEach((line, index) => {
      // 匹配 ftrace 格式: " 0)   :45       |  func() {"
      const match = line.match(/^(\s*)(\d+)\)?\s*(?::(\d+))?\s*\|\s*(?:(\[K\]|\[E\])?\s*)?(.*)$/)

      // 解析函数内容（用于后续处理）
      const funcPart = match ? match[5] : ''
      const trimmedFunc = funcPart.trim()

      if (match) {
        const [, prefix, cpu, lineNum, badge] = match

        // 解析函数名和异步标记
        let functionName = trimmedFunc
        let asyncMarker: string | null = null
        let asyncMechanism: string | null = null
        let confidence = ''
        let isError = false

        // 检查异步回调标记
        const asyncMatch = trimmedFunc.match(/\[(WQ|TM|IRQ|TI|TL|SI|NF|RCU|KT)\]/)
        if (asyncMatch) {
          asyncMarker = asyncMatch[0]
          asyncMechanism = asyncMatch[1]
        }

        // 提取函数名（去掉末尾的 {）
        const funcNameMatch = trimmedFunc.match(/(\w+)\s*\(?\s*\{?\s*$/)
        if (funcNameMatch) {
          functionName = funcNameMatch[1]
        }

        // 检查置信度标记
        const confMatch = trimmedFunc.match(/\[(Certain|Possible)\]/)
        if (confMatch) {
          confidence = confMatch[1].toLowerCase()
        }

        // 检查错误路径
        if (trimmedFunc.includes('[!]') || trimmedFunc.includes('[ERROR]')) {
          isError = true
        }

        // 计算缩进深度
        const depth = (prefix.length / 2) + (asyncMarker ? 1 : 0)

        lines.push({
          id: `line-${index}`,
          cpu: cpu || '0',
          depth: ' '.repeat(depth * 2) + (asyncMarker ? 'async ' : '') + '='.repeat(Math.min(depth, 8)),
          async: asyncMarker,
          function: functionName,
          isKernel: badge === '[K]',
          isUser: badge === undefined && functionName.length > 0,
          isAsyncCallback: !!asyncMarker,
          asyncMechanism,
          confidence,
          isError,
          lineNumber: lineNum ? parseInt(lineNum, 10) : 0
        })
      } else if (trimmedFunc.startsWith('//') || trimmedFunc.startsWith('/*')) {
        // 注释行
        lines.push({
          id: `line-${index}`,
          cpu: '',
          depth: '',
          async: null,
          function: trimmedFunc,
          isKernel: false,
          isUser: false,
          isAsyncCallback: false,
          asyncMechanism: null,
          confidence: '',
          isError: false,
          lineNumber: 0
        })
      }
    })

    return lines
  }, [content])

  // 更新内容
  useEffect(() => {
    if (flowTrees.length === 0) {
      setContent('// 暂无执行流数据\n// 请选择文件并点击刷新按钮')
      return
    }

    const text = viewMode === 'ftrace'
      ? toFtraceFormat(flowTrees)
      : toTreeFormat(flowTrees)
    setContent(text)
  }, [flowTrees, viewMode])

  // 高亮选中的函数
  useEffect(() => {
    if (!preRef.current || !selectedFunction) return

    // 移除旧的高亮
    preRef.current.querySelectorAll('.selected').forEach(el => {
      el.classList.remove('selected')
    })

    // 高亮匹配的函数名
    const funcElements = preRef.current.querySelectorAll('.func-name')
    funcElements.forEach(el => {
      if ((el as HTMLElement).dataset.func === selectedFunction) {
        el.classList.add('selected')
      }
    })
  }, [selectedFunction, content])

  // 处理行点击
  const handleLineClick = (line: TextLine) => {
    setSelectedLineId(line.id)
    if (line.function && onNodeClick && !line.function.startsWith('//') && !line.function.startsWith('/*')) {
      onNodeClick(line.function)
    }
  }

  // 处理点击
  const handleClick = (e: React.MouseEvent) => {
    const target = e.target as HTMLElement
    if (target.classList.contains('func-name') && onNodeClick) {
      const funcName = target.dataset.func
      if (funcName) {
        onNodeClick(funcName)
      }
    }
  }

  // 复制到剪贴板
  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(content)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (err) {
      console.error('复制失败:', err)
    }
  }

  // 导出文件
  const handleExport = async (format: ExportFormat) => {
    try {
      const ext = getExportExtension(format)
      const exportContent = exportFlowTrees(flowTrees, format, { title: '执行流分析' })

      const filePath = await save({
        defaultPath: `flow-analysis${ext}`,
        filters: [
          { name: format.toUpperCase(), extensions: [ext.slice(1)] }
        ]
      })

      if (filePath) {
        await invoke('export_flow_text', {
          path: filePath,
          content: exportContent
        })
      }
    } catch (err) {
      console.error('导出失败:', err)
    }
  }

  // 简单的 Markdown 渲染器
  const renderMarkdown = (md: string): string => {
    return md
      .replace(/^### (.+)$/gm, '<h3>$1</h3>')
      .replace(/^## (.+)$/gm, '<h2>$1</h2>')
      .replace(/```([^`]+)```/gs, '<pre><code>$1</code></pre>')
      .replace(/`([^`]+)`/g, '<code>$1</code>')
      .replace(/^\|(.+)\|$/gm, (_, content) => {
        const cells = content.split('|').map((c: string) => c.trim())
        const isHeader = cells.every((c: string) => c.match(/^-+$/))
        if (isHeader) return ''
        const tag = 'td'
        return `<tr>${cells.map((c: string) => `<${tag}>${c}</${tag}>`).join('')}</tr>`
      })
      .replace(/(<tr>.*<\/tr>\n?)+/g, '<table>$&</table>')
      .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
      .replace(/←/g, '&larr;')
      .replace(/\n\n/g, '</p><p>')
      .replace(/^/, '<p>')
      .replace(/$/, '</p>')
  }

  // 获取异步机制显示标签
  const getAsyncLabel = (mechanism: string | null): string => {
    const labels: Record<string, string> = {
      'WQ': 'WorkQueue',
      'TM': 'Timer',
      'IRQ': 'IRQ',
      'TI': 'Threaded',
      'TL': 'Tasklet',
      'SI': 'SoftIRQ',
      'KT': 'KThread',
      'NF': 'Notifier',
      'RCU': 'RCU'
    }
    return mechanism && labels[mechanism] ? labels[mechanism] : ''
  }

  // 获取置信度标签
  const getConfidenceLabel = (confidence: string): { text: string; class: string } => {
    const labels: Record<string, { text: string; class: string }> = {
      'certain': { text: '确定', class: 'confidence-certain' },
      'possible': { text: '可能', class: 'confidence-possible' }
    }
    return confidence && labels[confidence] ? labels[confidence] : { text: '', class: '' }
  }

  // 渲染 ftrace 风格行
  const renderFtraceLines = () => {
    if (parseFtraceLines.length === 0) {
      return renderContent()
    }

    return (
      <div className="ftrace-lines" ref={preRef}>
        {/* 表头 */}
        <div className="ftrace-header">
          <span className="cpu-col">CPU</span>
          <span className="depth-col">Function</span>
          <span className="function-col"></span>
        </div>

        {/* 内容行 */}
        {parseFtraceLines.map((line) => (
          <div
            key={line.id}
            className={`ftrace-line ${selectedLineId === line.id ? 'selected' : ''} ${line.isKernel ? 'kernel' : 'user'} ${line.isError ? 'error' : ''}`}
            onClick={() => handleLineClick(line)}
          >
            <span className="cpu-col">{line.cpu})</span>
            <span className="depth-col">
              <span className="depth-marker">{line.depth}</span>
              {line.async && (
                <span className={`async-indicator ${line.asyncMechanism?.toLowerCase()}`}>
                  {getAsyncLabel(line.asyncMechanism)}
                </span>
              )}
            </span>
            <span className="function-col">
              {line.isKernel && <span className="kernel-badge">内核</span>}
              {line.isUser && !line.isAsyncCallback && <span className="user-badge">用户</span>}
              <span className="function-name">{line.function}</span>
              {line.confidence && (
                <span className={`confidence-badge ${getConfidenceLabel(line.confidence).class}`}>
                  {getConfidenceLabel(line.confidence).text}
                </span>
              )}
              {line.lineNumber > 0 && (
                <span className="line-number">:{line.lineNumber}</span>
              )}
            </span>
          </div>
        ))}
      </div>
    )
  }

  // 渲染带语法高亮的内容（tree 模式）
  const renderContent = () => {
    if (!content) return null

    const highlighted = content
      .replace(/(\w+)\(\)/g, '<span class="func-name" data-func="$1">$1</span>()')
      .replace(/(\{|\})/g, '<span class="brace">$1</span>')
      .replace(/(\/\/.*$)/gm, '<span class="comment">$1</span>')
      .replace(/(\[.*?\])/g, '<span class="async-tag">$1</span>')
      .replace(/(⚙️|⏲️|⚡|🔄|🧵|📦|🔌|🚀|📢|🔒)/g, '<span class="icon">$1</span>')

    return (
      <pre
        className="flow-text-content"
        onClick={handleClick}
        dangerouslySetInnerHTML={{ __html: highlighted }}
      />
    )
  }

  // 渲染异步回调部分
  const renderAsyncCallbacks = () => {
    if (asyncCallbacks.length === 0) return null

    return (
      <div className="async-section">
        <h4>异步回调</h4>
        {asyncCallbacks.map((callback, index) => (
          <div key={index} className="async-callback-item">
            <div className="async-trigger">
              <span className={`async-type ${callback.mechanism.toLowerCase().replace(/[^a-z]/g, '-')}`}>
                {callback.mechanism}
              </span>
              <code>{callback.trigger_code}</code>
            </div>
            <span className="async-arrow">→</span>
            <div className="async-handler">
              <code>{callback.handler}()</code>
              <span className="async-context">[{callback.execution_context}]</span>
            </div>
          </div>
        ))}
      </div>
    )
  }

  return (
    <div className="flow-text-view">
      <div className="flow-text-toolbar">
        <div className="toolbar-left">
          <span className="view-label">文本视图</span>
          <div className="view-mode-toggle">
            <button
              className={viewMode === 'ftrace' ? 'active' : ''}
              onClick={() => setViewMode('ftrace')}
              title="ftrace 风格"
            >
              📝 ftrace
            </button>
            <button
              className={viewMode === 'tree' ? 'active' : ''}
              onClick={() => setViewMode('tree')}
              title="树形视图"
            >
              🌲 树形
            </button>
          </div>
        </div>

        <div className="toolbar-right">
          <button onClick={handleCopy} title="复制到剪贴板" className="toolbar-btn">
            {copied ? '✓ 已复制' : '📋 复制'}
          </button>
          <div className="export-dropdown">
            <button className="toolbar-btn export-btn">📥 导出 ▾</button>
            <div className="export-menu">
              <button onClick={() => handleExport('ftrace')}>
                📝 纯文本 (.txt)
              </button>
              <button onClick={() => handleExport('markdown')}>
                📄 Markdown (.md)
              </button>
              <button onClick={() => handleExport('json')}>
                🔧 JSON (.json)
              </button>
            </div>
          </div>
          <button onClick={() => setShowHelp(true)} title="查看帮助" className="toolbar-btn">
            ❓ 帮助
          </button>
        </div>
      </div>

      {/* 帮助弹窗 */}
      {showHelp && (
        <div className="help-overlay" onClick={() => setShowHelp(false)}>
          <div className="help-modal" onClick={e => e.stopPropagation()}>
            <div className="help-header">
              <h2>📖 格式说明</h2>
              <button className="help-close" onClick={() => setShowHelp(false)}>✕</button>
            </div>
            <div className="help-content">
              <div dangerouslySetInnerHTML={{ __html: renderMarkdown(LEGEND_CONTENT) }} />
            </div>
          </div>
        </div>
      )}

      <div className="flow-text-content">
        {viewMode === 'ftrace' ? renderFtraceLines() : renderContent()}
      </div>

      {renderAsyncCallbacks()}

      <div className="flow-text-footer">
        <span>{parseFtraceLines.length} 行</span>
        {asyncCallbacks.length > 0 && (
          <span>• {asyncCallbacks.length} 个异步回调</span>
        )}
        <span className="footer-hint">点击行可跳转代码</span>
      </div>
    </div>
  )
}

export default FlowTextView
