/**
 * FlowExportPanel - 执行流格式化导出面板
 *
 * 使用 FlowSight 统一的 UI 风格
 */

import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import {
  X,
  Copy,
  Check,
  Download,
  ExternalLink,
  Code2,
  Table2,
  TreeDeciduous,
  Braces,
  LayoutList,
  Loader2,
  Zap,
  GitBranch,
  Clock,
  AlertCircle,
} from 'lucide-react'
import { cn } from '../../lib/utils'

// 格式化选项
interface FormatOptions {
  format: 'mermaid' | 'markdown' | 'ascii' | 'json'
  include_kernel_internal?: boolean
  max_depth?: number
}

// 格式化结果
interface FormattedFlow {
  format: string
  content: string
  entry_function: string
  summary: string
}

// 展示数据
interface DisplayFlowData {
  entry_function: string
  summary: string
  mermaid_diagram: string
  nodes: DisplayNode[]
  async_patterns: AsyncPattern[]
  stats: FlowStats
}

interface DisplayNode {
  id: string
  name: string
  display_name: string
  node_type: string
  context: string | null
  can_sleep: boolean | null
  description: string | null
  depth: number
  children_count: number
}

interface AsyncPattern {
  mechanism: string
  trigger: string
  handler: string
  description: string
}

interface FlowStats {
  total_nodes: number
  direct_calls: number
  indirect_calls: number
  async_calls: number
}

export interface FlowExportPanelProps {
  filePath: string | null
  entryFunction: string | null
  onClose?: () => void
}

type TabId = 'summary' | 'mermaid' | 'markdown' | 'ascii' | 'json'

const TABS: { id: TabId; label: string; icon: React.ReactNode }[] = [
  { id: 'summary', label: '概览', icon: <LayoutList className="h-3.5 w-3.5" /> },
  { id: 'mermaid', label: 'Mermaid', icon: <GitBranch className="h-3.5 w-3.5" /> },
  { id: 'markdown', label: 'Markdown', icon: <Table2 className="h-3.5 w-3.5" /> },
  { id: 'ascii', label: 'ASCII', icon: <TreeDeciduous className="h-3.5 w-3.5" /> },
  { id: 'json', label: 'JSON', icon: <Braces className="h-3.5 w-3.5" /> },
]

export function FlowExportPanel({ filePath, entryFunction, onClose }: FlowExportPanelProps) {
  const [activeTab, setActiveTab] = useState<TabId>('summary')
  const [content, setContent] = useState<string>('')
  const [displayData, setDisplayData] = useState<DisplayFlowData | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  // 获取格式化内容
  const fetchFormattedContent = useCallback(async (format: 'mermaid' | 'markdown' | 'ascii' | 'json') => {
    if (!filePath || !entryFunction) return

    setLoading(true)
    setError(null)
    try {
      const result = await invoke<FormattedFlow>('format_execution_flow', {
        filePath,
        entryFunction,
        options: { format } as FormatOptions,
      })
      setContent(result.content)
    } catch (err) {
      console.error('格式化失败:', err)
      setError(err instanceof Error ? err.message : String(err))
      // 显示 Mock 数据用于演示
      setContent(getMockContent(format, entryFunction))
    } finally {
      setLoading(false)
    }
  }, [filePath, entryFunction])

  // 获取展示数据
  const fetchDisplayData = useCallback(async () => {
    if (!filePath || !entryFunction) return

    setLoading(true)
    try {
      const data = await invoke<DisplayFlowData>('get_flow_display_data', {
        filePath,
        entryFunction,
      })
      setDisplayData(data)
    } catch (err) {
      console.error('获取展示数据失败:', err)
      // Mock 数据用于演示
      setDisplayData(getMockDisplayData(entryFunction))
    } finally {
      setLoading(false)
    }
  }, [filePath, entryFunction])

  // 切换 Tab 时加载数据
  useEffect(() => {
    // 如果没有文件或入口函数，直接显示 Mock 数据作为演示
    if (!filePath || !entryFunction) {
      console.log('[FlowExportPanel] 缺少参数，显示演示数据', { filePath, entryFunction })
      if (activeTab === 'summary') {
        setDisplayData(getMockDisplayData(entryFunction || 'demo_function'))
      } else {
        setContent(getMockContent(activeTab, entryFunction || 'demo_function'))
      }
      return
    }

    if (activeTab === 'summary') {
      fetchDisplayData()
    } else {
      fetchFormattedContent(activeTab)
    }
  }, [activeTab, filePath, entryFunction, fetchDisplayData, fetchFormattedContent])

  // 复制到剪贴板
  const handleCopy = async () => {
    const textToCopy = activeTab === 'summary' && displayData
      ? displayData.mermaid_diagram
      : content

    try {
      await navigator.clipboard.writeText(textToCopy)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (err) {
      console.error('复制失败:', err)
    }
  }

  // 导出文件
  const handleExport = async () => {
    const extensions: Record<TabId, string> = {
      summary: 'md',
      mermaid: 'mmd',
      markdown: 'md',
      ascii: 'txt',
      json: 'json',
    }

    try {
      const path = await save({
        defaultPath: `${entryFunction}-flow.${extensions[activeTab]}`,
        filters: [{ name: 'All Files', extensions: ['*'] }],
      })
      if (path) {
        const textToSave = activeTab === 'summary' && displayData
          ? displayData.mermaid_diagram
          : content
        await invoke('write_file', { path, content: textToSave })
      }
    } catch (err) {
      console.error('导出失败:', err)
    }
  }

  // 渲染概览
  const renderSummary = () => {
    if (!displayData) return null

    return (
      <div className="flex flex-col gap-4 p-4">
        {/* 统计卡片 */}
        <div className="grid grid-cols-4 gap-3">
          <StatCard
            icon={<Code2 className="h-4 w-4" />}
            label="总节点"
            value={displayData.stats.total_nodes}
            color="text-blue-400"
          />
          <StatCard
            icon={<GitBranch className="h-4 w-4" />}
            label="直接调用"
            value={displayData.stats.direct_calls}
            color="text-green-400"
          />
          <StatCard
            icon={<Zap className="h-4 w-4" />}
            label="异步调用"
            value={displayData.stats.async_calls}
            color="text-purple-400"
          />
          <StatCard
            icon={<Clock className="h-4 w-4" />}
            label="间接调用"
            value={displayData.stats.indirect_calls}
            color="text-amber-400"
          />
        </div>

        {/* 异步模式 */}
        {displayData.async_patterns.length > 0 && (
          <div className="rounded-lg border border-[var(--border-light)] bg-[var(--bg-tertiary)]/50 p-3">
            <h3 className="text-xs font-medium text-[var(--text-primary)] mb-2 flex items-center gap-2">
              <Zap className="h-3.5 w-3.5 text-purple-400" />
              异步模式
            </h3>
            <div className="space-y-2">
              {displayData.async_patterns.map((pattern, i) => (
                <div key={i} className="flex items-center gap-2 text-xs">
                  <span className="px-1.5 py-0.5 rounded bg-purple-500/20 text-purple-400 font-mono">
                    {pattern.mechanism}
                  </span>
                  <span className="text-[var(--text-muted)]">→</span>
                  <span className="text-[var(--text-primary)] font-mono">{pattern.handler}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 节点列表 */}
        <div className="flex-1 overflow-auto">
          <h3 className="text-xs font-medium text-[var(--text-primary)] mb-2">
            函数列表 ({displayData.nodes.length})
          </h3>
          <div className="space-y-1">
            {displayData.nodes.slice(0, 20).map((node) => (
              <div
                key={node.id}
                className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-[var(--bg-tertiary)] text-xs"
                style={{ paddingLeft: `${node.depth * 12 + 8}px` }}
              >
                <span className={cn(
                  "w-1.5 h-1.5 rounded-full",
                  node.node_type === 'entry' && "bg-blue-400",
                  node.node_type === 'async' && "bg-purple-400",
                  node.node_type === 'callback' && "bg-amber-400",
                  !['entry', 'async', 'callback'].includes(node.node_type) && "bg-[var(--text-muted)]"
                )} />
                <span className="font-mono text-[var(--text-primary)]">{node.display_name}</span>
                {node.context && (
                  <span className="px-1 py-0.5 rounded bg-[var(--bg-tertiary)] text-[var(--text-muted)] text-[10px]">
                    {node.context}
                  </span>
                )}
              </div>
            ))}
            {displayData.nodes.length > 20 && (
              <div className="text-xs text-[var(--text-muted)] px-2 py-1">
                ... 还有 {displayData.nodes.length - 20} 个节点
              </div>
            )}
          </div>
        </div>
      </div>
    )
  }

  // 渲染代码内容
  const renderContent = () => {
    if (loading) {
      return (
        <div className="flex items-center justify-center h-full">
          <Loader2 className="h-5 w-5 animate-spin text-[var(--accent)]" />
        </div>
      )
    }

    if (error) {
      return (
        <div className="flex flex-col items-center justify-center h-full gap-2">
          <AlertCircle className="h-5 w-5 text-amber-400" />
          <p className="text-xs text-[var(--text-muted)]">加载失败，显示示例数据</p>
        </div>
      )
    }

    if (activeTab === 'summary') {
      return renderSummary()
    }

    return (
      <div className="h-full flex flex-col">
        {/* Mermaid 预览链接 */}
        {activeTab === 'mermaid' && content && (
          <div className="flex items-center gap-2 px-4 py-2 border-b border-[var(--border-subtle)] bg-[var(--bg-tertiary)]/30">
            <a
              href={`https://mermaid.live/edit#pako:${btoa(content)}`}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1.5 text-xs text-[var(--accent)] hover:underline"
            >
              <ExternalLink className="h-3 w-3" />
              在 Mermaid Live 中打开
            </a>
          </div>
        )}

        {/* 代码块 */}
        <div className="flex-1 overflow-auto p-4">
          <pre className="font-mono text-xs text-[var(--text-primary)] leading-relaxed whitespace-pre-wrap">
            {content}
          </pre>
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full bg-[var(--bg-primary)] rounded-lg border border-[var(--border-light)]">
      {/* 头部 */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
        <div className="flex items-center gap-2">
          <Code2 className="h-4 w-4 text-[var(--accent)]" />
          <span className="text-sm font-medium text-[var(--text-primary)]">导出执行流</span>
          {entryFunction && (
            <span className="px-2 py-0.5 rounded bg-[var(--accent)]/10 text-[var(--accent)] text-xs font-mono">
              {entryFunction}()
            </span>
          )}
        </div>
        <div className="flex items-center gap-1">
          {/* 复制 */}
          <button
            onClick={handleCopy}
            className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
            title={copied ? '已复制' : '复制'}
          >
            {copied ? <Check className="h-4 w-4 text-green-400" /> : <Copy className="h-4 w-4" />}
          </button>
          {/* 导出 */}
          <button
            onClick={handleExport}
            className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
            title="导出文件"
          >
            <Download className="h-4 w-4" />
          </button>
          {/* 关闭 */}
          {onClose && (
            <button
              onClick={onClose}
              className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors ml-1"
              title="关闭"
            >
              <X className="h-4 w-4" />
            </button>
          )}
        </div>
      </div>

      {/* 标签栏 */}
      <div className="flex items-center gap-1 px-2 py-1.5 border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)]/50">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={cn(
              "flex items-center gap-1.5 px-3 py-1.5 rounded text-xs transition-colors",
              activeTab === tab.id
                ? "bg-[var(--accent)] text-white"
                : "text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]"
            )}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>

      {/* 内容区 */}
      <div className="flex-1 overflow-hidden">
        {renderContent()}
      </div>
    </div>
  )
}

// 统计卡片组件
function StatCard({
  icon,
  label,
  value,
  color,
}: {
  icon: React.ReactNode
  label: string
  value: number
  color: string
}) {
  return (
    <div className="flex flex-col gap-1 p-3 rounded-lg border border-[var(--border-light)] bg-[var(--bg-tertiary)]/50">
      <div className={cn("flex items-center gap-1.5", color)}>
        {icon}
        <span className="text-[10px] text-[var(--text-muted)]">{label}</span>
      </div>
      <span className="text-lg font-semibold text-[var(--text-primary)]">{value}</span>
    </div>
  )
}

// Mock 数据 - 用于演示
function getMockContent(format: string, entry: string): string {
  switch (format) {
    case 'mermaid':
      return `flowchart TD
    ${entry}["${entry}()"] --> child1["function_a()"]
    ${entry} --> child2["function_b()"]
    child1 --> leaf1["helper_1()"]
    child2 --> leaf2["helper_2()"]
    child2 -->|async| async1["worker_fn()"]
    
    style ${entry} fill:#3b82f6,color:#fff
    style async1 fill:#a855f7,color:#fff`
    case 'markdown':
      return `| 函数 | 类型 | 行号 | 描述 |
|------|------|------|------|
| ${entry} | 入口 | 42 | 主入口函数 |
| function_a | 同步 | 58 | 辅助函数A |
| function_b | 同步 | 73 | 辅助函数B |
| worker_fn | 异步 | 95 | WorkQueue 处理函数 |`
    case 'ascii':
      return `${entry}()
├── function_a()
│   └── helper_1()
└── function_b()
    ├── helper_2()
    └── [async] worker_fn()`
    case 'json':
      return JSON.stringify({
        entry_function: entry,
        nodes: [
          { name: entry, type: 'entry', line: 42 },
          { name: 'function_a', type: 'function', line: 58 },
          { name: 'worker_fn', type: 'async', line: 95 },
        ],
        stats: { total: 5, direct: 3, async: 1 },
      }, null, 2)
    default:
      return ''
  }
}

function getMockDisplayData(entry: string): DisplayFlowData {
  return {
    entry_function: entry,
    summary: `执行流分析: ${entry}()`,
    mermaid_diagram: getMockContent('mermaid', entry),
    nodes: [
      { id: '1', name: entry, display_name: `${entry}()`, node_type: 'entry', context: 'process', can_sleep: true, description: '入口函数', depth: 0, children_count: 2 },
      { id: '2', name: 'function_a', display_name: 'function_a()', node_type: 'function', context: null, can_sleep: true, description: null, depth: 1, children_count: 1 },
      { id: '3', name: 'function_b', display_name: 'function_b()', node_type: 'function', context: null, can_sleep: true, description: null, depth: 1, children_count: 2 },
      { id: '4', name: 'worker_fn', display_name: 'worker_fn()', node_type: 'async', context: 'workqueue', can_sleep: true, description: 'WorkQueue 处理', depth: 2, children_count: 0 },
    ],
    async_patterns: [
      { mechanism: 'WorkQueue', trigger: 'schedule_work', handler: 'worker_fn', description: '异步工作队列' },
    ],
    stats: {
      total_nodes: 5,
      direct_calls: 3,
      indirect_calls: 1,
      async_calls: 1,
    },
  }
}
