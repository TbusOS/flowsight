"use client"

import * as React from "react"
import { Provider as JotaiProvider, useAtom } from "jotai"
import { MainLayout } from "./components/layout"
import { cn } from "./lib/utils"
import { initThemeAtom } from "./lib/atoms/layout-atoms"

// ============================================================================
// Theme Initializer Component
// ============================================================================

function ThemeInitializer() {
  const [, initTheme] = useAtom(initThemeAtom)

  React.useEffect(() => {
    initTheme()
  }, [initTheme])

  return null
}

// ============================================================================
// New FlowSight App - Minimal Design with shadcn/ui
// ============================================================================

export function App() {
  return (
    <JotaiProvider>
      <ThemeInitializer />
      {/* Skip Link for keyboard users */}
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-[100] focus:px-4 focus:py-2 focus:bg-[var(--accent)] focus:text-white focus:font-medium rounded-md"
      >
        跳转到主内容
      </a>
      <MainLayout />
    </JotaiProvider>
  )
}

export default App
