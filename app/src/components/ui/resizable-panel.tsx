"use client"

import * as React from "react"
import { motion, AnimatePresence } from "framer-motion"
import { GripVertical, X } from "lucide-react"
import { cn } from "../../lib/utils"
import { Button } from "../ui/button"
import { rightPanelOpenAtom, rightPanelWidthAtom } from "../../lib/atoms/layout-atoms"
import { useAtom } from "jotai"

interface ResizablePanelProps {
  open: boolean
  onClose: () => void
  side?: "right" | "left"
  defaultWidth?: number
  minWidth?: number
  maxWidth?: number
  title?: React.ReactNode
  children: React.ReactNode
  className?: string
}

export function ResizablePanel({
  open,
  onClose,
  side = "right",
  defaultWidth = 280,
  minWidth = 200,
  maxWidth = 400,
  title,
  children,
  className,
}: ResizablePanelProps) {
  const [width, setWidth] = useAtom(rightPanelWidthAtom)
  const [isResizing, setIsResizing] = React.useState(false)

  React.useEffect(() => {
    if (open) {
      setWidth(defaultWidth)
    }
  }, [open, defaultWidth, setWidth])

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault()
    setIsResizing(true)
    const startX = e.clientX
    const startWidth = width

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const diff = side === "right"
        ? startX - moveEvent.x
        : moveEvent.x - startX
      const newWidth = Math.min(Math.max(startWidth + diff, minWidth), maxWidth)
      setWidth(newWidth)
    }

    const handleMouseUp = () => {
      setIsResizing(false)
      document.removeEventListener("mousemove", handleMouseMove)
      document.removeEventListener("mouseup", handleMouseUp)
    }

    document.addEventListener("mousemove", handleMouseMove)
    document.addEventListener("mouseup", handleMouseUp)
  }

  const isRight = side === "right"

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          className={cn(
            "flex h-full border-l border-[var(--border-subtle)] bg-[var(--bg-secondary)]",
            className
          )}
          style={{ width }}
          initial={{ width: 0, opacity: 0 }}
          animate={{ width: width, opacity: 1 }}
          exit={{ width: 0, opacity: 0 }}
          transition={{ type: "spring", damping: 25, stiffness: 300 }}
        >
          {/* Panel Content */}
          <div className="flex h-full flex-1 flex-col overflow-hidden">
            {/* Header */}
            {(title) && (
              <div className="flex items-center justify-between border-b border-[var(--border-subtle)] px-3 py-2.5">
                <h3 className="text-xs font-medium uppercase tracking-wide text-[var(--text-secondary)]">
                  {title}
                </h3>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6 rounded-md text-[var(--text-muted)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]"
                  onClick={onClose}
                >
                  <X className="h-3.5 w-3.5" />
                </Button>
              </div>
            )}

            {/* Content */}
            <div className="flex-1 overflow-y-auto">
              {children}
            </div>
          </div>

          {/* Resize Handle */}
          <div
            className={cn(
              "absolute top-0 bottom-0 w-1 cursor-col-resize flex items-center justify-center",
              isRight ? "-left-0.5" : "-right-0.5",
              isResizing ? "bg-[var(--accent)]" : "hover:bg-[var(--border-medium)]"
            )}
            onMouseDown={handleMouseDown}
          >
            <GripVertical className={cn(
              "h-4 text-[var(--text-muted)] opacity-0",
              isResizing && "opacity-100"
            )} />
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}

// Inline Side Panel - integrated into main layout
interface SidePanelContainerProps {
  side?: "right" | "left"
  width: number
  children: React.ReactNode
  className?: string
}

export function SidePanelContainer({ side = "right", width, children, className }: SidePanelContainerProps) {
  const isRight = side === "right"

  return (
    <motion.div
      className={cn(
        "flex h-full flex-col border-l border-[var(--border-subtle)] bg-[var(--bg-secondary)] overflow-hidden",
        className
      )}
      initial={{ width: 0 }}
      animate={{ width: width }}
      transition={{ type: "spring", damping: 25, stiffness: 300 }}
    >
      {children}
    </motion.div>
  )
}
