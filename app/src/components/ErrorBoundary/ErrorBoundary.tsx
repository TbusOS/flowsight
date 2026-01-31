import React from 'react'
import { AlertTriangle, RefreshCw } from 'lucide-react'

interface ErrorBoundaryProps {
  children: React.ReactNode
  fallback?: React.ReactNode
  onReset?: () => void
}

interface ErrorBoundaryState {
  hasError: boolean
  error: Error | null
}

/**
 * 全局错误边界组件
 * 捕获子组件的 JavaScript 错误，显示友好的错误界面
 */
export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props)
    this.state = { hasError: false, error: null }
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo)
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null })
    this.props.onReset?.()
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback
      }

      return (
        <div className="flex flex-col items-center justify-center h-full p-8 bg-[var(--bg-secondary)]">
          <div className="flex flex-col items-center max-w-md text-center">
            <div className="p-4 mb-4 rounded-full bg-red-500/10">
              <AlertTriangle className="w-12 h-12 text-red-500" />
            </div>
            <h2 className="mb-2 text-xl font-semibold text-[var(--text-primary)]">
              出错了
            </h2>
            <p className="mb-4 text-sm text-[var(--text-secondary)]">
              {this.state.error?.message || '发生了意外错误'}
            </p>
            <button
              onClick={this.handleReset}
              className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 transition-colors"
            >
              <RefreshCw className="w-4 h-4" />
              重试
            </button>
            {this.state.error && (
              <pre className="mt-4 p-4 text-xs text-left bg-[var(--bg-tertiary)] rounded-md overflow-auto max-w-full max-h-48 text-red-400">
                {this.state.error.stack}
              </pre>
            )}
          </div>
        </div>
      )
    }

    return this.props.children
  }
}

/**
 * 局部错误边界 - 用于包裹可能出错的子组件
 * 错误不会影响其他组件
 */
export function LocalErrorBoundary({ 
  children, 
  name = '组件' 
}: { 
  children: React.ReactNode
  name?: string 
}) {
  return (
    <ErrorBoundary
      fallback={
        <div className="flex items-center justify-center p-4 text-sm text-[var(--text-secondary)]">
          <AlertTriangle className="w-4 h-4 mr-2 text-yellow-500" />
          {name}加载失败
        </div>
      }
    >
      {children}
    </ErrorBoundary>
  )
}

export default ErrorBoundary
