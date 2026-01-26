"use client"

import * as React from "react"
import { motion, AnimatePresence } from "framer-motion"
import { X } from "lucide-react"
import { cn } from "../../lib/utils"
import { Button } from "./button"

interface FloatingPanelProps {
  open: boolean
  onClose: () => void
  side?: "right" | "left" | "bottom"
  title?: React.ReactNode
  description?: React.ReactNode
  children: React.ReactNode
  className?: string
  showClose?: boolean
  width?: string
  height?: string
}

export function FloatingPanel({
  open,
  onClose,
  side = "right",
  title,
  description,
  children,
  className,
  showClose = true,
  width = "320px",
  height,
}: FloatingPanelProps) {
  const getPositionStyles = () => {
    switch (side) {
      case "right":
        return {
          container: "right-4 top-20 bottom-20",
          width: width,
          initial: { opacity: 0, x: 20 },
          animate: { opacity: 1, x: 0 },
          exit: { opacity: 0, x: 20 },
        }
      case "left":
        return {
          container: "left-4 top-20 bottom-20",
          width: width,
          initial: { opacity: 0, x: -20 },
          animate: { opacity: 1, x: 0 },
          exit: { opacity: 0, x: -20 },
        }
      case "bottom":
        return {
          container: "bottom-4 left-4 right-4",
          height: height || "240px",
          initial: { opacity: 0, y: 20 },
          animate: { opacity: 1, y: 0 },
          exit: { opacity: 0, y: 20 },
        }
      default:
        return {
          container: "right-4 top-20 bottom-20",
          width: width,
          initial: { opacity: 0, x: 20 },
          animate: { opacity: 1, x: 0 },
          exit: { opacity: 0, x: 20 },
        }
    }
  }

  const styles = getPositionStyles()
  const isBottom = side === "bottom"

  return (
    <AnimatePresence>
      {open && (
        <>
          {/* Backdrop */}
          <motion.div
            className="fixed inset-0 z-40 bg-black/20 backdrop-blur-sm"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
          />

          {/* Floating Panel */}
          <motion.div
            className={cn(
              "fixed z-50 overflow-hidden rounded-xl border border-[var(--border-light)] bg-[var(--bg-secondary)] shadow-2xl shadow-black/50",
              styles.container,
              className
            )}
            style={isBottom ? { height: styles.height } : { width: styles.width }}
            initial={styles.initial}
            animate={styles.animate}
            exit={styles.exit}
            transition={{ type: "spring", damping: 25, stiffness: 300 }}
          >
            <div className="flex h-full flex-col">
              {/* Header */}
              {(title || showClose) && (
                <div className="flex items-center justify-between border-b border-[var(--border-light)] px-4 py-3">
                  <div>
                    {title && (
                      <h3 className="text-sm font-medium text-[var(--text-primary)]">
                        {title}
                      </h3>
                    )}
                    {description && (
                      <p className="mt-0.5 text-xs text-[var(--text-muted)]">
                        {description}
                      </p>
                    )}
                  </div>
                  {showClose && (
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6 rounded-md text-[var(--text-muted)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]"
                      onClick={onClose}
                    >
                      <X className="h-4 w-4" />
                    </Button>
                  )}
                </div>
              )}

              {/* Content */}
              <div className="flex-1 overflow-y-auto p-4">
                {children}
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  )
}
