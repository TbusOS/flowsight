/**
 * KnowledgeInfoPanel - 知识库信息面板
 * 
 * 展示函数/API 的知识库信息：
 * - 执行上下文
 * - 是否可睡眠
 * - 完整内核调用链
 * - 开发者注意事项
 */

import { useState, useEffect, useCallback, memo } from 'react'
import {
  Cpu,
  Clock,
  Zap,
  AlertTriangle,
  Check,
  ChevronDown,
  ChevronRight,
  Info,
  FileCode,
  ArrowRight,
  Shield,
  BookOpen,
  Link2,
  AlertCircle,
  RefreshCw,
} from 'lucide-react'
import { getKnowledgeInfo, type KnowledgeInfo, type CallChainNode } from '../../lib/tauri-api'
import './KnowledgeInfoPanel.css'

export interface KnowledgeInfoPanelProps {
  /** 要查询的符号名称 */
  symbol?: string
  /** 代码上下文，用于更准确的匹配 */
  codeContext?: string
  /** 类名 */
  className?: string
  /** 是否显示调用链 */
  showCallChain?: boolean
}

// 执行上下文配置
const contextConfig: Record<string, { 
  icon: React.ReactNode; 
  label: string; 
  color: string; 
  bgColor: string;
  description: string;
}> = {
  process: { 
    icon: <Cpu className="h-3.5 w-3.5" />, 
    label: '进程上下文', 
    color: 'text-emerald-400',
    bgColor: 'bg-emerald-500/10',
    description: '可睡眠，可被调度'
  },
  softirq: { 
    icon: <Clock className="h-3.5 w-3.5" />, 
    label: '软中断上下文', 
    color: 'text-amber-400',
    bgColor: 'bg-amber-500/10',
    description: '不可睡眠，可被硬中断抢占'
  },
  hardirq: { 
    icon: <Zap className="h-3.5 w-3.5" />, 
    label: '硬中断上下文', 
    color: 'text-red-400',
    bgColor: 'bg-red-500/10',
    description: '不可睡眠，不可被抢占'
  },
  user: { 
    icon: <Shield className="h-3.5 w-3.5" />, 
    label: '用户空间', 
    color: 'text-blue-400',
    bgColor: 'bg-blue-500/10',
    description: '用户态代码'
  },
  unknown: { 
    icon: <AlertTriangle className="h-3.5 w-3.5" />, 
    label: '未知上下文', 
    color: 'text-gray-400',
    bgColor: 'bg-gray-500/10',
    description: '需要根据调用路径确定'
  },
  any: { 
    icon: <Check className="h-3.5 w-3.5" />, 
    label: '任意上下文', 
    color: 'text-cyan-400',
    bgColor: 'bg-cyan-500/10',
    description: '可在任何上下文中调用'
  },
}

// 调用链节点组件
const CallChainNodeItem = memo(function CallChainNodeItem({
  node,
  index,
  isLast,
}: {
  node: CallChainNode
  index: number
  isLast: boolean
}) {
  const ctx = contextConfig[node.context.toLowerCase()] || contextConfig.unknown
  
  return (
    <div className="call-chain-node">
      <div className="call-chain-connector">
        <span className={`connector-dot ${node.is_user_entry ? 'user-entry' : ''}`} />
        {!isLast && <span className="connector-line" />}
      </div>
      <div className={`call-chain-content ${node.is_user_entry ? 'user-entry' : ''}`}>
        <div className="call-chain-header">
          <code className="function-name">{node.function}</code>
          {node.is_user_entry && (
            <span className="user-entry-badge">用户代码入口</span>
          )}
        </div>
        {node.description && (
          <p className="call-chain-description">{node.description}</p>
        )}
        <div className="call-chain-meta">
          <span className={`context-badge ${ctx.bgColor} ${ctx.color}`}>
            {ctx.icon}
            <span>{ctx.label}</span>
          </span>
          {node.file && (
            <span className="file-badge">
              <FileCode className="h-3 w-3" />
              <span>{node.file}</span>
            </span>
          )}
        </div>
      </div>
    </div>
  )
})

// 可折叠的调用链组件
const CallChainSection = memo(function CallChainSection({
  callChain,
  title = '内核调用链',
}: {
  callChain: CallChainNode[]
  title?: string
}) {
  const [isExpanded, setIsExpanded] = useState(true)
  
  return (
    <div className="call-chain-section">
      <button 
        className="section-header"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <Link2 className="h-3.5 w-3.5 text-[var(--text-muted)]" />
        <span className="section-title">{title}</span>
        <span className="node-count">{callChain.length} 步</span>
        {isExpanded ? (
          <ChevronDown className="h-3.5 w-3.5 text-[var(--text-muted)]" />
        ) : (
          <ChevronRight className="h-3.5 w-3.5 text-[var(--text-muted)]" />
        )}
      </button>
      
      {isExpanded && (
        <div className="call-chain-list">
          {callChain.map((node, index) => (
            <CallChainNodeItem
              key={`${node.function}-${index}`}
              node={node}
              index={index}
              isLast={index === callChain.length - 1}
            />
          ))}
        </div>
      )}
    </div>
  )
})

export const KnowledgeInfoPanel = memo(function KnowledgeInfoPanel({
  symbol,
  codeContext,
  className = '',
  showCallChain = true,
}: KnowledgeInfoPanelProps) {
  const [info, setInfo] = useState<KnowledgeInfo | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // 加载知识库信息
  const loadInfo = useCallback(async () => {
    if (!symbol) {
      setInfo(null)
      return
    }

    setLoading(true)
    setError(null)

    try {
      const result = await getKnowledgeInfo(symbol, codeContext)
      setInfo(result)
    } catch (err) {
      setError(`加载失败: ${err}`)
      setInfo(null)
    } finally {
      setLoading(false)
    }
  }, [symbol, codeContext])

  useEffect(() => {
    loadInfo()
  }, [loadInfo])

  // 空状态
  if (!symbol) {
    return (
      <div className={`knowledge-info-panel empty ${className}`}>
        <div className="empty-state">
          <BookOpen className="h-8 w-8 text-[var(--text-muted)]" />
          <p>选择一个函数查看知识库信息</p>
        </div>
      </div>
    )
  }

  // 加载状态
  if (loading) {
    return (
      <div className={`knowledge-info-panel loading ${className}`}>
        <div className="loading-state">
          <RefreshCw className="h-5 w-5 animate-spin text-[var(--accent)]" />
          <p>加载知识库...</p>
        </div>
      </div>
    )
  }

  // 错误状态
  if (error) {
    return (
      <div className={`knowledge-info-panel error ${className}`}>
        <div className="error-state">
          <AlertCircle className="h-5 w-5 text-red-400" />
          <p>{error}</p>
          <button onClick={loadInfo} className="retry-btn">
            <RefreshCw className="h-3.5 w-3.5" />
            重试
          </button>
        </div>
      </div>
    )
  }

  // 无数据状态
  if (!info) {
    return (
      <div className={`knowledge-info-panel no-data ${className}`}>
        <div className="no-data-state">
          <Info className="h-5 w-5 text-[var(--text-muted)]" />
          <p>暂无 <code>{symbol}</code> 的知识库信息</p>
          <span className="hint">该函数可能是用户定义的或未被记录</span>
        </div>
      </div>
    )
  }

  const ctx = contextConfig[info.context || 'unknown'] || contextConfig.unknown

  return (
    <div className={`knowledge-info-panel ${className}`}>
      {/* 头部 */}
      <div className="panel-header">
        <div className="header-title">
          <BookOpen className="h-4 w-4 text-[var(--accent)]" />
          <span>知识库</span>
        </div>
        {info.framework && (
          <span className="framework-badge">
            {info.framework}
            {info.callback_type && ` / ${info.callback_type}`}
          </span>
        )}
      </div>

      {/* 基本信息 */}
      <div className="info-section">
        <h3 className="symbol-name">{info.name}</h3>
        {info.description && (
          <p className="description">{info.description}</p>
        )}
        {info.signature && (
          <code className="signature">{info.signature}</code>
        )}
      </div>

      {/* 执行上下文 */}
      <div className="context-section">
        <div className={`context-card ${ctx.bgColor}`}>
          <div className="context-header">
            <span className={`context-icon ${ctx.color}`}>{ctx.icon}</span>
            <span className={`context-label ${ctx.color}`}>{ctx.label}</span>
          </div>
          <p className="context-description">{ctx.description}</p>
          
          {/* 可睡眠指示器 */}
          <div className="sleep-indicator">
            {info.can_sleep ? (
              <>
                <Check className="h-3.5 w-3.5 text-emerald-400" />
                <span className="text-emerald-400">可睡眠</span>
              </>
            ) : (
              <>
                <AlertTriangle className="h-3.5 w-3.5 text-red-400" />
                <span className="text-red-400">不可睡眠</span>
              </>
            )}
          </div>
        </div>
      </div>

      {/* 触发条件 */}
      {info.trigger && (
        <div className="trigger-section">
          <div className="section-label">
            <Zap className="h-3.5 w-3.5" />
            触发条件
          </div>
          <p className="trigger-content">{info.trigger}</p>
        </div>
      )}

      {/* 调用链 */}
      {showCallChain && info.call_chain && info.call_chain.length > 0 && (
        <CallChainSection callChain={info.call_chain} />
      )}

      {/* 开发者注意事项 */}
      {info.notes && info.notes.length > 0 && (
        <div className="notes-section">
          <div className="section-label">
            <AlertTriangle className="h-3.5 w-3.5 text-amber-400" />
            注意事项
          </div>
          <ul className="notes-list">
            {info.notes.map((note, i) => (
              <li key={i}>{note}</li>
            ))}
          </ul>
        </div>
      )}

      {/* 示例代码 */}
      {info.examples && info.examples.length > 0 && (
        <div className="examples-section">
          <div className="section-label">
            <FileCode className="h-3.5 w-3.5" />
            示例
          </div>
          {info.examples.map((example, i) => (
            <pre key={i} className="example-code">{example}</pre>
          ))}
        </div>
      )}
    </div>
  )
})

export default KnowledgeInfoPanel
