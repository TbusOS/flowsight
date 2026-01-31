"use client"

import * as React from "react"
import { cn } from "../../lib/utils"

interface ResizableDividerProps {
  /** 拖拽方向 */
  direction: "horizontal" | "vertical"
  /** 当前尺寸变化回调 */
  onResize: (delta: number) => void
  /** 拖拽开始回调 */
  onResizeStart?: () => void
  /** 拖拽结束回调 */
  onResizeEnd?: () => void
  /** 自定义类名 */
  className?: string
  /** 最小尺寸约束 */
  minSize?: number
  /** 最大尺寸约束 */
  maxSize?: number
}

export function ResizableDivider({
  direction,
  onResize,
  onResizeStart,
  onResizeEnd,
  className,
}: ResizableDividerProps) {
  const [isDragging, setIsDragging] = React.useState(false)
  const startPosRef = React.useRef(0)

  const handleMouseDown = React.useCallback((e: React.MouseEvent) => {
    e.preventDefault()
    setIsDragging(true)
    startPosRef.current = direction === "horizontal" ? e.clientX : e.clientY
    onResizeStart?.()
  }, [direction, onResizeStart])

  React.useEffect(() => {
    if (!isDragging) return

    const handleMouseMove = (e: MouseEvent) => {
      const currentPos = direction === "horizontal" ? e.clientX : e.clientY
      const delta = currentPos - startPosRef.current
      startPosRef.current = currentPos
      onResize(delta)
    }

    const handleMouseUp = () => {
      setIsDragging(false)
      onResizeEnd?.()
    }

    document.addEventListener("mousemove", handleMouseMove)
    document.addEventListener("mouseup", handleMouseUp)

    // 设置拖拽时的光标样式
    document.body.style.cursor = direction === "horizontal" ? "col-resize" : "row-resize"
    document.body.style.userSelect = "none"

    return () => {
      document.removeEventListener("mousemove", handleMouseMove)
      document.removeEventListener("mouseup", handleMouseUp)
      document.body.style.cursor = ""
      document.body.style.userSelect = ""
    }
  }, [isDragging, direction, onResize, onResizeEnd])

  const isHorizontal = direction === "horizontal"

  return (
    <div
      data-testid="resizable-divider"
      className={cn(
        "group flex-shrink-0 transition-colors",
        isHorizontal 
          ? "w-1 cursor-col-resize hover:bg-[var(--accent)]/30" 
          : "h-1 cursor-row-resize hover:bg-[var(--accent)]/30",
        isDragging && "bg-[var(--accent)]/50",
        className
      )}
      onMouseDown={handleMouseDown}
    >
      {/* 可视化拖拽指示器 */}
      <div
        className={cn(
          "opacity-0 group-hover:opacity-100 transition-opacity bg-[var(--accent)]",
          isHorizontal
            ? "w-0.5 h-full mx-auto"
            : "h-0.5 w-full my-auto",
          isDragging && "opacity-100"
        )}
      />
    </div>
  )
}

/**
 * 可调整大小的面板容器
 */
interface ResizablePanelProps {
  /** 面板位置 */
  position: "left" | "right" | "bottom"
  /** 当前尺寸 */
  size: number
  /** 尺寸变化回调 */
  onSizeChange: (size: number) => void
  /** 最小尺寸 */
  minSize?: number
  /** 最大尺寸 */
  maxSize?: number
  /** 是否显示 */
  open: boolean
  /** 子元素 */
  children: React.ReactNode
  /** 自定义类名 */
  className?: string
}

export function ResizablePanel({
  position,
  size,
  onSizeChange,
  minSize = 150,
  maxSize = 600,
  open,
  children,
  className,
}: ResizablePanelProps) {
  const handleResize = React.useCallback((delta: number) => {
    // 根据位置调整 delta 方向
    const adjustedDelta = position === "left" || position === "bottom" ? delta : -delta
    const newSize = Math.min(maxSize, Math.max(minSize, size + adjustedDelta))
    onSizeChange(newSize)
  }, [position, size, minSize, maxSize, onSizeChange])

  if (!open) return null

  const isVertical = position === "bottom"
  const dividerPosition = position === "left" ? "right" : position === "right" ? "left" : "top"

  return (
    <div
      data-testid={`resizable-panel-${position}`}
      className={cn(
        "relative flex",
        isVertical ? "flex-col" : "flex-row",
        className
      )}
      style={{
        [isVertical ? "height" : "width"]: size,
      }}
    >
      {/* 左侧分隔条 */}
      {dividerPosition === "left" && (
        <ResizableDivider
          direction="horizontal"
          onResize={handleResize}
        />
      )}

      {/* 顶部分隔条 */}
      {dividerPosition === "top" && (
        <ResizableDivider
          direction="vertical"
          onResize={handleResize}
        />
      )}

      {/* 面板内容 */}
      <div className="flex-1 overflow-hidden">
        {children}
      </div>

      {/* 右侧分隔条 */}
      {dividerPosition === "right" && (
        <ResizableDivider
          direction="horizontal"
          onResize={handleResize}
        />
      )}
    </div>
  )
}
