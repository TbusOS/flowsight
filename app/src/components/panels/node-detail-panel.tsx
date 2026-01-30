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
} from "lucide-react"
import { cn } from "../../lib/utils"
import { Button } from "../ui/button"
import { useAtomValue } from "jotai"
import { selectedNodeAtom, type SelectedNodeDetail } from "../../lib/atoms/layout-atoms"

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

export function NodeDetailPanel({ 
  className, 
  detail: externalDetail,
  onCallClick,
  onCalledByClick,
  onLocationClick,
}: NodeDetailPanelProps) {
  // 从 Jotai atom 获取选中节点，外部传入优先
  const atomDetail = useAtomValue(selectedNodeAtom)
  const detail = externalDetail ?? atomDetail

  if (!detail) {
    return (
      <div 
        data-testid="detail-panel"
        className={cn("flex flex-col h-full items-center justify-center p-4 text-center", className)}
      >
        <Box className="h-8 w-8 text-[var(--text-muted)] mb-2" />
        <p className="text-xs text-[var(--text-muted)]">选择一个节点查看详情</p>
      </div>
    )
  }

  // 获取文件名（从完整路径）
  const fileName = detail.file_path?.split('/').pop() || '未知文件'

  return (
    <div 
      data-testid="detail-panel"
      className={cn("flex flex-col h-full overflow-hidden", className)}
    >
      {/* Header - 函数名称和基本信息 */}
      <div className="px-3 py-2 border-b border-[var(--border-subtle)] bg-[var(--bg-secondary)]">
        <div className="flex items-center gap-2 mb-2">
          {detail.is_callback ? (
            <Zap className="h-4 w-4 text-amber-400" />
          ) : (
            <FunctionSquare className="h-4 w-4 text-[var(--accent)]" />
          )}
          <span 
            data-testid="detail-function-name"
            className="text-sm font-medium text-[var(--text-primary)] truncate"
            title={detail.name}
          >
            {detail.name}
          </span>
          {detail.is_callback && (
            <span className="px-1.5 py-0.5 text-[10px] rounded bg-amber-500/20 text-amber-300">
              回调
            </span>
          )}
        </div>
        
        {/* 返回类型 */}
        {detail.return_type && (
          <div className="flex items-center gap-1.5 text-xs text-[var(--text-muted)] mb-1.5">
            <ArrowRight className="h-3 w-3" />
            <span className="text-[var(--accent)]">{detail.return_type}</span>
          </div>
        )}

        {/* 文件位置 */}
        <div 
          data-testid="detail-file-path"
          className="flex items-center gap-1.5 text-[10px] text-[var(--text-muted)] cursor-pointer hover:text-[var(--text-primary)] transition-colors"
          onClick={() => detail.file_path && onLocationClick?.(detail.file_path, detail.line)}
          title={detail.file_path || '未知位置'}
        >
          <MapPin className="h-3 w-3 flex-shrink-0" />
          <span className="truncate">{fileName}:{detail.line}</span>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {/* 回调上下文 */}
        {detail.is_callback && detail.callback_context && (
          <div className="px-3 py-2 border-b border-[var(--border-subtle)] bg-amber-500/5">
            <h4 className="text-[10px] uppercase tracking-wide text-amber-400 mb-1.5 flex items-center gap-1.5">
              <Zap className="h-3 w-3" />
              异步上下文
            </h4>
            <p className="text-xs text-[var(--text-secondary)]">{detail.callback_context}</p>
          </div>
        )}

        {/* 参数列表 */}
        {detail.parameters && detail.parameters.length > 0 && (
          <div 
            data-testid="detail-parameters"
            className="px-3 py-2 border-b border-[var(--border-subtle)]"
          >
            <h4 className="text-[10px] uppercase tracking-wide text-[var(--text-muted)] mb-1.5 flex items-center gap-1.5">
              <Hash className="h-3 w-3" />
              参数 ({detail.parameters.length})
            </h4>
            <div className="space-y-1">
              {detail.parameters.map((param, i) => (
                <div key={i} className="flex items-center gap-2 text-xs font-mono">
                  <span className="text-[var(--text-secondary)]">{param.name}</span>
                  <span className="text-[var(--text-muted)]">:</span>
                  <span className="text-[var(--accent)]">{param.type}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 调用的函数 */}
        {detail.calls && detail.calls.length > 0 && (
          <div 
            data-testid="detail-calls"
            className="px-3 py-2 border-b border-[var(--border-subtle)]"
          >
            <h4 className="text-[10px] uppercase tracking-wide text-[var(--text-muted)] mb-1.5 flex items-center gap-1.5">
              <ArrowRight className="h-3 w-3" />
              调用 ({detail.calls.length})
            </h4>
            <div className="space-y-0.5 max-h-32 overflow-y-auto">
              {detail.calls.map((call, i) => (
                <div
                  key={i}
                  data-testid={`detail-call-${i}`}
                  className="flex items-center gap-2 px-2 py-1 rounded-md hover:bg-[var(--bg-tertiary)] cursor-pointer text-xs transition-colors"
                  onClick={() => onCallClick?.(call)}
                >
                  <Code className="h-3 w-3 text-[var(--text-muted)]" />
                  <span className="text-[var(--accent)] font-mono truncate">{call}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 被调用 */}
        {detail.called_by && detail.called_by.length > 0 && (
          <div 
            data-testid="detail-called-by"
            className="px-3 py-2 border-b border-[var(--border-subtle)]"
          >
            <h4 className="text-[10px] uppercase tracking-wide text-[var(--text-muted)] mb-1.5 flex items-center gap-1.5">
              <ArrowLeft className="h-3 w-3" />
              被调用 ({detail.called_by.length})
            </h4>
            <div className="space-y-0.5 max-h-32 overflow-y-auto">
              {detail.called_by.map((caller, i) => (
                <div
                  key={i}
                  data-testid={`detail-called-by-${i}`}
                  className="flex items-center gap-2 px-2 py-1 rounded-md hover:bg-[var(--bg-tertiary)] cursor-pointer text-xs transition-colors"
                  onClick={() => onCalledByClick?.(caller)}
                >
                  <Code className="h-3 w-3 text-[var(--text-muted)]" />
                  <span className="text-[var(--text-secondary)] font-mono truncate">{caller}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 节点类型和描述 */}
        {(detail.node_type || detail.description) && (
          <div className="px-3 py-2 border-b border-[var(--border-subtle)]">
            <h4 className="text-[10px] uppercase tracking-wide text-[var(--text-muted)] mb-1.5">详情</h4>
            {detail.node_type && (
              <div className="flex items-center gap-2 text-xs mb-1">
                <span className="text-[var(--text-muted)]">类型:</span>
                <span className="px-1.5 py-0.5 rounded bg-[var(--bg-tertiary)] text-[var(--text-secondary)]">
                  {detail.node_type}
                </span>
              </div>
            )}
            {detail.description && (
              <p className="text-xs text-[var(--text-secondary)] mt-1">{detail.description}</p>
            )}
          </div>
        )}

        {/* 空状态 - 没有调用信息时显示 */}
        {(!detail.calls || detail.calls.length === 0) && 
         (!detail.called_by || detail.called_by.length === 0) && 
         (!detail.parameters || detail.parameters.length === 0) && (
          <div className="px-3 py-4 text-center">
            <FileCode className="h-6 w-6 mx-auto mb-2 text-[var(--text-muted)]" />
            <p className="text-xs text-[var(--text-muted)]">暂无调用关系信息</p>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="px-3 py-2 border-t border-[var(--border-subtle)] flex gap-2 bg-[var(--bg-secondary)]">
        <Button 
          variant="secondary" 
          size="sm" 
          className="flex-1 text-xs h-7"
          onClick={() => detail.file_path && onLocationClick?.(detail.file_path, detail.line)}
        >
          <GitBranch className="h-3 w-3 mr-1" />
          查看调用链
        </Button>
        <Button variant="ghost" size="sm" className="flex-1 text-xs h-7">
          <Box className="h-3 w-3 mr-1" />
          LLVM IR
        </Button>
      </div>
    </div>
  )
}
