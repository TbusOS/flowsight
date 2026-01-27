/**
 * FlowNode - 可折叠的执行流节点
 *
 * Features:
 * - 现代化视觉效果
 * - 丰富的交互反馈
 * - 键盘导航支持
 * - 无障碍访问
 */

import { memo, useState, useCallback } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import type { ConfidenceLevel, CallConfidence } from '../../types'
import {
  Check,
  HelpCircle,
  AlertTriangle,
  RefreshCw,
  Clock,
  Zap,
  FileText,
  Cpu,
  User,
  Settings,
  ArrowRight,
  ChevronDown,
  ChevronRight,
} from 'lucide-react'
import './FlowNode.css'

interface FlowNodeData {
  name: string
  icon: string
  nodeClass: string
  asyncLabel?: string | null
  isExpanded: boolean
  hasChildren: boolean
  childCount: number
  isSelected: boolean
  onToggle: () => void
  onContextMenu?: (e: React.MouseEvent) => void
  // 详细信息用于 tooltip
  file?: string
  line?: number
  nodeType?: string
  // 置信度信息
  confidence?: CallConfidence
}

// 置信度对应的图标和颜色
const confidenceInfo: Record<ConfidenceLevel, { icon: React.ReactNode; label: string; colorVar: string }> = {
  'Certain': { icon: <Check className="w-3 h-3" strokeWidth={2.5} />, label: '确定', colorVar: 'var(--success)' },
  'Possible': { icon: <HelpCircle className="w-3 h-3" strokeWidth={2.5} />, label: '可能', colorVar: 'var(--warning)' },
  'Unknown': { icon: <AlertTriangle className="w-3 h-3" strokeWidth={2.5} />, label: '未知', colorVar: 'var(--error)' },
}

// 异步机制信息
const asyncInfoMap: Record<string, { icon: React.ReactNode; label: string; description: string }> = {
  'WorkQueue': { icon: <RefreshCw className="w-3.5 h-3.5" strokeWidth={2} />, label: 'WorkQueue', description: '工作队列 (进程上下文，可睡眠)' },
  'Timer': { icon: <Clock className="w-3.5 h-3.5" strokeWidth={2} />, label: 'Timer', description: '定时器 (软中断上下文，不可睡眠)' },
  'IRQ': { icon: <Zap className="w-3.5 h-3.5" strokeWidth={2} />, label: 'IRQ', description: '硬中断 (中断上下文，不可睡眠)' },
  'Tasklet': { icon: <FileText className="w-3.5 h-3.5" strokeWidth={2} />, label: 'Tasklet', description: 'Tasklet (软中断上下文)' },
  'KThread': { icon: <Cpu className="w-3.5 h-3.5" strokeWidth={2} />, label: 'KThread', description: '内核线程 (进程上下文，可睡眠)' },
  'Async': { icon: <Clock className="w-3.5 h-3.5" strokeWidth={2} />, label: 'Async', description: '异步调用' },
}

// 节点类型标签
const nodeTypeLabels: Record<string, { icon: React.ReactNode; label: string }> = {
  'user': { icon: <User className="w-3.5 h-3.5" strokeWidth={2} />, label: '用户定义函数' },
  'kernel-api': { icon: <Settings className="w-3.5 h-3.5" strokeWidth={2} />, label: '内核 API' },
  'external': { icon: <ArrowRight className="w-3.5 h-3.5" strokeWidth={2} />, label: '外部函数' },
  'callback': { icon: <Zap className="w-3.5 h-3.5" strokeWidth={2} />, label: '回调函数' },
  'async-callback': { icon: <Clock className="w-3.5 h-3.5" strokeWidth={2} />, label: '异步回调' },
}

export const FlowNodeComponent = memo(({ data }: NodeProps) => {
  const nodeData = data as unknown as FlowNodeData
  const {
    name,
    icon,
    nodeClass,
    asyncLabel,
    isExpanded,
    hasChildren,
    childCount,
    isSelected,
    onToggle,
    onContextMenu,
    file,
    line,
    nodeType,
    confidence,
  } = nodeData

  // 点击波纹效果状态
  const [ripple, setRipple] = useState<{ x: number; y: number; id: number } | null>(null)
  const [rippleId, setRippleId] = useState(0)

  const handleToggleClick = useCallback((e: React.MouseEvent) => {
    e.stopPropagation()
    onToggle()
  }, [onToggle])

  // 处理节点点击 - 添加波纹效果
  const handleNodeClick = useCallback((e: React.MouseEvent) => {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top
    const newRippleId = rippleId + 1
    setRippleId(newRippleId)
    setRipple({ x, y, id: newRippleId })

    // 移除波纹效果
    setTimeout(() => setRipple(null), 400)
  }, [rippleId])

  // 构建详细 tooltip
  const buildTooltip = useCallback(() => {
    const parts = [`📌 ${name}()`]
    if (nodeType) {
      const typeInfo = nodeTypeLabels[nodeType]
      if (typeInfo) {
        parts.push(`${typeInfo.icon} ${typeInfo.label}`)
      }
    }
    // 异步机制信息
    if (asyncLabel) {
      const asyncInfo = asyncInfoMap[asyncLabel]
      if (asyncInfo) {
        parts.push(`${asyncInfo.icon} ${asyncInfo.description}`)
      }
    }
    // 置信度信息
    if (confidence) {
      const info = confidenceInfo[confidence.level]
      parts.push(`${info.icon} 置信度: ${info.label}`)
      if (confidence.reason) {
        parts.push(`  └ ${confidence.reason}`)
      }
    }
    if (file) {
      const fileName = file.split('/').pop()
      parts.push(`📄 ${fileName}`)
    }
    if (line !== undefined) {
      parts.push(`📍 第 ${line} 行`)
    }
    if (hasChildren) {
      parts.push(`📊 调用 ${childCount} 个函数`)
    }
    return parts.join('\n')
  }, [name, nodeType, asyncLabel, confidence, file, line, hasChildren, childCount])

  // 获取置信度样式类
  const getConfidenceClass = useCallback(() => {
    if (!confidence) return ''
    return `confidence-${confidence.level.toLowerCase()}`
  }, [confidence])

  // 构建 ARIA 标签
  const getAriaLabel = useCallback(() => {
    const parts = [name]
    if (asyncLabel) parts.push(`异步: ${asyncLabel}`)
    if (confidence) parts.push(`置信度: ${confidenceInfo[confidence.level].label}`)
    if (hasChildren) parts.push(`${childCount} 个子节点`)
    return parts.join(', ')
  }, [name, asyncLabel, confidence, hasChildren, childCount])

  return (
    <div
      className={`flow-node node-${nodeClass} ${isSelected ? 'selected' : ''} ${getConfidenceClass()}`}
      onContextMenu={onContextMenu}
      onClick={handleNodeClick}
      title={buildTooltip()}
      role="button"
      tabIndex={0}
      aria-label={getAriaLabel()}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault()
          onToggle()
        }
      }}
    >
      {/* 点击波纹效果 */}
      {ripple && (
        <span
          className="node-ripple"
          style={{ left: ripple.x, top: ripple.y }}
          key={ripple.id}
        />
      )}

      <Handle type="target" position={Position.Left} />

      {/* 置信度指示器 */}
      {confidence && confidence.level !== 'Certain' && (
        <div
          className={`node-confidence-badge confidence-${confidence.level.toLowerCase()}`}
          style={{ color: confidenceInfo[confidence.level].colorVar }}
          title={`${confidenceInfo[confidence.level].label}: ${confidence.reason}`}
          role="status"
          aria-label={`置信度: ${confidenceInfo[confidence.level].label}`}
        >
          {confidenceInfo[confidence.level].icon}
        </div>
      )}

      {/* 异步标签 - 根据类型显示不同颜色 */}
      {asyncLabel && (
        <div
          className={`node-async-badge async-${asyncLabel.toLowerCase()}`}
          data-async-type={asyncLabel}
          role="status"
          aria-label={`异步机制: ${asyncLabel}`}
        >
          <span className="async-badge-icon">{asyncInfoMap[asyncLabel]?.icon}</span>
          {asyncLabel}
        </div>
      )}

      <div className="node-main">
        {/* 展开/收起按钮 */}
        {hasChildren && (
          <button
            className={`node-toggle ${isExpanded ? 'expanded' : ''}`}
            onClick={handleToggleClick}
            title={isExpanded ? '收起' : `展开 (${childCount})`}
            aria-label={isExpanded ? '收起子节点' : `展开 ${childCount} 个子节点`}
          >
            {isExpanded ? (
              <ChevronDown className="w-3.5 h-3.5" strokeWidth={2} />
            ) : (
              <ChevronRight className="w-3.5 h-3.5" strokeWidth={2} />
            )}
          </button>
        )}

        {/* 图标 */}
        <span className="node-icon" aria-hidden="true">{icon}</span>

        {/* 函数名 */}
        <span className="node-name">{name}</span>

        {/* 子节点数量 */}
        {hasChildren && !isExpanded && (
          <span className="node-count" title={`${childCount} 个子节点`}>
            +{childCount}
          </span>
        )}
      </div>

      <Handle type="source" position={Position.Right} />
    </div>
  )
})

FlowNodeComponent.displayName = 'FlowNodeComponent'
