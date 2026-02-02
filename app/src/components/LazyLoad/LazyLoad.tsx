/**
 * Lazy loading wrapper components for code splitting
 */
import React, { Suspense, lazy } from 'react'
import { Loader2 } from 'lucide-react'
import { cn } from '../../lib/utils'

// Loading fallback component
interface LoadingFallbackProps {
  className?: string
  message?: string
}

export function LoadingFallback({ className, message = '加载中...' }: LoadingFallbackProps) {
  return (
    <div className={cn(
      'flex flex-col items-center justify-center gap-2 p-4 text-[var(--text-muted)]',
      className
    )}>
      <Loader2 className="w-6 h-6 animate-spin" />
      <span className="text-sm">{message}</span>
    </div>
  )
}

// Error fallback component
interface ErrorFallbackProps {
  error: Error
  resetError?: () => void
  className?: string
}

export function ErrorFallback({ error, resetError, className }: ErrorFallbackProps) {
  return (
    <div className={cn(
      'flex flex-col items-center justify-center gap-3 p-4 text-center',
      className
    )}>
      <div className="text-red-400 text-sm">加载失败</div>
      <div className="text-[var(--text-muted)] text-xs max-w-[200px] truncate">
        {error.message}
      </div>
      {resetError && (
        <button
          onClick={resetError}
          className="px-3 py-1 text-xs bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)] rounded transition-colors"
        >
          重试
        </button>
      )}
    </div>
  )
}

// Pre-defined lazy components for heavy modules
export const LazyMonacoEditor = lazy(() => import('../Editor/CodeEditor'))

export const LazyFlowView = lazy(() => 
  import('../FlowView/FlowView').then(m => ({ default: m.FlowView }))
)

export const LazyDiffView = lazy(() => 
  import('../DiffView/DiffView').then(m => ({ default: m.DiffView }))
)

export const LazySettings = lazy(() => 
  import('../Settings/Settings').then(m => ({ default: m.Settings }))
)

export const LazyLlvmIrPanel = lazy(() => 
  import('../LlvmIrPanel/LlvmIrPanel').then(m => ({ default: m.LlvmIrPanel }))
)

export const LazyFlowExportPanel = lazy(() => 
  import('../FlowExportPanel').then(m => ({ default: m.FlowExportPanel }))
)

/**
 * Wrapper component for lazy loaded content
 */
interface LazyWrapperProps {
  children: React.ReactNode
  fallback?: React.ReactNode
  fallbackMessage?: string
}

export function LazyWrapper({ children, fallback, fallbackMessage }: LazyWrapperProps) {
  return (
    <Suspense fallback={fallback || <LoadingFallback message={fallbackMessage} />}>
      {children}
    </Suspense>
  )
}
