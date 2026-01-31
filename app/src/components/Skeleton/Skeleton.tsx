import React from 'react'
import { cn } from '../../lib/utils/cn'

interface SkeletonProps {
  className?: string
  variant?: 'text' | 'rectangular' | 'circular'
  width?: number | string
  height?: number | string
  animation?: 'pulse' | 'wave' | 'none'
}

/**
 * 通用骨架屏组件
 */
export function Skeleton({
  className,
  variant = 'text',
  width,
  height,
  animation = 'pulse',
}: SkeletonProps) {
  const baseStyles = 'bg-[var(--bg-tertiary)]'
  
  const variantStyles = {
    text: 'rounded',
    rectangular: 'rounded-md',
    circular: 'rounded-full',
  }
  
  const animationStyles = {
    pulse: 'animate-pulse',
    wave: 'animate-shimmer',
    none: '',
  }

  const style: React.CSSProperties = {}
  if (width) style.width = typeof width === 'number' ? `${width}px` : width
  if (height) style.height = typeof height === 'number' ? `${height}px` : height

  return (
    <div
      className={cn(
        baseStyles,
        variantStyles[variant],
        animationStyles[animation],
        className
      )}
      style={style}
    />
  )
}

/**
 * 文件树骨架屏
 */
export function FileTreeSkeleton({ count = 8 }: { count?: number }) {
  return (
    <div className="p-2 space-y-1">
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} className="flex items-center gap-2 py-1" style={{ paddingLeft: `${(i % 3) * 12}px` }}>
          <Skeleton variant="rectangular" width={16} height={16} />
          <Skeleton variant="text" width={`${60 + Math.random() * 40}%`} height={16} />
        </div>
      ))}
    </div>
  )
}

/**
 * 执行流视图骨架屏
 */
export function FlowViewSkeleton() {
  return (
    <div className="flex items-center justify-center h-full p-8">
      <div className="flex flex-col items-center gap-4">
        {/* 中心节点 */}
        <Skeleton variant="rectangular" width={120} height={40} className="rounded-lg" />
        
        {/* 连接线和子节点 */}
        <div className="flex items-center gap-8">
          <div className="flex flex-col items-center gap-2">
            <Skeleton variant="rectangular" width={2} height={30} />
            <Skeleton variant="rectangular" width={100} height={36} className="rounded-lg" />
          </div>
          <div className="flex flex-col items-center gap-2">
            <Skeleton variant="rectangular" width={2} height={30} />
            <Skeleton variant="rectangular" width={100} height={36} className="rounded-lg" />
          </div>
          <div className="flex flex-col items-center gap-2">
            <Skeleton variant="rectangular" width={2} height={30} />
            <Skeleton variant="rectangular" width={100} height={36} className="rounded-lg" />
          </div>
        </div>
        
        <p className="text-xs text-[var(--text-muted)] mt-4">加载执行流中...</p>
      </div>
    </div>
  )
}

/**
 * 代码编辑器骨架屏
 */
export function EditorSkeleton() {
  return (
    <div className="p-4 space-y-2">
      {Array.from({ length: 15 }).map((_, i) => (
        <div key={i} className="flex items-center gap-3">
          {/* 行号 */}
          <Skeleton variant="text" width={30} height={16} />
          {/* 代码行 */}
          <Skeleton 
            variant="text" 
            width={`${Math.random() * 60 + 20}%`} 
            height={16} 
          />
        </div>
      ))}
    </div>
  )
}

/**
 * 详情面板骨架屏
 */
export function DetailPanelSkeleton() {
  return (
    <div className="p-4 space-y-4">
      {/* 标题 */}
      <Skeleton variant="text" width="60%" height={24} />
      
      {/* 分隔线 */}
      <Skeleton variant="rectangular" width="100%" height={1} />
      
      {/* 信息项 */}
      {Array.from({ length: 4 }).map((_, i) => (
        <div key={i} className="space-y-1">
          <Skeleton variant="text" width={80} height={14} />
          <Skeleton variant="text" width="80%" height={18} />
        </div>
      ))}
      
      {/* 列表 */}
      <div className="space-y-2 mt-4">
        <Skeleton variant="text" width={100} height={14} />
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="flex items-center gap-2 py-1">
            <Skeleton variant="circular" width={8} height={8} />
            <Skeleton variant="text" width={`${50 + Math.random() * 30}%`} height={16} />
          </div>
        ))}
      </div>
    </div>
  )
}

/**
 * 搜索结果骨架屏
 */
export function SearchResultSkeleton({ count = 5 }: { count?: number }) {
  return (
    <div className="p-2 space-y-3">
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} className="p-2 rounded-md bg-[var(--bg-secondary)]">
          <div className="flex items-center gap-2 mb-1">
            <Skeleton variant="rectangular" width={14} height={14} />
            <Skeleton variant="text" width="70%" height={14} />
          </div>
          <Skeleton variant="text" width="90%" height={12} />
        </div>
      ))}
    </div>
  )
}

export default Skeleton
