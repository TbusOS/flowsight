"use client"

import * as React from "react"
import {
  FunctionSquare,
  ArrowRight,
  Hash,
  Box,
  Clock,
  GitBranch,
  AlertCircle,
  CheckCircle2,
  XCircle,
} from "lucide-react"
import { cn } from "../../lib/utils"
import { Button } from "../ui/button"

// 模拟数据
interface NodeDetail {
  id: string
  name: string
  type: "function" | "call" | "branch" | "loop" | "entry" | "exit"
  file: string
  line: number
  column: number
  returnType?: string
  parameters?: { name: string; type: string }[]
  calls?: string[]
  calledBy?: string[]
  complexity?: number
  execCount?: number
}

interface NodeDetailPanelProps {
  className?: string
  detail?: NodeDetail | null
}

export function NodeDetailPanel({ className, detail = null }: NodeDetailPanelProps) {
  if (!detail) {
    return (
      <div className={cn("flex flex-col h-full items-center justify-center p-4 text-center", className)}>
        <Box className="h-8 w-8 text-[var(--text-muted)] mb-2" />
        <p className="text-xs text-[var(--text-muted)]">选择一个节点查看详情</p>
      </div>
    )
  }

  return (
    <div className={cn("flex flex-col h-full overflow-hidden", className)}>
      {/* Header */}
      <div className="px-3 py-2 border-b border-[var(--border-subtle)]">
        <div className="flex items-center gap-2 mb-2">
          <FunctionSquare className="h-4 w-4 text-[var(--accent)]" />
          <span className="text-sm font-medium text-[var(--text-primary)]">{detail.name}</span>
        </div>
        <div className="flex items-center gap-2 text-[10px] text-[var(--text-muted)]">
          <span className="px-1.5 py-0.5 rounded bg-[var(--bg-tertiary)]">
            {detail.file}:{detail.line}
          </span>
          {detail.returnType && (
            <span className="px-1.5 py-0.5 rounded bg-[var(--accent)]/10 text-[var(--accent)]">
              → {detail.returnType}
            </span>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {/* Parameters */}
        {detail.parameters && detail.parameters.length > 0 && (
          <div className="px-3 py-2 border-b border-[var(--border-subtle)]">
            <h4 className="text-[10px] uppercase tracking-wide text-[var(--text-muted)] mb-1.5">参数</h4>
            <div className="space-y-1">
              {detail.parameters.map((param, i) => (
                <div key={i} className="flex items-center gap-2 text-xs">
                  <Hash className="h-3 w-3 text-[var(--text-muted)]" />
                  <span className="text-[var(--text-secondary)]">{param.name}</span>
                  {param.type && (
                    <span className="text-[var(--text-muted)]">: {param.type}</span>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Calls */}
        {detail.calls && detail.calls.length > 0 && (
          <div className="px-3 py-2 border-b border-[var(--border-subtle)]">
            <h4 className="text-[10px] uppercase tracking-wide text-[var(--text-muted)] mb-1.5 flex items-center gap-1.5">
              <ArrowRight className="h-3 w-3" />
              调用
            </h4>
            <div className="space-y-1">
              {detail.calls.map((call, i) => (
                <div
                  key={i}
                  className="flex items-center gap-2 px-2 py-1 rounded-md hover:bg-[var(--bg-tertiary)] cursor-pointer text-xs"
                >
                  <span className="text-[var(--accent)]">{call}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Called By */}
        {detail.calledBy && detail.calledBy.length > 0 && (
          <div className="px-3 py-2 border-b border-[var(--border-subtle)]">
            <h4 className="text-[10px] uppercase tracking-wide text-[var(--text-muted)] mb-1.5 flex items-center gap-1.5">
              <GitBranch className="h-3 w-3" />
              被调用
            </h4>
            <div className="space-y-1">
              {detail.calledBy.map((caller, i) => (
                <div
                  key={i}
                  className="flex items-center gap-2 px-2 py-1 rounded-md hover:bg-[var(--bg-tertiary)] cursor-pointer text-xs"
                >
                  <span className="text-[var(--text-secondary)]">{caller}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Metrics */}
        <div className="px-3 py-2">
          <h4 className="text-[10px] uppercase tracking-wide text-[var(--text-muted)] mb-1.5">指标</h4>
          <div className="grid grid-cols-2 gap-2">
            {detail.complexity && (
              <div className="p-2 rounded-lg bg-[var(--bg-tertiary)]">
                <div className="flex items-center gap-1.5 mb-1">
                  <AlertCircle className="h-3 w-3 text-[var(--warning)]" />
                  <span className="text-[10px] text-[var(--text-muted)]">复杂度</span>
                </div>
                <span className="text-sm font-medium text-[var(--text-primary)]">{detail.complexity}</span>
              </div>
            )}
            {detail.execCount && (
              <div className="p-2 rounded-lg bg-[var(--bg-tertiary)]">
                <div className="flex items-center gap-1.5 mb-1">
                  <Clock className="h-3 w-3 text-[var(--info)]" />
                  <span className="text-[10px] text-[var(--text-muted)]">执行次数</span>
                </div>
                <span className="text-sm font-medium text-[var(--text-primary)]">{detail.execCount}</span>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Actions */}
      <div className="px-3 py-2 border-t border-[var(--border-subtle)] flex gap-2">
        <Button variant="secondary" size="sm" className="flex-1 text-xs h-7">
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
