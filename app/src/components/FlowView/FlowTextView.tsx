/**
 * 文本格式执行流视图
 * 
 * 支持 ftrace、tree 等纯文本格式显示
 */

import { useRef, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { FlowTreeNode } from '../../types'
import { toFtraceFormat, toTreeFormat, ExportFormat, exportFlowTrees, getExportExtension } from '../../utils/flowFormatters'
import './FlowTextView.css'

type TextViewMode = 'ftrace' | 'tree'

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
| \`[DW]\` | Delayed Work | Process (可睡眠) |
| \`[TM]\` | Timer | SoftIRQ (不可睡眠) |
| \`[HR]\` | HRTimer | HardIRQ (不可睡眠) |
| \`[IRQ]\` | 中断处理 | HardIRQ (不可睡眠) |
| \`[TI]\` | Threaded IRQ | Process (可睡眠) |
| \`[TL]\` | Tasklet | SoftIRQ (不可睡眠) |
| \`[SI]\` | SoftIRQ | SoftIRQ (不可睡眠) |
| \`[KT]\` | KThread | Process (可睡眠) |
| \`[RCU]\` | RCU Callback | SoftIRQ (不可睡眠) |
| \`[NF]\` | Notifier | 取决于链类型 |

### 格式示例

\`\`\`
 0)   :45       |  storage_probe() {        ← 第45行定义
 0)   :120      |    storage_inquiry() {
 0)   [K]       |      kmalloc();           ← 内核API
 0)   [WQ]      |      /* WorkQueue */ handler() {  ← 异步
 0)   :89       |        do_work();
 0)              |      }
 0)              |    }
 0)              |  }
\`\`\`

### 快捷键

- **Ctrl+P**: 打开命令面板
- **Alt+←/→**: 后退/前进导航
- 点击函数名可跳转到代码
`.trim()

// 简单的 Markdown 渲染器
function renderMarkdown(md: string): string {
  return md
    // 标题
    .replace(/^### (.+)$/gm, '<h3>$1</h3>')
    .replace(/^## (.+)$/gm, '<h2>$1</h2>')
    // 代码块
    .replace(/```([^`]+)```/gs, '<pre><code>$1</code></pre>')
    // 行内代码
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    // 表格
    .replace(/^\|(.+)\|$/gm, (_, content) => {
      const cells = content.split('|').map((c: string) => c.trim())
      const isHeader = cells.every((c: string) => c.match(/^-+$/))
      if (isHeader) return '' // 跳过分隔行
      const tag = 'td'
      return `<tr>${cells.map((c: string) => `<${tag}>${c}</${tag}>`).join('')}</tr>`
    })
    .replace(/(<tr>.*<\/tr>\n?)+/g, '<table>$&</table>')
    // 粗体
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    // 箭头符号
    .replace(/←/g, '&larr;')
    // 段落
    .replace(/\n\n/g, '</p><p>')
    .replace(/^/, '<p>')
    .replace(/$/, '</p>')
}

interface FlowTextViewProps {
  flowTrees: FlowTreeNode[]
  onNodeClick?: (functionName: string) => void
  selectedFunction?: string
}

export function FlowTextView({ flowTrees, onNodeClick, selectedFunction }: FlowTextViewProps) {
  const [viewMode, setViewMode] = useState<TextViewMode>('ftrace')
  const [content, setContent] = useState('')
  const [showHelp, setShowHelp] = useState(false)
  const preRef = useRef<HTMLPreElement>(null)
  
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
    preRef.current.querySelectorAll('.highlight').forEach(el => {
      el.classList.remove('highlight')
    })
    
    // TODO: 实现更精确的高亮
  }, [selectedFunction, content])
  
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
      // TODO: 显示 toast 提示
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
        // 调用 Rust 后端写入文件
        await invoke('export_flow_text', { 
          path: filePath, 
          content: exportContent 
        })
      }
    } catch (err) {
      console.error('导出失败:', err)
    }
  }
  
  // 渲染带语法高亮的内容
  const renderContent = () => {
    if (!content) return null
    
    // 简单的语法高亮：函数名、括号等
    const highlighted = content
      .replace(/(\w+)\(\)/g, '<span class="func-name" data-func="$1">$1</span>()')
      .replace(/(\{|\})/g, '<span class="brace">$1</span>')
      .replace(/(\/\/.*$)/gm, '<span class="comment">$1</span>')
      .replace(/(\[.*?\])/g, '<span class="async-tag">$1</span>')
      .replace(/(⚙️|⏲️|⚡|🔄|🧵|📦|🔌|🚀|📢|🔒)/g, '<span class="icon">$1</span>')
    
    return (
      <pre 
        ref={preRef}
        className="flow-text-content"
        onClick={handleClick}
        dangerouslySetInnerHTML={{ __html: highlighted }}
      />
    )
  }
  
  return (
    <div className="flow-text-view">
      <div className="flow-text-toolbar">
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
        
        <div className="toolbar-actions">
          <button onClick={handleCopy} title="复制到剪贴板">
            📋 复制
          </button>
          <div className="export-dropdown">
            <button className="export-btn">📥 导出 ▾</button>
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
          <button onClick={() => setShowHelp(true)} title="查看帮助">
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
      
      <div className="flow-text-container">
        {renderContent()}
      </div>
    </div>
  )
}

export default FlowTextView

