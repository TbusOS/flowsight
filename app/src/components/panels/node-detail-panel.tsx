"use client"

import * as React from "react"
import {
  FunctionSquare,
  ArrowRight,
  ArrowLeft,
  Hash,
  Box,
  FileCode,
  GitBranch,
  Zap,
  MapPin,
  Code,
  Copy,
  Check,
  ChevronDown,
  ChevronRight,
  ExternalLink,
  Clock,
  Cpu,
  Shield,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { Button } from "../ui/button"
import { useAtomValue, useSetAtom } from "jotai"
import { selectedNodeAtom, triggerJumpAtom, setEntryFunctionAtom, type SelectedNodeDetail } from "../../lib/atoms/layout-atoms"

interface NodeDetailPanelProps {
  className?: string
  /** 外部传入的详情数据，优先级高于 atom */
  detail?: SelectedNodeDetail | null
  /** 点击调用函数的回调 */
  onCallClick?: (funcName: string) => void
  /** 点击被调用函数的回调 */
  onCalledByClick?: (funcName: string) => void
  /** 跳转到源码位置的回调 */
  onLocationClick?: (filePath: string, line: number) => void
}

// 执行上下文信息
const contextInfo: Record<string, { icon: React.ReactNode; label: string; color: string; description: string }> = {
  process: { 
    icon: <Cpu className="h-3 w-3" />, 
    label: "进程上下文", 
    color: "text-emerald-400",
    description: "可睡眠，可被调度"
  },
  softirq: { 
    icon: <Clock className="h-3 w-3" />, 
    label: "软中断", 
    color: "text-amber-400",
    description: "不可睡眠，可被硬中断抢占"
  },
  hardirq: { 
    icon: <Zap className="h-3 w-3" />, 
    label: "硬中断", 
    color: "text-red-400",
    description: "不可睡眠，不可被抢占"
  },
  atomic: { 
    icon: <Shield className="h-3 w-3" />, 
    label: "原子上下文", 
    color: "text-purple-400",
    description: "不可睡眠，持有锁"
  },
  workqueue: { 
    icon: <Clock className="h-3 w-3" />, 
    label: "工作队列", 
    color: "text-blue-400",
    description: "进程上下文，可睡眠"
  },
}

// 可折叠列表组件
const CollapsibleList = React.memo(function CollapsibleList({
  title,
  icon,
  items,
  onItemClick,
  itemPrefix,
  maxHeight = "max-h-40",
  testId,
}: {
  title: string
  icon: React.ReactNode
  items: string[]
  onItemClick?: (item: string) => void
  itemPrefix?: React.ReactNode
  maxHeight?: string
  testId?: string
}) {
  const [isExpanded, setIsExpanded] = React.useState(true)
  const [copiedIndex, setCopiedIndex] = React.useState<number | null>(null)

  const handleCopy = React.useCallback(async (item: string, index: number, e: React.MouseEvent) => {
    e.stopPropagation()
    await navigator.clipboard.writeText(item)
    setCopiedIndex(index)
    setTimeout(() => setCopiedIndex(null), 1500)
  }, [])

  if (items.length === 0) return null

  return (
    <div data-testid={testId} className="border-b border-[var(--border-subtle)]">
      <button
        className="w-full px-3 py-2 flex items-center justify-between hover:bg-[var(--bg-hover)] transition-colors"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <h4 className="text-[10px] uppercase tracking-wide text-[var(--text-muted)] flex items-center gap-1.5">
          {icon}
          {title} ({items.length})
        </h4>
        {isExpanded ? (
          <ChevronDown className="h-3 w-3 text-[var(--text-muted)]" />
        ) : (
          <ChevronRight className="h-3 w-3 text-[var(--text-muted)]" />
        )}
      </button>
      
      {isExpanded && (
        <div className={cn("px-3 pb-2 space-y-0.5 overflow-y-auto", maxHeight)}>
          {items.map((item, i) => (
            <div
              key={i}
              data-testid={`${testId}-${i}`}
              className="group flex items-center gap-2 px-2 py-1.5 rounded-md hover:bg-[var(--bg-tertiary)] cursor-pointer text-xs transition-colors"
              onClick={() => onItemClick?.(item)}
            >
              {itemPrefix || <Code className="h-3 w-3 text-[var(--text-muted)] shrink-0" />}
              <span className="text-[var(--accent)] font-mono truncate flex-1">{item}</span>
              <button
                className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-[var(--bg-active)] transition-opacity"
                onClick={(e) => handleCopy(item, i, e)}
                title="复制函数名"
              >
                {copiedIndex === i ? (
                  <Check className="h-3 w-3 text-emerald-400" />
                ) : (
                  <Copy className="h-3 w-3 text-[var(--text-muted)]" />
                )}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  )
})

// 参数列表组件
const ParameterList = React.memo(function ParameterList({
  parameters,
}: {
  parameters: Array<{ name: string; type: string }>
}) {
  const [isExpanded, setIsExpanded] = React.useState(true)

  if (parameters.length === 0) return null

  return (
    <div data-testid="detail-parameters" className="border-b border-[var(--border-subtle)]">
      <button
        className="w-full px-3 py-2 flex items-center justify-between hover:bg-[var(--bg-hover)] transition-colors"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <h4 className="text-[10px] uppercase tracking-wide text-[var(--text-muted)] flex items-center gap-1.5">
          <Hash className="h-3 w-3" />
          参数 ({parameters.length})
        </h4>
        {isExpanded ? (
          <ChevronDown className="h-3 w-3 text-[var(--text-muted)]" />
        ) : (
          <ChevronRight className="h-3 w-3 text-[var(--text-muted)]" />
        )}
      </button>
      
      {isExpanded && (
        <div className="px-3 pb-2 space-y-1">
          {parameters.map((param, i) => (
            <div key={i} className="flex items-center gap-2 text-xs font-mono px-2 py-1 rounded hover:bg-[var(--bg-hover)]">
              <span className="text-[var(--text-secondary)]">{param.name}</span>
              <span className="text-[var(--text-muted)]">:</span>
              <span className="text-[var(--accent)] truncate">{param.type}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
})

export const NodeDetailPanel = React.memo(function NodeDetailPanel({ 
  className, 
  detail: externalDetail,
  onCallClick,
  onCalledByClick,
  onLocationClick,
}: NodeDetailPanelProps) {
  // 从 Jotai atom 获取选中节点，外部传入优先
  const atomDetail = useAtomValue(selectedNodeAtom)
  const detail = externalDetail ?? atomDetail
  const triggerJump = useSetAtom(triggerJumpAtom)
  const setEntryFunction = useSetAtom(setEntryFunctionAtom)
  const [copied, setCopied] = React.useState(false)

  // 处理点击位置跳转
  const handleLocationClick = React.useCallback((filePath: string, line: number) => {
    console.log('[NodeDetailPanel] Jump to:', filePath, line)
    if (onLocationClick) {
      onLocationClick(filePath, line)
    } else {
      triggerJump({ filePath, line })
    }
  }, [onLocationClick, triggerJump])

  // 复制函数签名
  const handleCopySignature = React.useCallback(async () => {
    if (!detail) return
    const params = detail.parameters?.map(p => `${p.type} ${p.name}`).join(', ') || ''
    const signature = `${detail.return_type || 'void'} ${detail.name}(${params})`
    await navigator.clipboard.writeText(signature)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }, [detail])

  // 以此函数为入口分析
  const handleAnalyzeAsEntry = React.useCallback(() => {
    if (detail) {
      setEntryFunction(detail.name)
    }
  }, [detail, setEntryFunction])

  if (!detail) {
    return (
      <div 
        data-testid="detail-panel"
        className={cn("flex flex-col h-full items-center justify-center p-4 text-center", className)}
      >
        <Box className="h-8 w-8 text-[var(--text-muted)] mb-2" />
        <p className="text-xs text-[var(--text-muted)]">选择一个节点查看详情</p>
        <p className="text-[10px] text-[var(--text-muted)] mt-1">点击执行流图中的节点</p>
      </div>
    )
  }

  // 获取文件名（从完整路径）
  const fileName = detail.file_path?.split('/').pop() || '未知文件'
  
  // 解析执行上下文
  const context = detail.callback_context?.toLowerCase() || ''
  const execContext = contextInfo[context] || (detail.is_callback ? contextInfo.workqueue : null)

  return (
    <div 
      data-testid="detail-panel"
      className={cn("flex flex-col h-full overflow-hidden", className)}
    >
      {/* Header - 函数名称和基本信息 */}
      <div className="px-3 py-2 border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
        <div className="flex items-center gap-2 mb-2">
          {detail.is_callback ? (
            <Zap className="h-4 w-4 text-amber-400 shrink-0" />
          ) : (
            <FunctionSquare className="h-4 w-4 text-[var(--accent)] shrink-0" />
          )}
          <span 
            data-testid="detail-function-name"
            className="text-sm font-medium text-[var(--text-primary)] truncate flex-1"
            title={detail.name}
          >
            {detail.name}
          </span>
          <button
            onClick={handleCopySignature}
            className="p-1 rounded hover:bg-[var(--bg-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
            title="复制函数签名"
          >
            {copied ? (
              <Check className="h-3.5 w-3.5 text-emerald-400" />
            ) : (
              <Copy className="h-3.5 w-3.5" />
            )}
          </button>
        </div>
        
        {/* 标签行 */}
        <div className="flex items-center gap-1.5 flex-wrap mb-2">
          {detail.is_callback && (
            <span className="px-1.5 py-0.5 text-[10px] rounded bg-amber-500/20 text-amber-300">
              回调
            </span>
          )}
          {detail.return_type && detail.return_type !== 'void' && (
            <span className="px-1.5 py-0.5 text-[10px] rounded bg-[var(--accent)]/20 text-[var(--accent)] font-mono">
              → {detail.return_type}
            </span>
          )}
          {detail.node_type && (
            <span className="px-1.5 py-0.5 text-[10px] rounded bg-[var(--bg-tertiary)] text-[var(--text-muted)]">
              {detail.node_type}
            </span>
          )}
        </div>

        {/* 文件位置 - 点击跳转到源码 */}
        <div 
          data-testid="detail-file-path"
          className="flex items-center gap-1.5 text-[10px] text-[var(--text-muted)] cursor-pointer hover:text-[var(--accent)] transition-colors group"
          onClick={() => detail.file_path && handleLocationClick(detail.file_path, detail.line)}
          title="点击跳转到源码位置"
        >
          <MapPin className="h-3 w-3 shrink-0" />
          <span className="truncate group-hover:underline">{fileName}:{detail.line}</span>
          <ExternalLink className="h-3 w-3 opacity-0 group-hover:opacity-100 transition-opacity" />
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {/* 执行上下文 */}
        {execContext && (
          <div className="px-3 py-2 border-b border-[var(--border-subtle)] bg-[var(--bg-tertiary)]/50">
            <div className={cn("flex items-center gap-2 text-xs", execContext.color)}>
              {execContext.icon}
              <span className="font-medium">{execContext.label}</span>
            </div>
            <p className="text-[10px] text-[var(--text-muted)] mt-1 ml-5">
              {execContext.description}
            </p>
          </div>
        )}

        {/* 描述 */}
        {detail.description && (
          <div className="px-3 py-2 border-b border-[var(--border-subtle)]">
            <p className="text-xs text-[var(--text-secondary)] leading-relaxed">{detail.description}</p>
          </div>
        )}

        {/* 参数列表 */}
        {detail.parameters && (
          <ParameterList parameters={detail.parameters} />
        )}

        {/* 调用的函数 */}
        {detail.calls && (
          <CollapsibleList
            title="调用"
            icon={<ArrowRight className="h-3 w-3" />}
            items={detail.calls}
            onItemClick={onCallClick}
            testId="detail-calls"
          />
        )}

        {/* 被调用 */}
        {detail.called_by && (
          <CollapsibleList
            title="被调用"
            icon={<ArrowLeft className="h-3 w-3" />}
            items={detail.called_by}
            onItemClick={onCalledByClick}
            itemPrefix={<ArrowLeft className="h-3 w-3 text-[var(--text-muted)] shrink-0" />}
            testId="detail-called-by"
          />
        )}

        {/* 空状态 */}
        {(!detail.calls || detail.calls.length === 0) && 
         (!detail.called_by || detail.called_by.length === 0) && 
         (!detail.parameters || detail.parameters.length === 0) && (
          <div className="px-3 py-6 text-center">
            <FileCode className="h-6 w-6 mx-auto mb-2 text-[var(--text-muted)]" />
            <p className="text-xs text-[var(--text-muted)]">暂无详细信息</p>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="px-3 py-2 border-t border-[var(--border-subtle)] flex gap-2 bg-[var(--bg-secondary)]">
        <Button 
          variant="secondary" 
          size="sm" 
          className="flex-1 text-xs h-7"
          onClick={handleAnalyzeAsEntry}
          title="以此函数为入口重新分析执行流"
        >
          <GitBranch className="h-3 w-3 mr-1" />
          分析调用链
        </Button>
        <Button 
          variant="ghost" 
          size="sm" 
          className="flex-1 text-xs h-7"
          onClick={() => detail.file_path && handleLocationClick(detail.file_path, detail.line)}
        >
          <FileCode className="h-3 w-3 mr-1" />
          查看源码
        </Button>
      </div>
    </div>
  )
})
