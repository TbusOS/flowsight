"use client"

import * as React from "react"
import { useAtom } from "jotai"
import { Check, ChevronDown } from "lucide-react"
import { cn } from "../../lib/utils"
import {
  themeAtom,
  themeInfoAtom,
  themesAtom,
  setThemeWithPersistenceAtom,
} from "../../lib/atoms/layout-atoms"

export function ThemeSelector() {
  const [theme] = useAtom(themeAtom)
  const [themeInfo] = useAtom(themeInfoAtom)
  const [themes] = useAtom(themesAtom)
  const [, setTheme] = useAtom(setThemeWithPersistenceAtom)
  const [isOpen, setIsOpen] = React.useState(false)

  const themeLabels: Record<string, { label: string; color: string }> = {
    dark: { label: "深色", color: "#18181b" },
    light: { label: "浅色", color: "#f5f5f7" },
    slate: { label: "Slate", color: "#64748b" },
    forest: { label: "Forest", color: "#10b981" },
    sunset: { label: "Sunset", color: "#d97706" },
    lavender: { label: "Lavender", color: "#8b5cf6" },
  }

  const handleThemeChange = (newTheme: string) => {
    setTheme(newTheme as any)
    setIsOpen(false)
  }

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        aria-label={`当前主题: ${themeInfo.label}，点击切换主题`}
        aria-expanded={isOpen}
        aria-haspopup="listbox"
        data-testid="theme-toggle"
        className={cn(
          "flex items-center gap-1.5 px-2.5 py-1.5 rounded-md",
          "text-xs font-medium text-[var(--text-secondary)]",
          "hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]",
          "transition-all duration-150"
        )}
      >
        <div
          className="w-4 h-4 rounded-full border border-[var(--border-light)]"
          style={{ backgroundColor: themeInfo.color }}
        />
        <span>{themeInfo.label}</span>
        <ChevronDown
          className={cn(
            "w-3 h-3 transition-transform duration-150",
            isOpen && "rotate-180"
          )}
        />
      </button>

      {isOpen && (
        <>
          <div
            className="fixed inset-0 z-40"
            onClick={() => setIsOpen(false)}
          />
          <div className="absolute right-0 top-full mt-1 z-50">
            <div
              role="listbox"
              aria-label="主题选择"
              data-testid="theme-menu"
              className={cn(
                "min-w-[160px] rounded-lg border border-[var(--border-light)]",
                "bg-[var(--bg-secondary)] shadow-xl py-1.5"
              )}
            >
              <div className="px-3 py-1.5 text-[10px] font-medium text-[var(--text-muted)] uppercase tracking-wider">
                选择主题
              </div>
              {themes.map((t) => (
                <button
                  key={t}
                  role="option"
                  aria-selected={theme === t}
                  data-testid={`theme-option-${t}`}
                  onClick={() => handleThemeChange(t)}
                  className={cn(
                    "w-full flex items-center gap-3 px-3 py-2",
                    "hover:bg-[var(--bg-tertiary)] transition-colors duration-100",
                    theme === t && "bg-[var(--bg-tertiary)]"
                  )}
                >
                  <div
                    className={cn(
                      "w-5 h-5 rounded-md border border-[var(--border-light)]",
                      "flex items-center justify-center"
                    )}
                    style={{ backgroundColor: themeLabels[t]?.color }}
                  >
                    {theme === t && (
                      <Check className="w-3 h-3 text-white mix-blend-difference" />
                    )}
                  </div>
                  <div className="flex flex-col items-start">
                    <span
                      className={cn(
                        "text-sm font-medium",
                        theme === t
                          ? "text-[var(--accent)]"
                          : "text-[var(--text-primary)]"
                      )}
                    >
                      {themeLabels[t]?.label}
                    </span>
                  </div>
                </button>
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  )
}
