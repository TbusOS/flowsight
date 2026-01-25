/**
 * NodeDetailPanel - 节点详情面板
 *
 * 显示选中节点的详细信息：
 * - 节点类型、名称、位置
 * - 上下文信息
 * - LLVM IR 代码片段
 * - 折叠/展开和复制功能
 * - 快捷操作按钮
 */

import { useState, useCallback, useRef } from 'react'
import type { FlowTreeNode, FlowNodeType, ConfidenceLevel } from '../../types'
import { Icons } from '../Icons/Icons'
import './NodeDetailPanel.css'

// 置信度信息
const confidenceInfo: Record<ConfidenceLevel, { icon: React.ReactNode; label: string; color: string }> = {
  'Certain': { icon: <Icons.Check size={12} />, label: '确定', color: '#34d399' },
  'Possible': { icon: <Icons.Alert size={12} />, label: '可能', color: '#fbbf24' },
  'Unknown': { icon: <Icons.Info size={12} />, label: '未知', color: '#f87171' },
}

// 节点类型标签映射
const nodeTypeLabels: Record<string, { icon: React.ReactNode; label: string; color: string }> = {
  'Function': { icon: <Icons.Function size={13} />, label: '函数', color: '#60a5fa' },
  'EntryPoint': { icon: <Icons.EntryPoint size={14} />, label: '入口点', color: '#34d399' },
  'KernelApi': { icon: <Icons.Api size={13} />, label: '内核 API', color: '#fbbf24' },
  'External': { icon: <Icons.ExternalLink size={12} />, label: '外部函数', color: '#9ca3af' },
  'AsyncCallback': { icon: <Icons.Zap size={13} />, label: '异步回调', color: '#a855f7' },
}

// 异步机制标签映射
const asyncMechanismLabels: Record<string, { icon: React.ReactNode; label: string; color: string }> = {
  'WorkQueue': { icon: <Icons.Refresh size={12} />, label: '工作队列', color: '#fbbf24' },
  'Timer': { icon: <Icons.Zap size={12} />, label: '定时器', color: '#34d399' },
  'Irq': { icon: <Icons.Zap size={12} />, label: '硬中断', color: '#f87171' },
  'Tasklet': { icon: <Icons.Code size={12} />, label: 'Tasklet', color: '#a855f7' },
  'KThread': { icon: <Icons.Settings size={12} />, label: '内核线程', color: '#3b82f6' },
  'Softirq': { icon: <Icons.Refresh size={12} />, label: '软中断', color: '#ec4899' },
  'Completion': { icon: <Icons.Check size={12} />, label: 'Completion', color: '#14b8a6' },
  'Rcu': { icon: <Icons.Lock size={12} />, label: 'RCU', color: '#f97316' },
}

// 节点详情数据类型
export interface NodeDetailData {
  name: string
  nodeType: FlowNodeType
  location?: {
    file: string
    line: number
    column?: number
  }
  description?: string
  confidence?: {
    level: ConfidenceLevel
    reason: string
  }
  children?: FlowTreeNode[]
  // 模拟的 LLVM IR 数据
  llvmIr?: string[]
  // 函数签名
  functionSignature?: string
  // 参数列表
  params?: { name: string; type: string }[]
  // 返回类型
  returnType?: string
  // 调用者信息
  callers?: string[]
}

export interface NodeDetailPanelProps {
  /** 节点详情数据 */
  node: NodeDetailData | null
  /** 面板标题 */
  title?: string
  /** 是否显示 LLVM IR 部分 */
  showLlvmIr?: boolean
  /** 加载状态 */
  loading?: boolean
  /** 主题模式 */
  theme?: 'dark' | 'light'
  /** 关闭回调 */
  onClose?: () => void
  /** 节点跳转回调 */
  onNodeClick?: (nodeName: string) => void
  /** 复制名称回调 */
  onCopyName?: (name: string) => void
  /** 跳转到源码回调 */
  onGoToSource?: (file: string, line: number) => void
  /** 展开调用图回调 */
  onExpandCallGraph?: (nodeName: string) => void
}

// 异步机制详情
interface AsyncMechanismDetail {
  type: string
  label: string
  icon: React.ReactNode
  color: string
  details: Record<string, string>
}

export function NodeDetailPanel({
  node,
  title = '节点详情',
  showLlvmIr = true,
  loading = false,
  theme = 'dark',
  onClose,
  onNodeClick,
  onCopyName,
  onGoToSource,
  onExpandCallGraph,
}: NodeDetailPanelProps) {
  // 展开状态
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
    info: true,
    llvmIr: true,
    calls: true,
    callers: false,
  })

  // 复制状态
  const [copiedSection, setCopiedSection] = useState<string | null>(null)

  // 动画引用
  const sectionRefs = useRef<Record<string, HTMLDivElement | null>>({})

  // 切换展开状态
  const toggleSection = useCallback((section: string) => {
    setExpandedSections(prev => ({
      ...prev,
      [section]: !prev[section],
    }))
  }, [])

  // 复制到剪贴板
  const copyToClipboard = useCallback(async (text: string, section: string) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopiedSection(section)
      setTimeout(() => setCopiedSection(null), 2000)
    } catch (err) {
      console.error('Failed to copy:', err)
    }
  }, [])

  // 快捷操作：复制函数名
  const handleCopyName = useCallback(() => {
    if (node) {
      copyToClipboard(node.name, 'name')
      onCopyName?.(node.name)
    }
  }, [node, copyToClipboard, onCopyName])

  // 快捷操作：跳转到源码
  const handleGoToSource = useCallback(() => {
    if (node?.location) {
      onGoToSource?.(node.location.file, node.location.line)
    }
  }, [node, onGoToSource])

  // 快捷操作：展开调用图
  const handleExpandCallGraph = useCallback(() => {
    if (node) {
      onExpandCallGraph?.(node.name)
    }
  }, [node, onExpandCallGraph])

  // 获取节点类型信息
  const getNodeTypeInfo = useCallback((nodeType: FlowNodeType): { icon: React.ReactNode; label: string; color: string; isAsync: boolean; asyncMechanism?: AsyncMechanismDetail } => {
    if (typeof nodeType === 'string') {
      const info = nodeTypeLabels[nodeType] || { icon: <Icons.Function size={13} />, label: nodeType, color: '#9ca3af' }
      return { ...info, isAsync: false }
    }

    if ('AsyncCallback' in nodeType) {
      const mechanism = nodeType.AsyncCallback?.mechanism
      if (mechanism) {
        const keys = Object.keys(mechanism)
        if (keys.length > 0) {
          const mechType = keys[0]
          const asyncInfo = asyncMechanismLabels[mechType] || { icon: <Icons.Zap size={12} />, label: mechType, color: '#a855f7' }

          // 获取机制详细信息
          const mechValue = mechanism[mechType as keyof typeof mechanism]
          let details: Record<string, string> = {}
          if (mechValue && typeof mechValue === 'object') {
            details = Object.entries(mechValue as Record<string, string>).reduce((acc, [k, v]) => {
              if (v) acc[k] = v
              return acc
            }, {} as Record<string, string>)
          }

          return {
            ...asyncInfo,
            isAsync: true,
            asyncMechanism: {
              type: mechType,
              ...asyncInfo,
              details,
            },
          }
        }
      }
      return { icon: <Icons.Zap size={13} />, label: '异步回调', color: '#a855f7', isAsync: true }
    }

    return { icon: <Icons.Function size={13} />, label: '未知', color: '#9ca3af', isAsync: false }
  }, [])

  // 格式化 LLVM IR
  const formatLlvmIr = useCallback((irLines: string[]): string => {
    return irLines.join('\n')
  }, [])

  // 渲染加载状态
  if (loading) {
    return (
      <div className={`node-detail-panel ${theme}`}>
        <div className="panel-header">
          <h3>{title}</h3>
        </div>
        <div className="panel-loading">
          <div className="loading-spinner" />
          <span>加载节点详情...</span>
        </div>
      </div>
    )
  }

  // 渲染空状态
  if (!node) {
    return (
      <div className={`node-detail-panel ${theme}`}>
        <div className="panel-header">
          <h3>{title}</h3>
        </div>
        <div className="panel-empty">
          <Icons.Link size={32} />
          <p>未选中节点</p>
          <span className="empty-hint">点击流程图中的节点查看详情</span>
        </div>
      </div>
    )
  }

  const nodeTypeInfo = getNodeTypeInfo(node.nodeType)
  const confidence = node.confidence ? confidenceInfo[node.confidence.level] : null
  const childCount = node.children?.length || 0
  const callerCount = node.callers?.length || 0

  // 示例 LLVM IR 数据（实际使用时从后端获取）
  const llvmIrLines = node.llvmIr || [
    `define ${node.returnType || 'void'} @${node.name}(${node.params?.map(p => `${p.type} ${p.name}`).join(', ') || ''}) {`,
    `entry:`,
    `  ; TODO: 函数实现`,
    `  ret ${node.returnType || 'void'} undef`,
    `}`,
  ]

  return (
    <div className={`node-detail-panel ${theme}`}>
      {/* Header */}
      <div className="panel-header">
        <div className="header-title">
          <span className="node-icon">{nodeTypeInfo.icon}</span>
          <h3>{node.name}()</h3>
          {nodeTypeInfo.isAsync && (
            <span
              className="async-badge"
              style={{ backgroundColor: `${nodeTypeInfo.color}22`, color: nodeTypeInfo.color }}
            >
              {nodeTypeInfo.asyncMechanism?.label || '异步'}
            </span>
          )}
        </div>
        {onClose && (
          <button className="close-btn" onClick={onClose} title="关闭">
            <Icons.X size={14} />
          </button>
        )}
      </div>

      {/* Quick Actions */}
      <div className="quick-actions">
        <button
          className="quick-action-btn"
          onClick={handleCopyName}
          title="复制函数名"
        >
          {copiedSection === 'name' ? <Icons.Check size={11} /> : <Icons.Copy size={11} />} 复制
        </button>
        {node.location && (
          <button
            className="quick-action-btn primary"
            onClick={handleGoToSource}
            title="跳转到源码"
          >
            <Icons.MapPin size={11} /> 源码
          </button>
        )}
        {childCount > 0 && (
          <button
            className="quick-action-btn"
            onClick={handleExpandCallGraph}
            title="展开调用图"
          >
            <Icons.Link size={11} /> 调用
          </button>
        )}
      </div>

      {/* Content */}
      <div className="panel-content">
        {/* 节点基本信息 */}
        <div className={`section ${expandedSections.info ? 'expanded' : ''}`}>
          <div
            className="section-header"
            onClick={() => toggleSection('info')}
          >
            <span className="expand-icon"><Icons.ChevronRight size={10} /></span>
            <span className="section-title">基本信息</span>
            {confidence && (
              <span
                className="confidence-badge"
                style={{ backgroundColor: `${confidence.color}22`, color: confidence.color }}
              >
                {confidence.icon} {confidence.label}
              </span>
            )}
          </div>

          {expandedSections.info && (
            <div className="section-content">
              {/* 返回类型 */}
              {node.returnType && (
                <div className="info-row">
                  <span className="info-label">返回类型</span>
                  <code className="info-value return-type">{node.returnType}</code>
                </div>
              )}

              {/* 位置信息 */}
              {node.location && (
                <div className="info-row">
                  <span className="info-label">位置</span>
                  <span className="info-value location">
                    <Icons.MapPin size={10} />
                    {node.location.file.split('/').pop()}:{node.location.line}
                  </span>
                </div>
              )}

              {/* 节点类型 */}
              <div className="info-row">
                <span className="info-label">节点类型</span>
                <span
                  className="info-value type-badge"
                  style={{ backgroundColor: `${nodeTypeInfo.color}22`, color: nodeTypeInfo.color }}
                >
                  {nodeTypeInfo.icon} {nodeTypeInfo.label}
                </span>
              </div>

              {/* 异步机制详情 */}
              {nodeTypeInfo.isAsync && nodeTypeInfo.asyncMechanism && (
                <div className="async-details">
                  <div className="async-detail-row">
                    <span className="async-label">异步机制</span>
                    <span
                      className="async-value"
                      style={{ color: nodeTypeInfo.asyncMechanism.color }}
                    >
                      {nodeTypeInfo.asyncMechanism.icon} {nodeTypeInfo.asyncMechanism.label}
                    </span>
                  </div>
                  {Object.entries(nodeTypeInfo.asyncMechanism.details).map(([key, value]) => (
                    <div key={key} className="async-detail-row">
                      <span className="async-label">{key}</span>
                      <code className="async-value">{value}</code>
                    </div>
                  ))}
                </div>
              )}

              {/* 参数列表 */}
              {node.params && node.params.length > 0 && (
                <div className="params-section">
                  <span className="info-label">参数 ({node.params.length})</span>
                  <div className="params-list">
                    {node.params.map((param, idx) => (
                      <div key={idx} className="param-item">
                        <code className="param-type">{param.type}</code>
                        <span className="param-name">{param.name}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* 调用计数 */}
              <div className="info-row">
                <span className="info-label">调用函数</span>
                <span className="info-value">
                  {childCount > 0 ? `${childCount} 个` : '无'}
                </span>
              </div>

              {/* 置信度原因 */}
              {node.confidence?.reason && (
                <div className="confidence-reason">
                  <span className="reason-label">原因</span>
                  <p>{node.confidence.reason}</p>
                </div>
              )}

              {/* 描述 */}
              {node.description && (
                <div className="description-section">
                  <span className="info-label">描述</span>
                  <p className="description">{node.description}</p>
                </div>
              )}
            </div>
          )}
        </div>

        {/* LLVM IR 代码 */}
        {showLlvmIr && (
          <div className={`section ${expandedSections.llvmIr ? 'expanded' : ''}`}>
            <div
              className="section-header"
              onClick={() => toggleSection('llvmIr')}
            >
              <span className="expand-icon"><Icons.ChevronRight size={10} /></span>
              <span className="section-title">LLVM IR</span>
              <span className="section-badge">{llvmIrLines.length} 行</span>
              <button
                className="copy-btn"
                onClick={(e) => {
                  e.stopPropagation()
                  copyToClipboard(formatLlvmIr(llvmIrLines), 'llvmIr')
                }}
                title="复制"
              >
                {copiedSection === 'llvmIr' ? <Icons.Check size={11} /> : <Icons.Copy size={11} />}
              </button>
            </div>

            {expandedSections.llvmIr && (
              <div className="section-content">
                <pre className="llvm-ir-code">
                  <code>{llvmIrLines.join('\n')}</code>
                </pre>
              </div>
            )}
          </div>
        )}

        {/* 被调用函数 */}
        {node.children && node.children.length > 0 && (
          <div className={`section ${expandedSections.calls ? 'expanded' : ''}`}>
            <div
              className="section-header"
              onClick={() => toggleSection('calls')}
            >
              <span className="expand-icon"><Icons.ChevronRight size={10} /></span>
              <span className="section-title">调用 ({childCount})</span>
            </div>

            {expandedSections.calls && (
              <div className="section-content">
                <ul className="calls-list">
                  {node.children.map((child, idx) => (
                    <li
                      key={idx}
                      className="call-item"
                      onClick={() => onNodeClick?.(child.name)}
                    >
                      <span className="call-icon">
                        {getNodeTypeInfo(child.node_type).icon}
                      </span>
                      <code className="call-name">{child.name}()</code>
                      {child.location && (
                        <span className="call-location">
                          {child.location.file.split('/').pop()}:{child.location.line}
                        </span>
                      )}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        )}

        {/* 调用者 */}
        {node.callers && node.callers.length > 0 && (
          <div className={`section ${expandedSections.callers ? 'expanded' : ''}`}>
            <div
              className="section-header"
              onClick={() => toggleSection('callers')}
            >
              <span className="expand-icon"><Icons.ChevronRight size={10} /></span>
              <span className="section-title">被调用 ({callerCount})</span>
            </div>

            {expandedSections.callers && (
              <div className="section-content">
                <ul className="calls-list">
                  {node.callers.map((caller, idx) => (
                    <li
                      key={idx}
                      className="call-item"
                      onClick={() => onNodeClick?.(caller)}
                    >
                      <span className="call-icon"><Icons.Function size={12} /></span>
                      <code className="call-name">{caller}()</code>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="panel-footer">
        <span className="footer-hint">点击节点跳转</span>
      </div>
    </div>
  )
}

export default NodeDetailPanel
